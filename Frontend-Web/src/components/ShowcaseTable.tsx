import { useEffect, useState } from "react";
import { formatChips } from "@/lib/money";
import { PlayingCard } from "./PlayingCard";

interface Seat {
  name: string;
  top: number;
  left: number;
  cards?: string[];
  chips: number;
  bet: number;
  dealer?: boolean;
  you?: boolean;
}

interface Scenario {
  label: string;
  seats: Seat[];
  boards: string[][];
  pots: number[];
}

const SCENARIOS: Scenario[] = [
  {
    label: "Você de A K contra 3 na mesa",
    seats: [
      { name: "você", top: 86, left: 50, cards: ["As", "Kd"], chips: 24_850, bet: 150, you: true },
      { name: "Ana", top: 28, left: 14, chips: 15_200, bet: 150 },
      { name: "Leo", top: 10, left: 50, chips: 30_100, bet: 0, dealer: true },
      { name: "Mari", top: 28, left: 86, chips: 9_800, bet: 150 },
    ],
    boards: [[], ["Ah", "7c", "2d"], ["Ah", "7c", "2d", "9s"], ["Ah", "7c", "2d", "9s", "Kh"]],
    pots: [450, 1_450, 3_200, 6_800],
  },
  {
    label: "Par de Damas trinca no flop",
    seats: [
      { name: "você", top: 86, left: 50, cards: ["Qh", "Qd"], chips: 21_300, bet: 400, you: true },
      { name: "Beto", top: 28, left: 14, chips: 18_700, bet: 400 },
      { name: "Leo", top: 10, left: 50, chips: 27_900, bet: 0, dealer: true },
      { name: "Sofia", top: 28, left: 86, chips: 12_400, bet: 400 },
    ],
    boards: [[], ["Qc", "7s", "2h"], ["Qc", "7s", "2h", "9d"], ["Qc", "7s", "2h", "9d", "3c"]],
    pots: [1_200, 3_600, 7_400, 12_000],
  },
];

/** Mesa de vitrine na home/auth — feltro real, sem WebSocket. Anima flop→river em loop. */
export function ShowcaseTable({ className = "" }: { className?: string }) {
  const [scenarioIdx, setScenarioIdx] = useState(0);
  const [streetIdx, setStreetIdx] = useState(1);

  useEffect(() => {
    if (
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ) {
      return;
    }
    const id = window.setInterval(() => {
      setStreetIdx((street) => {
        const scenario = SCENARIOS[scenarioIdx];
        if (street + 1 < scenario.boards.length) return street + 1;
        setScenarioIdx((s) => (s + 1) % SCENARIOS.length);
        return 0;
      });
    }, 3200);
    return () => window.clearInterval(id);
  }, [scenarioIdx]);

  const scenario = SCENARIOS[scenarioIdx];
  const board = scenario.boards[streetIdx];
  const pot = scenario.pots[streetIdx];

  return (
    <div className="flex w-full flex-col items-center gap-2">
      <div className={`zt-felt-table zt-showcase-table ${className}`.trim()} aria-hidden>
        <div className="absolute left-1/2 top-1/2 z-20 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2">
          <div className="rounded border border-gold/40 bg-black/40 px-3 py-1 text-center">
            <div className="text-[10px] uppercase tracking-wider text-gold-soft">Pot</div>
            <div key={pot} className="font-mono text-base font-bold text-white">
              {formatChips(pot)}
            </div>
          </div>
          <div className="flex min-h-[72px] items-center gap-1.5">
            {board.map((c, i) => (
              <span key={`${scenarioIdx}-${streetIdx}-${c}`} className="zt-deal" style={{ animationDelay: `${i * 160}ms` }}>
                <PlayingCard code={c} size="md" />
              </span>
            ))}
          </div>
        </div>
        {scenario.seats.map((p) => (
          <div
            key={p.name}
            className="zt-seat"
            style={{ top: `${p.top}%`, left: `${p.left}%` }}
          >
            <div className={`zt-seat-card ${p.you ? "active" : ""}`}>
              <div className="flex items-center justify-between gap-1 text-[10px] text-felt-300">
                <span>{p.dealer ? "D" : " "}</span>
                {p.you && <span className="text-gold-bright">você</span>}
              </div>
              <div className="truncate text-xs font-semibold text-cream">{p.name}</div>
              <div className="font-mono text-[11px] text-gold-soft">{formatChips(p.chips)}</div>
              {p.bet > 0 && (
                <div className="mt-0.5 text-[10px] text-felt-200">Aposta {formatChips(p.bet)}</div>
              )}
              {p.cards && p.cards.length > 0 && (
                <div className="mt-1 flex justify-center gap-0.5">
                  {p.cards.map((c) => (
                    <PlayingCard key={c} code={c} size="sm" />
                  ))}
                </div>
              )}
              {!p.cards && (
                <div className="mt-1 flex justify-center gap-0.5">
                  <PlayingCard faceDown size="sm" />
                  <PlayingCard faceDown size="sm" />
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
      <p className="text-[11px] text-felt-400" aria-hidden>
        {scenario.label} · mesa ilustrativa
      </p>
    </div>
  );
}
