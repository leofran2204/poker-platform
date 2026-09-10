import { lazy, Suspense } from "react";
import { Link } from "react-router-dom";
import { isAuthenticated } from "@/lib/auth";
import { OnlinePresenceHero } from "@/components/OnlinePresence";
import { PokerHistory } from "@/components/PokerHistory";
import { ShowcaseTable } from "@/components/ShowcaseTable";

const NewsTips = lazy(() =>
  import("@/components/NewsTips").then((m) => ({ default: m.NewsTips })),
);

export function HomePage() {
  const authed = isAuthenticated();

  return (
    <div className="w-full">
      <section className="zt-hero">
        <div className="zt-hero-copy">
          <p className="mb-3 text-xs font-bold uppercase tracking-[0.22em] text-gold-soft">
            Sem tilt. Só pôquer.
          </p>
          <h1 className="text-4xl font-bold tracking-tight text-gold-bright sm:text-5xl lg:text-6xl">
            R$ 150 para jogar agora
          </h1>
          <p className="mt-4 max-w-lg text-base leading-relaxed text-cream-muted">
            Play Money no cadastro. Torneio hoje às 21:30 (Brasília). Mesa só começa com
            pelo menos duas pessoas — chame quem joga com a cabeça fria.
          </p>
          <ul className="mt-5 grid gap-2 text-sm text-felt-200 sm:grid-cols-3">
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Play Money</strong>
              R$ 150 cash + R$ 150 torneio, todo dia
            </li>
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">21:30 SP</strong>
              MTT com 5+ jogadores, auto-start
            </li>
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Loss Deflator</strong>
              Bad beat devolve parte do pote
            </li>
          </ul>
          <div className="mt-6 flex flex-wrap items-center gap-4">
            <Link
              to={authed ? "/lobby" : "/register"}
              className="zt-btn-primary px-8 py-3 text-base"
            >
              {authed ? "Entrar no lobby" : "Criar conta e jogar"}
            </Link>
            {!authed && (
              <Link to="/login" className="text-sm font-semibold text-cream hover:text-gold-bright">
                Já tenho conta
              </Link>
            )}
          </div>
          <div className="mt-6 max-w-xl">
            <OnlinePresenceHero />
          </div>
        </div>
        <div className="zt-hero-table">
          <ShowcaseTable />
        </div>
      </section>

      <div className="mx-auto w-full max-w-6xl px-4 py-10">
        <div className="grid gap-4 sm:grid-cols-2">
          <Feature
            title="Loss Deflator"
            body="All-in com a melhor mão (mais de 56%) e mesmo assim perdeu: de 7% a 35% daquele pote volta na hora, sai do próprio pote, não do caixa da casa. O bad beat dói menos e a sessão continua."
          />
          <Feature
            title="Hold’em, Short Deck, Omaha, Pineapple"
            body="Baralho curto de 36 cartas (sem 2 a 5). No Short Deck flush vale mais que full house. Omaha 4 cartas e Ultimate Pineapple (3 cartas, sem descarte) usam 2 hole + 3 board."
          />
        </div>

        <div className="mt-8 grid gap-4 lg:grid-cols-2">
          <PokerHistory variant="world" />
          <PokerHistory variant="brazil" />
        </div>

        <Suspense fallback={<p className="mt-8 text-center text-sm text-felt-400">Carregando notícias…</p>}>
          <NewsTips className="mt-8" />
        </Suspense>
      </div>
    </div>
  );
}

function Feature({ title, body }: { title: string; body: string }) {
  return (
    <div className="zt-panel px-5 py-5">
      <h2 className="text-sm font-bold text-gold-bright">{title}</h2>
      <p className="mt-2 text-sm leading-relaxed text-felt-200">{body}</p>
    </div>
  );
}
