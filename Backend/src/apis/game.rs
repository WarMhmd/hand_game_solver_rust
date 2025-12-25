use crate::bot::RandomBot;
use crate::logic::{GameState, Player};
use crate::AppState;
use axum::{debug_handler, extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;

#[debug_handler]
pub async fn init_game(State(state): State<Arc<AppState>>) -> Json<GameState> {
    let players: Vec<Player> = vec![
        Player {
            id: Uuid::new_v4().into(),
            name: "Player 1".to_string(),
            bot_strategy: None,
            score: 0,
            sender: None,
            did_join: false,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: "Bot 1".to_string(),
            bot_strategy: Some(Box::new(RandomBot::new("Bot 1".into()))),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: "Bot 2".to_string(),
            bot_strategy: Some(Box::new(RandomBot::new("Bot 2".into()))),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: "Bot 3".to_string(),
            bot_strategy: Some(Box::new(RandomBot::new("Bot 3".into()))),
            score: 0,
            sender: None,
            did_join: true,
        },
    ];

    let game_state = GameState::new(players, 4);
    let game_id = game_state.id.clone();
    let result = Json(game_state.clone());

    let mut games = state.games.write().await;
    games.insert(game_id, Arc::new(tokio::sync::Mutex::new(game_state)));

    result
}
