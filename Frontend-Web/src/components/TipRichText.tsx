import { Fragment, type ReactNode } from "react";
import { extractConcreteCardCodes } from "@/lib/cards";
import { PlayingCard } from "./PlayingCard";

/**
 * Render tip/news body with concrete hole/board cards as mini PlayingCards
 * (same look as the table). Range shorthand (A9s+, KTo) stays as text.
 */
export function TipRichText({
  text,
  className,
}: {
  text: string;
  className?: string;
}) {
  const lines = text.split("\n");

  return (
    <div className={className}>
      {lines.map((line, li) => (
        <Fragment key={li}>
          {li > 0 && <br />}
          {line.length === 0 ? null : renderLine(line, li)}
        </Fragment>
      ))}
    </div>
  );
}

function renderLine(line: string, lineKey: number): ReactNode {
  // Split keeping whitespace; also split punctuation glued to tokens lightly
  const parts = line.split(/(\s+)/);
  const nodes: ReactNode[] = [];

  parts.forEach((part, i) => {
    if (!part) return;
    if (/^\s+$/.test(part)) {
      nodes.push(part);
      return;
    }

    // Trailing punctuation: As. → card + "."
    const m = part.match(/^([("'[]*)(.*?)([.,;:!?)\]}"]*)$/);
    const lead = m?.[1] ?? "";
    const core = m?.[2] ?? part;
    const trail = m?.[3] ?? "";

    const codes = extractConcreteCardCodes(core);
    if (codes.length > 0) {
      if (lead) nodes.push(lead);
      nodes.push(
        <span
          key={`${lineKey}-${i}`}
          className="inline-flex items-center gap-0.5 align-middle"
        >
          {codes.map((code, ci) => (
            <PlayingCard key={`${code}-${ci}`} code={code} size="xs" inline />
          ))}
        </span>,
      );
      if (trail) nodes.push(trail);
      return;
    }

    // Forma no meio da frase ainda com brackets: Board K[s] 7[d]
    const withBrackets = splitBracketCards(part, `${lineKey}-${i}`);
    if (withBrackets) {
      nodes.push(...withBrackets);
      return;
    }

    nodes.push(part);
  });

  return nodes;
}

/** Substitui A[s] / 10[h] no meio de um token maior (ex.: "K[s],"). */
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
