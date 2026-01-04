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

use crate::bots::use_joker::UseJokerBot;
use crate::bots::better_discard::UseBetterDiscard;
use crate::logic::GameState;
use crate::logic::{Card, Rank, Suit, melds_value};
use crate::bot::BotStrategy;
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


// fn main() {
//     let mut bot: UseJokerBot = UseJokerBot::new("test_bot".to_string());
//     let args: Vec<String> = std::env::args().collect();

//     let cards = args[1].split(' ');
//     let mut hand = Vec::new();
//     for card in cards {
//         let rank_char = &card[0..card.len() - 1];
//         let suit_char = &card[card.len() - 1..];

//         let rank = match rank_char {
//             "A" => Rank::Ace,
//             "2" => Rank::Number(2),
//             "3" => Rank::Number(3),
//             "4" => Rank::Number(4),
//             "5" => Rank::Number(5),
//             "6" => Rank::Number(6),
//             "7" => Rank::Number(7),
//             "8" => Rank::Number(8),
//             "9" => Rank::Number(9),
//             "10" => Rank::Number(10),
//             "J" => Rank::Jack,
//             "Q" => Rank::Queen,
//             "K" => Rank::King,
//             "Joke" => Rank::Joker,
//             _ => panic!("Invalid rank"),
//         };

//         let suit = match suit_char {
//             "H" => Suit::Hearts,
//             "D" => Suit::Diamonds,
//             "C" => Suit::Clubs,
//             "S" => Suit::Spades,
//             "r" => Suit::Joker,
//             _ => panic!("Invalid suit"),
//         };

//         hand.push(Card { 
//             id: uuid::Uuid::new_v4().to_string(),
//             rank,
//             suit
//         });
//     }
//     let melds = bot.find_melds(&hand);
//     println!("{}", melds_value(&melds));
//     for meld in &melds {
//         if meld.meld_type == crate::logic::MeldType::Rank {
//             println!("Rank Meld: ");
//             for card in &meld.cards {
//                 println!("{:?} {:?} ", card.rank, card.suit);
//             }
//             println!();
//         }
//         if meld.meld_type == crate::logic::MeldType::Sequence {
//             println!("Sequence Meld: ");
//             for card in &meld.cards {
//                 println!("{:?} {:?} ", card.rank, card.suit);
//             }
//             println!();
//         }
//     }
// }

//  AH 7H 3H QD 4C 7H 3D 2D JH KH 10H 3C QH AH Joker
//  
// 
// 

#[tokio::main]
async fn main() {
    let state = AppState {
        games: Arc::new(RwLock::new(HashMap::new())),
        bots: Arc::new(RwLock::new(HashMap::new())),
        ack_trackers: Arc::new(RwLock::new(HashMap::new())),
    };

    let allowed_origins_str =
        "http://localhost:3001,https://www.jawaker.com,https://cdn.jawaker.com,https://hand.warmhmd.online";
    let allowed_origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .map(|s| HeaderValue::from_str(s.trim()).unwrap())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
        ])
        .allow_credentials(true)
        .expose_headers([http::header::CONTENT_TYPE, http::header::CONTENT_LENGTH]);

    async fn health_check() -> &'static str {
        "OK"
    }

    async fn root() -> &'static str {
        "Backend is running"
    }

    let api = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/v1/bot/init-bot", post(init_bot))
        .route("/api/v1/bot/draw-card", post(draw_card))
        .route("/api/v1/bot/meld-cards", post(meld_cards))
        .route("/api/v1/bot/play-in-melds", post(play_in_melds))
        .route("/api/v1/bot/discard", post(discard))
        .route("/api/game/init_game", post(init_game))
        .route(
            "/api/game/init_game_with_random_strong_bots",
            post(init_game_with_random_strong_bots),
        )
        .route(
            "/api/game/init_game_with_random_all_bots",
            post(init_game_with_random_all_bots),
        );

    let ws = Router::new().route("/ws", get(websocket_handler));

    // Apply CORS to the entire app, not just the API router
    let app = Router::new()
        .merge(api)
        .merge(ws)
        .layer(cors)
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Backend running on http://0.0.0.0:3000");
    println!("🔌 WebSocket endpoint: ws://0.0.0.0:3000/ws");
    axum::serve(listener, app).await.unwrap();
}
