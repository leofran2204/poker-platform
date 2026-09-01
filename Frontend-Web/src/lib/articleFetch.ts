/** Busca texto e imagem oficiais da página da matéria. */

const PAGE_PROXY = "https://api.allorigins.win/raw?url=";

/** Banner/anúncio óbvio. */
export function isAdBanner(url: string): boolean {
  const u = url.toLowerCase();
  return (
    u.includes("820x100") ||
    u.includes("/ads/") ||
    u.includes("favicon") ||
    u.includes("emoji") ||
    u.includes("gravatar") ||
    u.includes("wp-content/uploads/2026/05/820x100") ||
    u.endsWith(".svg")
  );
}

/**
 * Logo / capa padrão de site / placeholder — não devem ser capa de matéria
 * (causa “mesma foto em várias notícias” e “foto de outro jogador”).
 */
export function isGenericSiteImage(url: string): boolean {
  const u = url.toLowerCase();
  return (
    isAdBanner(u) ||
    /\/(logo|logos|site-icon|siteicon|brand|branding)\b/.test(u) ||
    /\b(logo|favicon|site-icon|apple-touch|default[-_]?image|placeholder|avatar|sprite|watermark)\b/.test(
      u,
    ) ||
    /\/wp-content\/themes\//.test(u) ||
    /\/wp-includes\//.test(u) ||
    /cropped-logo/.test(u) ||
    /\/i\/logo/.test(u) ||
    /og[-_]?default/.test(u) ||
    /default[-_]?og/.test(u) ||
    /\/images\/default\//.test(u) ||
    /\/static\/.*logo/.test(u)
  );
}

