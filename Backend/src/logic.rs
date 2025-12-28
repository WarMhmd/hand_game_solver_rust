use crate::{
    bot::{BotStrategy, DecideDrawResult},
    bots::better_meld_play::UseBetterMeldPlay,
    websocket::{
        send_player_draw_card_and_wait, send_player_draw_phase_and_wait,
        send_player_playing_phase_and_wait, send_players_discarded_card, send_players_sync_melds,
        send_round_start_and_wait, PlayingPhaseData, WsResponse,
    },
    AppState,
};
use axum::extract::ws::Message;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize, Serializer};
use std::{fmt, sync::Arc, time::Duration};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use uuid::Uuid;

// --------------------
// Types
// --------------------

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
    Joker,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Number(i32),
    Jack,
    Queen,
    King,
    Ace,
    Joker,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rank::Number(n) => write!(f, "{}", n),
            Rank::Jack => write!(f, "J"),
            Rank::Queen => write!(f, "Q"),
            Rank::King => write!(f, "K"),
            Rank::Ace => write!(f, "A"),
            Rank::Joker => write!(f, "Joker"),
        }
    }
}

pub fn rank_order(rank: Rank) -> Vec<i32> {
    match rank {
        Rank::Number(2) => vec![1],
        Rank::Number(3) => vec![2],
        Rank::Number(4) => vec![3],
        Rank::Number(5) => vec![4],
        Rank::Number(6) => vec![5],
        Rank::Number(7) => vec![6],
        Rank::Number(8) => vec![7],
        Rank::Number(9) => vec![8],
        Rank::Number(10) => vec![9],
        Rank::Jack => vec![10],
        Rank::Queen => vec![11],
        Rank::King => vec![12],
        Rank::Ace => vec![0, 13],
        Rank::Joker => vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        _ => vec![],
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq)]
pub enum MeldType {
    Rank,
    Sequence,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meld {
    pub id: String,
    pub cards: Vec<Card>,
    pub meld_type: MeldType,
}

#[derive(Debug, Default)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub bot_strategy: Option<Box<dyn BotStrategy>>,
    pub score: i32,
    pub did_join: bool,
    pub sender: Option<UnboundedSender<Message>>,
}

impl Clone for Player {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            bot_strategy: self.bot_strategy.clone(),
            score: self.score,
            did_join: self.did_join,
            sender: self.sender.clone(),
        }
    }
}

impl Serialize for Player {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("Player", 4)?;

        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("hasBotStrategy", &self.bot_strategy.is_some())?;
        s.serialize_field("score", &self.score)?;
        s.serialize_field("didJoin", &self.did_join)?;
        s.end()
    }
}

pub struct ActivePlayer {
    pub id: String,
    pub name: String,
    pub bot_strategy: Option<Box<dyn BotStrategy>>,
    pub score: i32,
    pub hand: Vec<Card>,
    pub fire_card_id: Option<String>,
    pub melded: bool,
    pub did_join: bool,
    pub sender: Option<UnboundedSender<Message>>,
}

