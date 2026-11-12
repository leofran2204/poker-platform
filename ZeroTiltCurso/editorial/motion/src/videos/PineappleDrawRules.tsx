import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { Card } from "../components/Card";
import { ChipStack, Pot } from "../components/Chips";
import { Sfx, SpadeMark } from "../components/Frame";
import { Seat, Table } from "../components/Table";
import { Body, Headline, Kicker, Panel, Pill, Stamp } from "../components/Text";
import { brl, CLAMP, glide, pop, ramp, rise } from "../lib/anim";
import { flight, Point, toward } from "../lib/deal";
import { C, DISPLAY, UI } from "../theme";
import { SceneMap, SceneProps } from "../Video";
import { HandScene } from "./HandTable";

// Filme de regras do Pineapple Draw: cenas guiadas pelos dados de
// src/data/pineapple-draw-rules.json. Os exemplos de cartas, a ordem das trocas e
// os pagamentos são conferidos pelo motor (Motor-Rust/tests/pineapple_draw_film_examples.rs).

type Player = { name: string; stack: number; bet: number };
type Action = { player: number; to?: number; ante?: number; cue: string; label: string; next?: number | null };
type Visual = {
  mode?: string;
  kicker?: string;
  badge?: string;
  lines?: string[];
  note?: string;
  example?: string;
  cue?: string;
  /** Frase em que a privada extra é entregue (cenas de distribuição). */
  extraCue?: string;
  cues?: string[];
  hole?: string[];
  board?: string[];
  selectHole?: number[];
  selectBoard?: number[];
  invalid?: boolean;
  before?: number;
  after?: number;
  allin?: number[];
  folded?: number[];
  table?: { pot: number; actor?: number; players: Player[] };
  actions?: Action[];
  dealer?: number;
  metrics?: [string, string][];
  rows?: string[][];
};

const visualOf = (props: SceneProps) => (props.data.visual ?? {}) as Visual;

/** Deixa robusta: se a frase não estiver na narração, usa uma posição proporcional. */
const safeCue = (props: SceneProps, phrase: string | undefined, fallback: number) => {
  if (!phrase) return fallback;
  try {
    return props.cue(phrase);
  } catch {
    return fallback;
  }
};
const spaced = (props: SceneProps, index: number, count: number) =>
  Math.round(props.audioAt + (props.speechEnd - props.audioAt) * (0.1 + (0.75 * index) / Math.max(1, count - 1)));

const Abs: React.FC<{ x: number; y: number; children: React.ReactNode; style?: React.CSSProperties }> = ({ x, y, children, style }) => (
  <div style={{ position: "absolute", left: x, top: y, transform: "translate(-50%, -50%)" }}>
    <div style={style}>{children}</div>
  </div>
);

/** Cabeçalho comum: capítulo, título da cena e etiqueta do exemplo. */
const Header: React.FC<{ props: SceneProps }> = ({ props }) => {
  const visual = visualOf(props);
  return (
    <>
      <div style={{ position: "absolute", left: 140, top: 110, width: 1300 }}>
        <Kicker at={0}>{props.data.chapter}</Kicker>
        {props.data.title && <Headline text={props.data.title} at={3} size={60} style={{ marginTop: 10 }} />}
      </div>
      {visual.example && (
        <div style={{ position: "absolute", right: 120, top: 118 }}>
          <Pill tone="muted" at={8} size={20}>
            {visual.example}
          </Pill>
        </div>
      )}
    </>
  );
};

const Note: React.FC<{ text?: string; at: number; tone?: "gold" | "muted" | "danger" }> = ({ text, at, tone = "gold" }) =>
  text ? (
    <Abs x={960} y={995}>
      <Pill tone={tone} at={at} size={24}>
        {text}
      </Pill>
    </Abs>
  ) : null;

/* Abertura. */
const Intro: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const visual = visualOf(props);
  const title = props.data.title ?? "Brazilian Pineapple";
  const [first, ...rest] = title.split(" ");
  // Títulos de aula são longos: menor e em até duas linhas, sem encostar nas bordas.
  const size = title.length <= 16 ? 150 : title.length <= 26 ? 124 : 104;
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 18 }}>
      <SpadeMark size={120} draw={ramp(frame, 0, 40)} />
      <Kicker at={6}>{visual.kicker ?? "Zero Tilt Academy · regras completas"}</Kicker>
      <div
        style={{
          fontFamily: DISPLAY,
          fontWeight: 700,
          fontSize: size,
          color: C.ivory,
          lineHeight: 1.05,
          maxWidth: 1560,
          textAlign: "center",
          textWrap: "balance",
          ...rise(frame, 10, 50, 26),
        }}
      >
        {first} <span style={{ color: C.gold, fontStyle: "italic" }}>{rest.join(" ")}</span>
      </div>
      {(visual.lines ?? []).map((line, i) => (
        <Body key={line} at={30 + i * 14} size={36} color={i === 0 ? C.goldSoft : C.muted}>
          {line}
        </Body>
      ))}
      <div style={{ marginTop: 20 }}>
        <Pill tone="muted" at={50} size={22}>
          {visual.badge ?? "Valores ilustrativos · comece com Play Money"}
        </Pill>
      </div>
    </AbsoluteFill>
  );
};

