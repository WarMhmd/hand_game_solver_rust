// useGameEvents.ts
import { useEffect } from "react";
import type { Card } from "../logic";

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
    }
  | {
      event: "drawPhase";
      player_id: string;
    }
  | {
      event: "playingPhaseStarted";
      player_id: string;
    }
  | {
      event: "error";
      message: string;
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