// Convert Player to ActivePlayer helper
impl ActivePlayer {
    pub fn from_player(player: Player, hand: Vec<Card>) -> Self {
        Self {
            id: player.id,
            name: player.name,
            bot_strategy: player.bot_strategy,
            score: player.score,
            hand,
            fire_card_id: None,
            melded: false,
            did_join: player.did_join,
            sender: player.sender,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerResult {
    id: String,
    name: String,
    score: i32,
    wins: i32,
}

#[derive(Serialize, Clone)]
pub struct GameState {
    pub id: String,
    pub players: Vec<Player>,
    pub round: i32,
    pub max_rounds: i32,
}

impl GameState {
    pub fn new(players: Vec<Player>, max_rounds: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            players,
            round: 1,
            max_rounds,
        }
    }
}

pub async fn start_game(
    game_arc: Arc<Mutex<GameState>>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    // 1. SETUP: Lock briefly to get initial config (max_rounds, id) and a snapshot of players
    // We clone these so we can DROP the lock immediately.
    let (max_rounds, game_id, mut players_snapshot) = {
        let game = game_arc.lock().await;
        (game.max_rounds, game.id.clone(), game.players.clone())
    };

    // Initialize tracking results locally
    let mut results: Vec<PlayerResult> = players_snapshot
        .iter()
        .map(|p| PlayerResult {
            id: p.id.clone(),
            name: p.name.to_string(),
            score: 0,
            wins: 0,
        })
        .collect();

    println!("=== HAND GAME SIMULATION START ({}) ===", game_id);
    // --- MAIN GAME LOOP ---
    for r in 1..=max_rounds {
        println!("\n--- ROUND {} ---", r);

        // 2. INIT ROUND: Lock to generate the new round state
        let mut round_state = {
            let mut game = game_arc.lock().await;
            init_round(&mut *game)
        };

        let players_data: Vec<_> = round_state
            .players
            .iter()
            .map(|p| {
                (
                    p.id.clone(),
                    p.hand.clone(),
                    p.fire_card_id.clone(),
                    p.melded,
                )
            })
            .collect();

        // Re-construct refs from snapshot (or round_state) to pass to sender
        let player_refs: Vec<Player> = round_state
            .players
            .iter()
            .map(|ap| Player {
                id: ap.id.clone(),
                name: ap.name.clone(),
                bot_strategy: ap.bot_strategy.clone(),
                score: ap.score,
                did_join: ap.did_join,
                sender: ap.sender.clone(),
            })
            .collect();

        let scores: Vec<i32> = player_refs.clone().iter().map(|p| p.score).collect();

        match send_round_start_and_wait(
            &game_id,
            r,
            players_data,
            &player_refs,
            scores,
            state,
            std::time::Duration::from_mins(30),
        )
        .await
        {
            Ok(()) => println!("✅ All players acknowledged round {}", r),
            Err(e) => println!(
                "⚠️ Failed to get all acknowledgments for round {}: {}",
                r, e
            ),
        }
        let mut player_did_a_hand = false;

        // --- ROUND LOGIC LOOP ---
        while !is_round_over(&round_state) {
            let current_player_idx = round_state.current_player;

            // DRAW PHASE
            if round_state.phase == Phase::Draw {
                let draw_choice;

                let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                if let Some(ref mut s) = strategy {
                    draw_choice = s.decide_draw(&round_state);
                    wait_bot().await;
                } else {
                    match send_player_draw_phase_and_wait(
                        &round_state.players[current_player_idx],
                        &game_id,
                        r,
                        state,
                    )
                    .await
                    {
                        Ok(choice) => draw_choice = choice.draw_choice,
                        Err(e) => {
                            println!("⚠️ Failed to handle draw phase finished: {}", e);
                            draw_choice = DecideDrawResult::Deck;
                        }
                    }
                }
                round_state.players[current_player_idx].bot_strategy = strategy;

                let is_fire_card: bool;
                let card = match draw_choice {
                    DecideDrawResult::Fire => {
                        if !round_state.fire_pile.is_empty() {
                            is_fire_card = true;
                            draw_from_fire(&mut round_state)
                        } else {
                            is_fire_card = false;
                            draw_from_deck(&mut round_state)
                        }
                    }
                    DecideDrawResult::Deck => {
                        is_fire_card = false;
                        draw_from_deck(&mut round_state)
                    }
                };
                if card.is_err() {
                    // handle error
                } else {
                    round_state.phase = Phase::Meld;
                }

                match send_player_draw_card_and_wait(
                    &round_state.players[current_player_idx],
                    &game_id,
                    r,
                    card.clone().unwrap(),
                    is_fire_card,
                    round_state.fire_pile.is_empty(),
                    &round_state.players,
                    state,
                )
                .await
                {
                    Ok(_) => {
                        println!(
                            "Player {} received card",
                            round_state.players[current_player_idx].id
                        )
                    }
                    Err(err) => {
                        // handle error
                        println!("Error sending card: {}", err);
                    }
                }
            } else {
                let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                if let Some(ref mut s) = strategy {
                    wait_bot().await;
                    // MELD PHASE
                    if round_state.phase == Phase::Meld {
                        println!("Bot {} is in meld phase", s.name());
                        let melds = s.decide_melds(&round_state);

                        if !melds.is_empty() {
                            println!("Bot {} decided to meld", s.name());
                            let hand_count = round_state.players[current_player_idx].hand.len();
                            let has_fire_card = round_state.players[current_player_idx]
                                .fire_card_id
                                .is_some();
                            let result = lay_melds(&mut round_state, melds.clone());
                            let hand_count_after_meld =
                                round_state.players[current_player_idx].hand.len();
                            if hand_count == 15 && hand_count_after_meld == 1 && !has_fire_card {
                                player_did_a_hand = true;
                            }
                            if result.is_ok() {
                                match send_players_sync_melds(
                                    &game_id,
                                    r,
                                    &round_state.players[current_player_idx].id,
                                    &round_state.table_melds,
                                    melds.iter().map(|m| m.cards.clone()).flatten().collect(),
                                    round_state.players[current_player_idx].hand.len(),
                                    None,
                                    &round_state.players,
                                    state,
                                )
                                .await
                                {
                                    Ok(()) => println!("All melds sent successfully"),
                                    Err(err) => println!("Error sending melds: {}", err),
                                }
                                println!("==================");
                                println!(
                                    "Bot {}: melded with cards",
                                    round_state.players[current_player_idx].id
                                );
                                for (i, m) in melds.iter().enumerate() {
                                    println!("Meld {}:", i + 1);
                                    for c in &m.cards {
                                        println!("{} {}", c.rank, c.suit);
                                    }
                                }
                                println!("==================");
                                round_state.phase = Phase::PlayInMeld;
                            } else {
                                // print error
                                println!(
                                    "Bot {:?} failed to decide melds with error: {:?}",
                                    s.name(),
                                    result.err().unwrap()
                                );
                            }
                        } else {
                            round_state.phase = Phase::PlayInMeld;
                        }
                    }

                    // PLAY IN MELD PHASE
                    if round_state.phase == Phase::PlayInMeld {
                        println!("Bot {} is in play in meld phase", s.name());
                        let mut play_card: Option<Card> = None;
                        let mut play_index = -1;
                        let mut is_left = false;
                        let mut next_phase = Phase::Discard;

                        if s.can_use_features().contains(&"useMeld".to_string()) {
                            let res = s.decide_play_in_meld(&round_state);
                            next_phase = res.0;
                            play_card = res.1;
                            is_left = res.2;
                            play_index = res.3;
                        }

                        round_state.phase = next_phase;
                        if play_card.is_some() && play_index != -1 {
                            println!(
                                "Bot {} played card {} in meld",
                                s.name(),
                                play_card.clone().unwrap().id
                            );
                            let result = play_in_meld(
                                &mut round_state,
                                play_card.clone().unwrap(),
                                play_index as usize,
                                is_left,
                            );

                            if result.is_err() {
                                println!("Play in meld error: {}", result.err().unwrap());
                                round_state.phase = Phase::Discard;
                            } else {
                                match send_players_sync_melds(
                                    &game_id,
                                    r,
                                    &round_state.players[current_player_idx].id,
                                    &round_state.table_melds,
                                    vec![play_card.unwrap()],
                                    round_state.players[current_player_idx].hand.len(),
                                    result.unwrap(),
                                    &round_state.players,
                                    state,
                                )
                                .await
                                {
                                    Ok(()) => println!("All melds sent successfully"),
                                    Err(err) => println!("Error sending melds: {}", err),
                                }
                            }
                        }
                    }

                    // DISCARD PHASE
                    if round_state.phase == Phase::Discard {
                        println!("Bot {} is in discard phase", s.name());
                        let discard_idx = s.decide_discard(&round_state);
                        println!("Bot {} decided to discard card {}", s.name(), discard_idx);
                        let result = discard_card(&mut round_state, discard_idx);
                        if result.is_err() {
                            println!("Discard error: {}", result.err().unwrap());
                        } else {
                            match send_players_discarded_card(
                                &game_id,
                                r,
                                &round_state.players[current_player_idx].id,
                                &result.unwrap(),
                                round_state.players[current_player_idx].hand.len(),
                                &round_state.players,
                                state,
                            )
                            .await
                            {
                                Ok(()) => println!("Bot discarded card successfully"),
                                Err(err) => println!("Error sending melds: {}", err),
                            }
                        }
                        round_state.phase = Phase::Draw;
                    }
                } else {
                    match send_player_playing_phase_and_wait(
                        &round_state.players[current_player_idx],
                        &game_id,
                        r,
                        state,
                    )
                    .await
                    {
                        Ok(data) => match data {
                            PlayingPhaseData::PlayMeldPhase(data) => {
                                round_state.phase = Phase::Meld;
                                let melds = data.melds;
                                if !melds.is_empty() {
                                    let hand_count =
                                        round_state.players[current_player_idx].hand.len();

                                    let result = lay_melds(&mut round_state, melds.clone());
                                    let hand_count_after_meld =
                                        round_state.players[current_player_idx].hand.len();
                                    if hand_count == 15 && hand_count_after_meld == 1 {
                                        player_did_a_hand = true;
                                    }

                                    if result.is_ok() {
                                        match send_players_sync_melds(
                                            &game_id,
                                            r,
                                            &round_state.players[round_state.current_player].id,
                                            &round_state.table_melds,
                                            melds
                                                .iter()
                                                .map(|meld| meld.cards.clone())
                                                .flatten()
                                                .collect::<Vec<Card>>(),
                                            round_state.players[round_state.current_player]
                                                .hand
                                                .len(),
                                            None,
                                            &round_state.players,
                                            state,
                                        )
                                        .await
                                        {
                                            Ok(_) => {}
                                            Err(err) => {
                                                println!("Error sending melds: {}", err);
                                            }
                                        }
                                        println!("==================");
                                        println!(
                                            "Player {}: melded with cards",
                                            round_state.players[current_player_idx].id
                                        );
                                        for (i, m) in melds.iter().enumerate() {
                                            println!("Meld {}:", i + 1);
                                            for c in &m.cards {
                                                println!("{} {}", c.rank, c.suit);
                                            }
                                        }
                                        println!("==================");
                                    } else {
                                        // handle error later
                                    }
                                }
                            }
                            PlayingPhaseData::PlayInMeldPhase(data) => {
                                round_state.phase = Phase::PlayInMeld;

                                let play_card = data.card;
                                let play_index = round_state
                                    .table_melds
                                    .iter()
                                    .position(|meld| meld.id == data.meld_id);

                                if play_index.is_some() {
                                    let result = play_in_meld(
                                        &mut round_state,
                                        play_card.clone(),
                                        play_index.unwrap(),
                                        data.is_left,
                                    );
                                    if let Err(_err) = result {

                                        // handle error later
                                    } else {
                                        match send_players_sync_melds(
                                            &game_id,
                                            r,
                                            &round_state.players[round_state.current_player].id,
                                            &round_state.table_melds,
                                            vec![play_card],
                                            round_state.players[round_state.current_player]
                                                .hand
                                                .len(),
                                            result.unwrap(),
                                            &round_state.players,
                                            state,
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                println!("player in meld synced");
                                            }
                                            Err(err) => {
                                                println!("Error sending melds: {}", err);
                                            }
                                        }
                                    }
                                }
                            }
                            PlayingPhaseData::DiscardPhase(data) => {
                                round_state.phase = Phase::Discard;

                                let discard_idx = round_state.players[current_player_idx]
                                    .hand
                                    .iter()
                                    .position(|card| card.id == data.card.id);

                                if let Some(discard_idx) = discard_idx {
                                    let result = discard_card(&mut round_state, discard_idx);
                                    if result.is_err() {
                                        // Handle error later
                                        eprintln!("Error {}", result.err().unwrap());
                                    } else {
                                        match send_players_discarded_card(
                                            &game_id,
                                            r,
                                            &round_state.players[current_player_idx].id,
                                            &result.unwrap(),
                                            round_state.players[current_player_idx].hand.len(),
                                            &round_state.players,
                                            state,
                                        )
                                        .await
                                        {
                                            Ok(()) => {
                                                println!("Player discarded card successfully")
                                            }
                                            Err(err) => println!("Error sending melds: {}", err),
                                        }
                                        round_state.phase = Phase::Draw;
                                    }
                                } else {
                                    // handle error later
                                }
                            }
                        },
                        Err(e) => {
                            println!("⚠️ Failed to handle draw phase finished: {}", e);
                        }
                    }
                }
                round_state.players[current_player_idx].bot_strategy = strategy;
            }
        }

        // --- ROUND END ---
        // Calculate winner locally
        let winner = round_state
            .players
            .iter()
            .find(|p| p.hand.is_empty())
            .unwrap();
        let winner_name = winner.name.clone();
        let winner_id = winner.id.clone();
        println!("Round {} winner: {}", r, winner_name);

        // 4. SCORE UPDATE: Lock Global State
        {
            let mut game = game_arc.lock().await;

            // Pass mutable reference to the locked game
            score_round(&mut *game, &mut round_state, player_did_a_hand);

            // Update results based on the now-updated game state
            for p in &game.players {
                if let Some(bot) = results.iter_mut().find(|b| b.name == p.name) {
                    bot.score = p.score;
                    if p.id == winner_id {
                        bot.wins += 1;
                    }
                }
                println!("{} | Score: {}", p.name, p.score);
            }

            // Update our snapshot for the next round
            players_snapshot = game.players.clone();
        }
    }

    // --- FINAL RESULTS ---
    println!("\n=== FINAL RESULTS ===");
    results.sort_by(|a, b| a.score.cmp(&b.score));

    for player in &players_snapshot {
        if let Some(sender) = player.sender.clone() {
            let message = WsResponse::GameOver {
                players: results.clone(),
            };
            if let Ok(json) = serde_json::to_string(&message) {
                let _ = sender.send(Message::Text(json.into()));
            }
        }
    }

    for (i, r) in results.iter().enumerate() {
        println!(
            "{}. {} | Total: {} | Wins: {}",
            i + 1,
            r.name,
            r.score,
            r.wins
        );
    }

    if !results.is_empty() {
        println!("\n🏆 WINNER: {}", results[0].name);
    }

    Ok(())
}

async fn wait_bot() {
    tokio::time::sleep(Duration::from_millis(500)).await;
}

pub struct RoundState {
    pub players: Vec<ActivePlayer>,
    pub current_player: usize,
    pub deck: Vec<Card>,
    pub fire_pile: Vec<Card>,
    pub table_melds: Vec<Meld>,
    pub phase: Phase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Draw,
    Meld,
    PlayInMeld,
    Discard,
}

// --------------------
// Deck creation (2 decks + 2 jokers = 106 cards)
// --------------------

pub fn create_deck() -> Vec<Card> {
    let suits = vec![Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
    let ranks = vec![
        Rank::Number(2),
        Rank::Number(3),
        Rank::Number(4),
        Rank::Number(5),
        Rank::Number(6),
        Rank::Number(7),
        Rank::Number(8),
        Rank::Number(9),
        Rank::Number(10),
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    let mut deck = Vec::new();

    for d in 0..2 {
        for &suit in &suits {
            for &rank in &ranks {
                deck.push(Card {
                    id: format!("{}-{}-{}", suit, rank, d),
                    suit,
                    rank,
                });
            }
        }
        deck.push(Card {
            id: format!("Joker-{}", d),
            suit: Suit::Joker,
            rank: Rank::Joker,
        });
    }

    deck
}

pub fn shuffle(deck: Vec<Card>) -> Vec<Card> {
    let mut d = deck.clone();
    let mut rng = thread_rng();
    d.shuffle(&mut rng);
    d
}

// --------------------
// Game setup
// --------------------

pub fn init_round(game_state: &mut GameState) -> RoundState {
    let mut deck = shuffle(create_deck());
    let mut active_players = Vec::new();

    // Drain players from GameState to create ActivePlayers
    let players_drain: Vec<Player> = game_state.players.drain(..).collect();

    for (i, player) in players_drain.into_iter().enumerate() {
        let count = if i == 0 { 15 } else { 14 };
        let hand: Vec<Card> = deck.drain(0..count).collect();
        active_players.push(ActivePlayer::from_player(player, hand));
    }

    RoundState {
        current_player: 0,
        players: active_players,
        deck,
        fire_pile: Vec::new(),
        table_melds: Vec::new(),
        phase: Phase::Meld,
    }
}

// --------------------
// Helpers
// --------------------

pub fn rank_value_int(rank: Rank) -> i32 {
    match rank {
        Rank::Ace => 11,
        Rank::Joker => 0,
        Rank::Number(n) => n,
        _ => 10,
    }
}

pub fn card_value(card: &Card) -> i32 {
    rank_value_int(card.rank)
}

pub fn card_penality(card: &Card) -> i32 {
    match card.rank {
        Rank::Ace => 11,
        Rank::Joker => 15,
        Rank::Number(n) => n,
        _ => 10,
    }
}

pub fn meld_value(meld: &Meld) -> i32 {
    let mut val = meld.cards.iter().map(|c| card_value(c)).sum::<i32>();
    let joker_card = meld.cards.iter().find(|c| c.rank == Rank::Joker);

    if let Some(joker) = joker_card {
        if meld.meld_type == MeldType::Rank {
            let first_card = meld.cards.iter().find(|c| c.rank != Rank::Joker).unwrap();
            val += card_value(first_card);
        } else if meld.meld_type == MeldType::Sequence {
            let card_index = meld.cards.iter().position(|c| c == joker).unwrap();
            let prev_card = if card_index > 0 {
                meld.cards.get(card_index - 1)
            } else {
                None
            };
            let next_card = meld.cards.get(card_index + 1);

            if let Some(prev) = prev_card {
                val += if prev.rank == Rank::King {
                    11 // Joker is A
                } else if card_value(prev) == 10 {
                    10 // Joker is J, Q, K
                } else {
                    card_value(prev) + 1 // Joker is 3-10
                };
            } else if let Some(next) = next_card {
                val += if next.rank == Rank::Number(2) {
                    11 // Joker is A
                } else if card_value(next) == 10 && next.rank != Rank::Number(10) {
                    10
                } else {
                    card_value(next) - 1
                };
            }
        }
    }
    val
}

pub fn melds_value(melds: &Vec<Meld>) -> i32 {
    melds.iter().map(|m| meld_value(m)).sum()
}

pub fn valid_sequence_two_cards(
    left: &Card,
    right: &Card,
    is_first_card: bool,
    is_last_card: bool,
) -> bool {
    if left.rank == Rank::Joker || right.rank == Rank::Joker {
        return true;
    }
    if is_first_card && left.rank == Rank::Ace && right.rank == Rank::Number(2) {
        return true;
    }
    if left.rank == Rank::Number(10) && right.rank == Rank::Jack {
        return true;
    }
    if left.rank == Rank::Jack && right.rank == Rank::Queen {
        return true;
    }
    if left.rank == Rank::Queen && right.rank == Rank::King {
        return true;
    }
    if is_last_card && left.rank == Rank::King && right.rank == Rank::Ace {
        return true;
    }

    match (left.rank, right.rank) {
        (Rank::Number(l), Rank::Number(r)) => l + 1 == r,
        _ => false,
    }
}

pub fn check_seq_melds(meld: &Meld) -> Vec<Meld> {
    if !valid_sequence_meld(&meld.cards) {
        return vec![];
    }
    if meld.cards.len() >= 6 {
        let mut melds = Vec::new();
        for i in (0..meld.cards.len()).step_by(3) {
            if i + 3 <= meld.cards.len() && i + 6 <= meld.cards.len() {
                melds.push(Meld {
                    id: Uuid::new_v4().to_string(),
                    cards: meld.cards[i..i + 3].to_vec(),
                    meld_type: MeldType::Sequence,
                });
            } else {
                melds.push(Meld {
                    id: Uuid::new_v4().to_string(),
                    cards: meld.cards[i..].to_vec(),
                    meld_type: MeldType::Sequence,
                });
                break;
            }
        }
    } else {
        return vec![meld.clone()];
    }
    return vec![];
}

pub fn check_melds(state: &mut RoundState) {
    let mut remove_index: Vec<usize> = Vec::new();
    let mut added_melds: Vec<Meld> = Vec::new();

    for (index, meld) in state.table_melds.iter().enumerate() {
        if valid_rank_meld(&meld.cards) {
            if meld.cards.len() == 4 && meld.cards.iter().all(|c| c.rank != Rank::Joker) {
                for c in &meld.cards {
                    state.fire_pile.insert(state.fire_pile.len() / 2, c.clone());
                }
                remove_index.push(index);
            }
        }

        if valid_sequence_meld(&meld.cards) {
            if meld.cards.len() >= 6 {
                remove_index.push(index);
                for i in (0..meld.cards.len()).step_by(3) {
                    if i + 3 <= meld.cards.len() && i + 6 <= meld.cards.len() {
                        added_melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: meld.cards[i..i + 3].to_vec(),
                            meld_type: MeldType::Sequence,
                        });
                    } else {
                        added_melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: meld.cards[i..].to_vec(),
                            meld_type: MeldType::Sequence,
                        });
                        break;
                    }
                }
            }
        }
    }

    // Sort remove_index descending to avoid index shifting
    remove_index.sort_by(|a, b| b.cmp(a));
    for index in remove_index {
        state.table_melds.remove(index);
    }
    state.table_melds.extend(added_melds);
}

