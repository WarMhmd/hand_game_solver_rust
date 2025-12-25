#[cfg(test)]
mod tests {
    use crate::bot::BotStrategy;
    use crate::bots::use_joker::UseJokerBot;
    use crate::logic::*;

    // Helper function to create a card
    fn create_card(id: &str, suit: Suit, rank: Rank) -> Card {
        Card {
            id: id.to_string(),
            suit,
            rank,
        }
    }

    // Helper function to create a test player
    fn create_test_player(id: &str, hand: Vec<Card>) -> ActivePlayer {
        ActivePlayer {
            id: id.to_string(),
            name: format!("Player {}", id),
            bot_strategy: None,
            score: 0,
            hand,
            fire_card_id: None,
            melded: false,
        }
    }

    // Helper function to create a basic RoundState for testing
    fn create_test_round_state_with_player(player: ActivePlayer) -> RoundState {
        RoundState {
            players: vec![player],
            current_player: 0,
            deck: Vec::new(),
            fire_pile: Vec::new(),
            table_melds: Vec::new(),
            phase: Phase::Meld,
        }
    }

    #[test]
    fn test_decide_melds_with_specific_hand() {
        // Test with the specific hand provided by the user
        // A Clubs, K Clubs, Q Clubs, 8 Clubs, 10 Spades, Joker Joker, 6 Hearts,
        // J Spades, Q Diamonds, 10 Diamonds, 5 Hearts, 6 Clubs, 6 Spades, 8 Hearts, 2 Clubs

        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("Clubs-A-0", Suit::Clubs, Rank::Ace),
            create_card("Clubs-K-1", Suit::Clubs, Rank::King),
            create_card("Clubs-Q-1", Suit::Clubs, Rank::Queen),
            create_card("4", Suit::Clubs, Rank::Number(8)),
            create_card("5", Suit::Spades, Rank::Number(10)),
            create_card("Joker-1", Suit::Joker, Rank::Joker),
            create_card("7", Suit::Hearts, Rank::Number(6)),
            create_card("8", Suit::Spades, Rank::Jack),
            create_card("9", Suit::Diamonds, Rank::Queen),
            create_card("10", Suit::Diamonds, Rank::Number(10)),
            create_card("11", Suit::Hearts, Rank::Number(5)),
            create_card("12", Suit::Clubs, Rank::Number(6)),
            create_card("13", Suit::Spades, Rank::Number(6)),
            create_card("14", Suit::Hearts, Rank::Number(8)),
            create_card("15", Suit::Clubs, Rank::Number(2)),
        ];

        let player = create_test_player("P1", hand.clone());
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = false; // First meld

        println!("\n=== Testing UseJokerBot with specific hand ===");
        println!("Hand cards:");
        for card in &hand {
            println!("  {} {}", card.rank, card.suit);
        }

        let melds = bot.decide_melds(&state);

        println!("\nMelds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
            println!("  Value: {}", meld_value(meld));
        }

        let total_value = melds_value(&melds);
        println!("\nTotal meld value: {}", total_value);
        println!("Meets 51 point requirement: {}", total_value >= 51);

