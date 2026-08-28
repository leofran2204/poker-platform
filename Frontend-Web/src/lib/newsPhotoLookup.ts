/**
 * Quando a fonte não publica foto, busca na web (Wikipedia / Wikimedia Commons)
 * uma imagem do jogador ou do evento citado no título.
 * Se não houver registro público da pessoa → foto aleatória do evento.
 */

import newsMedia from "@/data/newsMedia.json";

const CACHE_PREFIX = "zt-news-photo-v2:";

type EventKey = "bsop" | "wsop" | "ept" | "triton" | "pokerstars" | "default";

const EVENT_PHOTO_POOLS = (newsMedia.eventPhotoPools ?? {}) as Record<string, string[]>;

const EVENT_QUERIES: Array<{ key: EventKey; match: RegExp; queries: string[] }> = [
  { key: "bsop", match: /\bBSOP\b|\bFloripa\b/i, queries: ["BSOP poker", "Brazilian Series of Poker"] },
  { key: "wsop", match: /\bWSOP\b/i, queries: ["World Series of Poker", "WSOP Las Vegas poker"] },
  { key: "ept", match: /\bEPT\b/i, queries: ["European Poker Tour", "EPT poker"] },
  { key: "triton", match: /\bTriton\b/i, queries: ["Triton Poker", "Triton Super High Roller"] },
  { key: "wsop", match: /\bWPT\b/i, queries: ["World Poker Tour"] },
  { key: "pokerstars", match: /\bPokerStars\b/i, queries: ["PokerStars live poker"] },
  { key: "default", match: /\bGGPoker\b/i, queries: ["GGPoker"] },
];

const TITLE_STOP = new Set(
  [
    "Morre",
    "Morreu",
    "Anos",
    "Lenda",
    "Belga",
    "Finalista",
    "Main",
    "Event",
    "Como",
    "Jogar",
    "Contra",
    "Usa",
    "Nota",
    "Acerta",
    "Leva",
    "Pote",
    "Blinds",
    "Fara",
    "Fará",
    "Cobertura",
    "Presencial",
    "Las",
    "Vegas",
    "Pelo",
    "Ano",
    "Jogadores",
    "Classificados",
    "Via",
    "Satelite",
    "Satélite",
    "Dobram",
    "Garantidos",
    "Mega",
    "Satelites",
    "Satélites",
    "Classificam",
    "Para",
    "Com",
    "Uma",
    "Sobre",
    "Apos",
    "Após",
    "Confira",
    "Detalhes",
    "Mundo",
    "Poker",
    "Noticias",
    "Notícias",
    "Geral",
    "Online",
    "The",
    "And",
    "For",
    "With",
  ].map((s) => s.toLowerCase()),
);

function cacheGet(key: string): string | null | undefined {
  try {
    const raw = sessionStorage.getItem(CACHE_PREFIX + key);
    if (raw === null) return undefined;
    if (raw === "") return null;
    return raw;
  } catch {
    return undefined;
  }
}

function cacheSet(key: string, value: string | null) {
  try {
    sessionStorage.setItem(CACHE_PREFIX + key, value ?? "");
  } catch {
    /* ignore quota */
  }
}

function cleanPhotoUrl(url: string): string {
  try {
    const u = new URL(url);
    u.search = "";
    return u.toString();
  } catch {
    return url;
  }
}

