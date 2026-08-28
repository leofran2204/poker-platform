/**
 * Quando a fonte não publica foto, busca na web (Wikipedia / Wikimedia Commons)
 * uma imagem do jogador ou do evento citado no título.
 * Se não houver registro público da pessoa → foto aleatória do evento.
 */

import newsMedia from "@/data/newsMedia.json";

const CACHE_PREFIX = "zt-news-photo-v5:";

type EventKey = "bsop" | "wsop" | "ept" | "triton" | "pokerstars" | "default";

export type PhotoKind = "article" | "person" | "event-web" | "event-local" | "curated" | "tip";

export interface StoryPhoto {
  url: string;
  caption: string;
  kind: PhotoKind;
}

const EVENT_LABELS: Record<EventKey, string> = {
  bsop: "BSOP",
  wsop: "WSOP",
  ept: "EPT",
  triton: "Triton",
  pokerstars: "PokerStars",
  default: "Poker",
};

const EVENT_PHOTO_POOLS = (newsMedia.eventPhotoPools ?? {}) as Record<string, string[]>;

/** Queries focadas em FOTO real (mesa/jogadores), nunca logo/branding. */
const EVENT_QUERIES: Array<{ key: EventKey; match: RegExp; queries: string[] }> = [
  {
    key: "bsop",
    match: /\bBSOP\b|\bFloripa\b/i,
    queries: [
      "poker tournament final table players",
      "poker live tournament room felt",
      "World Series of Poker final table",
    ],
  },
  {
    key: "wsop",
    match: /\bWSOP\b/i,
    queries: [
      "WSOP Main Event final table",
      "World Series of Poker players at table",
      "WSOP Las Vegas tournament floor",
    ],
  },
  {
    key: "ept",
    match: /\bEPT\b/i,
    queries: [
      "European Poker Tour player",
      "EPT poker final table",
      "poker tournament players felt",
    ],
  },
  {
    key: "triton",
    match: /\bTriton\b/i,
    queries: ["poker high roller final table", "poker tournament players"],
  },
  {
    key: "wsop",
    match: /\bWPT\b/i,
    queries: ["World Poker Tour final table", "poker tournament players"],
  },
  {
    key: "pokerstars",
    match: /\bPokerStars\b/i,
    queries: ["poker live tournament table", "poker players at final table"],
  },
  {
    key: "default",
    match: /\bGGPoker\b/i,
    queries: ["poker tournament final table"],
  },
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

function cacheGet(key: string): StoryPhoto | null | undefined {
  try {
    const raw = sessionStorage.getItem(CACHE_PREFIX + key);
    if (raw === null) return undefined;
    if (raw === "" || raw === "null") return null;
    return JSON.parse(raw) as StoryPhoto;
  } catch {
    return undefined;
  }
}

function cacheSet(key: string, value: StoryPhoto | null) {
  try {
    sessionStorage.setItem(CACHE_PREFIX + key, value ? JSON.stringify(value) : "null");
  } catch {
    /* ignore quota */
  }
}

/** Nome da pessoa ou do evento para a legenda. */
export function extractPrimarySubject(title: string, description?: string): string {
  const nick = title.match(
    /\b([A-ZÁÉÍÓÚ][a-záéíóú]+[A-ZÁÉÍÓÚ][A-Za-zÁÉÍÓÚáéíóúÂÊÔÃÕâêôãõÇç0-9]+)\b/,
  );
  if (nick) return nick[1];

  const nameRe =
    /\b([A-ZÁÉÍÓÚÂÊÔÃÕ][a-záéíóúâêôãõç]+(?:\s+[A-ZÁÉÍÓÚÂÊÔÃÕ][a-záéíóúâêôãõç]+){1,2})\b/g;
  let m: RegExpExecArray | null;
  while ((m = nameRe.exec(title)) !== null) {
    const parts = m[1].split(/\s+/);
    if (parts.every((p) => TITLE_STOP.has(p.toLowerCase()))) continue;
    if (parts.some((p) => TITLE_STOP.has(p.toLowerCase())) && parts.length === 2) continue;
    return m[1];
  }

  const eventKey = detectEventKey(title, description);
  if (eventKey !== "default") return EVENT_LABELS[eventKey];
  if (/\bFloripa\b/i.test(title)) return "BSOP Floripa";
  return "";
}

export function buildPhotoCaption(
  title: string,
  kind: PhotoKind,
  description?: string,
): string {
  const personOrTopic = extractPrimarySubject(title, description);
  const eventKey = detectEventKey(title, description);
  const eventName = EVENT_LABELS[eventKey];

  switch (kind) {
    case "article":
      return personOrTopic
        ? `Jogador: ${personOrTopic} (foto da matéria)`
        : "Foto da matéria";
    case "person":
      return personOrTopic
        ? `Jogador: ${personOrTopic}`
        : "Foto do jogador";
    case "event-web":
      return `Torneio: ${eventName}`;
    case "event-local":
      return `Torneio: ${eventName}`;
    case "curated":
      return personOrTopic
        ? `Jogador: ${personOrTopic}`
        : eventName !== "Poker"
          ? `Torneio: ${eventName}`
          : "Foto relacionada à matéria";
    case "tip":
      return personOrTopic
        ? `Jogador: ${personOrTopic} — guia`
        : "Guia de estratégia";
    default:
      return "Foto relacionada";
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

/** Rejeita logos, wordmarks, ícones e artes de branding. */
function isLogoOrBranding(text: string): boolean {
  const t = text.toLowerCase();
  return (
    t.includes("logo") ||
    t.includes("wordmark") ||
    t.includes("branding") ||
    t.includes("brand-") ||
    t.includes("logotipo") ||
    t.includes("icon") ||
    t.includes("favicon") ||
    t.includes("badge") ||
    t.includes("seal") ||
    t.includes("emblem") ||
    t.includes("svg") ||
    t.includes("vector") ||
    t.includes("banner") ||
    t.includes("820x100") ||
    t.includes("thumb-3") ||
    t.includes("garantido-do-mega") ||
    /\blogs?\b/.test(t)
  );
}

function isLikelyPhotoUrl(url: string): boolean {
  const u = url.toLowerCase();
  if (!/^https?:\/\//.test(u)) return false;
  if (u.includes("/ads/") || u.endsWith(".svg") || u.endsWith(".pdf")) return false;
  if (isLogoOrBranding(u)) return false;
  return true;
}

function isLikelyRealPhotoTitle(title: string): boolean {
  const t = title.toLowerCase();
  if (isLogoOrBranding(t)) return false;
  // Prefer titles that look like photos of people/tables
  const photoHints =
    /player|final table|tournament|felt|poker|wsop|ept|wpt|table|event|day \d|main event/i.test(
      t,
    );
  const junk =
    /\.pdf|\.djvu|stethoscope|diary|tour in palestine|punch on tour|crowsnest|southey/i.test(t);
  if (junk) return false;
  return photoHints || /\.(jpe?g|png|webp)$/i.test(t);
}

function pushUnique(queries: string[], seen: Set<string>, q: string) {
  const clean = q.replace(/\s+/g, " ").trim();
  if (clean.length < 3) return;
  const key = clean.toLowerCase();
  if (seen.has(key)) return;
  seen.add(key);
  queries.push(clean);
}

/** Só nomes/apelidos de jogador — para buscar FOTO DO JOGADOR. */
export function extractPlayerSearchQueries(title: string, description?: string): string[] {
  const text = `${title} ${description ?? ""}`;
  const queries: string[] = [];
  const seen = new Set<string>();

  const nick = title.match(
    /\b([A-ZÁÉÍÓÚ][a-záéíóú]+[A-ZÁÉÍÓÚ][A-Za-zÁÉÍÓÚáéíóúÂÊÔÃÕâêôãõÇç0-9]+)\b/,
  );
  if (nick) {
    pushUnique(queries, seen, `${nick[1]} poker player`);
    pushUnique(queries, seen, `${nick[1]} poker`);
    pushUnique(queries, seen, nick[1]);
  }

  // Aspas / nick online: "Godoy217", 'bnalon'
  const quoted = text.matchAll(/["“']([A-Za-z][A-Za-z0-9_]{2,24})["”']/g);
  for (const qm of quoted) {
    pushUnique(queries, seen, `${qm[1]} poker`);
  }

  const nameRe =
    /\b([A-ZÁÉÍÓÚÂÊÔÃÕ][a-záéíóúâêôãõç]+(?:\s+[A-ZÁÉÍÓÚÂÊÔÃÕ][a-záéíóúâêôãõç]+){1,2})\b/g;
  let m: RegExpExecArray | null;
  while ((m = nameRe.exec(title)) !== null) {
    const parts = m[1].split(/\s+/);
    if (parts.every((p) => TITLE_STOP.has(p.toLowerCase()))) continue;
    if (parts.some((p) => TITLE_STOP.has(p.toLowerCase())) && parts.length === 2) continue;
    pushUnique(queries, seen, `${m[1]} poker player`);
    pushUnique(queries, seen, `${m[1]} poker`);
    pushUnique(queries, seen, m[1]);
  }

  return queries.slice(0, 6);
}

/** Consultas de torneio/evento — mesa final, salão, jogadores no feltro. */
export function extractEventTournamentQueries(title: string, description?: string): string[] {
  const text = `${title} ${description ?? ""}`;
  const queries: string[] = [];
  const seen = new Set<string>();
  for (const ev of EVENT_QUERIES) {
    if (!ev.match.test(text)) continue;
    for (const q of ev.queries) pushUnique(queries, seen, q);
  }
  if (queries.length === 0) {
    pushUnique(queries, seen, "poker tournament final table");
    pushUnique(queries, seen, "poker live tournament players");
  }
  return queries.slice(0, 5);
}

/** @deprecated use extractPlayerSearchQueries + extractEventTournamentQueries */
export function buildPhotoSearchQueries(title: string, description?: string): string[] {
  return [
    ...extractPlayerSearchQueries(title, description),
    ...extractEventTournamentQueries(title, description),
  ].slice(0, 8);
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
    if (src && isLikelyPhotoUrl(src) && !isLogoOrBranding(src) && !isLogoOrBranding(titleQuery)) {
      return cleanPhotoUrl(src);
    }
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
  // Exclui logo/branding já na query do Commons
  const search = `${query} -logo -wordmark -branding -svg -icon -vector filetype:bitmap`;
  const api =
    "https://commons.wikimedia.org/w/api.php?" +
    new URLSearchParams({
      action: "query",
      generator: "search",
      gsrsearch: search,
      gsrnamespace: "6",
      gsrlimit: "12",
      prop: "imageinfo",
      iiprop: "url|mime|size",
      iiurlwidth: "1280",
      format: "json",
      origin: "*",
    }).toString();

  try {
    const res = await fetch(api, { signal: AbortSignal.timeout(9000) });
    if (!res.ok) return undefined;
    const data = (await res.json()) as {
      query?: {
        pages?: Record<
          string,
          {
            title?: string;
            imageinfo?: Array<{
              url?: string;
              thumburl?: string;
              mime?: string;
              width?: number;
              height?: number;
            }>;
          }
        >;
      };
    };
    const pages = Object.values(data.query?.pages ?? {});
    for (const page of pages) {
      const info = page.imageinfo?.[0];
      const mime = info?.mime ?? "";
      if (mime && !mime.startsWith("image/")) continue;
      if (mime === "image/svg+xml") continue;
      const fileTitle = page.title ?? "";
      if (!isLikelyRealPhotoTitle(fileTitle)) continue;
      // Logos costumam ser pequenos / quase quadrados art boards
      const w = info?.width ?? 0;
      const h = info?.height ?? 0;
      if (w > 0 && h > 0 && w < 400 && h < 400) continue;
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

function makePhoto(url: string, title: string, kind: PhotoKind, description?: string): StoryPhoto {
  return { url, kind, caption: buildPhotoCaption(title, kind, description) };
}

/** Foto aleatória do evento (pool local) — último recurso com sentido. */
export function randomEventPhoto(title: string, description?: string): StoryPhoto | undefined {
  const key = detectEventKey(title, description);
  const pool = EVENT_PHOTO_POOLS[key] ?? EVENT_PHOTO_POOLS.default ?? [];
  const fallback = EVENT_PHOTO_POOLS.default ?? [];
  const url = pickRandom(pool.length ? pool : fallback);
  if (!url) return undefined;
  return makePhoto(url, title, "event-local", description);
}

async function findPlayerPhoto(title: string, description?: string): Promise<string | undefined> {
  const playerQueries = extractPlayerSearchQueries(title, description);
  for (const q of playerQueries) {
    const bare = q
      .replace(/\s+poker player$/i, "")
      .replace(/\s+poker$/i, "")
      .trim();

    const fromWiki =
      (await wikipediaThumbnail(bare, "en")) ||
      (await wikipediaThumbnail(bare, "pt")) ||
      (await wikipediaSearchThumbnail(`${bare} poker`, "en")) ||
      (await wikipediaSearchThumbnail(`${bare} poker`, "pt")) ||
      (await wikipediaSearchThumbnail(q, "en"));

    if (fromWiki) return fromWiki;

    const fromCommons =
      (await commonsImage(`${bare} poker player`)) ||
      (await commonsImage(`${bare} poker`)) ||
      (await commonsImage(`${bare} WSOP`)) ||
      (await commonsImage(bare));

    if (fromCommons) return fromCommons;
  }
  return undefined;
}

async function findEventTournamentPhoto(
  title: string,
  description?: string,
): Promise<string | undefined> {
  for (const q of extractEventTournamentQueries(title, description)) {
    const img = await commonsImage(q);
    if (img) return img;
  }
  return undefined;
}

/**
 * Prioridade fixa:
 * 1) Foto do JOGADOR (Wikipedia/Commons)
 * 2) Se não houver → foto de TORNEIO do evento (mesa final / salão)
 * 3) Pool local de fotos reais de torneio (nunca logo)
 */
export async function lookupStoryPhoto(
  title: string,
  description?: string,
): Promise<StoryPhoto | undefined> {
  const cacheKey = title.toLowerCase().slice(0, 120);
  const cached = cacheGet(cacheKey);
  if (cached && typeof cached === "object" && cached.url) return cached;

  // 1) Jogador
  const playerUrl = await findPlayerPhoto(title, description);
  if (playerUrl) {
    const photo = makePhoto(playerUrl, title, "person", description);
    cacheSet(cacheKey, photo);
    return photo;
  }

  // 2) Torneio do evento (foto real, não logo)
  const eventUrl = await findEventTournamentPhoto(title, description);
  if (eventUrl) {
    const photo = makePhoto(eventUrl, title, "event-web", description);
    cacheSet(cacheKey, photo);
    return photo;
  }

  // 3) Pool local de torneio
  const localEvent = randomEventPhoto(title, description);
  if (localEvent) {
    cacheSet(cacheKey, localEvent);
    return localEvent;
  }

  cacheSet(cacheKey, null);
  return undefined;
}
