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

/** Uma carta concreta: As, Kd, 10h — case-sensitive para não confundir com artigo “as”. */
const ONE_CARD = new RegExp(`^(${RANK})(${SUIT})$`);
/** Forma legada nos tips: A[s], 10[h]. */
const BRACKET_CARD = new RegExp(`^(${RANK})\\[(${SUIT})\\]$`);
/** Duas cartas coladas: AsKd, Ks7h. */
const TWO_CARDS = new RegExp(`^(${RANK})(${SUIT})(${RANK})(${SUIT})$`);

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

export type SeatPos = { top: number; left: number };

/** Layouts por cap: índice 0 = base (herói). Coordenadas em % do oval. */
const LAYOUT_5: SeatPos[] = [
  { top: 88, left: 50 },
  { top: 58, left: 10 },
  { top: 16, left: 22 },
  { top: 16, left: 78 },
  { top: 58, left: 90 },
];

const LAYOUT_6: SeatPos[] = [
  { top: 88, left: 50 },
  { top: 68, left: 12 },
  { top: 28, left: 12 },
  { top: 10, left: 50 },
  { top: 28, left: 88 },
  { top: 68, left: 88 },
];

const LAYOUT_8: SeatPos[] = [
  { top: 88, left: 50 },
  { top: 74, left: 16 },
  { top: 50, left: 6 },
  { top: 24, left: 16 },
  { top: 10, left: 50 },
  { top: 24, left: 84 },
  { top: 50, left: 94 },
  { top: 74, left: 84 },
];

const LAYOUT_9: SeatPos[] = [
  { top: 90, left: 50 },
  { top: 76, left: 16 },
  { top: 50, left: 6 },
  { top: 24, left: 16 },
  { top: 8, left: 38 },
  { top: 8, left: 62 },
  { top: 24, left: 84 },
  { top: 50, left: 94 },
  { top: 76, left: 84 },
];

export function layoutForCap(maxPlayers: number): SeatPos[] {
  if (maxPlayers <= 5) return LAYOUT_5;
  if (maxPlayers <= 6) return LAYOUT_6;
  if (maxPlayers <= 8) return LAYOUT_8;
  return LAYOUT_9;
}

/** Compat: 9-max antigo. Preferir `seatPosition`. */
export const SEAT_LAYOUT: SeatPos[] = LAYOUT_9;

/** Herói sempre no índice 0 (base). `heroSeat` = assento físico do jogador local. */
export function seatPosition(
  seat: number,
  heroSeat: number | null | undefined,
  maxPlayers: number,
): SeatPos {
  const layout = layoutForCap(maxPlayers);
  const n = layout.length;
  const hero = heroSeat ?? 0;
  const visual = ((seat - hero) % n + n) % n;
  return layout[visual] ?? layout[0];
}
