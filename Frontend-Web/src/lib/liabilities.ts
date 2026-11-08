// Conciliação diária do Jogo Real: o saldo disponível na conta recebedora
// precisa cobrir tudo o que pertence aos jogadores (GET /api/admin/liabilities).

export type CoverageState = "covered" | "short";

export interface Coverage {
  state: CoverageState;
  /** Saldo da conta menos o total devido, em centavos (negativo = falta). */
  differenceCents: number;
  /** Quanto do total devido o saldo cobre, de 0 a 1+ (null se nada é devido). */
  ratio: number | null;
}

export function computeCoverage(accountBalanceCents: number, owedCents: number): Coverage {
  const differenceCents = accountBalanceCents - owedCents;
  return {
    state: differenceCents >= 0 ? "covered" : "short",
    differenceCents,
    ratio: owedCents > 0 ? accountBalanceCents / owedCents : null,
  };
}
