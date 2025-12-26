use axum::extract::ws::Message;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    logic::{start_game, ActivePlayer, Card, Meld, Player},
    websocket::{
        messages::{DrawPhaseData, PlayingPhaseData},
        responses::WsResponse,
    },
    AckSender, AppState,
};
use std::{sync::Arc, time::Duration};

// ============================================================================
// Game Setup Events
// ============================================================================

pub async fn handle_join_event(
    game_id: String,
    player_id: String,
    sender: UnboundedSender<Message>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    // Lock only the games map to get the game Arc
    let game_arc = {
        let games = state.games.read().await;
        games.get(&game_id).cloned()
    };

    let Some(game_arc) = game_arc else {
        return Err("No such room".into());
    };

    // Lock only this specific game
    let mut game = game_arc.lock().await;

    let Some(player) = game
        .players
        .iter_mut()
        .find(|player| player.id == player_id)
    else {
        return Err("No such player".into());
    };

    if player.did_join {
        return Err("Player already joined".into());
    }

    player.did_join = true;
    player.sender = Some(sender);

    println!("Player {} joined game {}", player_id, game_id);

    Ok(())
}

pub async fn handle_start_game_event(
    game_id: String,
    player_id: String,
    state: &Arc<AppState>,
) -> Result<(), String> {
    // Get the game Arc without holding the global lock
    let game_arc = {
        let games = state.games.read().await;
        games.get(&game_id).cloned()
    };

    let Some(game_arc) = game_arc else {
        return Err("No such room".into());
    };

    // Validate player in a separate scope
    {
        let game = game_arc.lock().await;

        let Some(player) = game.players.iter().find(|player| player.id == player_id) else {
            return Err("No such player".into());
        };

        if !player.did_join {
            return Err("Player did not join".into());
        }

        if game.players.iter().any(|player| !player.did_join) {
            return Err("Not all players joined".into());
        }
    }

    // Start the game in a background task to not block other games
    let state_clone = state.clone();
    tokio::spawn(async move {
        println!("🚀 Spawning background game: {}", game_id);

        if let Err(e) = start_game(game_arc, &state_clone).await {
            println!("❌ Game {} error: {}", game_id, e);
        }
    });

    Ok(())
}

// ============================================================================
// Message Sending Utilities
// ============================================================================

/// Send a message to a specific player
pub fn send_to_player(sender: &UnboundedSender<Message>, message: impl Serialize) {
    if let Ok(json) = serde_json::to_string(&message) {
        let _ = sender.send(Message::Text(json.into()));
    }
}

/// Broadcast a message to all players in a game
pub async fn broadcast_to_game(players: &Vec<Player>, message: WsResponse) -> Result<(), String> {
    if let Ok(json) = serde_json::to_string(&message) {
        for player in players {
            if let Some(sender) = &player.sender {
                let _ = sender.send(Message::Text(json.clone().into()));
            }
        }
    }

    Ok(())
}

// ============================================================================
// Generic Send-and-Wait Pattern
// ============================================================================

/// Generic function to send a message and wait for acknowledgment
async fn send_and_wait<T, F>(
    session_key: String,
    ack_sender_factory: F,
    send_messages: impl FnOnce(),
    state: &Arc<AppState>,
    timeout: std::time::Duration,
) -> Result<T, String>
where
    F: FnOnce() -> (
        Vec<tokio::sync::oneshot::Receiver<T>>,
        std::collections::HashMap<String, AckSender>,
    ),
{
    use futures::future::join_all;

    // Create channels and receivers
    let (receivers, ack_senders) = ack_sender_factory();

    // Store in tracker
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.insert(session_key.clone(), ack_senders);
    }

    // Send messages
    send_messages();

    // Wait for responses
    let wait_result = tokio::time::timeout(timeout, async {
        if receivers.len() == 1 {
            // Single receiver case
            match receivers.into_iter().next().unwrap().await {
                Ok(data) => Ok(data),
                Err(_) => Err("Player disconnected or failed to reply".to_string()),
            }
        } else {
            // Multiple receivers case
            let results = join_all(receivers).await;
            if results.iter().any(|res| res.is_err()) {
                return Err("Player acknowledgment failed".to_string());
            }
            // For multiple receivers returning (), we just return ()
            // This is a bit of a hack but works for our use case
            Ok(unsafe { std::mem::zeroed() })
        }
    })
    .await;

    // Clean up
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.remove(&session_key);
    }

    match wait_result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Timeout waiting for acknowledgment".to_string()),
    }
}

