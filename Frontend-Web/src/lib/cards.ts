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
