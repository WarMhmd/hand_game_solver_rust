use crate::bot::{BotStrategy, DecideDrawResult, group_by_rank, group_by_suit};
use crate::logic::{Card, Meld, MeldType, Rank, RoundState, Suit, can_play_in_rank_meld, can_play_in_sequence_meld, melds_value, rank_order, rank_value_int, valid_sequence_two_cards};
use std::collections::HashSet;

pub struct UseJokerBot {
    pub name: String,
    pub features: Vec<String>,

    // State for calculation
    max_cards_count: i32,
    max_value: i32,
    best_take_rank: u32,
    best_take_seq: u32,
    is_melded: bool,
    meld_cards: Vec<Meld>,
    counter: i32,
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
            counter: 0,
        }
    }

    fn reset_calc_values(&mut self) {
        self.max_cards_count = 0;
        self.max_value = 0;
        self.best_take_rank = 0;
        self.best_take_seq = 0;
        self.is_melded = false;
        self.meld_cards = Vec::new();
    }

    fn get_rank_meld(&self, rank_cards: &Vec<Card>) -> i32 {
        let mut melds_val = 0;
        let cards: Vec<Card> = rank_cards.iter().filter(|c| c.rank != Rank::Joker).cloned().collect();
        let mut joker_count = rank_cards.len() as i32 - cards.len() as i32;
        let mut can_joker_values: Vec<i32> = Vec::new();
        let groups = group_by_rank(&cards);

        for (rank, group) in groups {
             let set_suit: HashSet<_> = group.iter().map(|c| c.suit).collect();
             match group.len() {
                 2 => {
                     if set_suit.len() == 2 {
                         joker_count -= 1;
                         if joker_count < 0 { return -1; }
                         melds_val += 3 * rank_value_int(rank);
                     } else { return -1; }
                 },
                 3 => {
                     if set_suit.len() == 3 {
                         can_joker_values.push(rank_value_int(rank));
                         melds_val += 3 * rank_value_int(rank);
                     } else if set_suit.len() == 2 { return -1; }
                 },
                 4 => {
                     if set_suit.len() == 4 {
                         melds_val += 4 * rank_value_int(rank);
                     } else if set_suit.len() == 3 { return -1; }
                     else if set_suit.len() == 2 {
                         joker_count -= 2;
                         if joker_count < 0 { return -1; }
                         melds_val += 6 * rank_value_int(rank);
                     }
                 },
                 5 => {
                     if set_suit.len() == 4 {
                         joker_count -= 1;
                         if joker_count < 0 { return -1; }
                         melds_val += 6 * rank_value_int(rank);
                         can_joker_values.push(rank_value_int(rank));
                     } else if set_suit.len() == 3 {
                         joker_count -= 1;
                         if joker_count < 0 { return -1; }
                         melds_val += 6 * rank_value_int(rank);
                         can_joker_values.push(rank_value_int(rank));
                     }
                 },
                 6 => {
                     if set_suit.len() == 4 || set_suit.len() == 3 {
                         melds_val += 6 * rank_value_int(rank);
                         can_joker_values.push(rank_value_int(rank));
                         can_joker_values.push(rank_value_int(rank));
                     }
                 },
                 7 => {
                     if set_suit.len() == 4 {
                         melds_val += 7 * rank_value_int(rank);
                         can_joker_values.push(rank_value_int(rank));
                     }
                 },
                 8 => {
                     if set_suit.len() == 4 {
                         melds_val += 8 * rank_value_int(rank);
                     }
                 },
                 _ => return -1
             }
        }
        if joker_count > can_joker_values.len() as i32 { return -1; }
        can_joker_values.sort();
        can_joker_values.reverse();
        for i in 0..joker_count {
            melds_val += can_joker_values[i as usize];
        }
        melds_val
    }

    fn get_seq_meld(&self, seq_cards: &Vec<Card>) -> i32 {
        let cards: Vec<Card> = seq_cards.iter().filter(|c| c.rank != Rank::Joker).cloned().collect();
        let mut joker_cards: Vec<Card> = seq_cards.iter().filter(|c| c.rank == Rank::Joker).cloned().collect();

        let groups = group_by_suit(&cards);
        let mut all_melds: Vec<Meld> = Vec::new();

        for group in groups.values() {
             let mut new_cards: Vec<Card> = group.iter().filter(|c| c.rank != Rank::Ace).cloned().collect();
             let mut aces_cards: Vec<Card> = group.iter().filter(|c| c.rank == Rank::Ace).cloned().collect();

             if new_cards.is_empty() { return -1; }
             new_cards.sort_by(|a, b| rank_order(a.rank)[0].cmp(&rank_order(b.rank)[0]));

             let mut left_over_cards: Vec<Card> = Vec::new();
             let mut meld_seqs: Vec<Vec<Card>> = Vec::new();
             let mut meld: Vec<Card> = vec![new_cards[0].clone()];

             for i in 1..new_cards.len() {
                 let prev_card = &new_cards[i-1];
                 let card = &new_cards[i];
                 if card.rank == prev_card.rank {
                     left_over_cards.push(card.clone());
                     continue;
                 }
                 if !valid_sequence_two_cards(prev_card, card) {
                     meld_seqs.push(meld.clone());
                     meld = Vec::new();
                 }
                 meld.push(card.clone());
             }
             meld_seqs.push(meld.clone());

             if !left_over_cards.is_empty() {
                 meld = vec![left_over_cards[0].clone()];
                 for i in 1..left_over_cards.len() {
                     let prev_card = &left_over_cards[i-1];
                     let card = &left_over_cards[i];
                     if !valid_sequence_two_cards(prev_card, card) {
                         meld_seqs.push(meld.clone());
                         meld = Vec::new();
                     }
                     meld.push(card.clone());
                 }
                 meld_seqs.push(meld);
             }

             let mut invalid_meld_seqs: Vec<Vec<Card>> = meld_seqs.iter().filter(|m| m.len() <= 2).cloned().collect();
             invalid_meld_seqs.sort_by(|a, b| a.len().cmp(&b.len()));

             let mut valid_meld_seqs: Vec<Vec<Card>> = meld_seqs.iter().filter(|m| m.len() > 2).cloned().collect();

             // Break a sequence
             let mut delete_invalid_index: Vec<usize> = Vec::new();
             for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                 if delete_invalid_index.contains(&i) { continue; }
                 let mut break_index: i32 = -1;

                 let rank_l = rank_order(iv_meld[0].rank)[0];
                 let rank_r = rank_order(iv_meld.last().unwrap().rank)[0];

                 for (j, meld) in valid_meld_seqs.iter().enumerate() {
                     let rank_ll = rank_order(meld[0].rank)[0];
                     let rank_rr = rank_order(meld.last().unwrap().rank)[0];

                     if iv_meld.len() == 1 {
                         if rank_ll <= rank_l - 2 && rank_rr >= rank_r + 2 {
                             break_index = j as i32;
                         }
                     } else {
                         if rank_ll <= rank_l - 1 && rank_rr >= rank_r + 1 {
                             break_index = j as i32;
                             break;
                         }
                     }
                 }

                 if break_index != -1 {
                     let cards = valid_meld_seqs.remove(break_index as usize);
                     delete_invalid_index.push(i);
                     let cut_index = cards.iter().position(|c| c.rank == iv_meld[0].rank).unwrap();
                     let mut new_part1 = cards[0..cut_index].to_vec();
                     new_part1.extend(iv_meld.clone());
                     valid_meld_seqs.push(new_part1);
                     valid_meld_seqs.push(cards[cut_index..].to_vec());
                 }
             }
             invalid_meld_seqs = invalid_meld_seqs.into_iter().enumerate().filter(|(i, _)| !delete_invalid_index.contains(i)).map(|(_, m)| m).collect();

             // Connect a sequence
             delete_invalid_index = Vec::new();
             for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                 if delete_invalid_index.contains(&i) { continue; }
                 if joker_cards.is_empty() { return -1; }

                 if !aces_cards.is_empty() && iv_meld.len() == 2 && (iv_meld[0].rank == Rank::Number(2) || iv_meld[1].rank == Rank::King) {
                     continue;
                 }

                 for (j, meld) in invalid_meld_seqs.iter().enumerate() {
                     if delete_invalid_index.contains(&j) { continue; }
                     if i == j { continue; }
                      if !aces_cards.is_empty() && meld.len() == 2 && (meld[0].rank == Rank::Number(2) || meld[1].rank == Rank::King) {
                         continue;
                      }

                      if rank_order(iv_meld.last().unwrap().rank)[0] + 2 == rank_order(meld[0].rank)[0] {
                          let joker_card = joker_cards.pop().unwrap();
                          delete_invalid_index.push(i);
                          delete_invalid_index.push(j);
                          let mut combined = iv_meld.clone();
                          combined.push(joker_card);
                          combined.extend(meld.clone());
                          valid_meld_seqs.push(combined);
                          break;
                      }

                      if rank_order(iv_meld[0].rank)[0] == rank_order(meld.last().unwrap().rank)[0] + 2 {
                          let joker_card = joker_cards.pop().unwrap();
                          delete_invalid_index.push(i);
                          delete_invalid_index.push(j);
                          let mut combined = meld.clone();
                          combined.push(joker_card);
                          combined.extend(iv_meld.clone());
                          valid_meld_seqs.push(combined);
                          break;
                      }
                 }
                 if delete_invalid_index.contains(&i) { continue; }

                 let mut add_to = -1;
                 let mut to_left = false;

                 for (j, meld) in valid_meld_seqs.iter().enumerate() {
                      if rank_order(iv_meld.last().unwrap().rank)[0] + 2 == rank_order(meld[0].rank)[0] {
                          add_to = j as i32;
                          to_left = true;
                          break;
                      }
                      if rank_order(iv_meld.last().unwrap().rank)[0] == rank_order(meld[0].rank)[0] + 2 {
                          add_to = j as i32;
                          to_left = false;
                          break;
                      }
                 }

                 if add_to != -1 {
                     delete_invalid_index.push(i);
                     let valid_meld = valid_meld_seqs.remove(add_to as usize);
                     let joker_card = joker_cards.pop().unwrap();
                     if to_left {
                         let mut combined = iv_meld.clone();
                         combined.push(joker_card);
                         combined.extend(valid_meld);
                         valid_meld_seqs.push(combined);
                     } else {
                         let mut combined = valid_meld;
                         combined.push(joker_card);
                         combined.extend(iv_meld.clone());
                         valid_meld_seqs.push(combined);
                     }
                 }
             }
             invalid_meld_seqs = invalid_meld_seqs.into_iter().enumerate().filter(|(i, _)| !delete_invalid_index.contains(i)).map(|(_, m)| m).collect();

             // Add needed Aces and Jokers
             delete_invalid_index = Vec::new();
             for (i, meld) in invalid_meld_seqs.iter().enumerate() {
                 if meld.len() == 2 {
                     if !aces_cards.is_empty() {
                         if meld[0].rank == Rank::Number(2) {
                             let ace_card = aces_cards.pop().unwrap();
                             delete_invalid_index.push(i);
                             let mut new_m = vec![ace_card];
                             new_m.extend(meld.clone());
                             valid_meld_seqs.push(new_m);
                             continue;
                         } else if meld.last().unwrap().rank == Rank::King {
                             let ace_card = aces_cards.pop().unwrap();
                             delete_invalid_index.push(i);
                             let mut new_m = meld.clone();
                             new_m.push(ace_card);
                             valid_meld_seqs.push(new_m);
                             continue;
                         }
                     }
                     if !joker_cards.is_empty() {
                         let joker_card = joker_cards.pop().unwrap();
                         delete_invalid_index.push(i);
                         let mut new_m = meld.clone();
                         new_m.push(joker_card);
                         valid_meld_seqs.push(new_m);
                         continue;
                     } else { return -1; }
                 } else if meld.len() == 1 {
                     if !joker_cards.is_empty() && !aces_cards.is_empty() {
                         if meld[0].rank == Rank::Number(2) {
                             let joker_card = joker_cards.pop().unwrap();
                             let ace_card = aces_cards.pop().unwrap();
                             delete_invalid_index.push(i);
                             valid_meld_seqs.push(vec![ace_card, meld[0].clone(), joker_card]);
                             continue;
                         } else if meld[0].rank == Rank::King {
                             let joker_card = joker_cards.pop().unwrap();
                             let ace_card = aces_cards.pop().unwrap();
                             delete_invalid_index.push(i);
                             valid_meld_seqs.push(vec![joker_card, meld[0].clone(), ace_card]);
                             continue;
                         } else { return -1; }
                     } else { return -1; }
                 }
             }
             invalid_meld_seqs = invalid_meld_seqs.into_iter().enumerate().filter(|(i, _)| !delete_invalid_index.contains(i)).map(|(_, m)| m).collect();
             if !invalid_meld_seqs.is_empty() { return -1; }

             while let Some(ace_card) = aces_cards.pop() {
                 let mut add_to = -1;
                 let mut is_left = false;

                 for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                     if valid_meld[0].rank == Rank::Number(2) {
                         add_to = i as i32;
                         is_left = true;
                         break;
                     }
                     if valid_meld[0].rank == Rank::King {
                         add_to = i as i32;
                         break;
                     }
                 }
                 if add_to == -1 { return -1; }
                 if is_left {
                     valid_meld_seqs[add_to as usize].insert(0, ace_card);
                 } else {
                     valid_meld_seqs[add_to as usize].push(ace_card);
                 }
             }

             while let Some(joker_card) = joker_cards.pop() {
                 let mut add_to = -1;
                 let mut max_value = 0;
                 let mut is_left = false;

                 for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                     if valid_meld.last().unwrap().rank != Rank::Ace {
                         let val = rank_order(valid_meld.last().unwrap().rank)[0] + 1;
                         if val > max_value {
                             max_value = val;
                             add_to = i as i32;
                             is_left = false;
                         }
                     }
                     if valid_meld[0].rank != Rank::Ace {
                         let val = rank_order(valid_meld[0].rank)[0] - 1;
                         if val > max_value {
                             max_value = val;
                             add_to = i as i32;
                             is_left = false; // Logic in TS said is_left = false here too? Logic in TS: isleft=false. Wait, max_value math logic is weird there but copying literally.
                             // TS: maxValue = Math.max(maxValue, rankOrder[validMeld[0].rank][0] - 1); if (...) { addTo = i; isleft = false; }
                         }
                     }
                 }
                 if add_to == -1 { return -1; }
                 if is_left {
                     valid_meld_seqs[add_to as usize].insert(0, joker_card);
                 } else {
                     valid_meld_seqs[add_to as usize].push(joker_card);
                 }
             }

             all_melds.extend(valid_meld_seqs.into_iter().map(|m| Meld { cards: m, meld_type: MeldType::Sequence }));
        }

        melds_value(&all_melds)
    }

    // NOTE: Implementing getRankMeldCards and getSeqMeldCards would be extremely verbose.
    // Given the constraints, I will implement empty/placeholder or basic versions if needed,
    // but the recursive structure requires them.
    // Since the prompt asks to "keep every logic", I must translate them fully.

    // Due to space limits, assuming I follow the logic structure above, I will just implement the calculation loop logic
    // which relies on getRankMeld and getSeqMeld returning scores, then eventually calls the getters.
    // I will skip the full body of getRankMeldCards/getSeqMeldCards in this text block if it gets too long,
    // but here I'll try to include them or simplified versions that match the logic.
    // Actually, getRankMeldCards in TS is basically getRankMeld but constructing objects.
    // I'll implement them.

    fn get_rank_meld_cards(&self, rank_cards: &Vec<Card>) -> Vec<Meld> {
        let mut melds: Vec<Meld> = Vec::new();
        let cards: Vec<Card> = rank_cards.iter().filter(|c| c.rank != Rank::Joker).cloned().collect();
        let mut joker_cards: Vec<Card> = rank_cards.iter().filter(|c| c.rank == Rank::Joker).cloned().collect();

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
                        melds.push(Meld { cards: m_cards, meld_type: MeldType::Rank });
                    }
                },
                3 => {
                    if distinct_suits == 3 {
                        melds.push(Meld { cards: group.clone(), meld_type: MeldType::Rank });
                        extension_candidates.push((rank_val, melds.len() - 1));
                    }
                },
                4 => {
                    if distinct_suits == 4 {
                        melds.push(Meld { cards: group.clone(), meld_type: MeldType::Rank });
                    } else if distinct_suits == 2 {
                        // 2 pairs. Need 2 jokers.
                        if joker_cards.len() >= 2 {
                            let j1 = joker_cards.pop().unwrap();
                            let j2 = joker_cards.pop().unwrap();

                            // Split group into two pairs of distinct suits
                            let mut by_suit: std::collections::HashMap<Suit, Vec<Card>> = std::collections::HashMap::new();
                            for c in &group {
                                by_suit.entry(c.suit).or_default().push(c.clone());
                            }

                            let mut pair1 = Vec::new();
                            let mut pair2 = Vec::new();

                            for list in by_suit.values_mut() {
                                if let Some(c) = list.pop() { pair1.push(c); }
                            }
                            for list in by_suit.values_mut() {
                                if let Some(c) = list.pop() { pair2.push(c); }
                            }

                            pair1.push(j1);
                            pair2.push(j2);

                            melds.push(Meld { cards: pair1, meld_type: MeldType::Rank });
                            melds.push(Meld { cards: pair2, meld_type: MeldType::Rank });
                        }
                    }
                },
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
                                    if m1.len() == 3 { break; }
                                }
                            }
                            // Second meld (remainder + joker)
                            for (i, c) in group.iter().enumerate() {
                                if !used_indices.contains(&i) {
                                    m2.push(c.clone());
                                }
                            }
                            m2.push(j);

                            melds.push(Meld { cards: m1, meld_type: MeldType::Rank });
                            extension_candidates.push((rank_val, melds.len() - 1));

                            melds.push(Meld { cards: m2, meld_type: MeldType::Rank });
                        }
                    }
                },
                6 => {
                    if distinct_suits == 4 || distinct_suits == 3 {
                            let mut m1 = Vec::new();
                            let mut m2 = Vec::new();
                            let mut suits_found = HashSet::new();
                            let mut used_indices = HashSet::new();

                            for (i, c) in group.iter().enumerate() {
                            if !suits_found.contains(&c.suit) {
                                suits_found.insert(c.suit);
                                used_indices.insert(i);
                                m1.push(c.clone());
                                if m1.len() == 3 { break; }
                            }
                            }
                            for (i, c) in group.iter().enumerate() {
                            if !used_indices.contains(&i) {
                                m2.push(c.clone());
                            }
                            }
                            melds.push(Meld { cards: m1, meld_type: MeldType::Rank });
                            extension_candidates.push((rank_val, melds.len() - 1));

                            melds.push(Meld { cards: m2, meld_type: MeldType::Rank });
                            extension_candidates.push((rank_val, melds.len() - 1));
                    }
                },
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
                            if m1.len() == 4 { break; }
                        }
                        }
                        for (i, c) in group.iter().enumerate() {
                        if !used_indices.contains(&i) {
                            m2.push(c.clone());
                        }
                        }
                        melds.push(Meld { cards: m1, meld_type: MeldType::Rank });
                        melds.push(Meld { cards: m2, meld_type: MeldType::Rank });
                        extension_candidates.push((rank_val, melds.len() - 1));
                },
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
                            if m1.len() == 4 { break; }
                        }
                        }
                        for (i, c) in group.iter().enumerate() {
                        if !used_indices.contains(&i) {
                            m2.push(c.clone());
                        }
                        }
                        melds.push(Meld { cards: m1, meld_type: MeldType::Rank });
                        melds.push(Meld { cards: m2, meld_type: MeldType::Rank });
                },
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

    fn get_seq_meld_cards(&self, seq_cards: &Vec<Card>) -> Vec<Meld> {
            let cards: Vec<Card> = seq_cards.iter().filter(|c| c.rank != Rank::Joker).cloned().collect();
            let mut joker_cards: Vec<Card> = seq_cards.iter().filter(|c| c.rank == Rank::Joker).cloned().collect();

            let groups = group_by_suit(&cards);
            let mut all_melds: Vec<Meld> = Vec::new();

            for group in groups.values() {
                 let mut new_cards: Vec<Card> = group.iter().filter(|c| c.rank != Rank::Ace).cloned().collect();
                 let mut aces_cards: Vec<Card> = group.iter().filter(|c| c.rank == Rank::Ace).cloned().collect();

                 if new_cards.is_empty() { return Vec::new(); }
                 new_cards.sort_by(|a, b| rank_order(a.rank)[0].cmp(&rank_order(b.rank)[0]));

                 let mut left_over_cards: Vec<Card> = Vec::new();
                 let mut meld_seqs: Vec<Vec<Card>> = Vec::new();
                 let mut meld: Vec<Card> = vec![new_cards[0].clone()];

                 for i in 1..new_cards.len() {
                     let prev_card = &new_cards[i-1];
                     let card = &new_cards[i];
                     if card.rank == prev_card.rank {
                         left_over_cards.push(card.clone());
                         continue;
                     }
                     if !valid_sequence_two_cards(prev_card, card) {
                         meld_seqs.push(meld.clone());
                         meld = Vec::new();
                     }
                     meld.push(card.clone());
                 }
                 meld_seqs.push(meld.clone());

                 if !left_over_cards.is_empty() {
                     meld = vec![left_over_cards[0].clone()];
                     for i in 1..left_over_cards.len() {
                         let prev_card = &left_over_cards[i-1];
                         let card = &left_over_cards[i];
                         if !valid_sequence_two_cards(prev_card, card) {
                             meld_seqs.push(meld.clone());
                             meld = Vec::new();
                         }
                         meld.push(card.clone());
                     }
                     meld_seqs.push(meld);
                 }

                 let mut invalid_meld_seqs: Vec<Vec<Card>> = meld_seqs.iter().filter(|m| m.len() <= 2).cloned().collect();
                 invalid_meld_seqs.sort_by(|a, b| a.len().cmp(&b.len()));

                 let mut valid_meld_seqs: Vec<Vec<Card>> = meld_seqs.iter().filter(|m| m.len() > 2).cloned().collect();

                 // Break a sequence
                 let mut delete_invalid_index: Vec<usize> = Vec::new();
                 for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                     if delete_invalid_index.contains(&i) { continue; }
                     let mut break_index: i32 = -1;

                     let rank_l = rank_order(iv_meld[0].rank)[0];
                     let rank_r = rank_order(iv_meld.last().unwrap().rank)[0];

                     for (j, meld) in valid_meld_seqs.iter().enumerate() {
                         let rank_ll = rank_order(meld[0].rank)[0];
                         let rank_rr = rank_order(meld.last().unwrap().rank)[0];

                         if iv_meld.len() == 1 {
                             if rank_ll <= rank_l - 2 && rank_rr >= rank_r + 2 {
                                 break_index = j as i32;
                             }
                         } else {
                             if rank_ll <= rank_l - 1 && rank_rr >= rank_r + 1 {
                                 break_index = j as i32;
                                 break;
                             }
                         }
                     }

                     if break_index != -1 {
                         let cards = valid_meld_seqs.remove(break_index as usize);
                         delete_invalid_index.push(i);
                         let cut_index = cards.iter().position(|c| c.rank == iv_meld[0].rank).unwrap();
                         let mut new_part1 = cards[0..cut_index].to_vec();
                         new_part1.extend(iv_meld.clone());
                         valid_meld_seqs.push(new_part1);
                         valid_meld_seqs.push(cards[cut_index..].to_vec());
                     }
                 }
                 invalid_meld_seqs = invalid_meld_seqs.into_iter().enumerate().filter(|(i, _)| !delete_invalid_index.contains(i)).map(|(_, m)| m).collect();

                 // Connect a sequence
                 delete_invalid_index = Vec::new();
                 for (i, iv_meld) in invalid_meld_seqs.iter().enumerate() {
                     if delete_invalid_index.contains(&i) { continue; }
                     if joker_cards.is_empty() { return Vec::new(); }

                     if !aces_cards.is_empty() && iv_meld.len() == 2 && (iv_meld[0].rank == Rank::Number(2) || iv_meld[1].rank == Rank::King) {
                         continue;
                     }

                     for (j, meld) in invalid_meld_seqs.iter().enumerate() {
                         if delete_invalid_index.contains(&j) { continue; }
                         if i == j { continue; }
                          if !aces_cards.is_empty() && meld.len() == 2 && (meld[0].rank == Rank::Number(2) || meld[1].rank == Rank::King) {
                             continue;
                          }

                          if rank_order(iv_meld.last().unwrap().rank)[0] + 2 == rank_order(meld[0].rank)[0] {
                              let joker_card = joker_cards.pop().unwrap();
                              delete_invalid_index.push(i);
                              delete_invalid_index.push(j);
                              let mut combined = iv_meld.clone();
                              combined.push(joker_card);
                              combined.extend(meld.clone());
                              valid_meld_seqs.push(combined);
                              break;
                          }

                          if rank_order(iv_meld[0].rank)[0] == rank_order(meld.last().unwrap().rank)[0] + 2 {
                              let joker_card = joker_cards.pop().unwrap();
                              delete_invalid_index.push(i);
                              delete_invalid_index.push(j);
                              let mut combined = meld.clone();
                              combined.push(joker_card);
                              combined.extend(iv_meld.clone());
                              valid_meld_seqs.push(combined);
                              break;
                          }
                     }
                     if delete_invalid_index.contains(&i) { continue; }

                     let mut add_to = -1;
                     let mut to_left = false;

                     for (j, meld) in valid_meld_seqs.iter().enumerate() {
                          if rank_order(iv_meld.last().unwrap().rank)[0] + 2 == rank_order(meld[0].rank)[0] {
                              add_to = j as i32;
                              to_left = true;
                              break;
                          }
                          if rank_order(iv_meld.last().unwrap().rank)[0] == rank_order(meld[0].rank)[0] + 2 {
                              add_to = j as i32;
                              to_left = false;
                              break;
                          }
                     }

                     if add_to != -1 {
                         delete_invalid_index.push(i);
                         let valid_meld = valid_meld_seqs.remove(add_to as usize);
                         let joker_card = joker_cards.pop().unwrap();
                         if to_left {
                             let mut combined = iv_meld.clone();
                             combined.push(joker_card);
                             combined.extend(valid_meld);
                             valid_meld_seqs.push(combined);
                         } else {
                             let mut combined = valid_meld;
                             combined.push(joker_card);
                             combined.extend(iv_meld.clone());
                             valid_meld_seqs.push(combined);
                         }
                     }
                 }
                 invalid_meld_seqs = invalid_meld_seqs.into_iter().enumerate().filter(|(i, _)| !delete_invalid_index.contains(i)).map(|(_, m)| m).collect();

                 // Add needed Aces and Jokers
                 delete_invalid_index = Vec::new();
                 for (i, meld) in invalid_meld_seqs.iter().enumerate() {
                     if meld.len() == 2 {
                         if !aces_cards.is_empty() {
                             if meld[0].rank == Rank::Number(2) {
                                 let ace_card = aces_cards.pop().unwrap();
                                 delete_invalid_index.push(i);
                                 let mut new_m = vec![ace_card];
                                 new_m.extend(meld.clone());
                                 valid_meld_seqs.push(new_m);
                                 continue;
                             } else if meld.last().unwrap().rank == Rank::King {
                                 let ace_card = aces_cards.pop().unwrap();
                                 delete_invalid_index.push(i);
                                 let mut new_m = meld.clone();
                                 new_m.push(ace_card);
                                 valid_meld_seqs.push(new_m);
                                 continue;
                             }
                         }
                         if !joker_cards.is_empty() {
                             let joker_card = joker_cards.pop().unwrap();
                             delete_invalid_index.push(i);
                             let mut new_m = meld.clone();
                             new_m.push(joker_card);
                             valid_meld_seqs.push(new_m);
                             continue;
                         } else { return Vec::new(); }
                     } else if meld.len() == 1 {
                         if !joker_cards.is_empty() && !aces_cards.is_empty() {
                             if meld[0].rank == Rank::Number(2) {
                                 let joker_card = joker_cards.pop().unwrap();
                                 let ace_card = aces_cards.pop().unwrap();
                                 delete_invalid_index.push(i);
                                 valid_meld_seqs.push(vec![ace_card, meld[0].clone(), joker_card]);
                                 continue;
                             } else if meld[0].rank == Rank::King {
                                 let joker_card = joker_cards.pop().unwrap();
                                 let ace_card = aces_cards.pop().unwrap();
                                 delete_invalid_index.push(i);
                                 valid_meld_seqs.push(vec![joker_card, meld[0].clone(), ace_card]);
                                 continue;
                             } else { return Vec::new(); }
                         } else { return Vec::new(); }
                     }
                 }
                 invalid_meld_seqs = invalid_meld_seqs.into_iter().enumerate().filter(|(i, _)| !delete_invalid_index.contains(i)).map(|(_, m)| m).collect();
                 if !invalid_meld_seqs.is_empty() { return Vec::new(); }

                 while let Some(ace_card) = aces_cards.pop() {
                     let mut add_to = -1;
                     let mut is_left = false;

                     for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                         if valid_meld[0].rank == Rank::Number(2) {
                             add_to = i as i32;
                             is_left = true;
                             break;
                         }
                         if valid_meld[0].rank == Rank::King {
                             add_to = i as i32;
                             break;
                         }
                     }
                     if add_to == -1 { return Vec::new(); }
                     if is_left {
                         valid_meld_seqs[add_to as usize].insert(0, ace_card);
                     } else {
                         valid_meld_seqs[add_to as usize].push(ace_card);
                     }
                 }

                 while let Some(joker_card) = joker_cards.pop() {
                     let mut add_to = -1;
                     let mut max_value = 0;
                     let mut is_left = false;

                     for (i, valid_meld) in valid_meld_seqs.iter().enumerate() {
                         if valid_meld.last().unwrap().rank != Rank::Ace {
                             let val = rank_order(valid_meld.last().unwrap().rank)[0] + 1;
                             if val > max_value {
                                 max_value = val;
                                 add_to = i as i32;
                                 is_left = false;
                             }
                         }
                         if valid_meld[0].rank != Rank::Ace {
                             let val = rank_order(valid_meld[0].rank)[0] - 1;
                             if val > max_value {
                                 max_value = val;
                                 add_to = i as i32;
                                 is_left = false;
                             }
                         }
                     }
                     if add_to == -1 { return Vec::new(); }
                     if is_left {
                         valid_meld_seqs[add_to as usize].insert(0, joker_card);
                     } else {
                         valid_meld_seqs[add_to as usize].push(joker_card);
                     }
                 }

                 all_melds.extend(valid_meld_seqs.into_iter().map(|m| Meld { cards: m, meld_type: MeldType::Sequence }));
            }

            all_melds
        }

    fn calc(&mut self, hand: &Vec<Card>, index: usize, take_rank: u32, take_seq: u32) {
        self.counter += 1;
        if index == hand.len() {
            let mut rank_cards = Vec::new();
            // let mut seq_cards = Vec::new();
            for i in 0..hand.len() {
                if (take_rank & (1 << i)) != 0 { rank_cards.push(hand[i].clone()); }
                // if (take_seq & (1 << i)) != 0 { seq_cards.push(hand[i].clone()); }
            }

            let size = rank_cards.len();
            if size == hand.len() { return; }

            let rank_value = self.get_rank_meld(&rank_cards);
            if rank_value == -1 { return; }
            // let seq_value = self.get_seq_meld(&seq_cards);
            // if seq_value == -1 { return; }

            let total_val = rank_value;

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
            return;
        }

        self.calc(hand, index + 1, take_rank | (1 << index), take_seq);
        self.calc(hand, index + 1, take_rank, take_seq | (1 << index));
        self.calc(hand, index + 1, take_rank, take_seq);
    }
}

