use crate::bot::{BotStrategy, RandomBot, RankBot};
use crate::bots::better_discard::UseBetterDiscard;
use crate::bots::better_meld_play::UseBetterMeldPlay;
use crate::bots::play_in_melds::UseMeldBot;
use crate::bots::play_with_sequence::SequenceBot;
use crate::bots::use_fire::UseFireBot;
use crate::bots::use_joker::UseJokerBot;
use crate::logic::{GameState, Player};
use crate::AppState;
use axum::{debug_handler, extract::State, Json};
use rand::Rng;
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
            bot_strategy: Some(Box::new(UseBetterMeldPlay::new("fire 1".into()))),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: "Bot 2".to_string(),
            bot_strategy: Some(Box::new(UseBetterMeldPlay::new("Bot 2".into()))),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: "Bot 3".to_string(),
            bot_strategy: Some(Box::new(UseBetterMeldPlay::new("Bot 3".into()))),
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

fn get_strong_bot(bot_number: i32) -> (String, Box<dyn BotStrategy>) {
    let mut rng = rand::thread_rng();
    let bot_type = rng.gen_range(0..3);
    match bot_type {
        0 => (
            format!("UseBetterDiscard {}", bot_number),
            Box::new(UseBetterDiscard::new(format!(
                "UseBetterDiscard {}",
                bot_number
            ))),
        ),
        1 => (
            format!("UseBetterMeldPlay {}", bot_number),
            Box::new(UseBetterMeldPlay::new(format!(
                "UseBetterMeldPlay {}",
                bot_number
            ))),
        ),
        2 => (
            format!("UseFireBot {}", bot_number),
            Box::new(UseFireBot::new(format!("UseFireBot {}", bot_number))),
        ),
        _ => unreachable!(),
    }
}

#[debug_handler]
pub async fn init_game_with_random_strong_bots(
    State(state): State<Arc<AppState>>,
) -> Json<GameState> {
    let bot1 = get_strong_bot(1);
    let bot2 = get_strong_bot(2);
    let bot3 = get_strong_bot(3);
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
            name: bot1.0,
            bot_strategy: Some(bot1.1),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: bot2.0,
            bot_strategy: Some(bot2.1),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: bot3.0,
            bot_strategy: Some(bot3.1),
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

fn get_all_bot(bot_number: i32) -> (String, Box<dyn BotStrategy>) {
    let mut rng = rand::thread_rng();
    let bot_type = rng.gen_range(0..=6);
    match bot_type {
        0 => (
            format!("UseBetterDiscard-{}", bot_number),
            Box::new(UseBetterDiscard::new(format!(
                "UseBetterDiscard-{}",
                bot_number
            ))),
        ),
        1 => (
            format!("UseBetterMeldPlay-{}", bot_number),
            Box::new(UseBetterMeldPlay::new(format!(
                "UseBetterDiscard-{}",
                bot_number
            ))),
        ),
        2 => (
            format!("UseFireBot-{}", bot_number),
            Box::new(UseFireBot::new(format!("UseBetterDiscard-{}", bot_number))),
        ),
        3 => (
            format!("UseJokerBot-{}", bot_number),
            Box::new(UseJokerBot::new(format!("UseBetterDiscard-{}", bot_number))),
        ),
        4 => (
            format!("UseMeldBot-{}", bot_number),
            Box::new(UseMeldBot::new(format!("UseBetterDiscard-{}", bot_number))),
        ),
        5 => (
            format!("SequenceBot-{}", bot_number),
            Box::new(SequenceBot::new(format!("UseBetterDiscard-{}", bot_number))),
        ),
        6 => (
            format!("RankBot-{}", bot_number),
            Box::new(RankBot::new(format!("UseBetterDiscard-{}", bot_number))),
        ),
        _ => unreachable!(),
    }
}

#[debug_handler]
pub async fn init_game_with_random_all_bots(State(state): State<Arc<AppState>>) -> Json<GameState> {
    let bot1 = get_all_bot(1);
    let bot2 = get_all_bot(2);
    let bot3 = get_all_bot(3);
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
            name: bot1.0,
            bot_strategy: Some(bot1.1),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: bot2.0,
            bot_strategy: Some(bot2.1),
            score: 0,
            sender: None,
            did_join: true,
        },
        Player {
            id: Uuid::new_v4().into(),
            name: bot3.0,
            bot_strategy: Some(bot3.1),
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
