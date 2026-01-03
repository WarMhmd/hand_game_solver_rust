#[cfg(test)]
mod tests {
    use crate::bot::BotStrategy;
    use crate::bots::better_discard::UseBetterDiscard;
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
    fn create_test_player(id: &str, hand: Vec<Card>, melded: bool) -> ActivePlayer {
        ActivePlayer {
            id: id.to_string(),
            name: format!("Player {}", id),
            bot_strategy: None,
            score: 0,
            hand,
            fire_card_id: None,
            melded,
            did_join: true,
            sender: None,
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
            phase: Phase::Discard,
        }
    }

    #[test]
    fn test_decide_discard_not_melded() {
        // Test decide_discard with specific hand when player is not melded
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-7", Suit::Hearts, Rank::Number(7)),
            create_card("heart-9", Suit::Hearts, Rank::Number(9)),
            create_card("spade-3", Suit::Spades, Rank::Number(3)),
            create_card("spade-8", Suit::Spades, Rank::Number(8)),
            create_card("spade-10", Suit::Spades, Rank::Number(10)),
            create_card("spade-Q", Suit::Spades, Rank::Queen),
            create_card("spade-A", Suit::Spades, Rank::Ace),
            create_card("diamond-Q", Suit::Diamonds, Rank::Queen),
            create_card("club-2", Suit::Clubs, Rank::Number(2)),
            create_card("club-4", Suit::Clubs, Rank::Number(4)),
            create_card("club-8", Suit::Clubs, Rank::Number(8)),
            create_card("club-9", Suit::Clubs, Rank::Number(9)),
            create_card("club-A", Suit::Clubs, Rank::Ace),
            create_card("spade-2", Suit::Spades, Rank::Number(2)),
            create_card("spade-7", Suit::Spades, Rank::Number(7)),
        ];

        println!("\n=== Testing decide_discard (not melded) ===");
        println!("Hand:");
        for (i, card) in hand.iter().enumerate() {
            println!(
                "  [{}] Card: \"{}-{}\"",
                i,
                format!("{:?}", card.suit).to_lowercase(),
                match card.rank {
                    Rank::Ace => "A".to_string(),
                    Rank::King => "K".to_string(),
                    Rank::Queen => "Q".to_string(),
                    Rank::Jack => "J".to_string(),
                    Rank::Number(n) => n.to_string(),
                    Rank::Joker => "Joker".to_string(),
                }
            );
        }

        let player = create_test_player("P1", hand.clone(), false);
        let state = create_test_round_state_with_player(player);

        // First, find melds to populate internal state
        let melds = bot.find_melds(&hand);
        println!("\nMelds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let discard_index = bot.decide_discard(&state);

        println!("\n=== Discard Decision ===");
        println!("Discard index: {}", discard_index);
        println!(
            "Discarded card: {} {}",
            hand[discard_index].rank, hand[discard_index].suit
        );
        println!("Card ID: {}", hand[discard_index].id);
        println!("Card penalty: {}", card_penality(&hand[discard_index]));

        // Verify it's a valid index
        assert!(discard_index < hand.len());
        println!("\n✓ Test completed - valid discard chosen");
    }

    #[test]
    fn test_decide_discard_melded() {
        // Test decide_discard when player is already melded
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-7", Suit::Hearts, Rank::Number(7)),
            create_card("heart-9", Suit::Hearts, Rank::Number(9)),
            create_card("spade-3", Suit::Spades, Rank::Number(3)),
            create_card("spade-8", Suit::Spades, Rank::Number(8)),
            create_card("spade-10", Suit::Spades, Rank::Number(10)),
            create_card("spade-Q", Suit::Spades, Rank::Queen),
            create_card("spade-A", Suit::Spades, Rank::Ace),
            create_card("diamond-Q", Suit::Diamonds, Rank::Queen),
            create_card("club-2", Suit::Clubs, Rank::Number(2)),
            create_card("club-4", Suit::Clubs, Rank::Number(4)),
            create_card("club-8", Suit::Clubs, Rank::Number(8)),
            create_card("club-9", Suit::Clubs, Rank::Number(9)),
            create_card("club-A", Suit::Clubs, Rank::Ace),
            create_card("spade-2", Suit::Spades, Rank::Number(2)),
            create_card("spade-7", Suit::Spades, Rank::Number(7)),
        ];

        println!("\n=== Testing decide_discard (already melded) ===");
        println!("Hand:");
        for (i, card) in hand.iter().enumerate() {
            println!(
                "  [{}] Card: \"{}-{}\"",
                i,
                format!("{:?}", card.suit).to_lowercase(),
                match card.rank {
                    Rank::Ace => "A".to_string(),
                    Rank::King => "K".to_string(),
                    Rank::Queen => "Q".to_string(),
                    Rank::Jack => "J".to_string(),
                    Rank::Number(n) => n.to_string(),
                    Rank::Joker => "Joker".to_string(),
                }
            );
        }

        let player = create_test_player("P1", hand.clone(), true); // melded = true
        let state = create_test_round_state_with_player(player);

