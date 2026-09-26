import { Navigate, Link } from "react-router";
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
            <Link to="/register" className="zt-btn-primary px-8 py-3 text-base">
              Criar conta grátis
            </Link>
            <Link to="/curso" className="text-sm font-semibold text-cream hover:text-gold-bright">
              Conhecer a Academy
            </Link>
            <a href="#demonstracao" className="text-sm font-semibold text-cream hover:text-gold-bright">
              Assistir às demonstrações
            </a>
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

        <section className="zt-reveal zt-panel overflow-hidden p-5 sm:p-7">
          <div>
            <p className="text-xs font-bold uppercase tracking-[0.18em] text-gold-soft">
              Sua mesa começa com um convite
            </p>
            <h2 className="mt-2 max-w-2xl text-2xl font-bold text-gold-bright sm:text-3xl">
              Traga sua turma. Transforme uma partida em uma comunidade.
            </h2>
            <p className="mt-3 max-w-3xl text-sm leading-relaxed text-felt-200">
              Compartilhe seu link, combine uma mesa Play Money e evoluam juntos na Academy.
              Sua rede mostra quem chegou pelo seu convite, a atividade da turma e os pontos
              gerados pelas mãos jogadas. Você acompanha tudo em até dois níveis.
            </p>
            <div className="mt-5 grid gap-3 sm:grid-cols-3">
              <div className="rounded border border-felt-600 bg-felt-950/60 p-3">
                <strong className="text-sm text-cream">01 · Convide</strong>
                <p className="mt-1 text-xs leading-relaxed text-felt-300">Seu link conecta amigos à sua rede e à mesma sala.</p>
              </div>
              <div className="rounded border border-felt-600 bg-felt-950/60 p-3">
                <strong className="text-sm text-cream">02 · Joguem</strong>
                <p className="mt-1 text-xs leading-relaxed text-felt-300">Marquem uma sessão, estudem uma mão e voltem à mesa.</p>
              </div>
              <div className="rounded border border-gold/40 bg-gold/10 p-3">
                <strong className="text-sm text-gold-bright">03 · Acompanhe</strong>
                <p className="mt-1 text-xs leading-relaxed text-felt-200">Veja seus dois níveis, mãos da semana e pontos no painel.</p>
              </div>
            </div>
            <div className="mt-5 flex flex-wrap items-center gap-3">
              <Link to="/rede" className="zt-btn-primary">Descobrir como funciona →</Link>
              <span className="text-xs text-felt-300">Cadastro grátis · pontos só por atividade · 18+</span>
            </div>
            <p className="mt-3 text-xs text-felt-400">Pontos Play Money não são dinheiro. Poker envolve risco de perda e não é investimento.</p>
          </div>
        </section>

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
