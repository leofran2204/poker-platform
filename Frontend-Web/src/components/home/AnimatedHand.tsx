import { useEffect, useState } from "react";
import { PlayingCard } from "@/components/PlayingCard";

export interface AnimatedStreet {
  label: string;
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
}

/** Replay animado de uma mão — o "vídeo" didático sem arquivo MP4. */
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
    const id = window.setTimeout(() => setStep((s) => Math.min(s + 1, last)), 2600);
    return () => window.clearTimeout(id);
  }, [playing, step, last, reducedMotion]);

  const street = hand.streets[step];
  const finished = step === last;

  return (
    <div className="zt-card overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-felt-600 bg-felt-900/70 px-4 py-2.5">
        <h3 className="text-sm font-bold text-cream">{hand.title}</h3>
        <button
          type="button"
          onClick={() => {
            if (finished) {
              setStep(0);
              setPlaying(true);
              return;
            }
            setPlaying((p) => !p);
          }}
          className="zt-btn-secondary !px-3 !py-1 !text-xs"
          aria-label={playing ? "Pausar replay" : finished ? "Rever replay" : "Reproduzir replay"}
        >
          {playing ? "⏸ Pausar" : finished ? "↺ Rever" : "▶ Assistir"}
        </button>
      </div>

      <div className="space-y-3 p-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <div className="rounded border border-felt-600 bg-felt-950/70 p-2.5 text-center">
            <p className="text-[11px] font-semibold uppercase tracking-wider text-gold-soft">
              {hand.heroLabel}
            </p>
            <div className="mt-1.5 flex justify-center gap-1.5">
              {hand.heroCards.map((c) => (
                <PlayingCard key={c} code={c} size="sm" highlight={finished} />
              ))}
            </div>
          </div>
          <div className="rounded border border-felt-600 bg-felt-950/70 p-2.5 text-center">
            <p className="text-[11px] font-semibold uppercase tracking-wider text-felt-300">
              {hand.villainLabel}
            </p>
            <div className="mt-1.5 flex justify-center gap-1.5">
              {hand.villainCards.length > 0 ? (
                hand.villainCards.map((c) => (
                  <PlayingCard key={c} code={c} size="sm" />
                ))
              ) : (
                <span className="text-xs italic text-felt-500">cartas comunitárias abaixo</span>
              )}
            </div>
          </div>
        </div>

        <div
          key={`${hand.id}-${step}`}
          className="rounded border border-rail/40 bg-felt-900/60 p-3 text-center"
        >
          <p className="text-xs font-semibold text-gold-bright" aria-live="polite">
            {street.label}
          </p>
          <div className="mt-2 flex min-h-[52px] flex-wrap items-center justify-center gap-1.5">
            {street.board.length > 0 ? (
              street.board.map((c, i) => (
                <span key={c} className="zt-deal" style={{ animationDelay: `${i * 160}ms` }}>
                  <PlayingCard code={c} size="sm" />
                </span>
              ))
            ) : (
              <span className="text-xs italic text-felt-400">sem cartas comunitárias ainda</span>
            )}
          </div>
        </div>

        {finished && (
          <p className="rounded border-l-2 border-gold-bright bg-felt-950/80 p-2.5 text-xs leading-relaxed text-felt-100">
            {hand.result}
          </p>
        )}

        <div className="flex items-center justify-between gap-2">
          <button
            type="button"
            onClick={() => {
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
                setStep(0);
                setPlaying(true);
                return;
              }
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
