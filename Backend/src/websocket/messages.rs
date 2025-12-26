use serde::Deserialize;

use crate::{
    bot::DecideDrawResult,
    logic::{Card, Meld},
};

/// Incoming WebSocket messages from clients
#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum WsMessage {
    #[serde(rename_all = "camelCase")]
    Join {
        game_id: String,
        player_id: String,
    },
    #[serde(rename_all = "camelCase")]
    StartGame {
        game_id: String,
        player_id: String,
    },
    #[serde(rename_all = "camelCase")]
    RoundStarted {
        game_id: String,
        player_id: String,
        round_number: i32,
    },
    #[serde(rename_all = "camelCase")]
    DrawPhaseAck {
        game_id: String,
        player_id: String,
        sender_id: String,
        round_number: i32,
    },
    #[serde(rename_all = "camelCase")]
    DrawPhaseFinished {
        game_id: String,
        player_id: String,
        round_number: i32,
        data: DrawPhaseData,
    },
    #[serde(rename_all = "camelCase")]
    PlayingPhaseStarted {
        game_id: String,
        player_id: String,
        round_number: i32,
        data: PlayingPhaseData,
    },
    #[serde(rename_all = "camelCase")]
    SyncMeldsAck {
        game_id: String,
        player_id: String,
        sender_id: String,
        round_number: i32,
    },
    #[serde(rename_all = "camelCase")]
    PlayerDiscardedAck {
        game_id: String,
        player_id: String,
        sender_id: String,
        round_number: i32,
    },
    Ping,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawPhaseData {
    pub draw_choice: DecideDrawResult,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum PlayingPhaseData {
    PlayMeldPhase(PlayMeldPhaseData),
    PlayInMeldPhase(PlayInMeldPhaseData),
    DiscardPhase(DiscardPhaseData),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayMeldPhaseData {
    pub melds: Vec<Meld>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayInMeldPhaseData {
    pub card: Card,
    pub meld_id: String,
    pub is_left: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardPhaseData {
    pub card: Card,
}
