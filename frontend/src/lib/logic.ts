// --------------------
// Types
// --------------------

export type Suit = "Hearts" | "Diamonds" | "Clubs" | "Spades" | "Joker";
export type Rank = 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | "J" | "Q" | "K" | "A" | "Joker";

export const rankOrder: Record<Rank, number[]> = {
  2: [1],
  3: [2],
  4: [3],
  5: [4],
  6: [5],
  7: [6],
  8: [7],
  9: [8],
  10: [9],
  J: [10],
  Q: [11],
  K: [12],
  A: [0, 13],
  Joker: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
};

export interface Card {
  id: string;
  suit: Suit;
  rank: Rank;
}

export type MeldType = "rank" | "sequence";

export interface Meld {
  cards: Card[];
  type: "rank" | "sequence";
}

export interface Player {
  id: string;
  name: string;
  score: number;
}

export interface ActivePlayer extends Player {
  hand: Card[];
  FireCardId: string | null;
  melded: boolean;
}

export interface GameState {
  id: string;
  players: Player[];
  max_round: number;
}

export interface RoundState {
  players: ActivePlayer[];
  currentPlayer: number;
  deck: Card[];
  firePile: Card[];
  tableMelds: Meld[];
  phase: "draw" | "meld" | "playInMeld" | "discard";
}

// --------------------
// Deck creation (2 decks + 2 jokers = 106 cards)
// --------------------

export function createDeck(): Card[] {
  const suits: Suit[] = ["Hearts", "Diamonds", "Clubs", "Spades"];
  const ranks: Rank[] = [2, 3, 4, 5, 6, 7, 8, 9, 10, "J", "Q", "K", "A"];

  const deck: Card[] = [];

  for (let d = 0; d < 2; d++) {
    for (const suit of suits) {
      for (const rank of ranks) {
        deck.push({
          id: `${suit}-${rank}-${d}`,
          suit,
          rank,
        });
      }
    }
    deck.push({
      id: `Joker-${d}`,
      suit: "Joker",
      rank: "Joker",
    });
  }

  return deck;
}

export function shuffle(deck: Card[]): Card[] {
  const d = [...deck];
  for (let i = d.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [d[i], d[j]] = [d[j], d[i]];
  }
  return d;
}

// --------------------
// Game setup
// --------------------

// --------------------
// Helpers
// --------------------

export function rankValue(rank: Rank): number {
  if (rank === "A") return 11;
  if (rank === "Joker") return 0;
  if (typeof rank === "number") return rank;
  return 10;
}

export function cardValue(card: Card): number {
  return rankValue(card.rank);
}

export function meldValue(meld: Meld): number {
  let meldValue = meld.cards.reduce((sum, c) => sum + cardValue(c), 0);

  const jokerCard = meld.cards.find((card) => card.rank === "Joker");

  if (jokerCard) {
    // calculate joker value depending on what value he is in
    if (meld.type == "rank") {
      // get first non joker card
      const firstCard = meld.cards.find((card) => card.rank !== "Joker")!;
      meldValue += cardValue(firstCard);
    } else if (meld.type == "sequence") {
      // get the card before and after joker
      const cardIndex = meld.cards.indexOf(jokerCard);
      const prevCard = meld.cards.at(cardIndex - 1);
      const nextCard = meld.cards.at(cardIndex + 1);
      if (prevCard) {
        meldValue +=
          prevCard.rank == "K"
            ? 11 // joker is A
            : cardValue(prevCard) == 10
            ? 10 // joker is either J, Q, K
            : cardValue(prevCard) + 1; // joker is 3-10
      } else if (nextCard) {
        meldValue +=
          nextCard.rank == 2
            ? 11 // joker is A
            : cardValue(nextCard) === 10 && nextCard.rank !== 10
            ? 10 // joker is either 10, J, Q
            : cardValue(nextCard) - 1; // joker is 2-9
      }
    }
  }

  return meldValue;
}

export function meldsValue(melds: Meld[]): number {
  return melds.reduce((sum, m) => sum + meldValue(m), 0);
}

export function validSequenceTwoCards(left: Card, right: Card) {
  if (left.rank === "Joker" || right.rank === "Joker") {
    return true;
  }
  if (left.rank === "A" && right.rank === 2) return true;
  if (left.rank === 10 && right.rank === "J") return true;
  if (left.rank === "J" && right.rank === "Q") return true;
  if (left.rank === "Q" && right.rank === "K") return true;
  if (left.rank === "K" && right.rank === "A") return true;
  if (typeof left.rank === "number" && typeof right.rank === "number") return left.rank + 1 === right.rank;
  return false;
}