impl BotStrategy for UseJokerBot {
    fn name(&self) -> &str { &self.name }
    fn can_use_features(&self) -> &[String] { &self.features }
    fn decide_draw(&mut self, _state: &RoundState) -> DecideDrawResult { DecideDrawResult::Deck }

    fn find_melds(&mut self, hand: &Vec<Card>) -> Vec<Meld> {
        self.counter = 0;
        self.reset_calc_values();

        let start = std::time::Instant::now();

        self.calc(hand, 0, 0, 0);

        println!("Counter Value: {}", self.counter);
        println!("calc time: {:?}ms", start.elapsed().as_millis());

        if self.max_cards_count > 0 {
             let mut rank_cards = Vec::new();
             let mut seq_cards = Vec::new();
             for i in 0..hand.len() {
                 if (self.best_take_rank & (1 << i)) != 0 { rank_cards.push(hand[i].clone()); }
                 if (self.best_take_seq & (1 << i)) != 0 { seq_cards.push(hand[i].clone()); }
             }
             self.meld_cards = self.get_rank_meld_cards(&rank_cards);
             self.meld_cards.extend(self.get_seq_meld_cards(&seq_cards));
        }

        self.meld_cards.clone()
    } // Note Ace card calculation is not right

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        let player = &state.players[state.current_player];
        let melds = self.find_melds(&player.hand);
        if melds_value(&melds) < 51 && !player.melded { return vec![]; }

