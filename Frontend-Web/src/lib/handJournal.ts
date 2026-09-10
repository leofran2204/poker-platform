/** Diário de mãos local (navegador): registra SUAS mãos com suas cartas para
 * estudo posterior. O servidor guarda o histórico sem as cartas fechadas,
 * então o registro acontece ao vivo, na sua máquina — nada sai daqui. */

export interface JournalBet {
  name: string;
  bet: number;
  folded: boolean;
  chips: number;
}

export interface HandSnapshot {
  street: string;
  board: string[];
  heroCards: string[];
  pot: number;
  bets: JournalBet[];
  winners: string[];
}

export interface JournalShowdown {
  name: string;
  hand: string | null;
  cards: string[];
}

export interface JournalHand {
  key: number;
  tableId: string;
  tableName: string;
  startedAt: number;
  endedAt: number;
  heroName: string;
  winners: string[];
  winningHand: string | null;
  snapshots: HandSnapshot[];
  showdown: JournalShowdown[];
}

const MAX_HANDS = 100;
const MAX_SNAPS = 40;

const storeKey = (tableId: string) => `zt-hands-${tableId}`;

export function loadHands(tableId: string): JournalHand[] {
  try {
    const raw = localStorage.getItem(storeKey(tableId));
    if (!raw) return [];
    const parsed = JSON.parse(raw) as JournalHand[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function saveHands(tableId: string, hands: JournalHand[]): void {
  try {
    localStorage.setItem(storeKey(tableId), JSON.stringify(hands));
  } catch {
    // Quota cheia: mantém só as 20 mais recentes e tenta de novo.
    try {
      localStorage.setItem(storeKey(tableId), JSON.stringify(hands.slice(0, 20)));
    } catch {
      /* sem espaço: ignora */
    }
  }
}

export function appendHand(hand: JournalHand): JournalHand[] {
  const all = [hand, ...loadHands(hand.tableId)].slice(0, MAX_HANDS);
  saveHands(hand.tableId, all);
  return all;
}

export function clearHands(tableId: string): void {
  try {
    localStorage.removeItem(storeKey(tableId));
  } catch {
    /* ignora */
  }
}

export function pushSnapshot(snaps: HandSnapshot[], snap: HandSnapshot): HandSnapshot[] {
  const last = snaps[snaps.length - 1];
  if (last && snapshotKey(last) === snapshotKey(snap)) return snaps;
  const next = [...snaps, snap];
  return next.length > MAX_SNAPS ? next.slice(next.length - MAX_SNAPS) : next;
}

export function snapshotKey(s: HandSnapshot): string {
  return JSON.stringify([
    s.street,
    s.board,
    s.heroCards,
    s.pot,
    s.bets.map((b) => [b.name, b.bet, b.folded]),
  ]);
}

const STREET_PT: Record<string, string> = {
  waiting: "Aguardando",
  preflop: "Pré-flop",
  flop: "Flop",
  turn: "Turn",
  river: "River",
  showdown: "Showdown",
};

export function streetPt(street: string): string {
  return STREET_PT[street] ?? street;
}

function fmtCents(cents: number): string {
  const sign = cents < 0 ? "-" : "";
  const abs = Math.abs(Math.trunc(cents));
  return `${sign}R$ ${Math.floor(abs / 100).toLocaleString("pt-BR")},${(abs % 100)
    .toString()
    .padStart(2, "0")}`;
}

/** Versão legível para estudo (TXT). */
export function handToText(h: JournalHand): string {
  const lines: string[] = [];
  lines.push(`Zero Tilt — mão de ${h.heroName}`);
  lines.push(`Mesa: ${h.tableName} — ${new Date(h.endedAt).toLocaleString("pt-BR")}`);
  lines.push(`Vencedor: ${h.winners.join(" + ")}${h.winningHand ? ` com ${h.winningHand}` : ""}`);
  lines.push("");
  h.snapshots.forEach((s, i) => {
    lines.push(`[${i + 1}] ${streetPt(s.street)} — board: ${s.board.join(" ") || "—"} — pote: ${fmtCents(s.pot)}`);
    lines.push(`    Minhas cartas: ${s.heroCards.join(" ") || "—"}`);
    for (const b of s.bets) {
      lines.push(`    ${b.name}: aposta ${fmtCents(b.bet)}${b.folded ? " (fold)" : ""} — stack ${fmtCents(b.chips)}`);
    }
  });
  if (h.showdown.length > 0) {
    lines.push("");
    lines.push("Showdown:");
    for (const e of h.showdown) {
      lines.push(`  ${e.name}${e.hand ? ` (${e.hand})` : ""}: ${e.cards.join(" ")}`);
    }
  }
  return lines.join("\n");
}

export function downloadFile(filename: string, text: string, mime = "text/plain;charset=utf-8"): void {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 2000);
}
