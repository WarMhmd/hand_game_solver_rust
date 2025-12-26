// useGameEvents.ts
import { useEffect } from "react";
import type { Card, Meld } from "../logic";

type GameEvent =
  | {
      event: "joined";
      success: boolean;
      message?: string;
    }
  | {
      event: "roundStarted";
      roundNumber: number;
      hand: Card[];
      fireCardId: string;
      melded: boolean;
      scores: number[];
    }
  | {
      event: "drawPhase";
      player_id: string;
    }
  | {
      event: "drawnCard";
      playerId: string;
      card: Card;
      isFireCard: boolean;
      emptyFirePile: boolean;
    }
  | {
      event: "playingPhaseStarted";
      playerId: string;
    }
  | {
      event: "syncMelds";
      playerId: string;
      tableMelds: Meld[];
      playerMelds: Card[];
      playerHandSize: number;
      takeJoker?: Card;
    }
  | {
      event: "playerDiscarded";
      playerId: string;
      card: Card;
      playerHandSize: number;
    }
  | {
      event: "gameEnded" | "gameOver";
      players: Array<{
        id: string;
        name: string;
        score: number;
        wins: number;
      }>;
    }
  | {
      event: "error";
      message: string;
    };

export const convertCardDto = (dto: any): Card => {
  return {
    id: dto.id,
    suit: dto.suit,
    rank:
      typeof dto.rank === "object"
        ? dto.rank.Number
        : dto.rank == "Ace"
        ? "A"
        : dto.rank == "King"
        ? "K"
        : dto.rank == "Queen"
        ? "Q"
        : dto.rank == "Jack"
        ? "J"
        : dto.rank,
  };
};

export function useGameEvents(handler: (event: GameEvent) => void) {
  useEffect(() => {
    const listener = (e: Event) => {
      handler((e as CustomEvent<GameEvent>).detail);
    };

    window.addEventListener("game-event", listener);
    return () => window.removeEventListener("game-event", listener);
  }, [handler]);
}
