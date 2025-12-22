// src/models.rs
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Num(u8), // 2-10
    J,
    Q,
    K,
    A,
    Joker,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub id: String,
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeldType {
    Rank,
    Sequence,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meld {
    pub cards: Vec<Card>,
    pub meld_type: MeldType,
}

impl Rank {
    pub fn value(&self) -> u32 {
        match self {
            Rank::A => 11,
            Rank::Joker => 0, // Value inside a meld is calculated dynamically
            Rank::K | Rank::Q | Rank::J => 10,
            Rank::Num(n) => *n as u32,
        }
    }

    // Returns (primary_order, secondary_order) for sorting
    pub fn order(&self) -> (i32, i32) {
        match self {
            Rank::A => (0, 13), // A can be low or high
            Rank::Joker => (0, 14), // Special case handling needed usually
            Rank::K => (12, 12),
            Rank::Q => (11, 11),
            Rank::J => (10, 10),
            Rank::Num(n) => (*n as i32 - 1, *n as i32 - 1), // 2 maps to 1
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} of {:?}", self.rank, self.suit)
    }
}
