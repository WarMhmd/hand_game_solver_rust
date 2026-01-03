use std::any::Any;

use crate::apis::bot::MeldsData;
use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::better_meld_play::UseBetterMeldPlay;
use crate::logic::{
    can_play_in_rank_meld, can_play_in_sequence_meld, card_penality, melds_value, ActivePlayer,
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

            let mut cards: Vec<Card> = hand
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
                    if hand[i].rank == Rank::Joker {
                        // Joker can be used as a wildcard
                        continue;
                    }
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

    pub fn get_joker_card(&mut self, card: &Card, table_melds: &Vec<Meld>) -> bool {
        for meld in table_melds {
            if meld.meld_type == MeldType::Sequence {
                let (_, _, take_joker) = can_play_in_sequence_meld(meld, card, false);
                if take_joker {
                    return take_joker;
                }
            }

            if meld.meld_type == MeldType::Rank {
                let (_, take_joker) = can_play_in_rank_meld(meld, card);
                if take_joker {
                    return take_joker;
                }
            }
        }
        false
    }

    pub fn evaluate_discard_option(
        &mut self,
        hand: &Vec<Card>,
        candidate_cards: u32,
        melded: bool,
        table_melds: &Vec<Meld>,
    ) -> usize {
        let mut max_penalty = if melded { 0 } else { i32::MAX };
        let mut max_index = 0;
        for i in 0..hand.len() {
            let penalty = card_penality(&hand[i]);
            if melded && penalty > max_penalty && hand[i].rank != Rank::Joker {
                max_penalty = penalty;
                max_index = i;
            } else if !melded && penalty < max_penalty && hand[i].rank != Rank::Joker {
                max_penalty = penalty;
                max_index = i;
            }
        }
        let mut discard_card: Vec<(i32, usize)> = vec![(0, max_index)];
        if hand.len() <= 3 {
            return discard_card[0].1;
        }
        // (Score, index)
        let use_joker_base = &mut self.base.base.base;
        let mut mask = 0;
        let melds_cards = use_joker_base.meld_cards.clone();
        let melds_score = melds_value(&melds_cards);
        for meld in &melds_cards {
            for card in &meld.cards {
                let position = hand.iter().position(|c| c.id == card.id);
                if let Some(position) = position {
                    mask |= 1 << position;
                }
            }
        }
        for i in 0..hand.len() {
            let card_cost = if !melded {
                16 - card_penality(&hand[i])
            } else {
                card_penality(&hand[i])
            };
            if candidate_cards & (1 << i) == 0 {
                // not a candidate card
                // check if it's a duplicate
                let is_duplicate = hand
                    .iter()
                    .filter(|&c| c.rank == hand[i].rank && c.suit == hand[i].suit)
                    .count()
                    > 1;

                if is_duplicate {
                    // Duplicate card has the biggest priority
                    discard_card.push((1000000 * card_cost, i));
                    continue;
                } else {
                    // println!("Non used card {:?}:", hand[i]);
                    if !melded && card_cost > 6 {
                        // Non duplicate card & not candidate card has the third priority
                        discard_card.push((10000 * card_cost, i));
                    } else {
                        // Non duplicate card & candidate card has the second priority
                        discard_card.push((100000 * card_cost, i));
                    }
                    continue;
                }
            }
            if hand[i].rank == Rank::Joker {
                continue;
            }
            let mut new_candidate_cards = 0;
            let fake_joker = Card {
                id: "fake_joker".to_string(),
                suit: Suit::Joker,
                rank: Rank::Joker,
            };
            for j in 0..hand.len() {
                if i == j {
                    continue;
                }
                if candidate_cards & (1 << j) == 0 {
                    // not a candidate card
                    continue;
                }
                if hand[j].rank == Rank::Joker {
                    new_candidate_cards |= 1 << j;
                    continue;
                }
                for k in j + 1..hand.len() {
                    if k == i || k == j {
                        continue;
                    }
                    if candidate_cards & (1 << k) == 0 {
                        // not a candidate card
                        continue;
                    }
                    if hand[k].rank == Rank::Joker {
                        new_candidate_cards |= 1 << k;
                        continue;
                    }

                    let mut cards = hand
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index == j || *index == k)
                        .map(|(_, card)| card.clone())
                        .collect::<Vec<Card>>();

                    cards.push(fake_joker.clone());

                    if use_joker_base.get_rank_meld(&cards, &7, false) != -1 {
                        new_candidate_cards |= 1 << j;
                        new_candidate_cards |= 1 << k;
                    } else if use_joker_base.get_seq_meld(&cards, &7, false) != -1 {
                        new_candidate_cards |= 1 << j;
                        new_candidate_cards |= 1 << k;
                    }
                }
            }
            if mask & (1 << i) != 0 {
                if !melded {
                    continue;
                }
                // get new cards
                let new_cards: Vec<Card> = hand
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| new_candidate_cards & (1 << index) != 0)
                    .map(|(_, card)| card.clone())
                    .collect::<Vec<Card>>();
                use_joker_base.reset_calc_values();
                use_joker_base.calc(&new_cards);

                let mut rank_cards = Vec::new();
                for i in 0..hand.len() {
                    if (use_joker_base.best_take_rank & (1 << i)) != 0 {
                        rank_cards.push(hand[i].clone());
                    }
                }
                // Copy best_take_seq to avoid borrow checker issue
                let best_take_seq = use_joker_base.best_take_seq;
                let seq_cards = use_joker_base.get_seq_meld_cards(hand, &best_take_seq);
                use_joker_base.meld_cards = use_joker_base.get_rank_meld_cards(&rank_cards);
                use_joker_base.meld_cards.extend(seq_cards);
                let new_score = melds_value(&use_joker_base.meld_cards);
                let diff = new_score - melds_score;
                if diff <= 15 && 51 - melds_score >= 20 && card_penality(&hand[i]) <= 6 {
                    discard_card.push((10000 * card_cost, i));
                }
            } else {
                let diff: i32 = new_candidate_cards ^ (candidate_cards as i32);
                if diff.count_ones() == 1 {
                    discard_card.push((10000 * card_cost, i));
                } else if diff.count_ones() == 2 {
                    discard_card.push((7000 * card_cost, i));
                }
            }
        }
        // sort discard_card by cost, the highest cost card is discarded first
        discard_card.sort_by(|a, b| b.0.cmp(&a.0));
        let mut index = discard_card[0].1;
        if self.get_joker_card(&hand[index], table_melds) && discard_card.len() > 1 {
            index = discard_card[1].1;
            if self.get_joker_card(&hand[index], table_melds) && discard_card.len() > 2 {
                index = discard_card[2].1;
            }
        }
        index
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
        let player: &ActivePlayer = &state.players[state.current_player];
        let hand: &Vec<Card> = &player.hand;
        let melded = player.melded;
        let candidate_card = self.candidate_melds(hand);
        for (i, card) in hand.iter().enumerate() {
            if candidate_card & (1 << i) == 0 {
                println!("Card {:?} is bad", card.id.clone());
            }
        }
        let ans = self.evaluate_discard_option(hand, candidate_card, melded, &state.table_melds);
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