/* Seleção de cartas: exatamente 2 privadas + 3 comunitárias. */
const Cards: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const visual = visualOf(props);
  const hole = visual.hole ?? [];
  const board = visual.board ?? [];
  const select = safeCue(props, visual.cue, spaced(props, 1, 3));
  const t = glide(frame, select, fps, 20);
  const row = (codes: string[], y: number, picked: number[], delay: number, width: number) =>
    codes.map((code, i) => {
      const isPicked = picked.includes(i);
      return (
        <Abs key={code + i} x={960 + (i - (codes.length - 1) / 2) * (width + 26)} y={y - (isPicked ? 20 * t : 0)} style={rise(frame, delay + i * 3, 40)}>
          <Card code={code} width={width} glow={isPicked && !visual.invalid ? t : 0} dim={isPicked ? 0 : t * 0.9} />
          {isPicked && visual.invalid && (
            <div style={{ position: "absolute", inset: -6, borderRadius: 14, border: `4px solid ${C.danger}`, opacity: t }} />
          )}
        </Abs>
      );
    });
  return (
    <AbsoluteFill>
      <Header props={props} />
      <Abs x={250} y={420}>
        <div style={{ fontFamily: UI, fontWeight: 800, letterSpacing: 4, fontSize: 22, color: C.faint }}>COMUNITÁRIAS</div>
      </Abs>
      {row(board, 420, visual.selectBoard ?? [], 6, 140)}
      <Abs x={250} y={720}>
        <div style={{ fontFamily: UI, fontWeight: 800, letterSpacing: 4, fontSize: 22, color: C.faint }}>PRIVADAS</div>
      </Abs>
      {row(hole, 720, visual.selectHole ?? [], 12, 140)}
      {visual.invalid && (
        <Abs x={1640} y={570}>
          <Stamp text="INVÁLIDA" at={select + 10} size={70} color={C.danger} />
        </Abs>
      )}
      {!visual.invalid && (
        <Abs x={1680} y={570}>
          <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 96, color: C.gold, transform: `scale(${pop(frame, select + 6, fps, 12)})` }}>
            2 + 3
          </div>
        </Abs>
      )}
      <Note text={visual.note} at={select + 20} tone={visual.invalid ? "danger" : "gold"} />
      <Sfx at={select} name={visual.invalid ? "stamp" : "pop"} volume={0.3} />
    </AbsoluteFill>
  );
};

const SEATS3: Point[] = [
  { x: 960, y: 900 },
  { x: 380, y: 420 },
  { x: 1540, y: 420 },
];
const SEATS4: Point[] = [
  { x: 960, y: 900 },
  { x: 360, y: 560 },
  { x: 960, y: 250 },
  { x: 1560, y: 560 },
];
const TABLE_CENTER: Point = { x: 960, y: 575 };

// Geometria da mesa nas cenas de distribuição e troca.
const DECK: Point = { x: 600, y: 700 };
const MUCK: Point = { x: 1320, y: 700 };
const BOARD_Y = 520;
const HERO_Y = 765;
const boardX = (k: number) => 960 + (k - 2) * 122;
const heroSlot = (c: number): Point => ({ x: 960 + (c - 1) * 100, y: HERO_Y });
const oppSlot = (seat: Point, c: number): Point => {
  const base = toward(seat, TABLE_CENTER, 0.25);
  return { x: base.x + (c - 1) * 30, y: base.y };
};
const slotOf = (player: number, c: number) => (player === 0 ? heroSlot(c) : oppSlot(SEATS3[player], c));

/** Pilha de cartas fechadas com rótulo (baralho ou descarte). */
const Pile: React.FC<{ at: Point; label: string; count: number; opacity?: number }> = ({ at, label, count, opacity = 1 }) => (
  <Abs x={at.x} y={at.y} style={{ opacity, textAlign: "center" }}>
    <div style={{ position: "relative", width: 64, height: 91 }}>
      {Array.from({ length: Math.max(1, Math.min(4, count)) }).map((_, i) => (
        <div key={i} style={{ position: "absolute", left: i * 2, top: -i * 2 }}>
          <Card width={64} face={0} />
        </div>
      ))}
    </div>
    <div style={{ marginTop: 10, fontFamily: UI, fontWeight: 800, letterSpacing: 3, fontSize: 16, color: C.faint, whiteSpace: "nowrap" }}>
      {label}
    </div>
  </Abs>
);

type DrawEvent = { player: number; swap: boolean; position?: number; receive?: string; cue: string; delay?: number };
type Arrival = { card: string; cue: string };
type DrawVisual = Visual & {
  players?: Player[];
  heroCards?: string[];
  arrivals?: Arrival[];
  prior?: { player: number; swap: boolean }[];
  draws?: DrawEvent[];
  runout?: boolean;
  bannerCue?: string;
  spareCue?: string;
  resultCue?: string;
  resultLabel?: string;
  dealCue?: string;
  flopCue?: string;
};

const DEFAULT_PLAYERS: Player[] = [
  { name: "Ana · você", stack: 10000, bet: 0 },
  { name: "Beto", stack: 10000, bet: 0 },
  { name: "Caio", stack: 10000, bet: 0 },
];

