import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { allLessons, COURSE } from "./course";
import { newStudySeed, playStudy, readStudyRecords, saveStudyRecord, TRAINING, type StudyRecord } from "./academy";

const storage = new Map<string, string>();
const record = (seed = 1): StudyRecord => ({ version: TRAINING.version, id: `primeira-mao:${seed}`, scenario: "primeira-mao", seed, when: "2026-09-28T12:00:00Z", decisions: 1, correct: 1, notes: "Plano antes de agir", actions: [{ action: "call", answer: "50" }] });
beforeEach(() => { storage.clear(); vi.stubGlobal("localStorage", { getItem: (k: string) => storage.get(k) ?? null, setItem: (k: string, v: string) => storage.set(k, v) }); });
afterEach(() => vi.unstubAllGlobals());

describe("mesas de estudo", () => {
  it("cobre cada módulo e aula com IDs únicos e a modalidade correta", () => {
    expect(new Set(TRAINING.scenarios.map(s => s.id)).size).toBe(TRAINING.scenarios.length);
    for (const m of COURSE.modules) expect(TRAINING.scenarios.some(s => s.module === m.id)).toBe(true);
    for (const l of allLessons()) {
      const matches = TRAINING.scenarios.filter(s => s.lesson === l.id);
      expect(matches.length, l.id).toBeGreaterThan(0);
      for (const s of matches) expect(s.variant, s.id).toBe(l.variant ?? "holdem");
    }
  });
  it("preserva seed, decisões e notas e substitui a mesma sessão", () => {
    expect(saveStudyRecord(record())).toBe(true);
    expect(saveStudyRecord({ ...record(), notes: "Linha alternativa" })).toBe(true);
    expect(readStudyRecords()).toEqual([{ ...record(), notes: "Linha alternativa" }]);
    for (let i = 2; i <= 55; i++) saveStudyRecord(record(i));
    expect(readStudyRecords()).toHaveLength(50);
    expect(readStudyRecords()[0].seed).toBe(55);
  });
  it("descarta registros corrompidos ou de outra versão", () => {
    const invalid = [null, {}, { ...record(), version: -1 }, { ...record(), scenario: "absent" }, { ...record(), seed: -1 }, { ...record(), when: "invalid" }, { ...record(), correct: 2 }, { ...record(), actions: [{ action: "raise", amount: 2.5 }] }, { ...record(), actions: [{ action: "unknown" }] }];
    storage.set("zt-academy-practice-v1", JSON.stringify([...invalid, record()]));
    expect(readStudyRecords()).toEqual([record()]);
    storage.set("zt-academy-practice-v1", "not json");
    expect(readStudyRecords()).toEqual([]);
  });
  it("trata navegador sem armazenamento sem perder a mesa", () => {
    vi.stubGlobal("localStorage", { getItem: () => { throw Error("blocked"); }, setItem: () => { throw Error("blocked"); } });
    expect(readStudyRecords()).toEqual([]);
    expect(saveStudyRecord(record())).toBe(false);
    expect(saveStudyRecord({ ...record(), notes: "x".repeat(3001) })).toBe(false);
  });
  it("envia apenas cenário, seed, versão e decisões e propaga cancelamento", async () => {
    const signal = new AbortController().signal;
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ scenario: "primeira-mao" }) });
    vi.stubGlobal("fetch", fetcher);
    await expect(playStudy("primeira-mao", 42, [], signal)).resolves.toEqual({ scenario: "primeira-mao" });
    expect(fetcher).toHaveBeenCalledWith("/api/academy/play", expect.objectContaining({ signal, body: JSON.stringify({ version: TRAINING.version, scenario: "primeira-mao", seed: 42, actions: [] }) }));
    fetcher.mockResolvedValueOnce({ ok: false, json: async () => ({ error: "Cenário inexistente" }) });
    await expect(playStudy("missing", 42, [])).rejects.toThrow("Cenário inexistente");
  });
  it("gera seeds inteiras compatíveis com u32", () => {
    for (let i = 0; i < 20; i++) { const seed = newStudySeed(); expect(Number.isInteger(seed)).toBe(true); expect(seed).toBeGreaterThanOrEqual(0); expect(seed).toBeLessThanOrEqual(0xffffffff); }
  });
});
