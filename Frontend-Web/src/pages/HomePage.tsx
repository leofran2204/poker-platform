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

      {/* Top: preenche os vazios laterais do login/criar conta — 3 colunas no desktop */}
      <div className="mt-4 grid gap-4 lg:grid-cols-[300px_1fr_300px] lg:items-stretch">
        {/* Esquerda: acesso rápido Entrar */}
        <div className="zt-panel flex flex-col p-5">
          <h2 className="text-sm font-bold uppercase tracking-wide text-gold-bright">
            Já tem conta?
          </h2>
          <p className="mt-2 text-sm leading-relaxed text-felt-200">
            Entre para ver o lobby, escolher a mesa e jogar. Leva 10 segundos.
          </p>
          <ul className="mt-3 list-disc space-y-1 pl-5 text-xs leading-relaxed text-felt-300">
            <li>Lobby com filtros por stake e variante</li>
            <li>Mínimo 2 na mesa para iniciar a mão</li>
            <li>Play Money + Jogo Real separados</li>
          </ul>
          <Link to="/login" className="zt-btn-secondary mt-4 w-full justify-center py-2.5">
            Entrar
          </Link>
          <Link to="/verify-email" className="mt-2 text-center text-xs text-felt-400 hover:text-gold-soft hover:underline">
            Verificar e-mail
          </Link>
        </div>

        {/* Centro: hero */}
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

        {/* Direita: acesso rápido Criar conta */}
        <div className="zt-panel flex flex-col p-5">
          <h2 className="text-sm font-bold uppercase tracking-wide text-gold-bright">
            Novo por aqui?
          </h2>
          <p className="mt-2 text-sm leading-relaxed text-felt-200">
            Crie sua conta e ganhe fichas Play Money para treinar sem risco.
          </p>
          <ul className="mt-3 list-disc space-y-1 pl-5 text-xs leading-relaxed text-felt-300">
            <li>R$ 1.000 Play Money por dia</li>
            <li>Verificação por e-mail em 6 dígitos</li>
            <li>Mesas para iniciantes e avançados</li>
          </ul>
          <Link to="/register" className="zt-btn-primary mt-4 w-full justify-center py-2.5">
            Criar conta
          </Link>
          <p className="mt-2 text-center text-xs text-felt-400">Leva 30 segundos</p>
        </div>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-2">
        <PokerHistory variant="world" />
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
