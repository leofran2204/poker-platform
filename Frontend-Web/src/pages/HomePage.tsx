import { lazy, Suspense } from "react";
import { Link } from "react-router-dom";
import { isAuthenticated } from "@/lib/auth";
import { OnlinePresenceHero } from "@/components/OnlinePresence";
import { GamesSection } from "@/components/home/GamesSection";
import { LossDeflatorSection } from "@/components/home/LossDeflatorSection";
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
            <abbr
              title="Play Money: fichas de treino, não dinheiro real"
              className="ml-0.5 no-underline"
            >
              *
            </abbr>
          </h1>
          <p className="mt-4 max-w-lg text-base leading-relaxed text-cream-muted">
            <span aria-hidden="true">*</span> Play Money no cadastro — fichas de treino, não
            dinheiro real. Mesa cash só começa com pelo menos duas pessoas — chame quem joga
            com a cabeça fria.
          </p>
          <ul className="mt-5 grid gap-2 text-sm text-felt-200 sm:grid-cols-3">
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Play Money</strong>
              R$ 150 cash + R$ 150 torneio, todo dia
            </li>
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Torneios</strong>
              No lobby, com data e hora do evento — o inaugural inclusive
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

      <div className="mx-auto w-full max-w-6xl space-y-12 px-4 py-10">
        <div className="zt-reveal">
          <LossDeflatorSection />
        </div>

        <div className="zt-reveal">
          <GamesSection />
        </div>

        <div className="zt-reveal zt-panel flex flex-col items-start gap-3 p-5 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-base font-bold text-gold-bright">
              Do zero absoluto ao jogo pensante
            </h2>
            <p className="mt-1 max-w-xl text-sm text-felt-200">
              História do poker, posição, ranges e equidade — com vídeos de 2 minutos, quiz
              avaliado pelo motor e progresso salvo na conta.
            </p>
          </div>
          <Link to="/curso" className="zt-btn-primary shrink-0 px-6 py-2.5">
            Abrir a Academy →
          </Link>
        </div>

        <div className="grid gap-4 lg:grid-cols-2">
          <div className="zt-reveal space-y-2">
            <div className="flex items-baseline justify-between px-1">
              <h2 className="text-sm font-bold uppercase tracking-wider text-gold-bright">
                Notícias
              </h2>
              <Link to="/noticias" className="text-xs font-semibold text-gold-soft hover:underline">
                Ver todas →
              </Link>
            </div>
            <Suspense fallback={<p className="text-center text-sm text-felt-400">Carregando…</p>}>
              <NewsTips tab="news" compact previewLimit={2} />
            </Suspense>
          </div>
          <div className="zt-reveal space-y-2">
            <div className="flex items-baseline justify-between px-1">
              <h2 className="text-sm font-bold uppercase tracking-wider text-gold-bright">
                Dica do Pró
              </h2>
              <Link to="/dicas" className="text-xs font-semibold text-gold-soft hover:underline">
                Ver todas →
              </Link>
            </div>
            <Suspense fallback={<p className="text-center text-sm text-felt-400">Carregando…</p>}>
              <NewsTips tab="tips" compact previewLimit={2} />
            </Suspense>
          </div>
        </div>
      </div>
    </div>
  );
}
