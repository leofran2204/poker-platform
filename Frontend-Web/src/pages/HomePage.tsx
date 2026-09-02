import { Link } from "react-router-dom";
import { isAuthenticated } from "@/lib/auth";
import { NewsTips } from "@/components/NewsTips";
import { PokerHistory } from "@/components/PokerHistory";

export function HomePage() {
  const authed = isAuthenticated();

  return (
    <div className="mx-auto w-full max-w-6xl">

      {/* Top: histórias nas laterais preenchem os vazios do login/criar conta */}
      <div className="mt-4 grid gap-4 lg:grid-cols-[360px_1fr_360px] lg:items-start">
        <PokerHistory variant="world" />

        <div className="zt-panel overflow-hidden">
          <div className="border-b-2 border-rail bg-felt-850 px-6 py-8 text-center">
            <p className="mb-2 text-xs font-bold uppercase tracking-[0.2em] text-gold-soft">
              Sem tilt. Só pôquer. Só decisão.
            </p>
            <h1 className="text-4xl font-bold tracking-tight text-gold-bright sm:text-5xl">
              Zero Tilt Poker
            </h1>
            <p className="mx-auto mt-3 max-w-xl text-sm leading-relaxed text-cream-muted">
              Jogue com a cabeça fria. Aqui não há truque, só estrutura justa, rake transparente e
              tecnologia em Rust para você focar na próxima decisão. Do primeiro flop ao deep run,
              <span className="font-semibold text-gold-soft"> com Loss Deflator que tira o tilt do bad beat</span>,
              evolua no seu ritmo.
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
          <div className="grid gap-0 sm:grid-cols-2">
            <Feature
              title="Loss Deflator"
              body="Você foi all-in com a melhor mão (mais de 56% de chance de ganhar) e mesmo assim perdeu. Na hora, de 7% a 35% daquele pote volta para você — sai do próprio pote da mão, não do caixa da casa. Quanto mais favorito você era, maior a fatia. O bad beat dói menos e a sessão continua."
            />
            <Feature
              title="Short Deck"
              body="Baralho de 36 cartas (sem 2 a 5). Mais ação, mais all-ins. No Hold'em Short Deck flush vale mais que full house; no Omaha Short Deck são 4 cartas na mão."
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
