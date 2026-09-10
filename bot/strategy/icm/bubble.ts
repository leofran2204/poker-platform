// Bolha: ajustes de range por pressao ICM.
// Mid stacks sofrem mais; chip leader ataca; short joga perto de ChipEV.

export interface BubbleContext {
  playersLeft: number;
  paidPlaces: number;
  heroStackBB: number;
  fieldAvgBB: number;
  isChipLeader: boolean;
  isShort: boolean;
  isMid: boolean;
}

export function bubbleTightening(ctx: BubbleContext): number {
  // Retorna fator 0..1 para encolher o range de open (1 = sem ajuste).
  const fromMoney = ctx.playersLeft - ctx.paidPlaces;
  if (fromMoney > 10) return 1;
  if (fromMoney <= 0) return 1; // ja ITM: acumular
  if (ctx.isMid) return 0.55;
  if (ctx.isChipLeader) return 1.15; // abre mais
  if (ctx.isShort) return 0.9;
  return 0.8;
}

export function shouldShoveOrFold(stackBB: number, nearBubble: boolean): boolean {
  if (stackBB <= 12) return true;
  if (nearBubble && stackBB <= 15) return true;
  return false;
}

export const BUBBLE_OPEN_EXAMPLE = {
  standard_MP_pct: 19,
  bubble_MP_pct: 9.5,
  note: 'MP ChipEV ~19% (22+, ATs+, KTs+, QTs+, J9s+, T9s, 98s, 87s, 76s, AJo+, KQo) -> bolha ~9.5% (77+, ATs+, KJs+, QJs, AJo+, KQo)',
};
