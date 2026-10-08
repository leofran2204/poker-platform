import { lazy, Suspense, useState } from "react";
import { Navigate, Link } from "react-router";
import { isAuthenticated } from "@/lib/auth";
import { PlayingCard } from "@/components/PlayingCard";
import homeFilm from "@/data/homeFilm.json";
import pineappleFilm from "@/data/pineappleFilm.json";
import { PineappleFilm } from "@/components/PineappleFilm";
import { formatLessonDuration } from "@/lib/courseDuration";
import "./home-premium.css";

const Games = lazy(() => import("@/components/home/GamesSection").then(m => ({default: m.GamesSection})));
const Deflator = lazy(() => import("@/components/home/LossDeflatorSection").then(m => ({default: m.LossDeflatorSection})));

function DecisionChallenge() {
  const [answer, setAnswer] = useState<number | null>(null);
  return <section className="hp-section" id="desafio" aria-labelledby="decision-title">
    <header className="hp-heading"><p className="hp-eyebrow">01 / Experimente pensar o jogo</p><h2 id="decision-title">Uma mão. <em>Uma boa pergunta.</em></h2><p>A decisão começa antes de clicar em pagar. Experimente um pouco da Academy.</p></header>
    <div className="hp-challenge">
      <div className="hp-table" aria-label="Exemplo fictício de Texas Hold’em no river">
        <div className="hp-table-top"><span>Texas Hold’em</span><span>RIVER · EXEMPLO</span></div>
        <div className="hp-opponent"><PlayingCard faceDown size="sm" /><PlayingCard faceDown size="sm" /><span>Adversário · aposta 50</span></div>
        <div className="hp-pot"><span>POTE APÓS A APOSTA</span><strong>150 <small>fichas</small></strong></div>
        <div className="hp-board">{["Qs","7h","2c","9d","Kh"].map(code => <PlayingCard key={code} code={code} />)}</div>
        <div className="hp-hand"><PlayingCard code="Ah" /><PlayingCard code="Qd" /><span>SUA MÃO</span></div>
        <p className="hp-caption">Cartas ilustrativas. As chances dependem das mãos possíveis do adversário.</p>
      </div>
      <div className="hp-challenge-copy"><span className="hp-label">O PREÇO DO CALL</span><h3>Você precisa pagar 50.<br />Qual é a equidade mínima?</h3><p>Já há 150 fichas no pote, incluindo a aposta adversária. Considere a última decisão da mão, sem rake nem novas apostas.</p>
        <div className="hp-answers" aria-label="Escolha a equidade mínima para o equilíbrio">{[20,25,33].map(value => <button key={value} type="button" aria-pressed={answer===value} onClick={() => setAnswer(value)}>{value}%</button>)}</div>
        <div className="hp-answer" aria-live="polite" aria-atomic="true">{answer===null ? <p>Escolha uma resposta para revelar o raciocínio.</p> : <><strong>{answer===25 ? "Isso: 25% para empatar em valor esperado." : "O ponto de equilíbrio é 25%. Veja por quê."}</strong><p>50 ÷ (150 + 50) = 25%. Seu call forma um pote final de 200. Acima desse limiar, o call tem EV positivo neste modelo. Isso não revela a sua equidade real nem garante o resultado da mão.</p></>}</div>
        <Link to="/curso/m0l4" className="hp-link">Começar pelos fundamentos na Academy ↗</Link><p className="hp-source">Conceito: <a href="https://ocw.mit.edu/courses/15-s50-poker-theory-and-analytics-january-iap-2015/resources/mit15_s50iap15_l3_basic/" target="_blank" rel="noreferrer">MIT · Poker Theory and Analytics, aula 3</a>. Exemplo original ZT.</p>
      </div>
    </div>
  </section>;
}

