// Ranges cash 9-max (full ring) 100bb. UTG-MP mais tight que 6-max;
// CO em diante reaproveita a logica 6-max.

import type { GameState, StrategyOutput } from '../types.js';
import { handInRange, parseRange } from '../math/combos.js';
import { RFI_6MAX_100BB, THREE_BET_VS_RFI_6MAX } from './cash_6max.js';
import { openSizeBB, threeBetSizeBB } from './sizing.js';

export const RFI_9MAX_100BB: Record<string, string> = {
  UTG: '99+, AKs, AKo, AQs',
  'UTG+1': '88+, AK, AQs, AQo',
  'UTG+2': '77+, AK, AQ, AJs',
  MP1: '55+, ATs+, AJo+, KQs',
  MP2: '44+, A9s+, ATo+, KTs+, KJo+, QTs+, JTs, T9s',
  HJ: '55+, A9s+, ATo+, KTs+, KJo+, QTs+, JTs, T9s, 98s',
  CO: RFI_6MAX_100BB.CO,
  BTN: RFI_6MAX_100BB.BTN,
  SB: RFI_6MAX_100BB.SB,
  BB: '',
};

export function decidePreflopCash9max(state: GameState): StrategyOutput {
  const raises = state.actionHistory.filter((a) => a.street === 'preflop' && a.action === 'raise');
  if (raises.length === 0) {
    const range = parseRange(RFI_9MAX_100BB[state.position] ?? '');
    if (handInRange(state.myHand, range)) {
      return { action: 'raise', sizeBB: openSizeBB(state.position, 'cash_9max', state.stackBB), reasoning: `RFI ${state.position} 9-max` };
    }
    return { action: 'fold', reasoning: `Fora do RFI ${state.position} 9-max` };
  }
  if (raises.length >= 2) {
    if (handInRange(state.myHand, parseRange('QQ+, AK'))) {
      const last = raises[raises.length - 1].sizeBB ?? 9;
      return { action: 'raise', sizeBB: Math.round(last * 2.5 * 10) / 10, reasoning: '4-bet 9-max' };
    }
    if (handInRange(state.myHand, parseRange('JJ, TT, AQs'))) {
      return { action: 'call', reasoning: 'Call vs 3-bet 9-max' };
    }
    return { action: 'fold', reasoning: 'Fold vs 3-bet 9-max' };
  }
  const opener = raises[0].position;
  const key = `${state.position}_vs_${opener}`;
  const t = THREE_BET_VS_RFI_6MAX[key] ?? THREE_BET_VS_RFI_6MAX.DEFAULT;
  const openSize = raises[0].sizeBB ?? 2.5;
  const earlyOpener = opener === 'UTG' || opener === 'UTG+1' || opener === 'UTG+2';
  if (handInRange(state.myHand, parseRange(earlyOpener ? 'QQ+, AK' : t.value))) {
    return { action: 'raise', sizeBB: threeBetSizeBB(openSize, false, state.stackBB), reasoning: `3-bet valor vs ${opener} 9-max` };
  }
  if (!earlyOpener && handInRange(state.myHand, parseRange(t.bluff))) {
    return { action: 'raise', sizeBB: threeBetSizeBB(openSize, false, state.stackBB), reasoning: `3-bet blefe vs ${opener} 9-max` };
  }
  if (handInRange(state.myHand, parseRange(t.call))) {
    return { action: 'call', reasoning: `Flat vs ${opener} 9-max` };
  }
  return { action: 'fold', reasoning: `Fold vs ${opener} 9-max` };
}
