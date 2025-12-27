import type { GameState } from "../logic";

const API_ENDPOINT = (import.meta.env.VITE_GAME_API_ENDPOINT as string | undefined) ?? "http://localhost:3000/api/game";

export async function callStartGame(endPoint: string): Promise<GameState> {
  const res = await fetch(`${API_ENDPOINT}/${endPoint}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });

  if (!res.ok) throw new Error("Play rejected");
  return res.json();
}
