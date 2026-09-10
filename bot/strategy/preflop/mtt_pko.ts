// PKO: ajuste de bounty. Quem cobre joga mais solto; quem e coberto joga mais tight.
// Bounty grande (~50% do prizepool) aproxima de ChipEV; bounty pequeno aproxima de ICM.

export function bountyEquityAddOn(bountyBB: number, potBB: number): number {
  if (potBB <= 0) return 0;
  return Math.min(0.15, bountyBB / (potBB * 4));
}

/** % extra para abrir o range de shove quando voce cobre o alvo. */
export function pkoShoveWidenPct(bountyFraction: number): number {
  // bountyFraction: fracao do buy-in que o bounty representa (0 a 1)
  if (bountyFraction >= 0.5) return 12;
  if (bountyFraction >= 0.25) return 7;
  return 3;
}