/* Pré-flop: três privadas para cada um, sem troca; depois o flop fechado vira. */
const Deal3: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const visual = visualOf(props) as DrawVisual;
  const players = visual.players ?? DEFAULT_PLAYERS;
  const dealer = visual.dealer ?? 0;
  const hero = visual.heroCards ?? [];
  const board = visual.board ?? [];
  const deal = safeCue(props, visual.dealCue, spaced(props, 0, 4));
  const flop = Math.max(deal + Math.round(3.2 * fps), safeCue(props, visual.flopCue, spaced(props, 2, 4)));
  const flip = flop + Math.round(1.4 * fps);
  // Distribuição a partir da esquerda do botão, uma carta por vez, três voltas.
  const order = players.map((_, i) => (dealer + 1 + i) % players.length);
  const dealAt = (player: number, c: number) => deal + (c * players.length + order.indexOf(player)) * 5;
  return (
    <AbsoluteFill>
      <Header props={props} />
      <Table x={TABLE_CENTER.x} y={TABLE_CENTER.y} width={1400} height={620} />
      {players.map((player, i) => (
        <Seat key={player.name} x={SEATS3[i].x} y={SEATS3[i].y} name={player.name} hero={i === 0} dealer={i === dealer} />
      ))}
      <Pile at={DECK} label="BARALHO" count={4} />
      {players.map((_, i) =>
        [0, 1, 2].map((c) => {
          const at = dealAt(i, c);
          const f = flight(frame, at, fps, DECK, slotOf(i, c));
          const isHero = i === 0;
          return (
            <Abs key={`${i}-${c}`} x={f.x} y={f.y} style={{ opacity: f.opacity }}>
              <Card
                code={isHero ? hero[c] : undefined}
                width={isHero ? 86 : 42}
                face={isHero ? ramp(frame, at + 20, 12) : 0}
                style={{ transform: `rotate(${f.rotate}deg)` }}
              />
            </Abs>
          );
        }),
      )}
      {board.slice(0, 3).map((code, k) => {
        const f = flight(frame, flop + k * 6, fps, DECK, { x: boardX(k), y: BOARD_Y }, 0);
        return (
          <Abs key={code} x={f.x} y={f.y} style={{ opacity: f.opacity }}>
            <Card code={code} width={108} face={ramp(frame, flip + k * 6, Math.round(0.8 * fps))} />
          </Abs>
        );
      })}
      <Abs x={1720} y={300} style={{ textAlign: "right" }}>
        <div style={{ fontFamily: UI, fontWeight: 800, letterSpacing: 4, fontSize: 18, color: C.mint }}>SUAS PRIVADAS</div>
        <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 100, lineHeight: 1, color: C.gold }}>
          {Math.min(3, players.length ? Math.max(0, Math.floor((frame - deal) / (players.length * 5)) + 1) : 0)}
        </div>
      </Abs>
      <Note text={visual.note} at={flip + 20} />
      {players.map((_, i) => [0, 1, 2].map((c) => <Sfx key={`s${i}${c}`} at={dealAt(i, c)} name="deal" volume={0.18} />))}
      {[0, 1, 2].map((k) => (
        <React.Fragment key={k}>
          <Sfx at={flop + k * 6} name="deal" volume={0.3} />
          <Sfx at={flip + k * 6} name="flip" volume={0.32} />
        </React.Fragment>
      ))}
    </AbsoluteFill>
  );
};

