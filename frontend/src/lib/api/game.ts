import type { GameState } from "../logic";

const API = "/api/game";

export async function callStartGame(): Promise<GameState> {
  const res = await fetch(`${API}/start_game`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });

  if (!res.ok) throw new Error("Play rejected");
  return res.json();
}
