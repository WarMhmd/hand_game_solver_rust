import { motion, LayoutGroup, AnimatePresence } from "framer-motion";
import { useMemo, useState } from "react";
import type { ActivePlayer, Card, Player, Suit } from "./lib/logic";

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

function generateHand(count: number): Card[] {
  return Array.from({ length: count }).map((_, i) => ({
    id: crypto.randomUUID(),
    rank: RANKS[i % RANKS.length],
    suit: SUITS[i % SUITS.length],
  }));
}

const initialPlayers: ActivePlayer[] = [
  {
    name: "You",
    id: "p1",
    FireCardId: null,
    melded: false,
    score: 0,
    hand: generateHand(15), // 15 cards at start
  },
  { id: "p2", name: "Bot 1", hand: [] },
  { id: "p3", name: "Bot 2", hand: [] },
  { id: "p4", name: "Bot 3", hand: [] },
];

// ---- Card Component ----
function PlayingCard({ card, draggable = true }: { card: Card; draggable?: boolean }) {
  const isRed = card.suit === "Hearts" || card.suit === "Diamonds";

  return (
    <motion.div
      layoutId={card.id}
      layout
      draggable={draggable}
      onDragStart={(e) => {
        if (!draggable) return;
        e.dataTransfer.setData("cardId", card.id);
        // Hide the browser's default drag preview (we'll rely on the card staying visible + snap anim)
        const img = new Image();
        img.src = "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMSIgaGVpZ2h0PSIxIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjwvc3ZnPg==";
        e.dataTransfer.setDragImage(img, 0, 0);
      }}
      whileHover={draggable ? { y: -12 } : undefined}
      whileTap={draggable ? { scale: 0.95 } : undefined}
      transition={{ type: "spring", stiffness: 520, damping: 34 }}
      className={`w-16 h-24 rounded-xl shadow-lg border bg-white flex flex-col justify-between p-2 select-none ${
        draggable ? "cursor-grab" : "cursor-default"
      } ${isRed ? "text-red-600" : "text-black"}`}
    >
      <span className="text-sm font-bold">{card.rank}</span>
      <span className="text-xl text-center">{display_suit(card.suit)}</span>
      <span className="text-sm font-bold self-end">{card.rank}</span>
    </motion.div>
  );
}

// ---- Player Area ----
function PlayerArea({ player, isBottom }: { player: ActivePlayer; isBottom?: boolean }) {
  return (
    <div className={`flex flex-col items-center ${isBottom ? "mt-6" : ""}`}>
      <span className="mb-2 text-sm text-gray-300">{player.name}</span>

      <div className="flex gap-2 flex-wrap justify-center max-w-4xl">
        {isBottom
          ? player.hand.map((card) => <PlayingCard key={card.id} card={card} />)
          : Array.from({ length: 15 }).map((_, i) => <div key={i} className="w-12 h-20 rounded-lg bg-gray-700 border border-gray-600" />)}
      </div>
    </div>
  );
}

// ---- Table Meld Area (Drop Only Here) ----
function TableArea({
  onDropCard,
  onHoverChange,
  isHover,
}: {
  onDropCard: (cardId: string) => void;
  onHoverChange: (v: boolean) => void;
  isHover: boolean;
}) {
  return (
    <div className="flex gap-4 items-center">
      <motion.div
        onDragEnter={() => onHoverChange(true)}
        onDragLeave={() => onHoverChange(false)}
        onDragOver={(e) => {
          e.preventDefault();
          if (!isHover) onHoverChange(true);
        }}
        onDrop={(e) => {
          e.preventDefault();
          onHoverChange(false);
          const cardId = e.dataTransfer.getData("cardId");
          if (cardId) onDropCard(cardId);
        }}
        animate={{ scale: isHover ? 1.05 : 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 24 }}
        className={`w-56 h-32 rounded-xl bg-green-800 border-2 border-dashed flex items-center justify-center text-gray-200 ${
          isHover ? "border-green-200" : "border-green-400"
        }`}
      >
        Drop Cards Here (Meld)
      </motion.div>

      <div className="w-24 h-32 rounded-xl bg-gray-800 border border-gray-700 flex items-center justify-center text-gray-200">Deck</div>
    </div>
  );
}

// ---- Main Game UI ----
export default function GamePage() {
  const [players, setPlayers] = useState(initialPlayers);
  const [meld, setMeld] = useState<Card[]>([]);
  const [hoverMeld, setHoverMeld] = useState(false);

  // Keep stable references for animation groups
  const you = players[0];

  function handleDrop(cardId: string) {
    // Move the card from your hand -> meld
    setPlayers((prev) => {
      const me = prev[0];
      const card = me.hand.find((c) => c.id === cardId);
      if (!card) return prev;

      // Update meld (separate state)
      setMeld((m) => [...m, card]);

      return [{ ...me, hand: me.hand.filter((c) => c.id !== cardId) }, ...prev.slice(1)];
    });
  }

  const meldIds = useMemo(() => new Set(meld.map((c) => c.id)), [meld]);

  return (
    <div className="w-screen h-screen bg-gradient-to-b from-slate-900 to-black text-white flex items-center justify-center">
      <div className="relative w-full max-w-6xl h-full flex flex-col justify-between p-6">
        {/* Top player */}
        <PlayerArea player={players[1]} />

        {/* Middle table */}
        <div className="flex justify-between items-center">
          <PlayerArea player={players[2]} />

          <LayoutGroup>
            <div className="flex flex-col items-center gap-4">
              <TableArea onDropCard={handleDrop} isHover={hoverMeld} onHoverChange={setHoverMeld} />

              {/* Meld Row (snap animation target) */}
              <div className="min-h-[110px] flex gap-2 flex-wrap justify-center max-w-xl">
                <AnimatePresence initial={false}>
                  {meld.map((card) => (
                    <PlayingCard key={card.id} card={card} draggable={false} />
                  ))}
                </AnimatePresence>
              </div>
            </div>
          </LayoutGroup>

          <PlayerArea player={players[3]} />
        </div>

        {/* Bottom player (Your hand) */}
        <div className="flex flex-col items-center">
          <span className="mb-2 text-sm text-gray-300">{you.name}</span>

          <LayoutGroup>
            <div className="flex gap-2 flex-wrap justify-center max-w-6xl">
              {you.hand.map((card) => (
                <PlayingCard key={card.id} card={card} draggable={!meldIds.has(card.id)} />
              ))}
            </div>
          </LayoutGroup>
        </div>
      </div>
    </div>
  );
}
