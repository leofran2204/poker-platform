export type Suit = "s" | "h" | "d" | "c";

export interface ParsedCard {
  rank: string;
  suit: Suit;
  red: boolean;
}

const SUIT_SYMBOL: Record<Suit, string> = {
  s: "♠",
  h: "♥",
  d: "♦",
  c: "♣",
};

/** Parse engine notation e.g. "As", "Td", "10h" */
export function parseCard(code: string): ParsedCard | null {
  if (!code || code.length < 2) return null;
  const suitChar = code.slice(-1).toLowerCase() as Suit;
  if (!"shdc".includes(suitChar)) return null;
  let rank = code.slice(0, -1).toUpperCase();
  if (rank === "10") rank = "T";
  if (rank === "T") rank = "10";
  return {
    rank: rank === "10" ? "10" : rank,
    suit: suitChar,
    red: suitChar === "h" || suitChar === "d",
  };
}

export function suitSymbol(suit: Suit): string {
  return SUIT_SYMBOL[suit];
}

const RANK = "A|K|Q|J|T|10|[2-9]";
const SUIT = "[shdc]";

/** Uma carta concreta: As, Kd, 10h, T d (sem espaços no código). */
const ONE_CARD = new RegExp(`^(${RANK})(${SUIT})$`, "i");
/** Forma legada nos tips: A[s], 10[h]. */
const BRACKET_CARD = new RegExp(`^(${RANK})\\[(${SUIT})\\]$`, "i");
/** Duas cartas coladas: AsKd, Ks7h. */
const TWO_CARDS = new RegExp(`^(${RANK})(${SUIT})(${RANK})(${SUIT})$`, "i");

/**
 * Extrai códigos de carta concretos de um token.
 * Não interpreta ranges (A9s, KTo, AJs+) — nesses casos devolve [].
 */
export function extractConcreteCardCodes(token: string): string[] {
  const t = token.replace(/[.,;:!?)]+$/g, "").replace(/^[("]+/g, "");
  if (!t) return [];

  const bracket = t.match(BRACKET_CARD);
  if (bracket) {
    const rank = bracket[1].toUpperCase() === "10" ? "T" : bracket[1].toUpperCase();
    return [`${rank}${bracket[2].toLowerCase()}`];
  }

  // Range / shorthand: segundo rank + s/o/+ → não é naipe concreto
  // Ex: A9s, A9s+, KTo, 98o, AJs+
  if (/^(A|K|Q|J|T|10|[2-9])(A|K|Q|J|T|10|[2-9])[so]\+?$/i.test(t)) return [];
  // Pares em range: 77, 77+, AA (não são "sete de espadas")
  if (/^(AA|KK|QQ|JJ|TT|99|88|77|66|55|44|33|22)\+?$/i.test(t)) return [];

  const two = t.match(TWO_CARDS);
  if (two) {
    const r1 = two[1].toUpperCase() === "10" ? "T" : two[1].toUpperCase();
    const r2 = two[3].toUpperCase() === "10" ? "T" : two[3].toUpperCase();
    return [`${r1}${two[2].toLowerCase()}`, `${r2}${two[4].toLowerCase()}`];
  }

  const one = t.match(ONE_CARD);
  if (one) {
    const rank = one[1].toUpperCase() === "10" ? "T" : one[1].toUpperCase();
    return [`${rank}${one[2].toLowerCase()}`];
  }

  return [];
}

/** Fixed 9-max seat layout as % of oval table (top/left). */
export const SEAT_LAYOUT: { top: number; left: number }[] = [
  { top: 88, left: 50 }, // 0 bottom (hero-ish)
  { top: 78, left: 18 },
  { top: 50, left: 6 },
  { top: 22, left: 18 },
  { top: 12, left: 50 },
  { top: 22, left: 82 },
  { top: 50, left: 94 },
  { top: 78, left: 82 },
  { top: 88, left: 72 },
];