/// Generic function to handle acknowledgment
async fn handle_ack<T>(
    session_key: String,
    player_id: String,
    data: T,
    expected_type: &str,
    extractor: impl FnOnce(AckSender) -> Option<tokio::sync::oneshot::Sender<T>>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let mut trackers = state.ack_trackers.write().await;

    let Some(session) = trackers.get_mut(&session_key) else {
        return Err("Session not found".to_string());
    };

    let Some(sender_enum) = session.remove(&player_id) else {
        return Err("Player not found in session".to_string());
    };

    match extractor(sender_enum) {
        Some(tx) => {
            let _ = tx.send(data);
            Ok(())
        }
        None => Err(format!(
            "Unexpected acknowledgment type: Expected {}",
            expected_type
        )),
    }
}

// ============================================================================
// Round Started Event
// ============================================================================

pub async fn send_round_start_and_wait(
    game_id: &str,
    round_number: i32,
    players_data: Vec<(String, Vec<Card>, Option<String>, bool)>,
    players: &Vec<Player>,
    scores: Vec<i32>,
    state: &Arc<AppState>,
    timeout: std::time::Duration,
) -> Result<(), String> {
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let session_key = format!("{}:round_{}", game_id, round_number);
    let players_data_clone = players_data.clone();

    send_and_wait(
        session_key,
        || {
            let mut receivers = Vec::new();
            let mut ack_senders = HashMap::new();

            for player in players {
                if player.sender.is_some() {
                    let (tx, rx) = oneshot::channel::<()>();
                    ack_senders.insert(player.id.clone(), AckSender::Simple(tx));
                    receivers.push(rx);
                }
            }

            (receivers, ack_senders)
        },
        || {
            for (player_id, hand, fire_card_id, melded) in players_data_clone {
                if let Some(player) = players.iter().find(|p| p.id == player_id) {
                    if let Some(sender) = &player.sender {
                        let message = WsResponse::RoundStarted {
                            round_number,
                            hand,
                            fire_card_id,
                            melded,
                            scores: scores.clone(),
                        };
                        send_to_player(sender, message);
                    }
                }
            }
        },
        state,
        timeout,
    )
    .await?;

    println!("✅ All players acknowledged round {}", round_number);
    Ok(())
}

pub async fn handle_round_started_ack(
    game_id: &str,
    round_number: i32,
    player_id: &str,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("{}:round_{}", game_id, round_number);

    handle_ack(
        session_key,
        player_id.to_string(),
        (),
        "Simple",
        |sender_enum| match sender_enum {
            AckSender::Simple(tx) => Some(tx),
            _ => None,
        },
        state,
    )
    .await?;

    println!(
        "✅ Player {} acknowledged round {}",
        player_id, round_number
    );
    Ok(())
}

// ============================================================================
// Draw Phase Events
// ============================================================================

