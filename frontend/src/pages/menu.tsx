import { callStartGame } from "../lib/api/game";
import { useGameStore } from "../lib/store/gameStore";
import { ArrowLeft, Bot, Shuffle, Sparkles, Spade } from "lucide-react";

export default function MenuPage() {
  const startGame = useGameStore((s) => s.startGame);
  const setEndPoint = useGameStore((s) => s.setEndPoint);

  const handleGameStart = async (endPoint: string) => {
    // send fetch event to server
    try {
      const result = await callStartGame(endPoint);
      setEndPoint(endPoint);
      startGame(result);
    } catch (error) {
      console.error("Failed to start game:", error);
    }
  };

  const modeCardBase =
    "group relative w-full rounded-xl border border-white/10 bg-white/5 p-4 text-right transition hover:bg-white/10 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400/40 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900";
  const modeCardHeader = "flex items-start justify-between gap-3";
  const modeCardTitle = "text-base font-semibold text-gray-100";
  const modeCardDesc = "mt-1 text-sm text-gray-300";
  const modeCardCta =
    "mt-4 inline-flex w-full items-center justify-center gap-2 rounded-lg bg-black/20 px-4 py-2 text-sm font-semibold text-gray-100 ring-1 ring-white/10 transition group-hover:bg-black/30";
  const modeCardCtaPrimary =
    "mt-4 inline-flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-600/90 px-4 py-2 text-sm font-semibold text-white transition group-hover:bg-emerald-600";

  return (
    <div dir="rtl" className="relative min-h-screen overflow-hidden bg-linear-to-b from-slate-900 to-black text-gray-100">
      {/* Subtle background glows */}
      <div className="pointer-events-none absolute -top-24 -left-24 h-80 w-80 rounded-full bg-emerald-500/10 blur-3xl" />
      <div className="pointer-events-none absolute -bottom-28 -right-28 h-96 w-96 rounded-full bg-sky-500/10 blur-3xl" />

      <div className="mx-auto flex min-h-screen w-full max-w-5xl flex-col items-center justify-center px-4 py-10 sm:px-6 sm:py-16">
        <div className="w-full rounded-2xl border border-white/10 bg-white/5 p-6 backdrop-blur-xl shadow-2xl shadow-black/35 transition-transform duration-200 hover:-translate-y-0.5 sm:p-10">
          <div className="flex flex-col gap-6 sm:gap-8">
            <header className="space-y-3">
              <div className="flex items-start justify-between gap-4">
                <div className="flex flex-wrap items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5 sm:h-11 sm:w-11">
                    <Spade className="h-6 w-6 text-gray-100 sm:h-7 sm:w-7" aria-hidden="true" />
                  </div>
                  <div className="space-y-1">
                    <h1 className="text-3xl font-bold tracking-tight text-gray-100 sm:text-5xl">لعبة الهند</h1>
                    <div className="text-sm text-gray-300">
                      صُنع بواسطة <span className="text-gray-200">محمد علاء الوراوره</span>
                    </div>
                  </div>
                </div>
              </div>
            </header>

            {/* Tips on top */}
            <section className="rounded-xl border border-white/10 bg-black/20 p-4 sm:p-5">
              <h2 className="text-sm font-semibold text-gray-200">نصائح سريعة</h2>
              <ul className="mt-3 space-y-2 text-sm text-gray-300 list-disc pr-5 marker:text-emerald-300/70">
                <li>اسحب البطاقة إلى المحرقة للتخلص منها</li>
                <li>حاليا اذا سحبت من المحرقة يجب عليك اللعب بها 😅</li>
                <li>ترتيب اليد هو ما يحدد البطاقات المستخدمة لتنزيل المجموع</li>
              </ul>
            </section>

            {/* Modes below as a set of three */}
            <section className="space-y-3">
              <div className="flex flex-col gap-1 sm:flex-row sm:items-baseline sm:justify-between sm:gap-4">
                <h2 className="text-sm font-semibold text-gray-200">جاهز؟</h2>
                <div className="text-sm text-gray-300">اختر نمط اللعب المناسب وابدأ مباشرة.</div>
              </div>

              <div className="grid gap-3 sm:grid-cols-3">
                <button type="button" onClick={() => handleGameStart("init_game")} className={modeCardBase + " ring-1 ring-emerald-400/15"}>
                  <div className={modeCardHeader}>
                    <div className="flex items-start gap-3">
                      <span className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-black/20">
                        <Bot className="h-5 w-5 text-emerald-200" aria-hidden="true" />
                      </span>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className={modeCardTitle}>العب ضد أحدث بوت</span>
                          <span className="rounded-full border border-emerald-400/20 bg-emerald-500/10 px-2 py-0.5 text-[11px] font-semibold text-emerald-200">
                            مُوصى به
                          </span>
                        </div>
                        <div className={modeCardDesc}>أفضل خيار للتحدّي والأداء.</div>
                      </div>
                    </div>
                  </div>
                  <div className={modeCardCtaPrimary}>
                    ابدأ الآن
                    <ArrowLeft className="h-4 w-4" aria-hidden="true" />
                  </div>
                </button>

                <button type="button" onClick={() => handleGameStart("init_game_with_random_strong_bots")} className={modeCardBase}>
                  <div className={modeCardHeader}>
                    <div className="flex items-start gap-3">
                      <span className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-black/20">
                        <Sparkles className="h-5 w-5 text-sky-200" aria-hidden="true" />
                      </span>
                      <div>
                        <div className={modeCardTitle}>بوتات عشوائية قوية</div>
                        <div className={modeCardDesc}>تشكيلة عشوائية بمستوى أعلى.</div>
                      </div>
                    </div>
                  </div>
                  <div className={modeCardCta}>
                    ابدأ
                    <ArrowLeft className="h-4 w-4" aria-hidden="true" />
                  </div>
                </button>

                <button type="button" onClick={() => handleGameStart("init_game_with_random_all_bots")} className={modeCardBase}>
                  <div className={modeCardHeader}>
                    <div className="flex items-start gap-3">
                      <span className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-black/20">
                        <Shuffle className="h-5 w-5 text-gray-200" aria-hidden="true" />
                      </span>
                      <div>
                        <div className={modeCardTitle}>بوتات عشوائية</div>
                        <div className={modeCardDesc}>خيار خفيف لتجربة سريعة.</div>
                      </div>
                    </div>
                  </div>
                  <div className={modeCardCta}>
                    ابدأ
                    <ArrowLeft className="h-4 w-4" aria-hidden="true" />
                  </div>
                </button>
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>
  );
}
