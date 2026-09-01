import { Link } from "react-router-dom";
import { isAuthenticated } from "@/lib/auth";
import { OnlinePresenceHero } from "@/components/OnlinePresence";
import { NewsTips } from "@/components/NewsTips";
import { PokerHistory } from "@/components/PokerHistory";

export function HomePage() {
  const authed = isAuthenticated();

  return (
    <div className="mx-auto w-full max-w-6xl">
      <OnlinePresenceHero />

      {/* Top: histórias nas laterais preenchem os vazios do login/criar conta */}
      <div className="mt-4 grid gap-4 lg:grid-cols-[360px_1fr_360px] lg:items-start">
        <PokerHistory variant="world" />

        <div className="zt-panel overflow-hidden">
          <div className="border-b-2 border-rail bg-felt-850 px-6 py-8 text-center">
            <p className="mb-2 text-xs font-bold uppercase tracking-[0.2em] text-gold-soft">
              Texas Hold&apos;em · Cash &amp; Torneios
            </p>
            <h1 className="text-4xl font-bold tracking-tight text-gold-bright sm:text-5xl">
              Zero Tilt Poker
            </h1>
            <p className="mx-auto mt-3 max-w-xl text-sm leading-relaxed text-cream-muted">
              Full Tilt clássico, mesa de feltro e jogo limpo. Motor em Rust, interface em TypeScript.
            </p>
            <p className="mx-auto mt-2 max-w-lg text-xs font-semibold text-gold-soft">
              Mínimo 2 pessoas na mesma mesa para iniciar uma mão.
            </p>
            <div className="mt-6 flex flex-wrap justify-center gap-3">
              <Link to={authed ? "/lobby" : "/register"} className="zt-btn-primary px-6 py-2.5 text-sm">
                {authed ? "Entrar no lobby" : "Criar conta e jogar"}
              </Link>
              {!authed && (
                <Link to="/login" className="zt-btn-secondary px-6 py-2.5 text-sm">
                  Já tenho conta
                </Link>
              )}
            </div>
          </div>
          <div className="grid gap-0 sm:grid-cols-3">
            <Feature
              title="Cash 9-max"
              body="Mesas NL com blinds claros, buy-in e seats até 9 jogadores."
            />
            <Feature
              title="Rake honesto"
              body="Centavos inteiros no backend. Split B2B 15/85 para clubes."
            />
            <Feature
              title="Loss Deflator"
              body="Cashback em bad beats extremos — a marca Zero Tilt."
            />
          </div>
        </div>

        <PokerHistory variant="brazil" />
      </div>

      <NewsTips className="mt-6" />

      <p className="mt-6 text-center text-xs text-felt-400">
        Demo / staging · play-money · sem certificação de produção · regulação planejada para 2027
      </p>
    </div>
  );
}

function Feature({ title, body }: { title: string; body: string }) {
  return (
    <div className="border-t border-felt-600 px-5 py-5 sm:border-t-0 sm:border-l sm:first:border-l-0">
      <h2 className="text-sm font-bold text-gold-bright">{title}</h2>
      <p className="mt-2 text-sm leading-relaxed text-felt-200">{body}</p>
    </div>
  );
}