// --------------------
// Meld validation
// --------------------

pub fn valid_rank_meld(meld_cards: &Vec<Card>) -> bool {
    if meld_cards.len() < 3 || meld_cards.len() > 4 {
        return false;
    }
    if meld_cards.iter().filter(|c| c.rank == Rank::Joker).count() > 1 {
        return false;
    }

    let cards: Vec<&Card> = meld_cards
        .iter()
        .filter(|c| c.rank != Rank::Joker)
        .collect();
    if cards.is_empty() {
        return false;
    }

    let same_rank = cards.iter().all(|c| c.rank == cards[0].rank);
    let mut suits = std::collections::HashSet::new();
    for c in &cards {
        suits.insert(c.suit);
    }
    let different_suit = cards.len() == suits.len();

    same_rank && different_suit
}

pub fn valid_sequence_meld(cards: &Vec<Card>) -> bool {
    if cards.len() < 3 {
        return false;
    }
    if cards.iter().filter(|c| c.rank == Rank::Joker).count() > 1 {
        return false;
    }

    let first_card = cards.iter().find(|c| c.rank != Rank::Joker).unwrap();
    let same_suit = cards
        .iter()
        .all(|c| c.rank == Rank::Joker || c.suit == first_card.suit);

    if !same_suit {
        return false;
    }

    for i in 1..cards.len() {
        if !valid_sequence_two_cards(&cards[i - 1], &cards[i], i == 1, i == cards.len() - 1) {
            return false;
        }
        if i == 1 && cards[i - 1].rank == Rank::Joker && cards[i].rank == Rank::Ace {
            return false;
        }
        if i == cards.len() - 1 && cards[i].rank == Rank::Joker && cards[i - 1].rank == Rank::Ace {
            return false;
        }
        if cards[i].rank == Rank::Joker {
            if i == cards.len() - 1 {
                continue;
            }
            let next_card = &cards[i + 1];
            if *rank_order(next_card.rank).last().unwrap() != rank_order(cards[i - 1].rank)[0] + 2 {
                return false;
            }
        }
    }
    true
}

