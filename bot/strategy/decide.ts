// Ponto unico de decisao do bot: recebe GameState, devolve acao.
// Substitui o Math.random() dos scripts de carga (full-catalog, estrutura-bots).

import type { GameState, StrategyOutput } from './types.js';
import { decidePreflopCash6max } from './preflop/cash_6max.js';
import { decidePreflopCash9max } from './preflop/cash_9max.js';
import { decidePreflopMTT } from './preflop/mtt_icm.js';
import { decideCBet } from './postflop/cbet.js';
import { decideRiver } from './postflop/river.js';
import { evaluate, showdownEquity } from './postflop/evaluator.js';
import { requiredEquity } from './math/pot_odds.js';
import { equityByOuts } from './math/equity.js';

export function decideAction(state: GameState): StrategyOutput {
  if (state.street === 'preflop') {
    if (state.gameType === 'cash_9max') return decidePreflopCash9max(state);
    if (state.gameType === 'mtt' || state.gameType === 'sng' || state.gameType === 'pko') {
      return decidePreflopMTT(state);
    }
    return decidePreflopCash6max(state);
  }

  const ev = evaluate(state.myHand, state.board);

  // Sem aposta a pagar: c-bet heuristico no flop/turn; no river, valor real.
  if (state.betToCallBB <= 0) {
    if (state.street === 'river') {
      const strength =
        ev.category === 'straight_flush' || ev.category === 'quads' || ev.category === 'full_house'
          ? 'nuts'
          : ev.category === 'flush' || ev.category === 'straight' || ev.category === 'trips' || ev.category === 'two_pair'
            ? 'strong'
            : ev.category === 'overpair' || ev.category === 'top_pair'
              ? 'medium'
              : ev.drawOuts >= 8
                ? 'weak'
                : 'air';
      return decideRiver(state, strength, ev.nutFlushBlocker || ev.drawOuts >= 8);
    }
    const lastAggressor = [...state.actionHistory].reverse().find((a) => a.action === 'raise' || a.action === 'bet');
    const ip = !lastAggressor || lastAggressor.position === state.position;
    return decideCBet(state, ip);
  }

  // Facing bet: equity real (made hand + draws) vs pot odds, margem de 5pp.
  const need = requiredEquity(state.betToCallBB, state.potBB, 0);
  const drawEq = equityByOuts(ev.drawOuts, state.street === 'flop' ? 'flop' : 'turn');
  const eq = Math.max(showdownEquity(ev), drawEq);
  const label = ev.draws.length && ev.category === 'high' ? `draw(${ev.draws.join('+')})` : ev.category;
  if (eq >= need + 0.05) {
    if (state.betToCallBB >= state.effectiveStackBB * 0.7) {
      return { action: 'allin', reasoning: `${label} call all-in (eq~${Math.round(eq * 100)}% vs need ${Math.round(need * 100)}%)` };
    }
    return { action: 'call', reasoning: `${label} call (eq~${Math.round(eq * 100)}% vs need ${Math.round(need * 100)}%)` };
  }
  return { action: 'fold', reasoning: `${label} fold (eq~${Math.round(eq * 100)}% vs need ${Math.round(need * 100)}%)` };
}
