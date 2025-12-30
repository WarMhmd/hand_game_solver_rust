use std::any::Any;

use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::better_meld_play::UseBetterMeldPlay;
use crate::logic::{
    can_play_in_rank_meld, can_play_in_sequence_meld, card_penality, check_seq_melds, Card, Meld,
    MeldType, Phase, Rank, RoundState, Suit,
};

#[derive(Clone, Debug)]
pub struct UseBetterDiscard {
    base: UseBetterMeldPlay,
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
        let mut mask = 0;
        let use_joker_base = &mut self.base.base.base;
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
                .filter(|(index, _)| take_cards & (1 << index) != 0)
                .map(|(_, card)| card.clone())
                .collect::<Vec<_>>();

            // println!("Cards:");
            // for card in &cards {
            //     println!("Card: {:?}", card.id.clone());
            // }

            cards.push(fake_joker.clone());
            // if cards[0].id == "heart-7" && cards[1].id == "spade-7" {
            //     println!("{}", use_joker_base.get_rank_meld(&cards, &7, false));
            // }
            if use_joker_base.get_rank_meld(&cards, &7, false) != -1 {
                candidate_cards |= take_cards;
            } else if use_joker_base.get_seq_meld(&cards, &7, false) != -1 {
                candidate_cards |= take_cards;
            }
        }

        // println!("Candidate Cards: {:15b}", candidate_cards);

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
        if candidate_card.count_ones() == hand.len() as u32 {
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
