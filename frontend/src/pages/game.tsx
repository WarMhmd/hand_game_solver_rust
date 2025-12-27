import { AnimatePresence, motion, LayoutGroup } from "framer-motion";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  closestCenter,
  pointerWithin,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
  type CollisionDetection,
} from "@dnd-kit/core";
import { SortableContext, arrayMove, rectSortingStrategy, sortableKeyboardCoordinates, useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import {
  canPlayCardInMeld,
  handMelds,
  meldsValue,
  rankOrder,
  validRankMeld,
  validSequenceMeld,
  type ActivePlayer,
  type Card,
  type Meld,
  type Rank,
  type RoundState,
  type Suit,
} from "../lib/logic";
import { convertCardDto, useGameEvents } from "../lib/provider/event";
import { useGameSocket } from "../lib/provider/useGameSocket";
import { useGameStore } from "../lib/store/gameStore";
import { useRoundStore } from "../lib/store/roundStore";
import { useShallow } from "zustand/shallow";
import { convertToDTOCard } from "../lib/provider/WebSocketContext";
import { callStartGame } from "../lib/api/game";

function display_suit(suit: Suit) {
  switch (suit) {
    case "Hearts":
      return "♥";
    case "Diamonds":
      return "♦";
    case "Clubs":
      return "♣";
    case "Spades":
      return "♠";
    case "Joker":
      return "";
    default:
      return suit;
  }
}

function isJokerCard(card: Card) {
  return card.rank === "Joker" || card.suit === "Joker";
}

function suitSortKey(suit: Suit): number {
  // TODO: tweak suit ordering if your rules prefer a different order.
  switch (suit) {
    case "Clubs":
      return 0;
    case "Diamonds":
      return 1;
    case "Hearts":
      return 2;
    case "Spades":
      return 3;
    case "Joker":
      return 4;
  }
  return 5;
}

function rankSortKey(rank: Rank): number {
  const order = rankOrder[rank];
  return order.length > 0 ? order[0] : 999;
}

const EMPTY_HAND: Card[] = [];

function sortHandForRoundStart(cards: Card[]): Card[] {
  // Sort by suit then rank; always keep jokers at the end.
  return [...cards].sort((a, b) => {
    const aj = isJokerCard(a);
    const bj = isJokerCard(b);
    if (aj !== bj) return aj ? 1 : -1;
    const suitDiff = suitSortKey(a.suit) - suitSortKey(b.suit);
    if (suitDiff !== 0) return suitDiff;
    return rankSortKey(a.rank) - rankSortKey(b.rank);
  });
}

function meldsFromHandOrderForValue(hand: Card[]): Meld[] {
  const melds: Meld[] = [];

  for (let i = 0; i < hand.length; i++) {
    if (i + 3 > hand.length) break;
    const meldCards = hand.slice(i, i + 3);

    if (validRankMeld(meldCards)) {
      // Keep behavior consistent with `handMelds` (visual highlight) for now.
      if (i + 4 <= hand.length) {
        const fourthCard = hand[i + 3];
        if (validRankMeld([...meldCards, fourthCard])) {
          meldCards.push(fourthCard);
        }
      }

      melds.push({ id: crypto.randomUUID(), cards: [...meldCards], meldType: "Rank" });
      i = i + meldCards.length - 1;
      continue;
    }

    if (validSequenceMeld(meldCards)) {
      let j = i + 3;
      while (j < hand.length) {
        const nextCard = hand[j];
        if (validSequenceMeld([...meldCards, nextCard])) {
          meldCards.push(nextCard);
          j++;
        } else {
          break;
        }
      }

      melds.push({
        id: crypto.randomUUID(),
        cards: [...meldCards],
        meldType: "Sequence",
      });
      i = j - 1;
      continue;
    }

    meldCards.reverse();
    if (validSequenceMeld(meldCards)) {
      let j = i + 3;
      while (j < hand.length) {
        const nextCard = hand[j];
        if (validSequenceMeld([nextCard, ...meldCards])) {
          meldCards.unshift(nextCard);
          j++;
        } else {
          break;
        }
      }
      melds.push({
        id: crypto.randomUUID(),
        cards: [...meldCards],
        meldType: "Sequence",
      });
      i = j - 1;
      continue;
    }
  }

  return melds;
}

// ---- Card Component ----
function PlayingCard({
  card,
  draggable = true,
  highlight,
  size = "md",
  className,
}: {
  card: Card;
  draggable?: boolean;
  highlight?: boolean;
  size?: "md" | "sm";
  className?: string;
}) {
  const isRed = card.suit === "Hearts" || card.suit === "Diamonds";
  const sizeClasses = size === "sm" ? "w-10 h-16 sm:w-12 sm:h-20 rounded-lg p-1.5" : "w-12 h-20 sm:w-16 sm:h-24 rounded-xl p-2";
  const rankText = size === "sm" ? "text-[10px] sm:text-[11px]" : "text-xs sm:text-sm";
  const suitText = size === "sm" ? "text-base sm:text-lg" : "text-lg sm:text-xl";

  return (
    <motion.div
      whileHover={draggable ? { y: -12 } : undefined}
      // NOTE: Avoid whileTap on sortable cards; during dnd-kit drags Framer may miss pointer-up
      // and leave the card stuck scaled down.
      whileTap={undefined}
      transition={{ type: "spring", stiffness: 520, damping: 34 }}
      className={`${sizeClasses} shadow-lg border bg-white flex flex-col justify-between select-none ${
        draggable ? "cursor-grab" : "cursor-default"
      } ${isRed ? "text-red-600" : "text-black"} ${highlight ? "ring-2 ring-yellow-300 shadow-lg shadow-yellow-400/35" : ""} ${
        className ?? ""
      }`}
    >
      <span className={`${rankText} font-bold self-start`}>{card.rank.toString()}</span>
      <span className={`${suitText} text-center`}>{display_suit(card.suit)}</span>
      <span className={`${rankText} font-bold self-end`}>{card.rank.toString()}</span>
    </motion.div>
  );
}

function SortableHandCard({ card, disabled, zIndex, highlight }: { card: Card; disabled?: boolean; zIndex?: number; highlight?: boolean }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: card.id,
    disabled,
  });

  const style: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: isDragging ? 50 : zIndex,
    position: "relative",
  };

  return (
    <div ref={setNodeRef} style={style} className={isDragging ? "opacity-40" : undefined}>
      <div {...attributes} {...listeners} style={{ touchAction: "none" }}>
        <PlayingCard card={card} draggable={!disabled} highlight={highlight} />
      </div>
    </div>
  );
}

