mod bot;
mod logic;
mod websocket;
mod apis {
    pub mod game;
}
mod bots {
    pub mod play_in_melds;
    pub mod play_with_sequence;
    pub mod use_fire;
    pub mod use_joker;
}

use crate::logic::GameState;
use crate::websocket::websocket_handler;
use crate::{
    apis::game::init_game,
    websocket::{DrawPhaseData, PlayingPhaseData},
};

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex, RwLock};

pub enum AckSender {
    Simple(oneshot::Sender<()>),
    Draw(oneshot::Sender<DrawPhaseData>),
    Playing(oneshot::Sender<PlayingPhaseData>),
}

#[derive(Clone)]
pub struct AppState {
    pub games: Arc<RwLock<HashMap<String, Arc<Mutex<GameState>>>>>,
    pub ack_trackers: Arc<RwLock<HashMap<String, HashMap<String, AckSender>>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        games: Arc::new(RwLock::new(HashMap::new())),
        ack_trackers: Arc::new(RwLock::new(HashMap::new())),
    };

    let api = Router::new()
        .route("/api/game/init_game", post(init_game))
        .layer(CorsLayer::permissive());

    let ws = Router::new().route("/ws", get(websocket_handler));

    let app = Router::new()
        .merge(api)
        .merge(ws)
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("🚀 Backend running on http://localhost:3000");
    println!("🔌 WebSocket endpoint: ws://localhost:3000/ws");
    axum::serve(listener, app).await.unwrap();
}