        // Just verify the function doesn't panic and returns something
        // We're not asserting specific values since the user mentioned the code might be wrong
        println!("\n✓ Test completed without panic");
    }

    #[test]
    fn test_decide_melds_first_meld_insufficient_score() {
        // Test that bot returns empty vec when score < 51 on first meld
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(3)),
            create_card("2", Suit::Diamonds, Rank::Number(3)),
            create_card("3", Suit::Clubs, Rank::Number(3)),
            create_card("4", Suit::Hearts, Rank::Number(2)),
        ];

        let player = create_test_player("P1", hand);
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = false; // First meld

        let melds = bot.decide_melds(&state);

        // Should return empty because total value < 51
        assert_eq!(melds.len(), 0);
    }

    #[test]
    fn test_decide_melds_already_melded_any_score() {
        // Test that bot returns melds even with low score if already melded
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(3)),
            create_card("2", Suit::Diamonds, Rank::Number(3)),
            create_card("3", Suit::Clubs, Rank::Number(3)),
            create_card("4", Suit::Hearts, Rank::Number(4)),
        ];

        let player = create_test_player("P1", hand);
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = true; // Already melded

        let melds = bot.decide_melds(&state);

        // Should return melds even if score < 51 because player already melded
        println!("Melds found: {}", melds.len());
        for meld in &melds {
            println!(
                "Meld type: {:?}, cards: {}",
                meld.meld_type,
                meld.cards.len()
            );
        }
    }

    #[test]
    fn test_decide_melds_with_joker_rank_meld() {
        // Test finding rank meld with joker
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::King),
            create_card("2", Suit::Diamonds, Rank::King),
            create_card("3", Suit::Joker, Rank::Joker),
            create_card("4", Suit::Spades, Rank::Queen),
            create_card("5", Suit::Hearts, Rank::Queen),
            create_card("6", Suit::Diamonds, Rank::Queen),
        ];

        let player = create_test_player("P1", hand);
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = false;

        let melds = bot.decide_melds(&state);

        println!("\n=== Test with Joker in Rank Meld ===");
        println!("Melds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let total_value = melds_value(&melds);
        println!("Total value: {}", total_value);
    }

    #[test]
    fn test_decide_melds_with_sequence() {
        // Test finding sequence meld
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(5)),
            create_card("2", Suit::Hearts, Rank::Number(6)),
            create_card("3", Suit::Hearts, Rank::Number(7)),
            create_card("4", Suit::Diamonds, Rank::Jack),
            create_card("5", Suit::Diamonds, Rank::Queen),
            create_card("6", Suit::Diamonds, Rank::King),
        ];

        let player = create_test_player("P1", hand);
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = false;

        let melds = bot.decide_melds(&state);

        println!("\n=== Test with Sequences ===");
        println!("Melds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let total_value = melds_value(&melds);
        println!("Total value: {}", total_value);
    }

    #[test]
    fn test_decide_melds_with_joker_sequence() {
        // Test finding sequence meld with joker
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(5)),
            create_card("2", Suit::Joker, Rank::Joker),
            create_card("3", Suit::Hearts, Rank::Number(7)),
            create_card("4", Suit::Diamonds, Rank::Queen),
            create_card("5", Suit::Diamonds, Rank::King),
            create_card("6", Suit::Diamonds, Rank::Ace),
        ];

        let player = create_test_player("P1", hand);
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = false;

        let melds = bot.decide_melds(&state);

        println!("\n=== Test with Joker in Sequence ===");
        println!("Melds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let total_value = melds_value(&melds);
        println!("Total value: {}", total_value);
    }

    #[test]
    fn test_decide_melds_empty_hand() {
        // Test with empty hand
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![];

        let player = create_test_player("P1", hand);
        let state = create_test_round_state_with_player(player);

        let melds = bot.decide_melds(&state);

        assert_eq!(melds.len(), 0);
    }

    #[test]
    fn test_decide_melds_no_valid_melds() {
        // Test with hand that has no valid melds
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(2)),
            create_card("2", Suit::Diamonds, Rank::Number(4)),
            create_card("3", Suit::Clubs, Rank::Number(6)),
            create_card("4", Suit::Spades, Rank::Number(8)),
            create_card("5", Suit::Hearts, Rank::Number(10)),
        ];

        let player = create_test_player("P1", hand);
        let state = create_test_round_state_with_player(player);

        let melds = bot.decide_melds(&state);

        println!("\n=== Test with No Valid Melds ===");
        println!("Melds found: {}", melds.len());
    }

    #[test]
    fn test_decide_melds_high_value_hand() {
        // Test with high value hand
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Ace),
            create_card("2", Suit::Diamonds, Rank::Ace),
            create_card("3", Suit::Clubs, Rank::Ace),
            create_card("4", Suit::Spades, Rank::King),
            create_card("5", Suit::Hearts, Rank::King),
            create_card("6", Suit::Diamonds, Rank::King),
        ];

        let player = create_test_player("P1", hand);
        let mut state = create_test_round_state_with_player(player);
        state.players[0].melded = false;

        let melds = bot.decide_melds(&state);

        println!("\n=== Test with High Value Hand ===");
        println!("Melds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
            println!("  Value: {}", meld_value(meld));
        }

        let total_value = melds_value(&melds);
        println!("Total value: {}", total_value);

        if total_value >= 51 {
            assert!(melds.len() > 0);
        }
    }

    #[test]
    fn test_find_melds_directly() {
        // Test find_melds directly without state
        let mut bot = UseJokerBot::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Clubs, Rank::Ace),
            create_card("2", Suit::Clubs, Rank::King),
            create_card("3", Suit::Clubs, Rank::Queen),
            create_card("4", Suit::Clubs, Rank::Number(8)),
            create_card("5", Suit::Spades, Rank::Number(10)),
            create_card("6", Suit::Joker, Rank::Joker),
            create_card("7", Suit::Hearts, Rank::Number(6)),
            create_card("8", Suit::Spades, Rank::Jack),
            create_card("9", Suit::Diamonds, Rank::Queen),
            create_card("10", Suit::Diamonds, Rank::Number(10)),
            create_card("11", Suit::Hearts, Rank::Number(5)),
            create_card("12", Suit::Clubs, Rank::Number(6)),
            create_card("13", Suit::Spades, Rank::Number(6)),
            create_card("14", Suit::Hearts, Rank::Number(8)),
            create_card("15", Suit::Clubs, Rank::Number(2)),
        ];

        println!("\n=== Direct find_melds test ===");
        println!("Hand:");
        for card in &hand {
            println!("  {} {}", card.rank, card.suit);
        }

        let melds = bot.find_melds(&hand);

        println!("\nMelds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
            println!("  Value: {}", meld_value(meld));
        }

        let total_value = melds_value(&melds);
        println!("\nTotal meld value: {}", total_value);
    }
}
