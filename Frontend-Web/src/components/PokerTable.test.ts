import { createElement, type ComponentProps } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PokerTable } from "./PokerTable";

const base: ComponentProps<typeof PokerTable> = {
  players: [{ id: "me", name: "Eu", chips: 9900, bet: 100, cards: [], is_active: true, is_dealer: true, seat: 0 }],
  localPlayerId: "me", communityCards: [], pots: [{ name: "Main", amount: 200, eligible_players: ["me"] }],
  stage: "preflop", bettingStructure: "pineapple_draw_v1",
  availableActions: ["fold", "check", "raise"], callAmount: 0,
  minimumWager: 200, maximumWager: 350, raiseAmount: 350,
  onRaiseChange: () => {}, onAction: () => {},
};
const render = (props: Partial<typeof base> = {}) => renderToStaticMarkup(createElement(PokerTable, { ...base, ...props }));

describe("contrato de apostas do Brazilian Pineapple na mesa", () => {
  it("distingue ante nominal, parcial e pagador sem abater o call", () => {
    const html = render({ ante: 100, antePaid: 50, antePlayerId: "me", availableActions: ["fold", "call"], callAmount: 100 });
    expect(html).toContain("Ante do BB R$ 1,00");
    expect(html).toContain("pago R$ 0,50 por Eu");
    expect(html).toContain("Call R$ 1,00");
  });
  it("oferece o seletor pré-flop dentro dos limites enviados pelo motor", () => {
    const html = render();
    expect(html).toContain("Todas as rodadas");
    expect(html).toContain('max="3.5"');
    expect(html).toContain("Aumentar para R$ 3,50");
    expect(html).toContain('type="number"');
    expect(html).not.toContain("All-in");
  });

  it("limita o seletor pós-flop e bloqueia um total acima do máximo", () => {
    const html = render({ stage: "flop", minimumWager: 1000, maximumWager: 2000, callAmount: 500, raiseAmount: 2001 });
    expect(html).toContain("aumento limitado ao pote antes do call");
    expect(html).toContain('max="20"');
    expect(html).toContain("Máximo");
    expect(html).toMatch(/<button[^>]*disabled=""[^>]*>Aumentar para/);
    expect(html).not.toContain("All-in");
  });

  it("na rodada de troca mostra as três cartas, manter e trocar, sem apostas", () => {
    const players = [
      { ...base.players[0], cards: ["As", "Ks", "Ah"], bet: 0 },
      { id: "op", name: "Oponente", chips: 9800, bet: 0, cards: [], is_active: false, is_dealer: false, seat: 1, draw_count: 1 },
    ];
    const html = render({ stage: "flop", drawing: true, players, availableActions: ["keep", "swap"], communityCards: ["Ad", "Qc", "2s"] });
    expect(html).toContain("Rodada de troca: escolha uma carta para trocar ou mantenha as três.");
    expect(html).toContain("Manter as três");
    expect(html).toMatch(/<button[^>]*disabled=""[^>]*>Trocar carta/);
    expect(html.match(/Selecionar carta \d para trocar/g)).toHaveLength(3);
    expect(html).toContain("Trocou 1");
    expect(html).not.toContain(">Fold<");
    expect(html).not.toContain("Aumentar para");
    const waiting = render({ stage: "flop", drawing: true, players, availableActions: [] });
    expect(waiting).toContain("Rodada de troca: cada jogador mantém as cartas ou troca uma.");
  });

  it("exibe all-in curto somente quando o motor permite e oculta aumento no teto", () => {
    const short = render({ availableActions: ["fold", "call", "allin"], minimumWager: 0, maximumWager: 0 });
    expect(short).toContain("All-in");
    expect(short).not.toContain("Aumentar para");
    const capped = render({ availableActions: ["fold", "call"], minimumWager: 0, maximumWager: 0 });
    expect(capped).not.toContain("All-in");
    expect(capped).not.toContain("Aumentar para");
  });
});
