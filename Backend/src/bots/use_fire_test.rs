#[cfg(test)]
mod tests {
    use crate::bot::{BotStrategy, DecideDrawResult};
    use crate::bots::use_fire::UseFireBot;
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
    fn create_test_round_state(
        player: ActivePlayer,
        fire_pile: Vec<Card>,
        phase: Phase,
    ) -> RoundState {
        RoundState {
            players: vec![player],
            current_player: 0,
            deck: Vec::new(),
            fire_pile,
            table_melds: Vec::new(),
            phase,
        }
    }

    #[test]
    fn test_decide_draw_with_fire_card() {
        // Test decide_draw with specific hand and fire card
        // Hand: heart-5, spade-3, spade-Q, spade-K, diamond-5, diamond-10, diamond-3, heart-6
        // Fire card: heart-4
        let mut bot = UseFireBot::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-5", Suit::Hearts, Rank::Number(5)),
            create_card("spade-3", Suit::Spades, Rank::Number(3)),
            create_card("spade-Q", Suit::Spades, Rank::Queen),
            create_card("spade-K", Suit::Spades, Rank::King),
            create_card("diamond-5", Suit::Diamonds, Rank::Number(5)),
            create_card("diamond-10", Suit::Diamonds, Rank::Number(10)),
            create_card("diamond-3", Suit::Diamonds, Rank::Number(3)),
            create_card("heart-6", Suit::Hearts, Rank::Number(6)),
        ];

        let fire_card = create_card("heart-4", Suit::Hearts, Rank::Number(4));
        let fire_pile = vec![fire_card.clone()];

        println!("\n=== Testing decide_draw with Fire Card ===");
        println!("Hand ({} cards):", hand.len());
        for card in hand.iter() {
            println!(
                "  Card: \"{}-{}\"",
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

        println!("\nFire Card:");
        println!(
            "  Card: \"{}-{}\"",
            format!("{:?}", fire_card.suit).to_lowercase(),
            match fire_card.rank {
                Rank::Ace => "A".to_string(),
                Rank::King => "K".to_string(),
                Rank::Queen => "Q".to_string(),
                Rank::Jack => "J".to_string(),
                Rank::Number(n) => n.to_string(),
                Rank::Joker => "Joker".to_string(),
            }
        );

        // Test when player is NOT melded (needs 51+ points)
        println!("\n--- Test Case 1: Player NOT melded ---");
        let player = create_test_player("P1", hand.clone(), false);
        let state = create_test_round_state(player, fire_pile.clone(), Phase::Draw);

        let result = bot.decide_draw(&state);
        println!("Decision: {:?}", result);
        println!("Max value found: {}", bot.base.max_value);
        println!("Max cards count: {}", bot.base.max_cards_count);

        if bot.base.max_cards_count > 0 {
            println!("\nMelds that would be formed:");
            let melds = bot.base.meld_cards.clone();
            for (i, meld) in melds.iter().enumerate() {
                println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
                for card in &meld.cards {
                    println!(
                        "  Card: \"{}-{}\"",
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
                println!("  Meld value: {}", meld_value(meld));
            }
            println!("\nTotal value: {}", melds_value(&melds));
        }

        match result {
            DecideDrawResult::Fire => println!("\n✓ Bot decided to draw from FIRE pile"),
            DecideDrawResult::Deck => println!("\n✓ Bot decided to draw from DECK"),
        }

        // Test when player IS melded (needs any points)
        println!("\n--- Test Case 2: Player IS melded ---");
        bot.base.reset_calc_values();
        bot.is_fire_card = false;

        let player_melded = create_test_player("P1", hand.clone(), true);
        let state_melded = create_test_round_state(player_melded, fire_pile.clone(), Phase::Draw);
        let use_joker_base = &mut bot.base;
        use_joker_base.is_melded = true;
        let result_melded = bot.decide_draw(&state_melded);
        println!("Decision: {:?}", result_melded);
        println!("Max value found: {}", bot.base.max_value);
        println!("Max cards count: {}", bot.base.max_cards_count);

        if bot.base.max_cards_count > 0 {
            println!("\nMelds that would be formed:");
            let melds = bot.base.meld_cards.clone();
            for (i, meld) in melds.iter().enumerate() {
                println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
                for card in &meld.cards {
                    println!(
                        "  Card: \"{}-{}\"",
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
                println!("  Meld value: {}", meld_value(meld));
            }
            println!("\nTotal value: {}", melds_value(&melds));
        }

        match result_melded {
            DecideDrawResult::Fire => println!("\n✓ Bot decided to draw from FIRE pile"),
            DecideDrawResult::Deck => println!("\n✓ Bot decided to draw from DECK"),
        }

        println!("\n✓ Test completed - all scenarios tested");
    }

    #[test]
    fn test_decide_draw_no_fire_pile() {
        // Test decide_draw when fire pile is empty
        let mut bot = UseFireBot::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-5", Suit::Hearts, Rank::Number(5)),
            create_card("spade-3", Suit::Spades, Rank::Number(3)),
        ];

        println!("\n=== Testing decide_draw with Empty Fire Pile ===");

        let player = create_test_player("P1", hand.clone(), false);
        let state = create_test_round_state(player, Vec::new(), Phase::Draw);

        let result = bot.decide_draw(&state);
        println!("Decision: {:?}", result);

        assert!(matches!(result, DecideDrawResult::Deck));
        println!("\n✓ Bot correctly chose DECK when fire pile is empty");
    }
}
