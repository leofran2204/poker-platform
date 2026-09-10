import { formatChips } from "@/lib/money";
import { PlayingCard } from "./PlayingCard";

/** Mesa de vitrine na home/auth — feltro real, sem WebSocket. */
const SEATS: {
  name: string;
  top: number;
  left: number;
  cards?: string[];
  chips: number;
  bet: number;
  dealer?: boolean;
  you?: boolean;
}[] = [
  { name: "você", top: 86, left: 50, cards: ["As", "Kd"], chips: 24_850, bet: 150, you: true },
  { name: "Ana", top: 28, left: 14, chips: 15_200, bet: 150 },
  { name: "Leo", top: 10, left: 50, chips: 30_100, bet: 0, dealer: true },
  { name: "Mari", top: 28, left: 86, chips: 9_800, bet: 150 },
];

const BOARD = ["Ah", "7c", "2d"];

export function ShowcaseTable({ className = "" }: { className?: string }) {
  return (
    <div className={`zt-felt-table zt-showcase-table ${className}`.trim()} aria-hidden>
      <div className="absolute left-1/2 top-1/2 z-20 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2">
        <div className="rounded border border-gold/40 bg-black/40 px-3 py-1 text-center">
          <div className="text-[10px] uppercase tracking-wider text-gold-soft">Pot</div>
          <div className="font-mono text-base font-bold text-white">{formatChips(1_450)}</div>
        </div>
        <div className="flex gap-1.5">
          {BOARD.map((c, i) => (
            <span key={c} className="zt-deal" style={{ animationDelay: `${i * 220}ms` }}>
              <PlayingCard code={c} size="md" />
            </span>
          ))}
        </div>
      </div>
      {SEATS.map((p) => (
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
                {p.cards.map((c, i) => (
                  <span key={c} className="zt-deal" style={{ animationDelay: `${400 + i * 90}ms` }}>
                    <PlayingCard code={c} size="sm" />
                  </span>
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
  );
}
