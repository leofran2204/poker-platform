import { Link } from "react-router";
import { isAuthenticated } from "@/lib/auth";

const steps = [
  {
    title: "Chame para a sua mesa",
    body: "Compartilhe seu link pessoal com amigos maiores de 18 anos e combine um horário no Play Money. Quem entra por ele fica no seu 1º nível.",
  },
  {
    title: "Deixe a comunidade crescer",
    body: "Se um amigo também convidar, os jogadores dele formam o seu 2º nível. Seu painel mostra só esses dois níveis, com privacidade.",
  },
  {
    title: "Jogue e veja o progresso",
    body: "Acompanhe mãos, rake pessoal e pontos no painel. Para receber pontos da rede, você precisa jogar 100 mãos na semana; cadastro parado não pontua.",
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
              Rede Zero Tilt · convites entre jogadores
            </p>
            <h1 className="mt-3 text-3xl font-bold text-gold-bright sm:text-4xl">
              Sua próxima mesa pode começar com um amigo.
            </h1>
            <p className="mt-4 max-w-2xl text-sm leading-relaxed text-felt-200 sm:text-base">
              Poker fica mais interessante quando você tem com quem jogar e conversar sobre as
              mãos. Traga sua turma para o Play Money, estudem juntos na Academy e acompanhe a
              atividade da sua comunidade em um painel claro, limitado a dois níveis.
            </p>
            <div className="mt-6 flex flex-wrap gap-3">
              <Link to={authed ? "/estrutura" : "/register"} className="zt-btn-primary">
                {authed ? "Ver minha rede e meu convite" : "Criar conta e chamar a turma"}
              </Link>
              <Link to="/termos" className="zt-btn-secondary">
                Consultar as regras
              </Link>
            </div>
          </div>

          <div className="rounded border border-gold/50 bg-felt-950/70 p-5">
            <div className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-4 text-sm">
              <span className="zt-chip zt-chip-accent">1º</span>
              <div>
                <strong className="text-cream">Seu convite · 18%</strong>
                <p className="mt-0.5 text-xs text-felt-300">Dos pontos de rake Play Money das mãos do seu indicado.</p>
              </div>
              <span className="zt-chip zt-chip-accent">2º</span>
              <div>
                <strong className="text-cream">Convite do seu indicado · 12%</strong>
                <p className="mt-0.5 text-xs text-felt-300">Dos pontos de rake Play Money das mãos desse jogador.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section>
        <h2 className="text-xl font-bold text-gold-bright">Do convite à primeira sessão</h2>
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

      <section className="zt-panel p-5 sm:p-6">
        <p className="text-xs font-bold uppercase tracking-wider text-gold-soft">Um exemplo, sem letra miúda</p>
        <h2 className="mt-1 text-xl font-bold text-cream">Você → Ana → Bruno</h2>
        <p className="mt-2 max-w-3xl text-sm leading-relaxed text-felt-200">
          Ana entra pelo seu link: é seu 1º nível. Bruno entra pelo link da Ana: é seu 2º.
          Quando eles jogam, o painel apura separadamente 18% e 12% do rake individual de
          cada um em pontos Play Money. Para a sua linha pontuar, você também precisa completar
          100 mãos na semana. O convite sozinho não gera pontos.
        </p>
        <Link to={authed ? "/estrutura" : "/register"} className="zt-btn-secondary mt-4 inline-flex">
          {authed ? "Abrir meu painel →" : "Começar pelo Play Money →"}
        </Link>
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
          <h2 className="text-base font-bold text-amber-100">O que esses pontos significam</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-amber-50/90">
            <li>Cadastro gratuito, sem kit e sem taxa de ativação.</li>
            <li>Ninguém recebe por cadastrar ou recrutar pessoas.</li>
            <li>Play Money e Jogo Real permanecem em carteiras separadas.</li>
            <li>Pontos Play Money não são saldo real, não viram PIX e não garantem prêmio.</li>
            <li>A pontuação depende das mãos jogadas e da qualificação semanal.</li>
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
