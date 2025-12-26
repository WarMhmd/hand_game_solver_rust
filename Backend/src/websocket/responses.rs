use serde::Serialize;

use crate::logic::{Card, Meld, Player, PlayerResult};

/// Outgoing WebSocket messages sent to clients
#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum WsResponse {
    Joined {
        success: bool,
        message: Option<String>,
    },
    GameStarted,
    #[serde(rename_all = "camelCase")]
    RoundStarted {
        round_number: i32,
        hand: Vec<Card>,
        fire_card_id: Option<String>,
        melded: bool,
        scores: Vec<i32>,
    },
    #[serde(rename_all = "camelCase")]
    DrawPhase {
        player_id: String,
    },
    #[serde(rename_all = "camelCase")]
    DrawnCard {
        player_id: String,
        card: Option<Card>,
        is_fire_card: bool,
        empty_fire_pile: bool,
    },
    #[serde(rename_all = "camelCase")]
    PlayingPhaseStarted {
        player_id: String,
    },
    #[serde(rename_all = "camelCase")]
    SyncMelds {
        player_id: String,
        table_melds: Vec<Meld>,
        player_melds: Vec<Card>,
        player_hand_size: usize,
        take_joker: Option<Card>,
    },
    #[serde(rename_all = "camelCase")]
    PlayerDiscarded {
        player_id: String,
        card: Card,
        player_hand_size: usize,
    },
    #[serde(rename_all = "camelCase")]
    GameOver {
        players: Vec<PlayerResult>,
    },
    Error {
        message: String,
    },
    Pong,
}