// first boolean for success
// second boolean for can take joker card or no
pub fn can_play_in_rank_meld(meld: &Meld, card: &Card) -> (bool, bool) {
    let joker_card = meld.cards.iter().find(|c| c.rank == Rank::Joker);

    if joker_card.is_some() && card.rank == Rank::Joker {
        return (false, false);
    }

    let mut copy_meld = meld.cards.clone();
    let mut take_joker = false;

    if joker_card.is_some() && copy_meld.len() == 4 {
        copy_meld = copy_meld
            .into_iter()
            .map(|c| {
                if c.rank == Rank::Joker {
                    card.clone()
                } else {
                    c
                }
            })
            .collect();
        if !valid_rank_meld(&copy_meld) {
            return (false, false);
        } else {
            take_joker = true;
        }
    } else {
        copy_meld.push(card.clone());
        if !valid_rank_meld(&copy_meld) {
            return (false, false);
        }
    }

    (true, take_joker)
}

pub fn can_play_in_sequence_meld(meld: &Meld, card: &Card, play_end: bool) -> (bool, bool, bool) {
    let joker_card = meld.cards.iter().find(|c| c.rank == Rank::Joker);

    if joker_card.is_some() && card.rank == Rank::Joker {
        return (false, false, false);
    }

    if joker_card.is_some() {
        let mut copy_meld = meld.cards.clone();
        copy_meld = copy_meld
            .into_iter()
            .map(|c| {
                if c.rank == Rank::Joker {
                    card.clone()
                } else {
                    c
                }
            })
            .collect();
        if valid_sequence_meld(&copy_meld) {
            return (true, false, true);
        }
    }

    if card.rank != Rank::Joker {
        let mut copy_meld = meld.cards.clone();
        copy_meld.insert(0, card.clone());
        if valid_sequence_meld(&copy_meld) {
            return (true, true, false);
        }

        copy_meld = meld.cards.clone();
        copy_meld.push(card.clone());
        if valid_sequence_meld(&copy_meld) {
            return (true, false, false);
        }
    } else {
        if !play_end {
            let mut copy_meld = meld.cards.clone();
            copy_meld.insert(0, card.clone());
            if valid_sequence_meld(&copy_meld) {
                return (true, true, false);
            }
        } else {
            let mut copy_meld = meld.cards.clone();
            copy_meld.push(card.clone());
            if valid_sequence_meld(&copy_meld) {
                return (true, false, false);
            }
        }
    }

    (false, false, false)
}