function isLikelyPhotoUrl(url: string): boolean {
  const u = url.toLowerCase();
  if (!/^https?:\/\//.test(u)) return false;
  if (u.includes("banner") || u.includes("/ads/") || u.includes("favicon") || u.endsWith(".svg")) {
    return false;
  }
  if (u.includes("logo") && !u.includes("poker")) return false;
  return true;
}

/** Extrai possíveis nomes de pessoa e consultas de evento a partir do título. */
export function buildPhotoSearchQueries(title: string, description?: string): string[] {
  const text = `${title} ${description ?? ""}`;
  const queries: string[] = [];
  const seen = new Set<string>();

  const push = (q: string) => {
    const clean = q.replace(/\s+/g, " ").trim();
    if (clean.length < 3) return;
    const key = clean.toLowerCase();
    if (seen.has(key)) return;
    seen.add(key);
    queries.push(clean);
  };

  // Eventos conhecidos (consultas web)
  for (const ev of EVENT_QUERIES) {
    if (ev.match.test(text)) {
      for (const q of ev.queries) push(q);
    }
  }

  // Apelidos CamelCase / colados (AbacateLeao, bnalon)
  const nick = title.match(/\b([A-ZÁÉÍÓÚ][a-záéíóú]+[A-ZÁÉÍÓÚ][A-Za-zÁÉÍÓÚáéíóúÂÊÔÃÕâêôãõÇç0-9]+)\b/);
  if (nick) push(`${nick[1]} poker`);

  // Nomes próprios: 2–3 palavras capitalizadas
  const nameRe =
    /\b([A-ZÁÉÍÓÚÂÊÔÃÕ][a-záéíóúâêôãõç]+(?:\s+[A-ZÁÉÍÓÚÂÊÔÃÕ][a-záéíóúâêôãõç]+){1,2})\b/g;
  let m: RegExpExecArray | null;
  while ((m = nameRe.exec(title)) !== null) {
    const parts = m[1].split(/\s+/);
    if (parts.every((p) => TITLE_STOP.has(p.toLowerCase()))) continue;
    if (parts.some((p) => TITLE_STOP.has(p.toLowerCase())) && parts.length === 2) {
      // "Main Event" etc.
      continue;
    }
    push(`${m[1]} poker`);
    push(m[1]);
  }

  // Floripa / etapa no título
  if (/\bFloripa\b/i.test(title)) push("BSOP Floripa poker");

  return queries.slice(0, 6);
}

async function wikipediaThumbnail(titleQuery: string, lang: "en" | "pt"): Promise<string | undefined> {
  const slug = titleQuery.replace(/\s+/g, "_");
  const url = `https://${lang}.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(slug)}`;
  try {
    const res = await fetch(url, {
      headers: { Accept: "application/json" },
      signal: AbortSignal.timeout(7000),
    });
    if (!res.ok) return undefined;
    const data = (await res.json()) as {
      type?: string;
      thumbnail?: { source?: string };
      originalimage?: { source?: string };
    };
    if (data.type === "disambiguation") return undefined;
    const src = data.originalimage?.source || data.thumbnail?.source;
    if (src && isLikelyPhotoUrl(src)) return cleanPhotoUrl(src);
  } catch {
    /* ignore */
  }
  return undefined;
}

async function wikipediaSearchThumbnail(query: string, lang: "en" | "pt"): Promise<string | undefined> {
  const api = `https://${lang}.wikipedia.org/w/api.php?action=query&list=search&srsearch=${encodeURIComponent(
    query,
  )}&srlimit=3&format=json&origin=*`;
  try {
    const res = await fetch(api, { signal: AbortSignal.timeout(7000) });
    if (!res.ok) return undefined;
    const data = (await res.json()) as {
      query?: { search?: Array<{ title: string }> };
    };
    const hits = data.query?.search ?? [];
    for (const hit of hits) {
      const thumb = await wikipediaThumbnail(hit.title, lang);
      if (thumb) return thumb;
    }
  } catch {
    /* ignore */
  }
  return undefined;
}

async function commonsImage(query: string): Promise<string | undefined> {
  const api =
    "https://commons.wikimedia.org/w/api.php?" +
    new URLSearchParams({
      action: "query",
      generator: "search",
      gsrsearch: query,
      gsrnamespace: "6",
      gsrlimit: "8",
      prop: "imageinfo",
      iiprop: "url|mime",
      iiurlwidth: "1200",
      format: "json",
      origin: "*",
    }).toString();

  try {
    const res = await fetch(api, { signal: AbortSignal.timeout(8000) });
    if (!res.ok) return undefined;
    const data = (await res.json()) as {
      query?: {
        pages?: Record<
          string,
          {
            title?: string;
            imageinfo?: Array<{ url?: string; thumburl?: string; mime?: string }>;
          }
        >;
      };
    };
    const pages = Object.values(data.query?.pages ?? {});
    for (const page of pages) {
      const info = page.imageinfo?.[0];
      const mime = info?.mime ?? "";
      if (mime && !mime.startsWith("image/")) continue;
      const title = (page.title ?? "").toLowerCase();
      // Evita diagramas / logos genéricos demais
      if (title.includes("logo") && !/poker|wsop|ept|bsop/.test(query.toLowerCase())) continue;
      const src = info?.thumburl || info?.url;
      if (src && isLikelyPhotoUrl(src)) return cleanPhotoUrl(src);
    }
  } catch {
    /* ignore */
  }
  return undefined;
}

/** Detecta qual circuito/evento o título menciona. */
export function detectEventKey(title: string, description?: string): EventKey {
  const text = `${title} ${description ?? ""}`;
  for (const ev of EVENT_QUERIES) {
    if (ev.match.test(text)) return ev.key;
  }
  return "default";
}

function pickRandom<T>(arr: T[]): T | undefined {
  if (!arr.length) return undefined;
  return arr[Math.floor(Math.random() * arr.length)];
}

/** Foto aleatória do evento (pool local) — último recurso com sentido. */
export function randomEventPhoto(title: string, description?: string): string | undefined {
  const key = detectEventKey(title, description);
  const pool = EVENT_PHOTO_POOLS[key] ?? EVENT_PHOTO_POOLS.default ?? [];
  const fallback = EVENT_PHOTO_POOLS.default ?? [];
  return pickRandom(pool.length ? pool : fallback);
}

/**
 * Busca foto do jogador/evento.
 * Ordem: cache → Wikipedia/Commons (pessoa) → Commons (evento) → foto aleatória do evento.
 */
export async function lookupStoryPhoto(title: string, description?: string): Promise<string | undefined> {
  const queries = buildPhotoSearchQueries(title, description);
  const cacheKey = (queries[0] ?? title).toLowerCase().slice(0, 120);
  const cached = cacheGet(cacheKey);
  if (typeof cached === "string") return cached;
  // cached === null → pessoa sem registro; ainda tentamos evento abaixo

  // 1) Pessoa / queries específicas (Wikipedia + Commons)
  for (const q of queries) {
    const bare = q.replace(/\s+poker$/i, "").trim();

    const fromWiki =
      (await wikipediaThumbnail(bare, "en")) ||
      (await wikipediaThumbnail(bare, "pt")) ||
      (await wikipediaSearchThumbnail(q, "en")) ||
      (await wikipediaSearchThumbnail(q, "pt")) ||
      (await wikipediaSearchThumbnail(bare, "en"));

    if (fromWiki) {
      cacheSet(cacheKey, fromWiki);
      return fromWiki;
    }

    const fromCommons =
      (await commonsImage(q)) ||
      (await commonsImage(`${bare} poker`)) ||
      (await commonsImage(bare));

    if (fromCommons) {
      cacheSet(cacheKey, fromCommons);
      return fromCommons;
    }
  }

  // 2) Só o evento na Commons (sem a pessoa)
  const eventKey = detectEventKey(title, description);
  const eventMeta = EVENT_QUERIES.find((e) => e.key === eventKey);
  if (eventMeta) {
    for (const q of eventMeta.queries) {
      const evImg = await commonsImage(q);
      if (evImg) {
        cacheSet(cacheKey, evImg);
        return evImg;
      }
    }
  }

  // 3) Sem registro público → foto aleatória do evento (pool local)
  const localEvent = randomEventPhoto(title, description);
  if (localEvent) {
    cacheSet(cacheKey, localEvent);
    return localEvent;
  }

  cacheSet(cacheKey, null);
  return undefined;
}
