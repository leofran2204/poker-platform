import { useEffect, useRef, useState } from "react";
import type { PlayerWsData, PotWsData, ShowdownEntry } from "@/api/types";
import { PokerTable } from "@/components/PokerTable";

// Mesa viva da home: a mão-modelo dos vídeos do Brazilian Pineapple se jogando
// sozinha, com a mesma mesa e as mesmas animações do jogo. Os valores e as cartas
// são os dos vídeos (conferidos no motor em pineapple_draw_film_examples).

type Step = {
  caption: string;
  stage: string;
  board: string[];
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
  showdown?: boolean;
};

const FLOP = ["As", "Qd", "6h"];
const TURN = [...FLOP, "Kh"];
const RIVER = [...TURN, "6s"];
const STEPS: Step[] = [
  { caption: "Três cartas para cada jogador. Blinds e ante na mesa.", stage: "preflop", board: [], pot: 100, bets: [50, 100, 0, 0], stacks: [9950, 9800, 10000, 10000], hero: ["Ah", "Kd", "7c"], active: 2 },
  { caption: "Primeira rodada de apostas: todos pagam.", stage: "preflop", board: [], pot: 100, bets: [100, 100, 100, 100], stacks: [9900, 9800, 9900, 9900], hero: ["Ah", "Kd", "7c"], active: 1 },
  { caption: "Flop: três cartas na mesa.", stage: "flop", board: FLOP, pot: 500, bets: [0, 0, 0, 0], stacks: [9900, 9800, 9900, 9900], hero: ["Ah", "Kd", "7c"], active: 0 },
  { caption: "Primeiro as apostas. Ana aposta, Beto e Caio pagam, Duda desiste.", stage: "flop", board: FLOP, pot: 500, bets: [300, 300, 300, 0], stacks: [9600, 9500, 9600, 9900], hero: ["Ah", "Kd", "7c"], folded: [3] },
  { caption: "Só depois das apostas vem a troca.", stage: "flop", board: FLOP, pot: 1400, bets: [0, 0, 0, 0], stacks: [9600, 9500, 9600, 9900], hero: ["Ah", "Kd", "7c"], folded: [3], drawing: true, active: 0 },
  { caption: "Ana troca o sete e recebe outro ás: trinca.", stage: "flop", board: FLOP, pot: 1400, bets: [0, 0, 0, 0], stacks: [9600, 9500, 9600, 9900], hero: ["Ah", "Kd", "Ac"], folded: [3], drawing: true, drew: [1, 1, 1, null] },
  { caption: "Turn: nova rodada de apostas. Beto desiste, Caio paga.", stage: "turn", board: TURN, pot: 1400, bets: [700, 0, 700, 0], stacks: [8900, 9500, 8900, 9900], hero: ["Ah", "Kd", "Ac"], folded: [1, 3] },
  { caption: "River: cada jogador descarta uma carta.", stage: "river", board: RIVER, pot: 2800, bets: [0, 0, 0, 0], stacks: [8900, 9500, 8900, 9900], hero: ["Ah", "Kd", "Ac"], folded: [1, 3], discarding: true, active: 0 },
  { caption: "Ana descarta o rei e fica com duas cartas.", stage: "river", board: RIVER, pot: 2800, bets: [0, 0, 0, 0], stacks: [8900, 9500, 8900, 9900], hero: ["Ah", "Ac"], folded: [1, 3], drew: [1, null, 1, null] },
  { caption: "Última rodada de apostas, com duas cartas na mão.", stage: "river", board: RIVER, pot: 2800, bets: [1500, 0, 1500, 0], stacks: [7400, 9500, 7400, 9900], hero: ["Ah", "Ac"], folded: [1, 3] },
  { caption: "Showdown: full house de ases com seis. O pote vai para Ana.", stage: "showdown", board: RIVER, pot: 5800, bets: [0, 0, 0, 0], stacks: [7400, 9500, 7400, 9900], hero: ["Ah", "Ac"], folded: [1, 3], showdown: true },
  { caption: "Showdown: full house de ases com seis. O pote vai para Ana.", stage: "showdown", board: RIVER, pot: 0, bets: [0, 0, 0, 0], stacks: [13200, 9500, 7400, 9900], hero: ["Ah", "Ac"], folded: [1, 3], showdown: true },
];
const NAMES = ["Ana", "Beto", "Caio", "Duda"];
/** Lugares na mesa de seis: embaixo, alto à esquerda, topo e alto à direita. */
const SEATS = [0, 2, 3, 4];
const STEP_MS = 2600;
/** A mesa é desenhada nesta largura e reduzida por escala para caber no espaço. */
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
  const frame = useRef<HTMLDivElement>(null);
  // Em telas estreitas a mesa usa o formato de celular do próprio jogo, sem escala.
  const [scale, setScale] = useState<number | null>(0.6);
  useEffect(() => {
    const node = frame.current;
    if (!node || typeof ResizeObserver === "undefined") return;
    const fit = () => setScale(node.clientWidth < 480 ? null : Math.min(1, node.clientWidth / DESIGN_WIDTH));
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(node);
    return () => observer.disconnect();
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
  })) as unknown as PlayerWsData[];
  const showdown: ShowdownEntry[] = step.showdown
    ? ([
        { player_id: "p0", player_name: "Ana", hand_name: "Full House", cards: ["Ah", "Ac", "As", "6h", "6s"] },
        { player_id: "p2", player_name: "Caio", hand_name: "Straight", cards: ["Jh", "Th", "As", "Kh", "Qd"] },
      ] as unknown as ShowdownEntry[])
    : [];
  return (
    <div className="hp-live" aria-label="Demonstração: uma mão de Brazilian Pineapple do começo ao fim">
      <div ref={frame} className="hp-live-frame" style={scale === null ? undefined : { height: DESIGN_HEIGHT * scale }}>
        <div
          className={scale === null ? "hp-live-stage native" : "hp-live-stage"}
          style={scale === null ? undefined : { width: DESIGN_WIDTH, transform: `scale(${scale})` }}
        >
      <PokerTable
        showcase
        players={players}
        communityCards={step.board}
        stage={step.stage}
        pots={(step.pot > 0 ? [{ amount: step.pot, eligible_players: [] }] : []) as unknown as PotWsData[]}
        localPlayerId="p0"
        availableActions={[]}
        onAction={() => {}}
        bettingStructure="pineapple_draw_v1"
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
      />
        </div>
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
