import type { PlayerWsData, PotWsData } from "@/api/types";
import { SEAT_LAYOUT } from "@/lib/cards";
import { formatChips } from "@/lib/money";
import { PlayingCard } from "./PlayingCard";

interface Props {
  players: PlayerWsData[];
  communityCards: string[];
  stage: string;
  pots: PotWsData[];
  localPlayerId?: string | null;
  availableActions: string[];
  onAction: (action: string, amount?: number) => void;
  raiseAmount: number;
  onRaiseChange: (v: number) => void;
  callAmount: number;
  minimumWager: number;
  maximumWager: number;
}

export function PokerTable({
  players,
  communityCards,
  stage,
  pots,
  localPlayerId,
  availableActions,
  onAction,
  raiseAmount,
  onRaiseChange,
  callAmount,
  minimumWager,
  maximumWager,
}: Props) {
  const potTotal = pots.reduce((s, p) => s + p.amount, 0);
  const normalizedActions = availableActions.map((action) => action.toLowerCase());
  const wagerAction = normalizedActions.includes("bet")
    ? "bet"
    : normalizedActions.includes("raise")
      ? "raise"
      : null;
  const canAllIn = normalizedActions.some(
    (action) => action === "allin" || action === "all-in",
  );

  return (
    <div>
      <div className="mb-3 flex items-center justify-between text-sm">
        <span className="font-semibold text-gold-bright">Mesa ao vivo</span>
        <span className="text-felt-300">
          Street: <strong className="text-cream">{stage || "—"}</strong>
        </span>
      </div>

      <div className="zt-felt-table">
        {/* Center: pot + board */}
        <div className="absolute left-1/2 top-1/2 z-20 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2">
          <div className="rounded border border-gold/40 bg-black/40 px-3 py-1 text-center">
            <div className="text-[10px] uppercase tracking-wider text-gold-soft">Pot</div>
            <div className="font-mono text-base font-bold text-white">{formatChips(potTotal)}</div>
          </div>
          <div className="flex gap-1.5">
            {communityCards.length === 0 ? (
              <span className="text-xs italic text-felt-200/60">Aguardando flop…</span>
            ) : (
              communityCards.map((c, i) => <PlayingCard key={`${c}-${i}`} code={c} size="sm" />)
            )}
          </div>
        </div>

        {/* Seats */}
        {players.map((p) => {
          const layout = SEAT_LAYOUT[p.seat % SEAT_LAYOUT.length] ?? SEAT_LAYOUT[0];
          const isLocal = p.id === localPlayerId;
          const classes = [
            "zt-seat-card",
            p.is_active ? "active" : "",
            !p.is_active && p.cards.length === 0 ? "folded" : "",
          ]
            .filter(Boolean)
            .join(" ");

          return (
            <div
              key={p.id}
              className="zt-seat"
              style={{ top: `${layout.top}%`, left: `${layout.left}%` }}
            >
              <div className={classes}>
                <div className="flex items-center justify-between gap-1 text-[10px] text-felt-300">
                  <span>{p.is_dealer ? "D" : `S${p.seat}`}</span>
                  {isLocal && <span className="text-gold-bright">você</span>}
                </div>
                <div className="truncate text-xs font-semibold text-cream">{p.name}</div>
                <div className="font-mono text-[11px] text-gold-soft">{formatChips(p.chips)}</div>
                {p.bet > 0 && (
                  <div className="mt-0.5 text-[10px] text-felt-200">Aposta {formatChips(p.bet)}</div>
                )}
                {p.cards.length > 0 && (
                  <div className="mt-1 flex justify-center gap-0.5">
                    {p.cards.map((c, i) => (
                      <PlayingCard key={`${p.id}-${i}`} code={c} size="sm" />
                    ))}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      <div className="zt-action-bar">
        {availableActions.length === 0 ? (
          <span className="text-sm text-felt-400">Aguardando sua vez…</span>
        ) : (
          <>
            {availableActions.map((raw) => {
              const a = raw.toLowerCase();
              if (a === "bet" || a === "raise" || a === "allin" || a === "all-in") return null;
              const label =
                a === "fold"
                  ? "Fold"
                  : a === "check"
                    ? "Check"
                    : a === "call"
                      ? `Call ${formatChips(callAmount)}`
                      : raw;
              const danger = a === "fold";
              return (
                <button
                  key={raw}
                  type="button"
                  className={danger ? "zt-btn-danger" : "zt-btn-secondary"}
                  onClick={() => onAction(a)}
                >
                  {label}
                </button>
              );
            })}
            {wagerAction && (
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min={minimumWager}
                  max={maximumWager || undefined}
                  step={100}
                  className="zt-input w-28"
                  value={raiseAmount}
                  onChange={(e) => onRaiseChange(Number(e.target.value) || 0)}
                  aria-label="Valor da aposta em centavos"
                />
                <button
                  type="button"
                  className="zt-btn-primary"
                  onClick={() => onAction(wagerAction, raiseAmount)}
                >
                  {wagerAction === "bet" ? "Bet" : "Raise"}
                </button>
              </div>
            )}
            {canAllIn && (
              <button
                type="button"
                className="zt-btn-secondary"
                onClick={() => onAction("allin", 0)}
              >
                All-in
              </button>
            )}
          </>
        )}
      </div>
    </div>
  );
}
