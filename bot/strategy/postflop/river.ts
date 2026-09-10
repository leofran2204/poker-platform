// River: valor polarizado, blefes com blockers e overbet.
// Baseline: beta top do range, checa meio, blefa fundo com blockers.

import type { GameState, StrategyOutput } from '../types.js';

export function decideRiver(
  state: GameState,
  handStrength: 'nuts' | 'strong' | 'medium' | 'weak' | 'air',
  hasBlocker: boolean,
): StrategyOutput {
  switch (handStrength) {
    case 'nuts':
    case 'strong': {
      // Overbet quando o range do vilao esta capped e o board e limpo
      const overbet = handStrength === 'nuts' && state.board.length === 5;
      const size = overbet ? 1.5 : 0.75;
      return { action: 'bet', sizeBB: Math.round(state.potBB * size * 10) / 10, reasoning: `Valor river ${handStrength} ${Math.round(size * 100)}% pot` };
    }
    case 'medium':
      return { action: 'check', reasoning: 'Check river maos medias (controle, evita raise)' };
    case 'weak':
    case 'air': {
      if (hasBlocker) {
        return { action: 'bet', sizeBB: Math.round(state.potBB * 0.75 * 10) / 10, reasoning: 'Blefe river com blocker' };
      }
      return { action: 'check', reasoning: 'Check river sem blocker' };
    }
  }
}
