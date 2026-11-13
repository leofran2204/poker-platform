import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import type { PlayerWsData, PotWsData, ShowdownEntry } from "@/api/types";
import { buzz, playTableSound, setSoundEnabled, soundEnabled } from "@/lib/tableSounds";
import { seatPosition } from "@/lib/cards";
import { handNamePt, streetLabel } from "@/lib/gameLabels";
import { formatChips } from "@/lib/money";
import { PlayingCard } from "./PlayingCard";

export { HAND_NAME_PT, handNamePt } from "@/lib/gameLabels";

interface Props {
  players: PlayerWsData[];
  communityCards: string[];
  stage: string;
  pots: PotWsData[];
  localPlayerId?: string | null;
  availableActions: string[];
  onAction: (action: string, amount?: number) => void;
  bettingStructure?: string | null;
  /** Brazilian Pineapple: rodada de troca em andamento (depois das apostas do flop e do turn). */
  drawing?: boolean;
  /** Brazilian Pineapple: descarte obrigatório do river em andamento. */
  discarding?: boolean;
  ante?: number;
  antePaid?: number;
  antePlayerId?: string | null;
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
  maxPlayers?: number;
  /** Índice a partir do qual o board anima (flop 0; turn/river só a carta nova). */
  boardStaggerFrom?: number;
}