pub fn is_valid_set(meld: &Meld) -> bool {
    if meld.meld_type == MeldType::Rank && valid_rank_meld(&meld.cards) {
        return true;
    }
    if meld.meld_type == MeldType::Sequence && valid_sequence_meld(&meld.cards) {
        return true;
    }
    false
}

// --------------------
// Turn actions
// --------------------

pub fn draw_from_deck(state: &mut RoundState) -> Result<Card, String> {
    if state.phase != Phase::Draw {
        // panic!("Not draw phase");
        return Err("Not draw phase".to_string());
    }

    if state.deck.is_empty() {
        state.deck = shuffle(state.fire_pile.clone());
        state.fire_pile.clear();
    }

    let card = state.deck.remove(0);

    state.players[state.current_player].hand.push(card.clone());
    state.players[state.current_player].fire_card_id = None;
    Ok(card)
}

pub fn draw_from_fire(state: &mut RoundState) -> Result<Card, String> {
    if state.phase != Phase::Draw {
        // panic!("Not draw phase");
        return Err("Not draw phase".to_string());
    }
    if let Some(card) = state.fire_pile.pop() {
        state.players[state.current_player].hand.push(card.clone());
        state.players[state.current_player].fire_card_id = Some(card.id.clone());
        // println!(
        //     "Player drawn from fire pile: {:?}",
        //     state.players[state.current_player].hand.clone()
        // );
        return Ok(card);
    } else {
        // panic!("Fire pile empty");
        return Err("Fire pile empty".to_string());
    }
}