function VisitorHome() {
  const [gamesOpen,setGamesOpen]=useState(false);
  const [deflatorOpen,setDeflatorOpen]=useState(false);
  return <div className="hp">
    <section className="hp-hero hp-pineapple" aria-labelledby="home-title">
      <div className="hp-hero-inner"><div className="hp-hero-copy"><p className="hp-eyebrow"><span aria-hidden="true">♠</span> ZERO TILT POKER / COMECE PELAS REGRAS</p><h1 id="home-title">Brazilian<br /><em>Pineapple.</em></h1><p className="hp-description">Sua mão cresce. Sua decisão muda.<br />Aprenda as cartas, as apostas e os potes com uma mesa animada, exemplos e narração em português.</p>
        <div className="hp-actions"><PineappleFilm /><Link to="/curso" className="hp-link">Explorar a Academy ↗</Link></div><p className="hp-note">Vídeo de {formatLessonDuration(pineappleFilm.durationSeconds)} · legendas e capítulos · 18+</p></div>
        <figure className="hp-pineapple-cover"><img src={`/videos/${pineappleFilm.filename}.webp`} width="1280" height="720" alt="Cinco cartas privadas; duas destacadas para combinar com três comunitárias" fetchPriority="high" /><figcaption>2 → 3 → 4 → 5 privadas. Sem descarte.<br /><span>Pratique com Play Money, sem depósito para começar.</span></figcaption></figure><a className="hp-scroll" href="#desafio">A próxima decisão é sua <span aria-hidden="true">↓</span></a>
      </div>
    </section>
    <div className="hp-path" aria-label="Como começar">{[["01","Descubra o jogo","História, regras e possibilidades."],["02","Entenda a decisão","Aulas, exemplos e perguntas."],["03","Pratique no seu ritmo","Mesas de Play Money gratuitas."]].map(([n,title,body])=><div key={n}><span>{n}</span><p><strong>{title}</strong>{body}</p></div>)}</div>
    <DecisionChallenge />
    <section className="hp-section hp-academy" aria-labelledby="academy-title"><div><p className="hp-eyebrow">02 / Zero Tilt Academy</p><h2 id="academy-title">Muito além<br />da próxima <em>carta.</em></h2><p className="hp-body">De um baralho do século XV à matemática de uma aposta. Explore o jogo com contexto, exemplos visuais e referências que você pode consultar.</p><Link to="/curso" className="hp-link">Explorar a Academy ↗</Link><div className="hp-topics"><span>História</span><span>Estratégia</span><span>Matemática</span><span>Autocontrole</span></div></div>
      <Link to="/curso/m0l1" className="hp-lesson"><div className="hp-lesson-image"><img src="/videos/ep11-cartas-historia-v2.webp" width="1280" height="720" loading="lazy" alt="As cartas antes do poker — nova aula de história" /><span aria-hidden="true">↗</span></div><div className="hp-lesson-info"><span className="hp-label">HISTÓRIA / AULA 01</span><h3>As cartas antes do poker</h3><p>Um objeto de museu. Muitas perguntas.<br />Veja o que a história realmente documenta.</p><span className="hp-link">Assistir à aula →</span></div></Link>
    </section>
    <section className="hp-section hp-games" id="modalidades" aria-labelledby="games-title"><header className="hp-heading"><p className="hp-eyebrow">03 / Encontre a sua mesa</p><h2 id="games-title">Quatro jogos.<br className="hp-mobile-break" /> <em>Novas maneiras de pensar.</em></h2><p>Uma conta, diferentes desafios. Conheça as regras antes de entrar.</p></header>
      <div className="hp-variants">{[
        ["Brazilian Pineapple","Exclusivo ZT Poker","♥","Comece com duas. Receba mais uma após flop, turn e river, sem descartar. Use duas privadas e três da mesa.","2 → 3 → 4 → 5 PRIVADAS"],
        ["Short Deck","Outra dinâmica","♦","Do 6 ao ás. Aqui, flush supera full house e trinca supera sequência.","36 CARTAS"],
        ["Omaha 4","Mais combinações","♣","Quatro cartas na mão. Use exatamente duas delas e três comunitárias.","2 + 3 NO SHOWDOWN"],
        ["Texas Hold’em","O clássico","♠","Duas cartas na mão, cinco na mesa. Forme a melhor combinação de cinco.","52 CARTAS"],
      ].map(([name,detail,symbol,body,tag],i)=><article className="hp-variant" key={name}><div className="hp-variant-top"><span>0{i+1}</span><span aria-hidden="true">{symbol}</span></div><p className="hp-label">{detail}</p><h3>{name}</h3><p>{body}</p><span className="hp-tag">{tag}</span></article>)}</div>
      <button type="button" className="hp-disclosure" aria-expanded={gamesOpen} aria-controls="home-replays" onClick={()=>setGamesOpen(!gamesOpen)}>{gamesOpen ? "Fechar demonstrações" : "Ver uma mão de cada modalidade"}<span aria-hidden="true">{gamesOpen ? "−" : "+"}</span></button>
      <div id="home-replays" hidden={!gamesOpen}>{gamesOpen && <div className="hp-expanded"><Suspense fallback={<p role="status">Carregando demonstrações…</p>}><Games /></Suspense></div>}</div>
    </section>
    <section className="hp-section hp-control" aria-labelledby="control-title"><div><p className="hp-eyebrow">04 / Clareza para jogar</p><h2 id="control-title">O controle também<br />faz parte do <em>jogo.</em></h2></div><div className="hp-control-list">
      <article><span>01</span><div><h3>Treine com Play Money</h3><p>Fichas virtuais para experimentar. O saldo de treino é separado do saldo de Jogo Real.</p></div></article>
      <article><span>02</span><div><h3>Conheça o Loss Deflator</h3><p>Entenda quando se aplica a redistribuição de parte do pote líquido e acompanhe os exemplos. O mecanismo não elimina perdas.</p><button type="button" className="hp-link" aria-expanded={deflatorOpen} aria-controls="home-deflator-details" onClick={()=>setDeflatorOpen(!deflatorOpen)}>{deflatorOpen ? "Fechar explicação −" : "Ver regras e simulação +"}</button></div></article>
      <article><span>03</span><div><h3>Respeite seu limite</h3><p>Defina tempo e orçamento. Faça uma pausa quando a vontade de recuperar perdas começar a guiar suas decisões.</p><Link to="/jogo-responsavel" className="hp-link">Jogo responsável ↗</Link></div></article>
    </div><div id="home-deflator-details" className="hp-full" hidden={!deflatorOpen}>{deflatorOpen && <div className="hp-expanded"><Suspense fallback={<p role="status">Carregando explicação…</p>}><Deflator /></Suspense></div>}</div></section>
    <section className="hp-section hp-agent" aria-labelledby="agent-title"><div><p className="hp-eyebrow">Agente ZT Poker</p><h2 id="agent-title">Uma comunidade.<br /><em>Um único nível.</em></h2><p className="hp-body">Traga jogadores diretamente, acompanhe sua operação e desenvolva sua comunidade.</p><Link to="/rede" className="hp-link">Conhecer o programa ↗</Link></div><div className="hp-agent-numbers"><div><strong>30<small>%</small></strong><span>do NGR dos jogadores diretos</span></div><span className="hp-agent-arrow" aria-hidden="true">→</span><div><strong>35<small>%</small></strong><span>ao atingir a meta mensal</span></div><p>Sujeito à aprovação, às regras do programa e ao fechamento mensal conciliado. NGR é a receita líquida elegível, não o valor dos depósitos.</p></div></section>
    <section className="hp-section hp-final" aria-labelledby="final-title"><p className="hp-eyebrow">Zero Tilt Poker</p><h2 id="final-title">Sua próxima decisão?<br /><em>Começar a aprender.</em></h2><Link to="/register" className="hp-button">Criar conta grátis <span aria-hidden="true">↗</span></Link><p>18+ · Poker envolve risco de perda. Jogue com responsabilidade.</p></section>
    <p className="hp-archive"><a href={`/videos/${homeFilm.filename}.mp4?n=${homeFilm.narrationHash}`}>Filme institucional da ZT Poker · {formatLessonDuration(homeFilm.durationSeconds)}</a></p>
  </div>;
}
export function HomePage() { return isAuthenticated() ? <Navigate to="/curso" replace /> : <VisitorHome />; }
