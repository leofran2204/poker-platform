import { useEffect, useState } from "react";
import type { PlayerWsData, PotWsData, ShowdownEntry } from "@/api/types";
import { PokerTable } from "@/components/PokerTable";
import { ShowcaseFrame } from "@/components/home/ShowcaseFrame";
import "./live-hand.css";

// Mesa viva da home: a mão-modelo dos vídeos do Brazilian Pineapple se jogando
// sozinha, com a mesma mesa e as mesmas animações do jogo. Os valores e as cartas
// são os dos vídeos (conferidos no motor em pineapple_draw_film_examples): sem ante,
// troca na vez de cada jogador no flop, no turn e no river; no river, descarte
// depois da troca, com a carta descartada aberta por 2 s para todos (regra
// pineapple_draw_v4). Embaixo da mesa, os botões da vez, com o escolhido apertado,
// como nos vídeos, e a barra de tempo (15 s + 5 s de tempo extra) sob a placa de
// quem está na vez (decisões do dono em 21/11/2026).

type Tone = "fold" | "call" | "raise" | "draw";
type Bar = { who: string; options: [string, Tone][]; pressed?: string };

type Step = {
  caption: string;
  stage: string;
  board: string[];
  /** Pote total (centro + apostas da rodada), como o servidor envia. */
  pot: number;
  /** [Ana, Beto, Caio, Duda] */
  bets: [number, number, number, number];
  stacks: [number, number, number, number];
  hero: string[];
  folded?: number[];
  active?: number;
  drawing?: boolean;
  discarding?: boolean;
  drew?: (number | null)[];
  /** Carta descartada no river, pública: a mesa a mostra aberta e leva ao muck. [Ana, Caio] */
  discards?: [string | null, string | null];
  showdown?: boolean;
  bar?: Bar;
};

const FOLD: [string, Tone] = ["Fold", "fold"];
const KEEP: [string, Tone] = ["Manter", "draw"];
const SWAP: [string, Tone] = ["Trocar", "draw"];
const CHECK: [string, Tone] = ["Check", "call"];
const call = (amount: string): [string, Tone] => [`Call ${amount}`, "call"];
const raise = (label = "Raise"): [string, Tone] => [label, "raise"];
const bet = (label = "Aposta"): [string, Tone] => [label, "raise"];

