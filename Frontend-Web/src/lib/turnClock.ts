/** Tempo normal de cada jogada (BUSINESS_RULES 10.2). */
export const TURN_SECONDS = 15;
/** Tempo extra de cada jogada, depois do tempo normal; renova a cada jogada. */
export const EXTRA_SECONDS = 5;

export interface TurnClock {
  text: string;
  aria: string;
  /** Pouco tempo: o relógio fica vermelho. */
  urgent: boolean;
  /** O tempo normal acabou e o tempo extra está correndo. */
  extraTime: boolean;
}

/**
 * Relógio do turno do jogador: primeiro o tempo normal, depois o tempo extra
 * e, quando os dois acabam, a ação automática do servidor.
 */
export function turnClock(
  turnLeft: number,
  extraLeft: number,
  round: "bet" | "draw" | "discard",
): TurnClock {
  if (turnLeft > 0) {
    const hasExtra = extraLeft > 0;
    return {
      text: `Sua vez: ${turnLeft}s${hasExtra ? ` + ${extraLeft}s` : ""}`,
      aria: `Tempo para agir: ${turnLeft} segundos${hasExtra ? `, mais ${extraLeft} de tempo extra` : ""}`,
      urgent: turnLeft <= 5 && !hasExtra,
      extraTime: false,
    };
  }
  if (extraLeft > 0) {
    return {
      text: `TEMPO EXTRA: ${extraLeft}s`,
      aria: `Tempo extra: ${extraLeft} segundos`,
      urgent: true,
      extraTime: true,
    };
  }
  const text =
    round === "draw"
      ? "Mantendo as cartas…"
      : round === "discard"
        ? "Descartando a carta que sobra…"
        : "Fold automático…";
  return { text, aria: text, urgent: true, extraTime: false };
}

const TURN_TITLE = "Sua vez!";

/**
 * Título da aba na vez do jogador: alterna "Sua vez!" e o título original a
 * cada segundo (`tick` conta os segundos do turno). Fora da vez, o original.
 */
export function turnTabTitle(original: string, myTurn: boolean, tick: number): string {
  if (!myTurn) return original;
  return tick % 2 === 0 ? TURN_TITLE : original;
}
