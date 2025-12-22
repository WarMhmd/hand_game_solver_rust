use crate::logic::{Card, Meld, RoundState, Rank, Suit};
use std::collections::HashMap;

pub enum DecideDrawResult {
    Deck,
    Fire,
}

pub trait BotStrategy {
    fn name(&self) -> &str;
    fn can_use_features(&self) -> &[String];
    fn decide_draw(&mut self, state: &RoundState) -> DecideDrawResult;
    fn find_melds(&mut self, hand: &Vec<Card>) -> Vec<Meld>;
    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld>;
    fn decide_discard(&mut self, state: &RoundState) -> usize;
    // For UseMeldsStrategy
    fn decide_play_in_meld(&mut self, _state: &RoundState) -> (Option<Card>, i32) {
        (None, -1)
    }
}

// --------------------
// Helper utilities
// --------------------

pub fn group_by_rank(hand: &Vec<Card>) -> HashMap<Rank, Vec<Card>> {
    let mut map = HashMap::new();
    for c in hand {
        map.entry(c.rank).or_insert_with(Vec::new).push(c.clone());
    }
    map
}

pub fn group_by_suit(hand: &Vec<Card>) -> HashMap<Suit, Vec<Card>> {
    let mut map = HashMap::new();
    for c in hand {
        map.entry(c.suit).or_insert_with(Vec::new).push(c.clone());
    }
    map
}

// --------------------
// Bot strategies
// --------------------

// 1- Random Bot
pub struct RandomBot {
    pub name: String,
    pub features: Vec<String>,
}

impl RandomBot {
    pub fn new(name: String) -> Self {
        Self { name, features: vec![] }
    }
}

impl BotStrategy for RandomBot {
    fn name(&self) -> &str {
        &self.name
    }
    fn can_use_features(&self) -> &[String] {
        &self.features
    }
    fn decide_draw(&mut self, _state: &RoundState) -> DecideDrawResult {
        DecideDrawResult::Deck
    }
    fn find_melds(&mut self, _hand: &Vec<Card>) -> Vec<Meld> {
        vec![]
    }
    fn decide_melds(&mut self, _state: &RoundState) -> Vec<Meld> {
        vec![]
    }
    fn decide_discard(&mut self, state: &RoundState) -> usize {
        let hand = &state.players[state.current_player].hand;
        rand::random::<usize>() % hand.len()
    }
}

// 4- Aggressive Bot (RankBot)
pub struct RankBot {
    pub name: String,
    pub features: Vec<String>,
}

impl RankBot {
    pub fn new(name: String) -> Self {
        Self { name, features: vec![] }
    }
}

impl BotStrategy for RankBot {
    fn name(&self) -> &str {
        &self.name
    }
    fn can_use_features(&self) -> &[String] {
        &self.features
    }
    fn decide_draw(&mut self, _state: &RoundState) -> DecideDrawResult {
        DecideDrawResult::Deck
    }
    fn find_melds(&mut self, hand: &Vec<Card>) -> Vec<Meld> {
        let mut melds = Vec::new();
        let groups = group_by_rank(hand);

        for cards in groups.values() {
            if cards.len() >= 3 {
                let mut meld: Vec<Card> = Vec::new();
                for card in cards {
                    if meld.is_empty() || meld.iter().all(|c| c.suit != card.suit) {
                        meld.push(card.clone());
                        if meld.len() == 4 {
                            melds.push(Meld { cards: meld.clone(), meld_type: crate::logic::MeldType::Rank });
                            meld = Vec::new();
                        }
                    }
                }
                if meld.len() >= 3 {
                     melds.push(Meld { cards: meld, meld_type: crate::logic::MeldType::Rank });
                }
            }
        }
        melds
    }

    fn decide_melds(&mut self, state: &RoundState) -> Vec<Meld> {
        let player = &state.players[state.current_player];
        let melds = self.find_melds(&player.hand);

        if crate::logic::melds_value(&melds) < 51 && !player.melded {
            return vec![];
        }

        let melds_cards_count: usize = melds.iter().map(|m| m.cards.len()).sum();
        let player_cards_num_after_meld = player.hand.len() as i32 - melds_cards_count as i32;

        if player_cards_num_after_meld == 6 || player_cards_num_after_meld == 3 || player_cards_num_after_meld == 2 {
            return vec![];
        }
        melds
    }

    fn decide_discard(&mut self, state: &RoundState) -> usize {
        let hand = &state.players[state.current_player].hand;
        let melds = self.find_melds(hand);

        let discard = hand.iter().find(|card| {
             !melds.iter().any(|meld| meld.cards.iter().any(|c| c.id == card.id))
        });

        if let Some(c) = discard {
            hand.iter().position(|x| x.id == c.id).unwrap()
        } else {
             hand.len() - 1 // Fallback
        }
    }
}
