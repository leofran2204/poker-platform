import { describe, expect, it } from "vitest";
import { EXTRA_SECONDS, TURN_SECONDS, turnClock, turnTabTitle } from "./turnClock";

describe("turnClock — 15 s normais, 5 s extras e ação automática", () => {
  it("segue a regra 10.2: 15 + 5 segundos por jogada", () => {
    expect(TURN_SECONDS).toBe(15);
    expect(EXTRA_SECONDS).toBe(5);
  });

  it("mostra o tempo normal e o extra que vem depois", () => {
    const clock = turnClock(12, 5, "bet");
    expect(clock.text).toBe("Sua vez: 12s + 5s");
    expect(clock.urgent).toBe(false);
    expect(clock.extraTime).toBe(false);
  });

  it("com tempo extra, os últimos segundos normais não assustam", () => {
    expect(turnClock(3, 5, "bet").urgent).toBe(false);
    expect(turnClock(3, 0, "bet").urgent).toBe(true);
    expect(turnClock(3, 0, "bet").text).toBe("Sua vez: 3s");
  });

  it("acabado o tempo normal, mostra TEMPO EXTRA em vermelho", () => {
    const clock = turnClock(0, 4, "bet");
    expect(clock.text).toBe("TEMPO EXTRA: 4s");
    expect(clock.extraTime).toBe(true);
    expect(clock.urgent).toBe(true);
  });

  it("sem tempo extra, avisa a ação automática de cada rodada", () => {
    expect(turnClock(0, 0, "bet").text).toBe("Fold automático…");
    expect(turnClock(0, 0, "draw").text).toBe("Mantendo as cartas…");
    expect(turnClock(0, 0, "discard").text).toBe("Descartando a carta que sobra…");
  });
});

describe("turnTabTitle — título da aba piscando na vez do jogador", () => {
  it("alterna 'Sua vez!' e o título original a cada segundo", () => {
    expect(turnTabTitle("Mesa 1 | ZT Poker", true, 0)).toBe("Sua vez!");
    expect(turnTabTitle("Mesa 1 | ZT Poker", true, 1)).toBe("Mesa 1 | ZT Poker");
    expect(turnTabTitle("Mesa 1 | ZT Poker", true, 2)).toBe("Sua vez!");
  });

  it("fora da vez, mantém o título original", () => {
    expect(turnTabTitle("Mesa 1 | ZT Poker", false, 0)).toBe("Mesa 1 | ZT Poker");
  });
});
