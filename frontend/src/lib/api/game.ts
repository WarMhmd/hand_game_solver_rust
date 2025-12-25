import type { GameState } from "../logic";

const API_ENDPOINT = "http://localhost:3000/api/game";

export async function callStartGame(): Promise<GameState> {
  const res = await fetch(`${API_ENDPOINT}/init_game`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });

  if (!res.ok) throw new Error("Play rejected");
  return res.json();
}
