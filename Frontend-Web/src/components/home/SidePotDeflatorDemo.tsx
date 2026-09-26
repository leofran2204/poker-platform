import { useEffect, useState } from "react";
import { formatBrlFromCents } from "@/lib/money";

const STEPS = [
  {
    title: "1 · Três jogadores, dois potes",
    detail: "A entra all-in com menos fichas. B e C continuam apostando. Depois do rake, o pote principal tem R$ 300 e o side pot, disputado só por B e C, tem R$ 200.",
  },
  {
    title: "2 · C vence no showdown",
    detail: "A e B perdem. Suponha que os registros de equity nos respectivos all-ins coloquem ambos na faixa de 25%. A só disputou o pote principal; B disputou os dois.",
  },
  {
    title: "3 · Um teto no pote principal",
    detail: "25% de R$ 300 são R$ 75. Esse é o benefício total desse pote: A e B dividem R$ 37,50 cada. Não são R$ 75 para cada perdedor.",
  },
  {
    title: "4 · Side pot e conta final",
    detail: "No side pot, só B recebe 25% de R$ 200: R$ 50. Ao final A recebe R$ 37,50, B recebe R$ 87,50 e C fica com R$ 375. A soma é R$ 500.",
  },
] as const;

const STEP_MS = 4200;

/** Demonstração da liquidação por pote. Equities são premissas didáticas, sem mão inventada. */
export function SidePotDeflatorDemo() {
  const [step, setStep] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [reducedMotion] = useState(
    () => typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches,
  );

  useEffect(() => {
    if (!playing || reducedMotion) return;
    if (step === STEPS.length - 1) {
      setPlaying(false);
      return;
    }
    const timer = window.setTimeout(() => setStep((current) => current + 1), STEP_MS);
    return () => window.clearTimeout(timer);
  }, [playing, step, reducedMotion]);

  const mainShared = step >= 2;
  const settled = step >= 3;

  return (
    <div className="zt-card overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-felt-600 bg-felt-900/70 px-4 py-2.5">
        <h3 className="text-sm font-bold text-cream">Dois bad beats de 25% · pote principal + side pot</h3>
        <button
          type="button"
          onClick={() => {
            if (step === STEPS.length - 1) setStep(0);
            setPlaying((current) => !current || step === STEPS.length - 1);
          }}
          className="zt-btn-secondary !px-3 !py-1 !text-xs"
        >
          {playing ? "⏸ Pausar" : step === STEPS.length - 1 ? "↺ Rever" : "▶ Assistir"}
        </button>
      </div>

      <div className="space-y-4 p-4" aria-live="polite">
        <div className="rounded border border-felt-600 bg-felt-950/70 p-3">
          <p className="text-xs font-bold uppercase tracking-wider text-gold-soft">{STEPS[step].title}</p>
          <p className="mt-1.5 min-h-16 text-xs leading-relaxed text-felt-200">{STEPS[step].detail}</p>
        </div>

        <div className="grid grid-cols-3 gap-2 text-center text-xs">
          <div className="rounded border border-amber-500/40 bg-felt-900/70 p-2">
            <strong className="block text-cream">A · all-in curto</strong>
            <span className="text-felt-300">Só pote principal</span>
            {mainShared && <strong className="mt-1 block font-mono text-emerald-300">+ {formatBrlFromCents(3750)}</strong>}
          </div>
          <div className="rounded border border-amber-500/40 bg-felt-900/70 p-2">
            <strong className="block text-cream">B · all-in maior</strong>
            <span className="text-felt-300">Principal + side pot</span>
            {mainShared && <strong className="mt-1 block font-mono text-emerald-300">+ {formatBrlFromCents(settled ? 8750 : 3750)}</strong>}
          </div>
          <div className="rounded border border-gold/50 bg-felt-900/70 p-2">
            <strong className="block text-cream">C · vencedor</strong>
            <span className="text-felt-300">Ganha os dois potes</span>
            {settled && <strong className="mt-1 block font-mono text-gold-bright">{formatBrlFromCents(37500)}</strong>}
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div className={`rounded border p-3 text-center ${mainShared ? "border-emerald-500/60 bg-emerald-950/30" : "border-felt-600 bg-felt-950/60"}`}>
            <p className="text-[11px] uppercase tracking-wider text-felt-300">Pote principal · A, B e C</p>
            <strong className="font-mono text-base text-cream">{formatBrlFromCents(30000)}</strong>
            {mainShared && <p className="mt-1 text-[11px] text-emerald-200">25% = R$ 75 · A R$ 37,50 + B R$ 37,50</p>}
          </div>
          <div className={`rounded border p-3 text-center ${settled ? "border-emerald-500/60 bg-emerald-950/30" : "border-felt-600 bg-felt-950/60"}`}>
            <p className="text-[11px] uppercase tracking-wider text-felt-300">Side pot · só B e C</p>
            <strong className="font-mono text-base text-cream">{formatBrlFromCents(20000)}</strong>
            {settled && <p className="mt-1 text-[11px] text-emerald-200">25% = R$ 50 · só B recebe</p>}
          </div>
        </div>

        {settled && (
          <p className="rounded border border-gold/40 bg-gold/10 p-2 text-center text-xs font-semibold text-cream">
            R$ 37,50 + R$ 87,50 + R$ 375 = R$ 500 · nenhuma ficha criada
          </p>
        )}

        <div className="flex items-center justify-between gap-2">
          <button type="button" disabled={step === 0} onClick={() => { setPlaying(false); setStep(step - 1); }} className="zt-btn-secondary !px-3 !py-1 !text-xs">← Anterior</button>
          <span className="font-mono text-xs text-felt-300">{step + 1} / {STEPS.length}</span>
          <button type="button" onClick={() => { setPlaying(false); setStep(step === STEPS.length - 1 ? 0 : step + 1); }} className="zt-btn-secondary !px-3 !py-1 !text-xs">{step === STEPS.length - 1 ? "↺ Rever" : "Próxima →"}</button>
        </div>
      </div>
    </div>
  );
}
