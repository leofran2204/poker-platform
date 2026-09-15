import { describe, expect, it } from "vitest";
import homeContent from "@/data/homeContent.json";

/**
 * Garante que o `winningFive` de cada replay é exatamente o melhor jogo
 * de 5 cartas do vencedor — é ele que brilha no showdown (resto escurece).
 * Omaha/Pineapple respeitam a regra exata de 2 da mão + 3 da mesa.
 */

const RANK: Record<string, number> = {
  "2": 2, "3": 3, "4": 4, "5": 5, "6": 6, "7": 7, "8": 8,
  "9": 9, T: 10, J: 11, Q: 12, K: 13, A: 14,
};

interface Scored {
  cat: number;
  tb: number[];
  cards: string[];
}

/** Comparação numérica (nunca lexicográfica: "11" < "7" como string). */
function cmpScore(a: Scored, b: Scored): number {
  if (a.cat !== b.cat) return a.cat - b.cat;
  const n = Math.max(a.tb.length, b.tb.length);
  for (let i = 0; i < n; i++) {
    const d = (a.tb[i] ?? -1) - (b.tb[i] ?? -1);
    if (d !== 0) return d;
  }
  return 0;
}

function score5(cards: string[]): Scored {
  const ps = cards
    .map((c) => ({ r: RANK[c[0]], s: c[1] }))
    .sort((a, b) => b.r - a.r);
  const rs = ps.map((p) => p.r);
  const flush = ps.every((p) => p.s === ps[0].s);
  let ok = true;
  for (let i = 1; i < 5; i++) {
    if (rs[i] !== rs[0] - i) {
      ok = false;
      break;
    }
  }
  if (!ok && rs.join() === "14,5,4,3,2") ok = true;
  const cnt: Record<number, number> = {};
  rs.forEach((r) => {
    cnt[r] = (cnt[r] ?? 0) + 1;
  });
  const groups = Object.entries(cnt)
    .map(([r, n]) => ({ r: Number(r), n }))
    .sort((a, b) => b.n - a.n || b.r - a.r);
  const tb = groups.flatMap((g) => Array(g.n).fill(g.r));
  let cat: number;
  if (flush && ok) cat = 8;
  else if (groups[0].n === 4) cat = 7;
  else if (groups[0].n === 3 && groups[1]?.n === 2) cat = 6;
  else if (flush) cat = 5;
  else if (ok) cat = 4;
  else if (groups[0].n === 3) cat = 3;
  else if (groups[0].n === 2 && groups[1]?.n === 2) cat = 2;
  else if (groups[0].n === 2) cat = 1;
  else cat = 0;
  return { cat, tb, cards: [...cards].sort() };
}

function combos<T>(arr: T[], k: number): T[][] {
  if (k === 0) return [[]];
  if (arr.length < k) return [];
  const [head, ...tail] = arr;
  return [
    ...combos(tail, k - 1).map((c) => [head, ...c]),
    ...combos(tail, k),
  ];
}

/** Melhor jogo de 5; exactTwo = regra 2 da mão + 3 da mesa (Omaha/Pineapple). */
function bestFive(hole: string[], board: string[], exactTwo: boolean): Scored {
  let best: Scored | null = null;
  const consider = (cards: string[]) => {
    const s = score5(cards);
    if (!best || cmpScore(s, best) > 0) best = s;
  };
  if (!exactTwo) {
    for (const c of combos([...hole, ...board], 5)) consider(c);
  } else {
    for (const h of combos(hole, 2)) {
      for (const b of combos(board, 3)) consider([...h, ...b]);
    }
  }
  if (!best) throw new Error("sem combinação");
  return best;
}

interface DemoHand {
  id: string;
  heroCards: string[];
  villainCards: string[];
  streets: { label: string; board: string[] }[];
  seats?: { name: string; cards?: string[]; isWinner?: boolean }[];
  winningFive?: { hole: string[]; board: string[] };
}

const deflatorHands = homeContent.deflator.demoHands as DemoHand[];
const variantHands = (homeContent.variants as { demoHand: DemoHand }[]).map(
  (v) => v.demoHand,
);

function winnerHole(h: DemoHand): string[] {
  const w = h.seats?.find((s) => s.isWinner);
  if (w?.cards) return w.cards;
  // Legado/variantes: herói vence.
  return h.heroCards;
}

describe("winningFive — as 5 do jogo vencedor", () => {
  for (const h of [...deflatorHands, ...variantHands]) {
    it(`${h.id}: 5 cartas únicas dentro de hole+board`, () => {
      expect(h.winningFive).toBeDefined();
      const five = [...h.winningFive!.hole, ...h.winningFive!.board];
      expect(five).toHaveLength(5);
      expect(new Set(five).size).toBe(5);
      const pool = new Set([
        ...h.heroCards,
        ...h.villainCards,
        ...h.streets[h.streets.length - 1].board,
      ]);
      for (const c of five) expect(pool.has(c)).toBe(true);
    });

    it(`${h.id}: é o melhor jogo legal do vencedor`, () => {
      const hole = winnerHole(h);
      const board = h.streets[h.streets.length - 1].board;
      const exactTwo = hole.length > 2;
      const best = bestFive(hole, board, exactTwo);
      expect(best.cards).toEqual(
        [...h.winningFive!.hole, ...h.winningFive!.board].sort(),
      );
    });
  }

  it("shortdeck-demo: full house do vilão perde só pela regra do Short Deck", () => {
    const h = variantHands.find((x) => x.id === "shortdeck-demo")!;
    const board = h.streets[h.streets.length - 1].board;
    const heroBest = bestFive(h.heroCards, board, false);
    const vilBest = bestFive(h.villainCards, board, false);
    // No ranking clássico o full house bateria o flush.
    expect(cmpScore(vilBest, heroBest) > 0).toBe(true);
  });

  it("deflator: vencedor declarado vence no ranking clássico", () => {
    for (const h of deflatorHands) {
      const board = h.streets[h.streets.length - 1].board;
      const seats = h.seats!;
      const w = seats.find((s) => s.isWinner)!;
      const loser = seats.find((s) => !s.isWinner && !s.folded && s.cards)!;
      const wBest = bestFive(w.cards!, board, false);
      const lBest = bestFive(loser.cards!, board, false);
      expect(cmpScore(wBest, lBest) > 0).toBe(true);
    }
  });
});