// ---- Table Meld Group (Droppable left/right halves) ----
function TableMeldGroup({
  meld,
  meldIndex,
  dropDisabled,
  draggedCard,
  registerEl,
}: {
  meld: Meld;
  meldIndex: number;
  dropDisabled: boolean;
  draggedCard: Card | null;
  registerEl?: (meldId: string, el: HTMLDivElement | null) => void;
}) {
  const leftId = `meld:${meldIndex}:left`;
  const rightId = `meld:${meldIndex}:right`;

  const { setNodeRef: setLeftRef, isOver: isOverLeft } = useDroppable({ id: leftId, disabled: dropDisabled });
  const { setNodeRef: setRightRef, isOver: isOverRight } = useDroppable({ id: rightId, disabled: dropDisabled });

  const isOverAny = isOverLeft || isOverRight;
  const isDraggingJokerToSequence = Boolean(draggedCard && isJokerCard(draggedCard) && meld.meldType === "Sequence");
  const showWholeGroupGlow = !dropDisabled && isOverAny && !isDraggingJokerToSequence;

  return (
    <div
      ref={(el) => registerEl?.(meld.id, el)}
      dir="ltr"
      className={`relative rounded-lg sm:rounded-xl bg-white/5 border px-2 py-1.5 sm:px-3 sm:py-2 ${
        showWholeGroupGlow ? "border-amber-300/70 shadow-lg shadow-amber-400/10" : "border-white/10"
      }`}
    >
      <div className="flex items-center justify-center overflow-visible">
        {meld.cards.map((card, idx) => {
          const isLeftMost = idx === 0;
          const isRightMost = idx === meld.cards.length - 1;
          const glowLeft = !dropDisabled && isDraggingJokerToSequence && isOverLeft && isLeftMost;
          const glowRight = !dropDisabled && isDraggingJokerToSequence && isOverRight && isRightMost;
          const shouldGlowCard = glowLeft || glowRight;

          return (
            <div
              key={`${meldIndex}:${card.id}`}
              className={idx === 0 ? "" : "-ml-5"}
              style={{
                position: "relative",
                zIndex: shouldGlowCard ? 60 : 10 + idx,
              }}
            >
              <PlayingCard
                card={card}
                draggable={false}
                size="sm"
                className={`w-9 h-14 p-1 sm:w-10 sm:h-16 sm:p-1.5 ${
                  shouldGlowCard ? "ring-2 ring-emerald-300/70 shadow-lg shadow-emerald-400/20" : ""
                }`}
              />
            </div>
          );
        })}
      </div>

      {/* Drop overlays (visual-only; used to decide isLeft for joker placement) */}
      <div ref={setLeftRef} className="absolute inset-y-0 left-0 w-1/2" />
      <div ref={setRightRef} className="absolute inset-y-0 right-0 w-1/2" />
    </div>
  );
}

function CardBack({ className }: { className?: string }) {
  return (
    <div
      className={
        "w-10 h-16 sm:w-12 sm:h-20 rounded-lg bg-gray-800 border border-gray-600 shadow-md " +
        "bg-linear-to-br from-gray-700/60 to-gray-900/60 " +
        (className ?? "")
      }
    />
  );
}

type StackDirection = "horizontal" | "vertical";

function StackedHand({
  count,
  direction,
  mirror,
}: {
  count: number;
  direction: StackDirection;
  // Mirrors the fan direction (useful for left/right players so the hand fans toward the table center)
  mirror?: boolean;
}) {
  const visibleCount = Math.min(count, 14);
  const overlap = direction === "horizontal" ? 10 : 8;
  const mirrorSign = mirror ? -1 : 1;

  const width = direction === "horizontal" ? 48 + overlap * (visibleCount - 1) : 56;
  const height = direction === "vertical" ? 80 + overlap * (visibleCount - 1) : 92;

  const containerStyle: CSSProperties = {
    width,
    height,
  };

  return (
    <div className="relative" style={containerStyle}>
      {Array.from({ length: visibleCount }).map((_, idx) => {
        const t = visibleCount <= 1 ? 0 : idx / (visibleCount - 1);
        const rotate = (t - 0.5) * 10 * mirrorSign; // small fan
        const translate = idx * overlap;

        const style: CSSProperties =
          direction === "horizontal"
            ? {
                position: "absolute",
                left: translate,
                top: Math.sin(t * Math.PI) * 3,
                transform: `rotate(${rotate}deg)`,
              }
            : {
                position: "absolute",
                top: translate,
                left: Math.sin(t * Math.PI) * 3 * mirrorSign,
                transform: `rotate(${rotate}deg)`,
              };

        return (
          <div key={idx} style={style} className="origin-center">
            <CardBack />
          </div>
        );
      })}
    </div>
  );
}

// ---- Player Area ----
function PlayerArea({
  player,
  isBottom,
  stackDirection = "horizontal",
  stackMirror,
  isCurrent,
  registerEl,
}: {
  player: ActivePlayer;
  isBottom?: boolean;
  stackDirection?: StackDirection;
  stackMirror?: boolean;
  isCurrent?: boolean;
  registerEl?: (playerId: string, el: HTMLDivElement | null) => void;
}) {
  const spacingClasses = isBottom
    ? "px-2 py-2 sm:px-3 sm:py-2"
    : stackDirection === "vertical"
    ? "px-2 py-3 sm:px-4 sm:py-8"
    : "px-3 py-2 sm:px-8 sm:py-4";

  return (
    <div
      ref={(el) => registerEl?.(player.id, el)}
      className={`relative flex flex-col items-center rounded-2xl w-fit self-center ${spacingClasses} ${isBottom ? "mt-6" : ""} ${
        isCurrent ? "ring-2 ring-emerald-400/60 ring-offset-2 ring-offset-slate-900" : ""
      }`}
    >
      {isCurrent ? (
        <span
          className={`pointer-events-none absolute left-1/2 -translate-x-1/2 z-20 text-[10px] px-2 py-0.5 rounded-full bg-emerald-400/15 text-emerald-200 border border-emerald-400/30 whitespace-nowrap ${
            isBottom ? "top-1" : "bottom-0"
          }`}
        >
          الدور الآن
        </span>
      ) : null}

      <div className="mb-2">
        <span className={`text-sm ${isCurrent ? "text-emerald-200 font-semibold" : "text-gray-300"}`}>{player.name}</span>
      </div>

      <div className="flex gap-2 flex-wrap justify-center max-w-sm sm:max-w-4xl">
        {isBottom ? (
          player.hand.map((card) => <PlayingCard key={card.id} card={card} />)
        ) : (
          <StackedHand count={player.hand.length > 0 ? player.hand.length : 15} direction={stackDirection} mirror={stackMirror} />
        )}
      </div>
    </div>
  );
}

