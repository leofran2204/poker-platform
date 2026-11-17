import { useEffect, useMemo, useRef, useState } from "react";
import { PokerTable } from "@/components/PokerTable";
import { ShowcaseFrame } from "@/components/home/ShowcaseFrame";
import { cashbackFor } from "@/lib/deflator";
import { DEMO_TABLE_CAP, demoSeats, demoTableState, type AnimatedHandData } from "@/lib/demoTable";
import { formatBrlFromCents } from "@/lib/money";

export type { AnimatedHandData, AnimatedSeat, AnimatedStreet, WinningFive } from "@/lib/demoTable";

/** A mesa do jogo é desenhada nesta largura e reduzida por escala (`ShowcaseFrame`). */
const DESIGN_WIDTH = 840;
const DESIGN_HEIGHT = 502;
const noop = () => {};

/** Pausa entre uma batida e a próxima — as streets se acumulam, nada some da tela.
 *  É o ritmo de fallback (sem narração, narração desligada ou áudio indisponível).
 *  Com narração ligada, a próxima etapa aguarda o fim da fala + AUDIO_END_PAUSE_MS. */
const STEP_MS = 3000;
/** Respiro após a fala antes de avançar — dá tempo de olhar a cena. */
const AUDIO_END_PAUSE_MS = 900;

