import { useState, useEffect } from "react";
import tipsData from "@/data/tipsContent.json";

interface RSSItem {
  title: string;
  link: string;
  pubDate: string;
  description?: string;
  source: string;
  isLocal?: boolean;
}

interface FeedConfig {
  name: string;
  url: string;
  category: "news" | "tips";
}

const NEWS_FEEDS: FeedConfig[] = [
  { name: "Mundo Poker", url: "https://mundopoker.com.br/feed/", category: "news" },
];

interface LocalNews {
  id: string;
  title: string;
  description: string;
  category: string;
  link?: string;
  pubDate: string;
}

const LOCAL_NEWS: LocalNews[] = [
  {
    id: "n1",
    title: "BSOP Floripa: 6 jogadores classificados via satelite PokerStars",
    description: "Terceiro satelite garantiu buy-in de US$ 109 para o Main Event (R$ 4.000 direto). Bruno Godoy, Joao Gilvani Jr, Mauricio Nalin, Paulo Henrique Monteiro, Robson Mafra e Vitor Balthazar estao garantidos. Total de 45+ brasileiros classificados.",
    category: "BSOP",
    link: "https://mundopoker.com.br/noticias/bsop/seis-jogadores-se-classificam-para-o-bsop-floripa-em-satelite-no-pokerstars/",
    pubDate: "2026-08-19T20:18:05Z"
  },
  {
    id: "n2",
    title: "BSOP e PokerStars dobram garantidos: Mega Satelites classificam 40 para Floripa",
    description: "Parceria ampliou garantidos e mega satelites agora colocam 40 jogadores no Main Event do BSOP Floripa. Buy-in satelite permanece acessivel em US$ 109.",
    category: "BSOP",
    link: "https://mundopoker.com.br/noticias/bsop/bsop-e-pokerstars-dobram-garantidos-e-mega-satelites-classificarao-40-jogadores-para-o-bsop-floripa/",
    pubDate: "2026-08-18T15:30:00Z"
  },
  {
    id: "n3",
    title: "Mundo Poker DOC: Downswing, o lado invisivel do poker -- Serie de 4 artigos",
    description: "Nova serie editorial aborda: 1) O que e downswing e por que entender; 2) Matematica da variancia; 3) Craques brasileiros que revelaram downswings; 4) Psicologa e mental coach sobre impactos emocionais. Lancamentos semanais as quartas.",
    category: "Estrategia/Mental",
    link: "https://mundopoker.com.br/noticias/geral/mundo-poker-doc-downswing-o-lado-invisivel-do-poker/",
    pubDate: "2026-08-15T10:00:00Z"
  },
  {
    id: "n4",
    title: "Ian Simpson (888poker): Como explorar limp do Small Blind",
    description: "Embaixador ensina: BB deve aumentar raises para isolar SB que limpa. Vantagem posicional pos-flop e chave. Dica de randomizacao com segunda carta do naipe do board para balancear range de raise.",
    category: "Estrategia",
    link: "https://pokerlife.com.br/noticias/embaixador-888poker-ian-simpson-como-jogar-contra-limp-small-blind",
    pubDate: "2026-04-25T00:00:00Z"
  },
  {
    id: "n5",
    title: "Aaron Barone e Ian Simpson: Ajustes na bolha e defesa de BB",
    description: "Dois embaixadores 888poker compartilham: na bolha, adapte-se a mesa (pressure conservadores, evite variancia vs loose). No BB, prepare-se para aggression pos-flop: folde medios vs multi-barrels, top pair bluffcatcha.",
    category: "Torneios",
    link: "https://pokerlife.com.br/noticias/embaixadores-888poker-aaron-barone-ian-simpson-compartilham-dicas-bolha-defesa-big-blind",
    pubDate: "2026-02-23T00:00:00Z"
  },
  {
    id: "n6",
    title: "AbacateLeao usa nota, acerta read e leva pote de 200+ blinds",
    description: "Streamer brasileiro registrou fraqueza do vilao em nota, identificou shove de 109.5 BB como blefe, fez hero call com Q-high e ganhou pote gigante. Prova do valor de note-taking consistente.",
    category: "Highlights",
    link: "https://mundopoker.com.br/noticias/geral/abacateleao-usa-nota-para-identificar-rival-faz-leitura-perfeita-e-fatura-pote-gigantesco-com-mais-de-200-blinds/",
    pubDate: "2026-02-26T00:00:00Z"
  },
  {
    id: "n7",
    title: "888poker lanca Run It Twice no cash game online",
    description: "Funcionalidade chega ao online: quando dois players all-in, primeiro a shover escolhe rodar board 1x ou 2x. Reduz variancia dividindo pote em execucoes independentes. Disponivel em todos os stakes.",
    category: "Cash Game",
    link: "https://pokerlife.com.br/noticias/888poker-anuncia-funcionalidade-run-it-twice-cash-games-veja-detalhes",
    pubDate: "2026-04-17T00:00:00Z"
  },
  {
    id: "n8",
    title: "H2school lanca modulo 'Os Melhores do CPH' com Rafael Sahara",
    description: "Campeao do Main Event CPH 2025 analisa 6 maos da mesa final em 7 episodios. Serie transforma decisoes de alta pressao em material didatico. Novos protagonistas a cada edicao (finalistas do CPH).",
    category: "Ensino",
    link: "https://pokerlife.com.br/noticias/h2school-lanca-modulo-melhores-cph-com-profissional-poker-rafael-sahara-analisando-maos-jogadas",
    pubDate: "2025-09-02T00:00:00Z"
  },
  {
    id: "n9",
    title: "H2school lanca aulas semanais gratuitas e ao vivo",
    description: "Escola do H2poker oferece aulas tecnicas, teoricas e reviews ao vivo no YouTube/Twitch. Instrutores: Mario Schiavinnato, Rafael Sahara, Anderson Souza, Vitor Mendes + coaches convidados. Democratiza acesso a educacao de poker.",
    category: "Ensino",
    link: "https://portalrbn.com.br/escola-de-poker-do-h2-lanca-aulas-semanais-gratuitas-e-ao-vivo/",
    pubDate: "2026-06-02T00:00:00Z"
  },
  {
    id: "n10",
    title: "Mundo Poker fara cobertura presencial da WSOP Las Vegas pelo 6o ano",
    description: "Equipe (Augusto Cesar e Guilherme Schiff) cobrira in loco com materias, Instagram, stories ao vivo. Sexto ano consecutivo. Cobertura multimidia do principal evento de poker do mundo.",
    category: "WSOP",
    link: "https://mundopoker.com.br/noticias/wsop/mundo-poker-tera-cobertura-presencial-da-wsop-las-vegas-pelo-sexto-ano-consecutivo-confira-detalhes/",
    pubDate: "2026-05-26T00:00:00Z"
  },
];