// ---- Center Table Area (Fire Pile + Deck) ----
function CenterTableArea({
  onDeckClick,
  firePile,
  fireTop,
  onFireTopClick,
  deckDisabled,
  firePileDropDisabled,
  registerDeckEl,
  registerFirePileEl,
}: {
  onDeckClick: () => void;
  deckDisabled?: boolean;
  firePileDropDisabled?: boolean;
  firePile: Card[];
  fireTop: Card | null;
  onFireTopClick: (card: Card) => void;
  registerDeckEl?: (el: HTMLButtonElement | null) => void;
  registerFirePileEl?: (el: HTMLDivElement | null) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: "firePile", disabled: firePileDropDisabled });

  const topCard = fireTop ?? (firePile.length > 0 ? firePile[firePile.length - 1] : null);
  const shouldGlowFireDraw = Boolean(topCard && !deckDisabled);
  const shouldGlowDeckDraw = Boolean(!deckDisabled);
  const prevPeekCount = 4;
  const prevCards = firePile.slice(Math.max(0, firePile.length - 1 - prevPeekCount), Math.max(0, firePile.length - 1));
  const peekStepPx = 14;
  const cardW = 48;
  const stackW = cardW + prevCards.length * peekStepPx + 6;
  return (
    <div className="flex flex-row items-center gap-2 sm:gap-4">
      <motion.div
        ref={(node) => {
          registerFirePileEl?.(node);
        }}
        dir={topCard ? "ltr" : "rtl"}
        animate={{ scale: !firePileDropDisabled && isOver ? 1.05 : 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 24 }}
        className={`w-40 h-24 sm:w-56 sm:h-32 rounded-xl bg-gray-900/40 border-2 border-dashed relative overflow-visible text-gray-200 ${
          !firePileDropDisabled && isOver ? "border-gray-200" : "border-gray-500"
        } ${shouldGlowFireDraw ? "shadow-lg shadow-emerald-400/15" : ""}`}
      >
        {shouldGlowFireDraw ? (
          <div className="pointer-events-none absolute inset-0 rounded-xl ring-2 ring-emerald-400/50 animate-pulse" />
        ) : null}
        {/* Smaller droppable hitbox to avoid blocking table meld drops on mobile */}
        <div ref={setNodeRef} className="absolute left-1/2 top-1/2 h-20 w-28 -translate-x-1/2 -translate-y-1/2 sm:h-24 sm:w-40" />
        {topCard ? (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="relative h-20 sm:h-24 overflow-visible" style={{ width: stackW }}>
              {/* Previous cards (non-clickable): real cards, peeking to the left (consistent step) */}
              {prevCards.map((card, i) => {
                // `prevCards` is oldest -> newest, so newest should be closest to the top card.
                const right = (prevCards.length - i) * peekStepPx;
                return (
                  <div key={card.id} className="absolute top-0 pointer-events-none" style={{ right, zIndex: 10 + i }}>
                    <PlayingCard card={card} draggable={false} />
                  </div>
                );
              })}

              {/* Top card (clickable): fully visible, on the right edge */}
              <motion.button
                disabled={deckDisabled}
                type="button"
                onClick={() => onFireTopClick(topCard)}
                whileTap={{ scale: 0.98 }}
                className="absolute top-0 right-0 disabled:cursor-not-allowed"
                style={{ zIndex: 50 }}
              >
                <PlayingCard card={topCard} draggable={false} />
              </motion.button>
            </div>
          </div>
        ) : (
          <div dir="rtl" className="absolute inset-0 flex flex-col items-center justify-center">
            <div className="text-xs text-gray-300">لا توجد بطاقات</div>
            <div className="mt-1 text-xs text-gray-400">اسحب البطاقة هنا لرميها</div>
          </div>
        )}
      </motion.div>

      <motion.button
        ref={registerDeckEl}
        disabled={deckDisabled}
        type="button"
        onClick={onDeckClick}
        whileTap={{ scale: 0.98 }}
        whileHover={{ y: -2 }}
        className={`relative w-16 h-24 sm:w-24 sm:h-32 rounded-xl bg-gray-800 border border-gray-700 flex flex-col items-center justify-center text-gray-200 hover:bg-gray-750 focus:outline-none focus:ring-2 focus:ring-gray-400/40 disabled:cursor-not-allowed ${
          shouldGlowDeckDraw ? "shadow-lg shadow-emerald-400/15" : ""
        }`}
      >
        {shouldGlowDeckDraw ? (
          <div className="pointer-events-none absolute inset-0 rounded-xl ring-2 ring-emerald-400/50 animate-pulse" />
        ) : null}
        <div className="text-xs sm:text-sm font-semibold">الرزمة</div>
        <div className="mt-1 text-[10px] sm:text-[11px] text-gray-400">اضغط للسحب</div>
      </motion.button>
    </div>
  );
}