pub async fn send_player_draw_phase_and_wait(
    player: &ActivePlayer,
    game_id: &String,
    round_number: i32,
    state: &Arc<AppState>,
) -> Result<DrawPhaseData, String> {
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let session_key = format!("DrawPhase-{}-{}-{}", game_id, round_number, player.id);
    let player_id = player.id.clone();
    let player_sender = player.sender.clone();

    send_and_wait(
        session_key,
        || {
            let (tx, rx) = oneshot::channel::<DrawPhaseData>();
            let mut ack_senders = HashMap::new();
            ack_senders.insert(player_id.clone(), AckSender::Draw(tx));
            (vec![rx], ack_senders)
        },
        || {
            if let Some(sender) = &player_sender {
                let message = WsResponse::DrawPhase {
                    player_id: player_id.clone(),
                };
                send_to_player(sender, message);
            }
        },
        state,
        std::time::Duration::from_mins(45),
    )
    .await
}

pub async fn handle_draw_phase_ack(
    player_id: String,
    game_id: String,
    round_number: i32,
    data: DrawPhaseData,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("DrawPhase-{}-{}-{}", game_id, round_number, player_id);

    handle_ack(
        session_key,
        player_id,
        data,
        "Draw",
        |sender_enum| match sender_enum {
            AckSender::Draw(tx) => Some(tx),
            _ => None,
        },
        state,
    )
    .await
}

// ============================================================================
// Draw Card Events
// ============================================================================

pub async fn send_player_draw_card_and_wait(
    player: &ActivePlayer,
    game_id: &String,
    round_number: i32,
    card: Card,
    is_fire_card: bool,
    empty_fire_pile: bool,
    players: &Vec<ActivePlayer>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    use std::collections::HashMap;
    use tokio::sync::oneshot;
    let session_key = format!("DrawCard-{}-{}-{}", game_id, round_number, player.id);
    println!("session: {}", session_key);
    let player_id = player.id.clone();

    send_and_wait(
        session_key,
        || {
            let mut receivers = Vec::new();
            let mut ack_senders = HashMap::new();

            for player in players {
                if player.sender.is_some() {
                    let (tx, rx) = oneshot::channel::<()>();
                    ack_senders.insert(player.id.clone(), AckSender::Simple(tx));
                    receivers.push(rx);
                }
            }

            (receivers, ack_senders)
        },
        || {
            for player in players {
                if let Some(sender) = &player.sender {
                    let message = WsResponse::DrawnCard {
                        player_id: player_id.clone(),
                        empty_fire_pile,
                        card: if player.id == player_id {
                            Some(card.clone())
                        } else {
                            None
                        },
                        is_fire_card,
                    };
                    send_to_player(sender, message);
                }
            }
        },
        state,
        std::time::Duration::from_mins(45),
    )
    .await
}

pub async fn handle_draw_phase_finished(
    player_id: String,
    game_id: String,
    round_number: i32,
    sender_id: String,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("DrawCard-{}-{}-{}", game_id, round_number, player_id);
    println!("session: {}", session_key);
    handle_ack(
        session_key,
        sender_id,
        (),
        "Simple",
        |sender_enum| match sender_enum {
            AckSender::Simple(tx) => Some(tx),
            _ => None,
        },
        state,
    )
    .await
}

// ============================================================================
// Playing Phase Events
// ============================================================================

pub async fn send_player_playing_phase_and_wait(
    player: &ActivePlayer,
    game_id: &String,
    round_number: i32,
    state: &Arc<AppState>,
) -> Result<PlayingPhaseData, String> {
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let session_key = format!("PlayingPhase-{}-{}-{}", game_id, round_number, player.id);
    let player_id = player.id.clone();
    let player_sender = player.sender.clone();

    send_and_wait(
        session_key,
        || {
            let (tx, rx) = oneshot::channel::<PlayingPhaseData>();
            let mut ack_senders = HashMap::new();
            ack_senders.insert(player_id.clone(), AckSender::Playing(tx));
            (vec![rx], ack_senders)
        },
        || {
            if let Some(sender) = &player_sender {
                let message = WsResponse::PlayingPhaseStarted {
                    player_id: player_id.clone(),
                };
                send_to_player(sender, message);
            }
        },
        state,
        std::time::Duration::from_mins(30),
    )
    .await
}

