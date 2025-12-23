use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::use_joker::UseJokerBot;
use crate::logic::{melds_value, Card, Meld, RoundState};

pub struct UseFireBot {
    base: UseJokerBot,

    is_fire_card: bool,
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
            if take_rank.count_ones() >= 3 {
                self.base.get_rank_meld(hand, &take_rank);
            }
        }
        for take_seq in 0..limit {
            if take_seq.count_ones() >= 3 {
                self.base.get_seq_meld(hand, &take_seq);
            }
        }
        for take_rank in 0..limit {
            let rank_ones = take_rank.count_ones();
            if rank_ones < 3 {
                continue;
            }
            // cards NOT used in rank
            let remaining = (!take_rank) & (limit - 1);

            // iterate ALL submasks of remaining
            let mut take_seq = remaining;
            loop {
                let seq_ones = take_seq.count_ones();
                'check_value: {
                    if seq_ones >= 3 {
                        if self.is_fire_card && (take_rank & mask) == 0 && (take_seq & mask) == 0 {
                            // fire card is not used in rank or sequence
                            break 'check_value;
                        }
                        let size = rank_ones + seq_ones;
                        if size == n {
                            break 'check_value;
                        }

                        let rank_value = self.base.dp_rank_meld[take_rank as usize];
                        if rank_value == -1 {
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
                let mut seq_cards = Vec::new();
                for i in 0..hand.len() {
                    if (self.base.best_take_rank & (1 << i)) != 0 {
                        rank_cards.push(hand[i].clone());
                    }
                    if (self.base.best_take_seq & (1 << i)) != 0 {
                        seq_cards.push(hand[i].clone());
                    }
                }
                self.base.meld_cards = self.base.get_rank_meld_cards(&rank_cards);
                self.base
                    .meld_cards
                    .extend(self.base.get_seq_meld_cards(&seq_cards));
            }

            if self.base.is_melded {
                if self.base.max_value >= 51 {
                    println!("{:15b}", self.base.best_take_rank);
                    println!("{:15b}", self.base.best_take_seq);
                    return DecideDrawResult::Fire;
                } else {
                    self.is_fire_card = false;
                    return DecideDrawResult::Deck;
                }
            } else {
                if self.base.max_value > 0 {
                    println!("{:15b}", self.base.best_take_rank);
                    println!("{:15b}", self.base.best_take_seq);
                    return DecideDrawResult::Fire;
                } else {
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
            return self.base.meld_cards.clone();
        }
        self.base.reset_calc_values();

        self.calc(hand);

        if self.base.max_cards_count > 0 {
            let mut rank_cards = Vec::new();
            let mut seq_cards = Vec::new();
            for i in 0..hand.len() {
                if (self.base.best_take_rank & (1 << i)) != 0 {
                    rank_cards.push(hand[i].clone());
                }
                if (self.base.best_take_seq & (1 << i)) != 0 {
                    seq_cards.push(hand[i].clone());
                }
            }
            self.base.meld_cards = self.base.get_rank_meld_cards(&rank_cards);
            self.base
                .meld_cards
                .extend(self.base.get_seq_meld_cards(&seq_cards));
        }

        self.base.meld_cards.clone()
    }

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        let player = &state.players[state.current_player];
        let melds = self.find_melds(&player.hand);
        if melds_value(&melds) < 51 && !player.melded {
            return vec![];
        }

        melds
    }

    fn decide_play_in_meld(&mut self, state: &RoundState) -> (Option<Card>, i32) {
        self.base.decide_play_in_meld(state)
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        self.base.decide_discard(state)
    }
}
