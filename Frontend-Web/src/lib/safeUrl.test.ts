import { describe, expect, it } from "vitest";
import { safeHttpUrl } from "./safeUrl";

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