pub fn discard_fire_card(state: &mut RoundState) {
    let player = &mut state.players[state.current_player];
    if player.fire_card_id.is_none() {
        panic!("No fire card to discard");
    }

    let fire_id = player.fire_card_id.as_ref().unwrap().clone();
    let card_idx = player.hand.iter().position(|c| c.id == fire_id);

    if let Some(idx) = card_idx {
        let card = player.hand.remove(idx);
        state.fire_pile.push(card);
        state.players[state.current_player].fire_card_id = None;
    } else {
        panic!("Fire card not in hand");
    }
}

pub fn lay_melds(state: &mut RoundState, melds: Vec<Meld>) -> Result<(), String> {
    if melds.is_empty() {
        // panic!("No melds to lay");
        return Err("No melds to lay".to_string());
    }
    if !melds.iter().all(|m| is_valid_set(m)) {
        // print all melds
        for m in &melds {
            println!("Meld Type: {:?}", m.meld_type);
            for c in &m.cards {
                println!("{:?}", c);
            }
            println!("==========================");
        }
        // panic!("Invalid meld");
        return Err("Invalid meld".to_string());
    }

    let player_idx = state.current_player;
    // We must borrow player immutably first to check checks
    {
        let player = &state.players[player_idx];
        if !player.melded {
            let score = melds_value(&melds);
            if score < 51 {
                // panic!("Total meld score must be >= 51");
                return Err("Total meld score must be >= 51".to_string());
            }
        }

        if let Some(fire_id) = &player.fire_card_id {
            if !melds
                .iter()
                .any(|m| m.cards.iter().any(|c| c.id == *fire_id))
            {
                // print fire card
                println!("Fire Card: {:?}", fire_id);
                for m in &melds {
                    println!("Meld Type: {:?}", m.meld_type);
                    for c in &m.cards {
                        println!("{:?}", c);
                    }
                    println!("==========================");
                }
                return Err("Fire card not used in meld".to_string());
            }
        }
    }

    // Mutate player
    let player = &mut state.players[player_idx];
    for m in &melds {
        for c in &m.cards {
            if let Some(idx) = player.hand.iter().position(|h| h.id == c.id) {
                player.hand.remove(idx);
            } else {
                // print player hand
                for card in &player.hand {
                    println!("{:?}", card);
                }
                // print card
                println!("card: {:?}", c);
                return Err("Card not in hand".to_string());
            }
        }
    }
    player.melded = true;
    if player.fire_card_id.is_some() {
        println!("Used a fire card in lay melds");
    }

    player.fire_card_id = None;

    state.table_melds.extend(melds);
    check_melds(state);
    Ok(())
}