/* Rodada de troca: na ordem das apostas, cada um mantém ou troca uma carta. */
const DrawRound: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const visual = visualOf(props) as DrawVisual;
  const players = visual.players ?? DEFAULT_PLAYERS;
  const dealer = visual.dealer ?? 0;
  const allin = visual.allin ?? [];
  const folded = visual.folded ?? [];
  const board = visual.board ?? [];
  const arrivals = (visual.arrivals ?? []).map((arrival, i) => ({
    ...arrival,
    index: board.length + i,
    at: safeCue(props, arrival.cue, spaced(props, i, 4)),
  }));
  const draws = (visual.draws ?? []).map((draw, i) => ({
    ...draw,
    at: safeCue(props, draw.cue, spaced(props, i + 1, (visual.draws ?? []).length + 2)) + (draw.delay ?? 0),
  }));
  const lastArrival = arrivals.length ? arrivals[arrivals.length - 1].at + Math.round(2.2 * fps) : 0;
  const bannerAt = visual.runout ? (arrivals[0]?.at ?? 20) : Math.max(lastArrival, safeCue(props, visual.bannerCue, (draws[0]?.at ?? 30) - 20));
  // Tempos de cada troca: seleção, saída para o descarte e chegada da nova carta.
  const OUT = 8;
  const IN = 30;
  const FLIP = 52;
  const heroSwap = draws.find((draw) => draw.player === 0 && draw.swap);
  const spareAt = safeCue(props, visual.spareCue, heroSwap ? heroSwap.at - 30 : 0);
  const resultAt = safeCue(props, visual.resultCue, heroSwap ? heroSwap.at + FLIP + 20 : 0);
  const discarded = draws.filter((draw) => draw.swap && frame >= draw.at + OUT + 20).length;
  const pending = draws.find((draw) => frame < draw.at);
  const pendingIndex = pending ? draws.indexOf(pending) : -1;
  const turnStart = pendingIndex <= 0 ? bannerAt : draws[pendingIndex - 1].at + 10;
  const heroCards = visual.heroCards ?? [];

  const badgeFor = (i: number) => {
    if (allin.includes(i)) return { text: "ALL-IN · NÃO TROCA", color: C.danger, opacity: 1 };
    if (folded.includes(i)) return { text: "FOLD", color: C.muted, opacity: 1 };
    const event = [...draws].reverse().find((draw) => draw.player === i && frame >= draw.at);
    if (event) return { text: event.swap ? "TROCOU 1" : "MANTEVE", color: event.swap ? C.gold : C.goldSoft, opacity: ramp(frame, event.at, 8) };
    const prior = (visual.prior ?? []).find((p) => p.player === i);
    if (prior) return { text: prior.swap ? "TROCOU 1" : "MANTEVE", color: prior.swap ? C.gold : C.goldSoft, opacity: 1 };
    return undefined;
  };

  return (
    <AbsoluteFill>
      <Header props={props} />
      <Table x={TABLE_CENTER.x} y={TABLE_CENTER.y} width={1400} height={620} />
      {players.map((player, i) => {
        const badge = badgeFor(i);
        const acting = pending && pending.player === i && frame >= turnStart ? 1 : 0;
        return (
          <Seat
            key={player.name}
            x={SEATS3[i].x}
            y={SEATS3[i].y}
            name={player.name}
            hero={i === 0}
            dealer={i === dealer}
            folded={folded.includes(i) ? 1 : 0}
            active={acting}
            badge={badge?.text}
            badgeColor={badge?.color}
            badgeOpacity={badge?.opacity ?? 0}
          />
        );
      })}
      <Pile at={DECK} label="BARALHO" count={4} />
      <Pile at={MUCK} label={discarded ? `DESCARTE · ${discarded}` : "DESCARTE"} count={Math.max(1, discarded)} opacity={discarded ? 1 : 0.35} />
      {board.map((code, k) => (
        <Abs key={code} x={boardX(k)} y={BOARD_Y}>
          <Card code={code} width={108} />
        </Abs>
      ))}
      {arrivals.map((arrival) => {
        const f = flight(frame, arrival.at, fps, DECK, { x: boardX(arrival.index), y: BOARD_Y }, 0);
        return (
          <Abs key={arrival.card} x={f.x} y={f.y} style={{ opacity: f.opacity }}>
            <Card code={arrival.card} width={108} face={ramp(frame, arrival.at + Math.round(1.3 * fps), Math.round(0.8 * fps))} />
          </Abs>
        );
      })}
      {players.map((_, i) => {
        if (folded.includes(i)) return null;
        const isHero = i === 0;
        const swap = draws.find((draw) => draw.player === i && draw.swap);
        return [0, 1, 2].map((c) => {
          const slot = slotOf(i, c);
          const swapped = swap && (swap.position ?? 2) === c;
          const width = isHero ? 86 : 42;
          const glowAll = allin.includes(i) ? 0.5 : 0;
          if (!swapped) {
            return (
              <Abs key={`${i}-${c}`} x={slot.x} y={slot.y}>
                <Card
                  code={isHero ? heroCards[c] : undefined}
                  width={width}
                  face={isHero ? 1 : 0}
                  glow={isHero && heroSwap && frame >= spareAt && frame < heroSwap.at + OUT ? ramp(frame, spareAt, 12) * 0.8 : glowAll}
                />
              </Abs>
            );
          }
          // Carta que sai: levanta, vira para baixo e voa para o descarte.
          const lift = ramp(frame, swap.at, OUT);
          const out = flight(frame, swap.at + OUT, fps, { x: slot.x, y: slot.y - 26 * lift }, MUCK, 12);
          const leaving = frame < swap.at + OUT + 26;
          // Carta que entra: do baralho para a mesma posição; o herói a vê virar.
          const incoming = flight(frame, swap.at + IN, fps, DECK, slot);
          return (
            <React.Fragment key={`${i}-${c}`}>
              {leaving && (
                <Abs
                  x={frame < swap.at + OUT ? slot.x : out.x}
                  y={frame < swap.at + OUT ? slot.y - 26 * lift : out.y}
                  style={{ transform: `rotate(${frame < swap.at + OUT ? 0 : out.rotate}deg)` }}
                >
                  <Card
                    code={isHero ? heroCards[c] : undefined}
                    width={width}
                    face={isHero ? 1 - ramp(frame, swap.at + OUT, 10) : 0}
                    dim={isHero && frame >= spareAt ? 0.45 * ramp(frame, spareAt, 12) : 0}
                  />
                </Abs>
              )}
              <Abs x={incoming.x} y={incoming.y} style={{ opacity: incoming.opacity, transform: `rotate(${incoming.rotate}deg)` }}>
                <Card
                  code={isHero ? swap.receive : undefined}
                  width={width}
                  face={isHero ? ramp(frame, swap.at + FLIP, 12) : 0}
                  glow={isHero && frame >= resultAt ? ramp(frame, resultAt, 12) * 0.9 : 0}
                />
              </Abs>
            </React.Fragment>
          );
        });
      })}
      <Abs x={960} y={330}>
        <Pill tone={visual.runout ? "danger" : "mint"} at={bannerAt} size={22}>
          {visual.runout ? "SEM APOSTAS POSSÍVEIS · SEM RODADA DE TROCA" : "RODADA DE TROCA · ANTES DAS APOSTAS"}
        </Pill>
      </Abs>
      {visual.resultLabel && heroSwap && (
        <Abs x={960} y={HERO_Y - 115}>
          <Pill tone="gold" at={resultAt} size={24}>
            {visual.resultLabel}
          </Pill>
        </Abs>
      )}
      <Note text={visual.note} at={(draws[draws.length - 1]?.at ?? lastArrival) + 24} />
      {arrivals.map((arrival) => (
        <React.Fragment key={arrival.card}>
          <Sfx at={arrival.at} name="deal" volume={0.3} />
          <Sfx at={arrival.at + Math.round(1.3 * fps)} name="flip" volume={0.32} />
        </React.Fragment>
      ))}
      {draws.map((draw, i) =>
        draw.swap ? (
          <React.Fragment key={i}>
            <Sfx at={draw.at + OUT} name="whoosh" volume={0.22} />
            <Sfx at={draw.at + IN} name="deal" volume={0.26} />
          </React.Fragment>
        ) : (
          <Sfx key={i} at={draw.at} name="pop" volume={0.2} />
        ),
      )}
    </AbsoluteFill>
  );
};

