import { Fragment, type ReactNode } from "react";
import { extractConcreteCardCodes } from "@/lib/cards";
import { PlayingCard } from "./PlayingCard";

/**
 * Render tip/news body com suporte a Markdown lite para iniciantes:
 * - ## Heading → <h4>
 * - | tabela | → <table>
 * - - lista → <ul>
 * - Cartas concretas (As, Kh) → mini PlayingCard
 * Mantém compatível com textos antigos sem markdown.
 */
export function TipRichText({
  text,
  className,
}: {
  text: string;
  className?: string;
}) {
  const blocks = parseMarkdownBlocks(text);

  return (
    <div className={className}>
      {blocks.map((block, idx) => (
        <Fragment key={idx}>{renderBlock(block, idx)}</Fragment>
      ))}
    </div>
  );
}

type Block =
  | { type: "heading"; content: string }
  | { type: "table"; rows: string[][] }
  | { type: "list"; items: string[]; ordered: boolean }
  | { type: "paragraph"; content: string };

function parseMarkdownBlocks(text: string): Block[] {
  const lines = text.split("\n");
  const blocks: Block[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i].trim();
    if (!line) {
      i += 1;
      continue;
    }
    // Heading ## or ###
    if (/^#{2,4}\s+/.test(line)) {
      blocks.push({ type: "heading", content: line.replace(/^#{2,4}\s+/, "").trim() });
      i += 1;
      continue;
    }
    // Table: consecutive lines starting with |
    if (line.startsWith("|")) {
      const rows: string[][] = [];
      while (i < lines.length && lines[i].trim().startsWith("|")) {
        const cols = lines[i]
          .split("|")
          .map((c) => c.trim())
          .filter(Boolean);
        // ignora linha separadora |---|---|
        if (cols.length > 0 && !cols.every((c) => /^[-:]+$/.test(c))) {
          rows.push(cols);
        }
        i += 1;
      }
      if (rows.length > 0) blocks.push({ type: "table", rows });
      continue;
    }
    // Lista - ou 1.
    if (/^[-*]\s+/.test(line) || /^\d+\.\s+/.test(line)) {
      const items: string[] = [];
      const ordered = /^\d+\.\s+/.test(line);
      while (i < lines.length && (/^[-*]\s+/.test(lines[i].trim()) || /^\d+\.\s+/.test(lines[i].trim()))) {
        const item = lines[i].trim().replace(/^[-*]\s+/, "").replace(/^\d+\.\s+/, "").trim();
        items.push(item);
        i += 1;
      }
      blocks.push({ type: "list", items, ordered });
      continue;
    }
    // Parágrafo: junta linhas seguintes que não são bloco especial até linha vazia
    let para = line;
    i += 1;
    while (i < lines.length && lines[i].trim() && !/^#{2,4}\s+/.test(lines[i].trim()) && !lines[i].trim().startsWith("|") && !/^[-*]\s+/.test(lines[i].trim()) && !/^\d+\.\s+/.test(lines[i].trim())) {
      para += " " + lines[i].trim();
      i += 1;
    }
    blocks.push({ type: "paragraph", content: para });
  }
  return blocks;
}

function renderBlock(block: Block, key: number): ReactNode {
  switch (block.type) {
    case "heading":
      return (
        <h4 key={key} className="mt-4 first:mt-0 text-sm font-bold uppercase tracking-wide text-gold-bright">
          {renderInline(block.content, `${key}-h`)}
        </h4>
      );
    case "table":
      return (
        <div key={key} className="my-3 overflow-x-auto rounded border border-felt-600">
          <table className="w-full text-left text-xs">
            <thead>
              <tr className="bg-felt-700 text-gold-soft">
                {block.rows[0]?.map((cell, ci) => (
                  <th key={ci} className="px-2 py-1.5 font-semibold whitespace-nowrap">
                    {renderInline(cell, `${key}-th-${ci}`)}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.slice(1).map((row, ri) => (
                <tr key={ri} className="border-t border-felt-600 odd:bg-felt-800/40">
                  {row.map((cell, ci) => (
                    <td key={ci} className="px-2 py-1.5 whitespace-nowrap text-felt-100">
                      {renderInline(cell, `${key}-td-${ri}-${ci}`)}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    case "list":
      return block.ordered ? (
        <ol key={key} className="my-2 list-decimal space-y-1 pl-5 text-sm leading-relaxed">
          {block.items.map((it, idx) => (
            <li key={idx}>{renderInline(it, `${key}-li-${idx}`)}</li>
          ))}
        </ol>
      ) : (
        <ul key={key} className="my-2 list-disc space-y-1 pl-5 text-sm leading-relaxed">
          {block.items.map((it, idx) => (
            <li key={idx}>{renderInline(it, `${key}-li-${idx}`)}</li>
          ))}
        </ul>
      );
    case "paragraph":
      return (
        <p key={key} className="mt-2 first:mt-0 text-sm leading-relaxed">
          {renderInline(block.content, `${key}-p`)}
        </p>
      );
  }
}

function renderInline(text: string, keyPrefix: string): ReactNode {
  // usa lógica anterior de PlayingCard mas agora por bloco
  const parts = text.split(/(\s+)/);
  const nodes: ReactNode[] = [];
  parts.forEach((part, i) => {
    if (!part) return;
    if (/^\s+$/.test(part)) {
      nodes.push(part);
      return;
    }
    const m = part.match(/^([("'[]*)(.*?)([.,;:!?)\]}"]*)$/);
    const lead = m?.[1] ?? "";
    const core = m?.[2] ?? part;
    const trail = m?.[3] ?? "";
    const codes = extractConcreteCardCodes(core);
    if (codes.length > 0) {
      if (lead) nodes.push(lead);
      nodes.push(
        <span key={`${keyPrefix}-${i}`} className="inline-flex items-center gap-0.5 align-middle">
          {codes.map((code, ci) => (
            <PlayingCard key={`${code}-${ci}`} code={code} size="xs" inline />
          ))}
        </span>,
      );
      if (trail) nodes.push(trail);
      return;
    }
    const withBrackets = splitBracketCards(part, `${keyPrefix}-${i}`);
    if (withBrackets) {
      nodes.push(...withBrackets);
      return;
    }
    // **bold** → <strong>
    if (part.includes("**")) {
      const boldParts = part.split(/\*\*(.+?)\*\*/g);
      boldParts.forEach((bp, bi) => {
        if (bi % 2 === 1) {
          nodes.push(
            <strong key={`${keyPrefix}-${i}-b${bi}`} className="font-semibold text-cream">
              {bp}
            </strong>,
          );
        } else if (bp) {
          nodes.push(bp);
        }
      });
      return;
    }
    nodes.push(part);
  });
  return <>{nodes}</>;
}

function splitBracketCards(token: string, key: string): ReactNode[] | null {
  const re = /(A|K|Q|J|T|10|[2-9])\[([shdc])\]/gi;
  if (!re.test(token)) return null;
  re.lastIndex = 0;
  const out: ReactNode[] = [];
  let last = 0;
  let m: RegExpExecArray | null;
  let n = 0;
  while ((m = re.exec(token)) !== null) {
    if (m.index > last) out.push(token.slice(last, m.index));
    const rank = m[1].toUpperCase() === "10" ? "T" : m[1].toUpperCase();
    const code = `${rank}${m[2].toLowerCase()}`;
    out.push(
      <span key={`${key}-b${n}`} className="inline-flex items-center align-middle">
        <PlayingCard code={code} size="xs" inline />
      </span>,
    );
    last = m.index + m[0].length;
    n += 1;
  }
  if (last < token.length) out.push(token.slice(last));
  return out;
}
