use uuid::Uuid;

use crate::bot::{group_by_rank, group_by_suit, BotStrategy, DecideDrawResult};
use crate::logic::{
    can_play_in_rank_meld, can_play_in_sequence_meld, melds_value, rank_order, rank_value_int,
    valid_sequence_two_cards, Card, Meld, MeldType, Phase, Rank, RoundState, Suit,
};
use std::any::Any;
use std::collections::{HashMap, HashSet};

macro_rules! update_max_and_next {
    ($rank:expr, $max:expr, $next:expr) => {
        if $rank > $max {
            $next = $max;
            $max = $rank;
        } else if $rank >= $next {
            $next = $rank;
        }
    };
}

#[derive(Clone, Debug)]
pub struct UseJokerBot {
    pub name: String,
    pub features: Vec<String>,

    // State for calculation
    pub max_cards_count: i32,
    pub max_value: i32,
    pub best_take_rank: u32,
    pub best_take_seq: u32,
    pub is_melded: bool,
    pub meld_cards: Vec<Meld>,
    pub dp_rank_seq: Vec<i32>,
    pub dp_rank_meld: Vec<i32>,
}

impl UseJokerBot {
    pub fn new(name: String) -> Self {
        Self {
            name,
            features: vec!["useMeld".to_string()],
            max_cards_count: -1,
            max_value: 0,
            best_take_rank: 0,
            best_take_seq: 0,
            is_melded: false,
            meld_cards: Vec::new(),
            dp_rank_seq: Vec::new(),
            dp_rank_meld: Vec::new(),
        }
    }

    pub fn reset_calc_values(&mut self) {
        self.max_cards_count = 0;
        self.max_value = 0;
        self.best_take_rank = 0;
        self.best_take_seq = 0;
        // self.is_melded = false;
        self.meld_cards = Vec::new();
        self.dp_rank_seq = vec![-2; 32769];
        self.dp_rank_meld = vec![-2; 32769];
    }

    pub fn get_rank_meld(&mut self, hand: &Vec<Card>, take_rank: &u32, with_memo: bool) -> i32 {
        let take_rank_index = *take_rank as usize;
        if with_memo && self.dp_rank_meld[take_rank_index] != -2 {
            return self.dp_rank_meld[take_rank_index];
        }

        let mut melds_val = 0;

        let mut joker_count = 0;
        let mut max_can_joker_value: i32 = -1;
        let mut next_max_can_joker_value: i32 = -1;

        // Local grouping to avoid cloning (bot::group_by_rank clones)
        let mut groups: Vec<Vec<&Card>> = Vec::with_capacity(13);
        for i in 0..hand.len() {
            if 1 & (take_rank >> i) == 0 {
                continue;
            }
            if hand[i].rank == Rank::Joker {
                joker_count += 1;
            } else {
                let mut j = 0;
                while j < 13 {
                    if j >= groups.len() {
                        groups.push(Vec::new());
                    }
                    if groups[j].is_empty() || groups[j][0].rank == hand[i].rank {
                        groups[j].push(&hand[i]);
                        break;
                    }
                    j += 1;
                }
            }
        }

        for group in groups.iter() {
            let set_suit: HashSet<_> = group.iter().map(|c| c.suit).collect();
            let rank = rank_value_int(group[0].rank);
            match group.len() {
                2 => {
                    if set_suit.len() == 2 {
                        joker_count -= 1;
                        if joker_count < 0 {
                            if with_memo {
                                self.dp_rank_meld[take_rank_index] = -1;
                            }
                            return -1;
                        }
                        melds_val += 3 * rank;
                    } else {
                        if with_memo {
                            self.dp_rank_meld[take_rank_index] = -1;
                        }
                        return -1;
                    }
                }
                3 => {
                    if set_suit.len() == 3 {
                        melds_val += 3 * rank;
                        update_max_and_next!(rank, max_can_joker_value, next_max_can_joker_value);
                    } else if set_suit.len() == 2 {
                        if with_memo {
                            self.dp_rank_meld[take_rank_index] = -1;
                        }
                        return -1;
                    }
                }
                4 => {
                    if set_suit.len() == 4 {
                        melds_val += 4 * rank;
                    } else if set_suit.len() == 3 {
                        if with_memo {
                            self.dp_rank_meld[take_rank_index] = -1;
                        }
                        return -1;
                    } else if set_suit.len() == 2 {
                        joker_count -= 2;
                        if joker_count < 0 {
                            if with_memo {
                                self.dp_rank_meld[take_rank_index] = -1;
                            }
                            return -1;
                        }
                        melds_val += 6 * rank;
                    }
                }
                5 => {
                    if set_suit.len() == 4 {
                        joker_count -= 1;
                        if joker_count < 0 {
                            if with_memo {
                                self.dp_rank_meld[take_rank_index] = -1;
                            }
                            return -1;
                        }
                        melds_val += 6 * rank;
                        update_max_and_next!(rank, max_can_joker_value, next_max_can_joker_value);
                    } else if set_suit.len() == 3 {
                        joker_count -= 1;
                        if joker_count < 0 {
                            if with_memo {
                                self.dp_rank_meld[take_rank_index] = -1;
                            }
                            return -1;
                        }
                        melds_val += 6 * rank;
                        update_max_and_next!(rank, max_can_joker_value, next_max_can_joker_value);
                    }
                }
                6 => {
                    if set_suit.len() == 4 || set_suit.len() == 3 {
                        melds_val += 6 * rank;
                        update_max_and_next!(rank, max_can_joker_value, next_max_can_joker_value);
                        update_max_and_next!(rank, max_can_joker_value, next_max_can_joker_value);
                    }
                }
                7 => {
                    if set_suit.len() == 4 {
                        melds_val += 7 * rank;
                        update_max_and_next!(rank, max_can_joker_value, next_max_can_joker_value);
                    }
                }
                8 => {
                    if set_suit.len() == 4 {
                        melds_val += 8 * rank;
                    }
                }
                _ => {
                    if with_memo {
                        self.dp_rank_meld[take_rank_index] = -1;
                    }
                    return -1;
                }
            }
        }
        if joker_count > 0 {
            if max_can_joker_value == -1 {
                if with_memo {
                    self.dp_rank_meld[take_rank_index] = -1;
                }
                return -1;
            }
            joker_count -= 1;
            melds_val += max_can_joker_value;
            if joker_count > 0 {
                melds_val += next_max_can_joker_value;
            }
        }

        if with_memo {
            self.dp_rank_meld[take_rank_index] = melds_val;
        }
        melds_val
    }

