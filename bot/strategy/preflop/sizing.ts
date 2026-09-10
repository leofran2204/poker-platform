// Tamanhos padrao de aposta pre-flop (em BB). Baselines 100bb cash e MTT.

export function openSizeBB(position: string, gameType: string, stackBB: number): number {
  if (gameType.startsWith('cash')) {
    if (position === 'SB') return 3;
    return 2.5;
  }
  // MTT: menores com stacks curtos
  if (stackBB <= 20) return 2;
  if (stackBB <= 40) return 2.2;
  return 2.5;
}

export function threeBetSizeBB(
  openSize: number,
  inPosition: boolean,
  stackBB: number,
): number {
  const mult = inPosition ? 3 : 3.75;
  const size = openSize * mult;
  // Nao comprometer mais de 25% do stack com 3-bet nao all-in em stack curto
  if (stackBB <= 30) return Math.min(size, stackBB * 0.25);
  return Math.round(size * 10) / 10;
}

export function fourBetSizeBB(threeBetSize: number, inPosition: boolean): number {
  const mult = inPosition ? 2.3 : 2.6;
  return Math.round(threeBetSize * mult * 10) / 10;
}

export function isoRaiseSizeBB(limpers: number): number {
  return 4 + Math.max(0, limpers - 1);
}
