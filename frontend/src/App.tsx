import MenuPage from "./pages/menu";
import { useGameStore } from "./lib/store/gameStore";
import WebSocketProvider from "./lib/provider/WebSocketProvider";
import GamePage from "./pages/game";
// import ResultsPage from "./pages/ResultsPage";

export default function App() {
  const page = useGameStore((s) => s.page);
  const gameId = useGameStore((s) => s.gameState?.id ?? null);
  const playerId = useGameStore((s) => s.player?.id ?? null);

  return (
    <>
      <WebSocketProvider gameId={gameId} playerId={playerId}>
        {page === "menu" && <MenuPage />}
        {page === "game" && gameId && <GamePage />}
        {/* {page === "results" && <ResultsPage onRestart={() => setPage("menu")} />} */}
      </WebSocketProvider>
    </>
  );
}
