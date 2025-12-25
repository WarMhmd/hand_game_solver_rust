import { useContext } from "react";
import { WebSocketContext } from "./WebSocketContext";

export function useGameSocket() {
  const ctx = useContext(WebSocketContext);
  if (!ctx) {
    throw new Error("useGameSocket must be used inside WebSocketProvider");
  }
  return ctx;
}