    pub fn get_seq_meld(&mut self, hand: &Vec<Card>, take_seq: &u32, with_memo: bool) -> i32 {
        // print the ones and zeros of take_seq
        let take_seq_index = *take_seq as usize;
        if with_memo && self.dp_rank_seq[take_seq_index] != -2 {
            return self.dp_rank_seq[take_seq_index];
        }
        let mut joker_cards: Vec<&Card> = Vec::with_capacity(2);
        // Pre-allocate groups with proper capacity
        let mut groups: [Vec<&Card>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        for i in 0..hand.len() {
            if 1 & (take_seq >> i) == 0 {
                continue;
            }
            let card = &hand[i];

            if card.rank == Rank::Joker {
                joker_cards.push(card);
            } else {
                // Find or create group for this suit
                let mut placed = false;
                for group in &mut groups {
                    if group.is_empty() || group[0].suit == card.suit {
                        group.push(card);
                        placed = true;
                        break;
                    }
                }
                if !placed {
                    if with_memo {
                        self.dp_rank_seq[take_seq_index] = -1;
                    }
                    return -1; // More than 4 suits, impossible
                }
            }
        }

        let mut all_melds: Vec<Meld> = Vec::with_capacity(15);

        for group in &groups {
            if group.is_empty() {
                continue;
            }

            let mut new_cards: Vec<&Card> = Vec::with_capacity(15);
            let mut aces_cards: Vec<&Card> = Vec::with_capacity(4);

            for &card in group {
                if card.rank == Rank::Ace {
                    aces_cards.push(card);
                } else {
                    new_cards.push(card);
                }
            }

            if new_cards.is_empty() {
                if with_memo {
                    self.dp_rank_seq[take_seq_index] = -1;
                }

                return -1;
            }

            // Cache rank orders to avoid repeated function calls
            new_cards.sort_by_key(|card| rank_order(card.rank)[0]);

            let mut left_over_cards: Vec<&Card> = Vec::with_capacity(8);
            let mut meld_seqs: Vec<Vec<&Card>> = Vec::with_capacity(15);
            let mut meld: Vec<&Card> = vec![new_cards[0]];

            for i in 1..new_cards.len() {
                let prev_card = new_cards[i - 1];
                let card = new_cards[i];
                if card.rank == prev_card.rank {
                    left_over_cards.push(card);
                    continue;
                }
                if !valid_sequence_two_cards(prev_card, card, i == 1, i == new_cards.len() - 1) {
                    if !meld.is_empty() {
                        meld_seqs.push(meld);
                        meld = Vec::new();
                    }
                }
                meld.push(card);
            }
            if !meld.is_empty() {
                meld_seqs.push(meld);
            }

            if !left_over_cards.is_empty() {
                let mut meld = vec![left_over_cards[0]];
                for i in 1..left_over_cards.len() {
                    let prev_card = left_over_cards[i - 1];
                    let card = left_over_cards[i];
                    if !valid_sequence_two_cards(
                        prev_card,
                        card,
                        i == 1,
                        i == left_over_cards.len() - 1,
                    ) {
                        if !meld.is_empty() {
                            meld_seqs.push(meld);
                            meld = Vec::new();
                        }
                    }
                    meld.push(card);
                }
                if !meld.is_empty() {
                    meld_seqs.push(meld);
                }
            }

            let mut invalid_meld_seqs: Vec<Vec<&Card>> = Vec::with_capacity(15);
            let mut valid_meld_seqs: Vec<Vec<&Card>> = Vec::with_capacity(5);

            for meld in meld_seqs {
                if meld.len() > 2 {
                    valid_meld_seqs.push(meld);
                } else if !meld.is_empty() {
                    invalid_meld_seqs.push(meld);
                }
            }

            invalid_meld_seqs.sort_by_key(|m| m.len());

            // Break a sequence
            let mut delete_invalid_index: HashSet<usize> = HashSet::new();
            for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                if delete_invalid_index.contains(&i) {
                    continue;
                }
                let mut break_index: Option<usize> = None;

                let rank_l = rank_order(iv_meld[0].rank)[0];
                let rank_r = rank_order(iv_meld.last().unwrap().rank)[0];

                for (j, meld) in valid_meld_seqs.iter().enumerate() {
                    let rank_ll = rank_order(meld[0].rank)[0];
                    let rank_rr = rank_order(meld.last().unwrap().rank)[0];

                    if iv_meld.len() == 1 {
                        if rank_ll <= rank_l - 2 && rank_rr >= rank_r + 2 {
                            break_index = Some(j);
                        }
                    } else {
                        if rank_ll <= rank_l - 1 && rank_rr >= rank_r + 1 {
                            break_index = Some(j);
                            break;
                        }
                    }
                }

                if let Some(idx) = break_index {
                    let cards = valid_meld_seqs.swap_remove(idx);
                    delete_invalid_index.insert(i);
                    let cut_index = cards
                        .iter()
                        .position(|c| c.rank == iv_meld[0].rank)
                        .unwrap();
                    let mut new_part1: Vec<&Card> = cards[0..cut_index].to_vec();
                    new_part1.extend_from_slice(iv_meld);
                    let new_part2: Vec<&Card> = cards[cut_index..].to_vec();
                    valid_meld_seqs.push(new_part1);
                    valid_meld_seqs.push(new_part2);
                }
            }
            let temp: Vec<Vec<&Card>> = invalid_meld_seqs
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !delete_invalid_index.contains(i))
                .map(|(_, m)| m)
                .collect();
            invalid_meld_seqs = temp;

            // Connect a sequence
            delete_invalid_index.clear();
            for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                if delete_invalid_index.contains(&i) {
                    continue;
                }
                if joker_cards.is_empty() {
                    if with_memo {
                        self.dp_rank_seq[take_seq_index] = -1;
                    }
                    return -1;
                }

                if !aces_cards.is_empty()
                    && iv_meld.len() == 2
                    && (iv_meld[0].rank == Rank::Number(2) || iv_meld[1].rank == Rank::King)
                {
                    continue;
                }

                let iv_last_rank = rank_order(iv_meld.last().unwrap().rank)[0];
                let iv_first_rank = rank_order(iv_meld[0].rank)[0];

                for (j, meld) in invalid_meld_seqs.iter().enumerate() {
                    if delete_invalid_index.contains(&j) || i == j {
                        continue;
                    }
                    if !aces_cards.is_empty()
                        && meld.len() == 2
                        && (meld[0].rank == Rank::Number(2) || meld[1].rank == Rank::King)
                    {
                        continue;
                    }

                    let meld_first_rank = rank_order(meld[0].rank)[0];
                    let meld_last_rank = rank_order(meld.last().unwrap().rank)[0];

                    if iv_last_rank + 2 == meld_first_rank {
                        let joker_card = joker_cards.pop().unwrap();
                        delete_invalid_index.insert(i);
                        delete_invalid_index.insert(j);
                        let mut combined: Vec<&Card> = iv_meld.to_vec();
                        combined.push(joker_card);
                        combined.extend_from_slice(meld);
                        valid_meld_seqs.push(combined);
                        break;
                    }

                    if iv_first_rank == meld_last_rank + 2 {
                        let joker_card = joker_cards.pop().unwrap();
                        delete_invalid_index.insert(i);
                        delete_invalid_index.insert(j);
                        let mut combined: Vec<&Card> = meld.to_vec();
                        combined.push(joker_card);
                        combined.extend_from_slice(iv_meld);
                        valid_meld_seqs.push(combined);
                        break;
                    }
                }
                if delete_invalid_index.contains(&i) {
                    continue;
                }

                let mut add_to: Option<usize> = None;
                let mut to_left = false;

                for (j, meld) in valid_meld_seqs.iter().enumerate() {
                    let meld_first_rank = rank_order(meld[0].rank)[0];
                    let meld_last_rank = rank_order(meld[meld.len() - 1].rank)[0];

                    if iv_last_rank + 2 == meld_first_rank {
                        add_to = Some(j);
                        to_left = true;
                        break;
                    }
                    if iv_last_rank == meld_last_rank + 2 {
                        add_to = Some(j);
                        to_left = false;
                        break;
                    }
                }

                if let Some(idx) = add_to {
                    delete_invalid_index.insert(i);
                    let valid_meld = valid_meld_seqs.remove(idx);
                    let joker_card = joker_cards.pop().unwrap();
                    if to_left {
                        let mut combined: Vec<&Card> = iv_meld.to_vec();
                        combined.push(joker_card);
                        combined.extend(valid_meld);
                        valid_meld_seqs.push(combined);
                    } else {
                        let mut combined = valid_meld;
                        combined.push(joker_card);
                        combined.extend_from_slice(iv_meld);
                        valid_meld_seqs.push(combined);
                    }
                }
            }
            let temp: Vec<Vec<&Card>> = invalid_meld_seqs
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !delete_invalid_index.contains(i))
                .map(|(_, m)| m)
                .collect();
            invalid_meld_seqs = temp;

            // Add needed Aces and Jokers
            delete_invalid_index.clear();
            for (i, meld) in invalid_meld_seqs.iter().enumerate() {
                if meld.len() == 2 {
                    if !aces_cards.is_empty() {
                        if meld[0].rank == Rank::Number(2) {
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            let mut new_m = vec![ace_card];
                            new_m.extend_from_slice(meld);
                            valid_meld_seqs.push(new_m);
                            continue;
                        } else if meld.last().unwrap().rank == Rank::King {
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            let mut new_m = meld.to_vec();
                            new_m.push(ace_card);
                            valid_meld_seqs.push(new_m);
                            continue;
                        }
                    }
                    if !joker_cards.is_empty() {
                        let joker_card = joker_cards.pop().unwrap();
                        delete_invalid_index.insert(i);
                        let mut new_m = meld.to_vec();
                        new_m.push(joker_card);
                        valid_meld_seqs.push(new_m);
                        continue;
                    } else {
                        if with_memo {
                            self.dp_rank_seq[take_seq_index] = -1;
                        }
                        return -1;
                    }
                } else if meld.len() == 1 {
                    if !joker_cards.is_empty() || !aces_cards.is_empty() {
                        if !joker_cards.is_empty() {
                            // try last break
                            let mut break_index: Option<usize> = None;
                            let mut is_left = false;
                            let rank = rank_order(meld[0].rank)[0];

                            for (j, meld) in valid_meld_seqs.iter().enumerate() {
                                let rank_ll = rank_order(meld[0].rank)[0];
                                let rank_rr = rank_order(meld.last().unwrap().rank)[0];

                                if rank_ll <= rank - 2 && rank_rr >= rank + 1 {
                                    break_index = Some(j);
                                    is_left = false;
                                    break;
                                }
                                if rank_ll <= rank - 1 && rank_rr >= rank + 2 {
                                    is_left = true;
                                    break_index = Some(j);
                                }
                            }
                            if let Some(break_index) = break_index {
                                let cards = valid_meld_seqs.swap_remove(break_index);
                                delete_invalid_index.insert(i);
                                let joker_card = joker_cards.pop().unwrap();
                                let cut_index =
                                    cards.iter().position(|c| c.rank == meld[0].rank).unwrap();
                                let mut new_part1: Vec<&Card> = cards[0..=cut_index].to_vec();
                                let mut new_part2: Vec<&Card> = cards[cut_index + 1..].to_vec();
                                new_part2.insert(0, meld[0]);
                                if is_left {
                                    new_part1.push(joker_card);
                                } else {
                                    new_part2.push(joker_card);
                                }
                                valid_meld_seqs.push(new_part1);
                                valid_meld_seqs.push(new_part2);
                                continue;
                            }
                        }
                        if meld[0].rank == Rank::Number(2)
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![ace_card, meld[0], joker_card]);
                            continue;
                        } else if meld[0].rank == Rank::King
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![joker_card, meld[0], ace_card]);
                            continue;
                        } else if meld[0].rank == Rank::Number(3)
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![ace_card, joker_card, meld[0]]);
                        } else if meld[0].rank == Rank::Queen
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![meld[0], joker_card, ace_card]);
                        } else {
                            if with_memo {
                                self.dp_rank_seq[take_seq_index] = -1;
                            }
                            return -1;
                        }
                    } else {
                        if with_memo {
                            self.dp_rank_seq[take_seq_index] = -1;
                        }
                        return -1;
                    }
                }
            }
            let temp: Vec<Vec<&Card>> = invalid_meld_seqs
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !delete_invalid_index.contains(i))
                .map(|(_, m)| m)
                .collect();
            invalid_meld_seqs = temp;
            if !invalid_meld_seqs.is_empty() {
                if with_memo {
                    self.dp_rank_seq[take_seq_index] = -1;
                }
                return -1;
            }

            // Add ace cards
            while let Some(ace_card) = aces_cards.pop() {
                let mut add_to: Option<usize> = None;
                let mut is_left = false;

                for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                    if valid_meld[0].rank == Rank::Number(2)
                        || valid_meld[1].rank == Rank::Number(3)
                    {
                        add_to = Some(i);
                        is_left = true;
                        break;
                    }
                    if valid_meld.last().unwrap().rank == Rank::King
                        || valid_meld[valid_meld.len() - 2].rank == Rank::Queen
                    {
                        add_to = Some(i);
                        break;
                    }
                }
                if let Some(idx) = add_to {
                    if is_left {
                        valid_meld_seqs[idx].insert(0, ace_card);
                    } else {
                        valid_meld_seqs[idx].push(ace_card);
                    }
                } else {
                    if with_memo {
                        self.dp_rank_seq[take_seq_index] = -1;
                    }
                    return -1;
                }
            }

            // Add joker cards
            while let Some(joker_card) = joker_cards.pop() {
                let mut add_to: Option<usize> = None;
                let mut max_value = 0;
                let mut is_left = false;

                for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                    // check if valid_meld_seqs contains joker_card
                    let contains_joker = valid_meld.iter().any(|card| card.rank == Rank::Joker);
                    if contains_joker {
                        continue;
                    }
                    let last_card = valid_meld.last().unwrap();
                    if last_card.rank != Rank::Ace {
                        let val = match last_card.rank {
                            Rank::Number(n) if n >= 2 && n <= 9 => {
                                rank_order(last_card.rank)[0] + 1
                            }
                            Rank::King => 11,
                            _ => 10,
                        };
                        if val > max_value {
                            max_value = val;
                            add_to = Some(i);
                            is_left = false;
                        }
                    }
                    let first_card = valid_meld[0];
                    if first_card.rank != Rank::Ace {
                        let val = match first_card.rank {
                            Rank::Number(n) if n >= 3 && n <= 10 => {
                                rank_order(first_card.rank)[0] - 1
                            }
                            Rank::Number(2) => 11,
                            _ => 10,
                        };
                        if val > max_value {
                            max_value = val;
                            add_to = Some(i);
                            is_left = true;
                        }
                    }
                }
                if let Some(idx) = add_to {
                    if is_left {
                        valid_meld_seqs[idx].insert(0, joker_card);
                    } else {
                        valid_meld_seqs[idx].push(joker_card);
                    }
                } else {
                    if with_memo {
                        self.dp_rank_seq[take_seq_index] = -1;
                    }
                    return -1;
                }
            }

            all_melds.extend(valid_meld_seqs.into_iter().map(|cards| {
                let owned_cards: Vec<Card> = cards.into_iter().map(|c| (*c).clone()).collect();
                Meld {
                    id: Uuid::new_v4().to_string(),
                    cards: owned_cards,
                    meld_type: MeldType::Sequence,
                }
            }));
        }

        let res = melds_value(&all_melds);
        if with_memo {
            self.dp_rank_seq[take_seq_index] = res;
        }
        res
    }

    pub fn get_rank_meld_cards(&self, rank_cards: &Vec<Card>) -> Vec<Meld> {
        let mut melds: Vec<Meld> = Vec::new();
        let cards: Vec<Card> = rank_cards
            .iter()
            .filter(|c| c.rank != Rank::Joker)
            .cloned()
            .collect();
        let mut joker_cards: Vec<Card> = rank_cards
            .iter()
            .filter(|c| c.rank == Rank::Joker)
            .cloned()
            .collect();

        let groups = group_by_rank(&cards);

        // Track indices of melds in `melds` vector that can accept an extra joker
        let mut extension_candidates: Vec<(i32, usize)> = Vec::new();

        for (rank, group) in groups {
            let rank_val = rank_value_int(rank);
            let set_suit: HashSet<Suit> = group.iter().map(|c| c.suit).collect();
            let distinct_suits = set_suit.len();
            let count = group.len();

            match count {
                2 => {
                    // Need 1 joker
                    if let Some(j) = joker_cards.pop() {
                        let mut m_cards = group.clone();
                        m_cards.push(j);
                        melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: m_cards,
                            meld_type: MeldType::Rank,
                        });
                    }
                }
                3 => {
                    if distinct_suits == 3 {
                        melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: group.clone(),
                            meld_type: MeldType::Rank,
                        });
                        extension_candidates.push((rank_val, melds.len() - 1));
                    }
                }
                4 => {
                    if distinct_suits == 4 {
                        melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: group.clone(),
                            meld_type: MeldType::Rank,
                        });
                    } else if distinct_suits == 2 {
                        // 2 pairs. Need 2 jokers.
                        if joker_cards.len() >= 2 {
                            let j1 = joker_cards.pop().unwrap();
                            let j2 = joker_cards.pop().unwrap();

                            // Split group into two pairs of distinct suits
                            let mut by_suit: std::collections::HashMap<Suit, Vec<Card>> =
                                std::collections::HashMap::new();
                            for c in &group {
                                by_suit.entry(c.suit).or_default().push(c.clone());
                            }

                            let mut pair1 = Vec::new();
                            let mut pair2 = Vec::new();

                            for list in by_suit.values_mut() {
                                if let Some(c) = list.pop() {
                                    pair1.push(c);
                                }
                            }
                            for list in by_suit.values_mut() {
                                if let Some(c) = list.pop() {
                                    pair2.push(c);
                                }
                            }

                            pair1.push(j1);
                            pair2.push(j2);

                            melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: pair1,
                                meld_type: MeldType::Rank,
                            });
                            melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: pair2,
                                meld_type: MeldType::Rank,
                            });
                        }
                    }
                }
                5 => {
                    if distinct_suits == 4 || distinct_suits == 3 {
                        if joker_cards.len() >= 1 {
                            let j = joker_cards.pop().unwrap();
                            let mut m1 = Vec::new();
                            let mut m2 = Vec::new();
                            let mut suits_found = HashSet::new();
                            let mut used_indices = HashSet::new();

                            // First meld (3 distinct)
                            for (i, c) in group.iter().enumerate() {
                                if !suits_found.contains(&c.suit) {
                                    suits_found.insert(c.suit);
                                    used_indices.insert(i);
                                    m1.push(c.clone());
                                    if m1.len() == 3 {
                                        break;
                                    }
                                }
                            }
                            // Second meld (remainder + joker)
                            for (i, c) in group.iter().enumerate() {
                                if !used_indices.contains(&i) {
                                    m2.push(c.clone());
                                }
                            }
                            m2.push(j);

                            melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: m1,
                                meld_type: MeldType::Rank,
                            });
                            extension_candidates.push((rank_val, melds.len() - 1));

                            melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: m2,
                                meld_type: MeldType::Rank,
                            });
                        }
                    }
                }
                6 => {
                    if distinct_suits == 4 || distinct_suits == 3 {
                        let mut m1 = Vec::new();
                        let mut m2 = Vec::new();
                        let mut suits_found = HashSet::new();
                        let mut used_indices = HashSet::new();
                        // Clubs --> 1
                        // Diamonds --> 2
                        // Hearts --> 1
                        // Spades --> 2
                        for (i, c) in group.iter().enumerate() {
                            if !suits_found.contains(&c.suit) {
                                suits_found.insert(c.suit);
                            } else {
                                used_indices.insert(i);
                                m1.push(c.clone());
                                if m1.len() == 3 {
                                    break;
                                }
                            }
                        }
                        suits_found = m1.iter().map(|c| c.suit).collect();
                        for (i, c) in group.iter().enumerate() {
                            if !used_indices.contains(&i) {
                                if m1.len() == 3 || suits_found.contains(&c.suit) {
                                    m2.push(c.clone());
                                } else {
                                    m1.push(c.clone());
                                }
                            }
                        }
                        melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: m1,
                            meld_type: MeldType::Rank,
                        });
                        extension_candidates.push((rank_val, melds.len() - 1));

                        melds.push(Meld {
                            id: Uuid::new_v4().to_string(),
                            cards: m2,
                            meld_type: MeldType::Rank,
                        });
                        extension_candidates.push((rank_val, melds.len() - 1));
                    }
                }
                7 => {
                    let mut m1 = Vec::new();
                    let mut m2 = Vec::new();
                    let mut suits_found = HashSet::new();
                    let mut used_indices = HashSet::new();

                    for (i, c) in group.iter().enumerate() {
                        if !suits_found.contains(&c.suit) {
                            suits_found.insert(c.suit);
                            used_indices.insert(i);
                            m1.push(c.clone());
                            if m1.len() == 4 {
                                break;
                            }
                        }
                    }
                    for (i, c) in group.iter().enumerate() {
                        if !used_indices.contains(&i) {
                            m2.push(c.clone());
                        }
                    }
                    melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: m1,
                        meld_type: MeldType::Rank,
                    });
                    melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: m2,
                        meld_type: MeldType::Rank,
                    });
                    extension_candidates.push((rank_val, melds.len() - 1));
                }
                8 => {
                    let mut m1 = Vec::new();
                    let mut m2 = Vec::new();
                    let mut suits_found = HashSet::new();
                    let mut used_indices = HashSet::new();

                    for (i, c) in group.iter().enumerate() {
                        if !suits_found.contains(&c.suit) {
                            suits_found.insert(c.suit);
                            used_indices.insert(i);
                            m1.push(c.clone());
                            if m1.len() == 4 {
                                break;
                            }
                        }
                    }
                    for (i, c) in group.iter().enumerate() {
                        if !used_indices.contains(&i) {
                            m2.push(c.clone());
                        }
                    }
                    melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: m1,
                        meld_type: MeldType::Rank,
                    });
                    melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: m2,
                        meld_type: MeldType::Rank,
                    });
                }
                _ => {}
            }
        }

        extension_candidates.sort_by(|a, b| b.0.cmp(&a.0));

        for (_, meld_idx) in extension_candidates {
            if let Some(j) = joker_cards.pop() {
                if meld_idx < melds.len() {
                    melds[meld_idx].cards.push(j);
                }
            } else {
                break;
            }
        }

        melds
    }

    pub fn get_seq_meld_cards(&mut self, hand: &Vec<Card>, take_seq: &u32) -> Vec<Meld> {
        let mut joker_cards: Vec<&Card> = Vec::with_capacity(2);
        // Pre-allocate groups with proper capacity
        let mut groups: [Vec<&Card>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        for i in 0..hand.len() {
            if 1 & (take_seq >> i) == 0 {
                continue;
            }
            let card = &hand[i];

            if card.rank == Rank::Joker {
                joker_cards.push(card);
            } else {
                // Find or create group for this suit
                for group in &mut groups {
                    if group.is_empty() || group[0].suit == card.suit {
                        group.push(card);
                        break;
                    }
                }
            }
        }

        let mut all_melds: Vec<Meld> = Vec::with_capacity(15);

        for group in &groups {
            if group.is_empty() {
                continue;
            }

            let mut new_cards: Vec<&Card> = Vec::with_capacity(15);
            let mut aces_cards: Vec<&Card> = Vec::with_capacity(4);

            for &card in group {
                if card.rank == Rank::Ace {
                    aces_cards.push(card);
                } else {
                    new_cards.push(card);
                }
            }

            // Cache rank orders to avoid repeated function calls
            new_cards.sort_by_key(|card| rank_order(card.rank)[0]);

            let mut left_over_cards: Vec<&Card> = Vec::with_capacity(8);
            let mut meld_seqs: Vec<Vec<&Card>> = Vec::with_capacity(15);
            let mut meld: Vec<&Card> = vec![new_cards[0]];

            for i in 1..new_cards.len() {
                let prev_card = new_cards[i - 1];
                let card = new_cards[i];
                if card.rank == prev_card.rank {
                    left_over_cards.push(card);
                    continue;
                }
                if !valid_sequence_two_cards(prev_card, card, i == 1, i == new_cards.len() - 1) {
                    if !meld.is_empty() {
                        meld_seqs.push(meld);
                        meld = Vec::new();
                    }
                }
                meld.push(card);
            }
            if !meld.is_empty() {
                meld_seqs.push(meld);
            }

            if !left_over_cards.is_empty() {
                let mut meld = vec![left_over_cards[0]];
                for i in 1..left_over_cards.len() {
                    let prev_card = left_over_cards[i - 1];
                    let card = left_over_cards[i];
                    if !valid_sequence_two_cards(
                        prev_card,
                        card,
                        i == 1,
                        i == left_over_cards.len() - 1,
                    ) {
                        if !meld.is_empty() {
                            meld_seqs.push(meld);
                            meld = Vec::new();
                        }
                    }
                    meld.push(card);
                }
                if !meld.is_empty() {
                    meld_seqs.push(meld);
                }
            }

            let mut invalid_meld_seqs: Vec<Vec<&Card>> = Vec::with_capacity(15);
            let mut valid_meld_seqs: Vec<Vec<&Card>> = Vec::with_capacity(5);

            for meld in meld_seqs {
                if meld.len() > 2 {
                    valid_meld_seqs.push(meld);
                } else if !meld.is_empty() {
                    invalid_meld_seqs.push(meld);
                }
            }

            invalid_meld_seqs.sort_by_key(|m| m.len());

            // Break a sequence
            let mut delete_invalid_index: HashSet<usize> = HashSet::new();
            for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                if delete_invalid_index.contains(&i) {
                    continue;
                }
                let mut break_index: Option<usize> = None;

                let rank_l = rank_order(iv_meld[0].rank)[0];
                let rank_r = rank_order(iv_meld.last().unwrap().rank)[0];

                for (j, meld) in valid_meld_seqs.iter().enumerate() {
                    let rank_ll = rank_order(meld[0].rank)[0];
                    let rank_rr = rank_order(meld.last().unwrap().rank)[0];

                    if iv_meld.len() == 1 {
                        if rank_ll <= rank_l - 2 && rank_rr >= rank_r + 2 {
                            break_index = Some(j);
                        }
                    } else {
                        if rank_ll <= rank_l - 1 && rank_rr >= rank_r + 1 {
                            break_index = Some(j);
                            break;
                        }
                    }
                }

                if let Some(idx) = break_index {
                    let cards = valid_meld_seqs.swap_remove(idx);
                    delete_invalid_index.insert(i);
                    let cut_index = cards
                        .iter()
                        .position(|c| c.rank == iv_meld[0].rank)
                        .unwrap();
                    let mut new_part1: Vec<&Card> = cards[0..cut_index].to_vec();
                    new_part1.extend_from_slice(iv_meld);
                    let new_part2: Vec<&Card> = cards[cut_index..].to_vec();
                    valid_meld_seqs.push(new_part1);
                    valid_meld_seqs.push(new_part2);
                }
            }
            let temp: Vec<Vec<&Card>> = invalid_meld_seqs
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !delete_invalid_index.contains(i))
                .map(|(_, m)| m)
                .collect();
            invalid_meld_seqs = temp;

            // Connect a sequence
            delete_invalid_index.clear();
            for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                if delete_invalid_index.contains(&i) {
                    continue;
                }

                if !aces_cards.is_empty()
                    && iv_meld.len() == 2
                    && (iv_meld[0].rank == Rank::Number(2) || iv_meld[1].rank == Rank::King)
                {
                    continue;
                }

                let iv_last_rank = rank_order(iv_meld.last().unwrap().rank)[0];
                let iv_first_rank = rank_order(iv_meld[0].rank)[0];

                for (j, meld) in invalid_meld_seqs.iter().enumerate() {
                    if delete_invalid_index.contains(&j) || i == j {
                        continue;
                    }
                    if !aces_cards.is_empty()
                        && meld.len() == 2
                        && (meld[0].rank == Rank::Number(2) || meld[1].rank == Rank::King)
                    {
                        continue;
                    }

                    let meld_first_rank = rank_order(meld[0].rank)[0];
                    let meld_last_rank = rank_order(meld.last().unwrap().rank)[0];

                    if iv_last_rank + 2 == meld_first_rank {
                        let joker_card = joker_cards.pop().unwrap();
                        delete_invalid_index.insert(i);
                        delete_invalid_index.insert(j);
                        let mut combined: Vec<&Card> = iv_meld.to_vec();
                        combined.push(joker_card);
                        combined.extend_from_slice(meld);
                        valid_meld_seqs.push(combined);
                        break;
                    }

                    if iv_first_rank == meld_last_rank + 2 {
                        let joker_card = joker_cards.pop().unwrap();
                        delete_invalid_index.insert(i);
                        delete_invalid_index.insert(j);
                        let mut combined: Vec<&Card> = meld.to_vec();
                        combined.push(joker_card);
                        combined.extend_from_slice(iv_meld);
                        valid_meld_seqs.push(combined);
                        break;
                    }
                }
                if delete_invalid_index.contains(&i) {
                    continue;
                }

                let mut add_to: Option<usize> = None;
                let mut to_left = false;

                for (j, meld) in valid_meld_seqs.iter().enumerate() {
                    let meld_first_rank = rank_order(meld[0].rank)[0];
                    let meld_last_rank = rank_order(meld[meld.len() - 1].rank)[0];

                    if iv_last_rank + 2 == meld_first_rank {
                        add_to = Some(j);
                        to_left = true;
                        break;
                    }
                    if iv_last_rank == meld_last_rank + 2 {
                        add_to = Some(j);
                        to_left = false;
                        break;
                    }
                }

                if let Some(idx) = add_to {
                    delete_invalid_index.insert(i);
                    let valid_meld = valid_meld_seqs.remove(idx);
                    let joker_card = joker_cards.pop().unwrap();
                    if to_left {
                        let mut combined: Vec<&Card> = iv_meld.to_vec();
                        combined.push(joker_card);
                        combined.extend(valid_meld);
                        valid_meld_seqs.push(combined);
                    } else {
                        let mut combined = valid_meld;
                        combined.push(joker_card);
                        combined.extend_from_slice(iv_meld);
                        valid_meld_seqs.push(combined);
                    }
                }
            }
            let temp: Vec<Vec<&Card>> = invalid_meld_seqs
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !delete_invalid_index.contains(i))
                .map(|(_, m)| m)
                .collect();
            invalid_meld_seqs = temp;

            // Add needed Aces and Jokers
            delete_invalid_index.clear();
            for (i, meld) in invalid_meld_seqs.iter().enumerate() {
                if meld.len() == 2 {
                    if !aces_cards.is_empty() {
                        if meld[0].rank == Rank::Number(2) {
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            let mut new_m = vec![ace_card];
                            new_m.extend_from_slice(meld);
                            valid_meld_seqs.push(new_m);
                            continue;
                        } else if meld.last().unwrap().rank == Rank::King {
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            let mut new_m = meld.to_vec();
                            new_m.push(ace_card);
                            valid_meld_seqs.push(new_m);
                            continue;
                        }
                    }
                    if !joker_cards.is_empty() {
                        let joker_card = joker_cards.pop().unwrap();
                        delete_invalid_index.insert(i);
                        let mut new_m = meld.to_vec();
                        new_m.push(joker_card);
                        valid_meld_seqs.push(new_m);
                        continue;
                    }
                } else if meld.len() == 1 {
                    if !joker_cards.is_empty() || !aces_cards.is_empty() {
                        if !joker_cards.is_empty() {
                            // try last break
                            let mut break_index: Option<usize> = None;
                            let mut is_left = false;
                            let rank = rank_order(meld[0].rank)[0];

                            for (j, meld) in valid_meld_seqs.iter().enumerate() {
                                let rank_ll = rank_order(meld[0].rank)[0];
                                let rank_rr = rank_order(meld.last().unwrap().rank)[0];

                                if rank_ll <= rank - 2 && rank_rr >= rank + 1 {
                                    break_index = Some(j);
                                    is_left = false;
                                    break;
                                }
                                if rank_ll <= rank - 1 && rank_rr >= rank + 2 {
                                    is_left = true;
                                    break_index = Some(j);
                                }
                            }
                            if let Some(break_index) = break_index {
                                let cards = valid_meld_seqs.swap_remove(break_index);
                                delete_invalid_index.insert(i);
                                let joker_card = joker_cards.pop().unwrap();
                                let cut_index =
                                    cards.iter().position(|c| c.rank == meld[0].rank).unwrap();
                                let mut new_part1: Vec<&Card> = cards[0..=cut_index].to_vec();
                                let mut new_part2: Vec<&Card> = cards[cut_index + 1..].to_vec();
                                new_part2.insert(0, meld[0]);
                                if is_left {
                                    new_part1.push(joker_card);
                                } else {
                                    new_part2.push(joker_card);
                                }
                                valid_meld_seqs.push(new_part1);
                                valid_meld_seqs.push(new_part2);
                                continue;
                            }
                        }
                        if meld[0].rank == Rank::Number(2)
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![ace_card, meld[0], joker_card]);
                            continue;
                        } else if meld[0].rank == Rank::King
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![joker_card, meld[0], ace_card]);
                            continue;
                        } else if meld[0].rank == Rank::Number(3)
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![ace_card, joker_card, meld[0]]);
                        } else if meld[0].rank == Rank::Queen
                            && !aces_cards.is_empty()
                            && !joker_cards.is_empty()
                        {
                            let joker_card = joker_cards.pop().unwrap();
                            let ace_card = aces_cards.pop().unwrap();
                            delete_invalid_index.insert(i);
                            valid_meld_seqs.push(vec![meld[0], joker_card, ace_card]);
                        } else {
                            return vec![];
                        }
                    } else {
                        return vec![];
                    }
                }
            }
            let temp: Vec<Vec<&Card>> = invalid_meld_seqs
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !delete_invalid_index.contains(i))
                .map(|(_, m)| m)
                .collect();
            invalid_meld_seqs = temp;

            // Add ace cards
            while let Some(ace_card) = aces_cards.pop() {
                let mut add_to: Option<usize> = None;
                let mut is_left = false;

                for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                    if valid_meld[0].rank == Rank::Number(2)
                        || valid_meld[1].rank == Rank::Number(3)
                    {
                        add_to = Some(i);
                        is_left = true;
                        break;
                    }
                    if valid_meld.last().unwrap().rank == Rank::King
                        || valid_meld[valid_meld.len() - 2].rank == Rank::Queen
                    {
                        add_to = Some(i);
                        break;
                    }
                }
                if let Some(idx) = add_to {
                    if is_left {
                        valid_meld_seqs[idx].insert(0, ace_card);
                    } else {
                        valid_meld_seqs[idx].push(ace_card);
                    }
                }
            }

            // Add joker cards
            while let Some(joker_card) = joker_cards.pop() {
                let mut add_to: Option<usize> = None;
                let mut max_value = 0;
                let mut is_left = false;

                for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                    // check if valid_meld_seqs contains joker_card
                    let contains_joker = valid_meld.iter().any(|card| card.rank == Rank::Joker);
                    if contains_joker {
                        continue;
                    }
                    let last_card = valid_meld.last().unwrap();
                    if last_card.rank != Rank::Ace {
                        let val = match last_card.rank {
                            Rank::Number(n) if n >= 2 && n <= 9 => {
                                rank_order(last_card.rank)[0] + 1
                            }
                            Rank::King => 11,
                            _ => 10,
                        };
                        if val > max_value {
                            max_value = val;
                            add_to = Some(i);
                            is_left = false;
                        }
                    }
                    let first_card = valid_meld[0];
                    if first_card.rank != Rank::Ace {
                        let val = match first_card.rank {
                            Rank::Number(n) if n >= 3 && n <= 10 => {
                                rank_order(first_card.rank)[0] - 1
                            }
                            Rank::Number(2) => 11,
                            _ => 10,
                        };
                        if val > max_value {
                            max_value = val;
                            add_to = Some(i);
                            is_left = true;
                        }
                    }
                }
                if let Some(idx) = add_to {
                    if is_left {
                        valid_meld_seqs[idx].insert(0, joker_card);
                    } else {
                        valid_meld_seqs[idx].push(joker_card);
                    }
                }
            }

            all_melds.extend(valid_meld_seqs.into_iter().map(|cards| {
                let owned_cards: Vec<Card> = cards.into_iter().map(|c| (*c).clone()).collect();
                Meld {
                    id: Uuid::new_v4().to_string(),
                    cards: owned_cards,
                    meld_type: MeldType::Sequence,
                }
            }));
        }

        all_melds
    }

    pub fn calc(&mut self, hand: &Vec<Card>) {
        let n = hand.len() as u32;
        let limit = 1u32 << n;

        for take_rank in 0..limit {
            self.get_rank_meld(hand, &take_rank, true);
        }
        for take_seq in 0..limit {
            self.get_seq_meld(hand, &take_seq, true);
        }
        for take_rank in 0..limit {
            let rank_ones = take_rank.count_ones();
            if rank_ones < 3 && rank_ones != 0 {
                continue;
            }

            let rank_value = self.dp_rank_meld[take_rank as usize];
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
                        let size = rank_ones + seq_ones;
                        if size == n {
                            break 'check_value;
                        }

                        let seq_value = self.dp_rank_seq[take_seq as usize];
                        if seq_value == -1 {
                            break 'check_value;
                        }

                        let total_val = rank_value + seq_value;

                        if self.is_melded {
                            if self.max_cards_count < size as i32 {
                                self.max_cards_count = size as i32;
                                self.max_value = total_val;
                                self.best_take_rank = take_rank;
                                self.best_take_seq = take_seq;
                            } else if self.max_cards_count == size as i32 {
                                if total_val > self.max_value {
                                    self.max_value = total_val;
                                    self.best_take_rank = take_rank;
                                    self.best_take_seq = take_seq;
                                }
                            }
                        } else {
                            if self.max_value < total_val {
                                self.max_value = total_val;
                                self.max_cards_count = size as i32;
                                self.best_take_rank = take_rank;
                                self.best_take_seq = take_seq;
                            } else if self.max_value == total_val {
                                if (size as i32) > self.max_cards_count {
                                    self.max_cards_count = size as i32;
                                    self.best_take_rank = take_rank;
                                    self.best_take_seq = take_seq;
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

impl BotStrategy for UseJokerBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn clone_box(&self) -> Box<dyn BotStrategy> {
        Box::new(self.clone())
    }

    fn can_use_features(&self) -> &[String] {
        &self.features
    }
    fn decide_draw(&mut self, _state: &RoundState) -> DecideDrawResult {
        DecideDrawResult::Deck
    }

    fn find_melds(&mut self, hand: &Vec<Card>) -> Vec<Meld> {
        self.reset_calc_values();

        // let start = std::time::Instant::now();
        self.calc(hand);
        // println!("calc time: {:?}ms", start.elapsed().as_millis());

        if self.max_cards_count > 0 {
            let mut rank_cards = Vec::new();
            for i in 0..hand.len() {
                if (self.best_take_rank & (1 << i)) != 0 {
                    rank_cards.push(hand[i].clone());
                }
            }
            // Copy best_take_seq to avoid borrow checker issue
            let best_take_seq = self.best_take_seq;
            let seq_cards = self.get_seq_meld_cards(hand, &best_take_seq);
            self.meld_cards = self.get_rank_meld_cards(&rank_cards);
            // self.meld_cards.extend(self.get_seq_meld_cards(&seq_cards));
            self.meld_cards.extend(seq_cards);
        }

        self.meld_cards.clone()
    }

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        let player = &state.players[state.current_player];
        let melds = self.find_melds(&player.hand);
        if melds_value(&melds) < 51 && !player.melded {
            return vec![];
        }
        self.is_melded = true;
        melds
    }

    fn decide_play_in_meld(&mut self, state: &RoundState) -> (Phase, Option<Card>, bool, i32) {
        let player = &state.players[state.current_player];
        if !player.melded {
            return (Phase::Discard, None, false, -1);
        }
        if player.hand.len() == 1 {
            return (Phase::Discard, None, false, -1);
        }
        let mut meld_index = -1;
        let mut found_card: Option<Card> = None;
        let mut is_left = false;

        for card in &player.hand {
            let mut flag = false;
            for (index, meld) in state.table_melds.iter().enumerate() {
                if flag {
                    continue;
                }
                if meld.meld_type == MeldType::Sequence {
                    let (success, play_left, _) = can_play_in_sequence_meld(meld, card, true);
                    is_left = play_left;
                    if success {
                        flag = true;
                        meld_index = index as i32;
                    }
                }
                if meld.meld_type == MeldType::Rank {
                    let (success, _) = can_play_in_rank_meld(meld, card);
                    if success {
                        flag = true;
                        meld_index = index as i32;
                    }
                }
            }
            if flag {
                found_card = Some(card.clone());
                break;
            }
        }
        (
            if meld_index == -1 {
                Phase::Discard
            } else {
                Phase::PlayInMeld
            },
            found_card,
            is_left,
            meld_index,
        )
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        let hand = &state.players[state.current_player].hand;
        let melds = &self.meld_cards;
        let discard = hand.iter().find(|card| {
            !melds
                .iter()
                .any(|meld| meld.cards.iter().any(|c| c.id == card.id))
        });
        if let Some(c) = discard {
            hand.iter().position(|x| x.id == c.id).unwrap()
        } else {
            hand.len() - 1
        }
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
#[path = "./use_joker_test.rs"]
mod use_joker_test;
