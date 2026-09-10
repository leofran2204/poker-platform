// Bankroll management: regras de buy-in e risco de ruina.

export const BRM = {
  cashGame: { minBI: 30, moveUp: 40, moveDown: 25 },
  mtt: { minBI: 100, moveUp: 150, moveDown: 80 },
  sng: { minBI: 50, moveUp: 75, moveDown: 40 },
} as const;

export function canPlayStake(bankroll: number, buyin: number, format: keyof typeof BRM): boolean {
  return bankroll >= buyin * BRM[format].minBI;
}

/** Risco de ruina simplificado. Retorna 0 a 1. */
export function riskOfRuin(winrateBB100: number, stdDevBB100: number, bankrollBB: number): number {
  if (winrateBB100 <= 0) return 1;
  if (stdDevBB100 <= 0) return 0;
  return Math.exp((-2 * winrateBB100 * bankrollBB) / stdDevBB100 ** 2);
}