/** Só uma narração por vez na página: trocar de demonstração interrompe a anterior. */
let activeAudio: HTMLAudioElement | null = null;

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

  const seats = useMemo(() => demoSeats(hand), [hand]);
  const winnerIdx = Math.max(
    0,
    seats.findIndex((s) => s.isWinner),
  );

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

  const street = hand.streets[step];
  const finished = step === last;
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const speak = useRef(false);
  const loadedSrc = useRef<string | null>(null);
  const [audioFailed, setAudioFailed] = useState(false);

  useEffect(() => {
    speak.current = false;
    loadedSrc.current = null;
    setAudioFailed(false);
    const el = audioRef.current;
    if (el) {
      el.pause();
      el.removeAttribute("src");
      if (activeAudio === el) activeAudio = null;
    }
  }, [hand.id]);

  useEffect(() => {
    setAudioFailed(false);
  }, [step, hand.id]);

  // Pausa conserva o ponto da explicação: só troca o src quando a etapa mudou.
  // `restart` = recomeçar do zero (troca de etapa, rever); sem ele, continua de onde parou.
  const playStep = (src?: string, restart = true) => {
    const el = audioRef.current;
    if (!el || !src) return;
    if (activeAudio && activeAudio !== el) activeAudio.pause();
    activeAudio = el;
    if (loadedSrc.current !== src) {
      el.src = src;
      loadedSrc.current = src;
      el.currentTime = 0;
    } else if (restart) {
      el.currentTime = 0;
    }
    void el.play().catch(() => undefined);
  };

  const pauseAudio = () => {
    const el = audioRef.current;
    el?.pause();
    if (el && activeAudio === el) activeAudio = null;
  };

  /** Navegação manual: para tudo (som + timers) e posiciona — o visitante retoma quando quiser. */
  const stopForNav = () => {
    speak.current = false;
    if (endTimer.current !== null) window.clearTimeout(endTimer.current);
    pauseAudio();
    setPlaying(false);
  };

  useEffect(() => {
    if (!speak.current) return;
    playStep(street.audio, true);
    // Só quando o passo muda. O botão Assistir/Continuar dispara o áudio do passo atual.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step]);

  // Avanço: dirigido pela narração quando há áudio; temporizado como fallback.
  const audioDriven = speak.current && !!street.audio && !audioFailed;
  useEffect(() => {
    if (!playing || reducedMotion || audioDriven) return;
    if (step >= last) {
      setPlaying(false);
      return;
    }
    const id = window.setTimeout(() => setStep((s) => Math.min(s + 1, last)), STEP_MS);
    return () => window.clearTimeout(id);
  }, [playing, step, last, reducedMotion, audioDriven]);

  // Desmontar (trocar de aba/modalidade) interrompe a narração desta demonstração.
  useEffect(
    () => () => {
      const el = audioRef.current;
      el?.pause();
      if (el && activeAudio === el) activeAudio = null;
    },
    [],
  );

  const endTimer = useRef<number | null>(null);
  useEffect(
    () => () => {
      if (endTimer.current !== null) window.clearTimeout(endTimer.current);
    },
    [],
  );

  const handleAudioEnded = () => {
    if (!playing || !speak.current) return;
    if (step >= last) {
      setPlaying(false);
      speak.current = false;
      return;
    }
    if (endTimer.current !== null) window.clearTimeout(endTimer.current);
    endTimer.current = window.setTimeout(
      () => setStep((s) => Math.min(s + 1, last)),
      AUDIO_END_PAUSE_MS,
    );
  };
  const phase = phaseOf(street.board.length);
  // Board estável como nas plataformas famosas: o que já está na mesa fica
  // parado; só a carta nova anima. prevLen = cartas que já estavam no passo anterior.
  const prevLen = step > 0 ? hand.streets[step - 1].board.length : 0;
  // Suspense do river: o board completa e só depois o showdown abre; a própria
  // mesa acende as cinco cartas do jogo vencedor em seguida.
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

  const table = useMemo(() => demoTableState(hand, step, showWin), [hand, step, showWin]);
  const showdownHands = seats
    .map((seat, index) => ({ name: seat.name, hand: seat.showdownHand, won: index === winnerIdx, folded: seat.folded }))
    .filter((entry) => entry.hand && !entry.folded);

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
              if (!next) {
                if (endTimer.current !== null) window.clearTimeout(endTimer.current);
                pauseAudio();
              } else playStep(street.audio, false);
              return next;
            });
          }}
          className="zt-btn-secondary px-3! py-1! text-xs!"
          aria-label={
            playing
              ? "Pausar replay"
              : finished
                ? "Rever replay"
                : step > 0
                  ? "Continuar replay"
                  : "Reproduzir replay"
          }
        >
          {playing ? "⏸ Pausar" : finished ? "↺ Rever" : step > 0 ? "▶ Continuar" : "▶ Assistir"}
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

        <audio
          ref={audioRef}
          preload="none"
          onEnded={handleAudioEnded}
          onError={() => setAudioFailed(true)}
        />
        <ShowcaseFrame designWidth={DESIGN_WIDTH} designHeight={DESIGN_HEIGHT}>
          <PokerTable
            showcase
            players={table.players}
            communityCards={table.communityCards}
            stage={table.stage}
            pots={table.pots}
            localPlayerId={table.heroId}
            availableActions={[]}
            onAction={noop}
            raiseAmount={0}
            onRaiseChange={noop}
            callAmount={0}
            minimumWager={0}
            maximumWager={0}
            winners={table.winners}
            showdown={table.showdown}
            maxPlayers={DEMO_TABLE_CAP}
            boardStaggerFrom={prevLen}
          />
        </ShowcaseFrame>

        {showWin && showdownHands.length > 0 && (
          <ul
            className="flex flex-wrap justify-center gap-x-4 gap-y-1 text-xs"
            aria-label="Jogo de cada jogador no showdown"
          >
            {showdownHands.map((entry) => (
              <li key={entry.name} className={entry.won ? "font-bold text-gold-bright" : "text-felt-300"}>
                {entry.name}: {entry.hand}
                {entry.won ? " · venceu" : ""}
              </li>
            ))}
          </ul>
        )}

        <p className="text-xs font-semibold text-gold-bright" aria-live="polite">
          {street.label}
        </p>

        {showWin && (
          <div className="space-y-2">
            <p className="rounded-sm border-l-2 border-gold-bright bg-felt-950/80 p-2.5 text-xs leading-relaxed text-felt-100">
              {hand.result}
            </p>
            {split && (
              <p className="rounded-sm border border-rail/50 bg-felt-950/80 p-2.5 text-xs text-felt-200">
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
              stopForNav();
              setStep((s) => Math.max(s - 1, 0));
            }}
            disabled={step === 0}
            className="zt-btn-secondary px-3! py-1! text-xs!"
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
                  stopForNav();
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
              if (endTimer.current !== null) window.clearTimeout(endTimer.current);
              pauseAudio();
              speak.current = true;
              setPlaying(false);
              setStep((s) => Math.min(s + 1, last));
            }}
            className="zt-btn-secondary px-3! py-1! text-xs!"
          >
            {finished ? "↺ Rever" : "Próxima →"}
          </button>
        </div>
      </div>
    </div>
  );
}
