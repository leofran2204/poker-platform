import { describe, expect, it } from "vitest";
import { ApiError } from "./errors";
import { fatalWsTicketMessage, isFatalWsTicketError } from "./wsFatal";

describe("isFatalWsTicketError", () => {
  it("401 e 403 são finais", () => {
    expect(isFatalWsTicketError(new ApiError("x", 401))).toBe(true);
    expect(isFatalWsTicketError(new ApiError("x", 403))).toBe(true);
    expect(isFatalWsTicketError(new ApiError("x", 500))).toBe(false);
    expect(isFatalWsTicketError(new Error("net"))).toBe(false);
  });
});

describe("fatalWsTicketMessage", () => {
  it("explica 403 sem assento", () => {
    expect(fatalWsTicketMessage(new ApiError("no", 403))).toMatch(/não está mais nesta mesa/);
  });

  it("repassa 401", () => {
    expect(fatalWsTicketMessage(new ApiError("Sessão expirada", 401))).toBe("Sessão expirada");
  });
});
