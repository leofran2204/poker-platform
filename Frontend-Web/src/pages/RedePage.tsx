import { Link } from "react-router";
import { isAuthenticated } from "@/lib/auth";

const steps = [
  {
    title: "Traga jogadores diretamente",
    body: "Compartilhe seu código com maiores de 18 anos. Somente os jogadores ligados diretamente a você entram na apuração.",
  },
  {
    title: "Cuide da experiência",
    body: "Ajude no primeiro acesso, combine horários e encaminhe dúvidas. O agente não recebe depósitos nem movimenta carteiras.",
  },
  {
    title: "Acompanhe a meta",
    body: "A base é 30% do NGR direto. No mês em que a meta for atingida, a comissão chega a 35%.",
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
              Programa Agente ZT Poker
            </p>
            <h1 className="mt-3 text-3xl font-bold text-gold-bright sm:text-4xl">
              Um nível. Uma regra. Sem rodeios.
            </h1>
            <p className="mt-4 max-w-2xl text-sm leading-relaxed text-felt-200 sm:text-base">
              O Agente ZT cuida dos jogadores que trouxe diretamente. Recebe 30% do NGR dessa
              carteira e mais 5 pontos percentuais no mês em que atingir a meta de desempenho.
            </p>
            <div className="mt-6 flex flex-wrap gap-3">
              <Link to={authed ? "/estrutura" : "/register"} className="zt-btn-primary">
                {authed ? "Ver meu convite" : "Criar conta"}
              </Link>
              <Link to="/termos" className="zt-btn-secondary">
                Consultar as regras
              </Link>
            </div>
          </div>

          <div className="rounded border border-gold/50 bg-felt-950/70 p-5">
            <div className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-4 text-sm">
              <span className="zt-chip zt-chip-accent">30%</span>
              <div>
                <strong className="text-cream">Comissão base</strong>
                <p className="mt-0.5 text-xs text-felt-300">Sobre o NGR dos seus jogadores diretos.</p>
              </div>
              <span className="zt-chip zt-chip-accent">+5</span>
              <div>
                <strong className="text-cream">Desempenho mensal</strong>
                <p className="mt-0.5 text-xs text-felt-300">Atingiu a meta: 35% do NGR no mês.</p>
              </div>
            </div>
            <p className="mt-4 border-t border-felt-800 pt-3 text-xs text-amber-200">
              O agente precisa ser aprovado. A comissão é fechada mensalmente após a conciliação do NGR.
            </p>
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
        <p className="text-xs font-bold uppercase tracking-wider text-gold-soft">Exemplo direto</p>
        <h2 className="mt-1 text-xl font-bold text-cream">R$ 1.000 de NGR no mês</h2>
        <p className="mt-2 max-w-3xl text-sm leading-relaxed text-felt-200">
          O agente recebe R$ 300. Se atingir a meta mensal, recebe R$ 350. A casa preserva
          R$ 700 ou R$ 650 do NGR antes dos demais custos. Depósito, cadastro e prêmio não entram
          nessa conta.
        </p>
        <Link to={authed ? "/estrutura" : "/register"} className="zt-btn-secondary mt-4 inline-flex">
          {authed ? "Abrir meu painel →" : "Começar pelo Play Money →"}
        </Link>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <div className="zt-panel p-5">
          <h2 className="text-base font-bold text-gold-bright">O que você acompanha</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-felt-200">
            <li>Jogadores ligados diretamente ao seu código.</li>
            <li>NGR do mês e deduções da apuração.</li>
            <li>Percentual atual e progresso da meta.</li>
            <li>Somente apelidos; dados pessoais e bancários não aparecem.</li>
          </ul>
        </div>
        <div className="rounded border-2 border-amber-700/70 bg-amber-950/30 p-5">
          <h2 className="text-base font-bold text-amber-100">Regras do programa</h2>
          <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-amber-50/90">
            <li>Cadastro gratuito, sem kit e sem taxa de ativação.</li>
            <li>Ninguém recebe por cadastro, depósito ou recrutamento indireto.</li>
            <li>Play Money e Jogo Real permanecem em carteiras separadas.</li>
            <li>Pontos Play Money não são saldo real, não viram PIX e não garantem prêmio.</li>
            <li>O agente não recebe dinheiro nem movimenta a carteira do jogador.</li>
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
