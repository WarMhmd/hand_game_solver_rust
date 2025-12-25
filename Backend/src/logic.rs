use crate::bot::{BotStrategy, DecideDrawResult};
use crate::BotResult;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

// --------------------
// Types
// --------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeldType {
    Rank,
    Sequence,
}

#[derive(Debug, Clone)]
pub struct Meld {
    pub cards: Vec<Card>,
    pub meld_type: MeldType,
}

pub struct Player {
    pub id: String,
    pub name: String,
    pub bot_strategy: Option<Box<dyn BotStrategy>>,
    pub score: i32,
}

pub struct ActivePlayer {
    pub id: String,
    pub name: String,
    pub bot_strategy: Option<Box<dyn BotStrategy>>,
    pub score: i32,
    pub hand: Vec<Card>,
    pub fire_card_id: Option<String>,
    pub melded: bool,
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
        }
    }
}

#[derive(Clone)]
struct BotResult {
    name: String,
    total_score: i32,
    rounds_won: i32,
}

pub struct GameState {
    pub players: Vec<Player>,
    pub bots_startegies: Vec<Option<Box<dyn BotStrategy>>>,
    pub round: i32,
    pub max_rounds: i32,
}

impl GameState {
    pub fn new(
        players: Vec<Player>,
        bots_startegies: Vec<Option<Box<dyn BotStrategy>>>,
        max_rounds: i32,
    ) -> Self {
        Self {
            players,
            bots_startegies,
            round: 1,
            max_rounds,
        }
    }

