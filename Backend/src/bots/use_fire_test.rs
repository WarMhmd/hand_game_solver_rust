#[cfg(test)]
mod tests {
    use crate::bot::{BotStrategy, DecideDrawResult};
    use crate::bots::use_fire::UseFireBot;
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

    fn parse_hand(hand: &str) -> Vec<Card> {
        hand.split_whitespace()
            .enumerate()
            .map(|(idx, token)| parse_card_token(token, idx))
            .collect()
    }

    fn parse_card_token(token: &str, idx: usize) -> Card {
        if token.eq_ignore_ascii_case("joker") {
            return create_card(&format!("Joker-{}", idx), Suit::Joker, Rank::Joker);
        }

        let suit_ch = token
            .chars()
            .last()
            .expect("card token must have a suit char");
        let suit = match suit_ch {
            'H' | 'h' => Suit::Hearts,
            'D' | 'd' => Suit::Diamonds,
            'C' | 'c' => Suit::Clubs,
            'S' | 's' => Suit::Spades,
            _ => panic!("unknown suit char in token: {}", token),
        };

        let rank_str = &token[..token.len() - suit_ch.len_utf8()];
        let rank = match rank_str {
            "A" | "a" => Rank::Ace,
            "K" | "k" => Rank::King,
            "Q" | "q" => Rank::Queen,
            "J" | "j" => Rank::Jack,
            _ => {
                let n: i32 = rank_str
                    .parse()
                    .unwrap_or_else(|_| panic!("unknown rank in token: {}", token));
                Rank::Number(n)
            }
        };

        create_card(&format!("{}-{}", token, idx), suit, rank)
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

    #[test]
    fn test_use_joker_melds_value_matches_cpp_cases() {
        let cases: Vec<(&str, i32)> = vec![
            ("7S 5D 4C 4C 3S 7C JH 10H QH QH AH 6H 9C KH 4H", 61),
            ("2H 8S 4S 9D 3C 7C QH KH 2D AH 2H 6D 10H Joker 4D", 51),
            ("7C 4S QS 8C QH 3D AS 10S 6D QS AH KS 6D 2H JS", 61),
            ("9C 10S 5S 5C 9H JS AS 3D KC 9S QH Joker 2S 7C QS", 78),
            ("AH 7H 3H QD 4C 7H 3D 2D JH KH 10H 3C QH AH Joker", 70),
            ("4H 2C 3C 10S 9D Joker 6D AC KH Joker AS JS KS 5S 8H", 77),
            ("AC 10S QC 6H 10C 4C JC 7D 3D QH 2D 2D QH Joker AD", 67),
            ("9S 10D 7C 4D 9D Joker KC AD 6S 5S 7C Joker QH QD 2D", 60),
            ("QS KC 8D 5H JH Joker KC AH 6H 4H 2D 4D QS 7H 3H", 38),
        ];

        for (hand_str, expected_cpp) in cases {
            let mut bot = UseJokerBot::new("TestBot".to_string());
            let hand = parse_hand(hand_str);
            let melds = bot.find_melds(&hand);
            let total = melds_value(&melds);

            assert_eq!(
                total, expected_cpp,
                "melds_value mismatch for hand: {} (melds={:?})",
                hand_str, melds
            );
        }
    }
}
