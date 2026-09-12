/** Loss Deflator — faixas por equity do perdedor no instante do all-in.
 * Regra normativa: BUSINESS_RULES.md §11. Este arquivo só espelha os
 * percentuais para a UI educacional (simulador da home). */

export interface DeflatorTier {
  /** Equity mínima (inclusive) para esta faixa. */
  minEquity: number;
  /** Equity máxima (exclusive). null = sem teto. */
  maxEquity: number | null;
  /** Percentual devolvido sobre o pote líquido pós-rake. */
  percent: number;
  /** Rótulo curto para a UI. */
  label: string;
}

export const DEFLATOR_TIERS: DeflatorTier[] = [
  { minEquity: 86, maxEquity: null, percent: 35, label: "Bad beat extrema" },
  { minEquity: 76, maxEquity: 86, percent: 25, label: "Grande favorito" },
  { minEquity: 66, maxEquity: 76, percent: 15, label: "Favorito moderado" },
  { minEquity: 56, maxEquity: 66, percent: 7, label: "Favorito leve" },
];

/** Faixa aplicável a uma equity (0–100). null = abaixo de 56%, sem devolução. */
export function tierForEquity(equityPercent: number): DeflatorTier | null {
  if (!Number.isFinite(equityPercent) || equityPercent < 56) return null;
  for (const tier of DEFLATOR_TIERS) {
    if (equityPercent >= tier.minEquity && (tier.maxEquity === null || equityPercent < tier.maxEquity)) {
      return tier;
    }
  }
  return DEFLATOR_TIERS[0];
}

export interface CashbackResult {
  percent: number;
  cashbackCents: number;
  tier: DeflatorTier | null;
}

/** Cashback em centavos sobre o pote líquido (já sem rake). Trunca, como o motor. */
export function cashbackFor(equityPercent: number, potCents: number): CashbackResult {
  const tier = tierForEquity(equityPercent);
  if (!tier || !Number.isFinite(potCents) || potCents <= 0) {
    return { percent: 0, cashbackCents: 0, tier };
  }
  const pot = Math.trunc(potCents);
  return { percent: tier.percent, cashbackCents: Math.trunc((pot * tier.percent) / 100), tier };
}
