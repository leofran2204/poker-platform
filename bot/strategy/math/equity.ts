// Estimativa de equidade por outs (regra do 4 e do 2) e tabelas de outs comuns.
// Para runtime do bot: rapido e deterministico. Nao substitui solver.

import type { Street } from '../types.js';

/** Regra do 4 e do 2: outs -> equity aproximada. */
export function equityByOuts(outs: number, from: Street, to: 'river' = 'river'): number {
  if (from === 'flop' && to === 'river') return Math.min(0.95, (outs * 4) / 100);
  return Math.min(0.95, (outs * 2) / 100);
}

export const COMMON_OUTS = {
  flushDraw: 9,
  openEndedStraightDraw: 8,
  gutshot: 4,
  twoOvercards: 6,
  flushDrawPlusOvercards: 15,
  openEndedPlusFlushDraw: 15,
} as const;

/** Outs de draws comuns presentes no flop. Retorna lista de draws detectados. */
export function detectDrawOutsDescription(outs: number): string {
  if (outs >= 15) return 'combo-draw forte (~60% flop->river)';
  if (outs >= 9) return 'flush draw (~36% flop->river)';
  if (outs >= 8) return 'straight draw aberto (~32%)';
  if (outs >= 6) return 'overcards (~24%)';
  if (outs >= 4) return 'gutshot (~16%)';
  return 'poucos outs';
}
