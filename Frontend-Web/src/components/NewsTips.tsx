import { useState, useEffect, useCallback } from "react";
import tipsData from "@/data/tipsContent.json";
import { TipRichText } from "@/components/TipRichText";
import { looksLikePortuguese, translateNewsFields, translateToPortuguese } from "@/lib/translatePt";
import { parseJinaMarkdown, parseRssXml } from "@/lib/parseRss";
import {
  extractImagesFromHtml,
  htmlToPlainText,
  resolveArticleBody,
  resolveSourceImage,
} from "@/lib/articleFetch";

interface FeedItem {
  id: string;
  title: string;
  link: string;
  pubDate: string;
  description?: string;
  source: string;
  isLocal?: boolean;
  /** Somente foto oficial da fonte (og/corpo). Sem fallback. */
  imageUrl?: string;
  imageCaption?: string;
  /** Fonte internacional já traduzida para PT. */
  translated?: boolean;
  /** Precisa traduzir (fonte EN/ES). */
  needsTranslation?: boolean;
  fromLang?: "en" | "es" | "auto";
  street?: "preflop" | "flop" | "turn" | "river";
}

interface FeedConfig {
  name: string;
  url: string;
  /** pt = já em português; en/es = traduzir para PT. */
  lang: "pt" | "en" | "es";
  /** news = notícias; tips = estratégia; both = os dois. */
  kind: "news" | "tips" | "both";
}

/**
 * Fontes RSS.
 * - news → só aba Notícias
 * - tips → só aba Jogando melhor
 * - both → notícia na aba Notícias; tip só se for claramente estratégia
 */
const CONTENT_FEEDS: FeedConfig[] = [
  { name: "Mundo Poker", url: "https://mundopoker.com.br/feed/", lang: "pt", kind: "news" },
  { name: "Poker No Brasil", url: "https://pokernobrasil.com/feed/", lang: "pt", kind: "news" },
  { name: "Lobbyze", url: "https://blog.lobbyze.com/feed/", lang: "pt", kind: "both" },
  { name: "SuperPoker", url: "https://www.superpoker.com.br/feed/", lang: "pt", kind: "news" },
  { name: "SuperPoker Estratégia", url: "https://www.superpoker.com.br/category/estrategia/feed/", lang: "pt", kind: "tips" },
  { name: "PokerNews Brasil", url: "https://br.pokernews.com/rss.php?subject=news", lang: "pt", kind: "news" },
  { name: "Código Poker", url: "https://codigopoker.com.br/feed/", lang: "pt", kind: "news" },
  { name: "Código Poker Latam", url: "https://codigopoker.com/feed/", lang: "es", kind: "news" },
  { name: "PokerNews", url: "https://www.pokernews.com/rss.php?subject=news", lang: "en", kind: "news" },
  { name: "CardPlayer", url: "https://www.cardplayer.com/rss", lang: "en", kind: "news" },
  { name: "CardsChat", url: "https://www.cardschat.com/feed/", lang: "en", kind: "news" },
  { name: "Upswing Poker", url: "https://upswingpoker.com/feed/", lang: "en", kind: "tips" },
  { name: "FlushDraw", url: "https://www.flushdraw.net/feed/", lang: "en", kind: "tips" },
];

const ITEMS_PER_FEED = 5;
const NEWS_FEED_LIMIT = 32;
const TIPS_FEED_LIMIT = 40;

interface LocalNews {
  id: string;
  title: string;
  description: string;
  category: string;
  link?: string;
  pubDate: string;
}