/* Ranking clássico, do menor para o maior. */
const RANKS = ["Carta alta", "Par", "Dois pares", "Trinca", "Sequência", "Flush", "Full house", "Quadra", "Straight flush"];
const RANK_CUES = ["carta alta", "par", "dois pares", "trinca", "sequência", "flush", "full house", "quadra", "straight flush"];
const Ranking: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const supera = safeCue(props, "supera", spaced(props, 2, 3));
  return (
    <AbsoluteFill>
      <Header props={props} />
      <div style={{ position: "absolute", left: 160, top: 250, display: "flex", flexDirection: "column-reverse", gap: 8 }}>
        {RANKS.map((rank, i) => {
          const at = safeCue(props, RANK_CUES[i], spaced(props, i, 12));
          const emphasise = (rank === "Sequência" || rank === "Full house") && frame >= supera;
          return (
            <div
              key={rank}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 18,
                width: 640 + i * 40,
                padding: "8px 22px",
                borderRadius: 14,
                background: emphasise ? "rgba(212,168,67,0.18)" : C.panel,
                border: `1px solid ${emphasise ? C.gold : C.line}`,
                ...rise(frame, at, 24, 16),
              }}
            >
              <span style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 28, color: C.gold, width: 40 }}>{i + 1}</span>
              <span style={{ fontFamily: UI, fontWeight: 700, fontSize: 28, color: C.ivory }}>{rank}</span>
            </div>
          );
        })}
      </div>
      <div style={{ position: "absolute", left: 1220, top: 380, width: 560, display: "flex", flexDirection: "column", gap: 20 }}>
        <Panel at={supera} accent={C.gold}>
          <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 30, color: C.ivory }}>Sequência supera trinca</div>
          <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 30, color: C.ivory, marginTop: 8 }}>Full house supera flush</div>
        </Panel>
        <Panel at={safeCue(props, "royal", spaced(props, 1, 3))} accent={C.mint}>
          <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 26, color: C.muted }}>Royal flush: A K Q J 10 do mesmo naipe, o maior straight flush.</div>
        </Panel>
      </div>
    </AbsoluteFill>
  );
};