interface LocalTip {
  id: string;
  title: string;
  description: string;
  street: "preflop" | "flop" | "turn" | "river";
  category: string;
  link?: string;
}

const LOCAL_TIPS: LocalTip[] = tipsData.tips as LocalTip[];

const CORS_PROXY = "https://api.rss2json.com/v1/api.json?rss_url=";

function parseRSSDate(dateStr: string): Date {
  const date = new Date(dateStr);
  return isNaN(date.getTime()) ? new Date() : date;
}

function stripHtml(html: string): string {
  return html.replace(/<[^>]*>/g, "").replace(/&[^;]+;/g, " ").trim();
}

export function NewsTips({ className }: { className?: string }) {
  const [activeTab, setActiveTab] = useState<"news" | "tips">("news");
  const [activeStreet, setActiveStreet] = useState<"preflop" | "flop" | "turn" | "river">("preflop");
  const [newsItems, setNewsItems] = useState<RSSItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());

  const STREETS: { id: "preflop" | "flop" | "turn" | "river"; label: string; icon: React.ReactNode }[] = [
    { id: "preflop", label: "Pre-flop", icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" /></svg> },
    { id: "flop", label: "Flop", icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z" /></svg> },
    { id: "turn", label: "Turn", icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" /></svg> },
    { id: "river", label: "River", icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" /></svg> },
  ];

  function toggleExpand(itemKey: string) {
    setExpandedItems(prev => {
      const next = new Set(prev);
      if (next.has(itemKey)) next.delete(itemKey);
      else next.add(itemKey);
      return next;
    });
  }

  useEffect(() => {
    if (activeTab !== "news") return;

    let mounted = true;

    async function fetchNews() {
      setLoading(true);
      setError(null);
      const allItems: RSSItem[] = [];

      try {
        for (const feed of NEWS_FEEDS) {
          try {
            const response = await fetch(`${CORS_PROXY}${encodeURIComponent(feed.url)}`);
            if (!response.ok) continue;
            const data = await response.json();
            if (data.items) {
              for (const item of data.items.slice(0, 8)) {
                allItems.push({
                  title: item.title,
                  link: item.link,
                  pubDate: item.pubDate,
                  description: item.description ? stripHtml(item.description) : undefined,
                  source: feed.name,
                });
              }
            }
          } catch {
            continue;
          }
        }

        allItems.sort((a, b) => parseRSSDate(b.pubDate).getTime() - parseRSSDate(a.pubDate).getTime());

        if (mounted) {
          setNewsItems(allItems.slice(0, 15));
        }
      } catch (err) {
        if (mounted) setError("Falha ao carregar noticias. Tente novamente mais tarde.");
      } finally {
        if (mounted) setLoading(false);
      }
    }

    fetchNews();
    const interval = setInterval(fetchNews, 10 * 60 * 1000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [activeTab]);

  const allTips = LOCAL_TIPS.map((tip, idx) => ({
    id: tip.id,
    title: tip.title,
    link: tip.link || "",
    pubDate: new Date(Date.now() - idx * 86400000).toISOString(),
    description: tip.description,
    source: tip.category,
    street: tip.street,
    isLocal: true,
  }));

  const tipsByStreet = allTips.filter(tip => tip.street === activeStreet);

  const items = activeTab === "news"
    ? [
        ...newsItems.map(item => ({ ...item, id: item.link || item.title })),
        ...LOCAL_NEWS.map(news => ({
          id: news.id,
          title: news.title,
          link: news.link || "",
          pubDate: news.pubDate,
          description: news.description,
          source: news.category,
          isLocal: true,
        })),
      ].sort((a, b) => parseRSSDate(b.pubDate).getTime() - parseRSSDate(a.pubDate).getTime()).slice(0, 15)
    : tipsByStreet.map(tip => ({ ...tip, id: tip.id }));

  function formatDate(dateStr: string): string {
    const date = parseRSSDate(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffHours / 24);

    if (diffHours < 1) return "agora mesmo";
    if (diffHours < 24) return `ha ${diffHours}h`;
    if (diffDays < 7) return `ha ${diffDays}d`;
    return date.toLocaleDateString("pt-BR", { day: "2-digit", month: "short" });
  }

  return (
    <div className={`zt-panel ${className ?? ""}`}>
      <div className="border-b border-felt-600">
        <nav className="flex gap-1 p-1" role="tablist" aria-label="Categorias de conteudo">
          <button
            role="tab"
            aria-selected={activeTab === "news"}
            onClick={() => setActiveTab("news")}
            className={`zt-tab ${activeTab === "news" ? "zt-tab-active" : ""}`}
          >
            <span className="flex items-center gap-1.5">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
              </svg>
              Noticias
            </span>
          </button>
          <button
            role="tab"
            aria-selected={activeTab === "tips"}
            onClick={() => setActiveTab("tips")}
            className={`zt-tab ${activeTab === "tips" ? "zt-tab-active" : ""}`}
          >
            <span className="flex items-center gap-1.5">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.734-.988-2.386l-.548-.547z" />
              </svg>
              Jogando melhor
            </span>
          </button>
        </nav>
      </div>

      {activeTab === "tips" && (
        <div className="border-b border-felt-600 px-4">
          <nav className="flex gap-1 py-2 overflow-x-auto" role="tablist" aria-label="Streets de poker">
            {STREETS.map(street => (
              <button
                key={street.id}
                role="tab"
                aria-selected={activeStreet === street.id}
                onClick={() => setActiveStreet(street.id)}
                className={`zt-tab whitespace-nowrap ${activeStreet === street.id ? "zt-tab-active" : ""}`}
              >
                <span className="flex items-center gap-1.5">
                  {street.icon}
                  {street.label}
                </span>
              </button>
            ))}
          </nav>
        </div>
      )}

      <div className="p-4">
        {activeTab === "news" && loading && (
          <div className="flex items-center justify-center py-8">
            <div className="zt-spinner" />
            <span className="ml-3 text-felt-300">Carregando noticias...</span>
          </div>
        )}

        {activeTab === "news" && error && (
          <div className="text-center py-8 text-felt-300">
            <p className="text-red-400 mb-2">{error}</p>
            <button
              onClick={() => window.location.reload()}
              className="zt-btn-ghost text-sm"
            >
              Tentar novamente
            </button>
          </div>
        )}

        {!(activeTab === "news" && loading) && items.length === 0 && (
          <p className="text-center text-felt-400 py-8">Nenhum item encontrado.</p>
        )}

        <div className="space-y-3" role="feed" aria-label={`${activeTab === "news" ? "Noticias" : "Jogando melhor"} de poker`}>
          {!(activeTab === "news" && loading) && items.map(item => {
            const itemKey = item.id as string;
            const isExpanded = expandedItems.has(itemKey);
            return (
              <article
                key={itemKey}
                className="group zt-card p-4 hover:border-gold-soft/30 transition-colors"
              >
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-felt-700 flex items-center justify-center">
                    {activeTab === "news" ? (
                      <svg className="w-5 h-5 text-gold-soft" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
                      </svg>
                    ) : (
                      <svg className="w-5 h-5 text-gold-soft" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.734-.988-2.386l-.548-.547z" />
                      </svg>
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 text-xs text-felt-400 mb-1">
                      <span className="zt-chip text-[10px]">{item.source}</span>
                      {!item.isLocal && <time dateTime={item.pubDate}>{formatDate(item.pubDate)}</time>}
                    </div>
                    <button
                      onClick={() => toggleExpand(itemKey)}
                      className="w-full text-left flex items-center justify-between gap-2 text-cream font-medium leading-snug group-hover:text-gold-bright transition-colors focus:outline-none focus:ring-2 focus:ring-gold-bright focus:ring-offset-2 focus:ring-offset-felt-700 rounded"
                    >
                      <h3 className="text-base">{item.title}</h3>
                      <svg
                        className={`w-4 h-4 text-felt-400 flex-shrink-0 transition-transform ${isExpanded ? "rotate-180" : ""}`}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                      >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                      </svg>
                    </button>
                    {item.description && (
                      <div className="mt-2 text-sm text-felt-300 whitespace-pre-line overflow-hidden transition-all duration-300">
                        {isExpanded ? (
                          <span>{item.description}</span>
                        ) : (
                          <span className="line-clamp-3">{item.description}</span>
                        )}
                      </div>
                    )}
                    {item.link && item.link !== "#" && (
                      <a
                        href={item.link}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="mt-3 inline-flex items-center gap-1 text-xs text-gold-soft hover:text-gold-bright transition-colors"
                      >
                        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                        Ler na fonte
                      </a>
                    )}
                  </div>
                </div>
              </article>
            );
          })}
        </div>
      </div>
    </div>
  );
}