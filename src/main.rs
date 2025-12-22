mod logic;
mod bot;
mod bots {
    pub mod play_with_sequence;
    pub mod play_in_melds;
    pub mod use_joker;
    pub mod optimized_use_joker;
}

use crate::bot::{BotStrategy, RandomBot, RankBot, DecideDrawResult};
use crate::bots::optimized_use_joker::UseOptimizedJokerBot;
use crate::bots::play_in_melds::UseMeldBot;
use crate::bots::play_with_sequence::SequenceBot;
use crate::bots::use_joker::UseJokerBot;
use crate::logic::{init_game, init_round, is_round_over, draw_from_deck, draw_from_fire, discard_fire_card, lay_melds, play_in_meld, discard_card, score_round, Phase, Player};

#[derive(Clone)]
struct BotResult {
    name: String,
    total_score: i32,
    rounds_won: i32,
}

fn play_full_game(rounds: i32) {
    let mut players: Vec<Box<dyn BotStrategy>> = vec![
        Box::new(RandomBot::new("RandomBot".to_string())),
        Box::new(RandomBot::new("RandomBot-1".to_string())),
        Box::new(RandomBot::new("RandomBot-2".to_string())),
        Box::new(UseJokerBot::new("UseOptimizedjokerBot".to_string())),
    ];

    let mut results: Vec<BotResult> = players.iter().map(|p| BotResult {
        name: p.name().to_string(),
        total_score: 0,
        rounds_won: 0,
    }).collect();

    println!("=== HAND GAME SIMULATION START ===");

    // We can't move players into init_game and keep them in `players` var easily.
    // So we pass Option<Box>
    let player_names: Vec<String> = players.iter().map(|p| p.name().to_string()).collect();
    let strategies: Vec<Option<Box<dyn BotStrategy>>> = players.drain(..).map(|p| Some(p)).collect();

    let mut game_state = init_game(player_names, strategies);

    for r in 1..=rounds {
        let mut round_state = init_round(&mut game_state);
        println!("\n--- ROUND {} ---", r);
        let mut round_break = 0;

        while !is_round_over(&round_state) {
            // print all player cards (debug)
            if round_state.current_player == 3 {
                for p in &round_state.players {
                    if p.id == "P4" {
                        println!("Player {}:", p.id);
                        for c in &p.hand {
                            println!("{} {}", c.rank, c.suit);
                        }
                    }
                }
                println!("----------------");
            }

            round_break += 1;
            if round_break > 1000 {
                println!("Round timed out");
                break;
            }

            let current_player_idx = round_state.current_player;
            let mut draw_choice = DecideDrawResult::Deck;

            // Borrow strategy mutably
            let mut strategy = round_state.players[current_player_idx].bot_strategy.take();

            if let Some(ref mut s) = strategy {
                if round_state.phase == Phase::Draw {
                    draw_choice = s.decide_draw(&round_state);
                }
            }

            round_state.players[current_player_idx].bot_strategy = strategy;

            // DRAW PHASE
            if round_state.phase == Phase::Draw {
                 match draw_choice {
                     DecideDrawResult::Fire => {
                         if !round_state.fire_pile.is_empty() {
                             draw_from_fire(&mut round_state);
                         } else {
                             draw_from_deck(&mut round_state);
                         }
                     },
                     DecideDrawResult::Deck => draw_from_deck(&mut round_state),
                 }
                 round_state.phase = Phase::Meld;
            }

            // MELD PHASE
            if round_state.phase == Phase::Meld {
                let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                let mut melds = Vec::new();

                if let Some(ref mut s) = strategy {
                    melds = s.decide_melds(&round_state);
                }
                round_state.players[current_player_idx].bot_strategy = strategy;

                if !melds.is_empty() {
                    // lay_melds modifies round_state. Assume valid.
                    lay_melds(&mut round_state, melds.clone());

                    println!("==================");
                    println!("player {}: melded with cards", round_state.players[current_player_idx].id);
                    for (i, m) in melds.iter().enumerate() {
                         println!("Meld {}:", i + 1);
                         for c in &m.cards {
                             println!("{} {}", c.rank, c.suit);
                         }
                    }
                    println!("==================");
                    round_state.phase = Phase::PlayInMeld;
                } else {
                    round_state.phase = Phase::PlayInMeld;
                }
            }

            // PLAY IN MELD PHASE
            if round_state.phase == Phase::PlayInMeld {
                let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                let mut play_card = None;
                let mut play_index = -1;

                if let Some(ref mut s) = strategy {
                    if s.can_use_features().contains(&"useMeld".to_string()) {
                        let res = s.decide_play_in_meld(&round_state);
                        play_card = res.0;
                        play_index = res.1;
                    }
                }
                round_state.players[current_player_idx].bot_strategy = strategy;

                if play_card.is_none() || play_index == -1 {
                    round_state.phase = Phase::Discard;
                } else {
                    play_in_meld(&mut round_state, play_card.unwrap(), play_index as usize);
                }
            }

            // DISCARD PHASE
            if round_state.phase == Phase::Discard {
                let mut strategy = round_state.players[current_player_idx].bot_strategy.take();
                let mut discard_idx = 0;

                if let Some(ref mut s) = strategy {
                    discard_idx = s.decide_discard(&round_state);
                }
                round_state.players[current_player_idx].bot_strategy = strategy;

                discard_card(&mut round_state, discard_idx);
                round_state.phase = Phase::Draw;
            }
        }

        if round_break <= 1000 {
            let winner = round_state.players.iter().find(|p| p.hand.is_empty()).unwrap();
            let winner_name = winner.name.clone();
            let winner_id = winner.id.clone();
            println!("Round {} winner: {}", r, winner_name);

            score_round(&mut game_state, &mut round_state);

            // Update local results
            for p in &game_state.players {
                if let Some(bot) = results.iter_mut().find(|b| b.name == p.name) {
                    bot.total_score = p.score;
                    if p.id == winner_id {
                        bot.rounds_won += 1;
                    }
                }
            }

            // Print summary
            for p in &game_state.players {
                println!("{} | Score: {}", p.name, p.score);
            }
        } else {
            // Restore players if round timed out
            let mut players_back = Vec::new();
             for rp in round_state.players.drain(..) {
                 players_back.push(Player {
                     id: rp.id,
                     name: rp.name,
                     bot_strategy: rp.bot_strategy,
                     score: rp.score,
                 });
             }
             players_back.sort_by_key(|p| p.id.clone());
             game_state.players = players_back;
        }
    }

    println!("\n=== FINAL RESULTS ===");
    results.sort_by(|a, b| a.total_score.cmp(&b.total_score));

    for (i, r) in results.iter().enumerate() {
        println!("{}. {} | Total Score: {} | Rounds Won: {}", i + 1, r.name, r.total_score, r.rounds_won);
    }

    if !results.is_empty() {
        println!("\n🏆 WINNER: {}", results[0].name);
    }
}

fn main() {
    play_full_game(4);
}
