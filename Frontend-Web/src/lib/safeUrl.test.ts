import { describe, expect, it } from "vitest";
import { safeHttpUrl, safeInternalPath } from "./safeUrl";

describe("safeHttpUrl", () => {
  it("aceita https", () => {
    expect(safeHttpUrl("https://zerotiltpoker.net/lobby")).toBe(
      "https://zerotiltpoker.net/lobby",
    );
  });

  it("rejeita javascript e data", () => {
    expect(safeHttpUrl("javascript:alert(1)")).toBeNull();
    expect(safeHttpUrl("data:text/html,x")).toBeNull();
  });

  it("rejeita protocolo relativo", () => {
    expect(safeHttpUrl("//evil.example/phish")).toBeNull();
  });

  it("rejeita vazio", () => {
    expect(safeHttpUrl("")).toBeNull();
    expect(safeHttpUrl(null)).toBeNull();
  });
});

describe("safeInternalPath", () => {
  it("preserva rota, query e hash internos", () => {
    expect(safeInternalPath("/lobby?mode=real#cash", "/curso")).toBe(
      "/lobby?mode=real#cash",
    );
  });

  it("rejeita URLs absolutas e relativas a protocolo", () => {
    expect(safeInternalPath("https://evil.example", "/curso")).toBe("/curso");
    expect(safeInternalPath("//evil.example", "/curso")).toBe("/curso");
  });

  it("rejeita barras invertidas literais ou codificadas", () => {
    expect(safeInternalPath("/\\evil.example", "/curso")).toBe("/curso");
    expect(safeInternalPath("/%5cevil.example", "/curso")).toBe("/curso");
  });

  it("usa fallback para valor vazio ou caractere de controle", () => {
    expect(safeInternalPath(null, "/curso")).toBe("/curso");
    expect(safeInternalPath("/lobby\n/evil", "/curso")).toBe("/curso");
  });
});