pub async fn handle_playing_phase_ack(
    player_id: String,
    game_id: String,
    round_number: i32,
    data: PlayingPhaseData,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("PlayingPhase-{}-{}-{}", game_id, round_number, player_id);

    handle_ack(
        session_key,
        player_id,
        data,
        "Playing",
        |sender_enum| match sender_enum {
            AckSender::Playing(tx) => Some(tx),
            _ => None,
        },
        state,
    )
    .await
}

pub async fn send_players_sync_melds(
    game_id: &String,
    round_number: i32,
    player_id: &String,
    table_melds: &Vec<Meld>,
    player_melds: Vec<Card>,
    player_hand_size: usize,
    take_joker: Option<Card>,
    players: &Vec<ActivePlayer>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let session_key = format!("PlayerSyncMelds-{}-{}-{}", game_id, round_number, player_id);

    send_and_wait(
        session_key,
        || {
            let mut receivers = Vec::new();
            let mut ack_senders = HashMap::new();

            for player in players {
                if player.sender.is_some() {
                    let (tx, rx) = oneshot::channel::<()>();
                    ack_senders.insert(player.id.clone(), AckSender::Simple(tx));
                    receivers.push(rx);
                }
            }

            (receivers, ack_senders)
        },
        || {
            for player in players {
                if let Some(sender) = &player.sender {
                    let message = WsResponse::SyncMelds {
                        player_id: player_id.clone(),
                        table_melds: table_melds.clone(),
                        player_melds: player_melds.clone(),
                        take_joker: take_joker.clone(),
                        player_hand_size,
                    };
                    send_to_player(sender, message);
                }
            }
        },
        state,
        Duration::from_mins(30),
    )
    .await?;

    println!("✅ All players acknowledged sync melds {}", round_number);
    Ok(())
}

pub async fn handle_players_sync_melds_ack(
    game_id: String,
    round_number: i32,
    player_id: String,
    sender_id: String,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("PlayerSyncMelds-{}-{}-{}", game_id, round_number, player_id);

    handle_ack(
        session_key,
        sender_id,
        (),
        "Simple",
        |sender_enum| match sender_enum {
            AckSender::Simple(tx) => Some(tx),
            _ => None,
        },
        state,
    )
    .await?;

    Ok(())
}

pub async fn send_players_discarded_card(
    game_id: &String,
    round_number: i32,
    player_id: &String,
    card: &Card,
    player_hand_size: usize,
    players: &Vec<ActivePlayer>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let session_key = format!("PlayerDiscard-{}-{}-{}", game_id, round_number, player_id);

    send_and_wait(
        session_key,
        || {
            let mut receivers = Vec::new();
            let mut ack_senders = HashMap::new();

            for player in players {
                if player.sender.is_some() {
                    let (tx, rx) = oneshot::channel::<()>();
                    ack_senders.insert(player.id.clone(), AckSender::Simple(tx));
                    receivers.push(rx);
                }
            }

            (receivers, ack_senders)
        },
        || {
            for player in players {
                if let Some(sender) = &player.sender {
                    let message = WsResponse::PlayerDiscarded {
                        player_id: player_id.clone(),
                        card: card.clone(),
                        player_hand_size,
                    };
                    send_to_player(sender, message);
                }
            }
        },
        state,
        Duration::from_mins(30),
    )
    .await?;

    println!("✅ All players acknowledged discard card {}", player_id);
    Ok(())
}

pub async fn handle_players_discard_ack(
    game_id: String,
    round_number: i32,
    player_id: String,
    sender_id: String,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("PlayerDiscard-{}-{}-{}", game_id, round_number, player_id);

    handle_ack(
        session_key,
        sender_id,
        (),
        "Simple",
        |sender_enum| match sender_enum {
            AckSender::Simple(tx) => Some(tx),
            _ => None,
        },
        state,
    )
    .await?;

    Ok(())
}