const FLOP = ["As", "Qd", "6h"];
const TURN = [...FLOP, "Kh"];
const RIVER = [...TURN, "6s"];
const STEPS: Step[] = [
  { caption: "Três cartas para cada jogador. Só os blinds, sem ante. Caio fala primeiro: 15 s para agir, mais 5 s de tempo extra.", stage: "preflop", board: [], pot: 150, bets: [50, 100, 0, 0], stacks: [9950, 9900, 10000, 10000], hero: ["Ah", "Kd", "7c"], active: 2, bar: { who: "Caio", options: [FOLD, call("R$ 1"), raise()] } },
  { caption: "Caio aumenta para R$ 2,50, o máximo pelo limite do pote. Duda paga.", stage: "preflop", board: [], pot: 650, bets: [50, 100, 250, 250], stacks: [9950, 9900, 9750, 9750], hero: ["Ah", "Kd", "7c"], active: 0, bar: { who: "Duda", options: [FOLD, call("R$ 2,50"), raise()], pressed: "Call R$ 2,50" } },
  { caption: "Ana dá reraise até R$ 9, e todos pagam.", stage: "preflop", board: [], pot: 3600, bets: [900, 900, 900, 900], stacks: [9100, 9100, 9100, 9100], hero: ["Ah", "Kd", "7c"], bar: { who: "Ana", options: [FOLD, call("R$ 2"), raise("Raise R$ 9")], pressed: "Raise R$ 9" } },
  { caption: "Flop. Na sua vez, Ana decide a troca antes de apostar.", stage: "flop", board: FLOP, pot: 3600, bets: [0, 0, 0, 0], stacks: [9100, 9100, 9100, 9100], hero: ["Ah", "Kd", "7c"], drawing: true, active: 0, bar: { who: "Ana", options: [KEEP, SWAP] } },
  { caption: "Ana troca o sete, recebe outro ás e aposta R$ 12.", stage: "flop", board: FLOP, pot: 4800, bets: [1200, 0, 0, 0], stacks: [7900, 9100, 9100, 9100], hero: ["Ah", "Kd", "Ac"], drew: [1, null, null, null], drawing: true, active: 1, bar: { who: "Ana", options: [CHECK, bet("Aposta R$ 12")], pressed: "Aposta R$ 12" } },
  { caption: "Beto e Caio trocam uma e pagam. Duda desiste sem trocar.", stage: "flop", board: FLOP, pot: 7200, bets: [1200, 1200, 1200, 0], stacks: [7900, 7900, 7900, 9100], hero: ["Ah", "Kd", "Ac"], folded: [3], drew: [1, 1, 1, null], bar: { who: "Duda", options: [FOLD, KEEP, SWAP], pressed: "Fold" } },
  { caption: "Turn. Ana e Beto mantêm e passam; Caio mantém e aposta R$ 30.", stage: "turn", board: TURN, pot: 10200, bets: [0, 0, 3000, 0], stacks: [7900, 7900, 4900, 9100], hero: ["Ah", "Kd", "Ac"], folded: [3], drew: [0, 0, 0, null], active: 0, bar: { who: "Caio", options: [CHECK, bet("Aposta R$ 30")], pressed: "Aposta R$ 30" } },
  { caption: "Ana já decidiu a troca: agora só paga. Beto desiste.", stage: "turn", board: TURN, pot: 13200, bets: [3000, 0, 3000, 0], stacks: [4900, 7900, 4900, 9100], hero: ["Ah", "Kd", "Ac"], folded: [1, 3], drew: [0, 0, 0, null], bar: { who: "Ana", options: [FOLD, call("R$ 30"), raise()], pressed: "Call R$ 30" } },
  { caption: "River. Ana ia descartar o rei, então troca o rei antes: a carta nova não custa nada.", stage: "river", board: RIVER, pot: 13200, bets: [0, 0, 0, 0], stacks: [4900, 7900, 4900, 9100], hero: ["Ah", "2c", "Ac"], folded: [1, 3], drew: [1, null, null, null], discarding: true, active: 0, bar: { who: "Ana", options: [KEEP, SWAP], pressed: "Trocar" } },
  { caption: "Veio um dois. Ana descarta o dois, aberto para todos, fica com dois ases e aposta R$ 35.", stage: "river", board: RIVER, pot: 16700, bets: [3500, 0, 0, 0], stacks: [1400, 7900, 4900, 9100], hero: ["Ah", "Ac"], folded: [1, 3], drew: [1, null, null, null], discards: ["2c", null], drawing: true, active: 2, bar: { who: "Ana", options: [CHECK, bet("Aposta R$ 35")], pressed: "Aposta R$ 35" } },
  { caption: "Caio troca uma carta, descarta o quatro, também aberto, e paga.", stage: "river", board: RIVER, pot: 20200, bets: [3500, 0, 3500, 0], stacks: [1400, 7900, 1400, 9100], hero: ["Ah", "Ac"], folded: [1, 3], drew: [1, null, 1, null], discards: ["2c", "4s"], bar: { who: "Caio", options: [FOLD, call("R$ 35"), raise()], pressed: "Call R$ 35" } },
  { caption: "Showdown: full house de ases com seis vence a sequência. O pote vai para Ana.", stage: "showdown", board: RIVER, pot: 20200, bets: [0, 0, 0, 0], stacks: [1400, 7900, 1400, 9100], hero: ["Ah", "Ac"], folded: [1, 3], discards: ["2c", "4s"], showdown: true },
  { caption: "Showdown: full house de ases com seis vence a sequência. O pote vai para Ana.", stage: "showdown", board: RIVER, pot: 0, bets: [0, 0, 0, 0], stacks: [21600, 7900, 1400, 9100], hero: ["Ah", "Ac"], folded: [1, 3], discards: ["2c", "4s"], showdown: true },
];
const NAMES = ["Ana", "Beto", "Caio", "Duda"];
/** Lugares na mesa de seis: embaixo, alto à esquerda, topo e alto à direita. */
const SEATS = [0, 2, 3, 4];
const STEP_MS = 2600;
/** Segundos que sobram no relógio de quem está na vez (varia por passo, só para a demonstração). */
const CLOCK_LEFT = [12, 9, 13, 11];
/** A mesa é desenhada nesta largura e reduzida por escala para caber no espaço (`ShowcaseFrame`). */
const DESIGN_WIDTH = 940;
const DESIGN_HEIGHT = 540;

