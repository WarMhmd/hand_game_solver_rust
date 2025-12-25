use axum::extract::ws::Message;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    logic::{start_game, ActivePlayer, Player},
    websocket::handler::{DrawPhaseData, PlayingPhaseData, WsResponse},
    AckSender, AppState,
};
use std::sync::Arc;

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

        // Lock only this specific game, not the entire games map

        if let Err(e) = start_game(game_arc, &state_clone).await {
            println!("❌ Game {} error: {}", game_id, e);
        }
    });

    Ok(())
}

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

/// Handle acknowledgment from a player
pub async fn handle_round_started_ack(
    game_id: &str,
    round_number: i32,
    player_id: &str,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("{}:round_{}", game_id, round_number);

    let mut trackers = state.ack_trackers.write().await;

    let Some(session) = trackers.get_mut(&session_key) else {
        return Err("Session not found or already completed".to_string());
    };

    let Some(sender) = session.remove(player_id) else {
        return Err("Player not found in session or already acknowledged".to_string());
    };

    // Send acknowledgment (ignore error if receiver was dropped)
    match sender {
        AckSender::Simple(tx) => {
            let _ = tx.send(());
            println!(
                "✅ Player {} acknowledged round {}",
                player_id, round_number
            );
            Ok(())
        }
        _ => Err("Unexpected acknowledgment type".to_string()),
    }
}

pub async fn send_round_start_and_wait(
    game_id: &str,
    round_number: i32,
    players_data: Vec<(String, Vec<crate::logic::Card>, Option<String>, bool)>,
    players: &Vec<Player>,
    state: &Arc<AppState>,
    timeout: std::time::Duration,
) -> Result<(), String> {
    use futures::future::join_all;
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    // 1. Create receivers for VOID signal ()
    let mut receivers: Vec<oneshot::Receiver<()>> = Vec::new();
    let mut ack_senders = HashMap::new();

    for player in players {
        if player.sender.is_some() {
            let (tx, rx) = oneshot::channel::<()>(); // <--- Channel carries empty unit ()
                                                     // Wrap in the Enum variant Simple
            ack_senders.insert(player.id.clone(), AckSender::Simple(tx));
            receivers.push(rx);
        }
    }

    let session_key = format!("{}:round_{}", game_id, round_number);
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.insert(session_key.clone(), ack_senders);
    }

    // ... (Send logic remains the same) ...
    for (player_id, hand, fire_card_id, melded) in players_data {
        if let Some(player) = players.iter().find(|p| p.id == player_id) {
            if let Some(sender) = &player.sender {
                let message = WsResponse::RoundStarted {
                    round_number,
                    hand,
                    fire_card_id,
                    melded,
                };
                send_to_player(sender, message);
            }
        }
    }

    // 2. Wait logic
    let wait_result = tokio::time::timeout(timeout, async {
        let results = join_all(receivers).await;
        if results.iter().any(|res| res.is_err()) {
            return Err("Player acknowledgment failed".to_string());
        }
        Ok(())
    })
    .await;

    // Clean up
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.remove(&session_key);
    }

    match wait_result {
        Ok(Ok(())) => {
            println!("✅ All players acknowledged round {}", round_number);
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Timeout waiting for acknowledgments".to_string()),
    }
}

