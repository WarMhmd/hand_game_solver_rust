import { create } from "zustand";
import type { RoundState } from "../logic";

type StoreType = {
  roundState: RoundState | null;

  // actions
  startRound: (roundState: RoundState) => void;
  setRoundPhase: (phase: "draw" | "playing") => void;
};

export const useRoundStore = create<StoreType>((set) => ({
  roundState: null,
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
}));