/* Apostas na mesa: blinds, ante, calls e aumentos com fichas e pote animados. */
const Bets: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const visual = visualOf(props);
  const table = visual.table ?? { pot: 0, players: [] };
  const seats = table.players.length > 3 ? SEATS4 : SEATS3;
  const actions = (visual.actions ?? []).map((action, i) => ({
    ...action,
    at: safeCue(props, action.cue, spaced(props, i, (visual.actions ?? []).length + 1)),
  }));
  const done = actions.filter((action) => frame >= action.at + 12);
  const bets = table.players.map((player, i) => {
    const last = [...done].reverse().find((action) => action.player === i && action.to !== undefined);
    return last?.to ?? player.bet;
  });
  const potTarget =
    table.pot +
    actions.reduce((sum, action) => {
      const t = glide(frame, action.at + 12, fps, 14);
      const previous = (() => {
        const earlier = actions.filter((a) => a.player === action.player && a.at < action.at && a.to !== undefined);
        return earlier.length ? earlier[earlier.length - 1].to! : table.players[action.player].bet;
      })();
      const added = action.ante ?? (action.to !== undefined ? action.to - previous : 0);
      return sum + added * t;
    }, 0);
  const stack = (i: number) => {
    const paid = done
      .filter((action) => action.player === i)
      .reduce((sum, action) => {
        const earlier = actions.filter((a) => a.player === i && a.at < action.at && a.to !== undefined);
        const previous = earlier.length ? earlier[earlier.length - 1].to! : table.players[i].bet;
        return sum + (action.ante ?? (action.to !== undefined ? action.to - previous : 0));
      }, 0);
    return table.players[i].stack - paid;
  };
  const lastByPlayer = (i: number) => [...actions].reverse().find((action) => action.player === i && frame >= action.at);
  const current = [...actions].reverse().find((action) => frame >= action.at + 12);
  const actor = current ? (current.next ?? -1) : (table.actor ?? -1);
  const dealer = visual.dealer ?? 0;
  return (
    <AbsoluteFill>
      <Header props={props} />
      <Table x={TABLE_CENTER.x} y={TABLE_CENTER.y} width={1400} height={620} />
      {table.players.map((player, i) => {
        const label = lastByPlayer(i);
        return (
          <Seat
            key={player.name}
            x={seats[i].x}
            y={seats[i].y}
            name={player.name}
            stack={brl(stack(i) / 100)}
            hero={player.name.includes("você")}
            dealer={i === dealer}
            active={i === actor ? 1 : 0}
            badge={label?.label}
            badgeColor={label?.ante ? C.goldSoft : label?.label.toLowerCase().includes("all-in") ? C.danger : C.gold}
            badgeOpacity={label ? ramp(frame, label.at, 8) : 0}
          />
        );
      })}
      {table.players.map((_, i) => {
        if (!bets[i]) return null;
        const besideCards = visual.hole && i === 0;
        const at = besideCards ? { x: 960 + visual.hole!.length * 46 + 80, y: 760 } : toward(seats[i], TABLE_CENTER, 0.42);
        const recent = [...actions].reverse().find((action) => action.player === i && action.to !== undefined && frame >= action.at);
        const f = recent ? flight(frame, recent.at, fps, seats[i], at, 0) : { x: at.x, y: at.y, opacity: 1, rotate: 0 };
        return (
          <Abs key={`bet-${i}`} x={f.x} y={f.y} style={{ opacity: f.opacity }}>
            <ChipStack count={Math.min(6, 1 + Math.floor(bets[i] / 150))} color={i % 2 ? "blue" : "red"} size={44} label={brl(bets[i] / 100)} />
          </Abs>
        );
      })}
      {actions
        .filter((action) => action.ante)
        .map((action) => {
          const f = flight(frame, action.at, fps, seats[action.player], visual.board ? { x: 1200, y: 380 } : { x: 960, y: 440 }, 0);
          return (
            <Abs key={`ante-${action.at}`} x={f.x} y={f.y} style={{ opacity: f.opacity }}>
              <ChipStack count={3} color="gold" size={44} label={`ANTE ${brl((action.ante ?? 0) / 100)}`} />
            </Abs>
          );
        })}
      {visual.board && (
        <div style={{ position: "absolute", left: 960 - 190, top: 470, display: "flex", gap: 14 }}>
          {visual.board.map((code) => (
            <Card key={code} code={code} width={100} />
          ))}
        </div>
      )}
      {visual.hole && (
        <div style={{ position: "absolute", left: 960 - visual.hole.length * 46, top: 700, display: "flex", gap: 10 }}>
          {visual.hole.map((code) => (
            <Card key={code} code={code} width={82} />
          ))}
        </div>
      )}
      <Abs x={960} y={visual.board ? 420 : 560}>
        <Pot value={brl(Math.round(potTarget) / 100)} />
      </Abs>
      <Note text={visual.note} at={actions.length ? actions[actions.length - 1].at + 24 : 30} />
      {actions.map((action) => (
        <Sfx key={action.at + action.label} at={action.at + 10} name="chips" volume={0.32} />
      ))}
    </AbsoluteFill>
  );
};

/* Números em destaque (exemplos de valores, totais e potes). */
const Metrics: React.FC<SceneProps> = (props) => {
  const visual = visualOf(props);
  const metrics = visual.metrics ?? [];
  const isPots = visual.mode === "sidepots";
  return (
    <AbsoluteFill>
      <Header props={props} />
      <div style={{ position: "absolute", left: 140, top: 380, width: 1640, display: "grid", gridTemplateColumns: `repeat(${metrics.length}, 1fr)`, gap: 30 }}>
        {metrics.map(([label, value], i) => {
          const at = safeCue(props, visual.cues?.[i], spaced(props, i, metrics.length));
          const last = i === metrics.length - 1;
          return (
            <Panel key={label} at={at} accent={last ? C.gold : C.mint} style={{ padding: "34px 36px", minHeight: 260 }}>
              <div style={{ fontFamily: UI, fontWeight: 800, letterSpacing: 3, fontSize: 22, color: C.muted }}>{label}</div>
              <div
                style={{
                  fontFamily: DISPLAY,
                  fontWeight: 700,
                  fontSize: value.length <= 6 ? 92 : value.length <= 9 ? 78 : 62,
                  color: last ? C.gold : C.ivory,
                  marginTop: 18,
                  lineHeight: 1.05,
                  whiteSpace: "nowrap",
                }}
              >
                {value}
              </div>
              <Sfx at={at} name={isPots ? "chips" : "pop"} volume={0.28} />
            </Panel>
          );
        })}
      </div>
      <Note text={visual.note} at={spaced(props, 2, 3) + 10} tone="muted" />
    </AbsoluteFill>
  );
};

/* Fórmula do teto: pote antes do call. */
const Formula: React.FC<SceneProps> = (props) => {
  const visual = visualOf(props);
  const lines = visual.lines ?? [];
  const parts = [
    { key: "B", text: "o que você já pôs na rodada" },
    { key: "C", text: "o que falta pagar (call)" },
    { key: "P", text: "o pote formado antes do call" },
  ];
  return (
    <AbsoluteFill>
      <Header props={props} />
      <div style={{ position: "absolute", left: 160, top: 320, display: "flex", flexDirection: "column", gap: 18 }}>
        {lines.map((line, i) => (
          <Body key={line} at={safeCue(props, visual.cues?.[i], spaced(props, i, 4))} size={44} color={C.ivory}>
            {line}
          </Body>
        ))}
      </div>
      <Abs x={960} y={650}>
        <Panel at={spaced(props, 2, 4)} accent={C.gold} style={{ padding: "30px 50px" }}>
          <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 72, color: C.gold, whiteSpace: "nowrap" }}>
            máximo = min(B + saldo, B + C + P)
          </div>
        </Panel>
      </Abs>
      <div style={{ position: "absolute", left: 160, top: 800, display: "flex", gap: 30 }}>
        {parts.map((part, i) => (
          <Pill key={part.key} tone="muted" at={spaced(props, 2, 4) + 10 + i * 8} size={24}>
            <span style={{ color: C.gold }}>{part.key}</span> = {part.text}
          </Pill>
        ))}
      </div>
      <Note text="Não há teto fixo de quatro blinds" at={safeCue(props, "teto", spaced(props, 3, 4))} tone="muted" />
    </AbsoluteFill>
  );
};

