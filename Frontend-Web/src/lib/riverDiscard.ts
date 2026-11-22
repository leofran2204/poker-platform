/**
 * Brazilian Pineapple (`pineapple_draw_v4`): a carta descartada no river é
 * pública. A mesa a mostra aberta na frente do jogador por alguns segundos e
 * depois a leva fechada para o muck. As cartas trocadas continuam ocultas.
 */

/** Tempo em que a carta descartada fica aberta na frente do jogador. */
export const DISCARD_OPEN_MS = 2000;
/** Duração da ida da carta (fechada) até o muck. */
export const DISCARD_MUCK_MS = 600;

export interface RiverDiscardSeat {
  id: string;
  river_discard?: string | null;
}

/**
 * Compara o estado novo com os descartes já mostrados (`seen`, por jogador) e
 * devolve só os que acabaram de acontecer. Atualiza `seen`: quem ficou sem
 * descarte (mão nova) sai do mapa, para que o próximo descarte apareça.
 */
export function freshRiverDiscards(
  seen: Map<string, string>,
  players: RiverDiscardSeat[],
): { playerId: string; code: string }[] {
  const fresh: { playerId: string; code: string }[] = [];
  const present = new Set<string>();
  for (const player of players) {
    const code = player.river_discard ?? null;
    if (!code) {
      seen.delete(player.id);
      continue;
    }
    present.add(player.id);
    if (seen.get(player.id) === code) continue;
    seen.set(player.id, code);
    fresh.push({ playerId: player.id, code });
  }
  for (const id of [...seen.keys()]) {
    if (!present.has(id)) seen.delete(id);
  }
  return fresh;
}
