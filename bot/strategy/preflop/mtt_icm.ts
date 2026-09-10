// MTT geral: roteia ChipEV, bolha e push/fold por stack.

import type { GameState, StrategyOutput } from '../types.js';
import { handInRange, topPercentRange } from '../math/combos.js';
import { openSizeMTT, rfiPercentMTT } from './mtt_chipev.js';
import { bubbleTightening } from '../icm/bubble.js';
import { decideFinalTablePushFold } from '../icm/final_table.js';

export function decidePreflopMTT(state: GameState): StrategyOutput {
  // Push/fold curto ou mesa final com ICM vai para o modulo ICM
  if (state.icm && (state.stackBB <= 20 || (state.icm.playersLeft <= 9 && state.stackBB <= 30))) {
    return decideFinalTablePushFold(state);
  }
  if (state.stackBB <= 12) {
    const pct = state.position === 'BTN' || state.position === 'SB' ? 45 : 25;
    const range = topPercentRange(pct);
    if (handInRange(state.myHand, range)) {
      return { action: 'allin', reasoning: `MTT push ${state.stackBB}bb ${state.position}` };
    }
    return { action: 'fold', reasoning: `MTT fold ${state.stackBB}bb` };
  }
  const raises = state.actionHistory.filter((a) => a.street === 'preflop' && a.action === 'raise');
  if (raises.length > 0) {
    // Facing raise em MTT medio: 3-bet polarizado ou fold; flats so IP profundo
    if (handInRange(state.myHand, topPercentRange(6))) {
      return { action: 'raise', sizeBB: (raises[0].sizeBB ?? 2.2) * 3, reasoning: 'MTT 3-bet valor' };
    }
    if (state.stackBB >= 30 && handInRange(state.myHand, topPercentRange(12))) {
      return { action: 'call', reasoning: 'MTT flat IP profundo' };
    }
    return { action: 'fold', reasoning: 'MTT fold vs raise' };
  }
  let pct = rfiPercentMTT(state.position, state.stackBB);
  if (state.icm) {
    const stacks = state.icm.allStacks;
    const avg = stacks.reduce((a, b) => a + b, 0) / Math.max(1, stacks.length);
    const sorted = [...stacks].sort((a, b) => b - a);
    const factor = bubbleTightening({
      playersLeft: state.icm.playersLeft,
      paidPlaces: state.icm.paidPlaces,
      heroStackBB: state.stackBB,
      fieldAvgBB: avg,
      isChipLeader: state.stackBB >= (sorted[0] ?? 0),
      isShort: state.stackBB <= (sorted[sorted.length - 1] ?? 0) * 1.2,
      isMid: state.stackBB > avg * 0.6 && state.stackBB < avg * 1.5,
    });
    pct = Math.max(5, Math.round(pct * factor));
  }
  if (handInRange(state.myHand, topPercentRange(pct))) {
    return { action: 'raise', sizeBB: openSizeMTT(state.stackBB), reasoning: `MTT RFI ${state.position} top${pct}%` };
  }
  return { action: 'fold', reasoning: `MTT fora do RFI ${state.position} top${pct}%` };
}