    pub fn start_game(&mut self) {
        let mut results: Vec<BotResult> = self
            .players
            .iter()
            .map(|p| BotResult {
                name: p.name.to_string(),
                total_score: 0,
                rounds_won: 0,
            })
            .collect();

        println!("=== HAND GAME SIMULATION START ===");

        let player_names: Vec<String> = self.players.iter().map(|p| p.name.to_string()).collect();

        for r in 1..=self.max_rounds {
            let mut round_state = init_round(self);
            println!("\n--- ROUND {} ---", r);
            let mut round_break = 0;

            while !is_round_over(&round_state) {
                // print all player cards (debug)
                if round_state.current_player == 3 {
                    for p in &round_state.players {
                        if p.id == "P4" {
                            println!("Player {}:", p.id);
                            for c in &p.hand {
                                println!("{} {}", c.rank, c.suit);
                            }
                        }
                    }
                    println!("----------------");
                }

                round_break += 1;
                if round_break > 10000 {
                    println!("Round timed out");
                    break;
                }

                let current_player_idx = round_state.current_player;
                let mut draw_choice = DecideDrawResult::Deck;

                // Borrow strategy mutably
                let mut strategy = round_state.players[current_player_idx].bot_strategy.take();

                if let Some(ref mut s) = strategy {
                    if round_state.phase == Phase::Draw {
                        draw_choice = s.decide_draw(&round_state);
                    }
                }

                round_state.players[current_player_idx].bot_strategy = strategy;

                // DRAW PHASE
                if round_state.phase == Phase::Draw {
                    match draw_choice {
                        DecideDrawResult::Fire => {
                            if !round_state.fire_pile.is_empty() {
                                draw_from_fire(&mut round_state);
                            } else {
                                draw_from_deck(&mut round_state);
                            }
                        }
                        DecideDrawResult::Deck => draw_from_deck(&mut round_state),
                    }
                    round_state.phase = Phase::Meld;
                }

                // MELD PHASE
                if round_state.phase == Phase::Meld {
                    let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                    let mut melds = Vec::new();

                    if let Some(ref mut s) = strategy {
                        melds = s.decide_melds(&round_state);
                    }
                    round_state.players[current_player_idx].bot_strategy = strategy;

                    if !melds.is_empty() {
                        // lay_melds modifies round_state. Assume valid.
                        lay_melds(&mut round_state, melds.clone());

                        println!("==================");
                        println!(
                            "player {}: melded with cards",
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
                        round_state.phase = Phase::PlayInMeld;
                    }
                }

                // PLAY IN MELD PHASE
                if round_state.phase == Phase::PlayInMeld {
                    let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                    let mut play_card = None;
                    let mut play_index = -1;

                    if let Some(ref mut s) = strategy {
                        if s.can_use_features().contains(&"useMeld".to_string()) {
                            let res = s.decide_play_in_meld(&round_state);
                            play_card = res.0;
                            play_index = res.1;
                        }
                    }
                    round_state.players[current_player_idx].bot_strategy = strategy;

                    if play_card.is_none() || play_index == -1 {
                        round_state.phase = Phase::Discard;
                    } else {
                        play_in_meld(&mut round_state, play_card.unwrap(), play_index as usize);
                    }
                }

                // DISCARD PHASE
                if round_state.phase == Phase::Discard {
                    let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                    let mut discard_idx = 0;

                    if let Some(ref mut s) = strategy {
                        discard_idx = s.decide_discard(&round_state);
                    }
                    round_state.players[current_player_idx].bot_strategy = strategy;

                    discard_card(&mut round_state, discard_idx);
                    round_state.phase = Phase::Draw;
                }
            }

            if round_break <= 10000 {
                // print rounds used
                println!("Rounds used: {}", round_break);
                let winner = round_state
                    .players
                    .iter()
                    .find(|p| p.hand.is_empty())
                    .unwrap();
                let winner_name = winner.name.clone();
                let winner_id = winner.id.clone();
                println!("Round {} winner: {}", r, winner_name);

                score_round(self, &mut round_state);

                // Update local results
                for p in &self.players {
                    if let Some(bot) = results.iter_mut().find(|b| b.name == p.name) {
                        bot.total_score = p.score;
                        if p.id == winner_id {
                            bot.rounds_won += 1;
                        }
                    }
                }

                // Print summary
                for p in &self.players {
                    println!("{} | Score: {}", p.name, p.score);
                }
            } else {
                // Restore players if round timed out
                let mut players_back = Vec::new();
                for rp in round_state.players.drain(..) {
                    players_back.push(Player {
                        id: rp.id,
                        name: rp.name,
                        bot_strategy: rp.bot_strategy,
                        score: rp.score,
                    });
                }
                players_back.sort_by_key(|p| p.id.clone());
                self.players = players_back;
            }
        }

        println!("\n=== FINAL RESULTS ===");
        results.sort_by(|a, b| a.total_score.cmp(&b.total_score));

        for (i, r) in results.iter().enumerate() {
            println!(
                "{}. {} | Total Score: {} | Rounds Won: {}",
                i + 1,
                r.name,
                r.total_score,
                r.rounds_won
            );
        }

        if !results.is_empty() {
            println!("\n🏆 WINNER: {}", results[0].name);
        }
    }
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

pub fn valid_sequence_two_cards(left: &Card, right: &Card) -> bool {
    if left.rank == Rank::Joker || right.rank == Rank::Joker {
        return true;
    }
    if left.rank == Rank::Ace && right.rank == Rank::Number(2) {
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
    if left.rank == Rank::King && right.rank == Rank::Ace {
        return true;
    }

    match (left.rank, right.rank) {
        (Rank::Number(l), Rank::Number(r)) => l + 1 == r,
        _ => false,
    }
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
                            cards: meld.cards[i..i + 3].to_vec(),
                            meld_type: MeldType::Sequence,
                        });
                    } else {
                        added_melds.push(Meld {
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
        if !valid_sequence_two_cards(&cards[i - 1], &cards[i]) {
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

pub fn can_play_in_sequence_meld(meld: &Meld, card: &Card, play_end: bool) -> (bool, bool) {
    let joker_card = meld.cards.iter().find(|c| c.rank == Rank::Joker);

    if joker_card.is_some() && card.rank == Rank::Joker {
        return (false, false);
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
            return (true, true);
        }
    }

    if card.rank != Rank::Joker {
        let mut copy_meld = meld.cards.clone();
        copy_meld.insert(0, card.clone());
        if valid_sequence_meld(&copy_meld) {
            return (true, false);
        }

        copy_meld = meld.cards.clone();
        copy_meld.push(card.clone());
        if valid_sequence_meld(&copy_meld) {
            return (true, false);
        }
    } else {
        if !play_end {
            let mut copy_meld = meld.cards.clone();
            copy_meld.insert(0, card.clone());
            if valid_sequence_meld(&copy_meld) {
                return (true, false);
            }
        } else {
            let mut copy_meld = meld.cards.clone();
            copy_meld.push(card.clone());
            if valid_sequence_meld(&copy_meld) {
                return (true, false);
            }
        }
    }

    (false, false)
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

pub fn draw_from_deck(state: &mut RoundState) {
    if state.phase != Phase::Draw {
        panic!("Not draw phase");
    }

    if state.deck.is_empty() {
        state.deck = shuffle(state.fire_pile.clone());
        state.fire_pile.clear();
    }

    let card = state.deck.remove(0);

    state.players[state.current_player].hand.push(card);
    state.players[state.current_player].fire_card_id = None;
}

pub fn draw_from_fire(state: &mut RoundState) {
    if state.phase != Phase::Draw {
        panic!("Not draw phase");
    }
    if let Some(card) = state.fire_pile.pop() {
        state.players[state.current_player].hand.push(card.clone());
        state.players[state.current_player].fire_card_id = Some(card.id);
    } else {
        panic!("Fire pile empty");
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

pub fn lay_melds(state: &mut RoundState, melds: Vec<Meld>) {
    if melds.is_empty() {
        panic!("No melds to lay");
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
        panic!("Invalid meld");
    }

    let player_idx = state.current_player;
    // We must borrow player immutably first to check checks
    {
        let player = &state.players[player_idx];
        if !player.melded {
            let score = melds_value(&melds);
            if score < 51 {
                panic!("Total meld score must be >= 51");
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
                panic!("Fire card not used in meld");
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
                panic!("Card not in hand");
            }
        }
    }
    player.melded = true;

    state.table_melds.extend(melds);
    check_melds(state);
}

pub fn play_in_meld(state: &mut RoundState, card: Card, meld_index: usize) {
    let player_idx = state.current_player;

    // Checks
    if meld_index >= state.table_melds.len() {
        panic!("Meld not found");
    }
    let meld_type = state.table_melds[meld_index].meld_type.clone();

    {
        let player = &state.players[player_idx];
        if !player.hand.iter().any(|c| c.id == card.id) {
            panic!("Card not in hand");
        }
        if player.hand.len() == 1 {
            panic!("Cannot play last card in hand");
        }
    }

    // Logic
    if meld_type == MeldType::Rank {
        let meld = &state.table_melds[meld_index];
        let (success, take_joker) = can_play_in_rank_meld(meld, &card);
        if !success {
            return;
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
        if let Some(joker) = joker_to_return {
            player.hand.push(joker);
        }
        let idx = player.hand.iter().position(|c| c.id == card.id).unwrap();
        player.hand.remove(idx);
    }

    if meld_type == MeldType::Sequence {
        let meld = &state.table_melds[meld_index];
        let (success, take_joker) = can_play_in_sequence_meld(meld, &card, false);
        if !success {
            return;
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
        if let Some(joker) = joker_to_return {
            player.hand.push(joker);
        }
        let idx = player.hand.iter().position(|c| c.id == card.id).unwrap();
        player.hand.remove(idx);
    }

    check_melds(state);
}

pub fn discard_card(state: &mut RoundState, card_index: usize) {
    if state.phase != Phase::Discard {
        panic!("Not discard phase");
    }

    let player = &mut state.players[state.current_player];
    let card = player.hand.remove(card_index);
    state.fire_pile.push(card);

    state.current_player = (state.current_player + 1) % state.players.len();
}

// --------------------
// Round & scoring
// --------------------

pub fn is_round_over(state: &RoundState) -> bool {
    state.players.iter().any(|p| p.hand.is_empty())
}

pub fn score_round(game_state: &mut GameState, round_state: &mut RoundState) {
    let winner_id = round_state
        .players
        .iter()
        .find(|p| p.hand.is_empty())
        .unwrap()
        .id
        .clone();

    for rp in &mut round_state.players {
        if rp.id != winner_id {
            if !rp.melded {
                rp.score += 100;
            } else {
                let hand_score: i32 = rp.hand.iter().map(|c| card_penality(c)).sum();
                rp.score += hand_score;
            }
        } else {
            rp.score -= 30;
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
        });
    }

    // Sort to ensure order P1..P4 (optional but good for consistency)
    players_back.sort_by_key(|p| p.id.clone());
    game_state.players = players_back;
}

// Include test module
#[cfg(test)]
#[path = "logic_test.rs"]
mod logic_test;
