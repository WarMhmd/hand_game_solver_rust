import MenuPage from "./pages/menu";
import { useGameStore } from "./lib/store/gameStore";
// import GamePage from "./pages/game";
// import ResultsPage from "./pages/ResultsPage";

export default function App() {
  const page = useGameStore((s) => s.page);

  return (
    <>
      {page === "menu" && <MenuPage />}
      {/* {page === "game" && <GamePage />} */}
      {/* {page === "results" && <ResultsPage onRestart={() => setPage("menu")} />} */}
    </>
  );
}
