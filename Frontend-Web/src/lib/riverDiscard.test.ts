import { describe, expect, it } from "vitest";
import { DISCARD_OPEN_MS, freshRiverDiscards } from "./riverDiscard";

describe("freshRiverDiscards — descarte do river aberto para a mesa", () => {
  it("mostra a carta aberta por 2 segundos", () => {
    expect(DISCARD_OPEN_MS).toBe(2000);
  });

  it("avisa só o descarte novo, uma vez", () => {
    const seen = new Map<string, string>();
    expect(freshRiverDiscards(seen, [{ id: "ana" }, { id: "caio", river_discard: null }])).toEqual([]);
    expect(freshRiverDiscards(seen, [{ id: "ana", river_discard: "7c" }, { id: "caio" }])).toEqual([
      { playerId: "ana", code: "7c" },
    ]);
    // O mesmo estado de novo (outra ação na mesa) não repete a animação.
    expect(freshRiverDiscards(seen, [{ id: "ana", river_discard: "7c" }, { id: "caio" }])).toEqual([]);
    expect(
      freshRiverDiscards(seen, [
        { id: "ana", river_discard: "7c" },
        { id: "caio", river_discard: "Kd" },
      ]),
    ).toEqual([{ playerId: "caio", code: "Kd" }]);
  });

  it("mão nova limpa a memória: o mesmo descarte volta a aparecer", () => {
    const seen = new Map<string, string>();
    freshRiverDiscards(seen, [{ id: "ana", river_discard: "7c" }]);
    freshRiverDiscards(seen, [{ id: "ana", river_discard: null }]);
    expect(freshRiverDiscards(seen, [{ id: "ana", river_discard: "7c" }])).toEqual([
      { playerId: "ana", code: "7c" },
    ]);
  });

  it("quem saiu da mesa sai da memória", () => {
    const seen = new Map<string, string>();
    freshRiverDiscards(seen, [{ id: "ana", river_discard: "7c" }]);
    freshRiverDiscards(seen, []);
    expect(seen.size).toBe(0);
  });
});
