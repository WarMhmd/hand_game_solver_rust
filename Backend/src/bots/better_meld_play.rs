use std::any::Any;

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
        let mut possible_plays: Vec<(i32, Phase, i32, Option<Card>, bool)> =
            vec![(0, Phase::Discard, -1, None, false)];
        for card in &player.hand {
            if card.rank == Rank::Joker && !(2..=3).contains(&player.hand.len()) {
                continue;
            }
            for (index, meld) in state.table_melds.iter().enumerate() {
                if meld.meld_type == MeldType::Sequence {
                    let (success, play_left, take_joker) =
                        can_play_in_sequence_meld(meld, card, true);
                    if success && take_joker {
                        possible_plays.push((
                            1000,
                            Phase::Meld,
                            index as i32,
                            Some(card.clone()),
                            play_left,
                        ));
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
                        for card_after in &player.hand {
                            if card_after.id == card.id {
                                continue;
                            }
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
                        possible_plays.push((
                            score,
                            Phase::PlayInMeld,
                            index as i32,
                            Some(card.clone()),
                            play_left,
                        ));
                    }
                }
                if meld.meld_type == MeldType::Rank {
                    let (success, take_joker) = can_play_in_rank_meld(meld, card);
                    if success && take_joker {
                        possible_plays.push((
                            1000,
                            Phase::Meld,
                            index as i32,
                            Some(card.clone()),
                            false,
                        ));
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
                        possible_plays.push((
                            score,
                            Phase::PlayInMeld,
                            index as i32,
                            Some(card.clone()),
                            false,
                        ));
                    }
                }
            }
        }
        possible_plays.sort_by_key(|f| -f.0);
        let decision = &possible_plays[0];
        if decision.0 == 1000 {
            self.base.is_fire_card = false;
        }
        if decision.0 != 1000 || decision.0 != 901 {
            // the card is not joker
            if player.hand.len() == 4 && decision.0 == 1 {
                return (Phase::Discard, None, false, -1);
            }
            if player.hand.len() == 5 && decision.0 == 2 {
                return (Phase::Discard, None, false, -1);
            }
        }
        (
            decision.1.clone(),
            decision.3.clone(),
            decision.4,
            decision.2,
        )
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
