import { callStartGame } from "../lib/api/game";
import { useGameStore } from "../lib/store/gameStore";
import { Spade } from "lucide-react";

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
    <div dir="rtl" className="relative min-h-screen overflow-hidden bg-linear-to-b from-slate-900 to-black text-gray-100">
      {/* Subtle background glows */}
      <div className="pointer-events-none absolute -top-24 -left-24 h-80 w-80 rounded-full bg-emerald-500/10 blur-3xl" />
      <div className="pointer-events-none absolute -bottom-28 -right-28 h-96 w-96 rounded-full bg-sky-500/10 blur-3xl" />

      <div className="mx-auto flex min-h-screen w-full max-w-5xl flex-col items-center justify-center px-4 py-10 sm:px-6 sm:py-16">
        <div className="w-full rounded-2xl border border-white/10 bg-white/5 p-6 backdrop-blur-xl shadow-xl shadow-black/30 transition-transform duration-200 hover:-translate-y-0.5 sm:p-10">
          <div className="flex flex-col gap-8">
            <header className="space-y-3">
              <div className="flex items-center justify-between gap-4">
                <div className="flex flex-wrap items-center gap-3">
                  <Spade className="h-8 w-8 text-gray-100 sm:h-9 sm:w-9" aria-hidden="true" />
                  <h1 className="text-3xl font-bold tracking-tight text-gray-100 sm:text-5xl">لعبة الهند</h1>
                </div>
              </div>
              <div className="text-sm text-gray-300">
                صُنع بواسطة <span className="text-gray-200">محمد علاء الوراوره</span>
              </div>
            </header>

            <div className="grid gap-6 sm:grid-cols-2">
              <section className="rounded-xl border border-white/10 bg-black/20 p-4 sm:p-5">
                <h2 className="text-sm font-semibold text-gray-200">نصائح سريعة</h2>
                <ul className="mt-3 space-y-2 text-sm text-gray-300">
                  <li>اسحب البطاقة إلى المحرقة للتخلص منها</li>
                  <li>حاليا اذا صحبت من المحرقة يجب عليك اللعب بها 😅</li>
                  <li>ترتيب اليد هو ما يحدد البطاقات المستحدمة لتنزيل المجموع</li>
                </ul>
              </section>

              <section className="rounded-xl border border-white/10 bg-black/20 p-4 sm:p-5">
                <h2 className="text-sm font-semibold text-gray-200">جاهز؟</h2>

                <div className="mt-5 flex flex-col gap-3">
                  <button
                    type="button"
                    onClick={handleGameStart}
                    className="inline-flex w-full items-center justify-center rounded-lg bg-green-600 px-6 py-3 text-lg font-semibold text-white transition-transform duration-150 hover:bg-green-700 hover:-translate-y-0.5 active:translate-y-0 focus:outline-none focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
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
