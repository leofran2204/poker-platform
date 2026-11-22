import { describe, expect, it } from "vitest";
import homeContent from "@/data/homeContent.json";
import {
  demoChipsInPlay,
  demoSeatId,
  demoSeats,
  demoTableState,
  type AnimatedHandData,
} from "./demoTable";

const deflatorHands = homeContent.deflator.demoHands as AnimatedHandData[];
const variantHands = homeContent.variants.map((variant) => variant.demoHand as AnimatedHandData);
const allHands = [...deflatorHands, ...variantHands];
const byId = (id: string) => allHands.find((hand) => hand.id === id)!;

describe("demoTableState — mãos da home na mesa do jogo", () => {
  it.each(allHands.map((hand) => [hand.id, hand] as const))(
    "%s: nenhuma ficha aparece nem some entre o primeiro passo e o pagamento",
    (_id, hand) => {
      const total = demoChipsInPlay(demoTableState(hand, 0, false));
      hand.streets.forEach((_street, step) => {
        expect(demoChipsInPlay(demoTableState(hand, step, false))).toBe(total);
      });
      expect(demoChipsInPlay(demoTableState(hand, hand.streets.length - 1, true))).toBe(total);
    },
  );

  it.each(allHands.map((hand) => [hand.id, hand] as const))(
    "%s: antes do showdown só o herói mostra as cartas",
    (_id, hand) => {
      hand.streets.forEach((_street, step) => {
        const state = demoTableState(hand, step, false);
        expect(state.winners).toEqual([]);
        expect(state.showdown).toEqual([]);
        for (const player of state.players) {
          if (player.id === state.heroId) expect(player.cards.length).toBeGreaterThan(0);
          else expect(player.cards).toEqual([]);
        }
      });
    },
  );

  it.each(allHands.map((hand) => [hand.id, hand] as const))(
    "%s: no showdown o vencedor do roteiro recebe o pote e acende as cinco cartas do jogo",
    (_id, hand) => {
      const seats = demoSeats(hand);
      const winnerIndex = seats.findIndex((seat) => seat.isWinner);
      const state = demoTableState(hand, hand.streets.length - 1, true);
      expect(state.stage).toBe("showdown");
      expect(state.pots).toEqual([]);
      expect(state.winners).toEqual([demoSeatId(winnerIndex)]);
      expect([...state.showdown[0].cards].sort()).toEqual(
        [...hand.winningFive!.hole, ...hand.winningFive!.board].sort(),
      );
      for (const [index, seat] of seats.entries()) {
        const player = state.players[index];
        if (seat.folded) expect(player.cards).toEqual([]);
        else expect(player.cards.length).toBe(hand.id === "omaha-demo" ? 4 : 2);
      }
    },
  );

  it("all-in com 82%: R$ 50 dos R$ 200 voltam para quem perdeu", () => {
    const hand = byId("allin-preflop-aa-kk");
    const state = demoTableState(hand, hand.streets.length - 1, true);
    const [hero, winner, folded] = state.players;
    expect(hero.chips).toBe(5_000);
    expect(winner.chips).toBe(15_000);
    expect(folded.folded).toBe(true);
  });

  it("all-in no flop: fichas na frente antes, zero depois de pagar", () => {
    const hand = byId("allin-flop-set-draw");
    const before = demoTableState(hand, 0, false);
    expect(before.pots[0].amount).toBe(2_000);
    expect(before.players[0].chips).toBe(9_000);
    expect(before.players[1].chips).toBe(9_000);
    const paid = demoTableState(hand, hand.allInStreetIndex!, false);
    expect(paid.pots[0].amount).toBe(hand.potCents);
    expect(paid.players[0].chips).toBe(0);
    expect(paid.players[1].chips).toBe(0);
  });

  it("Brazilian Pineapple: a mesa mostra quem trocou, quem manteve e o descarte", () => {
    const hand = byId("pineapple-demo");
    const draws = (step: number) => demoTableState(hand, step, false).players.map((p) => p.draw_count);
    // Troca e descarte na vez de cada jogador, no mesmo passo das apostas da street.
    expect(draws(1)).toEqual([null, null, null]); // pré-flop: sem troca
    expect(draws(2)).toEqual([1, 1, 1]); // flop: os três trocam na sua vez
    expect(draws(3)).toEqual([0, 1, 0]); // turn: só Leo troca
    expect(draws(4)).toEqual([1, 1, 1]); // river: descarte na vez de cada um
    expect(demoTableState(hand, 4, false).stage).toBe("river");
    expect(draws(5)).toEqual([null, null, null]); // showdown
    // Três cartas até o river, duas depois do descarte.
    expect(demoTableState(hand, 3, false).players[0].cards).toHaveLength(3);
    expect(demoTableState(hand, 4, false).players[0].cards).toHaveLength(2);
  });
});
