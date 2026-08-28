/**
 * Traduz texto para português (PT-BR) via MyMemory (CORS ok, sem API key).
 * Usado só em notícias de fontes internacionais.
 */

const CACHE_PREFIX = "zt-tr-pt-v1:";
const CHUNK = 420;

function cacheGet(key: string): string | undefined {
  try {
    return sessionStorage.getItem(CACHE_PREFIX + key) ?? undefined;
  } catch {
    return undefined;
  }
}

function cacheSet(key: string, value: string) {
  try {
    sessionStorage.setItem(CACHE_PREFIX + key, value);
  } catch {
    /* quota */
  }
}

function hashKey(text: string): string {
  let h = 0;
  for (let i = 0; i < text.length; i += 1) h = (h * 31 + text.charCodeAt(i)) | 0;
  return `${text.length}:${h}`;
}

function chunkText(text: string, size: number): string[] {
  if (text.length <= size) return [text];
  const chunks: string[] = [];
  let rest = text;
  while (rest.length > 0) {
    if (rest.length <= size) {
      chunks.push(rest);
      break;
    }
    let cut = rest.lastIndexOf(" ", size);
    if (cut < size * 0.5) cut = size;
    chunks.push(rest.slice(0, cut).trim());
    rest = rest.slice(cut).trim();
  }
  return chunks.filter(Boolean);
}

async function translateChunk(chunk: string, fromLang: "en" | "es" | "auto"): Promise<string> {
  const pair = fromLang === "auto" ? "Autodetect|pt" : `${fromLang}|pt`;
  const url =
    "https://api.mymemory.translated.net/get?q=" +
    encodeURIComponent(chunk) +
    "&langpair=" +
    encodeURIComponent(pair);
  const res = await fetch(url, { signal: AbortSignal.timeout(12000) });
  if (!res.ok) return chunk;
  const data = (await res.json()) as {
    responseData?: { translatedText?: string };
    responseStatus?: number;
  };
  const out = data.responseData?.translatedText?.trim();
  if (!out || data.responseStatus === 429) return chunk;
  // MyMemory às vezes devolve a própria query se falhar
  if (out.toLowerCase() === chunk.toLowerCase()) return chunk;
  return out;
}

/** Heurística leve: texto já parece português? */
export function looksLikePortuguese(text: string): boolean {
  const t = text.toLowerCase();
  const hits = (t.match(/\b(ção|ões|não|você|também|para|com|uma|que|dos|das|pelo|pela|está|são|foi|após|torneio|jogadores)\b/g) || [])
    .length;
  const english = (t.match(/\b(the|and|with|from|this|that|have|will|poker|tournament|player)\b/g) || [])
    .length;
  return hits >= 3 && hits >= english;
}

/**
 * Traduz para português. Em falha, devolve o original.
 * @param fromLang idioma de origem (en/es) ou auto
 */
export async function translateToPortuguese(
  text: string,
  fromLang: "en" | "es" | "auto" = "en",
): Promise<string> {
  const trimmed = text.trim();
  if (!trimmed) return text;
  if (looksLikePortuguese(trimmed)) return text;

  const key = `${fromLang}:${hashKey(trimmed)}`;
  const cached = cacheGet(key);
  if (cached) return cached;

  try {
    const parts: string[] = [];
    for (const chunk of chunkText(trimmed, CHUNK)) {
      parts.push(await translateChunk(chunk, fromLang));
      // Pequena pausa para não estourar rate limit
      await new Promise((r) => setTimeout(r, 120));
    }
    const result = parts.join(" ").replace(/\s+/g, " ").trim();
    if (result) cacheSet(key, result);
    return result || text;
  } catch {
    return text;
  }
}

export async function translateNewsFields(input: {
  title: string;
  description?: string;
  fromLang?: "en" | "es" | "auto";
}): Promise<{ title: string; description?: string }> {
  const from = input.fromLang ?? "en";
  const title = await translateToPortuguese(input.title, from);
  let description = input.description;
  if (description && description.trim()) {
    // Limita corpo traduzido para caber na UI / API
    const clipped =
      description.length > 2500 ? `${description.slice(0, 2500).trim()}…` : description;
    description = await translateToPortuguese(clipped, from);
  }
  return { title, description };
}
