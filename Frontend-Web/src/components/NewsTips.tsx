import { useState, useEffect } from "react";
import tipsData from "@/data/tipsContent.json";

interface FeedItem {
  id: string;
  title: string;
  link: string;
  pubDate: string;
  description?: string;
  source: string;
  isLocal?: boolean;
  imageUrl?: string;
  images?: string[];
  street?: "preflop" | "flop" | "turn" | "river";
}

interface FeedConfig {
  name: string;
  url: string;
}

const NEWS_FEEDS: FeedConfig[] = [
  { name: "Mundo Poker", url: "https://mundopoker.com.br/feed/" },
];

interface LocalNews {
  id: string;
  title: string;
  description: string;
  category: string;
  link?: string;
  pubDate: string;
  imageUrl?: string;
  images?: string[];
}

/** Notícias locais com foto relacionada (jogador / evento) para memorização visual. */
const LOCAL_NEWS: LocalNews[] = [
  {
    id: "n0",
    title: "Morre aos 83 anos Pierre Neuville, lenda belga e finalista do Main Event da WSOP em 2015",
    description:
      "O belga Pierre Neuville faleceu aos 83 anos. Conhecido como “The Serial Qualifier” (23 satélites EPT seguidos), foi 7º no Main Event da WSOP 2015 (US$ 1,2 mi) e acumulou mais de US$ 5 mi em torneios ao vivo. Homenagem da comunidade ao recreativo que virou lenda nas mesas.",
    category: "Homenagem",
    link: "https://mundopoker.com.br/noticias/geral/morre-aos-83-anos-pierre-neuville-lenda-belga-e-finalista-do-main-event-da-wsop-em-2015/",
    pubDate: "2026-08-26T21:09:24Z",
    imageUrl: "https://mundopoker.com.br/wp-content/uploads/2026/08/40639387142_70a47aa071_k.webp",
    images: ["https://mundopoker.com.br/wp-content/uploads/2026/08/40639387142_70a47aa071_k.webp"],
  },
  {
    id: "n1",
    title: "BSOP Floripa: 6 jogadores classificados via satelite PokerStars",
    description:
      "Terceiro satelite garantiu buy-in de US$ 109 para o Main Event (R$ 4.000 direto). Bruno Godoy, Joao Gilvani Jr, Mauricio Nalin, Paulo Henrique Monteiro, Robson Mafra e Vitor Balthazar estao garantidos. Total de 45+ brasileiros classificados.",
    category: "BSOP",
    link: "https://mundopoker.com.br/noticias/bsop/seis-jogadores-se-classificam-para-o-bsop-floripa-em-satelite-no-pokerstars/",
    pubDate: "2026-08-19T20:18:05Z",
    imageUrl: "https://mundopoker.com.br/wp-content/uploads/2026/08/BSOP-Foz-Monti-WCOOP-5094.jpg.webp",
  },
  {
    id: "n2",
    title: "BSOP e PokerStars dobram garantidos: Mega Satelites classificam 40 para Floripa",
    description:
      "Parceria ampliou garantidos e mega satelites agora colocam 40 jogadores no Main Event do BSOP Floripa. Buy-in satelite permanece acessivel em US$ 109.",
    category: "BSOP",
    link: "https://mundopoker.com.br/noticias/bsop/bsop-e-pokerstars-dobram-garantidos-e-mega-satelites-classificarao-40-jogadores-para-o-bsop-floripa/",
    pubDate: "2026-08-18T15:30:00Z",
    imageUrl: "https://mundopoker.com.br/wp-content/uploads/2026/08/BSOP-Foz-Monti-WCOOP-5094.jpg.webp",
  },
  {
    id: "n4",
    title: "Ian Simpson (888poker): Como explorar limp do Small Blind",
    description:
      "Embaixador ensina: BB deve aumentar raises para isolar SB que limpa. Vantagem posicional pos-flop e chave. Dica de randomizacao com segunda carta do naipe do board para balancear range de raise.",
    category: "Estrategia",
    link: "https://pokerlife.com.br/noticias/embaixador-888poker-ian-simpson-como-jogar-contra-limp-small-blind",
    pubDate: "2026-04-25T00:00:00Z",
    imageUrl: "https://u.pokerlife.com.br/media/custom/8306/customimage1.jpg?1777129277000",
  },
  {
    id: "n6",
    title: "AbacateLeao usa nota, acerta read e leva pote de 200+ blinds",
    description:
      "Streamer brasileiro registrou fraqueza do vilao em nota, identificou shove de 109.5 BB como blefe, fez hero call com Q-high e ganhou pote gigante. Prova do valor de note-taking consistente.",
    category: "Highlights",
    link: "https://mundopoker.com.br/noticias/geral/abacateleao-usa-nota-para-identificar-rival-faz-leitura-perfeita-e-fatura-pote-gigantesco-com-mais-de-200-blinds/",
    pubDate: "2026-02-26T00:00:00Z",
    imageUrl: "https://mundopoker.com.br/wp-content/uploads/2024/09/IMG_1678-scaled.webp",
  },
  {
    id: "n10",
    title: "Mundo Poker fara cobertura presencial da WSOP Las Vegas pelo 6o ano",
    description:
      "Equipe (Augusto Cesar e Guilherme Schiff) cobrira in loco com materias, Instagram, stories ao vivo. Sexto ano consecutivo. Cobertura multimidia do principal evento de poker do mundo.",
    category: "WSOP",
    link: "https://mundopoker.com.br/noticias/wsop/mundo-poker-tera-cobertura-presencial-da-wsop-las-vegas-pelo-sexto-ano-consecutivo-confira-detalhes/",
    pubDate: "2026-05-26T00:00:00Z",
    imageUrl: "https://mundopoker.com.br/wp-content/uploads/2025/07/WSOP-Branding-B-RollHH231052-42-scaled.webp",
  },
];

