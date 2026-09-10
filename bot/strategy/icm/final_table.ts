// Mesa final: push/fold ICM por profundidade e posicao.
// Ranges em notacao (parseRange), nao em % sobre ordem cash — ases offsuit
// fracos valem muito no shove curto e ficariam de fora de um top% cash.
// Baselines simplificados. Para producao, gerar via ICMIZER/HRC e
// substituir por lookup JSON por (posicao, stack, payouts, jogadores).

import type { GameState, StrategyOutput } from '../types.js';
import { handInRange, parseRange } from '../math/combos.js';

// Shove BTN/SB por stack (ICM medio). EP ~55% dessa frequencia.
// Exportados para gerar bot/strategy/icm/tables/*.json (fonte unica).
export const SHOVE_LATE: Record<number, string> = {
  5: '22+, A2s+, A2o+, K2s+, K7o+, Q2s+, Q7o+, J5s+, J9o+, T7s+, 97s+, 87s, 76s',
  8: '22+, A2s+, A2o+, K2s+, K6o+, Q4s+, Q9o+, J7s+, JTo, T8s+, 98s, 87s',
  10: '22+, A2s+, A3o+, K4s+, K9o+, Q8s+, QJo, J9s+, T9s',
  12: '77+, A2s+, A5o+, K9s+, KJo+, QJs, JTs',
  15: '77+, A7s+, ATo+, KJs+, QJs, TT+',
  20: 'TT+, AJs+, AQo+',
};

export const SHOVE_EARLY: Record<number, string> = {
  5: '22+, A2s+, A5o+, K8s+, KJo+, QTs+, JTs',
  8: '66+, A2s+, A7o+, K8s+, KTo+, QTs+, JTs',
  10: '77+, A5s+, A9o+, K9s+, KQo, QJs',
  12: '88+, A7s+, ATo+, KQs',
  15: '99+, AJs+, AQo+',
  20: 'JJ+, AK',
};

export const CALL_VS_SHOVE = '88+, ATs+, AQo+';
export const CALL_VS_SHOVE_DEEP = 'TT+, AQs+, AK';

function nearestDepth(table: Record<number, string>, stackBB: number): number {
  const keys = Object.keys(table).map(Number).sort((a, b) => a - b);
  let best = keys[0];
  for (const k of keys) {
    if (Math.abs(k - stackBB) < Math.abs(best - stackBB)) best = k;
  }
  return best;
}

export function decideFinalTablePushFold(state: GameState): StrategyOutput {
  if (state.stackBB > 20) {
    return { action: 'fold', reasoning: 'FT: stack profundo, fora do push/fold' };
  }
  const late = state.position === 'BTN' || state.position === 'SB' || state.position === 'CO';
  const facingShove = state.actionHistory.some(
    (a) => a.action === 'allin' || (a.action === 'raise' && (a.sizeBB ?? 0) >= state.stackBB * 0.7),
  );
  if (!facingShove) {
    // 15bb+: premium joga de min-raise (dominante), nao shove direto
    if (state.stackBB >= 15 && handInRange(state.myHand, parseRange('JJ+, AK, AQs'))) {
      return { action: 'raise', sizeBB: 2, reasoning: `FT min-raise premium ${state.stackBB}bb ${state.position}` };
    }
    const table = late ? SHOVE_LATE : SHOVE_EARLY;
    const depth = nearestDepth(table, state.stackBB);
    const range = parseRange(table[depth]);
    if (handInRange(state.myHand, range)) {
      return { action: 'allin', reasoning: `FT push ${state.stackBB}bb ${state.position}` };
    }
    // Maos premium que nao dao shove direto: min-raise/call-off (12bb+)
    if (state.stackBB >= 12 && handInRange(state.myHand, parseRange('99+, AJs+, AQo+, KQs'))) {
      return { action: 'raise', sizeBB: 2, reasoning: `FT min-raise premium ${state.stackBB}bb` };
    }
    return { action: 'fold', reasoning: `FT fora do push ${state.stackBB}bb ${state.position}` };
  }
  const range = parseRange(state.effectiveStackBB >= 15 ? CALL_VS_SHOVE_DEEP : CALL_VS_SHOVE);
  if (handInRange(state.myHand, range)) {
    return { action: 'call', reasoning: `FT call ICM ${state.stackBB}bb` };
  }
  return { action: 'fold', reasoning: 'FT fold vs shove (premio de risco ICM)' };
}
