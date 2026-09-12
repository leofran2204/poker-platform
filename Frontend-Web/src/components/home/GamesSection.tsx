import { useState } from "react";
import { Link } from "react-router-dom";
import homeContent from "@/data/homeContent.json";
import { AnimatedHand, type AnimatedHandData } from "@/components/home/AnimatedHand";

const variants = homeContent.variants;

/** Explorador de variantes: abas por jogo + replay animado de exemplo. */
export function GamesSection() {
  const [activeId, setActiveId] = useState(variants[0].id);
  const active = variants.find((v) => v.id === activeId) ?? variants[0];

  return (
    <section aria-labelledby="home-games" className="space-y-4">
      <div>
        <p className="text-xs font-bold uppercase tracking-[0.2em] text-gold-soft">
          4 jogos, 1 conta
        </p>
        <h2 id="home-games" className="mt-1 text-2xl font-bold text-gold-bright">
          Qual mesa é a sua?
        </h2>
        <p className="mt-2 max-w-3xl text-sm leading-relaxed text-felt-200">
          Todo jogo usa cartas comunitárias na mesa, mas cada um muda o baralho, a mão inicial e
          até o que vale mais. Escolha uma aba e assista a um exemplo animado.
        </p>
      </div>

      <div className="flex gap-1 overflow-x-auto rounded border border-felt-600 bg-felt-950/60 p-1" role="tablist" aria-label="Variantes de poker">
        {variants.map((v) => (
          <button
            key={v.id}
            type="button"
            role="tab"
            aria-selected={v.id === activeId}
            onClick={() => setActiveId(v.id)}
            className={`zt-tab whitespace-nowrap !flex-none px-4 ${v.id === activeId ? "zt-tab-active" : ""}`}
          >
            {v.name}
          </button>
        ))}
      </div>

      <div className="grid gap-4 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <dl className="zt-card space-y-3 p-4 sm:p-5">
          <div>
            <dt className="zt-label">Baralho</dt>
            <dd className="text-sm font-semibold text-cream">{active.deck}</dd>
          </div>
          <div>
            <dt className="zt-label">Sua mão inicial</dt>
            <dd className="text-sm font-semibold text-cream">{active.hole}</dd>
          </div>
          <div>
            <dt className="zt-label">Como se forma o jogo</dt>
            <dd className="text-sm leading-relaxed text-felt-200">{active.make}</dd>
          </div>
          <div className="rounded border border-gold/30 bg-gold/10 p-3">
            <dt className="text-xs font-bold uppercase tracking-wider text-gold-soft">
              O pulo do gato
            </dt>
            <dd className="mt-1 text-sm leading-relaxed text-felt-100">{active.twist}</dd>
          </div>
          <div>
            <dt className="zt-label">Mesas</dt>
            <dd className="text-sm text-felt-200">{active.cap}</dd>
          </div>
        </dl>

        <AnimatedHand
          key={active.demoHand.id}
          hand={active.demoHand as AnimatedHandData}
        />
      </div>

      <p className="text-xs text-felt-400">
        Catálogo vigente (blinds e frentes) no lobby — e teoria completa em{" "}
        <Link to="/curso" className="font-semibold text-gold-soft hover:underline">
          Zero Tilt Academy →
        </Link>
      </p>
    </section>
  );
}