pub fn play_in_meld(
    state: &mut RoundState,
    card: Card,
    meld_index: usize,
    is_left: bool,
) -> Result<Option<Card>, String> {
    let player_idx = state.current_player;
    let mut result = Ok(None);
    // Checks
    if meld_index >= state.table_melds.len() {
        // panic!("Meld not found");
        return Err("Meld not found".to_string());
    }
    let meld_type = state.table_melds[meld_index].meld_type.clone();

    {
        let player = &state.players[player_idx];
        if !player.hand.iter().any(|c| c.id == card.id) {
            // panic!("Card not in hand");
            return Err(format!("Card not in hand {}", card.id));
        }
        if player.hand.len() == 1 {
            // panic!("Cannot play last card in hand");
            return Err("Cannot play last card in hand".to_string());
        }
    }

    // Logic
    if meld_type == MeldType::Rank {
        let meld = &state.table_melds[meld_index];
        let (success, take_joker) = can_play_in_rank_meld(meld, &card);
        if !success {
            return Err("Cannot play card in rank meld".to_string());
        }

        let mut joker_to_return: Option<Card> = None;
        if take_joker {
            let meld_mut = &mut state.table_melds[meld_index];
            let joker_idx = meld_mut
                .cards
                .iter()
                .position(|c| c.rank == Rank::Joker)
                .unwrap();
            joker_to_return = Some(meld_mut.cards[joker_idx].clone());
            meld_mut.cards[joker_idx] = card.clone();
        } else {
            state.table_melds[meld_index].cards.push(card.clone());
        }

        let player = &mut state.players[player_idx];
        result = Ok(joker_to_return.clone());
        if let Some(joker) = joker_to_return {
            player.hand.push(joker);
        }

        if let Some(fire_card_id) = &player.fire_card_id {
            if *fire_card_id == card.id {
                player.fire_card_id = None;
            }
        }
        let idx = player.hand.iter().position(|c| c.id == card.id).unwrap();
        player.hand.remove(idx);
    }

    if meld_type == MeldType::Sequence {
        let meld = &state.table_melds[meld_index];
        let (success, is_left, take_joker) = can_play_in_sequence_meld(meld, &card, !is_left);
        if !success {
            return Err("Cannot play card in sequence meld".to_string());
        }

        let mut joker_to_return: Option<Card> = None;
        if take_joker {
            let meld_mut = &mut state.table_melds[meld_index];
            let joker_idx = meld_mut
                .cards
                .iter()
                .position(|c| c.rank == Rank::Joker)
                .unwrap();
            joker_to_return = Some(meld_mut.cards[joker_idx].clone());
            meld_mut.cards[joker_idx] = card.clone();
        } else {
            if is_left {
                state.table_melds[meld_index].cards.insert(0, card.clone());
            } else {
                state.table_melds[meld_index].cards.push(card.clone());
            }
        }
        result = Ok(joker_to_return.clone());
        let player = &mut state.players[player_idx];
        if let Some(joker) = joker_to_return {
            player.hand.push(joker);
        }

        if let Some(fire_card_id) = &player.fire_card_id {
            if *fire_card_id == card.id {
                player.fire_card_id = None;
            }
        }
        let idx = player.hand.iter().position(|c| c.id == card.id).unwrap();
        player.hand.remove(idx);
    }

    check_melds(state);
    result
}

