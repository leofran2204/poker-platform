import type { PlayerWsData, PotWsData, ShowdownEntry } from "@/api/types";
import { SEAT_LAYOUT } from "@/lib/cards";
import { formatChips } from "@/lib/money";
import { PlayingCard } from "./PlayingCard";

/** Nomes de mão EN (motor) → PT-BR exibido. */
export const HAND_NAME_PT: Record<string, string> = {
  "High Card": "Carta Alta",
  "One Pair": "Um Par",
  "Two Pair": "Dois Pares",
  "Three of a Kind": "Trinca",
  Straight: "Sequência",
  Flush: "Flush",
  "Full House": "Full House",
  "Four of a Kind": "Quadra",
  "Straight Flush": "Straight Flush",
  "Royal Flush": "Royal Flush",
};

export function handNamePt(name: string | null | undefined): string {
  if (!name) return "a melhor mão";
  return HAND_NAME_PT[name] ?? name;
}

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
  winners?: string[];
  turnLeft?: number | null;
  showdown?: ShowdownEntry[];
  /** Ritual do crupiê: embaralhando + distribuindo carta por carta. */
  dealing?: boolean;
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
  winners = [],
  turnLeft = null,
  showdown = [],
  dealing = false,
}: Props) {
  const potTotal = pots.reduce((s, p) => s + p.amount, 0);
  const showdownCards = new Set(showdown.flatMap((entry) => entry.cards));
  const winnerNames = winners.map(
    (id) =>
      showdown.find((entry) => entry.player_id === id)?.player_name ??
      players.find((p) => p.id === id)?.name ??
      id,
  );
  const winningHand = showdown.find((entry) => winners.includes(entry.player_id))?.hand_name
    ?? showdown[0]?.hand_name
    ?? null;
  const normalizedActions = availableActions.map((action) => action.toLowerCase());
  const wagerAction = normalizedActions.includes("bet")
    ? "bet"
    : normalizedActions.includes("raise")
      ? "raise"
      : null;
  const canAllIn = normalizedActions.some(
    (action) => action === "allin" || action === "all-in",
  );
  // Ordem de distribuição a partir do dealer (SB primeiro, como no ao vivo).
  const orderFromDealer = (() => {
    const seated = [...players].sort((a, b) => a.seat - b.seat);
    const dealerIdx = Math.max(
      0,
      seated.findIndex((p) => p.is_dealer),
    );
    const order = new Map<string, number>();
    seated.forEach((p, i) => {
      // SB recebe primeiro: distância à frente do dealer (dealer = último).
      order.set(p.id, (i - dealerIdx - 1 + seated.length * 2) % seated.length);
    });
    return order;
  })();
  // Quantidade de cartas por jogador (espelha a minha mão p/ os versos).
  const holeCount =
    players.find((p) => p.id === localPlayerId)?.cards.length ??
    Math.max(0, ...players.map((p) => p.cards.length));

  return (
    <div>
      {dealing && (
        <div
          className="zt-deal-banner mb-3 rounded border-2 border-gold-bright bg-gold/15 px-4 py-2 text-center text-sm font-bold text-gold-bright"
          role="status"
        >
          🃏 Embaralhando o baralho e distribuindo as cartas…
        </div>
      )}
      {winners.length > 0 && (
        <div
          className="zt-winner-banner mb-3 rounded border-2 border-gold-bright bg-gold/15 px-4 py-2 text-center text-sm font-bold text-gold-bright"
          role="status"
        >
          🏆 {winnerNames.join(" + ")} venceu{winners.length > 1 ? "ram" : ""}{" "}
          {showdown.length > 0 ? (
            <>com {handNamePt(winningHand)}</>
          ) : (
            <>(todos foldaram)</>
          )}
        </div>
      )}
      <div className="mb-3 flex items-center justify-between text-sm">
        <span className="font-semibold text-gold-bright">Mesa ao vivo</span>
        <span className="text-felt-300">
          Street: <strong className="text-cream">{stage || "—"}</strong>
        </span>
        {turnLeft !== null && (
          <span
            className={`font-mono font-bold ${turnLeft <= 10 ? "text-red-300" : "text-gold-soft"}`}
            aria-label={`Tempo para agir: ${turnLeft} segundos`}
          >
            {turnLeft <= 0 ? "Fold automático…" : `Sua vez: ${turnLeft}s`}
          </span>
        )}
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
              communityCards.map((c, i) => (
                <span key={`${c}-${i}`} className="zt-deal" style={{ animationDelay: `${i * 220}ms` }}>
                  <PlayingCard code={c} size="md" highlight={showdownCards.has(c)} />
                </span>
              ))
            )}
          </div>
        </div>

        {/* Seats */}
        {players.map((p) => {
          const layout = SEAT_LAYOUT[p.seat % SEAT_LAYOUT.length] ?? SEAT_LAYOUT[0];
          const isLocal = p.id === localPlayerId;
          const isHouseBot = /^bot_\d{3}$/.test(p.name);
          const isWinner = winners.includes(p.id);
          const classes = [
            "zt-seat-card",
            p.is_active ? "active" : "",
            !p.is_active && p.cards.length === 0 ? "folded" : "",
            isWinner ? "winner" : "",
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
                <div className="truncate text-xs font-semibold text-cream">
                  {isHouseBot ? (
                    <span title="Bot da casa — garante ação na mesa">🤖 {p.name}</span>
                  ) : (
                    p.name
                  )}
                </div>
                {isWinner && <div className="text-[10px] font-bold text-gold-bright">VENCEDOR</div>}
                <div className="font-mono text-[11px] text-gold-soft">{formatChips(p.chips)}</div>
                {p.bet > 0 && (
                  <div className="mt-0.5 text-[10px] text-felt-200">Aposta {formatChips(p.bet)}</div>
                )}
                {p.cards.length > 0 && (
                  <div className="mt-1 flex justify-center gap-0.5">
                    {p.cards.map((c, i) => (
                      <span
                        key={`${p.id}-${i}`}
                        className="zt-deal"
                        style={{ animationDelay: `${(orderFromDealer.get(p.id) ?? 0) * 140 + i * 90}ms` }}
                      >
                        <PlayingCard code={c} size="md" highlight={showdownCards.has(c)} />
                      </span>
                    ))}
                  </div>
                )}
                {dealing && p.cards.length === 0 && !p.folded && p.is_sitting !== false && holeCount > 0 && (
                  <div className="mt-1 flex justify-center gap-0.5" aria-label="Cartas sendo distribuídas">
                    {Array.from({ length: holeCount }).map((_, i) => (
                      <span
                        key={`${p.id}-back-${i}`}
                        className="zt-deal"
                        style={{ animationDelay: `${(orderFromDealer.get(p.id) ?? 0) * 140 + i * 90}ms` }}
                      >
                        <PlayingCard faceDown size="md" />
                      </span>
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
