import { Link } from "react-router-dom";
import homeContent from "@/data/homeContent.json";
import { AnimatedHand, type AnimatedHandData } from "@/components/home/AnimatedHand";
import { DeflatorSimulator } from "@/components/home/DeflatorSimulator";

const { intro, steps, tiers, faq, demoHands } = homeContent.deflator;

const TIER_STYLE: Record<number, string> = {
  35: "border-red-400/50 bg-red-950/40 text-red-200",
  25: "border-orange-400/50 bg-orange-950/40 text-orange-200",
  15: "border-sky-400/50 bg-sky-950/40 text-sky-200",
  7: "border-teal-400/50 bg-teal-950/40 text-teal-200",
};

/** Loss Deflator explicado para quem nunca ouviu falar — com simulador e replays. */
export function LossDeflatorSection() {
  return (
    <section aria-labelledby="home-deflator" className="space-y-4">
      <div>
        <p className="text-xs font-bold uppercase tracking-[0.2em] text-gold-soft">
          Exclusivo Zero Tilt
        </p>
        <h2 id="home-deflator" className="mt-1 text-2xl font-bold text-gold-bright">
          Loss Deflator: o bad beat dói menos aqui
        </h2>
        <p className="mt-2 max-w-3xl text-sm leading-relaxed text-felt-200">{intro}</p>
      </div>

      <ol className="grid gap-3 sm:grid-cols-3">
        {steps.map((s) => (
          <li key={s.title} className="zt-card p-4">
            <p className="text-sm font-bold text-cream">{s.title}</p>
            <p className="mt-1.5 text-xs leading-relaxed text-felt-300">{s.body}</p>
          </li>
        ))}
      </ol>

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="zt-card overflow-hidden">
          <h3 className="border-b border-felt-600 px-4 py-2.5 text-sm font-bold text-cream">
            Quanto maior a sua chance, maior a volta
          </h3>
          <ul className="divide-y divide-felt-600/60">
            {tiers.map((t) => (
              <li key={t.percent} className="flex items-center gap-3 px-4 py-2.5">
                <span
                  className={`shrink-0 rounded border px-2 py-1 font-mono text-sm font-bold ${TIER_STYLE[t.percent] ?? "border-felt-600 text-cream"}`}
                >
                  {t.percent}%
                </span>
                <div className="min-w-0">
                  <p className="text-xs font-semibold text-cream">
                    {t.label} <span className="font-mono font-normal text-felt-400">· {t.range}</span>
                  </p>
                  <p className="truncate text-[11px] text-felt-400">{t.example}</p>
                </div>
              </li>
            ))}
          </ul>
          <p className="border-t border-felt-600 bg-felt-900/50 px-4 py-2 text-[11px] text-felt-400">
            Abaixo de 56% não há devolução — o benefício é para quem jogou na frente.
          </p>
        </div>
        <DeflatorSimulator />
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        {(demoHands as AnimatedHandData[]).map((h) => (
          <AnimatedHand key={h.id} hand={h} />
        ))}
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        {faq.map((f) => (
          <details key={f.q} className="zt-card group p-4">
            <summary className="cursor-pointer text-sm font-semibold text-gold-soft hover:text-gold-bright">
              {f.q}
            </summary>
            <p className="mt-2 text-xs leading-relaxed text-felt-200">{f.a}</p>
          </details>
        ))}
      </div>

      <p className="text-xs text-felt-400">
        Quer dominar o assunto?{" "}
        <Link to="/curso" className="font-semibold text-gold-soft hover:underline">
          Estude bad beats e equidade na Zero Tilt Academy →
        </Link>
      </p>
    </section>
  );
}
