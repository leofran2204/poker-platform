import { describe, expect, it } from "vitest";
import { turnClock } from "./turnClock";

describe("turnClock — tempo normal, banco de tempo e ação automática", () => {
  it("mostra o tempo normal e a reserva disponível", () => {
    const clock = turnClock(22, 30, "bet");
    expect(clock.text).toBe("Sua vez: 22s · reserva 30s");
    expect(clock.urgent).toBe(false);
    expect(clock.usingReserve).toBe(false);
  });

  it("com reserva, os últimos segundos do tempo normal não assustam", () => {
    expect(turnClock(5, 30, "bet").urgent).toBe(false);
    expect(turnClock(5, 0, "bet").urgent).toBe(true);
    expect(turnClock(5, 0, "bet").text).toBe("Sua vez: 5s");
  });

  it("passa para o banco de tempo quando o tempo normal acaba", () => {
    const clock = turnClock(0, 18, "bet");
    expect(clock.text).toBe("Banco de tempo: 18s");
    expect(clock.usingReserve).toBe(true);
    expect(clock.urgent).toBe(false);
    expect(turnClock(0, 9, "bet").urgent).toBe(true);
  });

  it("sem tempo e sem reserva, avisa a ação automática de cada rodada", () => {
    expect(turnClock(0, 0, "bet").text).toBe("Fold automático…");
    expect(turnClock(0, 0, "draw").text).toBe("Mantendo as cartas…");
    expect(turnClock(0, 0, "discard").text).toBe("Descartando a carta que sobra…");
  });
});
