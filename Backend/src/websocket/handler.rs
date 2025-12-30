use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use super::events::{
    handle_draw_phase_ack, handle_draw_phase_finished, handle_join_event, handle_playing_phase_ack,
    handle_round_started_ack, handle_start_game_event,
};
use crate::{
    websocket::{
        handle_players_discard_ack, handle_players_sync_melds_ack, messages::WsMessage,
        responses::WsResponse,
    },
    AppState,
};

async fn handle_ws_event(
    msg: WsMessage,
    tx: UnboundedSender<Message>,
    state: &Arc<AppState>,
) -> Option<WsResponse> {
    match msg {
        WsMessage::Join { game_id, player_id } => {
            match handle_join_event(game_id, player_id, tx, state).await {
                Ok(()) => Some(WsResponse::Joined {
                    success: true,
                    message: None,
                }),
                Err(error_msg) => Some(WsResponse::Joined {
                    success: false,
                    message: Some(error_msg),
                }),
            }
        }
        WsMessage::StartGame { game_id, player_id } => {
            match handle_start_game_event(game_id, player_id, state).await {
                Ok(()) => Some(WsResponse::GameStarted),
                Err(error_msg) => Some(WsResponse::GameStarted),
            }
        }

        WsMessage::RoundStarted {
            game_id,
            player_id,
            round_number,
        } => {
            if let Err(e) =
                handle_round_started_ack(&game_id, round_number, &player_id, state).await
            {
                println!("⚠️ Failed to handle round started ack: {}", e);
            }
            None
        }

        WsMessage::DrawPhaseFinished {
            game_id,
            player_id,
            round_number,
            data,
        } => {
            if let Err(e) =
                handle_draw_phase_ack(player_id, game_id, round_number, data, state).await
            {
                println!("⚠️ Failed to handle draw phase finished: {}", e);
            }
            None
        }

        WsMessage::PlayingPhaseStarted {
            game_id,
            player_id,
            round_number,
            data,
        } => {
            if let Err(e) =
                handle_playing_phase_ack(player_id, game_id, round_number, data, state).await
            {
                println!("⚠️ Failed to handle playing phase started: {}", e);
            }
            None
        }

        WsMessage::DrawPhaseAck {
            game_id,
            player_id,
            sender_id,
            round_number,
        } => {
            if let Err(e) =
                handle_draw_phase_finished(player_id, game_id, round_number, sender_id, state).await
            {
                println!("⚠️ Failed to handle draw phase ack: {}", e);
            }
            None
        }

        WsMessage::SyncMeldsAck {
            game_id,
            player_id,
            round_number,
            sender_id,
        } => {
            if let Err(e) =
                handle_players_sync_melds_ack(game_id, round_number, player_id, sender_id, state)
                    .await
            {
                println!("⚠️ Failed to handle sync melds ack: {}", e);
            }
            None
        }

        WsMessage::PlayerDiscardedAck {
            game_id,
            player_id,
            round_number,
            sender_id,
        } => {
            if let Err(e) =
                handle_players_discard_ack(game_id, round_number, player_id, sender_id, state).await
            {
                println!("⚠️ Failed to handle player discarded ack: {}", e);
            }
            None
        }

        WsMessage::Ping => {
            println!("🏓 Ping received");
            return Some(WsResponse::Pong);
        }
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let origin = headers.get("origin").and_then(|v| v.to_str().ok());

    // Allow specific origins
    if matches!(origin, Some("http://localhost:5173") | Some("https://hand-solver.web.app")) {
        return ws.on_upgrade(|socket| handle_socket(socket, state));
    }
    println!("❌ WebSocket connection rejected due to invalid origin: {:?}", origin);

    StatusCode::FORBIDDEN.into_response()
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (ws_sender, mut ws_receiver) = socket.split();

    // Create mpsc channel for sending messages to this WebSocket
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    println!("✅ New WebSocket connection established");

    // Spawn task to forward messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        let mut ws_sender = ws_sender;
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                println!("❌ Failed to send message to WebSocket");
                break;
            }
        }
    });

    // Clone tx for use in message handling
    let tx_clone = tx.clone();

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        println!("📨 Received message: {}", text);

                        // Parse JSON message
                        match serde_json::from_str::<WsMessage>(&text) {
                            Ok(ws_msg) => {
                                let response =
                                    handle_ws_event(ws_msg, tx_clone.clone(), &state).await;
                                if let Some(response) = response {
                                    let response_json = serde_json::to_string(&response).unwrap();

                                    if tx_clone.send(Message::Text(response_json.into())).is_err() {
                                        println!("❌ Failed to send response");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("⚠️ Failed to parse message: {}", e);
                                let error_response = WsResponse::Error {
                                    message: format!("Invalid message format: {}", e),
                                };
                                let error_json = serde_json::to_string(&error_response).unwrap();
                                let _ = tx_clone.send(Message::Text(error_json.into()));
                            }
                        }
                    }
                    Message::Binary(data) => {
                        println!("📦 Received binary data: {} bytes", data.len());
                    }
                    Message::Close(_) => {
                        println!("🔌 WebSocket connection closed");
                        break;
                    }
                    _ => {}
                }
            } else {
                println!("🔌 Client disconnected");
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    println!("👋 WebSocket connection terminated");
}
