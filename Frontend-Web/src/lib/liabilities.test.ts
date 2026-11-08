import { describe, expect, it } from "vitest";
import { computeCoverage } from "./liabilities";

describe("computeCoverage", () => {
  it("saldo maior ou igual ao devido = coberto, com a sobra positiva", () => {
    expect(computeCoverage(50_000, 42_500)).toEqual({ state: "covered", differenceCents: 7_500, ratio: 50_000 / 42_500 });
    expect(computeCoverage(42_500, 42_500).state).toBe("covered");
  });
  it("saldo menor que o devido = falta, com a diferença negativa", () => {
    expect(computeCoverage(40_000, 42_500)).toEqual({ state: "short", differenceCents: -2_500, ratio: 40_000 / 42_500 });
  });
  it("nada devido: coberto e sem razão percentual", () => {
    expect(computeCoverage(0, 0)).toEqual({ state: "covered", differenceCents: 0, ratio: null });
  });
});
