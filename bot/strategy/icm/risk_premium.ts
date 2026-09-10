// ICM: Malmuth-Harville simplificado, bubble factor e premio de risco.
// Para stacks e payouts reais. Uso em runtime do bot.

/** Probabilidade de terminar em cada posicao (Malmuth-Harville). */
export function icmEquities(stacks: number[], payouts: number[]): number[] {
  const n = stacks.length;
  const m = Math.min(n, payouts.length);
  const eq = new Array(n).fill(0);
  const total = stacks.reduce((a, b) => a + b, 0);
  if (total <= 0) return eq;

  // Probabilidade de 1o lugar proporcional ao stack
  const pFirst = stacks.map((s) => s / total);

  for (let i = 0; i < n; i++) {
    eq[i] += pFirst[i] * (payouts[0] ?? 0);
    // 2o..m: soma sobre possiveis vencedores
    for (let w = 0; w < n; w++) {
      if (w === i || stacks[w] <= 0) continue;
      const rest = total - stacks[w];
      if (rest <= 0) continue;
      const pSecond = (stacks[i] / rest) * pFirst[w];
      if (m >= 2) eq[i] += pSecond * (payouts[1] ?? 0);
      // 3o+: aproximacao iterativa (suficiente para runtime do bot)
      if (m >= 3) {
        for (let x = 0; x < n; x++) {
          if (x === i || x === w || stacks[x] <= 0) continue;
          const rest2 = rest - stacks[x];
          if (rest2 <= 0) continue;
          const pThird = (stacks[i] / rest2) * (stacks[x] / rest) * pFirst[w];
          eq[i] += pThird * (payouts[2] ?? 0);
        }
      }
    }
  }
  return eq;
}

export function bubbleFactor(
  heroStack: number,
  villainStack: number,
  allStacks: number[],
  payouts: number[],
): number {
  const heroIdx = allStacks.indexOf(heroStack);
  const idx = heroIdx >= 0 ? heroIdx : 0;
  const base = icmEquities(allStacks, payouts)[idx] ?? 0;
  const doubled = [...allStacks];
  doubled[idx] = heroStack + Math.min(villainStack, heroStack);
  const up = icmEquities(doubled, payouts)[idx] ?? 0;
  const gain = up - base;
  const loss = base; // bust = 0 se ainda nao ITM; conservador
  if (gain <= 0) return 2;
  return Math.max(1, loss / gain);
}

/** Premio de risco em pontos de equity (0 a 0.5). Ex: 0.15 = +15pp. */
export function riskPremium(
  heroStack: number,
  villainStack: number,
  allStacks: number[],
  payouts: number[],
): number {
  const bf = bubbleFactor(heroStack, villainStack, allStacks, payouts);
  return (bf - 1) / (bf + 1);
}

/** Referencia rapida quando nao ha payouts (cash ou ChipEV puro). */
export function chipEVRequiredEquity(callBB: number, potBB: number): number {
  return callBB / (potBB + callBB);
}
