import { callStartGame } from "../lib/api/game";
import { useGameStore } from "../lib/store/gameStore";

export default function MenuPage() {
  const startGame = useGameStore((s) => s.startGame);

  const handleGameStart = async () => {
    // send fetch event to server
    try {
      const result = await callStartGame();
      startGame(result);
    } catch (error) {
      console.error("Failed to start game:", error);
    }
  };

  return (
    <div dir="rtl" className="min-h-screen bg-linear-to-b from-slate-900 to-black text-gray-100">
      <div className="mx-auto flex min-h-screen w-full max-w-5xl flex-col items-center justify-center px-6 py-16">
        <div className="w-full rounded-2xl border border-white/10 bg-white/5 p-8 backdrop-blur sm:p-10">
          <div className="flex flex-col gap-8">
            <header className="space-y-3">
              <h1 className="text-4xl font-bold tracking-tight text-gray-100 sm:text-5xl">لعبة الهند</h1>
            </header>

            <div className="grid gap-6 sm:grid-cols-2">
              <section className="rounded-xl border border-white/10 bg-black/20 p-5">
                <h2 className="text-sm font-semibold text-gray-200">نصائح سريعة</h2>
                <ul className="mt-3 space-y-2 text-sm text-gray-300">
                  <li>اسحب البطاقات إلى المحرقة.</li>
                </ul>
              </section>

              <section className="rounded-xl border border-white/10 bg-black/20 p-5">
                <h2 className="text-sm font-semibold text-gray-200">جاهز؟</h2>

                <div className="mt-5 flex flex-col gap-3">
                  <button
                    type="button"
                    onClick={handleGameStart}
                    className="inline-flex w-full items-center justify-center rounded-lg bg-green-600 px-6 py-3 text-lg font-semibold text-white transition hover:bg-green-700 focus:outline-none focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
                  >
                    ابدأ اللعبة
                  </button>
                </div>
              </section>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
