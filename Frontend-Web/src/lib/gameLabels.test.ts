import { describe, expect, it } from "vitest";
import { gameNameLabel, handNamePt, tournamentStatusLabel } from "./gameLabels";
import { isAdminRole } from "./me";

describe("gameNameLabel", () => {
  it("distingue Short Deck de Hold’em", () => {
    expect(gameNameLabel({ poker_variant: "holdem" }, "cash")).toBe(
      "Texas Hold’em — Cash Game",
    );
    expect(gameNameLabel({ poker_variant: "short_deck" }, "cash")).toBe(
      "Texas Hold’em Short Deck — Cash Game",
    );
  });

  it("nomeia Omaha e Pineapple", () => {
    expect(gameNameLabel({ poker_variant: "short_deck_omaha" }, "tournament")).toBe(
      "Omaha 4 Cartas — Torneio",
    );
    expect(gameNameLabel({ poker_variant: "ultimate_pineapple" }, "cash")).toBe(
      "Ultimate Pineapple — Cash Game",
    );
  });
});

describe("handNamePt", () => {
  it("traduz mãos do motor", () => {
    expect(handNamePt("Three of a Kind")).toBe("Trinca");
    expect(handNamePt(null)).toBe("a melhor mão");
  });
});

describe("tournamentStatusLabel", () => {
  it("traduz status", () => {
    expect(tournamentStatusLabel("registering")).toBe("Inscrições abertas");
  });
});

describe("isAdminRole", () => {
  it("só admin", () => {
    expect(isAdminRole("admin")).toBe(true);
    expect(isAdminRole("player")).toBe(false);
    expect(isAdminRole(null)).toBe(false);
  });
});