/** Notícias locais — foto só se a matéria original tiver (resolvida no fetch). */
const LOCAL_NEWS: LocalNews[] = [
  {
    id: "n0",
    title: "Morre aos 83 anos Pierre Neuville, lenda belga e finalista do Main Event da WSOP em 2015",
    description:
      "O belga Pierre Neuville faleceu aos 83 anos. Conhecido como “The Serial Qualifier” (23 satélites EPT seguidos), foi 7º no Main Event da WSOP 2015 (US$ 1,2 mi) e acumulou mais de US$ 5 mi em torneios ao vivo. Homenagem da comunidade ao recreativo que virou lenda nas mesas.",
    category: "Homenagem",
    link: "https://mundopoker.com.br/noticias/geral/morre-aos-83-anos-pierre-neuville-lenda-belga-e-finalista-do-main-event-da-wsop-em-2015/",
    pubDate: "2026-08-26T21:09:24Z",
  },
  {
    id: "n1",
    title: "BSOP Floripa: 6 jogadores classificados via satelite PokerStars",
    description:
      "Terceiro satelite garantiu buy-in de US$ 109 para o Main Event (R$ 4.000 direto). Bruno Godoy, Joao Gilvani Jr, Mauricio Nalin, Paulo Henrique Monteiro, Robson Mafra e Vitor Balthazar estao garantidos. Total de 45+ brasileiros classificados.",
    category: "BSOP",
    link: "https://mundopoker.com.br/noticias/bsop/seis-jogadores-se-classificam-para-o-bsop-floripa-em-satelite-no-pokerstars/",
    pubDate: "2026-08-19T20:18:05Z",
  },
  {
    id: "n2",
    title: "BSOP e PokerStars dobram garantidos: Mega Satelites classificam 40 para Floripa",
    description:
      "Parceria ampliou garantidos e mega satelites agora colocam 40 jogadores no Main Event do BSOP Floripa. Buy-in satelite permanece acessivel em US$ 109.",
    category: "BSOP",
    link: "https://mundopoker.com.br/noticias/bsop/bsop-e-pokerstars-dobram-garantidos-e-mega-satelites-classificarao-40-jogadores-para-o-bsop-floripa/",
    pubDate: "2026-08-18T15:30:00Z",
  },
  {
    id: "n4",
    title: "Ian Simpson (888poker): Como explorar limp do Small Blind",
    description:
      "Embaixador ensina: BB deve aumentar raises para isolar SB que limpa. Vantagem posicional pos-flop e chave. Dica de randomizacao com segunda carta do naipe do board para balancear range de raise.",
    category: "Estrategia",
    link: "https://pokerlife.com.br/noticias/embaixador-888poker-ian-simpson-como-jogar-contra-limp-small-blind",
    pubDate: "2026-04-25T00:00:00Z",
  },
  {
    id: "n6",
    title: "AbacateLeao usa nota, acerta read e leva pote de 200+ blinds",
    description:
      "Streamer brasileiro registrou fraqueza do vilao em nota, identificou shove de 109.5 BB como blefe, fez hero call com Q-high e ganhou pote gigante. Prova do valor de note-taking consistente.",
    category: "Highlights",
    link: "https://mundopoker.com.br/noticias/geral/abacateleao-usa-nota-para-identificar-rival-faz-leitura-perfeita-e-fatura-pote-gigantesco-com-mais-de-200-blinds/",
    pubDate: "2026-02-26T00:00:00Z",
  },
  {
    id: "n10",
    title: "Mundo Poker fara cobertura presencial da WSOP Las Vegas pelo 6o ano",
    description:
      "Equipe (Augusto Cesar e Guilherme Schiff) cobrira in loco com materias, Instagram, stories ao vivo. Sexto ano consecutivo. Cobertura multimidia do principal evento de poker do mundo.",
    category: "WSOP",
    link: "https://mundopoker.com.br/noticias/wsop/mundo-poker-tera-cobertura-presencial-da-wsop-las-vegas-pelo-sexto-ano-consecutivo-confira-detalhes/",
    pubDate: "2026-05-26T00:00:00Z",
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

/** Só foto da fonte; rejeita só banner de anúncio óbvio. */
function pickSourceImage(candidates: Array<string | undefined | null>): string | undefined {
  for (const url of candidates) {
    if (url && !isAdBanner(url)) return url;
  }
  return undefined;
}

function parseRSSDate(dateStr: string): Date {
  const date = new Date(dateStr);
  return isNaN(date.getTime()) ? new Date() : date;
}

function isAdBanner(url: string): boolean {
  const u = url.toLowerCase();
  return (
    u.includes("820x100") ||
    u.includes("/ads/") ||
    u.includes("favicon") ||
    u.includes("emoji") ||
    u.includes("wp-content/uploads/2026/05/820x100") ||
    u.endsWith(".svg")
  );
}

function normalizeText(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "");
}

/** Classifica street só depois de confirmar que é conteúdo de estratégia. */
function classifyStreet(title: string, body = ""): FeedItem["street"] | undefined {
  const t = normalizeText(`${title} ${body}`);
  if (
    /pre[\s-]?flop|preflop|open[\s-]?raise|3[\s-]?bet|4[\s-]?bet|\blimp\b|iso[\s-]?raise|range de abertura|versus limp|vs limp|steal blind/.test(
      t,
    )
  ) {
    return "preflop";
  }
  if (/\bflop\b|c[\s-]?bet|continuation bet|donk bet|set mining|float pot/.test(t)) {
    return "flop";
  }
  if (/\bturn\b|double barrel|segundo barri|probe bet|scare card/.test(t)) {
    return "turn";
  }
  if (/\briver\b|bluff[\s-]?catch|value bet|overbet|blocking bet|thin value/.test(t)) {
    return "river";
  }
  return undefined;
}

/** Manchete tipicamente de notícia (resultado/torneio) — NÃO vai para Jogando melhor. */
function looksLikeNewsHeadline(title: string): boolean {
  const t = normalizeText(title);
  return /venceu|vence |campeao|campea|morre|faleceu|classific|lidera|caiu|eliminad|bracelet|premia|inscric|satelite|cobertura|resultado|final table|mesa final|dia \d|main event|high roller|forra|faturou|leva us\$|leva r\$|conquist/.test(
    t,
  );
}

/**
 * Só manda para Jogando melhor se for feed de tips OU (both) claramente estratégico.
 * Feed "news" nunca vaza para a aba de dicas.
 */
function isStrategyContent(title: string, body: string, feedKind: FeedConfig["kind"]): boolean {
  if (feedKind === "tips") return true;
  if (feedKind === "news") return false;

  // both — exige sinal claro de estratégia e rejeita manchete de resultado
  if (looksLikeNewsHeadline(title)) return false;
  const t = normalizeText(`${title} ${body.slice(0, 500)}`);
  return /estrateg|strategy|dica[s]? de|como jogar|hand review|gto|\bicm\b|bankroll|quiz|guia completo|tutorial|aprenda|melhorar seu jogo|vs limp|c-bet|pre-flop|preflop|postflop|ranges? de|spot:|teoria do|conceito/.test(
    t,
  );
}



function NewsImage({
  url,
  alt,
  caption,
  large,
  onClick,
}: {
  url: string;
  alt: string;
  caption?: string;
  large?: boolean;
  onClick?: () => void;
}) {
  const frame = (
    <div className={large ? "zt-news-photo-frame-lg" : "zt-news-photo-frame"}>
      <img
        src={url}
        alt={caption ? `${alt} — ${caption}` : alt}
        loading="lazy"
        decoding="async"
        referrerPolicy="no-referrer"
        className="zt-news-photo"
      />
      {caption && (
        <div className="zt-news-photo-caption" title={caption}>
          <span>{caption}</span>
        </div>
      )}
    </div>
  );
  if (!onClick) return frame;
  return (
    <button type="button" onClick={onClick} className="block w-full text-left">
      {frame}
    </button>
  );
}

export function NewsTips({ className }: { className?: string }) {
  const [activeTab, setActiveTab] = useState<"news" | "tips">("news");
  const [activeStreet, setActiveStreet] = useState<"preflop" | "flop" | "turn" | "river">("preflop");
  const [newsItems, setNewsItems] = useState<FeedItem[]>([]);
  const [remoteTips, setRemoteTips] = useState<FeedItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  /** Texto completo buscado da página da matéria (quando o RSS vem curto/vazio). */
  const [fullBodies, setFullBodies] = useState<Record<string, string>>({});
  const [loadingBodies, setLoadingBodies] = useState<Record<string, boolean>>({});

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

  const loadFullBody = useCallback(async (item: FeedItem) => {
    const key = String(item.id);
    const existing = (item.description ?? "").trim();
    if (existing.length >= 500 || fullBodies[key] || !item.link) return;
    if (loadingBodies[key]) return;
    setLoadingBodies((prev) => ({ ...prev, [key]: true }));
    try {
      let text = await resolveArticleBody(item.link);
      if (!text || text.trim().length <= existing.length) return;
      text = text.trim();
      // Sempre PT-BR na aba de dicas / fontes estrangeiras
      const mustTranslate =
        item.fromLang === "en" ||
        item.fromLang === "es" ||
        item.needsTranslation ||
        !looksLikePortuguese(text);
      if (mustTranslate) {
        text = await translateToPortuguese(text, item.fromLang ?? "auto", true);
      }
      setFullBodies((prev) => ({ ...prev, [key]: text }));
    } finally {
      setLoadingBodies((prev) => ({ ...prev, [key]: false }));
    }
  }, [fullBodies, loadingBodies]);

  function toggleExpand(item: FeedItem) {
    const itemKey = String(item.id);
    setExpandedItems((prev) => {
      const next = new Set(prev);
      if (next.has(itemKey)) {
        next.delete(itemKey);
      } else {
        next.add(itemKey);
        void loadFullBody(item);
      }
      return next;
    });
  }

  useEffect(() => {
    let mounted = true;

    function toFeedItems(
      feed: FeedConfig,
      rawItems: Array<{
        title: string;
        link: string;
        pubDate: string;
        description?: string;
        content?: string;
        thumbnail?: string;
      }>,
    ): FeedItem[] {
      const out: FeedItem[] = [];
      for (const item of rawItems.slice(0, ITEMS_PER_FEED)) {
        const rawBody =
          (typeof item.content === "string" && item.content) ||
          (typeof item.description === "string" && item.description) ||
          "";
        const body = htmlToPlainText(rawBody);
        const base = item.link || feed.url;
        const fromContent = extractImagesFromHtml(
          typeof item.content === "string" ? item.content : rawBody,
          base,
        );
        const thumb =
          typeof item.thumbnail === "string" && item.thumbnail && !isAdBanner(item.thumbnail)
            ? item.thumbnail
            : undefined;
        const early = pickSourceImage([fromContent[0], thumb]);
        const street = classifyStreet(item.title, body);
        out.push({
          id: item.link || `${feed.name}:${item.title}`,
          title: item.title,
          link: item.link,
          pubDate: item.pubDate,
          description: body.length > 0 ? body : undefined,
          source: feed.name,
          imageUrl: early,
          imageCaption: early ? "Foto da matéria" : undefined,
          needsTranslation: feed.lang !== "pt",
          fromLang: feed.lang === "pt" ? undefined : feed.lang === "es" ? "es" : "en",
          street,
        });
      }
      return out;
    }

    async function fetchOneFeed(feed: FeedConfig): Promise<FeedItem[]> {
      // 1) rss2json (caminho principal)
      try {
        const response = await fetch(`${CORS_PROXY}${encodeURIComponent(feed.url)}`);
        if (response.ok) {
          const data = await response.json();
          if (data.items?.length) return toFeedItems(feed, data.items);
        }
      } catch {
        /* tenta fallback */
      }

      // 2) XML cru via allorigins
      try {
        const rawRes = await fetch(`${PAGE_PROXY}${encodeURIComponent(feed.url)}`, {
          signal: AbortSignal.timeout(15000),
        });
        if (rawRes.ok) {
          const xml = await rawRes.text();
          const parsed = parseRssXml(xml, feed.url);
          if (parsed.length > 0) {
            return toFeedItems(
              feed,
              parsed.map((p) => ({
                title: p.title,
                link: p.link,
                pubDate: p.pubDate,
                description: p.description,
                content: p.content,
                thumbnail: p.thumbnail,
              })),
            );
          }
        }
      } catch {
        /* próximo fallback */
      }

      // 3) Jina (sites que bloqueiam RSS/CORS — ex.: Código Poker)
      try {
        const jinaUrl = `https://r.jina.ai/${feed.url}`;
        const jinaRes = await fetch(jinaUrl, {
          signal: AbortSignal.timeout(20000),
          headers: { Accept: "text/plain" },
        });
        if (!jinaRes.ok) return [];
        const body = await jinaRes.text();
        const fromXml = parseRssXml(body, feed.url);
        if (fromXml.length > 0) {
          return toFeedItems(
            feed,
            fromXml.map((p) => ({
              title: p.title,
              link: p.link,
              pubDate: p.pubDate,
              description: p.description,
              content: p.content,
              thumbnail: p.thumbnail,
            })),
          );
        }
        const fromMd = parseJinaMarkdown(body);
        return toFeedItems(
          feed,
          fromMd.map((p) => ({
            title: p.title,
            link: p.link,
            pubDate: p.pubDate,
            description: p.description,
            content: p.content,
            thumbnail: p.thumbnail,
          })),
        );
      } catch {
        return [];
      }
    }

    async function fetchAllContent() {
      setLoading(true);
      setError(null);

      try {
        const batches = await Promise.all(
          CONTENT_FEEDS.map(async (feed) => ({ feed, items: await fetchOneFeed(feed) })),
        );

        const newsPool: FeedItem[] = [];
        const tipsPool: FeedItem[] = [];

        for (const { feed, items: feedItems } of batches) {
          for (const item of feedItems) {
            // Jogando melhor: só tips dedicados ou "both" com estratégia clara
            if (isStrategyContent(item.title, item.description ?? "", feed.kind)) {
              tipsPool.push({
                ...item,
                street: item.street ?? classifyStreet(item.title, item.description ?? "") ?? "preflop",
                id: `tip:${item.id}`,
              });
            }
            // Notícias: nunca entra feed exclusivo de tips
            if (feed.kind !== "tips") {
              newsPool.push(item);
            }
          }
        }

        // Notícias locais — capa só se a matéria original tiver foto
        for (const news of LOCAL_NEWS) {
          newsPool.push({
            id: news.id,
            title: news.title,
            link: news.link || "",
            pubDate: news.pubDate,
            description: news.description,
            source: news.category,
            isLocal: true,
          });
        }

        const dedupe = (list: FeedItem[]) => {
          const seen = new Set<string>();
          const out: FeedItem[] = [];
          for (const item of list) {
            const key = (item.link || item.title).trim().toLowerCase();
            if (!key || seen.has(key)) continue;
            seen.add(key);
            out.push(item);
          }
          out.sort((a, b) => parseRSSDate(b.pubDate).getTime() - parseRSSDate(a.pubDate).getTime());
          return out;
        };

        const topNews = dedupe(newsPool).slice(0, NEWS_FEED_LIMIT);
        const topTips = dedupe(tipsPool).slice(0, TIPS_FEED_LIMIT);

        const enrichOg = async (list: FeedItem[]) => {
          await Promise.all(
            list.map(async (item) => {
              if (!item.link) return;
              // Sempre tenta a foto oficial da página; se falhar, mantém a do RSS
              const og = await resolveSourceImage(item.link);
              if (!og) return;
              item.imageUrl = og;
              item.imageCaption = "Foto da matéria";
            }),
          );
        };

        await Promise.all([enrichOg(topNews), enrichOg(topTips)]);

        if (mounted) {
          setNewsItems([...topNews]);
          setRemoteTips([...topTips]);
        }

        const translateList = async (
          list: FeedItem[],
          onUpdate: () => void,
          opts?: { forceAllForeign?: boolean },
        ) => {
          const forceAll = Boolean(opts?.forceAllForeign);
          for (const item of list) {
            if (!mounted) return;
            const foreign = item.fromLang === "en" || item.fromLang === "es" || item.needsTranslation;
            const looksForeign =
              !looksLikePortuguese(item.title) ||
              (item.description ? !looksLikePortuguese(item.description.slice(0, 280)) : false);
            if (!foreign && !(forceAll && looksForeign)) continue;
            try {
              const pt = await translateNewsFields({
                title: item.title,
                description: item.description,
                fromLang: item.fromLang ?? "auto",
                force: true,
              });
              item.title = pt.title;
              item.description = pt.description;
              item.translated = true;
              item.needsTranslation = false;
              item.street =
                classifyStreet(item.title, item.description ?? "") ?? item.street ?? "preflop";
              onUpdate();
            } catch {
              /* mantém original */
            }
          }
        };

        // Dicas: sempre forçar PT-BR em conteúdo estrangeiro
        await translateList(topTips, () => mounted && setRemoteTips([...topTips]), {
          forceAllForeign: true,
        });
        if (mounted) setRemoteTips([...topTips]);

        await translateList(topNews, () => mounted && setNewsItems([...topNews]));
        if (mounted) setNewsItems([...topNews]);
      } catch {
        if (mounted) setError("Falha ao carregar conteudo. Tente novamente mais tarde.");
      } finally {
        if (mounted) setLoading(false);
      }
    }

    void fetchAllContent();
    const interval = setInterval(() => void fetchAllContent(), 10 * 60 * 1000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  const localTips: FeedItem[] = LOCAL_TIPS.map((tip, idx) => ({
    id: tip.id,
    title: tip.title,
    link: tip.link || "",
    pubDate: new Date(Date.now() - idx * 86400000).toISOString(),
    description: tip.description,
    source: tip.category,
    street: tip.street,
    isLocal: true as const,
    imageUrl: tip.imageUrl && !isAdBanner(tip.imageUrl) ? tip.imageUrl : undefined,
  }));

  const allTips = [...localTips, ...remoteTips];
  const tipsByStreet = allTips.filter((tip) => tip.street === activeStreet);

  const items: FeedItem[] =
    activeTab === "news" ? newsItems : tipsByStreet.map((tip) => ({ ...tip, id: tip.id }));

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
        {loading && (
          <div className="flex items-center justify-center py-8">
            <div className="zt-spinner" />
            <span className="ml-3 text-felt-300">
              {activeTab === "news" ? "Carregando noticias..." : "Carregando dicas e estrategia..."}
            </span>
          </div>
        )}

        {error && (
          <div className="py-8 text-center text-felt-300">
            <p className="mb-2 text-red-400">{error}</p>
            <button type="button" onClick={() => window.location.reload()} className="zt-btn-ghost text-sm">
              Tentar novamente
            </button>
          </div>
        )}

        {!loading && items.length === 0 && (
          <p className="py-8 text-center text-felt-400">Nenhum item encontrado.</p>
        )}

        <div
          className="space-y-3"
          role="feed"
          aria-label={`${activeTab === "news" ? "Noticias" : "Jogando melhor"} de poker`}
        >
          {!loading &&
            items.map((item) => {
              const itemKey = String(item.id);
              const isExpanded = expandedItems.has(itemKey);
              const rssBody = (item.description ?? "").trim();
              const body = (fullBodies[itemKey] || rssBody).trim();
              const canExpand = Boolean(item.link) || body.length > 0;
              const isLoadingBody = Boolean(loadingBodies[itemKey]);
              const photo = item.imageUrl;
              const caption = item.imageCaption || (photo ? "Foto da matéria" : undefined);

              return (
                <article
                  key={itemKey}
                  className={`group zt-card overflow-hidden transition-colors hover:border-gold-soft/30 ${isExpanded ? "border-gold-soft/40" : ""}`}
                >
                  {photo && (
                    <NewsImage
                      url={photo}
                      alt={item.title}
                      caption={caption}
                      large={isExpanded}
                      onClick={() => canExpand && toggleExpand(item)}
                    />
                  )}

                  <div className="p-4">
                    <div className="mb-1 flex items-center gap-2 text-xs text-felt-400">
                      <span className="zt-chip text-[10px]">
                        {item.source}
                        {item.translated ? " · PT" : item.needsTranslation ? " · traduzindo…" : ""}
                      </span>
                      {!item.isLocal && <time dateTime={item.pubDate}>{formatDate(item.pubDate)}</time>}
                    </div>
                    <button
                      type="button"
                      aria-expanded={isExpanded}
                      onClick={() => canExpand && toggleExpand(item)}
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
                        {isExpanded ? (
                          isLoadingBody && body.length < 80 ? (
                            <p className="mt-3 text-sm text-felt-400">Carregando texto da matéria…</p>
                          ) : body ? (
                            <TipRichText
                              text={body}
                              className="mt-3 text-sm leading-relaxed text-felt-200 break-words"
                            />
                          ) : (
                            <p className="mt-3 text-sm text-felt-400">
                              Não foi possível carregar o texto aqui. Use o link da fonte abaixo.
                            </p>
                          )
                        ) : body ? (
                          <div
                            className="mt-2 text-sm leading-relaxed text-felt-300"
                            style={{
                              display: "-webkit-box",
                              WebkitLineClamp: 3,
                              WebkitBoxOrient: "vertical" as const,
                              overflow: "hidden",
                            }}
                          >
                            {body}
                          </div>
                        ) : (
                          <p className="mt-2 text-sm text-felt-400">
                            Clique no título para ler a matéria.
                          </p>
                        )}

                        <button
                          type="button"
                          onClick={() => toggleExpand(item)}
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
                </article>
              );
            })}
        </div>
      </div>
    </div>
  );
}
