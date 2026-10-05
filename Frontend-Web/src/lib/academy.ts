import data from "@/data/courseTraining.json";

export type TrainingScenario = typeof data.scenarios[number];
export const TRAINING = data;
export interface StudyAction { action: string; amount?: number; answer?: string }
export interface StudyPlayer {
  id: string; name: string; position: string; stack: number; bet: number; total: number;
  folded: boolean; all_in: boolean; cards: string[]; card_count: number; category: string | null; best_five: string[];
}
export interface StudySnapshot {
  phase: string; board: string[]; players: StudyPlayer[]; pot: number; acting: number; finished: boolean;
}
export interface StudyReview {
  decision: number; action: string; prompt: string; answer: string; expected: string; correct: boolean; explanation: string;
}
export interface StudyResponse {
  version: number; scenario: string; module: string; variant: string; state: StudySnapshot;
  legal: { can_check: boolean; to_call: number; min_total: number; max_total: number; can_raise: boolean; can_all_in: boolean; betting_structure: string; bet_exists: boolean; call_price_percent: number; eligible_pot_after_call: number };
  question: { prompt: string; options: string[]; unit: string } | null;
  hint: string; events: { actor: string; action: string; amount: number; setup: boolean; state: StudySnapshot }[];
  reviews: StudyReview[]; payouts: Record<string, number>; pots: { amount: number; eligible_players: string[] }[];
}
export async function playStudy(scenario: string, seed: number, actions: StudyAction[], signal?: AbortSignal): Promise<StudyResponse> {
  const response = await fetch("/api/academy/play", { method: "POST", headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ version: TRAINING.version, scenario, seed, actions }), signal });
  if (!response.ok) {
    const error = await response.json().catch(() => ({})) as { error?: string };
    throw new Error(error.error ?? "Não foi possível carregar a mesa de estudo.");
  }
  return response.json() as Promise<StudyResponse>;
}
export const actionLabel: Record<string, string> = { fold: "Desistir", check: "Check", call: "Pagar", bet: "Apostar", raise: "Aumentar", all_in: "All-in", deal: "Distribuição e blinds", showdown: "Resultado" };
export const phaseLabel: Record<string, string> = { preflop: "Pré-flop", flop: "Flop", turn: "Turn", river: "River", showdown: "Showdown" };
export const profileLabel: Record<string, string> = { caller: "Pagador", passive: "Passivo", pressure: "Pressão com tamanhos variados", tight: "Seletivo", value: "Check-raise por valor", limper: "Limp no small blind", opener: "Abertura pré-flop" };
export function newStudySeed(): number { return crypto.getRandomValues(new Uint32Array(1))[0]; }

export interface StudyRecord { version: number; id: string; scenario: string; seed: number; when: string; decisions: number; correct: number; notes: string; actions: StudyAction[] }
const STORAGE = "zt-academy-practice-v1";
function validRecord(value: unknown): value is StudyRecord {
  if (!value || typeof value !== "object") return false;
  const r = value as Partial<StudyRecord>;
  return r.version === TRAINING.version && typeof r.id === "string" && r.id.length < 120
    && TRAINING.scenarios.some(s => s.id === r.scenario)
    && Number.isInteger(r.seed) && r.seed! >= 0 && r.seed! <= 0xffffffff
    && typeof r.when === "string" && Number.isFinite(Date.parse(r.when))
    && typeof r.notes === "string" && r.notes.length <= 3000
    && Number.isInteger(r.decisions) && r.decisions! >= 0 && r.decisions! <= 64
    && Number.isInteger(r.correct) && r.correct! >= 0 && r.correct! <= r.decisions!
    && Array.isArray(r.actions) && r.actions.length <= 64 && r.actions.every(a => a && typeof a === "object"
      && ["fold", "check", "call", "bet", "raise", "all_in"].includes(a.action)
      && (a.amount === undefined || Number.isInteger(a.amount) && a.amount >= 0 && a.amount <= 100000)
      && (a.answer === undefined || typeof a.answer === "string" && a.answer.length <= 80));
}
export function readStudyRecords(): StudyRecord[] {
  try { const value: unknown = JSON.parse(localStorage.getItem(STORAGE) ?? "[]"); return Array.isArray(value) ? value.filter(validRecord).slice(0, 50) : []; }
  catch { return []; }
}
export function saveStudyRecord(record: StudyRecord): boolean {
  if (!validRecord(record)) return false;
  try { localStorage.setItem(STORAGE, JSON.stringify([record, ...readStudyRecords().filter(x => x.id !== record.id)].slice(0, 50))); return true; }
  catch { return false; }
}