/* Listas de lembretes (passos, estados, formatos e recapitulação). */
const Lines: React.FC<SceneProps> = (props) => {
  const visual = visualOf(props);
  const lines = visual.lines ?? [];
  const rows = visual.rows ?? [];
  return (
    <AbsoluteFill>
      <Header props={props} />
      <div style={{ position: "absolute", left: 160, top: 330, width: 1600, display: "flex", flexDirection: "column", gap: 22 }}>
        {lines.map((line, i) => (
          <Panel key={line} at={safeCue(props, visual.cues?.[i], spaced(props, i, lines.length))} accent={i === lines.length - 1 ? C.gold : C.mint} style={{ padding: "26px 34px" }}>
            <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 42, color: C.ivory }}>{line}</div>
          </Panel>
        ))}
        {rows.map((row, i) => (
          <Panel key={row.join("-")} at={safeCue(props, visual.cues?.[i], spaced(props, i, rows.length))} accent={row[2]?.includes("não") ? C.danger : C.ok} style={{ padding: "26px 34px" }}>
            <div style={{ display: "grid", gridTemplateColumns: "300px 1fr 1fr", alignItems: "center", fontFamily: UI, fontWeight: 700, fontSize: 40, color: C.ivory }}>
              <span>{row[0]}</span>
              <span style={{ fontFamily: DISPLAY, color: C.goldSoft }}>{row[1]}</span>
              <span style={{ color: row[2]?.includes("não") ? C.danger : C.ok }}>{row[2]}</span>
            </div>
          </Panel>
        ))}
      </div>
      <Note text={visual.note} at={spaced(props, 2, 3) + 12} tone="muted" />
      {[...lines, ...rows].map((_, i) => (
        <Sfx key={i} at={safeCue(props, visual.cues?.[i], spaced(props, i, Math.max(lines.length, rows.length)))} name="pop" volume={0.22} />
      ))}
    </AbsoluteFill>
  );
};

/* Encerramento. */
const Outro: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const visual = visualOf(props);
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 26 }}>
      <SpadeMark size={220} draw={ramp(frame, 0, 50)} withText />
      <Headline text="Estude. Jogue. Sem tilt." at={14} size={78} accent={["tilt."]} style={{ justifyContent: "center" }} />
      <div style={{ display: "flex", gap: 20 }}>
        {(visual.lines ?? []).map((line, i) => (
          <Pill key={line} tone={i === 1 ? "gold" : "muted"} at={spaced(props, i, 3)} size={24}>
            {line}
          </Pill>
        ))}
      </div>
    </AbsoluteFill>
  );
};

/* Perguntas de aula: alternativas, pausa para responder e resposta comentada.
   Coluna em fluxo (pergunta → alternativas → explicação): se adapta a perguntas de uma ou
   duas linhas sem invadir o cabeçalho nem o rodapé. Com cartas, elas ganham coluna própria. */
type QuizOption = { key: string; text: string; cue?: string };
type QuizVisual = Visual & {
  question?: string;
  options?: QuizOption[];
  correct?: string;
  explanation?: string;
  pauseCue?: string;
};

const QUIZ_LEFT = 160;
const QUIZ_TOP = 300;
const CARDS_X = 1555;

const QuizOptions: React.FC<{ options: QuizOption[]; correct?: string; at: (i: number) => number; reveal?: number }> = ({
  options,
  correct,
  at,
  reveal = 0,
}) => {
  const frame = useCurrentFrame();
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      {options.map((option, i) => {
        const right = option.key === correct;
        const enter = rise(frame, at(i), 30);
        const lit = right ? reveal : 0;
        return (
          <div
            key={option.key}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 26,
              padding: "18px 28px",
              borderRadius: 20,
              background: lit > 0 ? `rgba(95,211,148,${0.16 * lit})` : C.panel,
              border: `2px solid ${lit > 0 ? C.ok : C.line}`,
              transform: enter.transform,
              opacity: enter.opacity * (right ? 1 : 1 - 0.6 * reveal),
            }}
          >
            <div
              style={{
                width: 62,
                height: 62,
                borderRadius: 999,
                flexShrink: 0,
                display: "grid",
                placeItems: "center",
                background: lit > 0 ? C.ok : "rgba(236,217,154,0.12)",
                color: lit > 0 ? C.ink : C.goldSoft,
                fontFamily: UI,
                fontWeight: 800,
                fontSize: 30,
              }}
            >
              {lit > 0.5 ? "✓" : option.key}
            </div>
            <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 34, color: C.ivory }}>{option.text}</div>
          </div>
        );
      })}
    </div>
  );
};

/** Coluna de texto: largura menor quando as cartas ocupam a direita. */
const QuizColumn: React.FC<{ visual: QuizVisual; children: React.ReactNode }> = ({ visual, children }) => (
  <div
    style={{
      position: "absolute",
      left: QUIZ_LEFT,
      top: QUIZ_TOP,
      width: visual.hole ? 1060 : 1320,
      display: "flex",
      flexDirection: "column",
      gap: 30,
    }}
  >
    {children}
  </div>
);

