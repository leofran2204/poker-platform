import { Navigate, Link } from "react-router-dom";
import { isAuthenticated } from "@/lib/auth";
import { OnlinePresenceHero } from "@/components/OnlinePresence";
import { GamesSection } from "@/components/home/GamesSection";
import { LossDeflatorSection } from "@/components/home/LossDeflatorSection";
import { ShowcaseTable } from "@/components/ShowcaseTable";

export function HomePage() {
  if (isAuthenticated()) {
    return <Navigate to="/curso" replace />;
  }

  return (
    <div className="w-full min-w-0">
      <section className="zt-hero">
        <div className="zt-hero-copy">
          <p className="mb-3 text-xs font-bold uppercase tracking-[0.22em] text-gold-soft">
            ZT Poker · Zero Tilt Academy
          </p>
          <h1 className="text-4xl font-bold tracking-tight text-gold-bright sm:text-5xl lg:text-6xl">
            Estude. Jogue. Sem tilt.
          </h1>
          <p className="mt-4 max-w-lg text-base leading-relaxed text-cream-muted">
            Escola de poker com aulas curtas, quiz e mesa ao vivo. Treine de graça no Play Money.
            Quando quiser, compre fichas e jogue de verdade o que aprendeu.
          </p>
          <ul className="mt-5 grid gap-2 text-sm text-felt-200 sm:grid-cols-3">
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Academy</strong>
              Vídeos, quiz e progresso na conta
            </li>
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Treino grátis</strong>
              Play Money no cadastro, todo dia
            </li>
            <li className="rounded border border-felt-600 bg-felt-950/50 px-3 py-2">
              <strong className="block text-gold-soft">Mesa real</strong>
              Compre fichas quando quiser — saldos separados
            </li>
          </ul>
          <div className="mt-6 flex flex-wrap items-center gap-4">
            <Link to="/curso" className="zt-btn-primary px-8 py-3 text-base">
              Começar a Academy
            </Link>
            <a href="#demonstracao" className="text-sm font-semibold text-cream hover:text-gold-bright">
              Assistir à demonstração narrada
            </a>
            <Link to="/register" className="text-sm font-semibold text-cream hover:text-gold-bright">
              Criar conta
            </Link>
            <Link to="/login" className="text-sm font-semibold text-felt-300 hover:text-gold-bright">
              Já tenho conta
            </Link>
          </div>
          <div className="mt-6 max-w-xl">
            <OnlinePresenceHero />
          </div>
        </div>
        <div className="zt-hero-table">
          <ShowcaseTable />
        </div>
      </section>

      <div className="mx-auto w-full min-w-0 max-w-6xl space-y-12 px-4 py-10">
        <div className="zt-reveal">
          <GamesSection />
        </div>

        <div className="zt-reveal">
          <LossDeflatorSection />
        </div>

        <div className="zt-reveal zt-panel flex flex-col items-start gap-3 p-5 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-base font-bold text-gold-bright">
              Do zero absoluto ao jogo pensante
            </h2>
            <p className="mt-1 max-w-xl text-sm text-felt-200">
              História do poker, posição, ranges e equidade — com vídeos curtos, quiz avaliado pelo
              motor e progresso salvo na conta. Notícias e Dica do Pró ficam na Academy.
            </p>
          </div>
          <Link to="/curso" className="zt-btn-primary shrink-0 px-6 py-2.5">
            Abrir a Academy →
          </Link>
        </div>
      </div>
    </div>
  );
}
