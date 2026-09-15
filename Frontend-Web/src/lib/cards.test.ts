import { describe, expect, it } from "vitest";
import {
  bareAceNeighborIsCard,
  extractConcreteCardCodes,
  isBareAceWord,
} from "./cards";

describe("isBareAceWord", () => {
  it("marca As/Ah/Ad/Ac exatos como ambíguos", () => {
    for (const t of ["As", "Ah", "Ad", "Ac"]) expect(isBareAceWord(t)).toBe(true);
  });

  it("não marca o resto (inclui forma explícita e minúscula)", () => {
    for (const t of ["as", "Ks", "A[s]", "AA", "AsKs", "Xx", "10h"]) {
      expect(isBareAceWord(t)).toBe(false);
    }
  });
});

describe("bareAceNeighborIsCard", () => {
  it("'As cartas' (artigo) não tem carta ao lado", () => {
    expect(bareAceNeighborIsCard(undefined, "cartas")).toBe(false);
  });

  it("'As Únicas' (título) não tem carta ao lado", () => {
    expect(bareAceNeighborIsCard(undefined, "Únicas")).toBe(false);
  });

  it("'As Kh' tem carta ao lado (vale Ás de espadas)", () => {
    expect(bareAceNeighborIsCard("tem", "Kh")).toBe(true);
    expect(bareAceNeighborIsCard("As", undefined)).toBe(true); // par "As Ah": um avaliza o outro
  });

  it("vizinho com pontuação conta ('As,' ao lado de 'Ks,')", () => {
    expect(bareAceNeighborIsCard("card:", "Ks,")).toBe(true);
  });

  it("ranges não avalizam ('As' em 'As Xx' sozinho)", () => {
    expect(bareAceNeighborIsCard("Blockers:", "Xx")).toBe(false);
  });
});

describe("extractConcreteCardCodes (inalterado)", () => {
  it("forma explícita A[s] sempre vale", () => {
    expect(extractConcreteCardCodes("A[s]")).toEqual(["As"]);
  });

  it("par e colados continuam valendo", () => {
    expect(extractConcreteCardCodes("As")).toEqual(["As"]);
    expect(extractConcreteCardCodes("AsKs")).toEqual(["As", "Ks"]);
  });

  it("ranges continuam fora", () => {
    expect(extractConcreteCardCodes("A9s")).toEqual([]);
    expect(extractConcreteCardCodes("KTo")).toEqual([]);
    expect(extractConcreteCardCodes("AA")).toEqual([]);
  });
});
