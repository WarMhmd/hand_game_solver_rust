use std::any::Any;

use futures::future::OptionFuture;

use crate::bot::{BotStrategy, DecideDrawResult};
use crate::bots::use_fire::UseFireBot;
use crate::logic::{
    can_play_in_rank_meld, can_play_in_sequence_meld, check_seq_melds, Card, Meld, MeldType, Phase,
    Rank, RoundState,
};

#[derive(Clone, Debug)]
pub struct UseBetterMeldPlay {
    pub base: UseFireBot,
    features: Vec<String>,
}

impl UseBetterMeldPlay {
    pub fn new(name: String) -> Self {
        Self {
            base: UseFireBot::new(name),
            features: vec!["useMeld".to_string()],
        }
    }
}

impl BotStrategy for UseBetterMeldPlay {
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
        let player = &state.players[state.current_player];
        if !player.melded {
            return (Phase::Discard, None, false, -1);
        }
        if player.hand.len() == 1 {
            return (Phase::Discard, None, false, -1);
        }
        let mut possible_plays: Vec<(Phase, i32, Option<Card>, bool)> = vec![];
        let mut table_melds: Vec<Meld> = state.table_melds.clone();
        let mut hand = player.hand.clone();
        let mut skipped_hand = Vec::new();
        let mut changed = false;
        while hand.len() > 0 || changed {
            if hand.len() == 0 {
                if skipped_hand.len() == 0 {
                    break;
                }
                hand = skipped_hand;
                skipped_hand = Vec::new();
                changed = false;
            }
            let card = hand.pop().unwrap();
            if card.rank == Rank::Joker && !((2..=3).contains(&player.hand.len())) {
                skipped_hand.push(card);
                continue;
            }
            let mut max_score = 0;
            let mut best_possible_play: Option<(Phase, i32, Option<Card>, bool)> = None;
            for (index, meld) in table_melds.iter().enumerate() {
                if meld.meld_type == MeldType::Sequence {
                    let (success, play_left, take_joker) =
                        can_play_in_sequence_meld(meld, &card, true);
                    if success && take_joker {
                        max_score = 1000;
                        best_possible_play =
                            Some((Phase::Meld, index as i32, Some(card.clone()), play_left));
                        break;
                    }

                    if success {
                        let mut new_meld = meld.clone();
                        if play_left {
                            new_meld.cards.insert(0, card.clone());
                        } else {
                            new_meld.cards.push(card.clone());
                        }
                        let mut melds_after = check_seq_melds(&new_meld);
                        let mut score = 1;
                        for card_after in &hand {
                            if card_after.rank == Rank::Joker {
                                continue;
                            }
                            let mut m_index: i32 = -1;
                            let mut is_left = false;
                            for (index, meld) in melds_after.iter().enumerate() {
                                let foo = can_play_in_sequence_meld(meld, card_after, false);
                                if foo.0 {
                                    m_index = index as i32;
                                    is_left = foo.1;
                                    score += 1;
                                    break;
                                }
                            }
                            if m_index != -1 {
                                if is_left {
                                    melds_after[m_index as usize]
                                        .cards
                                        .insert(0, card_after.clone());
                                } else {
                                    melds_after[m_index as usize].cards.push(card_after.clone());
                                }
                            }
                        }
                        if score > max_score {
                            max_score = score;
                            best_possible_play = Some((
                                Phase::PlayInMeld,
                                index as i32,
                                Some(card.clone()),
                                play_left,
                            ));
                        }
                    }
                }
                if meld.meld_type == MeldType::Rank {
                    let (success, take_joker) = can_play_in_rank_meld(meld, &card);
                    if success && take_joker {
                        max_score = 1000;
                        best_possible_play =
                            Some((Phase::Meld, index as i32, Some(card.clone()), false));
                        break;
                    }

                    if success {
                        let mut new_meld = meld.clone();
                        new_meld.cards.push(card.clone());
                        let mut score = 1;
                        if new_meld.cards.len() == 4 {
                            for card_after in &player.hand {
                                if card_after.id == card.id {
                                    continue;
                                }
                                if card_after.rank == Rank::Joker {
                                    continue;
                                }
                                if can_play_in_rank_meld(&new_meld, card_after).1 {
                                    score += 900;
                                }
                            }
                        }
                        if max_score < score {
                            max_score = score;
                            best_possible_play =
                                Some((Phase::PlayInMeld, index as i32, Some(card.clone()), false));
                        }
                    }
                }
            }

            if max_score == 0 {
                skipped_hand.push(card);
            } else {
                changed = true;
                let best_possible_play = best_possible_play.unwrap();
                let meld: &Meld = &table_melds[best_possible_play.1 as usize];
                if meld.meld_type == MeldType::Rank {
                    table_melds[best_possible_play.1 as usize].cards.push(card);
                } else if meld.meld_type == MeldType::Sequence {
                    if best_possible_play.3 {
                        table_melds[best_possible_play.1 as usize]
                            .cards
                            .insert(0, card.clone());
                    } else {
                        table_melds[best_possible_play.1 as usize]
                            .cards
                            .push(card.clone());
                    }
                    let melds_after = check_seq_melds(&table_melds[best_possible_play.1 as usize]);
                    if melds_after.len() > 1 {
                        table_melds.remove(best_possible_play.1 as usize);
                        table_melds.extend(melds_after);
                    }
                }
                if max_score >= 900 {
                    // we took a joker
                    possible_plays.insert(0, best_possible_play);
                    break;
                } else {
                    possible_plays.push(best_possible_play);
                }
            }
        }

        if player.hand.len() - possible_plays.len() > 2 {
            if player.hand.len() == 4 as usize
                && (possible_plays.len() == 0 || possible_plays[0].0 != Phase::Meld)
            {
                return (Phase::Discard, None, false, -1);
            }
        }

        if possible_plays.len() == 0 {
            return (Phase::Discard, None, false, -1);
        } else {
            let decision = &possible_plays[0];
            (
                decision.0.clone(),
                decision.2.clone(),
                decision.3,
                decision.1,
            )
        }
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
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
#[path = "./better_meld_play_test.rs"]
mod better_meld_play_test;
