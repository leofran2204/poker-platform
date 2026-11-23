import { describe, expect, it } from "vitest";
import type { TournamentInfoResponse } from "@/api/types";
import { readRebuyAddon } from "./AdminTournamentsPage";
import { addonRuleText, rebuyRuleText } from "./TournamentPage";

const base = {
  allow_rebuy: true,
  rebuy_cost: 1_000,
  rebuy_chips: 10_000,
  rebuy_max_count: 1,
  rebuy_stack_threshold: 0,
  rebuy_max_level: 6,
  allow_addon: true,
  addon_cost: 2_000,
  addon_chips: 15_000,
  addon_max_level: 6,
} as TournamentInfoResponse;

describe("regras de rebuy e add-on na página do torneio", () => {
  it("mostra quantidade, nível, preço e fichas, sem taxa", () => {
    const text = rebuyRuleText(base);
    expect(text).toContain("1×");
    expect(text).toContain("até o nível 6");
    expect(text).toContain("após eliminação");
    expect(text).toContain("10.000 fichas");
    expect(text).toContain("sem taxa");
  });

  it("quantidade 0 é sem limite e o limiar de stack aparece", () => {
    const text = rebuyRuleText({ ...base, rebuy_max_count: 0, rebuy_stack_threshold: 5_000 });
    expect(text).toContain("Sem limite de vezes");
    expect(text).toContain("stack até 5.000");
  });

  it("add-on é uma compra por jogador; desligado mostra Não", () => {
    expect(addonRuleText(base)).toContain("1× por jogador, até o nível 6");
    expect(addonRuleText({ ...base, allow_addon: false })).toBe("Não");
    expect(rebuyRuleText({ ...base, allow_rebuy: false })).toBe("Não");
  });
});

describe("formulário de rebuy e add-on do admin", () => {
  it("converte reais em centavos e deixa campos vazios de fora", () => {
    const fd = new FormData();
    fd.set("allow_rebuy", "on");
    fd.set("rebuy_cost", "12.5");
    fd.set("rebuy_chips", "");
    fd.set("rebuy_max_count", "0");
    fd.set("rebuy_max_level", "6");
    fd.set("addon_cost", "20");
    fd.set("addon_max_level", "");
    const input = readRebuyAddon(fd);
    expect(input.allow_rebuy).toBe(true);
    expect(input.allow_addon).toBe(false);
    expect(input.rebuy_cost_cents).toBe(1_250);
    expect(input.addon_cost_cents).toBe(2_000);
    expect(input.rebuy_max_count).toBe(0);
    expect(JSON.parse(JSON.stringify(input))).not.toHaveProperty("rebuy_chips");
    expect(JSON.parse(JSON.stringify(input))).not.toHaveProperty("addon_max_level");
  });
});
