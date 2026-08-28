/**
 * Traduz texto para português brasileiro (PT-BR).
 * MyMemory primeiro; fallback Google gtx se falhar.
 */

const CACHE_PREFIX = "zt-tr-ptbr-v2:";
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

/** Heurística: texto já parece português BR? */
export function looksLikePortuguese(text: string): boolean {
  const t = text.toLowerCase();
  const hits = (
    t.match(
      /\b(ção|ções|ões|não|você|vocês|também|está|estão|são|foi|após|através|estratégia|torneio|jogadores|matéria|dica)\b/g,
    ) || []
  ).length;
  const english = (
    t.match(/\b(the|and|with|from|this|that|have|will|your|when|what|how to|should|would|could)\b/g) ||
    []
  ).length;
  // Se tem inglês claro, não trate como PT
  if (english >= 2) return false;
  return hits >= 2;
}

async function translateChunkMyMemory(
  chunk: string,
  fromLang: "en" | "es" | "auto",
): Promise<string | undefined> {
  const pair = fromLang === "auto" ? "Autodetect|pt-BR" : `${fromLang}|pt-BR`;
  const url =
    "https://api.mymemory.translated.net/get?q=" +
    encodeURIComponent(chunk) +
    "&langpair=" +
    encodeURIComponent(pair);
  const res = await fetch(url, { signal: AbortSignal.timeout(12000) });
  if (!res.ok) return undefined;
  const data = (await res.json()) as {
    responseData?: { translatedText?: string };
    responseStatus?: number | string;
  };
  const out = data.responseData?.translatedText?.trim();
  if (!out) return undefined;
  if (Number(data.responseStatus) === 429) return undefined;
  if (out.toLowerCase() === chunk.toLowerCase()) return undefined;
  // MyMemory free às vezes devolve aviso em inglês
  if (/MYMEMORY WARNING/i.test(out)) return undefined;
  return out;
}

async function translateChunkGoogle(
  chunk: string,
  fromLang: "en" | "es" | "auto",
): Promise<string | undefined> {
  const sl = fromLang === "auto" ? "auto" : fromLang;
  const url =
    "https://translate.googleapis.com/translate_a/single?client=gtx&sl=" +
    encodeURIComponent(sl) +
    "&tl=pt&dt=t&q=" +
    encodeURIComponent(chunk);
  try {
    const res = await fetch(url, { signal: AbortSignal.timeout(12000) });
    if (!res.ok) return undefined;
    const data = (await res.json()) as unknown;
    // formato: [[["trad", "orig", ...], ...], ...]
    if (!Array.isArray(data) || !Array.isArray(data[0])) return undefined;
    const parts = (data[0] as Array<[string]>).map((row) => row?.[0] ?? "").join("");
    const out = parts.trim();
    if (!out || out.toLowerCase() === chunk.toLowerCase()) return undefined;
    return out;
  } catch {
    return undefined;
  }
}

async function translateChunk(chunk: string, fromLang: "en" | "es" | "auto"): Promise<string> {
  const a = await translateChunkMyMemory(chunk, fromLang);
  if (a) return a;
  const b = await translateChunkGoogle(chunk, fromLang);
  if (b) return b;
  return chunk;
}

/**
 * Traduz para português brasileiro.
 * @param force se true, traduz mesmo quando a heurística acha que já é PT
 */
export async function translateToPortuguese(
  text: string,
  fromLang: "en" | "es" | "auto" = "en",
  force = false,
): Promise<string> {
  const trimmed = text.trim();
  if (!trimmed) return text;
  if (!force && looksLikePortuguese(trimmed)) return text;

  const key = `${fromLang}:${force ? "f" : "n"}:${hashKey(trimmed)}`;
  const cached = cacheGet(key);
  if (cached) return cached;

  try {
    const parts: string[] = [];
    for (const chunk of chunkText(trimmed, CHUNK)) {
      parts.push(await translateChunk(chunk, fromLang));
      await new Promise((r) => setTimeout(r, 80));
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
  /** Força tradução (dicas EN/ES). */
  force?: boolean;
}): Promise<{ title: string; description?: string }> {
  const from = input.fromLang ?? "auto";
  const force = Boolean(input.force);
  const title = await translateToPortuguese(input.title, from, force);
  let description = input.description;
  if (description && description.trim()) {
    const clipped =
      description.length > 3000 ? `${description.slice(0, 3000).trim()}…` : description;
    description = await translateToPortuguese(clipped, from, force);
  }
  return { title, description };
}