pub fn discard_card(state: &mut RoundState, card_index: usize) -> Result<Card, String> {
    if state.phase != Phase::Discard {
        return Err("Not discard phase".to_string());
    }

    let player = &mut state.players[state.current_player];
    if player.fire_card_id.is_some() {
        return Err(format!(
            "Player has a fire card {}",
            player.fire_card_id.clone().unwrap()
        ));
    }
    let card = player.hand.remove(card_index);
    let card_clone = card.clone();
    state.fire_pile.push(card);

    state.current_player = (state.current_player + 1) % state.players.len();
    Ok(card_clone)
}

// --------------------
// Round & scoring
// --------------------

pub fn is_round_over(state: &RoundState) -> bool {
    state.players.iter().any(|p| p.hand.is_empty())
}

pub fn score_round(
    game_state: &mut GameState,
    round_state: &mut RoundState,
    player_did_a_hand: bool,
) {
    let winner_id = round_state
        .players
        .iter()
        .find(|p| p.hand.is_empty())
        .unwrap()
        .id
        .clone();
    let multiplier = if player_did_a_hand { 2 } else { 1 };
    for rp in &mut round_state.players {
        if rp.id != winner_id {
            if !rp.melded {
                rp.score += 100 * multiplier;
            } else {
                let hand_score: i32 = rp.hand.iter().map(|c| card_penality(c)).sum();
                rp.score += hand_score * multiplier;
            }
        } else {
            rp.score -= 30 * multiplier;
        }
    }

    game_state.round += 1;

    let mut players_back = Vec::new();
    // drain gives us ownership of ActivePlayer (rp)
    for rp in round_state.players.drain(..) {
        players_back.push(Player {
            id: rp.id,
            name: rp.name,
            bot_strategy: rp.bot_strategy, // Move the strategy back
            score: rp.score,               // Use the updated score from the ActivePlayer
            did_join: rp.did_join,
            sender: rp.sender,
        });
    }

    // Sort to ensure order P1..P4 (optional but good for consistency)
    // players_back.sort_by_key(|p| p.id.clone());
    game_state.players = players_back;
}

// Include test module
#[cfg(test)]
#[path = "logic_test.rs"]
mod logic_test;
