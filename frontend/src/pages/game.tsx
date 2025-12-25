import { motion, LayoutGroup } from "framer-motion";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
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
import { useState, type CSSProperties } from "react";
import type { ActivePlayer, Card, Rank, Suit } from "../lib/logic";
import { useGameEvents } from "../lib/provider/event";
import { useGameSocket } from "../lib/provider/useGameSocket";
import { useGameStore } from "../lib/store/gameStore";
import { useRoundStore } from "../lib/store/roundStore";

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
}

function rankSortKey(rank: Rank): number {
  // TODO: tweak rank ordering if your rules differ.
  if (rank === "Joker") return 99;
  if (typeof rank === "number") return rank;
  switch (rank) {
    case "J":
      return 11;
    case "Q":
      return 12;
    case "K":
      return 13;
    case "A":
      return 14;
  }
}

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

// ---- Card Component ----
function PlayingCard({ card, draggable = true }: { card: Card; draggable?: boolean }) {
  const isRed = card.suit === "Hearts" || card.suit === "Diamonds";

  return (
    <motion.div
      whileHover={draggable ? { y: -12 } : undefined}
      whileTap={draggable ? { scale: 0.95 } : undefined}
      transition={{ type: "spring", stiffness: 520, damping: 34 }}
      className={`w-16 h-24 rounded-xl shadow-lg border bg-white flex flex-col justify-between p-2 select-none ${
        draggable ? "cursor-grab" : "cursor-default"
      } ${isRed ? "text-red-600" : "text-black"}`}
    >
      <span className="text-sm font-bold">{card.rank.toString()}</span>
      <span className="text-xl text-center">{display_suit(card.suit)}</span>
      <span className="text-sm font-bold self-end">{card.rank.toString()}</span>
    </motion.div>
  );
}

function SortableHandCard({ card, disabled, zIndex }: { card: Card; disabled?: boolean; zIndex?: number }) {
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
      <div {...attributes} {...listeners}>
        <PlayingCard card={card} draggable={!disabled} />
      </div>
    </div>
  );
}

