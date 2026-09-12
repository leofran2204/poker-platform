import { describe, expect, it } from "vitest";
import { cashbackFor, tierForEquity } from "./deflator";

describe("tierForEquity", () => {
  it("mapeia as 4 faixas normativas", () => {
    expect(tierForEquity(88)?.percent).toBe(35);
    expect(tierForEquity(80)?.percent).toBe(25);
    expect(tierForEquity(70)?.percent).toBe(15);
    expect(tierForEquity(60)?.percent).toBe(7);
  });

  it("bordas: 56 entra, abaixo de 56 não", () => {
    expect(tierForEquity(56)?.percent).toBe(7);
    expect(tierForEquity(55.9)).toBeNull();
    expect(tierForEquity(0)).toBeNull();
  });

  it("bordas entre faixas", () => {
    expect(tierForEquity(66)?.percent).toBe(15);
    expect(tierForEquity(76)?.percent).toBe(25);
    expect(tierForEquity(86)?.percent).toBe(35);
    expect(tierForEquity(100)?.percent).toBe(35);
  });
});

describe("cashbackFor", () => {
  it("AA vs KK (82%): 25% de R$ 200 = R$ 50", () => {
    const r = cashbackFor(82, 20000);
    expect(r.percent).toBe(25);
    expect(r.cashbackCents).toBe(5000);
  });

  it("AQ vs KJ (62%): 7% de R$ 400 = R$ 28", () => {
    const r = cashbackFor(62, 40000);
    expect(r.percent).toBe(7);
    expect(r.cashbackCents).toBe(2800);
  });

  it("abaixo do mínimo não devolve", () => {
    expect(cashbackFor(54, 20000).cashbackCents).toBe(0);
    expect(cashbackFor(54, 20000).percent).toBe(0);
  });

  it("pote zerado ou inválido zera", () => {
    expect(cashbackFor(90, 0).cashbackCents).toBe(0);
    expect(cashbackFor(90, -100).cashbackCents).toBe(0);
  });
});
