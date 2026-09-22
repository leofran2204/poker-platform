import { useEffect, useMemo, useRef, useState } from "react";
import { PlayingCard } from "@/components/PlayingCard";
import { cashbackFor } from "@/lib/deflator";
import { formatBrlFromCents } from "@/lib/money";

export interface AnimatedStreet {
  label: string;
  board: string[];
  /** Cartas visíveis de cada assento neste passo (Pineapple cresce street a street). */
  holes?: string[][];
  /** Narração deste passo (MP3). */
  audio?: string;
}

export interface AnimatedSeat {
  name: string;
  cards?: string[];
  folded?: boolean;
  isHero?: boolean;
  isWinner?: boolean;
  stack?: string;
}

export interface WinningFive {
  /** Cartas da mão do vencedor usadas no melhor jogo de 5. */
  hole: string[];
  /** Cartas do board usadas no melhor jogo de 5. */
  board: string[];
}

export interface AnimatedHandData {
  id: string;
  title: string;
  heroLabel: string;
  heroCards: string[];
  villainLabel: string;
  villainCards: string[];
  streets: AnimatedStreet[];
  equity: number;
  potCents: number;
  result: string;
  /** Índice da street em que o all-in foi pago. Ausente = exemplo sem all-in (variantes). */
  allInStreetIndex?: number;
  /** Assentos da mesa. Ausente = derivado de herói/vilão + 1 fictício foldado. */
  seats?: AnimatedSeat[];
  /** As 5 cartas do jogo vencedor. Ausente = destaca a mão do vencedor (legado). */
  winningFive?: WinningFive;
}

/** Posições dos assentos no feltro (herói embaixo, como na mesa real). */
const SEAT_POS = [
  { top: 88, left: 50 },
  { top: 16, left: 84 },
  { top: 16, left: 16 },
];

/** Deslocamento das fichas do pote até o assento vencedor. */
const CHIP_FLIGHT = [
  { dx: "0px", dy: "72px" },
  { dx: "110px", dy: "-56px" },
  { dx: "-110px", dy: "-56px" },
];

/** Pausa entre uma batida e a próxima — as streets se acumulam, nada some da tela. */
const STEP_MS = 3000;

function phaseOf(boardLen: number): string {
  if (boardLen === 0) return "PRÉ-FLOP";
  if (boardLen === 3) return "FLOP";
  if (boardLen === 4) return "TURN";
  return "RIVER";
}

