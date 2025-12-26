import { createContext } from "react";
import type { Card, Meld } from "../logic";

export type CommandType =
  | {
      event: "startGame";
      gameId: string;
      playerId: string;
    }
  | {
      event: "roundStarted";
      gameId: string;
      playerId: string;
      roundNumber: number;
    }
  | {
      event: "drawPhaseAck";
      gameId: string;
      playerId: string;
      senderId: string;
      roundNumber: number;
    }
  | {
      event: "syncMeldsAck";
      gameId: string;
      playerId: string;
      senderId: string;
      roundNumber: number;
    }
  | {
      event: "playerDiscardedAck";
      gameId: string;
      playerId: string;
      senderId: string;
      roundNumber: number;
    }
  | {
      event: "drawPhaseFinished";
      gameId: string;
      playerId: string;
      roundNumber: number;
      data: {
        drawChoice: "deck" | "fire";
      };
    }
  | {
      event: "playingPhaseStarted";
      gameId: string;
      playerId: string;
      roundNumber: number;
      data: {
        phase: "playMeldPhase";
        melds: Meld[];
      };
    }
  | {
      event: "playingPhaseStarted";
      gameId: string;
      playerId: string;
      roundNumber: number;
      data: {
        phase: "playInMeldPhase";
        card: Card;
        meldId: string;
        isLeft: boolean;
      };
    }
  | {
      event: "playingPhaseStarted";
      gameId: string;
      playerId: string;
      roundNumber: number;
      data: {
        phase: "discardPhase";
        card: Card;
      };
    };

export const convertToDTOCard = (dto: Card): any => {
  return {
    id: dto.id,
    suit: dto.suit,
    rank:
      typeof dto.rank === "number"
        ? {
            Number: dto.rank,
          }
        : dto.rank == "A"
        ? "Ace"
        : dto.rank == "K"
        ? "King"
        : dto.rank == "Q"
        ? "Queen"
        : dto.rank == "J"
        ? "Jack"
        : dto.rank,
  };
};

export type WSContextType = {
  send: (command: CommandType) => void;
};

export const WebSocketContext = createContext<WSContextType | null>(null);