/** Imagem aceitável como capa de um card. */
export function isAcceptableCoverImage(url: string | undefined | null): url is string {
  if (!url || !/^https?:\/\//i.test(url)) return false;
  return !isGenericSiteImage(url);
}

/** Normaliza URL para dedupe entre cards (ignora query/hash). */
export function normalizeImageUrl(url: string): string {
  try {
    const u = new URL(url);
    u.hash = "";
    u.search = "";
    // host lowercase; path as-is except trailing slash
    let path = u.pathname.replace(/\/+$/, "") || "/";
    return `${u.protocol}//${u.host.toLowerCase()}${path}`;
  } catch {
    return url.split(/[?#]/)[0].toLowerCase();
  }
}

function absolutize(url: string, base: string): string | undefined {
  try {
    return new URL(url, base).toString();
  } catch {
    return undefined;
  }
}

export function extractImagesFromHtml(html: string, baseUrl?: string): string[] {
  const found: string[] = [];
  const srcRe = /(?:src|data-src|data-lazy-src)=["']([^"']+\.(?:jpg|jpeg|png|webp)[^"']*)["']/gi;
  let m: RegExpExecArray | null;
  while ((m = srcRe.exec(html)) !== null) {
    const raw = m[1];
    const abs = raw.startsWith("http") ? raw : baseUrl ? absolutize(raw, baseUrl) : undefined;
    if (abs) found.push(abs);
  }
  const unique: string[] = [];
  for (const url of found) {
    if (!isAcceptableCoverImage(url)) continue;
    if (!unique.includes(url)) unique.push(url);
  }
  return unique;
}

function stripNoiseHtml(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, " ")
    .replace(/<style[\s\S]*?<\/style>/gi, " ")
    .replace(/<noscript[\s\S]*?<\/noscript>/gi, " ")
    .replace(/<!--[\s\S]*?-->/g, " ");
}

export function htmlToPlainText(html: string): string {
  const clean = stripNoiseHtml(html)
    .replace(/<\/(p|div|h[1-6]|li|br|tr)>/gi, "\n")
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<[^>]+>/g, " ")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
  return clean;
}

/** Preserva headings, listas e tabelas em Markdown para leitura de iniciante. */
export function htmlToStructuredMarkdown(html: string): string {
  let s = stripNoiseHtml(html);
  // headings → ##
  s = s.replace(/<h[1-6][^>]*>\s*/gi, "\n## ");
  s = s.replace(/<\/h[1-6]>/gi, "\n");
  // listas
  s = s.replace(/<li[^>]*>\s*/gi, "\n- ");
  s = s.replace(/<\/li>/gi, "");
  s = s.replace(/<\/(p|div|br|tr)>/gi, "\n");
  s = s.replace(/<br\s*\/?>/gi, "\n");
  // tabelas: mantém pipes
  s = s.replace(/<\/tr>/gi, "\n");
  s = s.replace(/<\/td>/gi, " | ");
  s = s.replace(/<\/th>/gi, " | ");
  s = s.replace(/<tr[^>]*>/gi, "\n| ");
  // remove tags restantes
  s = s.replace(/<[^>]+>/g, " ");
  s = s
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
  // limpa pipes duplicados
  s = s.replace(/\|\s+\|/g, "| |");
  return s;
}

function extractArticleHtml(html: string): string {
  const candidates = [
    html.match(/<article[^>]*>([\s\S]*?)<\/article>/i)?.[1],
    html.match(
      /class=["'][^"']*(?:entry-content|post-content|article-content|td-post-content|content-inner)[^"']*["'][^>]*>([\s\S]*?)<\/div>/i,
    )?.[1],
    html.match(/<main[^>]*>([\s\S]*?)<\/main>/i)?.[1],
  ];
  for (const c of candidates) {
    if (c && c.length > 200) return c;
  }
  return html;
}

async function fetchHtml(articleUrl: string): Promise<string | undefined> {
  try {
    const res = await fetch(`${PAGE_PROXY}${encodeURIComponent(articleUrl)}`, {
      signal: AbortSignal.timeout(12000),
    });
    if (res.ok) {
      const html = await res.text();
      if (html.length > 500 && !html.includes("error code: 522")) return html;
    }
  } catch {
    /* fallback */
  }

  try {
    const res = await fetch(`https://r.jina.ai/${articleUrl}`, {
      signal: AbortSignal.timeout(20000),
      headers: { Accept: "text/plain" },
    });
    if (!res.ok) return undefined;
    return await res.text();
  } catch {
    return undefined;
  }
}

/**
 * Tokens do título/slug que ajudam a casar com o filename da imagem
 * (ex.: “neuville” no path). Evita manter capa genérica duplicada.
 */
export function titleImageAffinity(title: string, link: string, imageUrl: string): number {
  const img = normalizeImageUrl(imageUrl);
  const imgPath = img.replace(/^https?:\/\//, "");
  const corpus = `${title} ${link}`
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "");
  const tokens = corpus
    .split(/[^a-z0-9]+/)
    .filter((t) => t.length >= 5)
    .filter((t) => !/^(https|http|www|noticias|news|poker|brasil|mundo|com|br|the|para|como|com|sobre)$/.test(t));
  let score = 0;
  for (const t of tokens) {
    if (imgPath.includes(t)) score += t.length;
  }
  return score;
}

/**
 * Resolve capa oficial da matéria.
 * Prioriza og/twitter; NÃO usa a 1ª img aleatória do body do RSS (fonte comum de mismatch).
 * Imagens do corpo só como último recurso e já filtradas.
 */
export async function resolveSourceImage(articleUrl: string): Promise<string | undefined> {
  const html = await fetchHtml(articleUrl);
  if (!html) return undefined;

  const metaCandidates = [
    html.match(/property=["']og:image["'][^>]*content=["']([^"']+)["']/i)?.[1],
    html.match(/content=["']([^"']+)["'][^>]*property=["']og:image["']/i)?.[1],
    html.match(/name=["']twitter:image["'][^>]*content=["']([^"']+)["']/i)?.[1],
    html.match(/content=["']([^"']+)["'][^>]*name=["']twitter:image["']/i)?.[1],
  ];
  for (const og of metaCandidates) {
    if (!og) continue;
    const abs = og.startsWith("http") ? og : absolutize(og, articleUrl);
    if (abs && isAcceptableCoverImage(abs)) return abs;
  }

  // Corpo da matéria: só se houver poucas imagens “boas” (evita sidebar)
  const fromBody = extractImagesFromHtml(html, articleUrl);
  if (fromBody.length === 1) return fromBody[0];
  if (fromBody.length > 1) {
    // Preferir a que mais casa com o slug da URL
    let best = fromBody[0];
    let bestScore = titleImageAffinity("", articleUrl, best);
    for (const cand of fromBody.slice(1)) {
      const s = titleImageAffinity("", articleUrl, cand);
      if (s > bestScore) {
        best = cand;
        bestScore = s;
      }
    }
    // Só usa body se houver afinidade mínima com o slug
    if (bestScore >= 5) return best;
  }

  const mdImg = html.match(/!\[[^\]]*\]\((https?:\/\/[^)\s]+\.(?:jpg|jpeg|png|webp)[^)\s]*)\)/i);
  if (mdImg && isAcceptableCoverImage(mdImg[1])) return mdImg[1];

  return undefined;
}

export async function resolveArticleBody(articleUrl: string): Promise<string | undefined> {
  const raw = await fetchHtml(articleUrl);
  if (!raw) return undefined;

  if (/^Title:|Markdown Content:/m.test(raw) || (!raw.includes("<html") && raw.includes("]("))) {
    const md = raw
      .replace(/^Title:.*$/m, "")
      .replace(/^URL Source:.*$/m, "")
      .replace(/^Published Time:.*$/m, "")
      .replace(/^Markdown Content:\s*/m, "")
      .replace(/!\[[^\]]*\]\([^)]+\)/g, "")
      .replace(/\[[^\]]*\]\(([^)]+)\)/g, "$1")
      .replace(/^#{1,6}\s+/gm, "")
      .replace(/\n{3,}/g, "\n\n")
      .trim();
    return md.length > 80 ? md.slice(0, 12000) : undefined;
  }

  const articleHtml = extractArticleHtml(raw);
  const text = htmlToPlainText(articleHtml);
  if (text.length < 80) return undefined;
  return text.slice(0, 12000);
}

/**
 * Remove capas repetidas entre cards.
 * - URL usada 1×: mantém
 * - URL usada 2+: mantém só no item com maior afinidade título↔filename;
 *   se ninguém tiver afinidade ≥ 5, remove de todos (fallback tema).
 */
export function dedupeCoverImages<T extends { title: string; link: string; imageUrl?: string; imageCaption?: string }>(
  items: T[],
): T[] {
  const groups = new Map<string, number[]>();
  items.forEach((item, idx) => {
    if (!item.imageUrl || !isAcceptableCoverImage(item.imageUrl)) {
      item.imageUrl = undefined;
      item.imageCaption = undefined;
      return;
    }
    const key = normalizeImageUrl(item.imageUrl);
    const list = groups.get(key) ?? [];
    list.push(idx);
    groups.set(key, list);
  });

  for (const [, idxs] of groups) {
    if (idxs.length < 2) continue;
    let bestIdx = idxs[0];
    let bestScore = -1;
    for (const i of idxs) {
      const it = items[i];
      const score = titleImageAffinity(it.title, it.link, it.imageUrl!);
      if (score > bestScore) {
        bestScore = score;
        bestIdx = i;
      }
    }
    for (const i of idxs) {
      if (i === bestIdx && bestScore >= 5) continue;
      items[i].imageUrl = undefined;
      items[i].imageCaption = undefined;
    }
    // Se o “melhor” também não casou com o título, limpa ele também
    if (bestScore < 5) {
      items[bestIdx].imageUrl = undefined;
      items[bestIdx].imageCaption = undefined;
    }
  }
  return items;
}
