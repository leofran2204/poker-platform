import { Link } from "react-router";
import { isAuthenticated } from "@/lib/auth";

const steps = [
  {
    title: "Compartilhe seu convite",
    body: "Cada conta recebe um link próprio. Quem conclui o cadastro por ele forma o seu 1º nível.",
  },
  {
    title: "A rede termina no 2º nível",
    body: "Quem entra pelo convite do seu indicado forma o 2º nível. Não existe 3º nível no seu painel.",
  },
  {
    title: "Jogar é obrigatório",
    body: "A qualificação depende de atividade real nas mesas. Cadastro parado não gera pontos nem bonificação.",
  },
];

export function RedePage() {
  const authed = isAuthenticated();

  return (
    <div className="space-y-8">
      <section className="zt-panel overflow-hidden">
        <div className="grid gap-6 p-6 sm:p-8 lg:grid-cols-[1.2fr_0.8fr] lg:items-center">
          <div>
            <p className="text-xs font-bold uppercase tracking-[0.2em] text-gold-soft">
              Rede Zero Tilt · exatamente 2 níveis
            </p>
            <h1 className="mt-3 text-3xl font-bold text-gold-bright sm:text-4xl">
              A sala cresce por convite de quem joga.
            </h1>
            <p className="mt-4 max-w-2xl text-sm leading-relaxed text-felt-200 sm:text-base">
              Convide amigos para aprender, combinar horários e manter as mesas vivas. A rede
              reconhece atividade de jogo — nunca a simples criação de contas.
            </p>
            <div className="mt-6 flex flex-wrap gap-3">
              <Link to={authed ? "/estrutura" : "/register"} className="zt-btn-primary">
                {authed ? "Abrir Minha Rede" : "Criar conta grátis"}
              </Link>
              <Link to="/termos" className="zt-btn-secondary">
                Ler regras completas
              </Link>
            </div>
          </div>

          <div className="rounded border border-gold/50 bg-felt-950/70 p-5">
            <div className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-4 text-sm">
              <span className="zt-chip zt-chip-accent">1º</span>
              <div>
                <strong className="text-cream">Convite direto · 18%</strong>
                <p className="mt-0.5 text-xs text-felt-300">Sobre o rake individual de quem jogou.</p>
              </div>
              <span className="zt-chip zt-chip-accent">2º</span>
              <div>
                <strong className="text-cream">Convite do seu indicado · 12%</strong>
                <p className="mt-0.5 text-xs text-felt-300">O painel encerra aqui. Sem árvore infinita.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section>
        <h2 className="text-xl font-bold text-gold-bright">Como funciona</h2>
        <div className="mt-3 grid gap-3 md:grid-cols-3">
          {steps.map((step, index) => (
            <article key={step.title} className="zt-card p-4">
              <span className="font-mono text-xs text-gold-soft">0{index + 1}</span>
              <h3 className="mt-2 font-bold text-cream">{step.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-felt-200">{step.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <div className="zt-panel p-5">
          <h2 className="text-base font-bold text-gold-bright">O que você acompanha</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-felt-200">
            <li>Seu progresso semanal de mãos e rake pessoal.</li>
            <li>Apuração separada do 1º e do 2º nível.</li>
            <li>Pontos creditados e valores retidos quando falta qualificação.</li>
            <li>Somente apelidos; dados pessoais e bancários não aparecem na rede.</li>
          </ul>
        </div>
        <div className="rounded border-2 border-amber-700/70 bg-amber-950/30 p-5">
          <h2 className="text-base font-bold text-amber-100">Sem promessa de renda</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-amber-50/90">
            <li>Cadastro gratuito, sem kit e sem taxa de ativação.</li>
            <li>Ninguém recebe por cadastrar ou recrutar pessoas.</li>
            <li>Play Money e Jogo Real permanecem em carteiras separadas.</li>
            <li>Os resultados dependem de atividade e das regras vigentes no painel.</li>
          </ul>
        </div>
      </section>

      <section className="rounded border border-felt-600 bg-felt-950/60 p-4 text-sm text-felt-200">
        <strong className="text-cream">Divulgação e jogo responsáveis:</strong> somente para maiores
        de 18 anos. Poker envolve risco de perda e não é investimento. A indicação não autoriza
        compartilhamento de conta, combinação de jogadas ou qualquer forma de conluio. Saiba mais
        em <Link to="/jogo-responsavel" className="font-semibold text-gold-soft underline">Jogo responsável</Link>.
      </section>
    </div>
  );
}