export const checkMelds = (state: RoundState) => {
  const removeIndex: number[] = [];
  const addedMelds: Meld[] = [];
  state.tableMelds.forEach((meld, index) => {
    // if a meld is a same rank
    if (validRankMeld(meld.cards)) {
      // meld has 4 cards and no joker
      if (meld.cards.length === 4 && meld.cards.every((c) => c.rank !== "Joker")) {
        // throw meld to fire pile and remove it
        state.firePile.push(...meld.cards);
        removeIndex.push(index);
      }
    }

    // if a meld is a sequence
    if (validSequenceMeld(meld.cards)) {
      if (meld.cards.length >= 6) {
        // split every three cards into a new meld
        removeIndex.push(index);
        for (let i = 0; i < meld.cards.length; i += 3) {
          if (i + 3 <= meld.cards.length && i + 6 <= meld.cards.length) {
            addedMelds.push({
              cards: meld.cards.slice(i, i + 3),
              type: "sequence",
            });
          } else {
            addedMelds.push({
              cards: meld.cards.slice(i),
              type: "sequence",
            });
            break;
          }
        }
      }
    }
  });

  removeIndex.forEach((index) => state.tableMelds.splice(index, 1));
  addedMelds.forEach((meld) => state.tableMelds.push(meld));
};

// --------------------
// Meld validation
// --------------------

export function validRankMeld(meldCards: Card[]): boolean {
  if (meldCards.length < 3 || meldCards.length > 4) return false;
  if (meldCards.filter((c) => c.rank === "Joker").length > 1) return false;

  const cards = meldCards.filter((c) => c.rank !== "Joker");

  const sameRank = cards.every((c) => c.rank === cards[0].rank);
  // check if they have different suits
  const setSuit = new Set(cards.map((c) => c.suit));
  const differentSuit = cards.length == setSuit.size;
  return sameRank && differentSuit;
}

export function validSequenceMeld(cards: Card[]): boolean {
  if (cards.length < 3) return false;
  if (cards.filter((c) => c.rank === "Joker").length > 1) return false;

  const firstCard = cards.find((c) => c.rank !== "Joker")!;
  const sameSuit = cards.every((c) => c.rank === "Joker" || c.suit === firstCard.suit);
  if (!sameSuit) return false;
  for (let i = 1; i < cards.length; i++) {
    if (!validSequenceTwoCards(cards[i - 1], cards[i])) return false;
    if (i == 1 && cards[i - 1].rank === "Joker" && cards[i].rank === "A") {
      return false;
    }
    if (i == cards.length - 1 && cards[i].rank === "Joker" && cards[i - 1].rank === "A") {
      return false;
    }
    if (cards[i].rank === "Joker") {
      if (i === cards.length - 1) continue;
      const nextCard = cards[i + 1];
      if (cardValue(nextCard) !== cardValue(cards[i - 1]) + 2) return false;
    }
  }
  return true;
}

// first boolean for success
// second boolean for can take joker card or no
export function canPlayInRankMeld(meld: Meld, card: Card): [boolean, boolean] {
  // check if there is a joker in meld
  const jokerCard = meld.cards.find((c) => c.rank === "Joker");

  // No more than one joker
  if (jokerCard && card.rank === "Joker") return [false, false];

  let copyMeld: Card[] = JSON.parse(JSON.stringify(meld.cards));
  let takeJoker = false;
  if (jokerCard && copyMeld.length == 4) {
    // replace joker with card
    copyMeld = copyMeld.map((c) => (c.rank === "Joker" ? card : c));

    if (!validRankMeld(copyMeld)) return [false, false];
    else takeJoker = true;
  } else {
    copyMeld.push(card);
    if (!validRankMeld(copyMeld)) return [false, false];
  }

  return [true, takeJoker];
}

// first boolean for success
// second boolean for take joker
export function canPlayInSequenceMeld(meld: Meld, card: Card, playEnd: boolean = false): [boolean, boolean] {
  // check if there is a joker in meld
  const jokerCard = meld.cards.find((c) => c.rank === "Joker");

  // No more than one joker
  if (jokerCard && card.rank === "Joker") return [false, false];

  if (jokerCard) {
    let copyMeld: Card[] = JSON.parse(JSON.stringify(meld.cards));
    // check if we can replace joker with card
    copyMeld = copyMeld.map((c) => (c.rank === "Joker" ? card : c));

    if (validSequenceMeld(copyMeld)) return [true, true];
  }

  if (card.rank != "Joker") {
    // try push card to first element
    let copyMeld: Card[] = JSON.parse(JSON.stringify(meld.cards));
    copyMeld.unshift(card);
    if (validSequenceMeld(copyMeld)) return [true, false];

    // try to push the card to last element
    copyMeld = JSON.parse(JSON.stringify(meld.cards));
    copyMeld.push(card);
    if (validSequenceMeld(copyMeld)) return [true, false];
  } else {
    if (!playEnd) {
      // add joker to the beginning of the meld
      const copyMeld: Card[] = JSON.parse(JSON.stringify(meld.cards));
      copyMeld.unshift(card);
      if (validSequenceMeld(copyMeld)) return [true, false];
    } else {
      // add joker to the end of the meld
      const copyMeld: Card[] = JSON.parse(JSON.stringify(meld.cards));
      copyMeld.push(card);
      if (validSequenceMeld(copyMeld)) return [true, false];
    }
  }

  return [false, false];
}

