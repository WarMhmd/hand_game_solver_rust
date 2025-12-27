use std::any::Any;

use uuid::Uuid;

use crate::bot::{group_by_rank, group_by_suit, BotStrategy, DecideDrawResult};
use crate::logic::{
    melds_value, rank_order, valid_sequence_two_cards, Card, Meld, MeldType, Rank, RoundState,
};

#[derive(Debug, Clone)]
pub struct SequenceBot {
    pub name: String,
    pub features: Vec<String>,
}

impl SequenceBot {
    pub fn new(name: String) -> Self {
        Self {
            name,
            features: vec![],
        }
    }
}

impl BotStrategy for SequenceBot {
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
        let mut melds: Vec<Meld> = Vec::new();
        let groups = group_by_rank(hand);

        for cards in groups.values() {
            if cards.len() >= 3 {
                let mut meld: Vec<Card> = Vec::new();
                for card in cards {
                    if meld.is_empty() || meld.iter().all(|c| c.suit != card.suit) {
                        meld.push(card.clone());
                        if meld.len() == 4 {
                            melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: meld.clone(),
                                meld_type: MeldType::Rank,
                            });
                            meld = Vec::new();
                        }
                    }
                }
                if meld.len() >= 3 {
                    melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: meld,
                        meld_type: MeldType::Rank,
                    });
                }
            }
        }

        let hand_after_melds: Vec<Card> = hand
            .iter()
            .filter(|card| {
                !melds
                    .iter()
                    .any(|meld| meld.cards.iter().any(|c| c.id == card.id))
            })
            .cloned()
            .collect();

        // check for sequences
        let suit_groups = group_by_suit(&hand_after_melds);
        for cards in suit_groups.values() {
            if cards.len() >= 3 {
                let mut sorted_cards = cards.clone();
                // sort cards by rankOrder
                sorted_cards.sort_by(|a, b| rank_order(a.rank)[0].cmp(&rank_order(b.rank)[0]));

                let mut meld: Vec<Card> = Vec::new();
                let mut first_melds: Vec<Meld> = Vec::new();

                meld.push(sorted_cards[0].clone());
                for i in 1..sorted_cards.len() {
                    let card = &sorted_cards[i];
                    let prev_card = &sorted_cards[i - 1];
                    if !valid_sequence_two_cards(
                        prev_card,
                        card,
                        i == 1,
                        i == sorted_cards.len() - 1,
                    ) {
                        if meld.len() >= 3 {
                            first_melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: meld.clone(),
                                meld_type: MeldType::Sequence,
                            });
                        }
                        meld = Vec::new();
                    }
                    meld.push(card.clone());
                }
                if meld.len() >= 3 {
                    first_melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: meld.clone(),
                        meld_type: MeldType::Sequence,
                    });
                }

                let first_meld_cards_count: usize = first_melds.iter().map(|m| m.cards.len()).sum();

                sorted_cards.sort_by(|a, b| {
                    let r_a = if a.rank == Rank::Ace {
                        rank_order(a.rank)[1]
                    } else {
                        rank_order(a.rank)[0]
                    };
                    let r_b = if b.rank == Rank::Ace {
                        rank_order(b.rank)[1]
                    } else {
                        rank_order(b.rank)[0]
                    };
                    r_a.cmp(&r_b)
                });

                let mut second_melds: Vec<Meld> = Vec::new();
                meld = Vec::new();
                meld.push(sorted_cards[0].clone());
                for i in 1..sorted_cards.len() {
                    let card = &sorted_cards[i];
                    let prev_card = &sorted_cards[i - 1];
                    if !valid_sequence_two_cards(
                        prev_card,
                        card,
                        i == 1,
                        i == sorted_cards.len() - 1,
                    ) {
                        if meld.len() >= 3 {
                            second_melds.push(Meld {
                                id: Uuid::new_v4().to_string(),
                                cards: meld.clone(),
                                meld_type: MeldType::Sequence,
                            });
                        }
                        meld = Vec::new();
                    }
                    meld.push(card.clone());
                }
                if meld.len() >= 3 {
                    second_melds.push(Meld {
                        id: Uuid::new_v4().to_string(),
                        cards: meld.clone(),
                        meld_type: MeldType::Sequence,
                    });
                }

                let second_meld_cards_count: usize =
                    second_melds.iter().map(|m| m.cards.len()).sum();

                if first_meld_cards_count > second_meld_cards_count {
                    melds.extend(first_melds);
                } else {
                    melds.extend(second_melds);
                }
            }
        }

        melds
    }

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        let player = &state.players[state.current_player];
        let melds = self.find_melds(&player.hand);
        if melds_value(&melds) < 51 && !player.melded {
            return vec![];
        }

        let player_cards_num_after_meld =
            player.hand.len() as i32 - melds.iter().map(|m| m.cards.len() as i32).sum::<i32>();
        if player_cards_num_after_meld == 3
            || player_cards_num_after_meld == 2
            || player_cards_num_after_meld == 0
        {
            return vec![];
        }
        melds
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        let hand = &state.players[state.current_player].hand;
        let melds = self.find_melds(hand);
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
