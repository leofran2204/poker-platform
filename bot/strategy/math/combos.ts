// Parsing de notacao de range (ex: "22+, A2s+, K9s+, QTs+, JTs, ATo+, KJo").
// Suporta: pares ("22", "TT+"), suited ("A2s+"), offsuit ("ATo+"),
// maos exatas ("KQs", "QJo", "T9s"), listas separadas por virgula,
// prefixos "raise:"/"limp:" ignorados, e faixas percentuais ("65%").

import type { Hand } from '../types.js';

export const RANKS = ['2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A'] as const;

function rankIndex(r: string): number {
  return RANKS.indexOf(r as (typeof RANKS)[number]);
}

/** Normaliza uma mao para chave 169: "AA", "AKs", "AKo", etc. */
export function handKey(hand: Hand): string {
  const [c1, c2] = hand;
  const r1 = c1[0];
  const r2 = c2[0];
  const s1 = c1[1];
  const s2 = c2[1];
  if (r1 === r2) return r1 + r2;
  const hi = rankIndex(r1) >= rankIndex(r2) ? r1 : r2;
  const lo = rankIndex(r1) >= rankIndex(r2) ? r2 : r1;
  return hi + lo + (s1 === s2 ? 's' : 'o');
}

function stripPrefixes(notation: string): string {
  return notation
    .replace(/raise\s*:/gi, '')
    .replace(/limp\s*:/gi, '')
    .replace(/;/g, ',');
}

function expandPair(token: string, out: Set<string>): void {
  // "22" ou "TT+"
  const plus = token.endsWith('+');
  const base = plus ? token.slice(0, 2) : token;
  const start = rankIndex(base[0]);
  const end = plus ? rankIndex('A') : start;
  for (let i = start; i <= end; i++) {
    const r = RANKS[i];
    out.add(r + r);
  }
}

function expandSuitedOffsuit(token: string, out: Set<string>): void {
  // "A2s", "A2s+", "ATo", "ATo+", "KQs", "QJo"
  const suited = token.endsWith('s') || (token.length === 4 && token.endsWith('s+'));
  const off = token.endsWith('o') || (token.length === 4 && token.endsWith('o+'));
  const plus = token.endsWith('+');
  const core = plus ? token.slice(0, -1) : token;
  if (core.length !== 3) return;
  const hi = core[0];
  const lo = core[1];
  const suffix = core[2];
  const loStart = rankIndex(lo);
  const hiIdx = rankIndex(hi);
  if (suited || suffix === 's') {
    const end = plus ? hiIdx - 1 : loStart;
    for (let i = loStart; i <= end && i < hiIdx; i++) {
      out.add(hi + RANKS[i] + 's');
    }
  } else if (off || suffix === 'o') {
    const end = plus ? hiIdx - 1 : loStart;
    for (let i = loStart; i <= end && i < hiIdx; i++) {
      out.add(hi + RANKS[i] + 'o');
    }
  }
}

export function parseRange(notation: string): Set<string> {
  const out = new Set<string>();
  const clean = stripPrefixes(notation);
  for (const raw of clean.split(',')) {
    const token = raw.trim().replace(/\s+/g, '');
    if (!token) continue;
    if (/^\d+%$/.test(token)) {
      for (const h of topPercentRange(Number(token.slice(0, -1))).keys()) out.add(h);
      continue;
    }
    if (/^[23456789TJQKA]{2}\+?$/.test(token)) {
      expandPair(token, out);
      continue;
    }
    if (/^[23456789TJQKA]{2}[so]\+?$/.test(token)) {
      expandSuitedOffsuit(token, out);
      continue;
    }
  }
  return out;
}

export function handInRange(hand: Hand, range: Set<string>): boolean {
  return range.has(handKey(hand));
}

// Ordem aproximada de forca pre-flop (169 maos, do mais forte ao mais fraco).
// Baseline simplificado para faixas percentuais. Nao e ranking exato de solver.
const STRENGTH_ORDER_169: string[] = [
  'AA', 'KK', 'QQ', 'JJ', 'AKs', 'TT', 'AQs', 'AKo', 'AJs', '99',
  'KQs', 'AQo', 'ATs', '88', 'KJs', 'QJs', 'AJo', 'JTs', '77', 'ATo',
  'KTs', 'KQo', 'QTs', 'QJo', '66', 'JTo', 'A9s', 'KJo', 'QTo', 'J9s',
  'T9s', 'A8s', 'KTo', 'Q9s', 'J8s', 'T8s', '98s', 'A7s', 'K9s', 'Q8s',
  'J7s', 'T7s', '97s', '87s', 'A6s', 'K8s', 'Q7s', 'J6s', 'T6s', '96s',
  '86s', '76s', 'A5s', 'K7s', 'Q6s', 'J5s', 'T5s', '95s', '85s', '75s',
  '65s', 'A4s', 'K6s', 'Q5s', 'J4s', 'T4s', '94s', '84s', '74s', '64s',
  '54s', 'A3s', 'K5s', 'Q4s', 'J3s', 'T3s', '93s', '83s', '73s', '63s',
  '53s', '43s', 'A2s', 'K4s', 'Q3s', 'J2s', 'T2s', '92s', '82s', '72s',
  '62s', '52s', '42s', '32s', 'A9o', 'K9o', 'Q9o', 'J9o', 'T9o', '98o',
  'A8o', 'K8o', 'Q8o', 'J8o', 'T8o', '97o', '87o', 'A7o', 'K7o', 'Q7o',
  'J7o', 'T7o', '96o', '86o', '76o', 'A6o', 'K6o', 'Q6o', 'J6o', 'T6o',
  '95o', '85o', '75o', '65o', 'A5o', 'K5o', 'Q5o', 'J5o', 'T5o', '94o',
  '84o', '74o', '64o', '54o', '55', '44', '33', '22', 'A4o', 'K4o',
  'Q4o', 'J4o', 'T4o', '93o', '83o', '73o', '63o', '53o', '43o', 'A3o',
  'K3o', 'Q3o', 'J3o', 'T3o', '92o', '82o', '72o', '62o', '52o', '42o',
  '32o', 'A2o', 'K2o', 'Q2o', 'J2o', 'T2o',
];

export function topPercentRange(pct: number): Set<string> {
  const n = Math.max(0, Math.min(169, Math.round((pct / 100) * 169)));
  return new Set(STRENGTH_ORDER_169.slice(0, n));
}

export function countCombosHands(range: Set<string>): number {
  // Converte chaves 169 em numero aproximado de combos (1326).
  let total = 0;
  for (const h of range) {
    if (h.length === 2) total += 6;
    else if (h.endsWith('s')) total += 4;
    else total += 12;
  }
  return total;
}
