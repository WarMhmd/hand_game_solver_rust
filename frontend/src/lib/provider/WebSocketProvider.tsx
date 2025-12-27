import { useEffect, useRef } from "react";
import { WebSocketContext } from "./WebSocketContext";

const WS_ENDPOINT = (import.meta.env.VITE_WS_ENDPOINT as string | undefined) ?? "ws://localhost:3000/ws";

export default function WebSocketProvider({
  gameId,
  playerId,
  children,
}: {
  gameId: string | null;
  playerId: string | null;
  children: React.ReactNode;
}) {
  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!gameId) return;
    if (!playerId) return;
    if (socketRef.current) return;
    const ws = new WebSocket(WS_ENDPOINT);
    socketRef.current = ws;

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      window.dispatchEvent(new CustomEvent("game-event", { detail: data }));
    };

    ws.onopen = () => {
      console.log("🟢 WS open");

      ws.send(JSON.stringify({ event: "join", gameId, playerId }));
    };

    ws.onclose = () => console.log("🔴 WS close");

    return () => {
      ws.close();
      socketRef.current = null;
    };
  }, [gameId, playerId]);

  const send = (command: unknown) => {
    const ws = socketRef.current;
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(command));
    }
  };

  return <WebSocketContext.Provider value={{ send }}>{children}</WebSocketContext.Provider>;
}
