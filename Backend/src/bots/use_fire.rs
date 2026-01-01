use std::any::Any;

use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::use_joker::UseJokerBot;
use crate::logic::{melds_value, Card, Meld, Phase, RoundState};

#[derive(Clone, Debug)]
pub struct UseFireBot {
    pub base: UseJokerBot,

    pub is_fire_card: bool,
}

impl UseFireBot {
    pub fn new(name: String) -> Self {
        Self {
            base: UseJokerBot::new(name),
            is_fire_card: false,
        }
    }

    fn calc(&mut self, hand: &Vec<Card>) {
        let n = hand.len() as u32;
        let limit = 1u32 << n;
        let mask = 1u32 << (n - 1);

        for take_rank in 0..limit {
            self.base.get_rank_meld(hand, &take_rank, true);
        }
        for take_seq in 0..limit {
            self.base.get_seq_meld(hand, &take_seq, true);
        }
        for take_rank in 0..limit {
            let rank_ones = take_rank.count_ones();
            if rank_ones < 3 && rank_ones != 0 {
                continue;
            }

            let rank_value = self.base.dp_rank_meld[take_rank as usize];
            if rank_value == -1 {
                continue;
            }

            // cards NOT used in rank
            let remaining = (!take_rank) & (limit - 1);

            // iterate ALL submasks of remaining
            let mut take_seq = remaining;
            loop {
                let seq_ones = take_seq.count_ones();
                'check_value: {
                    if seq_ones >= 3 || seq_ones == 0 {
                        if self.is_fire_card && (take_rank & mask) == 0 && (take_seq & mask) == 0 {
                            // fire card is not used in rank or sequence
                            break 'check_value;
                        }
                        let size = rank_ones + seq_ones;
                        if size == n {
                            break 'check_value;
                        }

                        let seq_value = self.base.dp_rank_seq[take_seq as usize];
                        if seq_value == -1 {
                            break 'check_value;
                        }

                        let total_val = rank_value + seq_value;

                        if self.base.is_melded {
                            if self.base.max_cards_count < size as i32 {
                                self.base.max_cards_count = size as i32;
                                self.base.max_value = total_val;
                                self.base.best_take_rank = take_rank;
                                self.base.best_take_seq = take_seq;
                            } else if self.base.max_cards_count == size as i32 {
                                if total_val > self.base.max_value {
                                    self.base.max_value = total_val;
                                    self.base.best_take_rank = take_rank;
                                    self.base.best_take_seq = take_seq;
                                }
                            }
                        } else {
                            if self.base.max_value < total_val {
                                self.base.max_value = total_val;
                                self.base.max_cards_count = size as i32;
                                self.base.best_take_rank = take_rank;
                                self.base.best_take_seq = take_seq;
                            } else if self.base.max_value == total_val {
                                if (size as i32) > self.base.max_cards_count {
                                    self.base.max_cards_count = size as i32;
                                    self.base.best_take_rank = take_rank;
                                    self.base.best_take_seq = take_seq;
                                }
                            }
                        }
                    }
                }

                if take_seq == 0 {
                    break;
                }
                take_seq = (take_seq - 1) & remaining;
            }
        }
        return;
    }
}

impl BotStrategy for UseFireBot {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn can_use_features(&self) -> &[String] {
        &self.base.features
    }

    fn clone_box(&self) -> Box<dyn BotStrategy> {
        Box::new(self.clone())
    }

    fn decide_draw(&mut self, state: &RoundState) -> DecideDrawResult {
        if state.fire_pile.len() > 0 {
            let fire_card = state.fire_pile.last().unwrap().clone();
            // emulate fireCard Draw
            let mut hand: Vec<Card> = state.players[state.current_player].hand.clone();
            hand.push(fire_card);
            self.base.reset_calc_values();
            self.is_fire_card = true;

            self.calc(&hand);

            if self.base.max_cards_count > 0 {
                let mut rank_cards = Vec::new();
                for i in 0..hand.len() {
                    if (self.base.best_take_rank & (1 << i)) != 0 {
                        // println!("rank card used: {}", hand[i].clone().id);
                        rank_cards.push(hand[i].clone());
                    }
                }
                self.base.meld_cards = self.base.get_rank_meld_cards(&rank_cards);
                let best_take_seq = self.base.best_take_seq;
                let seq_cards = self.base.get_seq_meld_cards(&hand, &best_take_seq);
                self.base.meld_cards.extend(seq_cards.clone());
            }

            if !self.base.is_melded {
                if self.base.max_value >= 51 {
                    return DecideDrawResult::Fire;
                } else {
                    self.base.reset_calc_values();
                    self.is_fire_card = false;
                    return DecideDrawResult::Deck;
                }
            } else {
                if self.base.max_value > 0 {
                    return DecideDrawResult::Fire;
                } else {
                    self.base.reset_calc_values();
                    self.is_fire_card = false;
                    return DecideDrawResult::Deck;
                }
            }
        } else {
            self.is_fire_card = false;
            DecideDrawResult::Deck
        }
    }

    fn find_melds(&mut self, hand: &Vec<Card>) -> Vec<Meld> {
        if self.is_fire_card {
            // println!("Already calculated");
            return self.base.meld_cards.clone();
        }
        self.base.reset_calc_values();

        self.calc(hand);

        if self.base.max_cards_count > 0 {
            let mut rank_cards = Vec::new();
            for i in 0..hand.len() {
                if (self.base.best_take_rank & (1 << i)) != 0 {
                    rank_cards.push(hand[i].clone());
                }
            }
            self.base.meld_cards = self.base.get_rank_meld_cards(&rank_cards);
            let best_take_seq = self.base.best_take_seq;
            let seq_cards = self.base.get_seq_meld_cards(&hand, &best_take_seq);
            self.base.meld_cards.extend(seq_cards);
        }

        self.base.meld_cards.clone()
    }

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        let player = &state.players[state.current_player];
        let melds = self.find_melds(&player.hand);
        self.is_fire_card = false;
        if melds_value(&melds) < 51 && !player.melded {
            return vec![];
        }

        melds
    }

    fn decide_play_in_meld(&mut self, state: &RoundState) -> (Phase, Option<Card>, bool, i32) {
        self.base.decide_play_in_meld(state)
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        // for card in &state.players[state.current_player].hand {
        // println!("Card {:?}", card.id);
        // }
        self.base.decide_discard(state)
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
#[path = "./use_fire_test.rs"]
mod use_fire_test;
