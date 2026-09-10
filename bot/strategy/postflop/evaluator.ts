// Avaliador de mao 5-7 cartas para o bot externo.
// Espelha a semantica do motor (Motor-Rust poker_engine::deck):
// straight com wheel (A-5), flush, pares/overs, draws (flush 9 outs,
// OESD 8, gutshot 4). Nao e solver: categorias + outs para decisao
// por pot odds. Formato de carta igual ao fio ("As", "Td").

import type { Card, Hand } from '../types.js';

export type MadeHand =
  | 'straight_flush' | 'quads' | 'full_house' | 'flush' | 'straight'
  | 'trips' | 'two_pair' | 'overpair' | 'top_pair' | 'middle_pair'
  | 'under_pair' | 'pocket_low' | 'high';

export interface Evaluation {
  category: MadeHand;
  /** Comparavel entre maos na MESMA mesa (maior = mais forte). */
  score: number;
  draws: string[];
  drawOuts: number;
  nutFlushBlocker: boolean;
  madeFlush: boolean;
}

const ORDER = '23456789TJQKA';

function rankOf(c: Card): number {
  return ORDER.indexOf(c[0]);
}

function suitOf(c: Card): string {
  return c[1];
}

function straightHigh(ranks: Set<number>): number {
  // Retorna o rank alto da maior sequencia de 5, ou -1. A joga baixo (wheel).
  const p = new Set(ranks);
  if (p.has(12)) p.add(-1);
  for (let hi = 12; hi >= 3; hi--) {
    let ok = true;
    for (let k = 0; k < 5; k++) {
      if (!p.has(hi - k)) { ok = false; break; }
    }
    if (ok) return hi;
  }
  return -1;
}

