// Turn: double barrel, probe e check-raise. Heuristicas por tipo de carta.

import type { Card, GameState, StrategyOutput } from '../types.js';
import { classifyFlop } from './cbet.js';

export type TurnCard = 'blank' | 'overcard' | 'flush_complete' | 'straight_complete' | 'pairs_board' | 'gives_equity';

export function classifyTurn(flop: Card[], turn: Card): TurnCard {
  const suits = [...flop, turn].map((c) => c[1]);
  const flushSuits = suits.filter((s, _, arr) => arr.filter((x) => x === s).length >= 3);
  if (flushSuits.length >= 3) {
    const turnSuit = turn[1];
    if (flop.filter((c) => c[1] === turnSuit).length === 2) return 'flush_complete';
  }
  const order = '23456789TJQKA';
  const flopHigh = Math.max(...flop.map((c) => order.indexOf(c[0])));
  if (order.indexOf(turn[0]) > flopHigh && order.indexOf(turn[0]) >= order.indexOf('Q')) return 'overcard';
  if (flop.some((c) => c[0] === turn[0])) return 'pairs_board';
  return 'blank';
}

const BARREL_FREQ: Record<TurnCard, number> = {
  blank: 0.48, overcard: 0.52, flush_complete: 0.37,
  straight_complete: 0.37, pairs_board: 0.47, gives_equity: 0.6,
};

export function decideTurn(state: GameState, turnKind: TurnCard, wasAggressor: boolean): StrategyOutput {
  const freq = BARREL_FREQ[turnKind];
  const size = turnKind === 'flush_complete' || turnKind === 'straight_complete' ? 0.9 : 0.7;
  if (wasAggressor) {
    // Deterministico por board+turn para auditoria
    const roll = (state.board.join('').length * 37 + turnKind.length * 13) % 100 / 100;
    if (roll < freq) {
      return { action: 'bet', sizeBB: Math.round(state.potBB * size * 10) / 10, reasoning: `Double barrel ${turnKind} ${Math.round(size * 100)}% pot` };
    }
    return { action: 'check', reasoning: `Check turn ${turnKind} (controle de pote)` };
  }
  // Probe OOP quando o agressor desiste
  if (turnKind === 'blank' || turnKind === 'pairs_board') {
    return { action: 'bet', sizeBB: Math.round(state.potBB * 0.66 * 10) / 10, reasoning: `Probe turn ${turnKind}` };
  }
  return { action: 'check', reasoning: `Check turn ${turnKind} sem iniciativa` };
}

export function flopOf(state: GameState): ReturnType<typeof classifyFlop> {
  return classifyFlop(state.board.slice(0, 3) as Card[]);
}
