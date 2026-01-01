mod bot;
mod logic;
mod websocket;
mod apis {
    pub mod bot;
    pub mod game;
}
mod bots {
    pub mod better_discard;
    pub mod better_meld_play;
    pub mod play_in_melds;
    pub mod play_with_sequence;
    pub mod use_fire;
    pub mod use_joker;
}

use crate::bots::better_discard::UseBetterDiscard;
use crate::logic::GameState;
use crate::websocket::websocket_handler;
use crate::{
    apis::bot::{discard, draw_card, init_bot, meld_cards, play_in_melds},
    apis::game::{init_game, init_game_with_random_all_bots, init_game_with_random_strong_bots},
    websocket::{DrawPhaseData, PlayingPhaseData},
};

use axum::http::{self, HeaderValue, Method};
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

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
    pub bots: Arc<RwLock<HashMap<String, Arc<Mutex<UseBetterDiscard>>>>>,
    pub ack_trackers: Arc<RwLock<HashMap<String, HashMap<String, AckSender>>>>,
}

impl AppState {
    pub async fn get_bot(&self, bot_id: &str) -> Option<Arc<Mutex<UseBetterDiscard>>> {
        let bots = self.bots.read().await;
        bots.get(bot_id).cloned()
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        games: Arc::new(RwLock::new(HashMap::new())),
        bots: Arc::new(RwLock::new(HashMap::new())),
        ack_trackers: Arc::new(RwLock::new(HashMap::new())),
    };

    let allowed_origins_str = "https://www.jawaker.com,https://cdn.jawaker.com,https://hand.warmhmd.online";
    let allowed_origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .map(|s| HeaderValue::from_str(s.trim()).unwrap())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([http::header::CONTENT_TYPE])
        .allow_credentials(true);

    async fn health_check() -> &'static str {
        "OK"
    }

    let api = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/bot/init-bot", post(init_bot))
        .route("/api/v1/bot/draw-card", post(draw_card))
        .route("/api/v1/bot/meld-cards", post(meld_cards))
        .route("/api/v1/bot/play-in-melds", post(play_in_melds))
        .route("/api/v1/bot/discard", post(discard))
        .route("/api/v1/bot/init_game", post(init_game))
        .route(
            "/api/game/init_game_with_random_strong_bots",
            post(init_game_with_random_strong_bots),
        )
        .route(
            "/api/game/init_game_with_random_all_bots",
            post(init_game_with_random_all_bots),
        )
        .layer(cors);

    let ws = Router::new().route("/ws", get(websocket_handler));

    let app = Router::new()
        .merge(api)
        .merge(ws)
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Backend running on http://0.0.0.0:3000");
    println!("🔌 WebSocket endpoint: ws://0.0.0.0:3000/ws");
    axum::serve(listener, app).await.unwrap();
}
