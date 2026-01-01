use std::any::Any;

use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::better_meld_play::UseBetterMeldPlay;
use crate::logic::{
    can_play_in_rank_meld, can_play_in_sequence_meld, card_penality, check_seq_melds, melds_value,
    Card, Meld, MeldType, Phase, Rank, RoundState, Suit,
};

#[derive(Clone, Debug)]
pub struct UseBetterDiscard {
    pub base: UseBetterMeldPlay,
    features: Vec<String>,
}

impl UseBetterDiscard {
    pub fn new(name: String) -> Self {
        Self {
            base: UseBetterMeldPlay::new(name),
            features: vec!["useMeld".to_string()],
        }
    }

    pub fn candidate_melds(&mut self, hand: &Vec<Card>) -> u32 {
        let n = hand.len() as u32;
        let limit = 1u32 << n;
        let use_joker_base = &mut self.base.base.base;
        let mut mask = 0;
        for meld in &use_joker_base.meld_cards {
            for card in &meld.cards {
                let position = hand.iter().position(|c| c.id == card.id);
                if let Some(position) = position {
                    mask |= 1 << position;
                }
            }
        }
        let mut candidate_cards: u32 = mask;

        let fake_joker = Card {
            id: "fake_joker".to_string(),
            suit: Suit::Joker,
            rank: Rank::Joker,
        };

        // Note: Can optimize this code later to n^2
        for take_cards in 0..limit {
            if mask & take_cards == take_cards {
                continue;
            }

            if take_cards.count_ones() != 2 {
                continue;
            }

            let mut cards = hand
                .iter()
                .enumerate()
                .filter(|(index, card)| {
                    if take_cards & (1 << index) != 0 {
                        if card.rank == Rank::Joker {
                            candidate_cards |= 1 << index;
                        }
                        true
                    } else {
                        false
                    }
                })
                .map(|(_, card)| card.clone())
                .collect::<Vec<_>>();

            if cards[0].rank == Rank::Joker || cards[1].rank == Rank::Joker {
                continue;
            }

            cards.push(fake_joker.clone());

            if use_joker_base.get_rank_meld(&cards, &7, false) != -1 {
                candidate_cards |= take_cards;
            } else if use_joker_base.get_seq_meld(&cards, &7, false) != -1 {
                candidate_cards |= take_cards;
            }
        }

        // Handle duplicate cards
        for i in 0..hand.len() {
            for j in i + 1..hand.len() {
                if candidate_cards & (1 << j) == 0 || candidate_cards & (1 << i) == 0 {
                    // either one of i and j is not a candidate card
                    continue;
                }
                if hand[i].suit == hand[j].suit && hand[i].rank == hand[j].rank {
                    if mask & (1 << i) == 0 || mask & (1 << j) == 0 {
                        // At least one of i and j is not in the mask
                        // remove one of them
                        candidate_cards ^= 1 << j;
                    }
                }
            }
        }
        println!("candidate_cards: {:15b}", candidate_cards);
        candidate_cards
    }

    // pub fn discard_card_fn(&mut self, hand: &Vec<Card>, candidate_cards: u32) -> usize {
    //     if hand.len() <= 3 {
    //         // return card index with the highest penalty
    //         let mut max_penalty = 0;
    //         let mut max_index = 0;
    //         for i in 0..hand.len() {
    //             let penalty = card_penality(&hand[i]);
    //             if penalty > max_penalty {
    //                 max_penalty = penalty;
    //                 max_index = i;
    //             }
    //         }
    //         return max_index;
    //     }
    //     // (Score, index)
    //     let mut discard_card: Vec<(i32, usize)> = Vec::new();
    //     let use_joker_base = &mut self.base.base.base;
    //     let mut mask = 0;
    //     let melds_cards = use_joker_base.meld_cards.clone();
    //     let melds_score = melds_value(&melds_cards);
    //     for meld in &melds_cards {
    //         for card in &meld.cards {
    //             let position = hand.iter().position(|c| c.id == card.id);
    //             if let Some(position) = position {
    //                 mask |= 1 << position;
    //             }
    //         }
    //     }
    //     for i in 0..hand.len() {
    //         if candidate_cards & (1 << i) == 0 {
    //             // not a candidate card
    //             discard_card.push((1000, i));
    //             continue;
    //         }
    //         if hand[i].rank == Rank::Joker {
    //             continue;
    //         }
    //         let new_candidate_cards = 0;
    //         for j in 0..hand.len() {
    //             if i == j {
    //                 continue;
    //             }
    //             if candidate_cards & (1 << j) == 0 {
    //                 // not a candidate card
    //                 continue;
    //             }
    //             if hand[j].rank == Rank::Joker {
    //                 new_candidate_cards |= 1 << j;
    //                 continue;
    //             }
    //             for k in j + 1..hand.len() {
    //                 if k == i || k == j {
    //                     continue;
    //                 }
    //                 if candidate_cards & (1 << k) == 0 {
    //                     // not a candidate card
    //                     continue;
    //                 }
    //                 if hand[k].rank == Rank::Joker {
    //                     new_candidate_cards |= 1 << k;
    //                     continue;
    //                 }