function CardBack({ className }: { className?: string }) {
  return (
    <div
      className={
        "w-12 h-20 rounded-lg bg-gray-800 border border-gray-600 shadow-md " +
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
}: {
  player: ActivePlayer;
  isBottom?: boolean;
  stackDirection?: StackDirection;
  stackMirror?: boolean;
}) {
  return (
    <div className={`flex flex-col items-center ${isBottom ? "mt-6" : ""}`}>
      <span className="mb-2 text-sm text-gray-300">{player.name}</span>

      <div className="flex gap-2 flex-wrap justify-center max-w-4xl">
        {isBottom ? (
          player.hand.map((card) => <PlayingCard key={card.id} card={card} />)
        ) : (
          <StackedHand
            // UI-only: other players' hands are hidden, so fall back to a reasonable placeholder count.
            // TODO: when server sends other players' hand sizes, use that instead of 15.
            count={player.hand.length > 0 ? player.hand.length : 15}
            direction={stackDirection}
            mirror={stackMirror}
          />
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
}: {
  onDeckClick: () => void;
  firePile: Card[];
  fireTop: Card | null;
  onFireTopClick: (card: Card) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: "firePile" });

  const topCard = fireTop ?? (firePile.length > 0 ? firePile[firePile.length - 1] : null);
  const prevPeekCount = 5;
  const prevCards = firePile.slice(Math.max(0, firePile.length - 1 - prevPeekCount), Math.max(0, firePile.length - 1));
  const peekStepPx = 18;
  const cardW = 64;
  const stackW = cardW + prevCards.length * peekStepPx + 6;

  return (
    <div className="flex gap-4 items-center">
      <motion.div
        ref={setNodeRef}
        animate={{ scale: isOver ? 1.05 : 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 24 }}
        className={`w-56 h-32 rounded-xl bg-gray-900/40 border-2 border-dashed relative overflow-visible text-gray-200 ${
          isOver ? "border-gray-200" : "border-gray-500"
        }`}
      >
        {topCard ? (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="relative h-24 overflow-visible" style={{ width: stackW }}>
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
                type="button"
                onClick={() => onFireTopClick(topCard)}
                whileTap={{ scale: 0.98 }}
                className="absolute top-0 right-0"
                style={{ zIndex: 50 }}
              >
                <PlayingCard card={topCard} draggable={false} />
              </motion.button>
            </div>
          </div>
        ) : (
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <div className="text-xs text-gray-300">لا توجد بطاقات</div>
            <div className="mt-1 text-xs text-gray-400">اسحب البطاقة هنا لرميها</div>
          </div>
        )}
      </motion.div>

      <motion.button
        type="button"
        onClick={onDeckClick}
        whileTap={{ scale: 0.98 }}
        whileHover={{ y: -2 }}
        className="w-24 h-32 rounded-xl bg-gray-800 border border-gray-700 flex flex-col items-center justify-center text-gray-200 hover:bg-gray-750 focus:outline-none focus:ring-2 focus:ring-gray-400/40"
      >
        <div className="text-sm font-semibold">الرزمة</div>
        <div className="mt-1 text-[11px] text-gray-400">اضغط للسحب</div>
      </motion.button>
    </div>
  );
}

// ---- Main Game UI ----
export default function GamePage() {
  // UI-only hand order for the local player.
  // TODO: when your backend supports it, sync this order to the server / store.
  const [handCards, setHandCards] = useState<Card[]>([]);
  const [firePileCards, setFirePileCards] = useState<Card[]>([]);
  const [fireTopCard, setFireTopCard] = useState<Card | null>(null);
  const [activeCardId, setActiveCardId] = useState<string | null>(null);
  const [currentRoundNumber, setCurrentRoundNumber] = useState<number>(1);
  const { send } = useGameSocket();
  const player = useGameStore((s) => s.player!);
  const gameState = useGameStore((s) => s.gameState!);
  const roundState = useRoundStore((s) => s.roundState);
  const startRound = useRoundStore((s) => s.startRound);
  const setRoundPhase = useRoundStore((s) => s.setRoundPhase);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 6 },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  // Prefer dropping onto the fire pile when the pointer is inside it.
  // This avoids cases where `closestCenter` picks a nearby hand card instead.
  const collisionDetection: CollisionDetection = (args) => {
    const pointerCollisions = pointerWithin(args);
    const firePileHit = pointerCollisions.find((c) => c.id === "firePile");
    if (firePileHit) return [firePileHit];
    return closestCenter(args);
  };

  useGameEvents((event) => {
    if (event.event === "joined") {
      const { success } = event;
      if (success) {
        console.log("Successfully joined the game, sending startGame event");
        send({
          event: "startGame",
          gameId: gameState.id,
          playerId: player.id,
        });
      }
    } else if (event.event === "roundStarted") {
      console.log("Received roundStarted event:", event);
      const { roundNumber, hand, fireCardId, melded } = event;

      // UI-only: used for the scoreboard.
      setCurrentRoundNumber(roundNumber);

      // Normalize incoming ranks (backend sometimes sends `{ Number: ... }`).
      const normalizedHand = hand.map((c) => ({
        ...c,
        rank: typeof c.rank === "object" ? (((c.rank as any).Number ?? "Joker") as Rank) : (c.rank as Rank),
      }));

      const sortedHand = sortHandForRoundStart(normalizedHand);

      // UI-only: set local hand order on round start.
      setHandCards(sortedHand);

      // UI-only: reset fire pile rendering on new round.
      // TODO: when your backend sends the fire pile cards, initialize them here instead.
      setFirePileCards([]);
      setFireTopCard(null);

      startRound({
        currentPlayer: 0,
        deck: [],
        firePile: [],
        phase: "meld",
        players: gameState.players.map((p) => ({
          ...p,
          hand: p.id === player.id ? sortedHand : [],
          FireCardId: p.id === player.id ? fireCardId : null,
          melded: p.id === player.id ? melded : false,
        })),
        tableMelds: [],
      });
      console.log(roundNumber);
      send({
        event: "roundStarted",
        gameId: gameState.id,
        playerId: player.id,
        roundNumber: roundNumber,
      });
    } else if (event.event === "drawPhase") {
      //   const { player_id } = event;
      setRoundPhase("draw");
    } else if (event.event === "playingPhaseStarted") {
      //   const { player_id } = event;
      setRoundPhase("playing");
    } else {
      console.warn("Unhandled game event:", event);
    }
  });

  // Keep stable references for animation groups
  const players = roundState?.players ?? [];
  const you = players[0];
  const isPlaying = roundState?.currentPlayer === 0;

  // Prefer UI-order if set; otherwise fall back to store order.
  // TODO: if you want this to survive refresh/reconnect, store this order in zustand or backend.
  const displayHand = handCards.length > 0 ? handCards : you?.hand ?? [];
  const handIds = displayHand.map((c) => c.id);
  const activeCard = activeCardId ? displayHand.find((c) => c.id === activeCardId) ?? null : null;

  if (!roundState) {
    return <div className="text-white">جارٍ التحميل...</div>;
  }

  function findCardInYourHand(cardId: string) {
    // TODO: once you move hand management into the store, replace this with a store selector/action.
    return displayHand.find((c) => c.id === cardId) ?? null;
  }

  function handleDropToFirePile(cardId: string) {
    if (!roundState || !isPlaying) return;
    if (roundState.phase !== "playing") {
      return;
    }
    // TODO: send discard action to server and update round store.
    // Example idea (depends on your backend protocol):
    // send({ event: "discard", gameId: gameState.id, playerId: player.id, cardId });
    const card = findCardInYourHand(cardId);
    if (!card) return;

    // UI-only: track top card separately so it always shows immediately.
    setFireTopCard(card);

    // UI-only: add to fire pile.
    setFirePileCards((prev) => {
      // Defensive: during HMR/state migrations, `prev` can be non-array.
      return Array.isArray(prev) ? [...prev, card] : [card];
    });

    // UI-only: remove discarded card from your hand.
    setHandCards((prev) => prev.filter((c) => c.id !== card.id));
  }

  function handleFireTopClick(card: Card) {
    if (!roundState || !isPlaying) return;
    if (roundState.phase !== "draw") return;
    // TODO: implement "take from fire pile" rules.
    // Example idea: send({ event: "takeFire", gameId: gameState.id, playerId: player.id });
    console.log("Fire pile top clicked (UI only):", card.id);
  }

  function handleCommitMeld() {
    // TODO: meld is based on the *current order of your hand*.
    // Send `handCards.map(c => c.id)` to your backend and let it compute meld groups.
    // Example idea (depends on your backend protocol):
    // send({ event: "meldByOrder", gameId: gameState.id, playerId: player.id, orderedCardIds: handCards.map(c => c.id) });
    console.log(
      "Meld clicked (UI only). Current order:",
      displayHand.map((c) => c.id)
    );
  }

  function handleDeckClick() {
    // TODO: implement draw logic based on current rules (from deck vs fire pile, allowed phase, etc.)
    // Example idea: send({ event: "draw", gameId: gameState.id, playerId: player.id });
    console.log("Deck clicked (UI only). Implement draw logic here.");
  }

  function onDragStart(event: DragStartEvent) {
    setActiveCardId(String(event.active.id));
    // If user starts dragging before we copied store order into UI order, adopt it now.
    if (handCards.length === 0 && you.hand.length > 0) {
      setHandCards(you.hand);
    }
  }

  function onDragEnd(event: DragEndEvent) {
    const activeId = String(event.active.id);
    const overId = event.over ? String(event.over.id) : null;

    setActiveCardId(null);

    if (!overId) return;
    if (overId === "firePile") {
      handleDropToFirePile(activeId);
      return;
    }
    if (activeId === overId) return;

    // Ensure we are operating on the UI-controlled list.
    const current = handCards.length > 0 ? handCards : you.hand;
    const oldIndex = current.findIndex((c) => c.id === activeId);
    const newIndex = current.findIndex((c) => c.id === overId);
    if (oldIndex < 0 || newIndex < 0) return;

    setHandCards(arrayMove(current, oldIndex, newIndex));
  }

  function onDragCancel() {
    setActiveCardId(null);
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={collisionDetection}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      onDragCancel={onDragCancel}
    >
      <div dir="rtl" className="w-screen h-screen bg-linear-to-b from-slate-900 to-black text-white flex items-center justify-center">
        <div className="relative w-full max-w-6xl h-full flex flex-col justify-between p-6">
          {/* Scoreboard (top-left) */}
          <div className="absolute top-6 left-6 w-64 rounded-xl bg-white/5 border border-white/10 backdrop-blur px-4 py-3">
            <div className="flex items-center justify-between">
              <div className="text-sm font-semibold text-gray-100">لوحة النتائج</div>
              <div className="text-xs text-gray-300">
                الجولة {currentRoundNumber} / {gameState.max_round}
              </div>
            </div>

            <div className="mt-3 space-y-1">
              {(roundState?.players ?? []).map((p) => (
                <div key={p.id} className="flex items-center justify-between text-sm">
                  <div className="text-gray-200 truncate max-w-40">{p.name}</div>
                  <div className="tabular-nums text-gray-100">{p.score}</div>
                </div>
              ))}
            </div>
            {/* TODO: if your server supports live score updates per action, update scores in roundStore/gameStore */}
          </div>

          {/* Top player */}
          <PlayerArea player={players[1]} stackDirection="horizontal" />

          {/* Middle table */}
          <div className="flex justify-between items-center">
            <PlayerArea player={players[2]} stackDirection="vertical" stackMirror />

            <LayoutGroup>
              <div className="flex flex-col items-center gap-4">
                <div className="w-full flex items-center justify-center gap-4">
                  <CenterTableArea
                    onDeckClick={handleDeckClick}
                    firePile={firePileCards}
                    fireTop={fireTopCard}
                    onFireTopClick={handleFireTopClick}
                  />

                  <div className="flex flex-col items-center gap-2">
                    <motion.button
                      type="button"
                      onClick={handleCommitMeld}
                      disabled={handCards.length === 0}
                      whileTap={{ scale: handCards.length === 0 ? 1 : 0.98 }}
                      className="px-4 py-2 rounded-lg bg-blue-600/80 border border-blue-500/50 text-white disabled:opacity-40 disabled:cursor-not-allowed hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-400/30"
                    >
                      إسقاط المجموعات
                    </motion.button>
                    <div className="text-[11px] text-gray-400 max-w-44 text-center">
                      {/* TODO: اربط هذا الزر بقوانين المرحلة/الدور في المنطق */}
                      رتّب بطاقات يدك ثم اضغط لتأكيد المجموعات
                    </div>
                  </div>
                </div>

                {/* Committed melds (UI-only for now) */}
                <div className="w-full max-w-xl">
                  <div className="mb-2 text-xs text-gray-300">المجموعات على الطاولة</div>
                  <div className="min-h-20 rounded-xl bg-white/5 border border-white/10 p-3">
                    <div className="flex gap-2 flex-wrap justify-center">
                      {roundState.tableMelds.length === 0 ? (
                        <div className="text-xs text-gray-400">لا توجد مجموعات بعد</div>
                      ) : (
                        roundState.tableMelds.flatMap((m, meldIndex) =>
                          m.cards.map((card) => <PlayingCard key={`${meldIndex}:${card.id}`} card={card} draggable={false} />)
                        )
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </LayoutGroup>

            <PlayerArea player={players[3]} stackDirection="vertical" />
          </div>

          {/* Bottom player (Your hand) */}
          <div className="flex flex-col items-center">
            <span className="mb-2 text-sm text-gray-300">{you.name}</span>

            <LayoutGroup>
              <SortableContext items={handIds} strategy={rectSortingStrategy}>
                <div className="flex flex-wrap justify-center max-w-6xl overflow-visible">
                  {displayHand.map((card, idx) => (
                    <div key={card.id} className={idx === 0 ? "" : "-mr-4"}>
                      <SortableHandCard card={card} zIndex={idx} />
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
            <PlayingCard card={activeCard} draggable={false} />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