        // First, find melds to populate internal state
        let melds = bot.find_melds(&hand);
        println!("\nMelds found: {}", melds.len());

        let discard_index = bot.decide_discard(&state);

        println!("\n=== Discard Decision ===");
        println!("Discard index: {}", discard_index);
        println!(
            "Discarded card: {} {}",
            hand[discard_index].rank, hand[discard_index].suit
        );
        println!("Card ID: {}", hand[discard_index].id);
        println!("Card penalty: {}", card_penality(&hand[discard_index]));

        // Verify it's a valid index
        assert!(discard_index < hand.len());
        println!("\n✓ Test completed - valid discard chosen");
    }

    #[test]
    fn test_decide_discard_simple_hand() {
        // Test with a simpler hand
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(2)),
            create_card("2", Suit::Diamonds, Rank::Number(5)),
            create_card("3", Suit::Clubs, Rank::Number(8)),
            create_card("4", Suit::Spades, Rank::King),
        ];

        println!("\n=== Testing decide_discard with simple hand ===");
        println!("Hand:");
        for card in &hand {
            println!("  {} {}", card.rank, card.suit);
        }

        let player = create_test_player("P1", hand.clone(), false);
        let state = create_test_round_state_with_player(player);

        bot.find_melds(&hand);
        let discard_index = bot.decide_discard(&state);

        println!("\nDiscard index: {}", discard_index);
        println!(
            "Discarded card: {} {}",
            hand[discard_index].rank, hand[discard_index].suit
        );

        assert!(discard_index < hand.len());
    }

    #[test]
    fn test_candidate_melds() {
        // Test the candidate_melds function directly
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(3)),
            create_card("2", Suit::Hearts, Rank::Number(4)),
            create_card("3", Suit::Hearts, Rank::Number(5)),
            create_card("4", Suit::Diamonds, Rank::Number(7)),
            create_card("5", Suit::Clubs, Rank::King),
        ];

        println!("\n=== Testing candidate_melds ===");
        println!("Hand:");
        for (i, card) in hand.iter().enumerate() {
            println!("  [{}] {} {}", i, card.rank, card.suit);
        }

        // Find melds first to populate internal state
        bot.find_melds(&hand);

        let candidates = bot.candidate_melds(&hand);

        println!("\nCandidate mask: {:05b}", candidates);
        println!("Candidate cards:");
        for i in 0..hand.len() {
            if candidates & (1 << i) != 0 {
                println!("  [{}] {} {}", i, hand[i].rank, hand[i].suit);
            }
        }
    }

    #[test]
    fn test_decide_discard_specific_hand_not_melded() {
        // Test decide_discard with the specific hand when player is not melded
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-4", Suit::Hearts, Rank::Number(4)),
            create_card("heart-5", Suit::Hearts, Rank::Number(5)),
            create_card("heart-K", Suit::Hearts, Rank::King),
            create_card("spade-3", Suit::Spades, Rank::Number(3)),
            create_card("spade-6", Suit::Spades, Rank::Number(6)),
            create_card("spade-A", Suit::Spades, Rank::Ace),
            create_card("diamond-5", Suit::Diamonds, Rank::Number(5)),
            create_card("diamond-6", Suit::Diamonds, Rank::Number(6)),
            create_card("diamond-A", Suit::Diamonds, Rank::Ace),
            create_card("club-2", Suit::Clubs, Rank::Number(2)),
            create_card("club-3", Suit::Clubs, Rank::Number(3)),
            create_card("club-6", Suit::Clubs, Rank::Number(6)),
            create_card("club-8", Suit::Clubs, Rank::Number(8)),
            create_card("club-J", Suit::Clubs, Rank::Jack),
            create_card("spade-5", Suit::Spades, Rank::Number(5)),
        ];

        println!("\n=== Testing decide_discard with specific hand (not melded) ===");
        println!("Hand:");
        for (i, card) in hand.iter().enumerate() {
            println!(
                "  [{}] Card: \"{}-{}\"",
                i,
                format!("{:?}", card.suit).to_lowercase(),
                match card.rank {
                    Rank::Ace => "A".to_string(),
                    Rank::King => "K".to_string(),
                    Rank::Queen => "Q".to_string(),
                    Rank::Jack => "J".to_string(),
                    Rank::Number(n) => n.to_string(),
                    Rank::Joker => "Joker".to_string(),
                }
            );
        }

        let player = create_test_player("P1", hand.clone(), false);
        let state = create_test_round_state_with_player(player);

        // First, find melds to populate internal state
        let melds = bot.find_melds(&hand);
        println!("\nMelds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let discard_index = bot.decide_discard(&state);

        println!("\n=== Discard Decision ===");
        println!("Discard index: {}", discard_index);
        println!(
            "Discarded card: {} {}",
            hand[discard_index].rank, hand[discard_index].suit
        );
        println!("Card ID: {}", hand[discard_index].id);
        println!("Card penalty: {}", card_penality(&hand[discard_index]));

        // Verify it's a valid index
        assert!(discard_index < hand.len());
        println!("\n✓ Test completed - valid discard chosen");
    }

    #[test]
    fn test_decide_discard_nine_card_hand_melded() {
        // Test decide_discard with 9-card hand when player is melded
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-K", Suit::Hearts, Rank::King),
            create_card("spade-10", Suit::Spades, Rank::Number(10)),
            create_card("spade-Q", Suit::Spades, Rank::Queen),
            create_card("diamond-6", Suit::Diamonds, Rank::Number(6)),
            create_card("diamond-7", Suit::Diamonds, Rank::Number(7)),
            create_card("club-5", Suit::Clubs, Rank::Number(5)),
            create_card("club-K", Suit::Clubs, Rank::King),
            create_card("club-4", Suit::Clubs, Rank::Number(4)),
            create_card("club-2", Suit::Clubs, Rank::Number(2)),
        ];

        println!("\n=== Testing decide_discard with 9-card hand (melded) ===");
        println!("Hand:");
        for (i, card) in hand.iter().enumerate() {
            println!(
                "  [{}] Card: \"{}-{}\"",
                i,
                format!("{:?}", card.suit).to_lowercase(),
                match card.rank {
                    Rank::Ace => "A".to_string(),
                    Rank::King => "K".to_string(),
                    Rank::Queen => "Q".to_string(),
                    Rank::Jack => "J".to_string(),
                    Rank::Number(n) => n.to_string(),
                    Rank::Joker => "Joker".to_string(),
                }
            );
        }

        let player = create_test_player("P1", hand.clone(), true); // melded = true
        let state = create_test_round_state_with_player(player);

        // First, find melds to populate internal state
        let melds = bot.find_melds(&hand);
        println!("\nMelds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let discard_index = bot.decide_discard(&state);

        println!("\n=== Discard Decision ===");
        println!("Discard index: {}", discard_index);
        println!(
            "Discarded card: {} {}",
            hand[discard_index].rank, hand[discard_index].suit
        );
        println!("Card ID: {}", hand[discard_index].id);
        println!("Card penalty: {}", card_penality(&hand[discard_index]));

        // Verify it's a valid index
        assert!(discard_index < hand.len());
        println!("\n✓ Test completed - valid discard chosen");
    }

    #[test]
    fn test_decide_discard_fifteen_card_hand_not_melded() {
        // Test decide_discard with 15-card hand when player is not melded
        let mut bot = UseBetterDiscard::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-7", Suit::Hearts, Rank::Number(7)),
            create_card("heart-9", Suit::Hearts, Rank::Number(9)),
            create_card("spade-7", Suit::Spades, Rank::Number(7)),
            create_card("diamond-9", Suit::Diamonds, Rank::Number(9)),
            create_card("diamond-J", Suit::Diamonds, Rank::Jack),
            create_card("diamond-A", Suit::Diamonds, Rank::Ace),
            create_card("club-6", Suit::Clubs, Rank::Number(6)),
            create_card("club-7", Suit::Clubs, Rank::Number(7)),
            create_card("club-8", Suit::Clubs, Rank::Number(8)),
            create_card("club-10", Suit::Clubs, Rank::Number(10)),
            create_card("diamond-4", Suit::Diamonds, Rank::Number(4)),
            create_card("spade-A", Suit::Spades, Rank::Ace),
            create_card("spade-K", Suit::Spades, Rank::King),
            create_card("spade-4", Suit::Spades, Rank::Number(4)),
            create_card("spade-9", Suit::Spades, Rank::Number(9)),
        ];

        println!("\n=== Testing decide_discard with 15-card hand (not melded) ===");
        println!("Hand:");
        for (i, card) in hand.iter().enumerate() {
            println!(
                "  [{}] Card: \"{}-{}\"",
                i,
                format!("{:?}", card.suit).to_lowercase(),
                match card.rank {
                    Rank::Ace => "A".to_string(),
                    Rank::King => "K".to_string(),
                    Rank::Queen => "Q".to_string(),
                    Rank::Jack => "J".to_string(),
                    Rank::Number(n) => n.to_string(),
                    Rank::Joker => "Joker".to_string(),
                }
            );
        }

        let player = create_test_player("P1", hand.clone(), false); // melded = false
        let state = create_test_round_state_with_player(player);

        // First, find melds to populate internal state
        let melds = bot.find_melds(&hand);
        println!("\nMelds found: {}", melds.len());
        for (i, meld) in melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!("  {} {}", card.rank, card.suit);
            }
        }

        let discard_index = bot.decide_discard(&state);

        println!("\n=== Discard Decision ===");
        println!("Discard index: {}", discard_index);
        println!(
            "Discarded card: {} {}",
            hand[discard_index].rank, hand[discard_index].suit
        );
        println!("Card ID: {}", hand[discard_index].id);
        println!("Card penalty: {}", card_penality(&hand[discard_index]));

        // Verify it's a valid index
        assert!(discard_index < hand.len());
        println!("\n✓ Test completed - valid discard chosen");
    }
}
