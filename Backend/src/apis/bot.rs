use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::better_discard::{self, UseBetterDiscard};
use crate::logic::{
    ActivePlayer, Card, GameState, Meld, MeldType, Phase, Player, Rank, RoundState,
};
use crate::AppState;
use axum::{debug_handler, extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ReturnInitBot {
    id: String,
}

#[debug_handler]
pub async fn init_bot(State(state): State<Arc<AppState>>) -> Json<ReturnInitBot> {
    let mut bots = state.bots.write().await;
    let id = Uuid::new_v4().to_string();
    let bot = UseBetterDiscard::new("test".into());
    bots.insert(id.clone(), Arc::new(tokio::sync::Mutex::new(bot)));

    let response = ReturnInitBot { id };
    Json(response)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawCardData {
    bot_id: String,
    cards: Vec<Card>,
    fire_card: Option<Card>,
}

#[derive(Serialize)]
pub struct DrawCardResponse {
    draw: DecideDrawResult,
}

#[debug_handler]
pub async fn draw_card(
    State(state): State<Arc<AppState>>,
    Json(data): Json<DrawCardData>,
) -> Json<DrawCardResponse> {
    // print all cards for draw
    println!("Fire Card: {:?}", data.fire_card);
    let round_state = RoundState {
        current_player: 0,
        table_melds: Vec::new(),
        fire_pile: if data.fire_card.is_some() {
            vec![data.fire_card.unwrap()]
        } else {
            vec![]
        },
        deck: vec![],
        phase: Phase::Draw,
        players: vec![ActivePlayer {
            hand: data.cards,
            bot_strategy: None,
            did_join: true,
            fire_card_id: None,
            id: "".into(),
            melded: false,
            name: "Player 1".into(),
            score: 0,
            sender: None,
        }],
    };

    let bot_arc = state.get_bot(&data.bot_id).await.unwrap();

    let result = {
        let mut bot = bot_arc.lock().await;
        bot.decide_draw(&round_state)
    };

    let response = DrawCardResponse { draw: result };

    Json(response)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeldsData {
    bot_id: String,
    cards: Vec<Card>,
    melded: bool,
}

#[derive(Serialize)]
pub struct MeldsResponse {
    melds: Vec<Vec<Card>>,
}

#[debug_handler]
pub async fn meld_cards(
    State(state): State<Arc<AppState>>,
    Json(data): Json<MeldsData>,
) -> Json<MeldsResponse> {
    // print incoming cards
    for card in &data.cards {
        println!("Card: {:?}", card.id.clone());
    }
    println!("meld_cards --> melded: {:?}", data.melded);

    let round_state = RoundState {
        current_player: 0,
        table_melds: Vec::new(),
        fire_pile: vec![],
        deck: vec![],
        phase: Phase::Meld,
        players: vec![ActivePlayer {
            hand: data.cards,
            bot_strategy: None,
            did_join: true,
            fire_card_id: None,
            id: "".into(),
            melded: data.melded,
            name: "Player 1".into(),
            score: 0,
            sender: None,
        }],
    };

    let bot_arc = state.get_bot(&data.bot_id).await.unwrap();

    let result: Vec<Vec<Card>> = {
        let mut bot = bot_arc.lock().await;
        bot.decide_melds(&round_state)
            .iter()
            .map(|melds| melds.cards.clone())
            .collect()
    };

    let response = MeldsResponse { melds: result };

    Json(response)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayInMeldsData {
    bot_id: String,
    cards: Vec<Card>,
    melded: bool,
    table_cards: Vec<Vec<Card>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayInMeldsResponse {
    phase: Phase,
    card: Option<Card>,
    is_left: bool,
    index: i32,
}

#[debug_handler]
pub async fn play_in_melds(
    State(state): State<Arc<AppState>>,
    Json(data): Json<PlayInMeldsData>,
) -> Json<PlayInMeldsResponse> {
    // print incoming cards

    let round_state = RoundState {
        current_player: 0,
        table_melds: data
            .table_cards
            .iter()
            .map(|cards| {
                let first_suit = cards.iter().find(|card| card.rank != Rank::Joker).unwrap();
                let second_suit = cards
                    .iter()
                    .find(|card| card.rank != Rank::Joker && card.suit != first_suit.suit);

                Meld {
                    id: Uuid::new_v4().into(),
                    cards: cards.clone(),
                    meld_type: if second_suit.is_some() {
                        MeldType::Rank
                    } else {
                        MeldType::Sequence
                    },
                }
            })
            .collect(),
        fire_pile: vec![],
        deck: vec![],
        phase: Phase::PlayInMeld,
        players: vec![ActivePlayer {
            hand: data.cards,
            bot_strategy: None,
            did_join: true,
            fire_card_id: None,
            id: "".into(),
            melded: data.melded,
            name: "Player 1".into(),
            score: 0,
            sender: None,
        }],
    };

    let bot_arc = state.get_bot(&data.bot_id).await.unwrap();

    let result = {
        let mut bot = bot_arc.lock().await;
        bot.decide_play_in_meld(&round_state)
    };

    let response = PlayInMeldsResponse {
        phase: result.0,
        card: result.1,
        is_left: result.2,
        index: result.3,
    };

    println!(
        "Play in meld phase: {:?}, card: {:?}, is_left: {:?}, index: {:?}",
        response.phase, response.card, response.is_left, response.index
    );

    Json(response)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardData {
    bot_id: String,
    cards: Vec<Card>,
    melded: bool,
}

#[derive(Serialize)]
pub struct DiscardResponse {
    index: usize,
}

#[debug_handler]
pub async fn discard(
    State(state): State<Arc<AppState>>,
    Json(data): Json<DiscardData>,
) -> Json<DiscardResponse> {
    // print incoming cards
    println!("Hand:");
    for card in &data.cards {
        println!("Card: {:?}", card.id.clone());
    }

    let round_state = RoundState {
        current_player: 0,
        table_melds: vec![],
        fire_pile: vec![],
        deck: vec![],
        phase: Phase::Discard,
        players: vec![ActivePlayer {
            hand: data.cards,
            bot_strategy: None,
            did_join: true,
            fire_card_id: None,
            id: "".into(),
            melded: data.melded,
            name: "Player 1".into(),
            score: 0,
            sender: None,
        }],
    };

    let bot_arc = state.get_bot(&data.bot_id).await.unwrap();

    let result = {
        let mut bot = bot_arc.lock().await;
        bot.decide_discard(&round_state)
    };

    let response = DiscardResponse { index: result };

    Json(response)
}
