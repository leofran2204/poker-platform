import { describe, expect, it } from "vitest";
import { formatBrlFromCents, parseBrlToCents } from "./money";

describe("formatBrlFromCents", () => {
  it("formata centavos em pt-BR", () => {
    expect(formatBrlFromCents(150)).toBe("R$ 1,50");
    expect(formatBrlFromCents(0)).toBe("R$ 0,00");
    expect(formatBrlFromCents(-2500)).toBe("-R$ 25,00");
  });
});

describe("parseBrlToCents", () => {
  it("lê vírgula brasileira", () => {
    expect(parseBrlToCents("R$ 1,50")).toBe(150);
    expect(parseBrlToCents("1.234,56")).toBe(123456);
  });

  it("lê ponto decimal", () => {
    expect(parseBrlToCents("1.50")).toBe(150);
  });

  it("rejeita inválido", () => {
    expect(parseBrlToCents("")).toBeNull();
    expect(parseBrlToCents("-1")).toBeNull();
    expect(parseBrlToCents("abc")).toBeNull();
  });
});