// ---- Main Game UI ----
export default function GamePage() {
  const DISPLAY_MAX_ROUNDS = 4;
  const [activeCardId, setActiveCardId] = useState<string | null>(null);
  const [currentRoundNumber, setCurrentRoundNumber] = useState<number>(1);

  const [isWaitingForServer, setIsWaitingForServer] = useState<boolean>(false);

  type WinPlayer = { id: string; name: string; score: number; wins: number };
  const [winScreen, setWinScreen] = useState<null | { players: WinPlayer[] }>(null);
  const [winActionPending, setWinActionPending] = useState(false);

  type FlyCardView = { kind: "hidden" } | { kind: "face"; card: Card; size?: "md" | "sm" };
  type FlyGroupView = { kind: "meld"; cards: Card[] };
  type FlyItem = {
    id: string;
    from: { x: number; y: number };
    to: { x: number; y: number };
    view: FlyCardView | FlyGroupView;
    durationMs: number;
    onArrive?: () => void;
  };

  const [flyItems, setFlyItems] = useState<FlyItem[]>([]);

  const deckElRef = useRef<HTMLButtonElement | null>(null);
  const firePileElRef = useRef<HTMLDivElement | null>(null);

  const playerAreaElsRef = useRef<Record<string, HTMLDivElement | null>>({});
  const bottomHandElRef = useRef<HTMLDivElement | null>(null);
  const meldElsRef = useRef<Record<string, HTMLDivElement | null>>({});

  const roundStateRef = useRef<RoundState | null>(null);
  const activePlayerRef = useRef<ActivePlayer | null>(null);

  const prevTableMeldsSnapshotRef = useRef<Record<string, Meld>>({});

  const registerPlayerAreaEl = (playerId: string, el: HTMLDivElement | null) => {
    playerAreaElsRef.current[playerId] = el;
  };

  const registerMeldEl = (meldId: string, el: HTMLDivElement | null) => {
    meldElsRef.current[meldId] = el;
  };

  const { send } = useGameSocket();

  const { player, gameState, endPoint } = useGameStore(
    useShallow((s) => ({ player: s.player!, gameState: s.gameState!, endPoint: s.endPoint }))
  );
  const startNewGame = useGameStore((s) => s.startGame);
  const resetToMenu = useGameStore((s) => s.reset);
  const { activePlayer, roundState } = useRoundStore(useShallow((s) => ({ activePlayer: s.player, roundState: s.roundState })));

  const startRound = useRoundStore((s) => s.startRound);
  const setRoundPhase = useRoundStore((s) => s.setRoundPhase);
  const setPlayer = useRoundStore((s) => s.setPlayer);
  const addCardToHand = useRoundStore((s) => s.addCardToHand);
  const setPlayerHand = useRoundStore((s) => s.setPlayerHand);
  const drawOtherPlayerHand = useRoundStore((s) => s.drawOtherPlayerHand);
  const setOtherPlayerHand = useRoundStore((s) => s.setOtherPlayerHand);
  const syncMelds = useRoundStore((s) => s.syncMelds);
  const removeMeldedCardsFromHand = useRoundStore((s) => s.removeMeldedCardsFromHand);
  const setPlayerMelded = useRoundStore((s) => s.setPlayerMelded);
  const addCardToFirePile = useRoundStore((s) => s.addCardToFirePile);
  const addCardsToFirePile = useRoundStore((s) => s.addCardsToFirePile);
  const discardCardFromHand = useRoundStore((s) => s.discardCardFromHand);
  const nextPlayer = useRoundStore((s) => s.nextPlayer);
  const setFirePile = useRoundStore((s) => s.setFirePile);
  const removeCardFromFirePile = useRoundStore((s) => s.removeCardFromFirePile);

  useEffect(() => {
    roundStateRef.current = roundState;
  }, [roundState]);

  useEffect(() => {
    activePlayerRef.current = activePlayer;
  }, [activePlayer]);

  useEffect(() => {
    const melds = roundState?.tableMelds ?? [];
    const snap: Record<string, Meld> = {};
    for (const m of melds) snap[m.id] = m;
    prevTableMeldsSnapshotRef.current = snap;
  }, [roundState?.tableMelds]);

  function getCenterFromEl(el: HTMLElement | null): { x: number; y: number } | null {
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
  }

  function getCenterFromRect(rect: { left: number; top: number; width: number; height: number } | null): { x: number; y: number } | null {
    if (!rect) return null;
    return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
  }

  function getHandArrivalPoint(playerId: string): { x: number; y: number } | null {
    // The fly overlay is `position: fixed`, so we must use viewport coordinates.
    // For the local player, land near the right edge of the hand, which visually matches
    // where new cards appear in the overlapped LTR hand.
    if (playerId === player.id) {
      const el = bottomHandElRef.current;
      if (!el) return null;
      const r = el.getBoundingClientRect();
      if (r.width <= 0 || r.height <= 0) return getCenterFromRect(r);

      const cardWidth = 64; // PlayingCard md is w-16
      return {
        x: r.right - cardWidth / 2,
        y: r.top + r.height / 2,
      };
    }

    // For opponents, use their whole area as a reasonable target.
    const el = playerAreaElsRef.current[playerId] ?? null;
    return getCenterFromEl(el);
  }

  function addFlyItem(item: Omit<FlyItem, "id">) {
    const id = crypto.randomUUID();
    const full: FlyItem = { ...item, id };
    setFlyItems((prev) => [...prev, full]);
    window.setTimeout(() => {
      full.onArrive?.();
      setFlyItems((prev) => prev.filter((x) => x.id !== id));
    }, full.durationMs);
  }

  function getPlayerTargetEl(playerId: string): HTMLDivElement | null {
    if (playerId === player.id) return bottomHandElRef.current;
    return playerAreaElsRef.current[playerId] ?? null;
  }

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 6 },
    }),
    useSensor(TouchSensor, {
      activationConstraint: { distance: 6 },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  // Prefer dropping onto meld targets when the pointer is inside them.
  // This prevents the fire pile from "stealing" drops on small screens.
  const collisionDetection: CollisionDetection = (args) => {
    const pointerCollisions = pointerWithin(args);
    const meldHit = pointerCollisions.find((c) => String(c.id).startsWith("meld:"));
    if (meldHit) return [meldHit];
    const firePileHit = pointerCollisions.find((c) => c.id === "firePile");
    if (firePileHit) return [firePileHit];
    return closestCenter(args);
  };

  useGameEvents((event) => {
    console.log("Received game event:", event);
    setIsWaitingForServer(false);
    if (event.event === "joined") {
      const { success } = event;
      if (success) {
        console.log("Successfully joined the game, sending startGame event");
        setIsWaitingForServer(true);
        send({
          event: "startGame",
          gameId: gameState.id,
          playerId: player.id,
        });
      }
    } else if (event.event === "roundStarted") {
      const { roundNumber, hand, fireCardId, melded, scores } = event;
      console.log("scores received for round start:", scores);

      // UI-only: used for the scoreboard.
      setCurrentRoundNumber(roundNumber);

      const normalizedHand = hand.map((c) => convertCardDto(c));

      const sortedHand = sortHandForRoundStart(normalizedHand);

      const myScore =
        scores?.[gameState.players.findIndex((p) => p.id === player.id)] ?? gameState.players.find((p) => p.id === player.id)?.score ?? 0;

      const activePlayer: ActivePlayer = {
        ...gameState.players.find((p) => p.id === player.id)!,
        score: myScore,
        hand: sortedHand,
        FireCardId: fireCardId,
        melded: melded,
      };

      startRound({
        currentPlayer: 0,
        deck: [],
        firePile: [],
        phase: "playing",
        players: gameState.players.map((p, index) => {
          const score = scores?.[index] ?? p.score ?? 0;
          return p.id === player.id
            ? {
                ...activePlayer,
                score,
              }
            : {
                ...p,
                hand: Array.from({ length: 14 }, () => ({ id: "hidden", suit: "Hidden", rank: "Hidden" })),
                FireCardId: null,
                melded: false,
                score,
              };
        }),
        tableMelds: [],
      });
      console.log("Round started, setting active player and round state in store", activePlayer);
      setPlayer(activePlayer);

      send({
        event: "roundStarted",
        gameId: gameState.id,
        playerId: player.id,
        roundNumber: roundNumber,
      });
    } else if (event.event === "drawPhase") {
      //   const { player_id } = event;
      setRoundPhase("draw");
    } else if (event.event === "drawnCard") {
      const { playerId, card, isFireCard, emptyFirePile } = event;

      const convertedCard = convertCardDto(
        card == null
          ? {
              id: "hidden",
              suit: "Hidden",
              rank: "Hidden",
            }
          : card
      );

      // Decide draw source:
      // - If the card existed in our local fire pile, treat it as a fire draw
      // - otherwise treat it as a deck draw.
      const currentFirePile = roundStateRef.current?.firePile ?? [];
      const fromFire = currentFirePile.some((c) => c.id === convertedCard.id);
      const fromEl = fromFire ? firePileElRef.current : deckElRef.current;
      const from = getCenterFromEl(fromEl);
      const to = getHandArrivalPoint(playerId);

      // If we can't measure, fall back to old behavior.
      if (!from || !to) {
        if (emptyFirePile) setFirePile([]);
        if (playerId === player.id) {
          addCardToHand(convertedCard, isFireCard);
        } else {
          drawOtherPlayerHand(playerId);
        }
      } else {
        // Animate a hidden card flying to the player's hand.
        addFlyItem({
          from,
          to,
          view: { kind: "hidden" },
          durationMs: 520,
          onArrive: () => {
            // Apply state only when the card reaches the hand.
            if (emptyFirePile) {
              setFirePile([]);
            } else if (fromFire) {
              // Remove the drawn card from fire pile locally.
              removeCardFromFirePile(convertedCard.id);
            }

            if (playerId === player.id) {
              addCardToHand(convertedCard, isFireCard);
            } else {
              drawOtherPlayerHand(playerId);
            }
          },
        });
      }

      send({
        event: "drawPhaseAck",
        gameId: gameState.id,
        playerId,
        senderId: player.id,
        roundNumber: currentRoundNumber,
      });
    } else if (event.event === "playingPhaseStarted") {
      //   const { player_id } = event;
      setRoundPhase("playing");
    } else if (event.event === "syncMelds") {
      const { playerId, playerHandSize, playerMelds, tableMelds, takeJoker } = event;
      const convertTableMelds: Meld[] = tableMelds.map((meldGroup) => ({
        ...meldGroup,
        cards: meldGroup.cards.map((c) => convertCardDto(c)),
      }));

      const prevTable = roundStateRef.current?.tableMelds ?? [];
      const prevIds = new Set(prevTable.map((m) => m.id));
      const nextIds = new Set(convertTableMelds.map((m) => m.id));
      const removedIds = Array.from(prevIds).filter((id) => !nextIds.has(id));
      const addedIds = Array.from(nextIds).filter((id) => !prevIds.has(id));

      // Apply table immediately (destinations exist); card flights are overlays.
      syncMelds(convertTableMelds);

      // Helper: find which meld contains a specific card in the *new* table.
      const findMeldContaining = (cardId: string) => convertTableMelds.find((m) => m.cards.some((c) => c.id === cardId)) ?? null;

      // Animate cards used by this action.
      const usedCards: Card[] = (playerMelds ?? []).map((c) => convertCardDto(c));
      const sourceEl = getPlayerTargetEl(playerId);
      const source = getCenterFromEl(sourceEl);

      if (source && usedCards.length > 0) {
        // Important: when laying melds, the destination meld DOM nodes don't exist until after
        // React renders `convertTableMelds`. Schedule flights on the next tick so refs are populated.
        window.setTimeout(() => {
          let remaining = usedCards.length;
          const onOneArrive = () => {
            remaining -= 1;
            if (remaining !== 0) return;

            // After all used cards arrive, handle removed melds that backend moved/split.
            if (removedIds.length > 0) {
              const playedCardId = usedCards.length === 1 ? usedCards[0].id : null;
              const playedStillOnTable = playedCardId ? Boolean(findMeldContaining(playedCardId)) : false;

              if (usedCards.length === 1 && playedCardId && removedIds.length > 0) {
                if (playedStillOnTable) {
                  const originMeldId = removedIds[0];
                  const originEl = meldElsRef.current[originMeldId] ?? null;
                  const origin = getCenterFromEl(originEl);
                  if (origin) {
                    window.setTimeout(() => {
                      for (const newId of addedIds) {
                        const targetEl = meldElsRef.current[newId] ?? null;
                        const target = getCenterFromEl(targetEl);
                        const meldObj = convertTableMelds.find((m) => m.id === newId) ?? null;
                        if (!target || !meldObj) continue;
                        addFlyItem({
                          from: origin,
                          to: target,
                          view: { kind: "meld", cards: meldObj.cards },
                          durationMs: 520,
                        });
                      }
                    }, 0);
                  }
                } else {
                  const fireCenter = getCenterFromEl(firePileElRef.current);
                  if (fireCenter) {
                    for (const removedId of removedIds) {
                      const originEl = meldElsRef.current[removedId] ?? null;
                      const origin = getCenterFromEl(originEl);
                      const removedMeld = prevTableMeldsSnapshotRef.current[removedId];
                      if (!origin || !removedMeld) continue;
                      addFlyItem({
                        from: origin,
                        to: fireCenter,
                        view: { kind: "meld", cards: removedMeld.cards },
                        durationMs: 560,
                        onArrive: () => {
                          addCardsToFirePile(removedMeld.cards);
                        },
                      });
                    }
                  }
                }
              } else {
                const fireCenter = getCenterFromEl(firePileElRef.current);
                if (fireCenter) {
                  for (const removedId of removedIds) {
                    const originEl = meldElsRef.current[removedId] ?? null;
                    const origin = getCenterFromEl(originEl);
                    const removedMeld = prevTableMeldsSnapshotRef.current[removedId];
                    if (!origin || !removedMeld) continue;
                    addFlyItem({
                      from: origin,
                      to: fireCenter,
                      view: { kind: "meld", cards: removedMeld.cards },
                      durationMs: 560,
                      onArrive: () => {
                        addCardsToFirePile(removedMeld.cards);
                      },
                    });
                  }
                }
              }
            }

            // Finally apply hand size updates.
            if (playerId == player.id) {
              removeMeldedCardsFromHand(playerMelds);
              setPlayerMelded(true);

              if (takeJoker) {
                const convertedJoker = convertCardDto(takeJoker);
                addCardToHand(convertedJoker, false);
              }
            } else {
              setOtherPlayerHand(playerId, playerHandSize);
            }
          };

          for (const usedCard of usedCards) {
            const targetMeld = findMeldContaining(usedCard.id);
            const targetEl = targetMeld ? meldElsRef.current[targetMeld.id] ?? null : firePileElRef.current;
            const target = getCenterFromEl(targetEl);
            if (!target) {
              onOneArrive();
              continue;
            }

            addFlyItem({
              from: source,

              to: target,
              view: { kind: "face", card: usedCard, size: "sm" },
              durationMs: 520,
              onArrive: onOneArrive,
            });
          }
        }, 0);
      } else {
        // No measured source; fall back to old immediate updates.
        if (playerId == player.id) {
          removeMeldedCardsFromHand(playerMelds);
          setPlayerMelded(true);

          if (takeJoker) {
            const convertedJoker = convertCardDto(takeJoker);
            addCardToHand(convertedJoker, false);
          }
        } else {
          setOtherPlayerHand(playerId, playerHandSize);
        }
      }

      send({
        event: "syncMeldsAck",
        gameId: gameState.id,
        playerId,
        senderId: player.id,
        roundNumber: currentRoundNumber,
      });
    } else if (event.event === "playerDiscarded") {
      const { playerId, playerHandSize, card } = event;

      const convertedCard = convertCardDto(card);

      if (playerId !== player.id) {
        const fromEl = getPlayerTargetEl(playerId);
        const toEl = firePileElRef.current;
        const from = getCenterFromEl(fromEl);
        const to = getCenterFromEl(toEl);

        if (from && to) {
          // Animate a hidden card flying from that player's hand to the fire pile.
          addFlyItem({
            from,
            to,
            view: { kind: "hidden" },
            durationMs: 520,
            onArrive: () => {
              addCardToFirePile(convertedCard);
              setOtherPlayerHand(playerId, playerHandSize);
            },
          });
        } else {
          addCardToFirePile(convertedCard);
          setOtherPlayerHand(playerId, playerHandSize);
        }
      } else {
        // Your discard is via drag/drop; we apply state immediately here.
        addCardToFirePile(convertedCard);
        discardCardFromHand(convertedCard);
      }

      nextPlayer();

      send({
        event: "playerDiscardedAck",
        gameId: gameState.id,
        playerId,
        senderId: player.id,
        roundNumber: currentRoundNumber,
      });
    } else if (event.event === "gameOver") {
      setWinScreen({ players: event.players ?? [] });
    } else {
      console.warn("Unhandled game event:", event);
    }
  });

  // Visual-only: cards that will be laid as melds if you press "إسقاط المجموعات".
  // Must be computed via a hook before any early return.
  const safeHandForVisuals = activePlayer?.hand ?? EMPTY_HAND;
  const meldCandidateIds = useMemo(() => new Set(handMelds(safeHandForVisuals)), [safeHandForVisuals]);

  const displayHand = safeHandForVisuals;
  const isMelded = activePlayer?.melded ?? false;
  const meldProgressValue = useMemo(() => {
    if (isMelded) return null;
    const melds = meldsFromHandOrderForValue(displayHand);
    return meldsValue(melds);
  }, [isMelded, displayHand]);

  if (!roundState || !activePlayer) {
    return <div className="text-black">جارٍ التحميل...</div>;
  }

  // Keep stable references for animation groups
  const players = roundState?.players ?? [];
  const isPlaying = roundState?.currentPlayer === 0;
  const canDiscardToFirePile = !isWaitingForServer && isPlaying && roundState?.phase === "playing";
  const canDrawFromDeckOrFire = !isWaitingForServer && isPlaying && roundState?.phase === "draw";
  const canLayMelds = !isWaitingForServer && isPlaying && roundState?.phase === "playing";
  const canPlayInMeld = canLayMelds;

  const handIds = displayHand.map((c) => c.id);
  const activeCard = activeCardId ? displayHand.find((c) => c.id === activeCardId) ?? null : null;

  const isMeldThresholdMet = meldProgressValue != null && meldProgressValue >= 51;
  const shouldGlowMeldButton = canLayMelds && (activePlayer.melded || isMeldThresholdMet);

  function findCardInYourHand(cardId: string) {
    // TODO: once you move hand management into the store, replace this with a store selector/action.
    return displayHand.find((c) => c.id === cardId) ?? null;
  }

  function handleDropToFirePile(cardId: string) {
    console.log("canDiscardToFirePile:", canDiscardToFirePile, "isWaitingForServer:", isWaitingForServer);

    if (!roundState || !canDiscardToFirePile) return;
    const card = findCardInYourHand(cardId);
    if (!card) return;

    setIsWaitingForServer(true);
    send({
      event: "playingPhaseStarted",
      gameId: gameState.id,
      playerId: player.id,
      roundNumber: currentRoundNumber,
      data: {
        phase: "discardPhase",
        card: convertToDTOCard(card),
      },
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function handleFireTopClick(_card: Card) {
    if (!roundState || !canDrawFromDeckOrFire) return;

    setIsWaitingForServer(true);
    send({
      event: "drawPhaseFinished",
      gameId: gameState.id,
      playerId: player.id,
      roundNumber: currentRoundNumber,
      data: {
        drawChoice: "fire",
      },
    });
  }

  function handleCommitMeld() {
    if (!roundState || !canLayMelds || !activePlayer) return;

    const melds: Meld[] = meldsFromHandOrderForValue(displayHand);
    if (melds.length === 0) return;

    const meldValue = meldsValue(melds);
    if (activePlayer.melded === false && meldValue < 51) {
      return;
    }
    setIsWaitingForServer(true);
    send({
      event: "playingPhaseStarted",
      gameId: gameState.id,
      playerId: player.id,
      roundNumber: currentRoundNumber,
      data: {
        phase: "playMeldPhase",
        melds: melds.map((meld) => ({
          ...meld,
          cards: meld.cards.map((c) => convertToDTOCard(c)),
        })),
      },
    });
  }

  function handlePlayInMeld(meldIndex: number, cardId: string, isLeft: boolean) {
    if (!roundState || !canPlayInMeld || !activePlayer || !activePlayer.melded) return;
    const card = findCardInYourHand(cardId);
    if (!card) return;

    if (!canPlayCardInMeld(roundState, card, meldIndex, isLeft)) {
      return;
    }

    const meldId = roundState.tableMelds[meldIndex]?.id;
    if (!meldId) return;

    setIsWaitingForServer(true);
    send({
      event: "playingPhaseStarted",
      gameId: gameState.id,
      playerId: player.id,
      roundNumber: currentRoundNumber,
      data: {
        phase: "playInMeldPhase",
        meldId: meldId,
        card: convertToDTOCard(card),
        isLeft: isLeft,
      },
    });
  }

  function handleDeckClick() {
    if (!roundState || !canDrawFromDeckOrFire) return;
    setIsWaitingForServer(true);
    send({
      event: "drawPhaseFinished",
      gameId: gameState.id,
      playerId: player.id,
      roundNumber: currentRoundNumber,
      data: {
        drawChoice: "deck",
      },
    });
  }

  function onDragStart(event: DragStartEvent) {
    if (isWaitingForServer) return;
    setActiveCardId(String(event.active.id));
  }

  function onDragEnd(event: DragEndEvent) {
    if (!activePlayer) return;
    if (isWaitingForServer) {
      setActiveCardId(null);
      return;
    }
    const activeId = String(event.active.id);
    const overId = event.over ? String(event.over.id) : null;

    setActiveCardId(null);

    if (!overId) return;
    if (overId === "firePile") {
      // Discarding is only allowed during the playing phase.
      if (canDiscardToFirePile) {
        // Optional: animate your dropped card flying to the fire pile.
        const card = findCardInYourHand(activeId);
        const to = getCenterFromEl(firePileElRef.current);
        const from = getCenterFromRect(event.active.rect.current.translated ?? event.active.rect.current.initial ?? null);
        if (card && from && to) {
          addFlyItem({
            from,
            to,
            view: { kind: "face", card, size: "md" },
            durationMs: 360,
          });
        }
        handleDropToFirePile(activeId);
      }
      return;
    }

    if (overId.startsWith("meld:")) {
      // Expected format: meld:{index}:{left|right}
      const parts = overId.split(":");
      const meldIndex = Number(parts[1]);
      const side = parts[2];
      if (!Number.isNaN(meldIndex)) {
        handlePlayInMeld(meldIndex, activeId, side !== "right");
      }
      return;
    }

    if (activeId === overId) return;

    // Ensure we are operating on the UI-controlled list.
    const current = activePlayer.hand;
    const oldIndex = current.findIndex((c) => c.id === activeId);
    const newIndex = current.findIndex((c) => c.id === overId);
    if (oldIndex < 0 || newIndex < 0) return;

    setPlayerHand(arrayMove(current, oldIndex, newIndex));
  }

  function onDragCancel() {
    setActiveCardId(null);
  }
  //   console.log(roundState, canDiscardToFirePile);
  return (
    <DndContext
      sensors={sensors}
      collisionDetection={collisionDetection}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      onDragCancel={onDragCancel}
    >
      <div
        dir="rtl"
        className="w-screen h-screen bg-linear-to-b from-slate-900 to-black text-white flex items-center justify-center overflow-hidden"
      >
        {/* Win screen */}
        <AnimatePresence>
          {winScreen ? (
            <motion.div
              key="win-screen"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.18 }}
              className="fixed inset-0 z-6000"
            >
              <div className="absolute inset-0 bg-black/60" />
              <div className="absolute inset-0 flex items-center justify-center p-6">
                <motion.div
                  initial={{ y: 10, opacity: 0 }}
                  animate={{ y: 0, opacity: 1 }}
                  exit={{ y: 10, opacity: 0 }}
                  transition={{ type: "spring", stiffness: 420, damping: 32 }}
                  className="w-full max-w-md rounded-2xl bg-white/5 border border-white/10 backdrop-blur p-5"
                >
                  <div className="text-lg font-semibold text-gray-100">انتهت اللعبة</div>

                  <div className="mt-4 rounded-xl bg-black/20 border border-white/10 overflow-hidden">
                    <div className="grid grid-cols-[1fr_auto_auto] gap-3 px-4 py-2 text-xs text-gray-300">
                      <div>اللاعب</div>
                      <div dir="ltr" className="tabular-nums">
                        النقاط
                      </div>
                      <div dir="ltr" className="tabular-nums">
                        الفوز
                      </div>
                    </div>
                    <div className="h-px bg-white/10" />
                    <div className="max-h-72 overflow-auto">
                      {winScreen.players.map((p) => (
                        <div key={p.id} className="grid grid-cols-[1fr_auto_auto] gap-3 px-4 py-2 text-sm">
                          <div className="text-gray-100 truncate">{p.name}</div>
                          <div dir="ltr" className="tabular-nums text-gray-100">
                            {p.score}
                          </div>
                          <div dir="ltr" className="tabular-nums text-gray-100">
                            {p.wins}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="mt-4 grid grid-cols-2 gap-3">
                    <button
                      type="button"
                      disabled={winActionPending}
                      onClick={async () => {
                        try {
                          setWinActionPending(true);
                          const result = await callStartGame(endPoint);
                          startNewGame(result);
                          setWinScreen(null);
                        } catch (e) {
                          console.error("Failed to replay:", e);
                        } finally {
                          setWinActionPending(false);
                        }
                      }}
                      className="inline-flex items-center justify-center rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      إعادة اللعب
                    </button>

                    <button
                      type="button"
                      disabled={winActionPending}
                      onClick={() => {
                        resetToMenu();
                        setWinScreen(null);
                      }}
                      className="inline-flex items-center justify-center rounded-lg bg-white/5 border border-white/10 px-4 py-2 text-sm font-semibold text-gray-100 transition hover:bg-white/10 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      الصفحة الرئيسية
                    </button>
                  </div>
                </motion.div>
              </div>
            </motion.div>
          ) : null}
        </AnimatePresence>

        {/* Fly animation layer */}
        <AnimatePresence>
          {flyItems.map((it) => (
            <motion.div
              key={it.id}
              initial={{ left: it.from.x, top: it.from.y, opacity: 1, scale: 1 }}
              animate={{ left: it.to.x, top: it.to.y, opacity: 1, scale: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: it.durationMs / 1000, ease: "easeInOut" }}
              style={{
                position: "fixed",
                transform: "translate(-50%, -50%)",
                pointerEvents: "none",
                zIndex: 5000,
              }}
            >
              {it.view.kind === "hidden" ? (
                <CardBack className="w-12 h-20" />
              ) : it.view.kind === "face" ? (
                <PlayingCard card={it.view.card} draggable={false} size={it.view.size ?? "md"} />
              ) : (
                <div dir="ltr" className="flex items-center">
                  {it.view.cards.map((c, idx) => (
                    <div key={c.id} className={idx === 0 ? "" : "-ml-3"} style={{ zIndex: 10 + idx, position: "relative" }}>
                      <PlayingCard card={c} draggable={false} size="sm" />
                    </div>
                  ))}
                </div>
              )}
            </motion.div>
          ))}
        </AnimatePresence>

        <div className="relative w-full max-w-6xl h-full flex flex-col justify-between p-2 sm:p-6">
          {/* Scoreboard (top-left) */}
          <div className="absolute top-2 left-2 z-40 w-44 sm:top-6 sm:left-6 sm:w-64 rounded-xl bg-white/5 border border-white/10 backdrop-blur px-3 py-2 sm:px-4 sm:py-3">
            <div className="flex items-center justify-between">
              <div className="text-xs sm:text-sm font-semibold text-gray-100">لوحة النتائج</div>
              <div className="text-[11px] sm:text-xs text-gray-300">
                <span className="ml-1">الجولة</span>
                <span dir="ltr" className="tabular-nums">
                  {currentRoundNumber} / {DISPLAY_MAX_ROUNDS}
                </span>
              </div>
            </div>

            <div className="mt-3 space-y-1">
              {(roundState?.players ?? []).map((p) => (
                <div key={p.id} className="flex items-center justify-between text-xs sm:text-sm">
                  <div className="text-gray-200 truncate max-w-28 sm:max-w-40">{p.name}</div>
                  <div dir="ltr" className="tabular-nums text-gray-100 text-xs sm:text-sm">
                    {p.score}
                  </div>
                </div>
              ))}
            </div>
            {/* TODO: if your server supports live score updates per action, update scores in roundStore/gameStore */}
          </div>

          {/* Top player */}
          <PlayerArea
            player={players[2]}
            stackDirection="horizontal"
            isCurrent={roundState.currentPlayer === 2}
            registerEl={registerPlayerAreaEl}
          />

          {/* Middle table */}
          <div className="flex justify-between items-center gap-2 sm:gap-0">
            <PlayerArea
              player={players[1]}
              stackDirection="vertical"
              stackMirror
              isCurrent={roundState.currentPlayer === 1}
              registerEl={registerPlayerAreaEl}
            />

            <LayoutGroup>
              <div className="flex flex-col items-center gap-4">
                <div className="w-full flex flex-col items-center justify-center gap-4 sm:flex-row">
                  <CenterTableArea
                    deckDisabled={!canDrawFromDeckOrFire}
                    firePileDropDisabled={!canDiscardToFirePile}
                    onDeckClick={handleDeckClick}
                    firePile={roundState.firePile}
                    fireTop={roundState.firePile.length > 0 ? roundState.firePile[roundState.firePile.length - 1] : null}
                    onFireTopClick={handleFireTopClick}
                    registerDeckEl={(el) => {
                      deckElRef.current = el;
                    }}
                    registerFirePileEl={(el) => {
                      firePileElRef.current = el;
                    }}
                  />
                </div>

                {/* Committed melds (UI-only for now) */}
                <div className="w-full max-w-full sm:max-w-xl">
                  <div className={`mb-2 text-xs text-gray-300 ${roundState.tableMelds.length > 0 ? "hidden sm:block" : ""}`}>
                    المجموعات على الطاولة
                  </div>
                  <div className="min-h-16 sm:min-h-20 rounded-xl bg-white/5 border border-white/10 p-2 sm:p-3">
                    <div dir="ltr" className="flex gap-2 sm:gap-4 flex-wrap justify-center">
                      {roundState.tableMelds.length === 0 ? (
                        <div className="text-xs text-gray-400">لا توجد مجموعات بعد</div>
                      ) : (
                        roundState.tableMelds.map((m, meldIndex) => (
                          <TableMeldGroup
                            key={meldIndex}
                            meld={m}
                            meldIndex={meldIndex}
                            dropDisabled={!canPlayInMeld}
                            draggedCard={activeCard}
                            registerEl={registerMeldEl}
                          />
                        ))
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </LayoutGroup>

            <PlayerArea
              player={players[3]}
              stackDirection="vertical"
              isCurrent={roundState.currentPlayer === 3}
              registerEl={registerPlayerAreaEl}
            />
          </div>

          {/* Bottom player (Your hand) */}
          <div className="flex flex-col items-center w-fit mx-auto">
            <div className="mb-2 w-full max-w-6xl flex items-start justify-between gap-3  sm:max-w-6xl">
              {/* In RTL, first child sits on the right */}
              <div className="flex items-center gap-1">
                <motion.button
                  type="button"
                  onClick={handleCommitMeld}
                  disabled={!canLayMelds}
                  className={`px-2.5 py-1 rounded-md bg-blue-600/80 border border-blue-500/50 text-white text-xs disabled:opacity-40 disabled:cursor-not-allowed hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-400/30 ${
                    shouldGlowMeldButton ? "ring-2 ring-emerald-400/60 shadow-lg shadow-emerald-400/20" : ""
                  }`}
                >
                  إسقاط المجموعات
                </motion.button>

                {!activePlayer.melded && meldProgressValue != null ? (
                  <div dir="ltr" className={`text-[11px] tabular-nums ${isMeldThresholdMet ? "text-emerald-200" : "text-gray-300"}`}>
                    {meldProgressValue} / 51
                  </div>
                ) : null}
              </div>

              {/* Second child sits on the left */}
              <div className="flex items-center gap-2">
                <span className={`text-sm ${roundState.currentPlayer === 0 ? "text-emerald-200 font-semibold" : "text-gray-300"}`}>
                  {activePlayer.name}
                </span>
                {roundState.currentPlayer === 0 ? (
                  <span className="text-[10px] px-2 py-0.5 rounded-full bg-emerald-400/15 text-emerald-200 border border-emerald-400/30">
                    الدور الآن
                  </span>
                ) : null}
              </div>
            </div>

            <LayoutGroup>
              <SortableContext items={handIds} strategy={rectSortingStrategy}>
                <div
                  ref={(el) => {
                    bottomHandElRef.current = el;
                    playerAreaElsRef.current[player.id] = el;
                  }}
                  dir="ltr"
                  className="flex flex-wrap justify-center max-w-full sm:max-w-6xl overflow-visible"
                >
                  {displayHand.map((card, idx) => (
                    <div key={card.id} className={idx === 0 ? "" : "-ml-3 sm:-ml-4"}>
                      <SortableHandCard card={card} zIndex={idx} disabled={isWaitingForServer} highlight={meldCandidateIds.has(card.id)} />
                    </div>
                  ))}
                </div>
              </SortableContext>
            </LayoutGroup>
          </div>
        </div>
      </div>

      <DragOverlay>
        {activeCard ? (
          <div className="opacity-95">
            <PlayingCard card={activeCard} draggable={false} highlight={meldCandidateIds.has(activeCard.id)} />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