export function evaluate(hole: Hand, board: Card[]): Evaluation {
  const all = [...hole, ...board];
  const rankCount = new Map<number, number>();
  const suitRanks = new Map<string, number[]>();
  for (const c of all) {
    const r = rankOf(c);
    rankCount.set(r, (rankCount.get(r) ?? 0) + 1);
    const s = suitOf(c);
    if (!suitRanks.has(s)) suitRanks.set(s, []);
    suitRanks.get(s)!.push(r);
  }

  // Straight flush: 5+ do mesmo naipe em sequencia.
  let sfHigh = -1;
  for (const ranks of suitRanks.values()) {
    if (ranks.length >= 5) {
      const h = straightHigh(new Set(ranks));
      if (h > sfHigh) sfHigh = h;
    }
  }
  if (sfHigh >= 0) {
    return { category: 'straight_flush', score: 8e8 + sfHigh, draws: [], drawOuts: 0, nutFlushBlocker: true, madeFlush: true };
  }

  const quads = [...rankCount.entries()].filter(([, n]) => n >= 4).map(([r]) => r).sort((a, b) => b - a);
  if (quads.length > 0) {
    return { category: 'quads', score: 7e8 + quads[0], draws: [], drawOuts: 0, nutFlushBlocker: false, madeFlush: false };
  }

  const trips = [...rankCount.entries()].filter(([, n]) => n >= 3).map(([r]) => r).sort((a, b) => b - a);
  const pairs = [...rankCount.entries()].filter(([, n]) => n >= 2).map(([r]) => r).sort((a, b) => b - a);
  if (trips.length > 0 && pairs.some((p) => p !== trips[0])) {
    return { category: 'full_house', score: 6e8 + trips[0] * 13 + pairs.find((p) => p !== trips[0])!, draws: [], drawOuts: 0, nutFlushBlocker: false, madeFlush: false };
  }

  let flushSuit: string | null = null;
  let flushRanks: number[] = [];
  for (const [s, ranks] of suitRanks) {
    if (ranks.length >= 5) {
      const top = [...ranks].sort((a, b) => b - a).slice(0, 5);
      if (!flushSuit || top[0] > flushRanks[0]) { flushSuit = s; flushRanks = top; }
    }
  }
  if (flushSuit) {
    return { category: 'flush', score: 5e8 + flushRanks[0] * 1e4 + flushRanks[1] * 1e2, draws: [], drawOuts: 0, nutFlushBlocker: hole.some((c) => c[0] === 'A' && c[1] === flushSuit), madeFlush: true };
  }

  const stHigh = straightHigh(new Set(rankCount.keys()));
  if (stHigh >= 0) {
    return { category: 'straight', score: 4e8 + stHigh, draws: [], drawOuts: 0, nutFlushBlocker: false, madeFlush: false };
  }
  if (trips.length > 0) {
    return { category: 'trips', score: 3e8 + trips[0], draws: [], drawOuts: 0, nutFlushBlocker: false, madeFlush: false };
  }
  if (pairs.length >= 2) {
    return { category: 'two_pair', score: 2e8 + pairs[0] * 13 + pairs[1], draws: [], drawOuts: 0, nutFlushBlocker: false, madeFlush: false };
  }

  // Um par: classifica contra o board.
  const draws: string[] = [];
  let outs = 0;
  // Flush draw: 4 cartas do naipe (inclui hole).
  let flushDrawSuit: string | null = null;
  for (const [s, ranks] of suitRanks) {
    if (ranks.length === 4) { flushDrawSuit = s; break; }
  }
  if (flushDrawSuit) { draws.push('flush_draw'); outs += 9; }
  // Straight draws: melhor janela de 4 em 5.
  const present = new Set(rankCount.keys());
  if (present.has(12)) present.add(-1);
  let open = false;
  let gut = false;
  for (let hi = 12; hi >= 3; hi--) {
    let have = 0;
    for (let k = 0; k < 5; k++) if (present.has(hi - k)) have++;
    if (have === 4) {
      const low = hi - 4;
      const high = hi;
      const missingLow = !present.has(low);
      const missingHigh = !present.has(high);
      if (missingLow !== missingHigh) open = true;
      else gut = true;
    }
  }
  if (open) { draws.push('oesd'); outs += 8; }
  else if (gut) { draws.push('gutshot'); outs += 4; }

  const boardRanks = board.map(rankOf);
  const boardHigh = boardRanks.length ? Math.max(...boardRanks) : -1;
  const [h1, h2] = [rankOf(hole[0]), rankOf(hole[1])];
  const pocket = h1 === h2;

  if (pairs.length === 1) {
    const pr = pairs[0];
    const pairedBoard = boardRanks.includes(pr);
    const usesHole = hole.some((c) => rankOf(c) === pr);
    if (pocket && !pairedBoard) {
      if (h1 > boardHigh) return fin('overpair', 175e6 + h1, draws, outs, hole, board);
      if (h1 >= 9) return fin('middle_pair', 120e6 + h1, draws, outs, hole, board);
      return fin('under_pair', 100e6 + h1, draws, outs, hole, board);
    }
    if (usesHole) {
      if (pr === boardHigh) return fin('top_pair', 150e6 + pr, draws, outs, hole, board);
      if (pr >= 9) return fin('middle_pair', 120e6 + pr, draws, outs, hole, board);
      return fin('under_pair', 100e6 + pr, draws, outs, hole, board);
    }
    // Par so no board, hole nao joga: high com draws.
    return fin('high', 50e6 + Math.max(h1, h2), draws, outs, hole, board);
  }

  // Sem par: overcards valem outs (3 cada, teto 6) se nao contados.
  const over = [h1, h2].filter((r) => r > boardHigh).length;
  if (over > 0 && outs < 9) {
    draws.push('overcards');
    outs += over * 3;
  }
  if (pocket && h1 < 7) return fin('pocket_low', 60e6 + h1, draws, outs, hole, board);
  return fin('high', 50e6 + Math.max(h1, h2), draws, outs, hole, board);
}

function fin(
  category: Evaluation['category'], score: number, draws: string[],
  drawOuts: number, hole: Hand, board: Card[],
): Evaluation {
  // Blocker do nut flush: As do naipe com 3+ no board.
  let nutFlushBlocker = false;
  const boardSuits = new Map<string, number>();
  for (const c of board) boardSuits.set(suitOf(c), (boardSuits.get(suitOf(c)) ?? 0) + 1);
  for (const [s, n] of boardSuits) {
    if (n >= 3 && hole.some((c) => c[0] === 'A' && c[1] === s)) nutFlushBlocker = true;
  }
  return { category, score, draws, drawOuts: Math.min(15, drawOuts), nutFlushBlocker, madeFlush: false };
}

/** Equity estimada de showdown por categoria (conservadora, vs range medio). */
export function showdownEquity(e: Evaluation): number {
  switch (e.category) {
    case 'straight_flush': case 'quads': case 'full_house': return 0.95;
    case 'flush': case 'straight': return 0.85;
    case 'trips': return 0.75;
    case 'two_pair': return 0.65;
    case 'overpair': return 0.6;
    case 'top_pair': return 0.5;
    case 'middle_pair': case 'under_pair': return 0.35;
    case 'pocket_low': return 0.3;
    default: return 0.15;
  }
}