interface LocalTip {
  id: string;
  title: string;
  description: string;
  street: "preflop" | "flop" | "turn" | "river";
  category: string;
  link?: string;
  imageUrl?: string;
}

const LOCAL_TIPS: LocalTip[] = tipsData.tips as LocalTip[];

const CORS_PROXY = "https://api.rss2json.com/v1/api.json?rss_url=";
const PAGE_PROXY = "https://api.allorigins.win/raw?url=";

const TIP_STREET_IMAGES: Record<LocalTip["street"], string> = {
  preflop:
    "https://images.unsplash.com/photo-1511193311914-0346f16efe90?auto=format&fit=crop&w=900&q=80",
  flop: "https://images.unsplash.com/photo-1606167668584-78701c57f13d?auto=format&fit=crop&w=900&q=80",
  turn: "https://images.unsplash.com/photo-1541278107931-e006f21b8e67?auto=format&fit=crop&w=900&q=80",
  river: "https://images.unsplash.com/photo-1596838132731-3301c3fd4311?auto=format&fit=crop&w=900&q=80",
};

function parseRSSDate(dateStr: string): Date {
  const date = new Date(dateStr);
  return isNaN(date.getTime()) ? new Date() : date;
}

function stripHtml(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, " ")
    .replace(/<style[\s\S]*?<\/style>/gi, " ")
    .replace(/<[^>]*>/g, " ")
    .replace(/&nbsp;/g, " ")
    .replace(/&[^;]+;/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function isAdOrUselessImage(url: string): boolean {
  const u = url.toLowerCase();
  return (
    u.includes("820x100") ||
    u.includes("banner") ||
    u.includes("/ads/") ||
    u.includes("emoji") ||
    u.includes("favicon") ||
    u.includes("cropped-512") ||
    u.includes("wp-content/uploads/2026/05/820x100") ||
    u.endsWith(".svg")
  );
}

function extractImagesFromHtml(html: string): string[] {
  const found: string[] = [];
  const srcRe = /(?:src|data-src)=["'](https?:\/\/[^"']+\.(?:jpg|jpeg|png|webp)[^"']*)["']/gi;
  const srcsetRe = /srcset=["']([^"']+)["']/gi;
  let m: RegExpExecArray | null;
  while ((m = srcRe.exec(html)) !== null) {
    found.push(m[1]);
  }
  while ((m = srcsetRe.exec(html)) !== null) {
    const first = m[1].split(",")[0]?.trim().split(/\s+/)[0];
    if (first?.startsWith("http")) found.push(first);
  }
  const unique: string[] = [];
  for (const url of found) {
    if (isAdOrUselessImage(url)) continue;
    if (!unique.includes(url)) unique.push(url);
  }
  return unique;
}

async function resolveOgImage(articleUrl: string): Promise<string | undefined> {
  try {
    const response = await fetch(`${PAGE_PROXY}${encodeURIComponent(articleUrl)}`, {
      signal: AbortSignal.timeout(8000),
    });
    if (!response.ok) return undefined;
    const html = await response.text();
    const og =
      html.match(/property=["']og:image["'][^>]*content=["']([^"']+)["']/i)?.[1] ||
      html.match(/content=["']([^"']+)["'][^>]*property=["']og:image["']/i)?.[1];
    if (og && !isAdOrUselessImage(og)) return og;
    const fromBody = extractImagesFromHtml(html);
    return fromBody[0];
  } catch {
    return undefined;
  }
}

function NewsImage({
  url,
  alt,
  large,
}: {
  url: string;
  alt: string;
  large?: boolean;
}) {
  const [failed, setFailed] = useState(false);
  if (failed) return null;
  return (
    <img
      src={url}
      alt={alt}
      loading="lazy"
      onError={() => setFailed(true)}
      className={
        large
          ? "mt-3 max-h-80 w-full rounded-lg border border-felt-600 object-cover object-center"
          : "h-20 w-28 flex-shrink-0 rounded-md border border-felt-600 object-cover object-center"
      }
    />
  );
}

export function NewsTips({ className }: { className?: string }) {
  const [activeTab, setActiveTab] = useState<"news" | "tips">("news");
  const [activeStreet, setActiveStreet] = useState<"preflop" | "flop" | "turn" | "river">("preflop");
  const [newsItems, setNewsItems] = useState<FeedItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());

  const STREETS: { id: "preflop" | "flop" | "turn" | "river"; label: string; icon: React.ReactNode }[] = [
    {
      id: "preflop",
      label: "Pre-flop",
      icon: (
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
        </svg>
      ),
    },
    {
      id: "flop",
      label: "Flop",
      icon: (
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z"
          />
        </svg>
      ),
    },
    {
      id: "turn",
      label: "Turn",
      icon: (
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
      ),
    },
    {
      id: "river",
      label: "River",
      icon: (
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
        </svg>
      ),
    },
  ];

  function toggleExpand(itemKey: string) {
    setExpandedItems((prev) => {
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
      const allItems: FeedItem[] = [];

      try {
        for (const feed of NEWS_FEEDS) {
          try {
            const response = await fetch(`${CORS_PROXY}${encodeURIComponent(feed.url)}`);
            if (!response.ok) continue;
            const data = await response.json();
            if (!data.items) continue;

            for (const item of data.items.slice(0, 10)) {
              const rawBody =
                (typeof item.content === "string" && item.content) ||
                (typeof item.description === "string" && item.description) ||
                "";
              const body = stripHtml(rawBody);
              const fromContent = extractImagesFromHtml(typeof item.content === "string" ? item.content : "");
              const thumb =
                typeof item.thumbnail === "string" && item.thumbnail && !isAdOrUselessImage(item.thumbnail)
                  ? item.thumbnail
                  : undefined;
              const images = [...(thumb ? [thumb] : []), ...fromContent].filter(
                (url, idx, arr) => arr.indexOf(url) === idx,
              );

              allItems.push({
                id: item.link || item.title,
                title: item.title,
                link: item.link,
                pubDate: item.pubDate,
                description: body.length > 0 ? body : undefined,
                source: feed.name,
                imageUrl: images[0],
                images,
              });
            }
          } catch {
            continue;
          }
        }

        // Completa capa faltante (ex.: foto do jogador) via og:image da matéria.
        const needCover = allItems.filter((item) => !item.imageUrl && item.link).slice(0, 8);
        await Promise.all(
          needCover.map(async (item) => {
            const og = await resolveOgImage(item.link);
            if (!og) return;
            item.imageUrl = og;
            item.images = [og, ...(item.images ?? [])].filter((u, i, a) => a.indexOf(u) === i);
          }),
        );

        // Se o thumb era só banner, tenta og:image de verdade.
        const maybeAds = allItems
          .filter((item) => item.imageUrl && isAdOrUselessImage(item.imageUrl) && item.link)
          .slice(0, 6);
        await Promise.all(
          maybeAds.map(async (item) => {
            const og = await resolveOgImage(item.link);
            if (!og) return;
            item.imageUrl = og;
            item.images = [og, ...(item.images ?? []).filter((u) => !isAdOrUselessImage(u))];
          }),
        );

        allItems.sort((a, b) => parseRSSDate(b.pubDate).getTime() - parseRSSDate(a.pubDate).getTime());

        if (mounted) {
          setNewsItems(allItems.slice(0, 15));
        }
      } catch {
        if (mounted) setError("Falha ao carregar noticias. Tente novamente mais tarde.");
      } finally {
        if (mounted) setLoading(false);
      }
    }

    void fetchNews();
    const interval = setInterval(() => void fetchNews(), 10 * 60 * 1000);
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
    isLocal: true as const,
    imageUrl: tip.imageUrl || TIP_STREET_IMAGES[tip.street],
    images: [tip.imageUrl || TIP_STREET_IMAGES[tip.street]],
  }));

  const tipsByStreet = allTips.filter((tip) => tip.street === activeStreet);

  const items: FeedItem[] =
    activeTab === "news"
      ? [
          ...newsItems,
          ...LOCAL_NEWS.map((news) => ({
            id: news.id,
            title: news.title,
            link: news.link || "",
            pubDate: news.pubDate,
            description: news.description,
            source: news.category,
            isLocal: true as const,
            imageUrl: news.imageUrl,
            images: news.images ?? (news.imageUrl ? [news.imageUrl] : []),
          })),
        ]
          .sort((a, b) => parseRSSDate(b.pubDate).getTime() - parseRSSDate(a.pubDate).getTime())
          .slice(0, 18)
      : tipsByStreet.map((tip) => ({ ...tip, id: tip.id }));

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
            type="button"
            role="tab"
            aria-selected={activeTab === "news"}
            onClick={() => setActiveTab("news")}
            className={`zt-tab ${activeTab === "news" ? "zt-tab-active" : ""}`}
          >
            <span className="flex items-center gap-1.5">
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z"
                />
              </svg>
              Noticias
            </span>
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "tips"}
            onClick={() => setActiveTab("tips")}
            className={`zt-tab ${activeTab === "tips" ? "zt-tab-active" : ""}`}
          >
            <span className="flex items-center gap-1.5">
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.734-.988-2.386l-.548-.547z"
                />
              </svg>
              Jogando melhor
            </span>
          </button>
        </nav>
      </div>

      {activeTab === "tips" && (
        <div className="border-b border-felt-600 px-4">
          <nav className="flex gap-1 overflow-x-auto py-2" role="tablist" aria-label="Streets de poker">
            {STREETS.map((street) => (
              <button
                key={street.id}
                type="button"
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
            <span className="ml-3 text-felt-300">Carregando noticias e fotos...</span>
          </div>
        )}

        {activeTab === "news" && error && (
          <div className="py-8 text-center text-felt-300">
            <p className="mb-2 text-red-400">{error}</p>
            <button type="button" onClick={() => window.location.reload()} className="zt-btn-ghost text-sm">
              Tentar novamente
            </button>
          </div>
        )}

        {!(activeTab === "news" && loading) && items.length === 0 && (
          <p className="py-8 text-center text-felt-400">Nenhum item encontrado.</p>
        )}

        <div
          className="space-y-3"
          role="feed"
          aria-label={`${activeTab === "news" ? "Noticias" : "Jogando melhor"} de poker`}
        >
          {!(activeTab === "news" && loading) &&
            items.map((item) => {
              const itemKey = String(item.id);
              const isExpanded = expandedItems.has(itemKey);
              const body = (item.description ?? "").trim();
              const canExpand = body.length > 0;
              const gallery = (item.images?.length ? item.images : item.imageUrl ? [item.imageUrl] : []).filter(
                (u) => !isAdOrUselessImage(u),
              );
              const cover = gallery[0];

              return (
                <article
                  key={itemKey}
                  className={`group zt-card p-4 transition-colors hover:border-gold-soft/30 ${isExpanded ? "border-gold-soft/40" : ""}`}
                >
                  <div className="flex items-start gap-3">
                    {cover ? (
                      <button type="button" className="flex-shrink-0" onClick={() => canExpand && toggleExpand(itemKey)}>
                        <NewsImage url={cover} alt={item.title} />
                      </button>
                    ) : (
                      <div className="flex h-20 w-28 flex-shrink-0 items-center justify-center rounded-md border border-felt-600 bg-felt-700">
                        <svg className="h-6 w-6 text-gold-soft" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                          />
                        </svg>
                      </div>
                    )}

                    <div className="min-w-0 flex-1">
                      <div className="mb-1 flex items-center gap-2 text-xs text-felt-400">
                        <span className="zt-chip text-[10px]">{item.source}</span>
                        {!item.isLocal && <time dateTime={item.pubDate}>{formatDate(item.pubDate)}</time>}
                      </div>
                      <button
                        type="button"
                        aria-expanded={isExpanded}
                        onClick={() => canExpand && toggleExpand(itemKey)}
                        className="flex w-full items-start justify-between gap-2 rounded text-left font-medium leading-snug text-cream transition-colors hover:text-gold-bright focus:outline-none focus:ring-2 focus:ring-gold-bright focus:ring-offset-2 focus:ring-offset-felt-800"
                      >
                        <h3 className="text-base">{item.title}</h3>
                        {canExpand && (
                          <svg
                            className={`mt-1 h-4 w-4 flex-shrink-0 text-felt-400 transition-transform ${isExpanded ? "rotate-180" : ""}`}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                          >
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                          </svg>
                        )}
                      </button>

                      {canExpand && (
                        <>
                          <div
                            className={
                              isExpanded
                                ? "mt-3 text-sm leading-relaxed text-felt-200 whitespace-pre-wrap break-words"
                                : "mt-2 text-sm leading-relaxed text-felt-300"
                            }
                            style={
                              isExpanded
                                ? undefined
                                : {
                                    display: "-webkit-box",
                                    WebkitLineClamp: 3,
                                    WebkitBoxOrient: "vertical" as const,
                                    overflow: "hidden",
                                  }
                            }
                          >
                            {body}
                          </div>

                          {isExpanded && gallery.length > 0 && (
                            <div className="mt-3 grid gap-2 sm:grid-cols-2">
                              {gallery.slice(0, 4).map((url) => (
                                <NewsImage key={url} url={url} alt={`${item.title} — foto`} large />
                              ))}
                            </div>
                          )}

                          <button
                            type="button"
                            onClick={() => toggleExpand(itemKey)}
                            className="mt-2 text-xs font-bold uppercase tracking-wide text-gold-soft hover:text-gold-bright"
                          >
                            {isExpanded ? "Recolher texto" : "Ver texto completo"}
                          </button>
                        </>
                      )}

                      {item.link && item.link !== "#" && (
                        <a
                          href={item.link}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="mt-3 inline-flex items-center gap-1 text-xs text-gold-soft transition-colors hover:text-gold-bright"
                        >
                          <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                            />
                          </svg>
                          Abrir matéria completa na fonte
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
