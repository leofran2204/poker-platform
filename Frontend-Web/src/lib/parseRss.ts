/** Parse mínimo de RSS/Atom a partir de XML cru (fallback quando rss2json falha). */

export interface ParsedRssItem {
  title: string;
  link: string;
  pubDate: string;
  description: string;
  content: string;
  thumbnail?: string;
}

function textOf(el: Element | null): string {
  return (el?.textContent || "").trim();
}

function first(parent: Element, names: string[]): Element | null {
  for (const name of names) {
    const found = parent.getElementsByTagName(name);
    if (found.length > 0) return found[0];
  }
  return null;
}

function absUrl(href: string, base?: string): string {
  try {
    return new URL(href, base || "https://example.com").toString();
  } catch {
    return href;
  }
}

/**
 * Extrai itens de um documento RSS 2.0 ou Atom.
 */
export function parseRssXml(xml: string, feedUrl?: string): ParsedRssItem[] {
  const doc = new DOMParser().parseFromString(xml, "application/xml");
  if (doc.querySelector("parsererror")) return [];

  const items = Array.from(doc.getElementsByTagName("item"));
  const entries = items.length > 0 ? items : Array.from(doc.getElementsByTagName("entry"));
  const out: ParsedRssItem[] = [];

  for (const node of entries) {
    const title = textOf(first(node, ["title"]));
    let link = "";
    const linkEl = first(node, ["link"]);
    if (linkEl) {
      link = linkEl.getAttribute("href") || textOf(linkEl);
    }
    if (!link) {
      const guid = textOf(first(node, ["guid", "id"]));
      if (guid.startsWith("http")) link = guid;
    }
    link = absUrl(link, feedUrl);

    const pubDate =
      textOf(first(node, ["pubDate", "published", "updated", "dc:date"])) || new Date().toISOString();

    const description = textOf(first(node, ["description", "summary", "content:encoded", "content"]));
    const content = textOf(first(node, ["content:encoded", "content", "description"])) || description;

    // Só enclosure/media — NÃO a 1ª <img> do HTML (costuma ser sidebar/outra matéria)
    let thumbnail: string | undefined;
    const enclosure = first(node, ["enclosure", "media:content", "media:thumbnail"]);
    if (enclosure) {
      const encUrl = enclosure.getAttribute("url") || enclosure.getAttribute("href");
      if (encUrl && /\.(jpg|jpeg|png|webp)/i.test(encUrl)) thumbnail = absUrl(encUrl, feedUrl);
    }

    if (!title) continue;
    out.push({ title, link, pubDate, description, content, thumbnail });
  }

  return out;
}

/**
 * Alguns sites (ex.: Código Poker) bloqueiam o XML e só liberam via leitores.
 * O Jina devolve markdown com links — extraímos as matérias daí.
 */
export function parseJinaMarkdown(md: string): ParsedRssItem[] {
  const out: ParsedRssItem[] = [];
  const seen = new Set<string>();
  const re = /#{1,4}\s*\[([^\]]{8,200})\]\((https?:\/\/[^)\s]+)\)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(md)) !== null) {
    const title = m[1].replace(/\s+/g, " ").trim();
    const link = m[2].trim();
    if (!title || !link.startsWith("http")) continue;
    if (/bcpoker|isNewInstall|static\.|wp-content\/themes/i.test(link)) continue;
    if (seen.has(link)) continue;
    seen.add(link);
    out.push({
      title,
      link,
      pubDate: new Date().toISOString(),
      description: "",
      content: "",
    });
  }
  return out;
}