pub async fn send_player_draw_phase_and_wait(
    player: &ActivePlayer,
    game_id: &String,
    round_number: i32,
    state: &Arc<AppState>,
) -> Result<DrawPhaseData, String> {
    // <--- RETURN TYPE CHANGED
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let mut ack_senders = HashMap::new();

    // 1. Create channel that carries DATA
    let (tx, rx) = oneshot::channel::<DrawPhaseData>();

    // Wrap in the Enum variant Draw
    ack_senders.insert(player.id.clone(), AckSender::Draw(tx));

    let session_key = format!("DrawPhase-{}-{}-{}", game_id, round_number, player.id);

    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.insert(session_key.clone(), ack_senders);
    }

    if let Some(sender) = &player.sender {
        let message = WsResponse::DrawPhase {
            player_id: player.id.clone(),
        };
        send_to_player(sender, message);
    }

    // 2. Wait for the DATA
    let wait_result = tokio::time::timeout(std::time::Duration::from_secs(45), async {
        // We only have one receiver here, no need for join_all logic
        match rx.await {
            Ok(data) => Ok(data), // We got the struct!
            Err(_) => Err("Player disconnected or failed to reply".to_string()),
        }
    })
    .await;

    // Clean up
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.remove(&session_key);
    }

    // 3. Return the actual data
    match wait_result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Timeout waiting for draw choice".to_string()),
    }
}

pub async fn handle_draw_phase_ack(
    player_id: String,
    game_id: String,
    round_number: i32,
    data: DrawPhaseData, // The data from frontend
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("DrawPhase-{}-{}-{}", game_id, round_number, player_id);

    let mut trackers = state.ack_trackers.write().await;

    let Some(session) = trackers.get_mut(&session_key) else {
        return Err("Session not found".to_string());
    };

    let Some(sender_enum) = session.remove(&player_id) else {
        return Err("Player not found in session".to_string());
    };

    // Match the enum to get the correct channel type
    match sender_enum {
        AckSender::Draw(tx) => {
            // Send the data structure into the channel
            let _ = tx.send(data);
            Ok(())
        }
        _ => Err("Unexpected acknowledgment type: Expected Draw, got Simple".to_string()),
    }
}

pub async fn send_player_playing_phase_and_wait(
    player: &ActivePlayer,
    game_id: &String,
    round_number: i32,
    state: &Arc<AppState>,
) -> Result<PlayingPhaseData, String> {
    // <--- RETURN TYPE CHANGED
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    let mut ack_senders = HashMap::new();

    // 1. Create channel that carries DATA
    let (tx, rx) = oneshot::channel::<PlayingPhaseData>();

    // Wrap in the Enum variant Draw
    ack_senders.insert(player.id.clone(), AckSender::Playing(tx));

    let session_key = format!("PlayingPhase-{}-{}-{}", game_id, round_number, player.id);

    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.insert(session_key.clone(), ack_senders);
    }

    if let Some(sender) = &player.sender {
        let message = WsResponse::PlayingPhaseStarted {
            player_id: player.id.clone(),
        };
        send_to_player(sender, message);
    }

    // 2. Wait for the DATA
    let wait_result = tokio::time::timeout(std::time::Duration::from_secs(60), async {
        // We only have one receiver here, no need for join_all logic
        match rx.await {
            Ok(data) => Ok(data), // We got the struct!
            Err(_) => Err("Player disconnected or failed to reply".to_string()),
        }
    })
    .await;

    // Clean up
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.remove(&session_key);
    }

    // 3. Return the actual data
    match wait_result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Timeout waiting for playing choice".to_string()),
    }
}

pub async fn handle_playing_phase_ack(
    player_id: String,
    game_id: String,
    round_number: i32,
    data: PlayingPhaseData, // The data from frontend
    state: &Arc<AppState>,
) -> Result<(), String> {
    let session_key = format!("PlayingPhase-{}-{}-{}", game_id, round_number, player_id);

    let mut trackers = state.ack_trackers.write().await;

    let Some(session) = trackers.get_mut(&session_key) else {
        return Err("Session not found".to_string());
    };

    let Some(sender_enum) = session.remove(&player_id) else {
        return Err("Player not found in session".to_string());
    };

    // Match the enum to get the correct channel type
    match sender_enum {
        AckSender::Playing(tx) => {
            // Send the data structure into the channel
            let _ = tx.send(data);
            Ok(())
        }
        _ => Err("Unexpected acknowledgment type: Expected Playing".to_string()),
    }
}
