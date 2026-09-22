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
    expect(tierForEquity(85)?.percent).toBe(25);
    expect(tierForEquity(86)?.percent).toBe(35);
    expect(tierForEquity(99.9)?.percent).toBe(35);
  });

  it("100% não tem tier: quem tem 100% não perde", () => {
    expect(tierForEquity(100)).toBeNull();
    expect(cashbackFor(100, 20000).cashbackCents).toBe(0);
    expect(cashbackFor(100, 20000).percent).toBe(0);
  });
});

describe("cashbackFor", () => {
  it("AA vs KK no pré-flop (82%): 25% de R$ 200 = R$ 50", () => {
    const r = cashbackFor(82, 20000);
    expect(r.percent).toBe(25);
    expect(r.cashbackCents).toBe(5000);
  });

  it("trinca vs flush draw no flop (68%): 15% de R$ 200 = R$ 30", () => {
    const r = cashbackFor(68, 20000);
    expect(r.percent).toBe(15);
    expect(r.cashbackCents).toBe(3000);
  });

  it("flush vs trinca no turn (77%): 25% de R$ 200 = R$ 50 — fase não muda a faixa", () => {
    const r = cashbackFor(77, 20000);
    expect(r.percent).toBe(25);
    expect(r.cashbackCents).toBe(5000);
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
