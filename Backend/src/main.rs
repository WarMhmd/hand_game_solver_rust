mod bot;
mod logic;
mod bots {
    pub mod play_in_melds;
    pub mod play_with_sequence;
    pub mod use_fire;
    pub mod use_joker;
}

use crate::bot::{BotStrategy, RankBot};
use crate::bots::play_in_melds::UseMeldBot;
use crate::bots::use_fire::UseFireBot;
use crate::bots::use_joker::UseJokerBot;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

// fn play_full_game(rounds: i32) {
//     let mut players: Vec<Box<dyn BotStrategy>> = vec![
//         Box::new(RankBot::new("RankBot".to_string())),
//         Box::new(UseMeldBot::new("UseMeldBot".to_string())),
//         Box::new(UseJokerBot::new("UseJokerBot".to_string())),
//         Box::new(UseFireBot::new("UseFireBot".to_string())),
//     ];
// }

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("🚀 Backend running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