/** Replay animado de uma mão numa mini-mesa com jogadores — o "vídeo" didático sem MP4. */
export function AnimatedHand({ hand }: { hand: AnimatedHandData }) {
  const last = hand.streets.length - 1;
  const [step, setStep] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [reducedMotion] = useState(
    () =>
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );

  const seats: AnimatedSeat[] = useMemo(() => {
    if (hand.seats && hand.seats.length > 0) return hand.seats.slice(0, 3);
    if (hand.villainCards.length === 0) {
      return [
        { name: hand.heroLabel, cards: hand.heroCards, isHero: true, isWinner: true },
        { name: "Leo", folded: true },
        { name: "Mari", folded: true },
      ];
    }
    return [
      { name: hand.heroLabel, cards: hand.heroCards, isHero: true },
      { name: hand.villainLabel, cards: hand.villainCards, isWinner: true },
      { name: "Ana", folded: true },
    ];
  }, [hand]);

  const winnerIdx = Math.max(
    0,
    seats.findIndex((s) => s.isWinner),
  );
  const flight = CHIP_FLIGHT[Math.min(winnerIdx, CHIP_FLIGHT.length - 1)];

  const win = hand.winningFive;
  const useWinFive = !!win && win.hole.length + win.board.length === 5;
  const winHole = useMemo(() => new Set(hand.winningFive?.hole ?? []), [hand]);
  const winBoard = useMemo(() => new Set(hand.winningFive?.board ?? []), [hand]);

  const allInIdx = hand.allInStreetIndex;
  const hasAllIn = typeof allInIdx === "number";
  const allInPaid = hasAllIn && step >= (allInIdx as number);
  const split =
    hasAllIn && hand.equity >= 56
      ? cashbackFor(hand.equity, hand.potCents)
      : null;

  useEffect(() => {
    setStep(0);
    setPlaying(false);
  }, [hand.id]);

  useEffect(() => {
    if (!playing || reducedMotion) return;
    if (step >= last) {
      setPlaying(false);
      return;
    }
    const id = window.setTimeout(() => setStep((s) => Math.min(s + 1, last)), STEP_MS);
    return () => window.clearTimeout(id);
  }, [playing, step, last, reducedMotion]);

  const street = hand.streets[step];
  const finished = step === last;
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const speak = useRef(false);

  useEffect(() => {
    speak.current = false;
    audioRef.current?.pause();
  }, [hand.id]);

  const playStep = (src?: string) => {
    const el = audioRef.current;
    if (!el || !src) return;
    el.src = src;
    el.currentTime = 0;
    void el.play().catch(() => undefined);
  };

  useEffect(() => {
    if (!speak.current) return;
    playStep(street.audio);
    // Só quando o passo muda. O botão Assistir dispara o áudio do passo atual.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step]);
  const phase = phaseOf(street.board.length);
  // Board estável como nas plataformas famosas: o que já está na mesa fica
  // parado; só a carta nova anima. prevLen = cartas que já estavam no passo anterior.
  const prevLen = step > 0 ? hand.streets[step - 1].board.length : 0;
  // Suspense do river: o board completa e só ~1s depois o jogo vencedor acende.
  const [showWin, setShowWin] = useState(false);
  useEffect(() => {
    setShowWin(false);
    if (!finished) return;
    if (reducedMotion) {
      setShowWin(true);
      return;
    }
    const id = window.setTimeout(() => setShowWin(true), 2000);
    return () => window.clearTimeout(id);
  }, [finished, hand.id, step, reducedMotion]);

  return (
    <div className="zt-card overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-felt-600 bg-felt-900/70 px-4 py-2.5">
        <h3 className="text-sm font-bold text-cream">{hand.title}</h3>
        <button
          type="button"
          onClick={() => {
            if (finished) {
              speak.current = true;
              setStep(0);
              setPlaying(true);
              return;
            }
            setPlaying((p) => {
              const next = !p;
              speak.current = next;
              if (!next) audioRef.current?.pause();
              else playStep(street.audio);
              return next;
            });
          }}
          className="zt-btn-secondary !px-3 !py-1 !text-xs"
          aria-label={playing ? "Pausar replay" : finished ? "Rever replay" : "Reproduzir replay"}
        >
          {playing ? "⏸ Pausar" : finished ? "↺ Rever" : "▶ Assistir"}
        </button>
      </div>

      <div className="space-y-3 p-4">
        {hasAllIn && (
          <p
            className={`rounded border px-3 py-1.5 text-center text-xs font-bold ${
              allInPaid
                ? "border-red-400/60 bg-red-950/50 text-red-200"
                : "border-felt-600 bg-felt-900/60 text-felt-300"
            }`}
            aria-live="polite"
          >
            {step === allInIdx
              ? "🔴 ALL-IN PAGO AQUI — a equity trava neste instante"
              : allInPaid
                ? "🔴 All-in já pago — equity travada, só o baralho decide"
                : "All-in ainda não aconteceu — preste atenção nas cartas"}
          </p>
        )}

        <div key={`${hand.id}-phase-${step}`} className="zt-street-banner" aria-live="polite">
          {phase}
        </div>

        <audio ref={audioRef} preload="none" />
        <div className="zt-felt-table zt-demo-table" aria-hidden={false}>
          <div className="absolute left-1/2 top-[46%] z-[1] flex w-[58%] -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2">
            <div className="rounded border border-gold/40 bg-black/40 px-3 py-1 text-center">
              <div className="text-[10px] uppercase tracking-wider text-gold-soft">Pote</div>
              <div className="font-mono text-base font-bold text-white">
                {formatBrlFromCents(showWin && split ? hand.potCents - split.cashbackCents : hand.potCents)}
              </div>
            </div>
            <div className="flex min-h-[52px] flex-wrap items-center justify-center gap-1.5">
              {street.board.length > 0 ? (
                street.board.map((c, i) => {
                  const isWin = showWin && (!useWinFive || winBoard.has(c));
                  const dim = showWin && useWinFive && !winBoard.has(c);
                  const isNew = i >= prevLen;
                  return (
                    <span
                      key={c}
                      className={
                        isWin
                          ? "zt-win-pop zt-win-card"
                          : dim
                            ? "zt-dim"
                            : isNew
                              ? "zt-deal"
                              : undefined
                      }
                      style={isNew && !showWin ? { animationDelay: `${(i - prevLen) * 350}ms` } : undefined}
                    >
                      <PlayingCard code={c} size="sm" highlight={isWin} />
                    </span>
                  );
                })
              ) : (
                <span className="text-xs italic text-felt-400">sem cartas comunitárias ainda</span>
              )}
            </div>
          </div>

          {seats.map((s, i) => {
            const pos = SEAT_POS[Math.min(i, SEAT_POS.length - 1)];
            const isWinnerSeat = showWin && i === winnerIdx;
            const shown = street.holes?.[i] ?? s.cards ?? [];
            const many = shown.length > 2;
            return (
              <div
                key={s.name}
                className={`zt-seat ${many ? "wide" : ""}`}
                style={{ top: `${pos.top}%`, left: `${pos.left}%` }}
              >
                <div className={`zt-seat-card ${s.isHero ? "active" : ""} ${isWinnerSeat ? "winner" : ""} ${s.folded ? "folded" : ""}`}>
                  <div className="truncate text-xs font-semibold text-cream">{s.name}</div>
                  {s.stack && (
                    <div className="font-mono text-[10px] text-gold-soft">{s.stack}</div>
                  )}
                  {s.folded ? (
                    <div className={`mt-1 flex justify-center gap-0.5 ${showWin && useWinFive ? "zt-dim" : ""}`}>
                      <PlayingCard faceDown size="sm" />
                      <PlayingCard faceDown size="sm" />
                    </div>
                  ) : (
                    <div className={many ? "zt-hole-grid" : "mt-1 flex justify-center gap-0.5"}>
                      {shown.map((c) => {
                        const inFive = i === winnerIdx && winHole.has(c);
                        const isWin = showWin && (useWinFive ? inFive : i === winnerIdx);
                        const dim = showWin && useWinFive && !inFive;
                        return (
                          <span key={c} className={isWin ? "zt-win-pop zt-win-card" : dim ? "zt-dim" : undefined}>
                            <PlayingCard code={c} size="sm" highlight={isWin} />
                          </span>
                        );
                      })}
                    </div>
                  )}
                  {s.folded && <div className="mt-0.5 text-[10px] text-felt-400">foldou</div>}
                  {isWinnerSeat && <div className="mt-0.5 text-[10px] font-bold text-gold-bright">VENCEDOR</div>}
                </div>
              </div>
            );
          })}

          {showWin && !reducedMotion && (
            <>
              {[0, 1, 2].map((i) => (
                <span
                  key={`chip-${hand.id}-${i}`}
                  className="zt-chip zt-chip-fly"
                  style={{
                    ["--chip-dx" as string]: flight.dx,
                    ["--chip-dy" as string]: flight.dy,
                    animationDelay: `${i * 180}ms`,
                  }}
                >
                  ⛁
                </span>
              ))}
            </>
          )}
        </div>

        <p className="text-xs font-semibold text-gold-bright" aria-live="polite">
          {street.label}
        </p>

        {showWin && (
          <div className="space-y-2">
            <p className="rounded border-l-2 border-gold-bright bg-felt-950/80 p-2.5 text-xs leading-relaxed text-felt-100">
              {hand.result}
            </p>
            {split && (
              <p className="rounded border border-rail/50 bg-felt-950/80 p-2.5 text-xs text-felt-200">
                Pote de {formatBrlFromCents(hand.potCents)}: vencedor leva{" "}
                <span className="font-mono font-bold text-cream">
                  {formatBrlFromCents(hand.potCents - split.cashbackCents)}
                </span>{" "}
                · de volta para você{" "}
                <span className="font-mono font-bold text-emerald-300">
                  {formatBrlFromCents(split.cashbackCents)} ({split.percent}%)
                </span>
                .
              </p>
            )}
          </div>
        )}

        <div className="flex items-center justify-between gap-2">
          <button
            type="button"
            onClick={() => {
              speak.current = false;
              audioRef.current?.pause();
              setPlaying(false);
              setStep((s) => Math.max(s - 1, 0));
            }}
            disabled={step === 0}
            className="zt-btn-secondary !px-3 !py-1 !text-xs"
          >
            ← Anterior
          </button>
          <div className="flex gap-1.5" role="tablist" aria-label="Etapas da mão">
            {hand.streets.map((s, i) => (
              <button
                key={s.label}
                type="button"
                role="tab"
                aria-selected={i === step}
                aria-label={`Etapa ${i + 1}: ${s.label}`}
                onClick={() => {
                  speak.current = false;
                  audioRef.current?.pause();
                  setPlaying(false);
                  setStep(i);
                }}
                className={`h-2 rounded-full transition-all ${
                  i === step ? "w-6 bg-gold-bright" : "w-2 bg-felt-600 hover:bg-felt-400"
                }`}
              />
            ))}
          </div>
          <button
            type="button"
            onClick={() => {
              if (finished) {
                speak.current = true;
                setStep(0);
                setPlaying(true);
                return;
              }
              speak.current = true;
              setPlaying(false);
              setStep((s) => Math.min(s + 1, last));
            }}
            className="zt-btn-secondary !px-3 !py-1 !text-xs"
          >
            {finished ? "↺ Rever" : "Próxima →"}
          </button>
        </div>
      </div>
    </div>
  );
}
