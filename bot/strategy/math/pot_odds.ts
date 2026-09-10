// Pot odds, EV e fold equity. Funcoes puras, testaveis.

/** Pot odds: valor a pagar / (pote + valor a pagar). Retorna 0 a 1. */
export function potOdds(callAmountBB: number, potBB: number): number {
  if (callAmountBB <= 0) return 0;
  return callAmountBB / (potBB + callAmountBB);
}

/** Equity necessaria para um call ser +EV, com premio de risco ICM opcional. */
export function requiredEquity(callAmountBB: number, potBB: number, riskPremium = 0): number {
  return potOdds(callAmountBB, potBB) + riskPremium;
}

/** EV de um call em BB: equity * (pote + call) - call. */
export function evCall(equity: number, callAmountBB: number, potBB: number): number {
  return equity * (potBB + callAmountBB) - callAmountBB;
}

/** EV de um blefe em BB. */
export function evBluff(
  foldFreq: number,
  betSizeBB: number,
  potBB: number,
  equityWhenCalled = 0,
): number {
  return (
    foldFreq * potBB +
    (1 - foldFreq) * (equityWhenCalled * (potBB + betSizeBB * 2) - betSizeBB)
  );
}

/** Frequencia minima de fold para um blefe puro ser lucrativo. */
export function breakevenFoldFreq(betSizeBB: number, potBB: number): number {
  return betSizeBB / (potBB + betSizeBB);
}
