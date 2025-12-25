#[cfg(test)]
mod tests {
    use crate::logic::*;

    // Helper function to create a card
    fn create_card(id: &str, suit: Suit, rank: Rank) -> Card {
        Card {
            id: id.to_string(),
            suit,
            rank,
        }
    }

    // Helper function to create a basic RoundState for testing
    fn create_test_round_state() -> RoundState {
        RoundState {
            players: Vec::new(),
            current_player: 0,
            deck: Vec::new(),
            fire_pile: Vec::new(),
            table_melds: Vec::new(),
            phase: Phase::Meld,
        }
    }

    #[test]
    fn test_check_melds_rank_meld_with_four_cards_moves_to_fire_pile() {
        // Test that a rank meld with 4 cards (no jokers) gets moved to fire pile
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(5)),
                create_card("2", Suit::Diamonds, Rank::Number(5)),
                create_card("3", Suit::Clubs, Rank::Number(5)),
                create_card("4", Suit::Spades, Rank::Number(5)),
            ],
            meld_type: MeldType::Rank,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        // The meld should be removed from table_melds
        assert_eq!(state.table_melds.len(), 0);

        // The cards should be in the fire pile (inserted at middle positions)
        assert_eq!(state.fire_pile.len(), 4);
    }

    #[test]
    fn test_check_melds_rank_meld_with_three_cards_stays() {
        // Test that a rank meld with 3 cards stays on the table
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(7)),
                create_card("2", Suit::Diamonds, Rank::Number(7)),
                create_card("3", Suit::Clubs, Rank::Number(7)),
            ],
            meld_type: MeldType::Rank,
        };

        state.table_melds.push(meld.clone());

        check_melds(&mut state);

        // The meld should still be on the table
        assert_eq!(state.table_melds.len(), 1);
        assert_eq!(state.table_melds[0].cards.len(), 3);
    }

    #[test]
    fn test_check_melds_rank_meld_with_four_cards_including_joker_stays() {
        // Test that a rank meld with 4 cards including a joker stays on the table
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(9)),
                create_card("2", Suit::Diamonds, Rank::Number(9)),
                create_card("3", Suit::Clubs, Rank::Number(9)),
                create_card("joker", Suit::Joker, Rank::Joker),
            ],
            meld_type: MeldType::Rank,
        };

        state.table_melds.push(meld.clone());

        check_melds(&mut state);

        // The meld should still be on the table (not moved to fire pile)
        assert_eq!(state.table_melds.len(), 1);
        assert_eq!(state.fire_pile.len(), 0);
    }

    #[test]
    fn test_check_melds_sequence_meld_with_six_cards_splits() {
        // Test that a sequence meld with 6 cards gets split into two 3-card sequences
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(2)),
                create_card("2", Suit::Hearts, Rank::Number(3)),
                create_card("3", Suit::Hearts, Rank::Number(4)),
                create_card("4", Suit::Hearts, Rank::Number(5)),
                create_card("5", Suit::Hearts, Rank::Number(6)),
                create_card("6", Suit::Hearts, Rank::Number(7)),
            ],
            meld_type: MeldType::Sequence,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        // The original meld should be removed and split into 2 melds
        assert_eq!(state.table_melds.len(), 2);
        assert_eq!(state.table_melds[0].cards.len(), 3);
        assert_eq!(state.table_melds[1].cards.len(), 3);

        // Check that the first meld contains the first 3 cards
        assert_eq!(state.table_melds[0].cards[0].rank, Rank::Number(2));
        assert_eq!(state.table_melds[0].cards[1].rank, Rank::Number(3));
        assert_eq!(state.table_melds[0].cards[2].rank, Rank::Number(4));

        // Check that the second meld contains the next 3 cards
        assert_eq!(state.table_melds[1].cards[0].rank, Rank::Number(5));
        assert_eq!(state.table_melds[1].cards[1].rank, Rank::Number(6));
        assert_eq!(state.table_melds[1].cards[2].rank, Rank::Number(7));
    }

    #[test]
    fn test_check_melds_sequence_meld_with_nine_cards_splits_into_three() {
        // Test that a sequence meld with 9 cards gets split into three 3-card sequences
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Clubs, Rank::Number(2)),
                create_card("2", Suit::Clubs, Rank::Number(3)),
                create_card("3", Suit::Clubs, Rank::Number(4)),
                create_card("4", Suit::Clubs, Rank::Number(5)),
                create_card("5", Suit::Clubs, Rank::Number(6)),
                create_card("6", Suit::Clubs, Rank::Number(7)),
                create_card("7", Suit::Clubs, Rank::Number(8)),
                create_card("8", Suit::Clubs, Rank::Number(9)),
                create_card("9", Suit::Clubs, Rank::Number(10)),
            ],
            meld_type: MeldType::Sequence,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        // Should be split into 3 melds of 3 cards each
        assert_eq!(state.table_melds.len(), 3);
        assert_eq!(state.table_melds[0].cards.len(), 3);
        assert_eq!(state.table_melds[1].cards.len(), 3);
        assert_eq!(state.table_melds[2].cards.len(), 3);
    }

    #[test]
    fn test_check_melds_sequence_meld_with_seven_cards_splits_unevenly() {
        // Test that a sequence meld with 7 cards gets split into 3-card and 4-card sequences
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Diamonds, Rank::Number(3)),
                create_card("2", Suit::Diamonds, Rank::Number(4)),
                create_card("3", Suit::Diamonds, Rank::Number(5)),
                create_card("4", Suit::Diamonds, Rank::Number(6)),
                create_card("5", Suit::Diamonds, Rank::Number(7)),
                create_card("6", Suit::Diamonds, Rank::Number(8)),
                create_card("7", Suit::Diamonds, Rank::Number(9)),
            ],
            meld_type: MeldType::Sequence,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        // Should be split into 2 melds: one with 3 cards, one with 4 cards
        assert_eq!(state.table_melds.len(), 2);
        assert_eq!(state.table_melds[0].cards.len(), 3);
        assert_eq!(state.table_melds[1].cards.len(), 4);
    }

    #[test]
    fn test_check_melds_sequence_meld_with_five_cards_stays() {
        // Test that a sequence meld with 5 cards stays on the table
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Spades, Rank::Number(4)),
                create_card("2", Suit::Spades, Rank::Number(5)),
                create_card("3", Suit::Spades, Rank::Number(6)),
                create_card("4", Suit::Spades, Rank::Number(7)),
                create_card("5", Suit::Spades, Rank::Number(8)),
            ],
            meld_type: MeldType::Sequence,
        };

        state.table_melds.push(meld.clone());

        check_melds(&mut state);

        // The meld should still be on the table (not split)
        assert_eq!(state.table_melds.len(), 1);
        assert_eq!(state.table_melds[0].cards.len(), 5);
    }

    #[test]
    fn test_check_melds_multiple_melds_mixed() {
        // Test with multiple melds of different types and sizes
        let mut state = create_test_round_state();

        // 4-card rank meld (should be removed to fire pile)
        let rank_meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Queen),
                create_card("2", Suit::Diamonds, Rank::Queen),
                create_card("3", Suit::Clubs, Rank::Queen),
                create_card("4", Suit::Spades, Rank::Queen),
            ],
            meld_type: MeldType::Rank,
        };

        // 6-card sequence meld (should be split)
        let seq_meld = Meld {
            cards: vec![
                create_card("5", Suit::Hearts, Rank::Number(5)),
                create_card("6", Suit::Hearts, Rank::Number(6)),
                create_card("7", Suit::Hearts, Rank::Number(7)),
                create_card("8", Suit::Hearts, Rank::Number(8)),
                create_card("9", Suit::Hearts, Rank::Number(9)),
                create_card("10", Suit::Hearts, Rank::Number(10)),
            ],
            meld_type: MeldType::Sequence,
        };

        // 3-card rank meld (should stay)
        let small_rank_meld = Meld {
            cards: vec![
                create_card("11", Suit::Hearts, Rank::King),
                create_card("12", Suit::Diamonds, Rank::King),
                create_card("13", Suit::Clubs, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        state.table_melds.push(rank_meld);
        state.table_melds.push(seq_meld);
        state.table_melds.push(small_rank_meld);

        check_melds(&mut state);

        // Should have: 2 split sequences + 1 small rank meld = 3 melds
        // The 4-card rank meld should be in fire pile
        assert_eq!(state.table_melds.len(), 3);
        assert_eq!(state.fire_pile.len(), 4);
    }

    #[test]
    fn test_check_melds_empty_table() {
        // Test with no melds on the table
        let mut state = create_test_round_state();

        check_melds(&mut state);

        assert_eq!(state.table_melds.len(), 0);
        assert_eq!(state.fire_pile.len(), 0);
    }

    #[test]
    fn test_check_melds_preserves_fire_pile_contents() {
        // Test that existing fire pile contents are preserved
        let mut state = create_test_round_state();

        // Add some cards to the fire pile
        state
            .fire_pile
            .push(create_card("existing1", Suit::Hearts, Rank::Ace));
        state
            .fire_pile
            .push(create_card("existing2", Suit::Diamonds, Rank::Number(2)));

        // Add a 4-card rank meld
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Jack),
                create_card("2", Suit::Diamonds, Rank::Jack),
                create_card("3", Suit::Clubs, Rank::Jack),
                create_card("4", Suit::Spades, Rank::Jack),
            ],
            meld_type: MeldType::Rank,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        // Should have original 2 cards + 4 new cards = 6 total
        assert_eq!(state.fire_pile.len(), 6);

        // Original cards should still be present
        assert!(state.fire_pile.iter().any(|c| c.id == "existing1"));
        assert!(state.fire_pile.iter().any(|c| c.id == "existing2"));
    }

    #[test]
    fn test_check_melds_sequence_with_exactly_six_cards() {
        // Edge case: exactly 6 cards should split into two groups of 3
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Clubs, Rank::Jack),
                create_card("2", Suit::Clubs, Rank::Queen),
                create_card("3", Suit::Clubs, Rank::King),
                create_card("4", Suit::Clubs, Rank::Ace),
                create_card("5", Suit::Clubs, Rank::Number(2)),
                create_card("6", Suit::Clubs, Rank::Number(3)),
            ],
            meld_type: MeldType::Sequence,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        assert_eq!(state.table_melds.len(), 2);
        assert_eq!(state.table_melds[0].cards.len(), 3);
        assert_eq!(state.table_melds[1].cards.len(), 3);

        // Verify both are sequence type
        assert_eq!(state.table_melds[0].meld_type, MeldType::Sequence);
        assert_eq!(state.table_melds[1].meld_type, MeldType::Sequence);
    }

    #[test]
    fn test_check_melds_invalid_rank_meld_stays_untouched() {
        // Test that invalid melds (that wouldn't pass validation) are still processed
        // Note: check_melds assumes melds are valid, but we can test edge cases
        let mut state = create_test_round_state();

        // Create a 4-card "meld" with same suit (invalid for rank meld in game rules)
        // but check_melds will still process it if valid_rank_meld returns false
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(3)),
                create_card("2", Suit::Hearts, Rank::Number(3)),
                create_card("3", Suit::Hearts, Rank::Number(3)),
                create_card("4", Suit::Hearts, Rank::Number(3)),
            ],
            meld_type: MeldType::Rank,
        };

        state.table_melds.push(meld);

        check_melds(&mut state);

        // Since valid_rank_meld will fail (same suit), it should stay on table
        assert_eq!(state.table_melds.len(), 1);
        assert_eq!(state.fire_pile.len(), 0);
    }

    // ==========================================
    // Tests for can_play_in_rank_meld
    // ==========================================

    #[test]
    fn test_can_play_in_rank_meld_add_fourth_card() {
        // Test adding a fourth card to a 3-card rank meld
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(8)),
                create_card("2", Suit::Diamonds, Rank::Number(8)),
                create_card("3", Suit::Clubs, Rank::Number(8)),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("4", Suit::Spades, Rank::Number(8));
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_rank_meld_cannot_add_fifth_card() {
        // Test that we cannot add a fifth card to a 4-card rank meld
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Queen),
                create_card("2", Suit::Diamonds, Rank::Queen),
                create_card("3", Suit::Clubs, Rank::Queen),
                create_card("4", Suit::Spades, Rank::Queen),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("5", Suit::Hearts, Rank::Queen);
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_rank_meld_wrong_rank() {
        // Test that we cannot add a card with different rank
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(5)),
                create_card("2", Suit::Diamonds, Rank::Number(5)),
                create_card("3", Suit::Clubs, Rank::Number(5)),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("4", Suit::Spades, Rank::Number(6));
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_rank_meld_duplicate_suit() {
        // Test that we cannot add a card with duplicate suit
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::King),
                create_card("2", Suit::Diamonds, Rank::King),
                create_card("3", Suit::Clubs, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("4", Suit::Hearts, Rank::King);
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_rank_meld_replace_joker() {
        // Test replacing a joker in a 4-card rank meld
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(7)),
                create_card("2", Suit::Diamonds, Rank::Number(7)),
                create_card("3", Suit::Clubs, Rank::Number(7)),
                create_card("joker", Suit::Joker, Rank::Joker),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("4", Suit::Spades, Rank::Number(7));
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(can_play);
        assert!(take_joker);
    }

    #[test]
    fn test_can_play_in_rank_meld_cannot_add_second_joker() {
        // Test that we cannot add a second joker to a meld with joker
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(9)),
                create_card("2", Suit::Diamonds, Rank::Number(9)),
                create_card("joker1", Suit::Joker, Rank::Joker),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("joker2", Suit::Joker, Rank::Joker);
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_rank_meld_joker_in_three_card_meld() {
        // Test adding a joker to a 3-card rank meld
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Ace),
                create_card("2", Suit::Diamonds, Rank::Ace),
                create_card("3", Suit::Clubs, Rank::Ace),
            ],
            meld_type: MeldType::Rank,
        };

        let card = create_card("joker", Suit::Joker, Rank::Joker);
        let (can_play, take_joker) = can_play_in_rank_meld(&meld, &card);

        assert!(can_play);
        assert!(!take_joker);
    }

    // ==========================================
    // Tests for can_play_in_sequence_meld
    // ==========================================

    #[test]
    fn test_can_play_in_sequence_meld_add_to_beginning() {
        // Test adding a card to the beginning of a sequence
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(5)),
                create_card("2", Suit::Hearts, Rank::Number(6)),
                create_card("3", Suit::Hearts, Rank::Number(7)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("4", Suit::Hearts, Rank::Number(4));
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_add_to_end() {
        // Test adding a card to the end of a sequence
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Diamonds, Rank::Number(3)),
                create_card("2", Suit::Diamonds, Rank::Number(4)),
                create_card("3", Suit::Diamonds, Rank::Number(5)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("4", Suit::Diamonds, Rank::Number(6));
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_wrong_suit() {
        // Test that we cannot add a card with different suit
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Clubs, Rank::Number(8)),
                create_card("2", Suit::Clubs, Rank::Number(9)),
                create_card("3", Suit::Clubs, Rank::Number(10)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("4", Suit::Hearts, Rank::Number(7));
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_wrong_rank() {
        // Test that we cannot add a card that doesn't continue the sequence
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Spades, Rank::Number(4)),
                create_card("2", Suit::Spades, Rank::Number(5)),
                create_card("3", Suit::Spades, Rank::Number(6)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("4", Suit::Spades, Rank::Number(9));
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_replace_joker() {
        // Test replacing a joker in a sequence
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Number(2)),
                create_card("joker", Suit::Joker, Rank::Joker),
                create_card("3", Suit::Hearts, Rank::Number(4)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("2", Suit::Hearts, Rank::Number(3));
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(can_play);
        assert!(take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_cannot_add_second_joker() {
        // Test that we cannot add a second joker to a sequence with joker
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Diamonds, Rank::Number(7)),
                create_card("joker1", Suit::Joker, Rank::Joker),
                create_card("3", Suit::Diamonds, Rank::Number(9)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("joker2", Suit::Joker, Rank::Joker);
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(!can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_joker_at_beginning() {
        // Test adding a joker at the beginning of a sequence
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Clubs, Rank::Number(5)),
                create_card("2", Suit::Clubs, Rank::Number(6)),
                create_card("3", Suit::Clubs, Rank::Number(7)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("joker", Suit::Joker, Rank::Joker);
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_joker_at_end() {
        // Test adding a joker at the end of a sequence
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Spades, Rank::Number(8)),
                create_card("2", Suit::Spades, Rank::Number(9)),
                create_card("3", Suit::Spades, Rank::Number(10)),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("joker", Suit::Joker, Rank::Joker);
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, true);

        assert!(can_play);
        assert!(!take_joker);
    }

    #[test]
    fn test_can_play_in_sequence_meld_with_face_cards() {
        // Test adding to a sequence with face cards
        let meld = Meld {
            cards: vec![
                create_card("1", Suit::Hearts, Rank::Jack),
                create_card("2", Suit::Hearts, Rank::Queen),
                create_card("3", Suit::Hearts, Rank::King),
            ],
            meld_type: MeldType::Sequence,
        };

        let card = create_card("4", Suit::Hearts, Rank::Ace);
        let (can_play, take_joker) = can_play_in_sequence_meld(&meld, &card, false);

        assert!(can_play);
        assert!(!take_joker);
    }

    // ==========================================
    // Tests for play_in_meld
    // ==========================================

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

    #[test]
    fn test_play_in_meld_rank_meld_add_card() {
        // Test adding a card to a rank meld (will complete to 4 and move to fire pile)
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::Number(6)),
                create_card("m2", Suit::Diamonds, Rank::Number(6)),
                create_card("m3", Suit::Clubs, Rank::Number(6)),
            ],
            meld_type: MeldType::Rank,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Spades, Rank::Number(6));
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Hearts, Rank::Number(3)),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // The 4-card rank meld (no jokers) should be moved to fire pile by check_melds
        assert_eq!(state.table_melds.len(), 0);
        assert_eq!(state.fire_pile.len(), 4);
        // Card should be removed from hand
        assert_eq!(state.players[0].hand.len(), 1);
        assert_eq!(state.players[0].hand[0].id, "p2");
    }

    #[test]
    fn test_play_in_meld_rank_meld_replace_joker() {
        // Test replacing a joker in a rank meld (will complete to 4 no-joker cards and move to fire)
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::Queen),
                create_card("m2", Suit::Diamonds, Rank::Queen),
                create_card("m3", Suit::Clubs, Rank::Queen),
                create_card("joker", Suit::Joker, Rank::Joker),
            ],
            meld_type: MeldType::Rank,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Spades, Rank::Queen);
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Hearts, Rank::Number(5)),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // The 4-card rank meld (no jokers after replacement) moves to fire pile
        assert_eq!(state.table_melds.len(), 0);
        assert_eq!(state.fire_pile.len(), 4);
        // Player should have joker in hand now
        assert_eq!(state.players[0].hand.len(), 2);
        assert!(state.players[0].hand.iter().any(|c| c.rank == Rank::Joker));
    }

    #[test]
    fn test_play_in_meld_sequence_meld_add_to_end() {
        // Test adding a card to the end of a sequence meld
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::Number(4)),
                create_card("m2", Suit::Hearts, Rank::Number(5)),
                create_card("m3", Suit::Hearts, Rank::Number(6)),
            ],
            meld_type: MeldType::Sequence,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Hearts, Rank::Number(7));
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Diamonds, Rank::Number(2)),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // Card should be added to meld
        assert_eq!(state.table_melds[0].cards.len(), 4);
        // Card should be removed from hand
        assert_eq!(state.players[0].hand.len(), 1);
    }

    #[test]
    fn test_play_in_meld_sequence_meld_replace_joker() {
        // Test replacing a joker in a sequence meld
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Clubs, Rank::Number(8)),
                create_card("joker", Suit::Joker, Rank::Joker),
                create_card("m3", Suit::Clubs, Rank::Number(10)),
            ],
            meld_type: MeldType::Sequence,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Clubs, Rank::Number(9));
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Hearts, Rank::Ace),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // Meld should still have 3 cards (joker replaced)
        assert_eq!(state.table_melds[0].cards.len(), 3);
        // Player should have joker in hand now
        assert_eq!(state.players[0].hand.len(), 2);
        assert!(state.players[0].hand.iter().any(|c| c.rank == Rank::Joker));
        assert_eq!(
            state.players[0]
                .hand
                .iter()
                .find(|c| c.rank == Rank::Joker)
                .unwrap()
                .id,
            "joker"
        );
    }

    #[test]
    #[should_panic(expected = "Meld not found")]
    fn test_play_in_meld_invalid_meld_index() {
        // Test that playing in non-existent meld panics
        let mut state = create_test_round_state();

        let card_to_play = create_card("p1", Suit::Hearts, Rank::Number(5));
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Diamonds, Rank::Number(3)),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0); // No melds exist
    }

    #[test]
    #[should_panic(expected = "Card not in hand")]
    fn test_play_in_meld_card_not_in_hand() {
        // Test that playing a card not in hand panics
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::King),
                create_card("m2", Suit::Diamonds, Rank::King),
                create_card("m3", Suit::Clubs, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Spades, Rank::King);
        let player_hand = vec![create_card("p2", Suit::Hearts, Rank::Number(3))];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);
    }

    #[test]
    #[should_panic(expected = "Cannot play last card in hand")]
    fn test_play_in_meld_last_card_in_hand() {
        // Test that playing the last card in hand panics
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::Jack),
                create_card("m2", Suit::Diamonds, Rank::Jack),
                create_card("m3", Suit::Clubs, Rank::Jack),
            ],
            meld_type: MeldType::Rank,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Spades, Rank::Jack);
        let player_hand = vec![card_to_play.clone()];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);
    }

    #[test]
    fn test_play_in_meld_triggers_check_melds() {
        // Test that play_in_meld triggers check_melds for sequences >= 6 cards
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Diamonds, Rank::Number(2)),
                create_card("m2", Suit::Diamonds, Rank::Number(3)),
                create_card("m3", Suit::Diamonds, Rank::Number(4)),
                create_card("m4", Suit::Diamonds, Rank::Number(5)),
                create_card("m5", Suit::Diamonds, Rank::Number(6)),
            ],
            meld_type: MeldType::Sequence,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Diamonds, Rank::Number(7));
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Hearts, Rank::Ace),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // check_melds should split the 6-card sequence into two 3-card sequences
        assert_eq!(state.table_melds.len(), 2);
        assert_eq!(state.table_melds[0].cards.len(), 3);
        assert_eq!(state.table_melds[1].cards.len(), 3);
    }

    #[test]
    fn test_play_in_meld_rank_becomes_four_cards_moves_to_fire() {
        // Test that completing a 4-card rank meld (no jokers) moves it to fire pile
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::Number(10)),
                create_card("m2", Suit::Diamonds, Rank::Number(10)),
                create_card("m3", Suit::Clubs, Rank::Number(10)),
            ],
            meld_type: MeldType::Rank,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Spades, Rank::Number(10));
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Hearts, Rank::Number(2)),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // Meld should be moved to fire pile
        assert_eq!(state.table_melds.len(), 0);
        assert_eq!(state.fire_pile.len(), 4);
    }

    #[test]
    fn test_play_in_meld_invalid_play_does_nothing() {
        // Test that an invalid play doesn't modify the state
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("m1", Suit::Hearts, Rank::Number(7)),
                create_card("m2", Suit::Diamonds, Rank::Number(7)),
                create_card("m3", Suit::Clubs, Rank::Number(7)),
            ],
            meld_type: MeldType::Rank,
        };
        state.table_melds.push(meld);

        let card_to_play = create_card("p1", Suit::Spades, Rank::Number(8)); // Wrong rank
        let player_hand = vec![
            card_to_play.clone(),
            create_card("p2", Suit::Hearts, Rank::Number(2)),
        ];
        state.players.push(create_test_player("P1", player_hand));

        play_in_meld(&mut state, card_to_play, 0);

        // State should remain unchanged
        assert_eq!(state.table_melds[0].cards.len(), 3);
        assert_eq!(state.players[0].hand.len(), 2); // Card still in hand
    }

    // ==========================================
    // Tests for lay_melds
    // ==========================================

    #[test]
    fn test_lay_melds_first_meld_valid_score() {
        // Test laying first meld with total score >= 51
        let mut state = create_test_round_state();

        let meld1 = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::King),   // 10
                create_card("p2", Suit::Diamonds, Rank::King), // 10
                create_card("p3", Suit::Clubs, Rank::King),    // 10
            ],
            meld_type: MeldType::Rank,
        };

        let meld2 = Meld {
            cards: vec![
                create_card("p4", Suit::Spades, Rank::Queen),   // 10
                create_card("p5", Suit::Hearts, Rank::Queen),   // 10
                create_card("p6", Suit::Diamonds, Rank::Queen), // 10
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::King),
            create_card("p2", Suit::Diamonds, Rank::King),
            create_card("p3", Suit::Clubs, Rank::King),
            create_card("p4", Suit::Spades, Rank::Queen),
            create_card("p5", Suit::Hearts, Rank::Queen),
            create_card("p6", Suit::Diamonds, Rank::Queen),
            create_card("p7", Suit::Hearts, Rank::Number(2)),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false; // First meld
        state.players.push(player);

        let melds = vec![meld1, meld2];
        lay_melds(&mut state, melds);

        // Melds should be on table
        assert_eq!(state.table_melds.len(), 2);
        // Cards should be removed from hand
        assert_eq!(state.players[0].hand.len(), 1);
        assert_eq!(state.players[0].hand[0].id, "p7");
        // Player should be marked as melded
        assert!(state.players[0].melded);
    }

    #[test]
    #[should_panic(expected = "Total meld score must be >= 51")]
    fn test_lay_melds_first_meld_insufficient_score() {
        // Test that first meld with score < 51 panics
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::Number(3)), // 3
                create_card("p2", Suit::Diamonds, Rank::Number(3)), // 3
                create_card("p3", Suit::Clubs, Rank::Number(3)),  // 3
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::Number(3)),
            create_card("p2", Suit::Diamonds, Rank::Number(3)),
            create_card("p3", Suit::Clubs, Rank::Number(3)),
            create_card("p4", Suit::Hearts, Rank::Number(2)),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false; // First meld
        state.players.push(player);

        let melds = vec![meld];
        lay_melds(&mut state, melds); // Should panic - total is only 9
    }

    #[test]
    fn test_lay_melds_subsequent_meld_no_score_requirement() {
        // Test that subsequent melds don't need to meet 51 point requirement
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::Number(4)),
                create_card("p2", Suit::Diamonds, Rank::Number(4)),
                create_card("p3", Suit::Clubs, Rank::Number(4)),
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::Number(4)),
            create_card("p2", Suit::Diamonds, Rank::Number(4)),
            create_card("p3", Suit::Clubs, Rank::Number(4)),
            create_card("p4", Suit::Hearts, Rank::Ace),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = true; // Already melded
        state.players.push(player);

        let melds = vec![meld];
        lay_melds(&mut state, melds);

        // Should succeed even though score is only 12
        assert_eq!(state.table_melds.len(), 1);
        assert_eq!(state.players[0].hand.len(), 1);
    }

    #[test]
    fn test_lay_melds_with_fire_card() {
        // Test laying melds that include the fire card
        let mut state = create_test_round_state();

        let fire_card = create_card("fire", Suit::Hearts, Rank::King);

        let meld1 = Meld {
            cards: vec![
                fire_card.clone(),
                create_card("p2", Suit::Diamonds, Rank::King),
                create_card("p3", Suit::Clubs, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        let meld2 = Meld {
            cards: vec![
                create_card("p4", Suit::Spades, Rank::Queen),
                create_card("p5", Suit::Hearts, Rank::Queen),
                create_card("p6", Suit::Diamonds, Rank::Queen),
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            fire_card.clone(),
            create_card("p2", Suit::Diamonds, Rank::King),
            create_card("p3", Suit::Clubs, Rank::King),
            create_card("p4", Suit::Spades, Rank::Queen),
            create_card("p5", Suit::Hearts, Rank::Queen),
            create_card("p6", Suit::Diamonds, Rank::Queen),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false;
        player.fire_card_id = Some("fire".to_string());
        state.players.push(player);

        let melds = vec![meld1, meld2];
        lay_melds(&mut state, melds);

        // Should succeed
        assert_eq!(state.table_melds.len(), 2);
        assert_eq!(state.players[0].hand.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Fire card not used in meld")]
    fn test_lay_melds_fire_card_not_used() {
        // Test that laying melds without fire card panics when fire card exists
        let mut state = create_test_round_state();

        let meld1 = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::King),
                create_card("p2", Suit::Diamonds, Rank::King),
                create_card("p3", Suit::Clubs, Rank::King),
                create_card("p4", Suit::Spades, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        let meld2 = Meld {
            cards: vec![
                create_card("p5", Suit::Hearts, Rank::Ace),
                create_card("p6", Suit::Diamonds, Rank::Ace),
                create_card("p7", Suit::Clubs, Rank::Ace),
            ],
            meld_type: MeldType::Rank,
        };

        let fire_card = create_card("fire", Suit::Hearts, Rank::Queen);
        let player_hand = vec![
            fire_card.clone(),
            create_card("p1", Suit::Hearts, Rank::King),
            create_card("p2", Suit::Diamonds, Rank::King),
            create_card("p3", Suit::Clubs, Rank::King),
            create_card("p4", Suit::Spades, Rank::King),
            create_card("p5", Suit::Hearts, Rank::Ace),
            create_card("p6", Suit::Diamonds, Rank::Ace),
            create_card("p7", Suit::Clubs, Rank::Ace),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false;
        player.fire_card_id = Some("fire".to_string());
        state.players.push(player);

        let melds = vec![meld1, meld2];
        lay_melds(&mut state, melds); // Should panic - fire card not used (score is 73 but fire card not included)
    }

    #[test]
    #[should_panic(expected = "Card not in hand")]
    fn test_lay_melds_card_not_in_hand() {
        // Test that laying melds with card not in hand panics
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::King),
                create_card("p2", Suit::Diamonds, Rank::King),
                create_card("p3", Suit::Clubs, Rank::King),
                create_card("p4", Suit::Spades, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::King),
            create_card("p2", Suit::Diamonds, Rank::King),
            // p3 is missing!
            create_card("p4", Suit::Spades, Rank::King),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = true;
        state.players.push(player);

        let melds = vec![meld];
        lay_melds(&mut state, melds); // Should panic
    }

    #[test]
    #[should_panic(expected = "Invalid meld")]
    fn test_lay_melds_invalid_meld() {
        // Test that laying an invalid meld panics
        let mut state = create_test_round_state();

        let meld = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::King),
                create_card("p2", Suit::Hearts, Rank::King), // Same suit - invalid
                create_card("p3", Suit::Hearts, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::King),
            create_card("p2", Suit::Hearts, Rank::King),
            create_card("p3", Suit::Hearts, Rank::King),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = true;
        state.players.push(player);

        let melds = vec![meld];
        lay_melds(&mut state, melds); // Should panic
    }

    #[test]
    #[should_panic(expected = "No melds to lay")]
    fn test_lay_melds_empty_melds() {
        // Test that laying empty melds panics
        let mut state = create_test_round_state();

        let player_hand = vec![create_card("p1", Suit::Hearts, Rank::King)];
        let player = create_test_player("P1", player_hand);
        state.players.push(player);

        let melds: Vec<Meld> = vec![];
        lay_melds(&mut state, melds); // Should panic
    }

    #[test]
    fn test_lay_melds_sequence_meld() {
        // Test laying a sequence meld
        let mut state = create_test_round_state();

        let meld1 = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::Number(5)),
                create_card("p2", Suit::Hearts, Rank::Number(6)),
                create_card("p3", Suit::Hearts, Rank::Number(7)),
                create_card("p4", Suit::Hearts, Rank::Number(8)),
                create_card("p5", Suit::Hearts, Rank::Number(9)),
                create_card("p6", Suit::Hearts, Rank::Number(10)),
            ],
            meld_type: MeldType::Sequence,
        };

        let meld2 = Meld {
            cards: vec![
                create_card("p7", Suit::Diamonds, Rank::Jack),
                create_card("p8", Suit::Diamonds, Rank::Queen),
                create_card("p9", Suit::Diamonds, Rank::King),
            ],
            meld_type: MeldType::Sequence,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::Number(5)),
            create_card("p2", Suit::Hearts, Rank::Number(6)),
            create_card("p3", Suit::Hearts, Rank::Number(7)),
            create_card("p4", Suit::Hearts, Rank::Number(8)),
            create_card("p5", Suit::Hearts, Rank::Number(9)),
            create_card("p6", Suit::Hearts, Rank::Number(10)),
            create_card("p7", Suit::Diamonds, Rank::Jack),
            create_card("p8", Suit::Diamonds, Rank::Queen),
            create_card("p9", Suit::Diamonds, Rank::King),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false;
        state.players.push(player);

        let melds = vec![meld1, meld2];
        lay_melds(&mut state, melds);

        // check_melds should split the 6-card sequence
        assert_eq!(state.table_melds.len(), 3); // 2 from 6-card split + 1 from 3-card
        assert!(state.players[0].melded);
        assert_eq!(state.players[0].hand.len(), 0);
    }

    #[test]
    fn test_lay_melds_with_joker() {
        // Test laying melds with a joker
        let mut state = create_test_round_state();

        let meld1 = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::King),
                create_card("p2", Suit::Diamonds, Rank::King),
                create_card("joker", Suit::Joker, Rank::Joker),
            ],
            meld_type: MeldType::Rank,
        };

        let meld2 = Meld {
            cards: vec![
                create_card("p3", Suit::Spades, Rank::Queen),
                create_card("p4", Suit::Hearts, Rank::Queen),
                create_card("p5", Suit::Diamonds, Rank::Queen),
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::King),
            create_card("p2", Suit::Diamonds, Rank::King),
            create_card("joker", Suit::Joker, Rank::Joker),
            create_card("p3", Suit::Spades, Rank::Queen),
            create_card("p4", Suit::Hearts, Rank::Queen),
            create_card("p5", Suit::Diamonds, Rank::Queen),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false;
        state.players.push(player);

        let melds = vec![meld1, meld2];
        lay_melds(&mut state, melds);

        // Should succeed - joker counts as King (10 + 10 + 10 + 10 + 10 + 10 = 60)
        assert_eq!(state.table_melds.len(), 2);
        assert!(state.players[0].melded);
    }

    #[test]
    fn test_lay_melds_mixed_rank_and_sequence() {
        // Test laying both rank and sequence melds together
        let mut state = create_test_round_state();

        let rank_meld = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::Ace),   // 11
                create_card("p2", Suit::Diamonds, Rank::Ace), // 11
                create_card("p3", Suit::Clubs, Rank::Ace),    // 11
            ],
            meld_type: MeldType::Rank,
        };

        let seq_meld = Meld {
            cards: vec![
                create_card("p4", Suit::Spades, Rank::Jack),  // 10
                create_card("p5", Suit::Spades, Rank::Queen), // 10
                create_card("p6", Suit::Spades, Rank::King),  // 10
            ],
            meld_type: MeldType::Sequence,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::Ace),
            create_card("p2", Suit::Diamonds, Rank::Ace),
            create_card("p3", Suit::Clubs, Rank::Ace),
            create_card("p4", Suit::Spades, Rank::Jack),
            create_card("p5", Suit::Spades, Rank::Queen),
            create_card("p6", Suit::Spades, Rank::King),
            create_card("p7", Suit::Hearts, Rank::Number(2)),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false;
        state.players.push(player);

        let melds = vec![rank_meld, seq_meld];
        lay_melds(&mut state, melds);

        // Total: 33 + 30 = 63 >= 51
        assert_eq!(state.table_melds.len(), 2);
        assert_eq!(state.players[0].hand.len(), 1);
        assert!(state.players[0].melded);
    }

    #[test]
    fn test_lay_melds_triggers_check_melds_for_four_card_rank() {
        // Test that laying a 4-card rank meld triggers check_melds and moves to fire
        let mut state = create_test_round_state();

        let meld1 = Meld {
            cards: vec![
                create_card("p1", Suit::Hearts, Rank::Number(10)),
                create_card("p2", Suit::Diamonds, Rank::Number(10)),
                create_card("p3", Suit::Clubs, Rank::Number(10)),
                create_card("p4", Suit::Spades, Rank::Number(10)),
            ],
            meld_type: MeldType::Rank,
        };

        let meld2 = Meld {
            cards: vec![
                create_card("p5", Suit::Hearts, Rank::King),
                create_card("p6", Suit::Diamonds, Rank::King),
                create_card("p7", Suit::Clubs, Rank::King),
            ],
            meld_type: MeldType::Rank,
        };

        let player_hand = vec![
            create_card("p1", Suit::Hearts, Rank::Number(10)),
            create_card("p2", Suit::Diamonds, Rank::Number(10)),
            create_card("p3", Suit::Clubs, Rank::Number(10)),
            create_card("p4", Suit::Spades, Rank::Number(10)),
            create_card("p5", Suit::Hearts, Rank::King),
            create_card("p6", Suit::Diamonds, Rank::King),
            create_card("p7", Suit::Clubs, Rank::King),
        ];

        let mut player = create_test_player("P1", player_hand);
        player.melded = false;
        state.players.push(player);

        let melds = vec![meld1, meld2];
        lay_melds(&mut state, melds);

        // The 4-card rank meld should be moved to fire pile
        assert_eq!(state.table_melds.len(), 1); // Only the 3-card King meld remains
        assert_eq!(state.fire_pile.len(), 4); // 4 cards moved to fire
    }
}
