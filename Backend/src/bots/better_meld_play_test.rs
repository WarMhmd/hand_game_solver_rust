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

    #[test]
    fn test_decide_play_in_meld_complex_table() {
        // Test decide_play_in_meld with complex hand and extensive table melds
        let mut bot = UseBetterMeldPlay::new("TestBot".to_string());

        let hand = vec![
            create_card("spade-10", Suit::Spades, Rank::Number(10)),
            create_card("diamond-9", Suit::Diamonds, Rank::Number(9)),
            create_card("club-5", Suit::Clubs, Rank::Number(5)),
            create_card("club-6", Suit::Clubs, Rank::Number(6)),
            create_card("heart-Q", Suit::Hearts, Rank::Queen),
            create_card("diamond-Q", Suit::Diamonds, Rank::Queen),
            create_card("heart-Q-dup", Suit::Hearts, Rank::Queen),
        ];

        let table_melds = vec![
            // Meld 1: Four Aces (Rank meld)
            create_meld(
                vec![
                    create_card("heart-A", Suit::Hearts, Rank::Ace),
                    create_card("club-A", Suit::Clubs, Rank::Ace),
                    create_card("spade-A", Suit::Spades, Rank::Ace),
                    create_card("diamond-A", Suit::Diamonds, Rank::Ace),
                ],
                MeldType::Rank,
            ),
            // Meld 2: Four 7s (Rank meld)
            create_meld(
                vec![
                    create_card("spade-7", Suit::Spades, Rank::Number(7)),
                    create_card("diamond-7", Suit::Diamonds, Rank::Number(7)),
                    create_card("club-7", Suit::Clubs, Rank::Number(7)),
                    create_card("heart-7", Suit::Hearts, Rank::Number(7)),
                ],
                MeldType::Rank,
            ),
            // Meld 3: DJ-Joker-DK (Sequence with Joker)
            create_meld(
                vec![
                    create_card("diamond-J", Suit::Diamonds, Rank::Jack),
                    create_card("joker-black", Suit::Joker, Rank::Joker),
                    create_card("diamond-K", Suit::Diamonds, Rank::King),
                ],
                MeldType::Sequence,
            ),
            // Meld 4: C8-C9-C10-CJ-CQ (Clubs sequence)
            create_meld(
                vec![
                    create_card("club-8", Suit::Clubs, Rank::Number(8)),
                    create_card("club-9", Suit::Clubs, Rank::Number(9)),
                    create_card("club-10", Suit::Clubs, Rank::Number(10)),
                    create_card("club-J", Suit::Clubs, Rank::Jack),
                    create_card("club-Q", Suit::Clubs, Rank::Queen),
                ],
                MeldType::Sequence,
            ),
            // Meld 5: D8-D9-D10-DJ (Diamonds sequence)
            create_meld(
                vec![
                    create_card("diamond-8", Suit::Diamonds, Rank::Number(8)),
                    create_card("diamond-9-table", Suit::Diamonds, Rank::Number(9)),
                    create_card("diamond-10", Suit::Diamonds, Rank::Number(10)),
                    create_card("diamond-J-table", Suit::Diamonds, Rank::Jack),
                ],
                MeldType::Sequence,
            ),
            // Meld 6: DA-D2-D3 (Diamonds sequence)
            create_meld(
                vec![
                    create_card("diamond-A-table", Suit::Diamonds, Rank::Ace),
                    create_card("diamond-2", Suit::Diamonds, Rank::Number(2)),
                    create_card("diamond-3", Suit::Diamonds, Rank::Number(3)),
                ],
                MeldType::Sequence,
            ),
            // Meld 7: S8-S9-S10-SJ-SQ (Spades sequence)
            create_meld(
                vec![
                    create_card("spade-8", Suit::Spades, Rank::Number(8)),
                    create_card("spade-9", Suit::Spades, Rank::Number(9)),
                    create_card("spade-10-table", Suit::Spades, Rank::Number(10)),
                    create_card("spade-J", Suit::Spades, Rank::Jack),
                    create_card("spade-Q", Suit::Spades, Rank::Queen),
                ],
                MeldType::Sequence,
            ),
            // Meld 8: H8-Joker-H10 (Hearts sequence with Joker)
            create_meld(
                vec![
                    create_card("heart-8", Suit::Hearts, Rank::Number(8)),
                    create_card("joker-red", Suit::Joker, Rank::Joker),
                    create_card("heart-10", Suit::Hearts, Rank::Number(10)),
                ],
                MeldType::Sequence,
            ),
            // Meld 9: SQ-SK-SA (Spades sequence)
            create_meld(
                vec![
                    create_card("spade-Q-table", Suit::Spades, Rank::Queen),
                    create_card("spade-K", Suit::Spades, Rank::King),
                    create_card("spade-A-table", Suit::Spades, Rank::Ace),
                ],
                MeldType::Sequence,
            ),
            // Meld 10: HA-H2-H3 (Hearts sequence)
            create_meld(
                vec![
                    create_card("heart-A-table", Suit::Hearts, Rank::Ace),
                    create_card("heart-2", Suit::Hearts, Rank::Number(2)),
                    create_card("heart-3", Suit::Hearts, Rank::Number(3)),
                ],
                MeldType::Sequence,
            ),
            // Meld 11: H4-H5-H6 (Hearts sequence)
            create_meld(
                vec![
                    create_card("heart-4", Suit::Hearts, Rank::Number(4)),
                    create_card("heart-5", Suit::Hearts, Rank::Number(5)),
                    create_card("heart-6", Suit::Hearts, Rank::Number(6)),
                ],
                MeldType::Sequence,
            ),
        ];

        println!("\n=== Testing decide_play_in_meld with complex table ===");
        println!("Hand ({} cards):", hand.len());
        for (i, card) in hand.iter().enumerate() {
            println!(
                "  [{}] Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                i, card.id, card.suit, card.rank
            );
        }

        println!("\nTable melds ({} melds):", table_melds.len());
        for (i, meld) in table_melds.iter().enumerate() {
            println!("\nMeld {} ({:?}):", i + 1, meld.meld_type);
            for card in &meld.cards {
                println!(
                    "  Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                    card.id, card.suit, card.rank
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
                "Card to play: Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                card.id, card.suit, card.rank
            );
        } else {
            println!("No card to play (discard phase)");
        }

        println!("\n✓ Test completed - decision made");
    }

    #[test]
    fn test_decide_play_in_meld_use_joker_get_seq() {
        // Test decide_play_in_meld with joker in sequence melds
        // Hand: heart-2, spade-10, club-9, spade-8
        // Player is melded
        let mut bot = UseBetterMeldPlay::new("TestBot".to_string());

        let hand = vec![
            create_card("heart-2", Suit::Hearts, Rank::Number(2)),
            create_card("spade-10", Suit::Spades, Rank::Number(10)),
            create_card("club-9", Suit::Clubs, Rank::Number(9)),
            create_card("spade-8", Suit::Spades, Rank::Number(8)),
        ];

        let table_melds = vec![
            // Meld 1: DJ-CJ-SJ (Rank meld of Jacks)
            create_meld(
                vec![
                    create_card("diamond-J", Suit::Diamonds, Rank::Jack),
                    create_card("club-J", Suit::Clubs, Rank::Jack),
                    create_card("spade-J", Suit::Spades, Rank::Jack),
                ],
                MeldType::Rank,
            ),
            // Meld 2: C3-C4-C5-C6 (Clubs sequence)
            create_meld(
                vec![
                    create_card("club-3", Suit::Clubs, Rank::Number(3)),
                    create_card("club-4", Suit::Clubs, Rank::Number(4)),
                    create_card("club-5", Suit::Clubs, Rank::Number(5)),
                    create_card("club-6", Suit::Clubs, Rank::Number(6)),
                ],
                MeldType::Sequence,
            ),
            // Meld 3: D5-D6-Joker-D8 (Diamonds sequence with Joker as 7)
            create_meld(
                vec![
                    create_card("diamond-5", Suit::Diamonds, Rank::Number(5)),
                    create_card("diamond-6", Suit::Diamonds, Rank::Number(6)),
                    create_card("joker-red", Suit::Joker, Rank::Joker),
                    create_card("diamond-8", Suit::Diamonds, Rank::Number(8)),
                ],
                MeldType::Sequence,
            ),
            // Meld 4: C10-CJ-CQ-CK-CA (Clubs high sequence)
            create_meld(
                vec![
                    create_card("club-10", Suit::Clubs, Rank::Number(10)),
                    create_card("club-J", Suit::Clubs, Rank::Jack),
                    create_card("club-Q", Suit::Clubs, Rank::Queen),
                    create_card("club-K", Suit::Clubs, Rank::King),
                    create_card("club-A", Suit::Clubs, Rank::Ace),
                ],
                MeldType::Sequence,
            ),
            // Meld 5: DA-D2-D3 (Diamonds low sequence)
            create_meld(
                vec![
                    create_card("diamond-A", Suit::Diamonds, Rank::Ace),
                    create_card("diamond-2", Suit::Diamonds, Rank::Number(2)),
                    create_card("diamond-3", Suit::Diamonds, Rank::Number(3)),
                ],
                MeldType::Sequence,
            ),
            // Meld 6: HQ-HK-HA (Hearts high sequence)
            create_meld(
                vec![
                    create_card("heart-Q", Suit::Hearts, Rank::Queen),
                    create_card("heart-K", Suit::Hearts, Rank::King),
                    create_card("heart-A", Suit::Hearts, Rank::Ace),
                ],
                MeldType::Sequence,
            ),
            // Meld 7: H9-H10-HJ (Hearts sequence)
            create_meld(
                vec![
                    create_card("heart-9", Suit::Hearts, Rank::Number(9)),
                    create_card("heart-10", Suit::Hearts, Rank::Number(10)),
                    create_card("heart-J", Suit::Hearts, Rank::Jack),
                ],
                MeldType::Sequence,
            ),
            // Meld 8: SQ-HQ-CQ (Rank meld of Queens)
            create_meld(
                vec![
                    create_card("spade-Q", Suit::Spades, Rank::Queen),
                    create_card("heart-Q", Suit::Hearts, Rank::Queen),
                    create_card("club-Q", Suit::Clubs, Rank::Queen),
                ],
                MeldType::Rank,
            ),
            // Meld 9: DJ-Joker-DK-DA (Diamonds high sequence with Joker as Q)
            create_meld(
                vec![
                    create_card("diamond-J", Suit::Diamonds, Rank::Jack),
                    create_card("joker-black", Suit::Joker, Rank::Joker),
                    create_card("diamond-K", Suit::Diamonds, Rank::King),
                    create_card("diamond-A", Suit::Diamonds, Rank::Ace),
                ],
                MeldType::Sequence,
            ),
            // Meld 10: H6-H7-H8 (Hearts sequence)
            create_meld(
                vec![
                    create_card("heart-6", Suit::Hearts, Rank::Number(6)),
                    create_card("heart-7", Suit::Hearts, Rank::Number(7)),
                    create_card("heart-8", Suit::Hearts, Rank::Number(8)),
                ],
                MeldType::Sequence,
            ),
            // Meld 11: C7-C8-C9 (Clubs sequence)
            create_meld(
                vec![
                    create_card("club-7", Suit::Clubs, Rank::Number(7)),
                    create_card("club-8", Suit::Clubs, Rank::Number(8)),
                    create_card("club-9", Suit::Clubs, Rank::Number(9)),
                ],
                MeldType::Sequence,
            ),
        ];

        println!("\n=== Testing decide_play_in_meld with Joker in Sequence ===");
        println!("Hand:");
        for card in hand.iter() {
            println!(
                "  Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                card.id, card.suit, card.rank
            );
        }

        println!("\nTable Cards:");
        for (i, meld) in table_melds.iter().enumerate() {
            println!("\nCards:");
            for card in &meld.cards {
                println!(
                    "  Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                    card.id, card.suit, card.rank
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
                "Card to play: Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                card.id, card.suit, card.rank
            );
        } else {
            println!("No card to play (discard phase)");
        }

        println!("\n✓ Test completed - decision made");
    }

    #[test]
    fn test_decide_play_in_meld_joker_no_play() {
        // Test decide_play_in_meld with joker where bot should not play
        // Hand: spade-4, joker-red, club-Q (or spade-A in second scenario)
        // Player is melded
        let mut bot = UseBetterMeldPlay::new("TestBot".to_string());

        println!("\n=== Testing decide_play_in_meld with Joker - No Play Scenario ===");

        // Scenario 1: Hand with spade-4, joker-red, club-Q
        println!("\n--- Scenario 1: Hand with spade-4, joker-red, club-Q ---");
        let hand1 = vec![
            create_card("spade-4", Suit::Spades, Rank::Number(4)),
            create_card("joker-red", Suit::Joker, Rank::Joker),
            create_card("club-Q", Suit::Clubs, Rank::Queen),
        ];

        let table_melds1 = vec![
            // Meld 1: S7-C7-H7-D7 (Rank meld of 7s)
            create_meld(
                vec![
                    create_card("spade-7", Suit::Spades, Rank::Number(7)),
                    create_card("club-7", Suit::Clubs, Rank::Number(7)),
                    create_card("heart-7", Suit::Hearts, Rank::Number(7)),
                    create_card("diamond-7", Suit::Diamonds, Rank::Number(7)),
                ],
                MeldType::Rank,
            ),
            // Meld 2: CJ-CQ-CK (Clubs sequence)
            create_meld(
                vec![
                    create_card("club-J", Suit::Clubs, Rank::Jack),
                    create_card("club-Q", Suit::Clubs, Rank::Queen),
                    create_card("club-K", Suit::Clubs, Rank::King),
                ],
                MeldType::Sequence,
            ),
            // Meld 3: DK-HK-SK-Joker (Rank meld of Kings with Joker)
            create_meld(
                vec![
                    create_card("diamond-K", Suit::Diamonds, Rank::King),
                    create_card("heart-K", Suit::Hearts, Rank::King),
                    create_card("spade-K", Suit::Spades, Rank::King),
                    create_card("joker-black", Suit::Joker, Rank::Joker),
                ],
                MeldType::Rank,
            ),
            // Meld 4: S5-D5-C5 (Rank meld of 5s)
            create_meld(
                vec![
                    create_card("spade-5", Suit::Spades, Rank::Number(5)),
                    create_card("diamond-5", Suit::Diamonds, Rank::Number(5)),
                    create_card("club-5", Suit::Clubs, Rank::Number(5)),
                ],
                MeldType::Rank,
            ),
            // Meld 5: CA-C2-C3-C4 (Clubs low sequence)
            create_meld(
                vec![
                    create_card("club-A", Suit::Clubs, Rank::Ace),
                    create_card("club-2", Suit::Clubs, Rank::Number(2)),
                    create_card("club-3", Suit::Clubs, Rank::Number(3)),
                    create_card("club-4", Suit::Clubs, Rank::Number(4)),
                ],
                MeldType::Sequence,
            ),
            // Meld 6: SA-HA-DA-CA (Rank meld of Aces)
            create_meld(
                vec![
                    create_card("spade-A", Suit::Spades, Rank::Ace),
                    create_card("heart-A", Suit::Hearts, Rank::Ace),
                    create_card("diamond-A", Suit::Diamonds, Rank::Ace),
                    create_card("club-A", Suit::Clubs, Rank::Ace),
                ],
                MeldType::Rank,
            ),
            // Meld 7: H6-S6-D6-C6 (Rank meld of 6s)
            create_meld(
                vec![
                    create_card("heart-6", Suit::Hearts, Rank::Number(6)),
                    create_card("spade-6", Suit::Spades, Rank::Number(6)),
                    create_card("diamond-6", Suit::Diamonds, Rank::Number(6)),
                    create_card("club-6", Suit::Clubs, Rank::Number(6)),
                ],
                MeldType::Rank,
            ),
            // Meld 8: D8-D9-D10 (Diamonds sequence)
            create_meld(
                vec![
                    create_card("diamond-8", Suit::Diamonds, Rank::Number(8)),
                    create_card("diamond-9", Suit::Diamonds, Rank::Number(9)),
                    create_card("diamond-10", Suit::Diamonds, Rank::Number(10)),
                ],
                MeldType::Sequence,
            ),
            // Meld 9: D10-H10-S10-C10 (Rank meld of 10s)
            create_meld(
                vec![
                    create_card("diamond-10", Suit::Diamonds, Rank::Number(10)),
                    create_card("heart-10", Suit::Hearts, Rank::Number(10)),
                    create_card("spade-10", Suit::Spades, Rank::Number(10)),
                    create_card("club-10", Suit::Clubs, Rank::Number(10)),
                ],
                MeldType::Rank,
            ),
            // Meld 10: HK-SK-DK-CK (Rank meld of Kings)
            create_meld(
                vec![
                    create_card("heart-K", Suit::Hearts, Rank::King),
                    create_card("spade-K", Suit::Spades, Rank::King),
                    create_card("diamond-K", Suit::Diamonds, Rank::King),
                    create_card("club-K", Suit::Clubs, Rank::King),
                ],
                MeldType::Rank,
            ),
        ];

        println!("Hand:");
        for card in hand1.iter() {
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

        println!("\nTable Cards:");
        for (_i, meld) in table_melds1.iter().enumerate() {
            println!("\nCards:");
            for card in &meld.cards {
                println!(
                    "  Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                    card.id, card.suit, card.rank
                );
            }
        }

        let player1 = create_test_player("P1", hand1.clone(), true); // melded = true
        let state1 = create_test_round_state(player1, table_melds1);

        let (phase1, card_opt1, play_left1, meld_index1) = bot.decide_play_in_meld(&state1);

        println!("\n=== Play in Meld Decision ===");
        println!("Phase: {:?}", phase1);
        println!("Meld index: {}", meld_index1);
        println!("Play left: {}", play_left1);

        if let Some(ref card) = card_opt1 {
            println!(
                "Card to play: Card {{ id: \"{}\", suit: {:?}, rank: {:?} }}",
                card.id, card.suit, card.rank
            );
        } else {
            println!("No card to play (discard phase)");
        }
    }
}
