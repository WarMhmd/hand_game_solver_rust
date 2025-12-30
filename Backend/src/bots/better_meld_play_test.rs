#[cfg(test)]
mod tests {
    use crate::bot::BotStrategy;
    use crate::bots::better_meld_play::UseBetterMeldPlay;
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

    // Helper function to create a meld
    fn create_meld(cards: Vec<Card>, meld_type: MeldType) -> Meld {
        Meld {
            id: uuid::Uuid::new_v4().to_string(),
            cards,
            meld_type,
        }
    }

    // Helper function to create a basic RoundState for testing
    fn create_test_round_state(player: ActivePlayer, table_melds: Vec<Meld>) -> RoundState {
        RoundState {
            players: vec![player],
            current_player: 0,
            deck: Vec::new(),
            fire_pile: Vec::new(),
            table_melds,
            phase: Phase::PlayInMeld,
        }
    }

    #[test]
    fn test_decide_play_in_meld_with_table_melds() {
        // Test decide_play_in_meld with specific hand and table melds
        let mut bot = UseBetterMeldPlay::new("TestBot".to_string());

        let hand = vec![
            create_card("diamond-6", Suit::Diamonds, Rank::Number(6)),
            create_card("club-6", Suit::Clubs, Rank::Number(6)),
            create_card("club-8", Suit::Clubs, Rank::Number(8)),
            create_card("heart-3", Suit::Hearts, Rank::Number(3)),
            create_card("heart-2", Suit::Hearts, Rank::Number(2)),
        ];

        let table_melds = vec![
            // Meld 1: 5-5-5 (Rank meld)
            create_meld(
                vec![
                    create_card("heart-5", Suit::Hearts, Rank::Number(5)),
                    create_card("spade-5", Suit::Spades, Rank::Number(5)),
                    create_card("club-5", Suit::Clubs, Rank::Number(5)),
                ],
                MeldType::Rank,
            ),
            // Meld 2: H8-H9-H10 (Sequence)
            create_meld(
                vec![
                    create_card("heart-8", Suit::Hearts, Rank::Number(8)),
                    create_card("heart-9", Suit::Hearts, Rank::Number(9)),
                    create_card("heart-10", Suit::Hearts, Rank::Number(10)),
                ],
                MeldType::Sequence,
            ),
            // Meld 3: A-A-A-Joker (Rank meld with Joker)
            create_meld(
                vec![
                    create_card("club-A", Suit::Clubs, Rank::Ace),
                    create_card("heart-A", Suit::Hearts, Rank::Ace),
                    create_card("diamond-A", Suit::Diamonds, Rank::Ace),
                    create_card("joker-black", Suit::Joker, Rank::Joker),
                ],
                MeldType::Rank,
            ),
            // Meld 4: D7-D8-D9-D10 (Sequence)
            create_meld(
                vec![
                    create_card("diamond-7", Suit::Diamonds, Rank::Number(7)),
                    create_card("diamond-8", Suit::Diamonds, Rank::Number(8)),
                    create_card("diamond-9", Suit::Diamonds, Rank::Number(9)),
                    create_card("diamond-10", Suit::Diamonds, Rank::Number(10)),
                ],
                MeldType::Sequence,
            ),
            // Meld 5: SA-S2-S3-S4 (Sequence)
            create_meld(
                vec![
                    create_card("spade-A", Suit::Spades, Rank::Ace),
                    create_card("spade-2", Suit::Spades, Rank::Number(2)),
                    create_card("spade-3", Suit::Spades, Rank::Number(3)),
                    create_card("spade-4", Suit::Spades, Rank::Number(4)),
                ],
                MeldType::Sequence,
            ),
            // Meld 6: DQ-DK-Joker (Sequence with Joker)
            create_meld(
                vec![
                    create_card("diamond-Q", Suit::Diamonds, Rank::Queen),
                    create_card("diamond-K", Suit::Diamonds, Rank::King),
                    create_card("joker-red", Suit::Joker, Rank::Joker),
                ],
                MeldType::Sequence,
            ),
            // Meld 7: HJ-HQ-HK-HA (Sequence)
            create_meld(
                vec![
                    create_card("heart-J", Suit::Hearts, Rank::Jack),
                    create_card("heart-Q", Suit::Hearts, Rank::Queen),
                    create_card("heart-K", Suit::Hearts, Rank::King),
                    create_card("heart-A", Suit::Hearts, Rank::Ace),
                ],
                MeldType::Sequence,
            ),
            // Meld 8: C3-C4-C5-C6-C7 (Sequence)
            create_meld(
                vec![
                    create_card("club-3", Suit::Clubs, Rank::Number(3)),
                    create_card("club-4", Suit::Clubs, Rank::Number(4)),
                    create_card("club-5", Suit::Clubs, Rank::Number(5)),
                    create_card("club-6", Suit::Clubs, Rank::Number(6)),
                    create_card("club-7", Suit::Clubs, Rank::Number(7)),
                ],
                MeldType::Sequence,
            ),
        ];

        println!("\n=== Testing decide_play_in_meld ===");
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

        println!("\nTable melds ({} melds):", table_melds.len());
        for (i, meld) in table_melds.iter().enumerate() {
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
        }

        let player = create_test_player("P1", hand.clone(), true); // melded = true
        let state = create_test_round_state(player, table_melds);

        let (phase, card_opt, play_left, meld_index) = bot.decide_play_in_meld(&state);

        println!("\n=== Play in Meld Decision ===");
        println!("Phase: {:?}", phase);
        println!("Meld index: {}", meld_index);
        println!("Play left: {}", play_left);

        if let Some(card) = card_opt {
            println!(
                "Card to play: \"{}-{}\"",
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
            println!("Card ID: {}", card.id);
        } else {
            println!("No card to play (discard phase)");
        }

        println!("\n✓ Test completed - decision made");
    }

    #[test]
    fn test_decide_play_in_meld_not_melded() {
        // Test when player hasn't melded yet
        let mut bot = UseBetterMeldPlay::new("TestBot".to_string());

        let hand = vec![
            create_card("1", Suit::Hearts, Rank::Number(5)),
            create_card("2", Suit::Diamonds, Rank::Number(5)),
        ];

        let table_melds = vec![create_meld(
            vec![
                create_card("3", Suit::Hearts, Rank::Number(3)),
                create_card("4", Suit::Diamonds, Rank::Number(3)),
                create_card("5", Suit::Clubs, Rank::Number(3)),
            ],
            MeldType::Rank,
        )];

        let player = create_test_player("P1", hand.clone(), false); // melded = false
        let state = create_test_round_state(player, table_melds);

        let (phase, card_opt, _, _) = bot.decide_play_in_meld(&state);

        println!("\n=== Test decide_play_in_meld (not melded) ===");
        println!("Phase: {:?}", phase);
        println!("Should be Discard phase: {}", phase == Phase::Discard);
        println!("Card: {:?}", card_opt);

        assert_eq!(phase, Phase::Discard);
        assert!(card_opt.is_none());
    }

    #[test]
    fn test_decide_play_in_meld_one_card_left() {
        // Test when player has only one card left
        let mut bot = UseBetterMeldPlay::new("TestBot".to_string());

        let hand = vec![create_card("1", Suit::Hearts, Rank::Number(5))];

        let table_melds = vec![create_meld(
            vec![
                create_card("2", Suit::Hearts, Rank::Number(3)),
                create_card("3", Suit::Hearts, Rank::Number(4)),
                create_card("4", Suit::Hearts, Rank::Number(6)),
            ],
            MeldType::Sequence,
        )];

        let player = create_test_player("P1", hand.clone(), true); // melded = true
        let state = create_test_round_state(player, table_melds);

        let (phase, card_opt, _, _) = bot.decide_play_in_meld(&state);

        println!("\n=== Test decide_play_in_meld (one card left) ===");
        println!("Phase: {:?}", phase);
        println!("Should be Discard phase: {}", phase == Phase::Discard);

        assert_eq!(phase, Phase::Discard);
        assert!(card_opt.is_none());
    }
}