    //                 let mut cards = hand
    //                     .iter()
    //                     .enumerate()
    //                     .filter(|(index, _)| index == j || index == k)
    //                     .map(|(_, card)| card.clone())
    //                     .collect::<Vec<_>>();

    //                 cards.push(fake_joker.clone());

    //                 if use_joker_base.get_rank_meld(&cards, &7, false) != -1 {
    //                     candidate_cards |= 1 << j;
    //                     candidate_cards |= 1 << k;
    //                 } else if use_joker_base.get_seq_meld(&cards, &7, false) != -1 {
    //                     candidate_cards |= 1 << j;
    //                     candidate_cards |= 1 << k;
    //                 }
    //             }
    //         }
    //         let diff = candidate_cards ^ new_candidate_cards;
    //         if mask & (1 << i) != 0 {
    //             // get new cards
    //             let new_cards = hand
    //                 .iter()
    //                 .enumerate()
    //                 .filter(|(index, _)| new_candidate_cards & (1 << index) != 0)
    //                 .map(|(_, card)| card.clone())
    //                 .collect::<Vec<_>>();
    //             use_joker_base.reset_calc_values();
    //             use_joker_base.calc(&new_cards);

    //             let mut rank_cards = Vec::new();
    //             for i in 0..hand.len() {
    //                 if (use_joker_base.best_take_rank & (1 << i)) != 0 {
    //                     rank_cards.push(hand[i].clone());
    //                 }
    //             }
    //             // Copy best_take_seq to avoid borrow checker issue
    //             let best_take_seq = use_joker_base.best_take_seq;
    //             let seq_cards = use_joker_base.get_seq_meld_cards(hand, &best_take_seq);
    //             use_joker_base.meld_cards = use_joker_base.get_rank_meld_cards(&rank_cards);
    //             use_joker_base.meld_cards.extend(seq_cards);
    //             let new_score = melds_value(&use_joker_base.meld_cards);
    //             let diff = new_score - melds_score;
    //             if diff <= 15 && card_penality(&hand[i]) <= 6 {
    //                 discard_card.push((16 - diff, i));
    //             }
    //         }
    //     }
    //     return 0;
    // }
}

impl BotStrategy for UseBetterDiscard {
    fn name(&self) -> &str {
        &self.base.name()
    }
    fn can_use_features(&self) -> &[String] {
        &self.features
    }

    fn clone_box(&self) -> Box<dyn BotStrategy> {
        Box::new(self.clone())
    }

    fn decide_draw(&mut self, state: &RoundState) -> DecideDrawResult {
        self.base.decide_draw(state)
    }

    fn find_melds(&mut self, hand: &Vec<Card>) -> Vec<Meld> {
        self.base.find_melds(hand)
    }

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        self.base.decide_melds(state)
    }

    fn decide_play_in_meld(&mut self, state: &RoundState) -> (Phase, Option<Card>, bool, i32) {
        self.base.decide_play_in_meld(state)
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        let player = &state.players[state.current_player];
        let hand = &player.hand;
        let candidate_card = self.candidate_melds(hand);
        for (i, card) in hand.iter().enumerate() {
            if candidate_card & (1 << i) == 0 {
                println!("Card {:?} is bad", card.id.clone());
            }
        }
        let ans;
        if candidate_card.count_ones() == hand.len() as u32 || (2..=3).contains(&hand.len()) {
            if player.melded {
                // return highest card
                let max = hand
                    .iter()
                    .max_by_key(|c| {
                        if c.rank == Rank::Joker {
                            0
                        } else {
                            card_penality(c)
                        }
                    })
                    .unwrap();
                ans = hand.iter().position(|c| c.id == max.id).unwrap();
            } else {
                let min = hand
                    .iter()
                    .min_by_key(|c| {
                        if c.rank == Rank::Joker {
                            999
                        } else {
                            card_penality(c)
                        }
                    })
                    .unwrap();
                println!("{}", card_penality(&min));
                ans = hand.iter().position(|c| c.id == min.id).unwrap();
            }
        } else {
            if player.melded {
                // return highest card
                let max = hand
                    .iter()
                    .enumerate()
                    .max_by_key(|(i, c)| {
                        if c.rank == Rank::Joker {
                            0
                        } else {
                            if candidate_card & (1 << i) != 0 {
                                0
                            } else {
                                card_penality(c)
                            }
                        }
                    })
                    .unwrap();
                ans = hand.iter().position(|c| c.id == max.1.id).unwrap();
            } else {
                let min = hand
                    .iter()
                    .enumerate()
                    .min_by_key(|(i, c)| {
                        if c.rank == Rank::Joker {
                            999
                        } else {
                            if candidate_card & (1 << i) != 0 {
                                999
                            } else {
                                card_penality(c)
                            }
                        }
                    })
                    .unwrap();
                ans = hand.iter().position(|c| c.id == min.1.id).unwrap();
            }
        }
        // print the card
        println!("Discarding card: {:?}", hand[ans]);
        ans
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Include test module
#[cfg(test)]
#[path = "./better_discard_test.rs"]
mod better_discard_test;
