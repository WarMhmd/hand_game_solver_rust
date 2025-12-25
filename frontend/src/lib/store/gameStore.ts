import { create } from "zustand";
import type { GameState } from "../logic";

type Page = "menu" | "game" | "results";

type GameStore = {
  page: Page;
  gameState: GameState | null;

  // actions
  goTo: (page: Page) => void;
  startGame: (gameState: GameState) => void;
  updateGame: (gameState: GameState) => void;
  reset: () => void;
};

export const useGameStore = create<GameStore>((set) => ({
  page: "menu",
  gameState: null,

  goTo: (page) => set({ page }),

  startGame: (gameState) =>
    set({
      page: "game",
      gameState,
    }),

  updateGame: (gameState) => set({ gameState }),

  reset: () => set({ page: "menu", gameState: null }),
}));