const QuizCards: React.FC<{ visual: Visual }> = ({ visual }) => {
  const frame = useCurrentFrame();
  if (!visual.hole) return null;
  const board = visual.board ?? [];
  // Até cinco comunitárias cabem na coluna de 460 px.
  const boardWidth = Math.min(108, Math.floor((460 - (board.length - 1) * 10) / Math.max(1, board.length)));
  const label = (text: string) => (
    <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 18, letterSpacing: 4, color: C.muted }}>{text}</div>
  );
  return (
    <div
      style={{
        position: "absolute",
        left: CARDS_X - 240,
        top: QUIZ_TOP - 6,
        width: 480,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 14,
        ...rise(frame, 6, 30),
      }}
    >
      {board.length > 0 && label("COMUNITÁRIAS")}
      <div style={{ display: "flex", gap: 10 }}>
        {board.map((code) => (
          <Card key={code} code={code} width={boardWidth} />
        ))}
      </div>
      <div style={{ height: 10 }} />
      {label("SUAS PRIVADAS")}
      <div style={{ display: "flex", gap: 12 }}>
        {visual.hole.map((code) => (
          <Card key={code} code={code} width={118} />
        ))}
      </div>
    </div>
  );
};

const PauseRing: React.FC<{ visual: Visual; at: number; progress: number }> = ({ visual, at, progress }) => {
  const frame = useCurrentFrame();
  const size = visual.hole ? 150 : 180;
  return (
    <Abs x={visual.hole ? CARDS_X : 1650} y={visual.hole ? 860 : 560} style={{ opacity: ramp(frame, at, 12) }}>
      <div style={{ position: "relative", width: size, height: size }}>
        <svg width={size} height={size} viewBox="0 0 220 220">
          <circle cx="110" cy="110" r="94" fill="none" stroke="rgba(255,255,255,0.1)" strokeWidth="12" />
          <circle
            cx="110"
            cy="110"
            r="94"
            fill="none"
            stroke={C.gold}
            strokeWidth="12"
            strokeLinecap="round"
            strokeDasharray={2 * Math.PI * 94}
            strokeDashoffset={2 * Math.PI * 94 * progress}
            transform="rotate(-90 110 110)"
          />
        </svg>
        <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", gap: size * 0.08 }}>
          <div style={{ width: size * 0.1, height: size * 0.36, borderRadius: 5, background: C.ivory }} />
          <div style={{ width: size * 0.1, height: size * 0.36, borderRadius: 5, background: C.ivory }} />
        </div>
      </div>
    </Abs>
  );
};

const Quiz: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const visual = visualOf(props) as QuizVisual;
  const options = visual.options ?? [];
  const pause = safeCue(props, visual.pauseCue ?? "Pause", props.speechEnd - 20);
  const countdown = interpolate(frame, [pause, props.duration], [0, 1], CLAMP);
  const optionAt = (i: number) => safeCue(props, options[i].cue ?? options[i].key, spaced(props, i + 1, options.length + 2));
  return (
    <AbsoluteFill>
      <Header props={props} />
      <QuizColumn visual={visual}>
        <Body at={4} size={42} color={C.ivory}>
          {visual.question ?? ""}
        </Body>
        <QuizOptions options={options} at={optionAt} />
      </QuizColumn>
      <QuizCards visual={visual} />
      <PauseRing visual={visual} at={pause} progress={countdown} />
      {options.map((option, i) => (
        <Sfx key={option.key} at={optionAt(i)} name="pop" volume={0.2} />
      ))}
    </AbsoluteFill>
  );
};

const Answer: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const visual = visualOf(props) as QuizVisual;
  const options = visual.options ?? [];
  const at = safeCue(props, visual.cue, 12);
  const reveal = glide(frame, at, fps, 18);
  return (
    <AbsoluteFill>
      <Header props={props} />
      <QuizColumn visual={visual}>
        <Body at={0} size={42} color={C.ivory}>
          {visual.question ?? ""}
        </Body>
        <QuizOptions options={options} correct={visual.correct} at={() => -30} reveal={reveal} />
        {visual.explanation && (
          <Panel at={at + 24} accent={C.ok} style={{ padding: "22px 30px" }}>
            <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 30, lineHeight: 1.35, color: C.ivory }}>{visual.explanation}</div>
          </Panel>
        )}
      </QuizColumn>
      <QuizCards visual={visual} />
      <Sfx at={at} name="pop" volume={0.32} />
    </AbsoluteFill>
  );
};

export const PINEAPPLE_DRAW_SCENES: SceneMap = {
  "mode:hand": HandScene,
  "mode:intro": Intro,
  "mode:cards": Cards,
  "mode:deal3": Deal3,
  "mode:draw": DrawRound,
  "mode:ranking": Ranking,
  "mode:bets": Bets,
  "mode:money": Metrics,
  "mode:totals": Metrics,
  "mode:sidepots": Metrics,
  "mode:formula": Formula,
  "mode:steps": Lines,
  "mode:reopen": Lines,
  "mode:status": Lines,
  "mode:formats": Lines,
  "mode:recap": Lines,
  "mode:outro": Outro,
  "mode:quiz": Quiz,
  "mode:answer": Answer,
};

