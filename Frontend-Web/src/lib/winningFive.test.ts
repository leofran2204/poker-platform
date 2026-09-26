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
  streets: { label: string; board: string[]; holes?: string[][] }[];
  seats?: { name: string; cards?: string[]; isWinner?: boolean; folded?: boolean }[];
  winningFive?: { hole: string[]; board: string[] };
}

/**
 * Ranking Texas Hold'em Short Deck (BUSINESS_RULES §2.6):
 * trinca > sequência e flush > full house. O `score5` acima usa o
 * ranking clássico (straight=4, trips=3, flush=5, full=6); aqui só
 * remapeamos a ordem das categorias — o desempate (tb) é o mesmo.
 */
function shortDeckCat(classicCat: number): number {
  const order: Record<number, number> = {
    0: 0, // carta alta
    1: 1, // par
    2: 2, // dois pares
    4: 3, // sequência passa para baixo da trinca
    3: 4, // trinca sobe
    6: 5, // full house passa para baixo do flush
    5: 6, // flush sobe
    7: 7, // quadra
    8: 8, // straight flush
  };
  return order[classicCat] ?? classicCat;
}

function cmpShortDeck(a: Scored, b: Scored): number {
  const d = shortDeckCat(a.cat) - shortDeckCat(b.cat);
  if (d !== 0) return d;
  return cmpScore({ ...a, cat: 0 }, { ...b, cat: 0 });
}

/** Melhor jogo de 5 com a regra da modalidade. */
function bestFiveRuled(
  hole: string[],
  board: string[],
  exactTwo: boolean,
  shortDeck: boolean,
): Scored {
  let best: Scored | null = null;
  const consider = (cards: string[]) => {
    const s = score5(cards);
    if (!best || (shortDeck ? cmpShortDeck(s, best) : cmpScore(s, best)) > 0) best = s;
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
        ...(h.seats?.flatMap((seat) => seat.cards ?? []) ?? []),
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

  it("pineapple-demo: usa exatamente 2 da mão + 3 da mesa", () => {
    const h = variantHands.find((x) => x.id === "pineapple-demo")!;
    expect(h.heroCards).toHaveLength(5);
    expect(h.winningFive!.hole).toHaveLength(2);
    expect(h.winningFive!.board).toHaveLength(3);
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

  it("variantes: vencedor declarado vence todos com a regra da modalidade", () => {
    for (const h of variantHands) {
      const seats = h.seats;
      // Demos sem assentos (herói joga contra a mesa) não têm adversário.
      if (!seats || seats.length === 0) continue;
      const shortDeck = h.id === "short-deck-demo";
      const board = h.streets[h.streets.length - 1].board;
      const w = seats.find((s) => s.isWinner)!;
      expect(w.cards).toBeDefined();
      const wBest = bestFiveRuled(w.cards!, board, w.cards!.length > 2, shortDeck);
      for (const o of seats) {
        if (o.isWinner || o.folded || !o.cards) continue;
        const oBest = bestFiveRuled(o.cards, board, o.cards.length > 2, shortDeck);
        const cmp = shortDeck ? cmpShortDeck(wBest, oBest) : cmpScore(wBest, oBest);
        expect(cmp > 0, `${h.id}: ${w.name} deve vencer ${o.name}`).toBe(true);
      }
    }
  });

  it("pineapple-demo: sequência da Mari vence a trinca do herói (ranking clássico)", () => {
    const h = variantHands.find((x) => x.id === "pineapple-demo")!;
    const board = h.streets[h.streets.length - 1].board;
    const hero = h.seats!.find((s) => s.isHero)!;
    const mari = h.seats!.find((s) => s.isWinner)!;
    const heroBest = bestFiveRuled(hero.cards!, board, true, false);
    const mariBest = bestFiveRuled(mari.cards!, board, true, false);
    expect(cmpScore(mariBest, heroBest) > 0).toBe(true);
  });

  it("mãos com assentos: distribuição progressiva termina nas cartas dos assentos", () => {
    for (const h of [...deflatorHands, ...variantHands]) {
      const seats = h.seats;
      const withHoles = h.streets.filter((s) => s.holes);
      if (!seats || withHoles.length === 0) continue;
      const lastHoles = withHoles[withHoles.length - 1].holes!;
      expect(lastHoles).toEqual(seats.map((s) => s.cards ?? []));
      // Ninguém perde carta no caminho: 2 no pré-flop, +1 por street.
      for (const s of withHoles) {
        s.holes!.forEach((hole, i) => {
          const prev = withHoles[0].holes![i] ?? [];
          expect(hole.slice(0, prev.length)).toEqual(prev);
        });
      }
    }
  });
});
