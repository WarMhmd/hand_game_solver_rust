import { createContext } from "react";

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
    };

export type WSContextType = {
  send: (command: CommandType) => void;
};

export const WebSocketContext = createContext<WSContextType | null>(null);
