import { createElement, type ComponentProps } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PokerTable } from "./PokerTable";

const base: ComponentProps<typeof PokerTable> = {
  players: [{ id: "me", name: "Eu", chips: 9900, bet: 100, cards: [], is_active: true, is_dealer: true, seat: 0 }],
  localPlayerId: "me", communityCards: [], pots: [{ name: "Main", amount: 200, eligible_players: ["me"] }],
  stage: "preflop", bettingStructure: "pineapple_draw_v4",
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
    expect(html).toContain("Pagar R$ 1,00");
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

  it("na vez do jogador, a troca vem antes da aposta e desistir continua possível", () => {
    const players = [
      { ...base.players[0], cards: ["As", "Ks", "Ah"], bet: 0 },
      { id: "op", name: "Oponente", chips: 9800, bet: 200, cards: [], is_active: false, is_dealer: false, seat: 1, draw_count: 1 },
    ];
    const html = render({ stage: "flop", drawing: true, players, availableActions: ["keep", "swap", "fold"], callAmount: 200, communityCards: ["Ad", "Qc", "2s"] });
    expect(html).toContain("Sua vez: escolha uma carta para trocar ou mantenha as três; depois você aposta.");
    expect(html).toContain("A pagar: R$ 2,00.");
    expect(html).toContain("Manter as três");
    expect(html).toMatch(/<button[^>]*disabled=""[^>]*>Trocar carta/);
    expect(html).toContain(">Desistir<");
    expect(html.match(/Selecionar carta \d para trocar/g)).toHaveLength(3);
    expect(html).toContain("Trocou 1");
    expect(html).not.toContain("Aumentar para");
    const others = players.map((p) => ({ ...p, is_active: p.id === "op" }));
    const waiting = render({ stage: "flop", drawing: true, players: others, availableActions: [] });
    expect(waiting).toContain("Oponente decide a troca antes de apostar.");
    // Enquanto o outro troca, a ação antecipada continua disponível.
    expect(waiting).toContain("Ação antecipada");
  });

  it("no river, a troca vem antes do descarte e o selo conta só a troca", () => {
    const players = [
      { ...base.players[0], cards: ["As", "Ks", "Ah"], bet: 0 },
      { id: "op", name: "Oponente", chips: 9800, bet: 0, cards: [], is_active: false, is_dealer: false, seat: 1, draw_count: 1 },
    ];
    const river = { stage: "river", players, callAmount: 0, communityCards: ["Ad", "Qc", "2s", "7h", "9d"] };
    const swap = render({ ...river, drawing: true, availableActions: ["keep", "swap", "fold"] });
    expect(swap).toContain("Sua vez no river: troque uma carta ou mantenha as três; depois você descarta uma e aposta.");
    expect(swap).toContain("Manter as três");
    expect(swap).toContain("Trocou 1");
    expect(swap).not.toContain("Descartou");
    const discard = render({ ...river, discarding: true, availableActions: ["discard", "fold"] });
    expect(discard).toContain("Agora o descarte do river");
    expect(discard).not.toContain("Manter as três");
    expect(discard.match(/Selecionar carta \d para descartar/g)).toHaveLength(3);
  });

  it("mostra a barra de tempo sob a placa de quem está na vez", () => {
    const players = [
      { ...base.players[0], is_active: false },
      { id: "op", name: "Oponente", chips: 9800, bet: 0, cards: [], is_active: true, is_dealer: false, seat: 1 },
    ];
    // 15 s normais: com 9 s restantes, a barra está em 60%.
    const normal = render({ stage: "flop", players, availableActions: [], seatClock: { left: 9, reserve: 5 } });
    expect(normal.match(/role="timer"/g)).toHaveLength(1);
    expect(normal).toContain("Oponente: 9 segundos para agir");
    expect(normal).toContain("width:60%");
    expect(normal).not.toContain("TEMPO EXTRA");
    // Tempo extra: barra vermelha com o aviso e a contagem dos 5 s.
    const extra = render({ stage: "flop", players, availableActions: [], seatClock: { left: 0, reserve: 4 } });
    expect(extra).toContain('class="zt-seat-timer extra"');
    expect(extra).toContain("Oponente: 4 segundos de tempo extra");
    expect(extra).toContain("width:80%");
    expect(extra).toMatch(/TEMPO EXTRA (<!-- -->)?4(<!-- -->)?s/);
    expect(render({ stage: "flop", players, availableActions: [] })).not.toContain('role="timer"');
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
