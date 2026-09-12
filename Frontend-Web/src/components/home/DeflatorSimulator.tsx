import { useMemo, useState } from "react";
import { cashbackFor } from "@/lib/deflator";
import { formatBrlFromCents, parseBrlToCents } from "@/lib/money";

/** Simulador interativo: equity + pote → tier + quanto volta na hora. */
export function DeflatorSimulator() {
  const [equity, setEquity] = useState(82);
  const [potText, setPotText] = useState("200,00");
  const [potTouched, setPotTouched] = useState(false);

  const potCents = useMemo(() => parseBrlToCents(potText), [potText]);
  const result = useMemo(
    () => cashbackFor(equity, potCents ?? 0),
    [equity, potCents],
  );
  const winnerCents = (potCents ?? 0) - result.cashbackCents;

  return (
    <div className="zt-card p-4 sm:p-5">
      <h3 className="text-sm font-bold text-gold-bright">
        Simulador: quanto voltaria para você?
      </h3>
      <p className="mt-1 text-xs text-felt-300">
        Arraste a sua chance no all-in e digite o pote líquido (já sem rake).
      </p>

      <div className="mt-4 grid gap-4 sm:grid-cols-2">
        <div>
          <label htmlFor="sim-equity" className="zt-label">
            Sua chance no all-in: <span className="font-mono text-gold-bright">{equity}%</span>
          </label>
          <input
            id="sim-equity"
            type="range"
            min={0}
            max={100}
            step={1}
            value={equity}
            onChange={(e) => setEquity(Number(e.target.value))}
            className="w-full accent-[#c9a227]"
          />
          <div className="flex justify-between text-[11px] text-felt-400">
            <span>0% (azarão)</span>
            <span>56% (mínimo)</span>
            <span>100%</span>
          </div>
        </div>
        <div>
          <label htmlFor="sim-pot" className="zt-label">
            Pote líquido (R$)
          </label>
          <input
            id="sim-pot"
            type="text"
            inputMode="decimal"
            value={potText}
            onChange={(e) => {
              setPotText(e.target.value);
              setPotTouched(true);
            }}
            placeholder="200,00"
            className="zt-input font-mono"
          />
          {potTouched && potCents === null && (
            <p className="mt-1 text-xs text-red-300">
              Digite um valor válido, ex. 200,00.
            </p>
          )}
        </div>
      </div>

      <div
        className="mt-4 rounded border border-rail/50 bg-felt-950/80 p-3.5"
        aria-live="polite"
      >
        {result.tier ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-xs uppercase tracking-wider text-felt-300">
                Faixa de {result.percent}% · {result.tier.label}
              </p>
              <p className="mt-1 text-2xl font-bold text-emerald-300">
                {formatBrlFromCents(result.cashbackCents)}{" "}
                <span className="text-xs font-normal text-felt-300">de volta para você</span>
              </p>
            </div>
            <div className="text-right text-xs text-felt-300">
              <p>
                Vencedor fica com{" "}
                <span className="font-mono text-cream">{formatBrlFromCents(winnerCents)}</span>
              </p>
              <p className="mt-0.5">A soma fecha exata — nada é criado.</p>
            </div>
          </div>
        ) : (
          <p className="text-sm text-felt-200">
            Com <span className="font-mono font-bold text-amber-300">{equity}%</span> você estava
            abaixo dos 56% — aqui não volta nada. O Deflator protege quem jogou na frente.
          </p>
        )}
      </div>
    </div>
  );
}