export function LiveHandShowcase() {
  const [index, setIndex] = useState(0);
  const [still, setStill] = useState(false);
  useEffect(() => {
    // Quem pediu menos movimento vê o showdown parado, sem o ciclo.
    if (typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setStill(true);
      setIndex(STEPS.length - 2);
      return;
    }
    const id = window.setInterval(() => setIndex((current) => (current + 1) % STEPS.length), STEP_MS);
    return () => window.clearInterval(id);
  }, []);
  const step = STEPS[index];
  const players: PlayerWsData[] = NAMES.map((name, i) => ({ name, i })).map(({ name, i: seat }) => ({
    id: `p${seat}`,
    name,
    chips: step.stacks[seat],
    bet: step.bets[seat],
    cards: seat === 0 ? step.hero : seat === 2 && step.showdown ? ["Jh", "Th"] : [],
    folded: step.folded?.includes(seat) ?? false,
    is_active: step.active === seat,
    is_dealer: seat === 3,
    seat: SEATS[seat],
    is_sitting: true,
    draw_count: step.drew?.[seat] ?? null,
    river_discard: seat === 0 ? (step.discards?.[0] ?? null) : seat === 2 ? (step.discards?.[1] ?? null) : null,
  })) as unknown as PlayerWsData[];
  const showdown: ShowdownEntry[] = step.showdown
    ? ([
        { player_id: "p0", player_name: "Ana", hand_name: "Full House", cards: ["Ah", "Ac", "As", "6h", "6s"] },
        { player_id: "p2", player_name: "Caio", hand_name: "Straight", cards: ["Jh", "Th", "As", "Kh", "Qd"] },
      ] as unknown as ShowdownEntry[])
    : [];
  return (
    <div className="hp-live" aria-label="Demonstração: uma mão de Brazilian Pineapple do começo ao fim">
      <ShowcaseFrame designWidth={DESIGN_WIDTH} designHeight={DESIGN_HEIGHT}>
      <PokerTable
        showcase
        players={players}
        communityCards={step.board}
        stage={step.stage}
        pots={(step.pot > 0 ? [{ amount: step.pot, eligible_players: [] }] : []) as unknown as PotWsData[]}
        localPlayerId="p0"
        availableActions={[]}
        onAction={() => {}}
        bettingStructure="pineapple_draw_v4"
        drawing={step.drawing}
        discarding={step.discarding}
        raiseAmount={0}
        onRaiseChange={() => {}}
        callAmount={0}
        minimumWager={0}
        maximumWager={0}
        winners={step.showdown ? ["p0"] : []}
        showdown={showdown}
        maxPlayers={6}
        seatClock={step.active === undefined ? null : { left: CLOCK_LEFT[index % CLOCK_LEFT.length], reserve: 5 }}
      />
      </ShowcaseFrame>
      <div className="hp-live-bar" aria-hidden="true">
        {step.bar && (
          <>
            <span className="hp-live-bar-who">Vez de {step.bar.who}</span>
            {step.bar.options.map(([label, tone]) => (
              <span key={label} className={`hp-live-btn ${tone}${step.bar?.pressed === label ? " pressed" : step.bar?.pressed ? " dim" : ""}`}>
                {label}
              </span>
            ))}
          </>
        )}
      </div>
      <p className="hp-live-caption" aria-live={still ? undefined : "off"}>
        <span className="hp-live-step">
          {String(Math.min(index + 1, STEPS.length - 1)).padStart(2, "0")}/{STEPS.length - 1}
        </span>
        {step.caption}
      </p>
    </div>
  );
}
