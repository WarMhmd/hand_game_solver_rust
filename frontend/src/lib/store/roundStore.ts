import { create } from "zustand";
import type { ActivePlayer, Card, Meld, RoundState } from "../logic";

type StoreType = {
  roundState: RoundState | null;
  player: ActivePlayer | null;
  // actions
  startRound: (roundState: RoundState) => void;
  setRoundPhase: (phase: "draw" | "playing") => void;
  setPlayer: (player: ActivePlayer) => void;
  setPlayerMelded: (melded: boolean) => void;
  addCardToHand: (card: Card, isFireCard: boolean) => void;
  setPlayerHand: (hand: Card[]) => void;
  drawOtherPlayerHand: (player_id: string) => void;
  setOtherPlayerHand: (player_id: string, handSize: number) => void;
  syncMelds: (table_melds: Meld[]) => void;
  removeMeldedCardsFromHand: (melds: Card[]) => void;
  addCardToFirePile: (card: Card) => void;
  addCardsToFirePile: (cards: Card[]) => void;
  removeCardFromFirePile: (cardId: string) => void;
  discardCardFromHand: (card: Card) => void;
  nextPlayer: () => void;
  setFirePile: (firePile: Card[]) => void;
};

export const useRoundStore = create<StoreType>((set) => ({
  roundState: null,
  player: null,
  setPlayer: (player) => set({ player }),
  setPlayerMelded: (melded) =>
    set((state) => {
      if (!state.player) return {};
      const updatedPlayer: ActivePlayer = {
        ...state.player,
        melded,
      };

      return {
        player: updatedPlayer,
        roundState: state.roundState
          ? {
              ...state.roundState,
              players: state.roundState.players.map((p) => (p.id === updatedPlayer.id ? updatedPlayer : p)),
            }
          : state.roundState,
      };
    }),
  startRound: (roundState) => set({ roundState }),
  setRoundPhase: (phase) =>
    set((state) => {
      if (state.roundState) {
        return {
          roundState: {
            ...state.roundState,
            phase,
          },
        };
      }
      return {};
    }),

  setPlayerHand: (hand) =>
    set((state) => {
      if (state.player) {
        return {
          player: {
            ...state.player,
            hand: hand,
          },
        };
      }
      return {};
    }),

  addCardToHand: (card, isFireCard) =>
    set((state) => {
      if (!state.player || !state.roundState) return {};

      const updatedPlayer = {
        ...state.player,
        FireCardId: isFireCard ? card.id : null,
        hand: [...state.player.hand, card],
      };

      return {
        player: updatedPlayer,
        roundState: {
          ...state.roundState,
          players: state.roundState.players.map((p) => (p.id === updatedPlayer.id ? updatedPlayer : p)),
          firePile: isFireCard ? state.roundState.firePile.filter((c) => c.id !== card.id) : state.roundState.firePile,
        },
      };
    }),

  drawOtherPlayerHand: (player_id) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }

      return {
        roundState: {
          ...state.roundState,
          players: state.roundState.players.map((p) =>
            p.id === player_id
              ? {
                  ...p,
                  // Create a NEW array with the old items + the new item
                  hand: [...p.hand, { id: "hidden", suit: "Hidden", rank: "Hidden" }],
                }
              : p
          ),
        },
      };
    }),

  setOtherPlayerHand: (player_id, handSize) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }
      return {
        roundState: {
          ...state.roundState,
          players: state.roundState.players.map((p) =>
            p.id === player_id
              ? {
                  ...p,
                  hand: Array(handSize).fill({ id: "hidden", suit: "Hidden", rank: "Hidden" }),
                }
              : p
          ),
        },
      };
    }),

  syncMelds: (table_melds) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }
      return {
        roundState: {
          ...state.roundState,
          tableMelds: table_melds,
        },
      };
    }),

  removeMeldedCardsFromHand: (melds) =>
    set((state) => {
      if (!state.player || !state.roundState) {
        return {};
      }
      const meldedCardIds = melds.flat().map((card) => card.id);
      const updatedHand = state.player.hand.filter((card) => !meldedCardIds.includes(card.id));
      const activePlayer: ActivePlayer = {
        ...state.player,
        hand: updatedHand,
        FireCardId: state.player.FireCardId && meldedCardIds.includes(state.player.FireCardId) ? null : state.player.FireCardId,
      };
      return {
        player: activePlayer,
        roundState: {
          ...state.roundState,
          players: state.roundState.players.map((p) => (p.id === state.player!.id ? activePlayer : p)),
        },
      };
    }),

  addCardToFirePile: (card) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }
      return {
        roundState: {
          ...state.roundState,
          firePile: [...state.roundState.firePile, card],
        },
      };
    }),

  addCardsToFirePile: (cards) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }
      if (cards.length === 0) return {};
      return {
        roundState: {
          ...state.roundState,
          firePile: [...state.roundState.firePile, ...cards],
        },
      };
    }),

  removeCardFromFirePile: (cardId) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }
      return {
        roundState: {
          ...state.roundState,
          firePile: state.roundState.firePile.filter((c) => c.id !== cardId),
        },
      };
    }),

  discardCardFromHand: (card) =>
    set((state) => {
      if (!state.player || !state.roundState) {
        return {};
      }
      const updatedHand = state.player.hand.filter((c) => c.id !== card.id);
      return {
        player: {
          ...state.player,
          hand: updatedHand,
        },
        roundState: {
          ...state.roundState,
          players: state.roundState.players.map((p) =>
            p.id === state.player!.id
              ? {
                  ...p,
                  hand: updatedHand,
                }
              : p
          ),
        },
      };
    }),

  nextPlayer: () =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }

      const nextPlayer = state.roundState.currentPlayer + 1 >= state.roundState.players.length ? 0 : state.roundState.currentPlayer + 1;

      return {
        roundState: {
          ...state.roundState,
          currentPlayer: nextPlayer,
        },
      };
    }),

  setFirePile: (firePile) =>
    set((state) => {
      if (!state.roundState) {
        return {};
      }
      return {
        roundState: {
          ...state.roundState,
          firePile: firePile,
        },
      };
    }),
}));
