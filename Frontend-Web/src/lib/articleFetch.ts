/** Busca texto e imagem oficiais da página da matéria. */

const PAGE_PROXY = "https://api.allorigins.win/raw?url=";

function isAdBanner(url: string): boolean {
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
    if (isAdBanner(url)) continue;
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

function extractArticleHtml(html: string): string {
  const candidates = [
    html.match(/<article[^>]*>([\s\S]*?)<\/article>/i)?.[1],
    html.match(/class=["'][^"']*(?:entry-content|post-content|article-content|td-post-content|content-inner)[^"']*["'][^>]*>([\s\S]*?)<\/div>/i)?.[1],
    html.match(/<main[^>]*>([\s\S]*?)<\/main>/i)?.[1],
  ];
  for (const c of candidates) {
    if (c && c.length > 200) return c;
  }
  return html;
}

async function fetchHtml(articleUrl: string): Promise<string | undefined> {
  // 1) allorigins
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

  // 2) jina (bom contra 403/cloudflare)
  try {
    const res = await fetch(`https://r.jina.ai/${articleUrl}`, {
      signal: AbortSignal.timeout(20000),
      headers: { Accept: "text/plain" },
    });
    if (!res.ok) return undefined;
    const text = await res.text();
    // Jina markdown — devolve como "html" lógico via plain
    return text;
  } catch {
    return undefined;
  }
}

export async function resolveSourceImage(articleUrl: string): Promise<string | undefined> {
  const html = await fetchHtml(articleUrl);
  if (!html) return undefined;

  // HTML meta
  const metaCandidates = [
    html.match(/property=["']og:image["'][^>]*content=["']([^"']+)["']/i)?.[1],
    html.match(/content=["']([^"']+)["'][^>]*property=["']og:image["']/i)?.[1],
    html.match(/name=["']twitter:image["'][^>]*content=["']([^"']+)["']/i)?.[1],
    html.match(/content=["']([^"']+)["'][^>]*name=["']twitter:image["']/i)?.[1],
  ];
  for (const og of metaCandidates) {
    if (!og) continue;
    const abs = og.startsWith("http") ? og : absolutize(og, articleUrl);
    if (abs && !isAdBanner(abs)) return abs;
  }

  const fromBody = extractImagesFromHtml(html, articleUrl);
  if (fromBody[0]) return fromBody[0];

  // Jina markdown image
  const mdImg = html.match(/!\[[^\]]*\]\((https?:\/\/[^)\s]+\.(?:jpg|jpeg|png|webp)[^)\s]*)\)/i);
  if (mdImg && !isAdBanner(mdImg[1])) return mdImg[1];

  return undefined;
}

export async function resolveArticleBody(articleUrl: string): Promise<string | undefined> {
  const raw = await fetchHtml(articleUrl);
  if (!raw) return undefined;

  // Se veio markdown do Jina
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