export function PokerTable({
  players,
  communityCards,
  stage,
  pots,
  localPlayerId,
  availableActions,
  onAction,
  bettingStructure,
  drawing = false,
  discarding = false,
  ante = 0,
  antePaid = 0,
  antePlayerId,
  raiseAmount,
  onRaiseChange,
  callAmount,
  minimumWager,
  maximumWager,
  winners = [],
  turnLeft = null,
  showdown = [],
  dealing = false,
  maxPlayers = 9,
  boardStaggerFrom = 0,
}: Props) {
  const pineapple = bettingStructure === "pineapple_draw_v1";
  const wager = raiseAmount;
  const streetBase = pineapple ? (players.find(p => p.id === localPlayerId)?.bet ?? 0) + callAmount : 0;
  const potTotal = pots.reduce((s, p) => s + p.amount, 0);
  const clampWager = (amount: number) => {
    const withMinimum = Math.max(minimumWager || 0, amount);
    return maximumWager > 0 ? Math.min(maximumWager, withMinimum) : withMinimum;
  };
  const wagerPresets = [
    { label: "½ pote", amount: clampWager(streetBase + Math.round(potTotal / 2)) },
    { label: "⅔ pote", amount: clampWager(streetBase + Math.round((potTotal * 2) / 3)) },
    { label: pineapple ? "Máximo" : "Pote", amount: pineapple ? maximumWager : clampWager(potTotal) },
  ].filter((preset, index, all) => all.findIndex((item) => item.amount === preset.amount) === index);
  const showdownCards = new Set(showdown.flatMap((entry) => entry.cards));
  // Só as cartas do jogo vencedor saltam: cartas dos entries de quem ganhou.
  // (Antes brilhava toda carta revelada, o que diluía o vencedor.)
  const winnerEntries = showdown.filter((entry) => winners.includes(entry.player_id));
  const winningCards = new Set(winnerEntries.flatMap((entry) => entry.cards));
  const glowCards = winningCards.size > 0 ? winningCards : showdownCards;
  // Suspense do showdown: mostra tudo normal e após ~1s acende as 5 do jogo
  // vencedor com glow forte, escurecendo o resto. Vale p/ cash/MTT, PM/Real.
  const showdownKey =
    winners.length > 0 && winningCards.size > 0
      ? `${winners.join("+")}:${[...winningCards].sort().join(",")}`
      : "";
  const [revealStrong, setRevealStrong] = useState(false);
  useEffect(() => {
    if (!showdownKey) {
      setRevealStrong(false);
      return;
    }
    if (
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ) {
      setRevealStrong(true);
      return;
    }
    const id = window.setTimeout(() => setRevealStrong(true), 2000);
    return () => window.clearTimeout(id);
  }, [showdownKey]);
  const strongWin = revealStrong && winningCards.size > 0;
  const winWrap = (c: string) => {
    if (!glowCards.has(c)) return strongWin ? "zt-dim" : undefined;
    if (!strongWin) return "zt-win-pop";
    return "zt-win-pop zt-win-card";
  };
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
  // Brazilian Pineapple: na minha vez da rodada de troca, escolho manter ou
  // trocar uma carta; no river, escolho a carta que descarto.
  const canDraw = normalizedActions.includes("keep");
  const canDiscard = normalizedActions.includes("discard");
  const myCards = players.find((p) => p.id === localPlayerId)?.cards ?? [];
  const [swapPick, setSwapPick] = useState<number | null>(null);
  useEffect(() => {
    if (!canDraw && !canDiscard) setSwapPick(null);
  }, [canDraw, canDiscard]);
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
  const heroSeat = players.find((p) => p.id === localPlayerId)?.seat ?? null;
  const cap = Math.max(maxPlayers, ...players.map((p) => p.seat + 1), 2);
  const dealerPos = (() => {
    const dealer = players.find((p) => p.is_dealer);
    if (!dealer) return { top: 12, left: 50 };
    return seatPosition(dealer.seat, heroSeat, cap);
  })();
  const blindIds = (() => {
    const seated = [...players].sort((a, b) => a.seat - b.seat);
    const dealerIdx = seated.findIndex((p) => p.is_dealer);
    if (dealerIdx < 0 || seated.length < 2) return { sb: null as string | null, bb: null as string | null };
    if (seated.length === 2) {
      return { sb: seated[dealerIdx].id, bb: seated[(dealerIdx + 1) % 2].id };
    }
    return {
      sb: seated[(dealerIdx + 1) % seated.length].id,
      bb: seated[(dealerIdx + 2) % seated.length].id,
    };
  })();
  const holeSize = holeCount >= 3 ? "sm" : "md";

  /* ── Mesa viva: fichas em movimento e sons, deduzidos da mudança de estado ── */
  type Flight = { id: string; from: { top: number; left: number }; to: { top: number; left: number }; gold?: boolean };
  const [flights, setFlights] = useState<Flight[]>([]);
  const [sound, setSound] = useState(soundEnabled);
  const previous = useRef<{ bets: Map<string, number>; pot: number; board: number; myTurn: boolean; winners: string; cards: number } | null>(null);
  const POT_SPOT = { top: 41, left: 44 };
  const betSpot = (layout: { top: number; left: number }) => ({
    top: layout.top + (50 - layout.top) * 0.42,
    // Assentos do meio (em cima e embaixo): as fichas ficam ao lado das cartas.
    left: Math.abs(layout.left - 50) < 5 ? (layout.top < 45 ? 62 : 38) : layout.left + (50 - layout.left) * 0.36,
  });
  const betsKey = players.map((p) => `${p.id}:${p.bet}`).join("|");
  const winnersKey = winners.join("+");
  const myTurn = availableActions.length > 0;
  const myCardCount = myCards.length;
  useEffect(() => {
    const now = {
      bets: new Map(players.map((p) => [p.id, p.bet])),
      pot: potTotal,
      board: communityCards.length,
      myTurn,
      winners: winnersKey,
      cards: myCardCount,
    };
    const before = previous.current;
    previous.current = now;
    if (!before) return;
    const launched: Flight[] = [];
    const stamp = Date.now();
    for (const p of players) {
      const was = before.bets.get(p.id) ?? 0;
      const layout = seatPosition(p.seat, heroSeat, cap);
      if (p.bet > was) {
        // Aposta: as fichas saem do jogador e param na frente dele.
        launched.push({ id: `b-${p.id}-${stamp}`, from: layout, to: betSpot(layout) });
      } else if (was > 0 && p.bet === 0 && now.pot >= before.pot) {
        // Rodada fechada: as apostas são recolhidas para o pote.
        launched.push({ id: `c-${p.id}-${stamp}`, from: betSpot(layout), to: POT_SPOT });
      }
    }
    if (winnersKey && winnersKey !== before.winners) {
      for (const id of winners) {
        const winner = players.find((p) => p.id === id);
        if (winner) launched.push({ id: `w-${id}-${stamp}`, from: POT_SPOT, to: seatPosition(winner.seat, heroSeat, cap), gold: true });
      }
    }
    if (launched.length > 0) {
      setFlights((current) => [...current, ...launched]);
      playTableSound(launched.length > 1 || launched[0].gold ? "chips" : "chip");
      const ids = new Set(launched.map((flight) => flight.id));
      window.setTimeout(() => setFlights((current) => current.filter((flight) => !ids.has(flight.id))), 750);
    }
    if (now.board > before.board) playTableSound("flip");
    if (now.cards > before.cards && before.cards === 0) playTableSound("deal");
    if (now.myTurn && !before.myTurn) {
      playTableSound("pop");
      buzz();
    }
    // As dependências são chaves de valor: o efeito roda quando o estado muda de fato.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [betsKey, potTotal, communityCards.length, myTurn, winnersKey, myCardCount]);
  // Mão em andamento: adversários que não desistiram aparecem com as cartas fechadas.
  const inHand = ["preflop", "flop", "turn", "river"].includes(stage) && winners.length === 0;

  return (
    <div>
      {dealing && (
        <div
          className="zt-deal-banner mb-3 rounded-sm border-2 border-gold-bright bg-gold/15 px-4 py-2 text-center text-sm font-bold text-gold-bright"
          role="status"
        >
          Embaralhando o baralho e distribuindo as cartas…
        </div>
      )}
      {winners.length > 0 && (
        <div
          className="zt-winner-banner mb-3 rounded-sm border-2 border-gold-bright bg-gold/15 px-4 py-2 text-center text-sm font-bold text-gold-bright"
          role="status"
        >
          {winnerNames.join(" + ")} {winners.length > 1 ? "venceram" : "venceu"}{" "}
          {showdown.length > 0 ? (
            <>com {handNamePt(winningHand)}</>
          ) : (
            <>(todos foldaram)</>
          )}
        </div>
      )}
      <div className="mb-3 flex items-center justify-between text-sm">
        <span className="flex items-center gap-2 font-semibold text-gold-bright">
          Mesa ao vivo
          <button
            type="button"
            className="zt-sound-toggle"
            aria-pressed={sound}
            aria-label={sound ? "Desligar o som da mesa" : "Ligar o som da mesa"}
            title={sound ? "Som ligado" : "Som desligado"}
            onClick={() => {
              setSoundEnabled(!sound);
              setSound(!sound);
              if (!sound) playTableSound("pop");
            }}
          >
            {sound ? "🔊" : "🔇"}
          </button>
        </span>
        <span className="text-felt-300">
          Street: <strong className="text-cream">{streetLabel(stage)}</strong>
        </span>
        {turnLeft !== null && (
          <span
            className={`font-mono font-bold ${turnLeft <= 10 ? "text-red-300" : "text-gold-soft"}`}
            aria-label={`Tempo para agir: ${turnLeft} segundos`}
          >
            {turnLeft <= 0 ? (drawing ? "Mantendo as cartas…" : discarding ? "Descartando a carta que sobra…" : "Fold automático…") : `Sua vez: ${turnLeft}s`}
          </span>
        )}
      </div>

      <div className="zt-felt-table">
        {/* Center: pot + board */}
        <div className="absolute left-1/2 top-1/2 z-20 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2">
          <div className="zt-pot">
            {potTotal > 0 && (
              <span className="zt-chip-stack gold" aria-hidden="true">
                <i />
                <i />
                <i />
                <i />
              </span>
            )}
            <span className="zt-pot-label">Pote</span>
            <motion.span
              key={potTotal}
              className="zt-pot-value"
              initial={{ scale: 1.22, color: "#fff3c4" }}
              animate={{ scale: 1, color: "#ecd99a" }}
              transition={{ duration: 0.35 }}
            >
              {formatChips(potTotal)}
            </motion.span>
          </div>
          <div className="flex gap-1.5">
            {communityCards.length === 0 ? (
              <span className="text-xs italic text-felt-200/60">
                {players.filter((p) => p.is_sitting !== false).length < 2
                  ? "Precisa de 2 na mesa"
                  : stage === "preflop"
                    ? "Pré-flop"
                    : "Sem board"}
              </span>
            ) : (
              communityCards.map((c, i) => (
                <motion.span
                  key={`${c}-${i}`}
                  initial={{ opacity: 0, y: -26, rotateY: 90, scale: 0.9 }}
                  animate={{ opacity: 1, y: 0, rotateY: 0, scale: 1 }}
                  transition={{ delay: Math.max(0, i - boardStaggerFrom) * 0.18, duration: 0.38, ease: [0.16, 1, 0.3, 1] }}
                  style={{ display: "inline-block", transformPerspective: 600 }}
                >
                  <span className={winWrap(c)}>
                    <PlayingCard code={c} size="md" highlight={glowCards.has(c)} />
                  </span>
                </motion.span>
              ))
            )}
          </div>
        </div>

        {/* Fichas apostadas na rodada: sobre o feltro, entre o jogador e o pote */}
        <AnimatePresence>
          {players.map((p) => {
            if (p.bet <= 0) return null;
            const spot = betSpot(seatPosition(p.seat, heroSeat, cap));
            return (
              <motion.div
                key={`bet-${p.id}`}
                className="zt-bet-spot"
                style={{ top: `${spot.top}%`, left: `${spot.left}%` }}
                initial={{ opacity: 0, scale: 0.6 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.7 }}
                transition={{ duration: 0.22, delay: 0.25 }}
                aria-label={`Aposta de ${p.name}: ${formatChips(p.bet)}`}
              >
                <span className="zt-chip-stack" aria-hidden="true">
                  <i />
                  <i />
                  <i />
                </span>
                <motion.span key={p.bet} className="zt-bet-amount" initial={{ scale: 1.2 }} animate={{ scale: 1 }}>
                  {formatChips(p.bet)}
                </motion.span>
              </motion.div>
            );
          })}
        </AnimatePresence>

        {/* Fichas em movimento: aposta, recolhimento para o pote e pote para o vencedor */}
        {flights.map((flight) => (
          <motion.span
            key={flight.id}
            className={`zt-chip-flight ${flight.gold ? "gold" : ""}`}
            aria-hidden="true"
            initial={{ top: `${flight.from.top}%`, left: `${flight.from.left}%`, opacity: 0.2, scale: 0.8 }}
            animate={{ top: `${flight.to.top}%`, left: `${flight.to.left}%`, opacity: 1, scale: 1 }}
            transition={{ duration: 0.55, ease: [0.16, 1, 0.3, 1] }}
          >
            <i />
            <i />
            <i />
          </motion.span>
        ))}

        {/* Seats */}
        {players.map((p) => {
          const layout = seatPosition(p.seat, heroSeat, cap);
          const isLocal = p.id === localPlayerId;
          const isHouseBot = /^bot_\d{3}$/.test(p.name);
          const isWinner = winners.includes(p.id);
          const sittingOut = p.is_sitting === false;
          const classes = [
            "zt-seat-plate",
            p.is_active ? "active" : "",
            p.folded === true ? "folded" : "",
            sittingOut ? "folded" : "",
            isWinner ? "winner" : "",
            isLocal ? "local" : "",
          ]
            .filter(Boolean)
            .join(" ");
          const dealDx = `${((dealerPos.left - layout.left) * 6).toFixed(0)}px`;
          const dealDy = `${((dealerPos.top - layout.top) * 4).toFixed(0)}px`;
          const blindTag = blindIds.sb === p.id ? "SB" : blindIds.bb === p.id ? "BB" : null;
          // Metade de cima da mesa: as cartas ficam abaixo da placa, viradas para o centro.
          const upper = layout.top < 45;
          const status = sittingOut
            ? "Sit-out"
            : isWinner
              ? "Vencedor"
              : typeof p.draw_count === "number" && stage !== "preflop"
                ? p.draw_count === 0
                  ? "Manteve"
                  : stage === "river"
                    ? "Descartou"
                    : `Trocou ${p.draw_count}`
                : null;
          const cards = (
            <>
              {p.cards.length > 0 && (
                <div className="zt-seat-cards">
                  {p.cards.map((c, i) => (
                    <span
                      key={`${p.id}-${i}`}
                      className="zt-deal"
                      style={{
                        animationDelay: `${(orderFromDealer.get(p.id) ?? 0) * 140 + i * 90}ms`,
                        ["--deal-dx" as string]: dealDx,
                        ["--deal-dy" as string]: dealDy,
                      }}
                    >
                      <span className={winWrap(c)}>
                        <PlayingCard code={c} size={holeSize} highlight={glowCards.has(c)} />
                      </span>
                    </span>
                  ))}
                </div>
              )}
              {!dealing && !isLocal && p.cards.length === 0 && inHand && !p.folded && p.is_sitting !== false && holeCount > 0 && (
                <div className="zt-seat-cards hidden-hand" aria-label={`${p.name} tem ${holeCount} cartas fechadas`}>
                  {Array.from({ length: holeCount }).map((_, i) => (
                    <PlayingCard key={`${p.id}-hidden-${i}`} faceDown size="sm" />
                  ))}
                </div>
              )}
              {dealing && p.cards.length === 0 && !p.folded && p.is_sitting !== false && holeCount > 0 && (
                <div className="zt-seat-cards" aria-label="Cartas sendo distribuídas">
                  {Array.from({ length: holeCount }).map((_, i) => (
                    <span
                      key={`${p.id}-back-${i}`}
                      className="zt-deal"
                      style={{
                        animationDelay: `${(orderFromDealer.get(p.id) ?? 0) * 140 + i * 90}ms`,
                        ["--deal-dx" as string]: dealDx,
                        ["--deal-dy" as string]: dealDy,
                      }}
                    >
                      <PlayingCard faceDown size={holeSize} />
                    </span>
                  ))}
                </div>
              )}
            </>
          );

          return (
            <div
              key={p.id}
              className={`zt-seat zt-seat-ft ${upper ? "upper" : ""}`}
              style={{ top: `${layout.top}%`, left: `${layout.left}%` }}
            >
              {!upper && cards}
              <div className={classes}>
                <span className="zt-avatar" aria-hidden="true">
                  {isHouseBot ? "🤖" : p.name.slice(0, 1).toUpperCase()}
                </span>
                <span className="zt-seat-info">
                  <span className="zt-seat-name" title={isHouseBot ? "Bot da casa — garante ação na mesa" : p.name}>
                    {p.name}
                    {isLocal && <span className="zt-seat-you"> · você</span>}
                  </span>
                  <span className="zt-seat-chips">{formatChips(p.chips)}</span>
                </span>
                {p.is_dealer && (
                  <span className="zt-dealer-button" title="Botão do dealer" aria-label="Dealer">
                    D
                  </span>
                )}
                {blindTag && <span className="zt-blind-tag">{blindTag}</span>}
              </div>
              {status && <div className={`zt-seat-status ${isWinner ? "winner" : ""}`}>{status}</div>}
              {upper && cards}
            </div>
          );
        })}
      </div>

      {pineapple && <p className="text-center text-xs text-felt-200 mt-3">Troca opcional de uma carta depois das apostas do flop e do turn · No river, cada jogador descarta uma carta e joga a última rodada com duas · Todas as rodadas: aumento limitado ao pote antes do call</p>}
      {drawing && !canDraw && <p className="text-center text-xs font-semibold text-gold-soft" role="status">Rodada de troca: cada jogador mantém as cartas ou troca uma.</p>}
      {discarding && !canDiscard && <p className="text-center text-xs font-semibold text-gold-soft" role="status">Descarte do river: cada jogador descarta uma carta e fica com duas.</p>}
      {ante > 0 && <p className="text-center text-xs text-felt-200">Ante do BB {formatChips(ante)} · pago {formatChips(antePaid)} por {players.find(p => p.id === antePlayerId)?.name ?? "big blind"}</p>}
      <div className="zt-action-bar">
        {canDraw || canDiscard ? (
          <div className="flex flex-wrap items-end gap-3" aria-label={canDiscard ? "Descarte do river" : "Rodada de troca"}>
            <div>
              <p className="text-xs font-semibold text-felt-200">
                {canDiscard
                  ? "Descarte do river: escolha a carta que sai. Você joga a última rodada com duas."
                  : "Rodada de troca: escolha uma carta para trocar ou mantenha as três."}
              </p>
              <div className="mt-1 flex gap-1.5">
                {myCards.map((code, index) => (
                  <button
                    key={`${code}-${index}`}
                    type="button"
                    className={`rounded-sm p-0.5 ${swapPick === index ? "ring-2 ring-gold-bright" : "ring-1 ring-transparent"}`}
                    aria-pressed={swapPick === index}
                    aria-label={`Selecionar carta ${index + 1} para ${canDiscard ? "descartar" : "trocar"}`}
                    onClick={() => setSwapPick(swapPick === index ? null : index)}
                  >
                    <PlayingCard code={code} size="md" />
                  </button>
                ))}
              </div>
            </div>
            {canDraw && (
              <button type="button" className="zt-btn-secondary" onClick={() => onAction("keep", 0)}>
                Manter as três
              </button>
            )}
            <button
              type="button"
              className="zt-btn-primary"
              disabled={swapPick === null}
              onClick={() => swapPick !== null && onAction(canDiscard ? "discard" : "swap", swapPick)}
            >
              {canDiscard ? "Descartar carta" : "Trocar carta"}
            </button>
          </div>
        ) : availableActions.length === 0 ? (
          <span className="text-sm text-felt-400">
            {players.filter((p) => p.is_sitting !== false).length < 2
              ? "Aguardando ≥2 jogadores na mesa para começar a mão."
              : stage === "waiting" || stage === "finished"
                ? "Entre mãos."
                : players.find((p) => p.is_active)
                  ? `Vez de ${players.find((p) => p.is_active)?.name ?? "outro jogador"}.`
                  : communityCards.length === 0 && stage === "preflop"
                    ? "Distribuindo…"
                    : "Aguardando sua vez…"}
          </span>
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
              <div className="flex flex-wrap items-end gap-2">
                <label className="text-xs font-semibold text-felt-200">
                  Total na rodada
                  <input
                    type="number"
                    min={(minimumWager || 0) / 100}
                    max={maximumWager ? maximumWager / 100 : undefined}
                    step={0.01}
                    className="zt-input mt-1 w-28"
                    value={(raiseAmount / 100).toFixed(2)}
                    onChange={(e) => {
                      const reais = Number(e.target.value);
                      onRaiseChange(Number.isFinite(reais) ? Math.round(reais * 100) : 0);
                    }}
                    aria-label="Valor da aposta em reais"
                  />
                </label>
                <div className="flex gap-1" aria-label="Atalhos de tamanho da aposta">
                  {wagerPresets.map((preset) => (
                    <button
                      key={preset.label}
                      type="button"
                      className="zt-tab px-2! py-1! text-xs!"
                      onClick={() => onRaiseChange(preset.amount)}
                    >
                      {preset.label}
                    </button>
                  ))}
                </div>
                <button
                  type="button"
                  disabled={!Number.isInteger(wager) || wager < minimumWager || wager > maximumWager}
                  className="zt-btn-primary"
                  onClick={() => onAction(wagerAction, wager)}
                >
                  {wagerAction === "bet" ? `Apostar ${formatChips(wager)}` : `Aumentar para ${formatChips(wager)}`}
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
