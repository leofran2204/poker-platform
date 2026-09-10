// Ranges cash 6-max 100bb, rake 5% 4bb cap. Baselines simplificados implementaveis.
// Formato: notacao parseada por math/combos.ts.

import type { GameState, Position, StrategyOutput } from '../types.js';
import { handInRange, parseRange } from '../math/combos.js';
import { openSizeBB, threeBetSizeBB } from './sizing.js';

export const RFI_6MAX_100BB: Record<string, string> = {
  UTG: '77+, ATs+, AJo+, KQs, KQo',
  HJ: '55+, A9s+, ATo+, KTs+, KJo+, QTs+, JTs, T9s, 98s',
  CO: '33+, A2s+, ATo+, K9s+, KTo+, Q9s+, QJo, J9s+, T8s+, 97s+, 87s, 76s',
  BTN: '22+, A2s+, A2o+, K2s+, K9o+, Q2s+, Q9o+, J2s+, J8o+, T2s+, T8o+, 92s+, 97o+, 82s+, 86o+, 72s+, 75o+, 62s+, 64o+, 52s+, 42s+, 32s',
  SB: '22+, A2s+, A2o+, K2s+, K5o+, Q2s+, Q8o+, J2s+, J8o+, T2s+, T8o+, 92s+, 82s+, 72s+, 62s+, 52s+, 42s+, 32s',
  BB: '',
};

const SB_LIMP_6MAX = '85o, 74o, 63o, 52o, 42o, 32o, 94o, 83o, 72o';

export const THREE_BET_VS_RFI_6MAX: Record<string, { value: string; bluff: string; call: string }> = {
  BTN_vs_UTG: { value: 'JJ+, AK', bluff: 'A5s-A2s, K9s-K8s, Q9s, J9s, T8s, 97s', call: 'TT-22, AQs-AJs, ATs, KQs, KJs, QJs, JTs, T9s, 98s, 87s' },
  BTN_vs_HJ: { value: 'TT+, AK, AQs', bluff: 'A5s-A2s, K9s-K8s, Q9s-Q8s, J9s, T9s, 98s, 87s', call: '99-22, AQo, AJs, ATs, KQs, KJs, QJs, JTs, T9s, 98s, 87s, 76s' },
  BTN_vs_CO: { value: 'TT+, AK, AQs', bluff: 'A5s-A2s, K9s-K8s, Q9s-Q8s, J9s, T9s, 98s, 87s', call: '99-22, AQo, AJs, ATs, KQs, KJs, QJs, JTs, T9s, 98s, 87s, 76s' },
  SB_vs_BTN: { value: '99+, AK, AQs', bluff: 'A5s-A2s, K9s-K8s, Q9s, J9s, T8s', call: '88-22, AQo, AJs, ATs, KQs, KJs, QJs, JTs, T9s, 98s, 87s' },
  BB_vs_SB: { value: '88+, AK, AQs, AQo', bluff: 'A5s-A2s, K9s-K8s, Q9s-Q8s, J9s, T9s', call: '77-22, AJs, ATs, KQs, KJs, QJs, JTs, T9s, 98s, 87s, 76s' },
  DEFAULT: { value: 'JJ+, AK, AQs', bluff: 'A5s-A2s, K9s-K8s', call: 'TT-22, AQo, AJs, KQs, QJs, JTs' },
};

const FOUR_BET_VALUE_6MAX = 'QQ+, AK';
const FOUR_BET_BLUFF_6MAX = 'A5s-A2s';

function openerOf(state: GameState): Position | null {
  const r = state.actionHistory.find((a) => a.street === 'preflop' && a.action === 'raise');
  return r ? r.position : null;
}

export function decidePreflopCash6max(state: GameState): StrategyOutput {
  const raises = state.actionHistory.filter((a) => a.street === 'preflop' && a.action === 'raise');
  const opener = openerOf(state);

  if (raises.length === 0) {
    if (state.position === 'SB' && handInRange(state.myHand, parseRange(SB_LIMP_6MAX))) {
      return { action: 'call', reasoning: 'SB limp bottom range 6-max' };
    }
    const range = parseRange(RFI_6MAX_100BB[state.position] ?? '');
    if (handInRange(state.myHand, range)) {
      return { action: 'raise', sizeBB: openSizeBB(state.position, 'cash_6max', state.stackBB), reasoning: `RFI ${state.position} 6-max` };
    }
    return { action: 'fold', reasoning: `Fora do RFI ${state.position} 6-max` };
  }

  if (raises.length === 1 && opener) {
    const key = `${state.position}_vs_${opener}`;
    const t = THREE_BET_VS_RFI_6MAX[key] ?? THREE_BET_VS_RFI_6MAX.DEFAULT;
    if (handInRange(state.myHand, parseRange(t.value))) {
      const openSize = raises[0].sizeBB ?? 2.5;
      const ip = state.position === 'BTN' || state.position === 'CO';
      return { action: 'raise', sizeBB: threeBetSizeBB(openSize, ip, state.stackBB), reasoning: `3-bet valor vs ${opener}` };
    }
    if (handInRange(state.myHand, parseRange(t.bluff))) {
      const openSize = raises[0].sizeBB ?? 2.5;
      const ip = state.position === 'BTN' || state.position === 'CO';
      return { action: 'raise', sizeBB: threeBetSizeBB(openSize, ip, state.stackBB), reasoning: `3-bet blefe vs ${opener}` };
    }
    if (handInRange(state.myHand, parseRange(t.call))) {
      return { action: 'call', reasoning: `Flat vs ${opener}` };
    }
    return { action: 'fold', reasoning: `Fora do 3-bet/call vs ${opener}` };
  }

  // Facing 3-bet: 4-bet ou fold/call tight
  if (handInRange(state.myHand, parseRange(FOUR_BET_VALUE_6MAX)) || handInRange(state.myHand, parseRange(FOUR_BET_BLUFF_6MAX))) {
    const last = raises[raises.length - 1].sizeBB ?? 9;
    const ip = state.position === 'BTN' || state.position === 'CO';
    const mult = ip ? 2.3 : 2.6;
    return { action: 'raise', sizeBB: Math.round(last * mult * 10) / 10, reasoning: '4-bet 6-max' };
  }
  if (handInRange(state.myHand, parseRange('JJ, TT, AQs, AJs, KQs'))) {
    return { action: 'call', reasoning: 'Call vs 3-bet 6-max' };
  }
  return { action: 'fold', reasoning: 'Fold vs 3-bet 6-max' };
}