export function isValidSet(meld: Meld): boolean {
  // check if meld is a same rank
  if (meld.type === "rank" && validRankMeld(meld.cards)) return true;

  // check if meld is a sequence
  if (meld.type === "sequence" && validSequenceMeld(meld.cards)) return true;

  return false;
}

// --------------------
// Turn actions
// --------------------

export function drawFromDeck(state: RoundState): RoundState {
  if (state.phase !== "draw") throw new Error("Not draw phase");
  let card = state.deck.shift();
  if (!card) {
    // put Fire
    state.deck = shuffle(state.firePile);
    state.firePile = [];
    // draw from deck again
    card = state.deck.shift();
  }
  if (!card) throw new Error("Deck empty");

  state.players[state.currentPlayer].hand.push(card);
  return state;
}

export function drawFromFire(state: RoundState): RoundState {
  if (state.phase !== "draw") throw new Error("Not draw phase");
  const card = state.firePile.pop();
  if (!card) throw new Error("Fire pile empty");

  state.players[state.currentPlayer].hand.push(card);
  state.players[state.currentPlayer].FireCardId = card.id;
  return state;
}

export function discardFireCard(state: RoundState) {
  const player = state.players[state.currentPlayer];
  if (!player.FireCardId) throw new Error("No fire card to discard");

  const card = player.hand.find((c) => c.id === player.FireCardId);
  if (!card) throw new Error("Fire card not in hand");

  // add card to fire pile
  state.firePile.push(card);

  player.hand = player.hand.filter((c) => c !== card);
  player.FireCardId = null;
}

export function layMelds(state: RoundState, melds: Meld[]): RoundState {
  if (melds.length === 0) throw new Error("No melds to lay");
  if (!melds.every((m) => isValidSet(m))) throw new Error("Invalid meld");

  const player = state.players[state.currentPlayer];

  // check for total meld score if not melded
  if (!player.melded) {
    const score = meldsValue(melds);
    if (score < 51) throw new Error("Total meld score must be >= 51");
  }

  if (player.FireCardId != null) {
    // check if player used fire card in melds
    if (!melds.some((m) => m.cards.some((c) => c.id === player.FireCardId))) {
      throw new Error("Fire card not used in meld");
    }
  }

  // Remove cards from hand
  for (const m of melds) {
    m.cards.forEach((c) => {
      const idx = player.hand.findIndex((h) => h === c);
      if (idx === -1) throw new Error("Card not in hand");
      player.hand.splice(idx, 1);
    });
  }

  player.melded = true;

  state.tableMelds.push(...melds);
  checkMelds(state);
  return state;
}

export function PlayInMeld(state: RoundState, card: Card, meldIndex: number) {
  const player = state.players[state.currentPlayer];
  const meld = state.tableMelds[meldIndex];

  if (!meld) throw new Error("Meld not found");
  if (!player.hand.some((c) => c.id === card.id)) throw new Error("Card not in hand");

  if (player.hand.length == 1) throw new Error("Cannot play last card in hand");

  if (meld.type == "rank") {
    const [success, takeJoker] = canPlayInRankMeld(meld, card);
    if (!success) return;
    if (takeJoker) {
      // replace joker with card
      const jokerCard = meld.cards.find((c) => c.rank === "Joker");
      meld.cards = meld.cards.map((c) => (c.rank === "Joker" ? card : c));
      // add jokerCard to player hand
      player.hand.push(jokerCard!);
    } else {
      // add card to meld
      meld.cards.push(card);
    }
    // remove card from player hand
    player.hand.splice(
      player.hand.findIndex((c) => c.id === card.id),
      1
    );
  }

  if (meld.type == "sequence") {
    const [success, takeJoker] = canPlayInSequenceMeld(meld, card);
    if (!success) return;
    if (takeJoker) {
      // replace joker with card
      const jokerCard = meld.cards.find((c) => c.rank === "Joker");
      meld.cards = meld.cards.map((c) => (c.rank === "Joker" ? card : c));
      // add jokerCard to player hand
      player.hand.push(jokerCard!);
    } else {
      // add card to meld
      meld.cards.push(card);
    }
    // remove card from player hand
    player.hand.splice(
      player.hand.findIndex((c) => c.id === card.id),
      1
    );
  }

  checkMelds(state);
  return;
}

export function discardCard(state: RoundState, cardIndex: number): RoundState {
  if (state.phase !== "discard") throw new Error("Not discard phase");

  const player = state.players[state.currentPlayer];
  const card = player.hand.splice(cardIndex, 1)[0];
  state.firePile.push(card);

  state.currentPlayer = (state.currentPlayer + 1) % state.players.length;
  return state;
}