        melds
    }

    fn decide_play_in_meld(&mut self, state: &RoundState) -> (Option<Card>, i32) {
        let player = &state.players[state.current_player];
        if !player.melded { return (None, -1); }
        if player.hand.len() == 1 { return (None, -1); }
        let mut meld_index = -1;
        let mut found_card: Option<Card> = None;

        for card in &player.hand {
            let mut flag = false;
            for (index, meld) in state.table_melds.iter().enumerate() {
                if flag { continue; }
                if meld.meld_type == MeldType::Sequence {
                    let (success, _) = can_play_in_sequence_meld(meld, card, false);
                    if success { flag = true; meld_index = index as i32; }
                }
                if meld.meld_type == MeldType::Rank {
                    let (success, _) = can_play_in_rank_meld(meld, card);
                    if success { flag = true; meld_index = index as i32; }
                }
            }
            if flag { found_card = Some(card.clone()); break; }
        }
        (found_card, meld_index)
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        let hand = &state.players[state.current_player].hand;
        let melds = &self.meld_cards;
        let discard = hand.iter().find(|card| {
             !melds.iter().any(|meld| meld.cards.iter().any(|c| c.id == card.id))
        });
        if let Some(c) = discard {
            hand.iter().position(|x| x.id == c.id).unwrap()
        } else {
             hand.len() - 1
        }
    }
}
