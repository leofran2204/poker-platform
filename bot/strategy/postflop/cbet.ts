// Classificacao de textura do flop e decisao de c-bet IP/OOP.
// Heuristicas GTO simplificadas para 100bb single-raised pots.

import type { Card, GameState, StrategyOutput } from '../types.js';

export type FlopTexture =
  | 'high_dry' | 'high_wet' | 'paired' | 'monotone' | 'connected_low' | 'disconnected_low';

function rankOf(c: Card): string {
  return c[0];
}

function suitOf(c: Card): string {
  return c[1];
}

export function classifyFlop(board: Card[]): FlopTexture {
  const [a, b, c] = board;
  const suits = [suitOf(a), suitOf(b), suitOf(c)];
  if (suits[0] === suits[1] && suits[1] === suits[2]) return 'monotone';
  const ranks = [rankOf(a), rankOf(b), rankOf(c)];
  if (ranks[0] === ranks[1] || ranks[1] === ranks[2] || ranks[0] === ranks[2]) return 'paired';
  const order = '23456789TJQKA';
  const idx = ranks.map((r) => order.indexOf(r)).sort((x, y) => x - y);
  const connected = idx[2] - idx[0] <= 4;
  const high = idx.some((i) => i >= order.indexOf('J'));
  if (connected && idx[2] < order.indexOf('9')) return 'connected_low';
  if (connected && high) return 'high_wet';
  if (high) return 'high_dry';
  return 'disconnected_low';
}

const CBET_FREQ_IP: Record<FlopTexture, number> = {
  high_dry: 0.65, high_wet: 0.5, paired: 0.6,
  monotone: 0.32, connected_low: 0.47, disconnected_low: 0.57,
};

const CBET_SIZE_IP: Record<FlopTexture, number> = {
  high_dry: 0.33, high_wet: 0.66, paired: 0.4,
  monotone: 0.6, connected_low: 0.6, disconnected_low: 0.4,
};

function hashHand(board: Card[], street: string): number {
  let h = 0;
  const s = board.join('') + street;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) % 1000;
  return h / 1000;
}

export function decideCBet(state: GameState, inPosition: boolean): StrategyOutput {
  const texture = classifyFlop(state.board);
  const freq = inPosition ? CBET_FREQ_IP[texture] : CBET_FREQ_IP[texture] * 0.6;
  const size = inPosition ? CBET_SIZE_IP[texture] : CBET_SIZE_IP[texture] * 1.25;
  // Randomizacao deterministica por board para estrategia mista auditavel
  const roll = hashHand(state.board, state.street);
  if (roll < freq) {
    const sizeBB = Math.round(state.potBB * size * 10) / 10;
    return { action: state.street === 'preflop' ? 'raise' : 'bet', sizeBB, reasoning: `C-bet ${texture} ${inPosition ? 'IP' : 'OOP'} ${Math.round(size * 100)}% pot` };
  }
  return { action: 'check', reasoning: `Check ${texture} ${inPosition ? 'IP' : 'OOP'} (fora da freq ${Math.round(freq * 100)}%)` };
}
