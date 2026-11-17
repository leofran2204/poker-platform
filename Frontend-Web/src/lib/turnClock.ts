export interface TurnClock {
  text: string;
  aria: string;
  /** Pouco tempo: o relógio fica vermelho. */
  urgent: boolean;
  /** O tempo normal acabou e a reserva do banco de tempo está correndo. */
  usingReserve: boolean;
}

/**
 * Relógio do turno do jogador: primeiro o tempo normal, depois a reserva do
 * banco de tempo e, quando os dois acabam, a ação automática do servidor.
 */
export function turnClock(
  turnLeft: number,
  reserveLeft: number,
  round: "bet" | "draw" | "discard",
): TurnClock {
  if (turnLeft > 0) {
    const hasReserve = reserveLeft > 0;
    return {
      text: `Sua vez: ${turnLeft}s${hasReserve ? ` · reserva ${reserveLeft}s` : ""}`,
      aria: `Tempo para agir: ${turnLeft} segundos${hasReserve ? `, mais ${reserveLeft} de reserva` : ""}`,
      urgent: turnLeft <= 10 && !hasReserve,
      usingReserve: false,
    };
  }
  if (reserveLeft > 0) {
    return {
      text: `Banco de tempo: ${reserveLeft}s`,
      aria: `Banco de tempo: ${reserveLeft} segundos de reserva`,
      urgent: reserveLeft <= 10,
      usingReserve: true,
    };
  }
  const text =
    round === "draw"
      ? "Mantendo as cartas…"
      : round === "discard"
        ? "Descartando a carta que sobra…"
        : "Fold automático…";
  return { text, aria: text, urgent: true, usingReserve: false };
}
