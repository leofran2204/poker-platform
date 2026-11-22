import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { Card } from "../components/Card";
import { Chip, ChipStack } from "../components/Chips";
import { Sfx } from "../components/Frame";
import { Table } from "../components/Table";
import { Headline, Kicker, Pill } from "../components/Text";
import { brl, glide, lerp, ramp } from "../lib/anim";
import { flight, Point } from "../lib/deal";
import { C, UI } from "../theme";
import { SceneProps } from "../Video";

// Cena "hand": uma mesa oval com quatro jogadores, fichas e cartas, guiada por
// eventos do roteiro (src/data/*.json). Cada evento acontece na frase indicada em
// `cue`: apostas saem do jogador e vão para a frente dele, as apostas da rodada
// são recolhidas para o pote, cartas são trocadas ou descartadas e o pote vai
// para o vencedor. As mãos e os valores são conferidos pelo motor do jogo
// (Motor-Rust/tests/pineapple_draw_film_examples.rs).

export type HandPlayer = {
  name: string;
  stack: number;
  /** Aposta já na frente do jogador quando a cena começa. */
  bet?: number;
  cards?: string[];
  folded?: boolean;
  allIn?: boolean;
  /** Selo da troca na street atual quando a cena começa ("Trocou 1", "Manteve", "Descartou"). */
  drawn?: string;
};

export type HandEvent = {
  cue?: string;
  /** Atraso em segundos depois da frase (eventos encadeados na mesma frase). */
  delay?: number;
  type:
    | "deal"
    | "post"
    | "ante"
    | "bet"
    | "call"
    | "raise"
    | "allin"
    | "check"
    | "fold"
    | "collect"
    | "board"
    | "keep"
    | "swap"
    | "discard"
    | "reveal"
    | "highlight"
    | "award"
    | "banner"
    | "turn";
  player?: number;
  /** "turn": segundos do relógio já gastos quando a vez começa (padrão 0). */
  elapsed?: number;
  /** "turn": velocidade do relógio (1 = tempo real). */
  speed?: number;
  /** "turn": o tempo normal e o tempo extra acabam na primeira ação (o sistema age pelo jogador). */
  timeout?: boolean;
  players?: number[];
  /** Total do jogador na rodada depois da ação (post, bet, call, raise, allin). */
  to?: number;
  /** Rótulo curto no selo do jogador. */
  label?: string;
  cards?: string[];
  position?: number;
  receive?: string;
  hole?: number[];
  boardPick?: number[];
  text?: string;
  /** Divisão do pote: valor entregue a cada vencedor em `players`. */
  amounts?: number[];
};

export type HandVisual = {
  mode: "hand";
  players: HandPlayer[];
  dealer?: number;
  hero?: number;
  /** Jogadores com as cartas abertas desde o início da cena. */
  open?: number[];
  board?: string[];
  pot?: number;
  banner?: string;
  note?: string;
  events?: HandEvent[];
};

// Layout das mesas da plataforma (decisão do dono em 20/11/2026, inspirado nas mesas
// de app atuais, sem copiar marca): placa do jogador na borda da mesa; cartas dos
// adversários pequenas e sobrepostas, saindo de trás da placa; as do herói em leque,
// mostrando o canto; comunitárias grandes no centro, com o "Pote total" acima.
// As apostas saem da placa e o pote ganho volta para ela.
// `show`: onde a carta descartada no river fica aberta antes de ir para o muck.
type SeatSpot = { plate: Point; cards: Point; bet: Point; show: Point };
const SEATS: SeatSpot[] = [
  { plate: { x: 960, y: 990 }, cards: { x: 960, y: 868 }, bet: { x: 745, y: 800 }, show: { x: 1175, y: 790 } },
  { plate: { x: 330, y: 640 }, cards: { x: 330, y: 582 }, bet: { x: 548, y: 668 }, show: { x: 505, y: 555 } },
  { plate: { x: 960, y: 318 }, cards: { x: 960, y: 260 }, bet: { x: 1196, y: 440 }, show: { x: 690, y: 400 } },
  { plate: { x: 1590, y: 640 }, cards: { x: 1590, y: 582 }, bet: { x: 1372, y: 668 }, show: { x: 1415, y: 555 } },
];
const TABLE = { x: 960, y: 620, width: 1380, height: 560 };
const BOARD: Point = { x: 960, y: 598 };
const TOTAL: Point = { x: 960, y: 462 };
const POT: Point = { x: 960, y: 712 };
const DECK: Point = { x: 560, y: 456 };
const MUCK: Point = { x: 1360, y: 456 };
const BOARD_W = 116;
const BOARD_GAP = 128;
/** Cartas fechadas dos adversários: pequenas e sobrepostas. */
const BACK_W = 64;
const BACK_GAP = 28;
/** Cartas abertas dos adversários no showdown. */
const OPEN_W = 86;
const OPEN_GAP = 92;
/** Descarte do river: aberto para todos por 2 s (regra pineapple_draw_v4) e depois fechado para o muck. */
const SHOW_W = 92;
const SHOW_SECONDS = 2;
/** Leque do herói: cada carta cobre metade da anterior e só o canto aparece. */
const HERO_W = 124;
const HERO_GAP = 62;
const HERO_TILT = 7;

const MONEY = new Set(["post", "bet", "call", "raise", "allin"]);
// Mesmos nomes dos botões de ação (decisão do dono em 21/11/2026).
const DEFAULT_LABEL: Record<string, string> = {
  bet: "Aposta",
  call: "Call",
  raise: "Raise",
  allin: "All-in",
  check: "Check",
  fold: "Fold",
};
const DRAW_TAG: Record<string, string> = { keep: "Manteve", swap: "Trocou 1", discard: "Descartou" };

/* Vez do jogador: barra de tempo sob a placa e barra de botões, como numa mesa real.
   No flop, no turn e no river, a primeira vez de cada um na street começa pela troca
   (Trocar/Manter); no river vem o descarte; por fim, a aposta.
   Relógio (BUSINESS_RULES 10.2, 21/11/2026): 15 s por jogada + 5 s de tempo extra, renovados a cada jogada. */
const TURN_SECONDS = 15;
const EXTRA_SECONDS = 5;
const PRESS = 14; // quadros com o botão apertado antes de passar à etapa seguinte
const LINGER = 20; // quadros que a barra fica na tela depois da última ação
const BETTING = new Set(["check", "fold", "bet", "call", "raise", "allin"]);
const ACTIONS = new Set([...BETTING, "keep", "swap", "discard"]);
type Stage = "draw" | "discard" | "bet";
type Turn = {
  player: number;
  start: number;
  steps: Timed[];
  stages: Stage[];
  /** Valor a pagar quando a vez começou (0 = ninguém apostou). */
  toCall: number;
  /** Maior aposta da rodada quando a vez começou. */
  maxBet: number;
  elapsed: number;
  speed: number;
  timeout: boolean;
};
const stageOf = (type: string): Stage => (type === "keep" || type === "swap" ? "draw" : type === "discard" ? "discard" : "bet");

type Timed = HandEvent & { at: number };
type Occupant = { code: string; from: number; to: number; arrive: "start" | "deal" | "swap"; leave: "none" | "muck" | "show" };

const chipCount = (amount: number, big: number) => Math.max(1, Math.min(6, Math.round(amount / big) + 1));

/** Evento ainda não começou (0), em andamento (0–1) ou concluído (1). */
const progress = (frame: number, at: number, fps: number, length = 22) => (frame < at ? 0 : glide(frame, at, fps, length));

export const HandScene: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const visual = props.data.visual as unknown as HandVisual;
  const players = visual.players;
  const hero = visual.hero ?? 0;
  const count = (visual.events ?? []).length;

  // Quadro de cada evento: a frase da narração; sem frase, logo depois do anterior.
  const events: Timed[] = [];
  (visual.events ?? []).forEach((event, index) => {
    let at: number;
    if (event.cue) {
      try {
        at = props.cue(event.cue);
      } catch {
        at = Math.round(props.audioAt + ((props.speechEnd - props.audioAt) * (index + 1)) / (count + 1));
      }
    } else {
      at = (events[events.length - 1]?.at ?? props.audioAt) + Math.round(0.5 * fps);
    }
    at += Math.round((event.delay ?? 0) * fps);
    events.push({ ...event, at: Math.max(at, (events[events.length - 1]?.at ?? 0) + 1) });
  });
  const big = Math.max(100, ...players.map((p) => p.bet ?? 0).filter((bet) => bet > 0).slice(0, 1), 100);

  /* ── Vez de cada jogador: quando começa, quais etapas tem e quanto há a pagar ── */
  const turns: Turn[] = [];
  {
    let boardCount = (visual.board ?? []).length;
    const drew = players.map((p) => Boolean(p.drawn));
    const roundBets = players.map((p) => p.bet ?? 0);
    const left = players.map((p) => p.stack);
    let boundary = props.audioAt;
    let explicit = null as Timed | null;
    let open = null as Turn | null;
    const close = () => {
      open = null;
    };
    for (const event of events) {
      const who = event.player ?? -1;
      if (event.type === "turn" && who >= 0) {
        close();
        explicit = event;
      } else if (ACTIONS.has(event.type) && who >= 0) {
        if (!open || open.player !== who) {
          close();
          const previous = turns[turns.length - 1];
          const floor = previous ? previous.steps[previous.steps.length - 1].at + 6 : props.audioAt;
          const mine: Timed | null = explicit && explicit.player === who ? explicit : null;
          const start: number = mine ? mine.at : Math.max(floor, Math.min(boundary, event.at - 12));
          const draws: boolean = boardCount >= 3 && !drew[who];
          const facing: number = Math.max(...roundBets) - roundBets[who];
          open = {
            player: who,
            start,
            steps: [],
            // Se a vez volta por causa de um aumento, só há aposta.
            stages: draws ? (boardCount >= 5 ? ["draw", "discard", "bet"] : ["draw", "bet"]) : ["bet"],
            toCall: Math.min(facing, left[who]),
            maxBet: Math.max(...roundBets),
            elapsed: mine?.elapsed ?? 0,
            // 0 = automático: o relógio corre em tempo real, mas não passa de 60% do tempo
            // enquanto a narração explica a decisão.
            speed: mine?.speed ?? 0,
            timeout: Boolean(mine?.timeout),
          };
          turns.push(open);
          explicit = null;
        }
        open.steps.push(event);
        if (event.type === "keep" || event.type === "swap") drew[who] = true;
        if (MONEY.has(event.type)) {
          left[who] -= Math.max(0, (event.to ?? roundBets[who]) - roundBets[who]);
          roundBets[who] = event.to ?? roundBets[who];
        }
        if (BETTING.has(event.type)) close();
        boundary = event.at + 6;
      } else if (event.type === "post" && who >= 0) {
        left[who] -= Math.max(0, (event.to ?? 0) - roundBets[who]);
        roundBets[who] = event.to ?? roundBets[who];
        boundary = event.at + 10;
      } else if (event.type === "board") {
        close();
        boardCount += (event.cards ?? []).length;
        drew.fill(false);
        roundBets.fill(0);
        boundary = event.at + 5 * (event.cards ?? []).length + 20;
      } else if (event.type === "collect") {
        close();
        roundBets.fill(0);
        boundary = event.at + 24;
      } else if (event.type !== "banner" && event.type !== "highlight") {
        close();
        boundary = event.at + 12;
      }
    }
    close();
  }
  const turn = [...turns].reverse().find((t) => frame >= t.start && frame <= t.steps[t.steps.length - 1].at + PRESS + LINGER) ?? null;

  /* ── Fichas: stacks, apostas da rodada e pote ── */
  const stacks = players.map((p) => p.stack);
  const bets = players.map((p) => p.bet ?? 0);
  const folded: number[] = players.map((p) => (p.folded ? 1 : 0));
  let pot = visual.pot ?? 0;
  const flying: { from: Point; to: Point; at: number; key: string; color: "red" | "gold" }[] = [];
  const badges: ({ text: string; at: number; tone: string } | null)[] = players.map(() => null);
  // Selo da troca: quantas cartas cada um trocou na street (vale até abrir a próxima carta).
  const drawTags: ({ text: string; at: number } | null)[] = players.map((p) => (p.drawn ? { text: p.drawn, at: -1000 } : null));
  let actor = -1;
  let actorAt = -1000;
  let banner = visual.banner;
  let bannerAt = 0;
  const winners: { player: number; at: number }[] = [];

  for (const event of events) {
    if (frame < event.at) break;
    const t = progress(frame, event.at, fps);
    const who = event.player ?? -1;
    if (MONEY.has(event.type) && who >= 0) {
      const target = event.to ?? bets[who];
      const paid = Math.max(0, target - bets[who]);
      stacks[who] -= paid;
      if (t > 0.9) bets[who] = target;
      else flying.push({ from: SEATS[who].plate, to: SEATS[who].bet, at: event.at, key: `b${event.at}`, color: "red" });
      badges[who] = { text: event.label ?? DEFAULT_LABEL[event.type] ?? "", at: event.at, tone: C.gold };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "ante" && who >= 0) {
      // Ante do big blind: vai direto para o pote, sem contar como aposta da rodada.
      const amount = event.to ?? 0;
      stacks[who] -= amount;
      if (t > 0.9) pot += amount;
      else flying.push({ from: SEATS[who].plate, to: POT, at: event.at, key: `n${event.at}`, color: "red" });
      badges[who] = { text: event.label ?? "Ante", at: event.at, tone: C.gold };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "keep" || event.type === "swap" || event.type === "discard") {
      drawTags[who] = { text: event.label ?? DRAW_TAG[event.type], at: event.at };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "check") {
      badges[who] = { text: event.label ?? DEFAULT_LABEL.check, at: event.at, tone: C.goldSoft };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "fold") {
      folded[who] = t;
      badges[who] = { text: event.label ?? DEFAULT_LABEL.fold, at: event.at, tone: C.danger };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "collect") {
      const total = bets.reduce((sum, bet) => sum + bet, 0);
      if (t > 0.9) {
        pot += total;
        bets.fill(0);
        badges.fill(null);
      } else {
        bets.forEach((bet, i) => {
          if (bet > 0) flying.push({ from: SEATS[i].bet, to: POT, at: event.at, key: `c${event.at}-${i}`, color: "red" });
        });
        // Durante o voo, as pilhas da frente dos jogadores somem aos poucos.
        bets.forEach((bet, i) => (bets[i] = t > 0.35 ? 0 : bet));
        if (t > 0.35) pot += 0;
      }
      actor = -1;
    } else if (event.type === "award") {
      const list = event.players ?? (who >= 0 ? [who] : []);
      const shares = event.amounts ?? list.map(() => Math.floor(pot / Math.max(1, list.length)));
      list.forEach((player, i) => {
        winners.push({ player, at: event.at });
        if (t > 0.9) stacks[player] += shares[i];
        else flying.push({ from: POT, to: SEATS[player].plate, at: event.at, key: `a${event.at}-${player}`, color: "gold" });
        badges[player] = { text: event.label ?? "Vence o pote", at: event.at, tone: C.ok };
      });
      if (t > 0.35) pot = Math.max(0, pot - shares.reduce((sum, share) => sum + share, 0));
      actor = -1;
    } else if (event.type === "banner") {
      banner = event.text;
      bannerAt = event.at;
    } else if (event.type === "board") {
      badges.fill(null);
      drawTags.fill(null);
      actor = -1;
    }
  }

  /* ── Cartas dos jogadores: quem ocupa cada posição ao longo da cena ── */
  const dealAt = events.find((event) => event.type === "deal")?.at;
  const slots: Occupant[][][] = players.map((p, i) =>
    (p.cards ?? []).map((code, k) => [
      {
        code,
        from: dealAt === undefined ? -1000 : dealAt + (k * players.length + i) * 3,
        to: Infinity,
        arrive: dealAt === undefined ? ("start" as const) : ("deal" as const),
        leave: "none" as const,
      },
    ]),
  );
  const removed: { player: number; slot: number; at: number }[] = [];
  const revealAt = players.map((_, i) => (i === hero || (visual.open ?? []).includes(i) ? -1000 : Infinity));
  const foldAt = players.map((p) => (p.folded ? -1000 : Infinity));
  let pick: { player?: number; hole: number[]; board: number[]; at: number; text?: string } | null = null;
  for (const event of events) {
    const who = event.player ?? -1;
    if (event.type === "swap" && who >= 0) {
      const slot = slots[who][event.position ?? 0];
      const current = slot[slot.length - 1];
      current.to = event.at;
      current.leave = "muck";
      slot.push({ code: event.receive ?? current.code, from: event.at + 12, to: Infinity, arrive: "swap", leave: "none" });
    } else if (event.type === "discard" && who >= 0) {
      const index = event.position ?? 0;
      const slot = slots[who][index];
      const current = slot[slot.length - 1];
      current.to = event.at;
      current.leave = "show";
      removed.push({ player: who, slot: index, at: event.at });
    } else if (event.type === "fold" && who >= 0) {
      foldAt[who] = event.at;
    } else if (event.type === "reveal") {
      (event.players ?? (who >= 0 ? [who] : [])).forEach((player) => (revealAt[player] = Math.min(revealAt[player], event.at)));
    } else if (event.type === "highlight" && frame >= event.at) {
      pick = { player: event.player, hole: event.hole ?? [], board: event.boardPick ?? [], at: event.at, text: event.text };
    }
  }

  /* ── Mesa: cartas comunitárias que já estavam e as que chegam ── */
  const board: { code: string; at: number }[] = (visual.board ?? []).map((code) => ({ code, at: -1000 }));
  for (const event of events) {
    if (event.type === "board") (event.cards ?? []).forEach((code, k) => board.push({ code, at: event.at + k * 5 }));
  }

  /** Deslocamento de uma carta na mão: as que restam se reagrupam depois do descarte. */
  const slotOffset = (player: number, slot: number) => {
    let index = slot;
    let size = slots[player].length;
    for (const item of removed.filter((r) => r.player === player)) {
      const t = progress(frame, item.at + 8, fps, 18);
      if (item.slot < slot) index -= t;
      size -= t;
    }
    return index - (size - 1) / 2;
  };
  /** 0 = cartas fechadas atrás da placa; 1 = abertas no showdown. */
  const openness = (player: number) => (revealAt[player] === Infinity ? 0 : ramp(frame, revealAt[player], 14));
  const cardSpot = (player: number, slot: number) => {
    const offset = slotOffset(player, slot);
    const base = SEATS[player].cards;
    if (player === hero) {
      return { x: base.x + offset * HERO_GAP, y: base.y + Math.abs(offset) * 9, width: HERO_W, rotate: offset * HERO_TILT, z: 2 };
    }
    const open = openness(player);
    return {
      x: base.x + offset * lerp(BACK_GAP, OPEN_GAP, open),
      y: base.y - 46 * open,
      width: lerp(BACK_W, OPEN_W, open),
      rotate: offset * 5 * (1 - open),
      z: open > 0.5 ? 4 : 1,
    };
  };

  /* ── Relógio e botões da vez ── */
  // Com tempo esgotado, o relógio corre até zerar na primeira ação: o tempo extra começa
  // quando os 15 s normais acabam (é ali que toca o aviso curto).
  const timedOut = turns.find((item) => item.timeout);
  const extraAt = timedOut
    ? timedOut.start + ((Math.max(0, TURN_SECONDS - timedOut.elapsed) / (TURN_SECONDS + EXTRA_SECONDS - timedOut.elapsed)) * Math.max(1, timedOut.steps[0].at - timedOut.start))
    : null;
  let clock: Clock | null = null;
  let bar: { name: string; buttons: Button[]; pressed: string | null; opacity: number } | null = null;
  if (turn) {
    const last = turn.steps[turn.steps.length - 1];
    // Tempo esgotado: o relógio zera na primeira ação, que o sistema faz pelo jogador.
    const stop = turn.timeout ? turn.steps[0].at : last.at;
    const span = Math.max(1, stop - turn.start);
    const speed = turn.timeout
      ? ((TURN_SECONDS + EXTRA_SECONDS - turn.elapsed) * fps) / span
      : turn.speed > 0
        ? turn.speed
        : Math.min(1, (0.6 * TURN_SECONDS * fps) / span);
    const used = turn.elapsed + ((Math.min(frame, stop) - turn.start) / fps) * speed;
    const main = TURN_SECONDS - used;
    const extra = Math.max(0, EXTRA_SECONDS + Math.min(0, main));
    clock = main > 0 ? { pct: main / TURN_SECONDS, extra: false, secs: Math.ceil(main) } : { pct: extra / EXTRA_SECONDS, extra: true, secs: Math.ceil(extra) };
    let current = 0;
    let pressed: Timed | null = null;
    for (let k = 0; k < turn.steps.length; k++) {
      const step = turn.steps[k];
      if (frame < step.at) break;
      const index = step.type === "fold" ? current : Math.max(current, turn.stages.indexOf(stageOf(step.type)));
      if (frame < step.at + PRESS || k === turn.steps.length - 1) {
        pressed = step;
        current = index;
      } else {
        pressed = null;
        current = Math.min(index + 1, turn.stages.length - 1);
      }
    }
    const fade = Math.min(ramp(frame, turn.start, 8), 1 - ramp(frame, last.at + PRESS + LINGER - 8, 8));
    bar = {
      name: players[turn.player].name.split(" · ")[0],
      buttons: buttonsFor(turn.stages[current], turn, pressed),
      pressed: pressed ? pressedKey(pressed, turn) : null,
      opacity: fade,
    };
  }

  const title = props.data.title;
  const picked = pick as { player?: number; hole: number[]; board: number[]; at: number; text?: string } | null;
  const total = pot + bets.reduce((sum, bet) => sum + bet, 0);

  return (
    <>
      <div style={{ position: "absolute", left: 140, top: 96, width: 680 }}>
        <Kicker at={0}>{props.data.chapter}</Kicker>
        {title && <Headline text={title} at={3} size={44} style={{ marginTop: 8 }} />}
      </div>
      {banner && (
        <div style={{ position: "absolute", right: 120, top: 112 }}>
          <Pill key={banner} tone="mint" at={bannerAt} size={24}>
            {banner}
          </Pill>
        </div>
      )}

      <Table x={TABLE.x} y={TABLE.y} width={TABLE.width} height={TABLE.height} opacity={ramp(frame, 0, 14)} />

      {/* Baralho e descarte */}
      <div style={{ position: "absolute", left: DECK.x - 27, top: DECK.y - 38, opacity: 0.9 }}>
        <Card face={0} width={54} />
      </div>
      {(removed.some((item) => frame >= item.at) || events.some((e) => (e.type === "swap" || e.type === "fold") && frame >= e.at + 16)) && (
        <div style={{ position: "absolute", left: MUCK.x - 27, top: MUCK.y - 38, opacity: 0.55, transform: "rotate(8deg)" }}>
          <Card face={0} width={54} dim={0.5} />
        </div>
      )}

      {/* Pote total (pote do centro + apostas da rodada), acima das comunitárias */}
      {total > 0 && (
        <div
          style={{
            position: "absolute",
            left: TOTAL.x,
            top: TOTAL.y,
            transform: "translate(-50%, -50%)",
            padding: "8px 26px",
            borderRadius: 999,
            background: "rgba(4,12,8,0.82)",
            border: "1px solid rgba(236,217,154,0.35)",
            fontFamily: UI,
            fontWeight: 800,
            fontSize: 30,
            color: C.goldSoft,
            whiteSpace: "nowrap",
            boxShadow: "0 10px 24px rgba(0,0,0,0.45)",
          }}
        >
          Pote total: {brl(total / 100)}
        </div>
      )}
      {/* Fichas do pote, abaixo das comunitárias */}
      {pot > 0 && (
        <div style={{ position: "absolute", left: POT.x, top: POT.y, transform: "translate(-50%, -50%)" }}>
          <ChipStack count={chipCount(pot, big * 2)} color="gold" size={36} label={brl(pot / 100)} />
        </div>
      )}

      {/* Comunitárias, em destaque */}
      {board.map((card, k) => {
        if (frame < card.at) return null;
        const x = BOARD.x + (k - 2) * BOARD_GAP;
        const move = card.at < 0 ? null : flight(frame, card.at, fps, DECK, { x, y: BOARD.y }, -10);
        const face = card.at < 0 ? 1 : ramp(frame, card.at + 10, 12);
        const glow = picked && picked.board.includes(k) ? ramp(frame, picked.at, 12) : 0;
        const dim = picked && !picked.board.includes(k) ? ramp(frame, picked.at, 12) : 0;
        return (
          <div
            key={`board-${k}`}
            style={{
              position: "absolute",
              left: (move?.x ?? x) - BOARD_W / 2,
              top: (move?.y ?? BOARD.y) - (BOARD_W * 1.42) / 2 - 10 * glow,
              opacity: move?.opacity ?? 1,
            }}
          >
            <Card code={card.code} face={face} width={BOARD_W} glow={glow} dim={dim} />
          </div>
        );
      })}

      {/* Cartas dos jogadores */}
      {slots.map((playerSlots, player) =>
        playerSlots.map((slot, k) =>
          slot.map((card, version) => {
            if (frame < card.from) return null;
            const spot = cardSpot(player, k);
            const width = spot.width;
            const home = { x: spot.x, y: spot.y };
            let x = home.x;
            let y = home.y;
            let opacity = 1;
            if (card.arrive !== "start") {
              const move = flight(frame, card.from, fps, DECK, home, -14);
              x = move.x;
              y = move.y;
              opacity = move.opacity;
            }
            if (card.leave === "show" && frame >= card.to) {
              // Descarte do river: vai aberto para a frente do jogador, fica 2 s e segue fechado para o muck.
              const spotShow = SEATS[player].show;
              const come = progress(frame, card.to, fps, 12);
              const go = progress(frame, card.to + SHOW_SECONDS * fps, fps, 18);
              if (go >= 1) return null;
              const at = { x: lerp(home.x, spotShow.x, come), y: lerp(home.y, spotShow.y, come) };
              const w = lerp(width, SHOW_W, come) * (1 - 0.35 * go);
              const face = player === hero ? 1 - go : Math.min(come, 1 - go);
              return (
                <div
                  key={`hole-${player}-${k}-${version}`}
                  style={{
                    position: "absolute",
                    left: lerp(at.x, MUCK.x, go) - w / 2,
                    top: lerp(at.y, MUCK.y, go) - (w * 1.42) / 2,
                    opacity: 1 - go * 0.9,
                    zIndex: 6,
                    transform: `rotate(${spot.rotate * (1 - come) + 8 * go}deg)`,
                  }}
                >
                  <Card code={card.code} face={face} width={w} />
                </div>
              );
            }
            const out = card.to === Infinity ? 0 : progress(frame, card.to, fps, 18);
            const gone = Math.max(out, progress(frame, foldAt[player], fps, 18));
            if (gone >= 1) return null;
            if (gone > 0) {
              x = lerp(home.x, MUCK.x, gone);
              y = lerp(home.y, MUCK.y, gone);
              opacity = 1 - gone * 0.9;
            }
            const shown =
              revealAt[player] === Infinity
                ? 0
                : ramp(frame, Math.max(revealAt[player], card.arrive === "start" ? -1000 : card.from + 12), 12);
            const face = gone > 0 && player !== hero ? Math.min(shown, 1 - gone) : shown;
            const chosen = picked && picked.player === player;
            const glow = chosen && picked.hole.includes(k) ? ramp(frame, picked.at, 12) : 0;
            const dim = chosen && !picked.hole.includes(k) ? ramp(frame, picked.at, 12) : 0;
            return (
              <div
                key={`hole-${player}-${k}-${version}`}
                style={{
                  position: "absolute",
                  left: x - width / 2,
                  top: y - (width * 1.42) / 2 - 12 * glow,
                  opacity,
                  zIndex: gone > 0 ? 5 : spot.z,
                  transform: `rotate(${spot.rotate * (1 - gone)}deg)`,
                  transformOrigin: "50% 100%",
                }}
              >
                <Card code={card.code} face={face} width={width} glow={glow} dim={Math.max(dim, 0.4 * (folded[player] ?? 0))} />
              </div>
            );
          }),
        ),
      )}

      {/* Apostas na frente de cada jogador */}
      {bets.map((bet, i) =>
        bet > 0 ? (
          <div key={`bet-${i}`} style={{ position: "absolute", left: SEATS[i].bet.x, top: SEATS[i].bet.y, transform: "translate(-50%, -50%)" }}>
            <ChipStack count={chipCount(bet, big)} color="red" size={38} label={brl(bet / 100)} />
          </div>
        ) : null,
      )}

      {/* Fichas em movimento */}
      {flying.map((chip) => {
        const move = flight(frame, chip.at, fps, chip.from, chip.to, 0);
        return (
          <React.Fragment key={chip.key}>
            {[0, 1, 2].map((n) => {
              const lag = flight(frame, chip.at + n * 2, fps, chip.from, chip.to, 0);
              return (
                <div key={n} style={{ position: "absolute", left: lag.x - 19, top: lag.y - 19 - n * 3, opacity: move.opacity, zIndex: 6 }}>
                  <Chip color={chip.color} size={38} />
                </div>
              );
            })}
          </React.Fragment>
        );
      })}

      {/* Placas dos jogadores */}
      {players.map((player, i) => {
        const badge = badges[i];
        const tag = drawTags[i];
        const win = winners.find((winner) => winner.player === i);
        const acting = i === actor ? 1 - ramp(frame, actorAt + 24, 20) * 0.6 : 0;
        return (
          <Plate
            key={player.name}
            x={SEATS[i].plate.x}
            y={SEATS[i].plate.y}
            name={player.name}
            stack={stacks[i] <= 0 && (player.allIn || bets[i] > 0 || pot > 0) ? "All-in" : brl(stacks[i] / 100)}
            hero={i === hero}
            dealer={visual.dealer === i}
            folded={folded[i]}
            active={win ? ramp(frame, win.at, 14) : acting}
            badge={badge?.text || undefined}
            badgeColor={badge?.tone}
            badgeOpacity={badge ? ramp(frame, badge.at, 8) : 0}
            tag={tag?.text}
            tagOpacity={tag ? ramp(frame, tag.at, 8) : 0}
            opacity={ramp(frame, 4 + i * 3, 12)}
            clock={turn && turn.player === i ? clock : null}
            clockOpacity={bar?.opacity ?? 0}
          />
        );
      })}

      {/* Botões da vez, como na mesa real */}
      {bar && bar.opacity > 0 && <ActionBar {...bar} />}

      {/* Mão destacada */}
      {picked?.text && (
        <div style={{ position: "absolute", left: 1500, top: 902, transform: "translate(-50%, -50%)" }}>
          <Pill tone="gold" at={picked.at} size={26}>
            {picked.text}
          </Pill>
        </div>
      )}
      {visual.note && (
        <div
          style={{
            position: "absolute",
            left: 140,
            top: 880,
            width: 520,
            fontFamily: UI,
            fontWeight: 600,
            fontSize: 24,
            lineHeight: 1.35,
            color: C.muted,
            opacity: ramp(frame, 20, 16),
          }}
        >
          {visual.note}
        </div>
      )}

      {/* Sons */}
      {events.map((event, i) => {
        if (MONEY.has(event.type) || event.type === "ante" || event.type === "collect" || event.type === "award") return <Sfx key={i} at={event.at} name="chips" volume={0.4} />;
        if (event.type === "deal") return <Sfx key={i} at={event.at} name="deal" />;
        if (event.type === "board" || event.type === "reveal") return <Sfx key={i} at={event.at + 8} name="flip" />;
        if (event.type === "swap" || event.type === "discard" || event.type === "fold") return <Sfx key={i} at={event.at} name="whoosh" volume={0.3} />;
        return null;
      })}
      {/* Aviso curto quando começa o tempo extra (mesmo som da mesa do site). */}
      {extraAt !== null && <Sfx at={extraAt} name="alert" volume={0.5} />}
    </>
  );
};

type PlateProps = {
  x: number;
  y: number;
  name: string;
  stack: string;
  hero: boolean;
  dealer: boolean;
  folded: number;
  active: number;
  badge?: string;
  badgeColor?: string;
  badgeOpacity: number;
  tag?: string;
  tagOpacity: number;
  opacity: number;
  clock: Clock | null;
  clockOpacity: number;
};

type Clock = { pct: number; extra: boolean; secs: number };
type Button = { key: string; label: string; tone: "fold" | "call" | "raise" | "draw" };

/** Botões de cada etapa da vez (rótulos pedidos pelo dono: Check, Aposta, Call, Raise e Fold). */
const buttonsFor = (stage: Stage, turn: Turn, pressed: Timed | null): Button[] => {
  const toCall = turn.toCall;
  const fold: Button = { key: "fold", label: "Fold", tone: "fold" };
  const amount = pressed?.to !== undefined && pressed.to > 0 ? ` ${brl(pressed.to / 100)}` : "";
  if (stage === "draw") {
    return [...(toCall > 0 ? [fold] : []), { key: "keep", label: "Manter", tone: "draw" }, { key: "swap", label: "Trocar", tone: "draw" }];
  }
  if (stage === "discard") return [...(toCall > 0 ? [fold] : []), { key: "discard", label: "Descartar", tone: "draw" }];
  const allin = pressed?.type === "allin";
  if (toCall > 0) {
    return [
      fold,
      { key: "call", label: allin && pressedKey(pressed, turn) === "call" ? "All-in" : `Call ${brl(toCall / 100)}`, tone: "call" },
      { key: "raise", label: allin && pressedKey(pressed, turn) === "raise" ? "All-in" : pressed?.type === "raise" ? `Raise${amount}` : "Raise", tone: "raise" },
    ];
  }
  return [
    { key: "check", label: "Check", tone: "call" },
    { key: "bet", label: allin ? "All-in" : pressed?.type === "bet" ? `Aposta${amount}` : "Aposta", tone: "raise" },
  ];
};

/** Qual botão a ação aperta. All-in que só iguala é um call; o resto, aposta ou raise. */
const pressedKey = (step: Timed, turn: Turn) => {
  if (step.type !== "allin") return step.type;
  if ((step.to ?? 0) <= turn.maxBet) return "call";
  return turn.toCall > 0 ? "raise" : "bet";
};

const TONES: Record<Button["tone"], string> = { fold: C.danger, call: C.ok, raise: C.gold, draw: C.mint };

/** Barra de botões da vez, no canto inferior direito, como nas mesas de app. */
const ActionBar: React.FC<{ name: string; buttons: Button[]; pressed: string | null; opacity: number }> = ({ name, buttons, pressed, opacity }) => (
  <div style={{ position: "absolute", right: 64, bottom: 44, opacity, zIndex: 7, display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 10 }}>
    <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 22, color: C.muted, letterSpacing: 1 }}>Vez de {name}</div>
    <div style={{ display: "flex", gap: 12 }}>
      {buttons.map((button) => {
        const on = pressed === button.key;
        const tone = TONES[button.tone];
        return (
          <div
            key={button.key}
            style={{
              minWidth: 168,
              padding: "14px 22px",
              borderRadius: 14,
              textAlign: "center",
              fontFamily: UI,
              fontWeight: 800,
              fontSize: 26,
              whiteSpace: "nowrap",
              color: on ? C.ink : C.ivory,
              background: on ? tone : "linear-gradient(180deg, rgba(22,32,27,0.96), rgba(8,14,11,0.96))",
              border: `2px solid ${tone}`,
              opacity: pressed && !on ? 0.45 : 1,
              transform: on ? "translateY(3px) scale(0.95)" : undefined,
              boxShadow: on ? `inset 0 3px 8px rgba(0,0,0,0.35), 0 0 22px ${tone}` : "0 10px 22px rgba(0,0,0,0.45)",
            }}
          >
            {button.label}
          </div>
        );
      })}
    </div>
  </div>
);

/** Barra de tempo sob a placa: verde no tempo normal, dourada nos 5 s finais e vermelha,
    com "TEMPO EXTRA" e a contagem, nos 5 s de tempo extra (como na mesa do site). */
const ClockBar: React.FC<{ clock: Clock; opacity: number }> = ({ clock, opacity }) => {
  const color = clock.extra ? "#e05a4f" : clock.secs <= 5 ? "#d4a843" : "#6fcf7f";
  return (
    <>
      <div style={{ position: "absolute", left: 14, right: 14, bottom: 5, height: 7, borderRadius: 999, background: "rgba(5,14,9,0.9)", overflow: "hidden", opacity }}>
        <div style={{ width: `${Math.max(0, Math.min(1, clock.pct)) * 100}%`, height: "100%", borderRadius: 999, background: color }} />
      </div>
      <div style={{ position: "absolute", left: "100%", top: "50%", marginLeft: 10, transform: "translateY(-50%)", opacity, fontFamily: UI, fontWeight: 800, fontSize: 20, color, whiteSpace: "nowrap" }}>
        {clock.extra ? `TEMPO EXTRA ${clock.secs} s` : `${clock.secs} s`}
      </div>
    </>
  );
};

/** Placa do jogador: avatar, nome e fichas; abaixo, o selo da troca e a última ação. */
const Plate: React.FC<PlateProps> = ({ x, y, name, stack, hero, dealer, folded, active, badge, badgeColor = C.gold, badgeOpacity, tag, tagOpacity, opacity, clock, clockOpacity }) => (
  <div
    style={{
      position: "absolute",
      left: x,
      top: y,
      transform: "translate(-50%, -50%)",
      zIndex: 3,
      opacity: opacity * (1 - 0.5 * folded),
      filter: folded > 0 ? `grayscale(${folded})` : undefined,
    }}
  >
    <div
      style={{
        position: "relative",
        display: "flex",
        alignItems: "center",
        gap: 14,
        width: hero ? 268 : 236,
        padding: "8px 18px 8px 10px",
        borderRadius: 16,
        background: "linear-gradient(180deg, rgba(16,24,20,0.97), rgba(5,10,8,0.97))",
        border: `2px solid ${active > 0 ? `rgba(236,200,95,${0.4 + 0.6 * active})` : hero ? "rgba(236,200,95,0.45)" : "rgba(255,255,255,0.1)"}`,
        boxShadow: `0 14px 30px rgba(0,0,0,0.55)${active > 0 ? `, 0 0 ${28 * active}px rgba(236,200,95,${0.45 * active})` : ""}`,
        whiteSpace: "nowrap",
      }}
    >
      <div
        style={{
          width: 54,
          height: 54,
          flex: "none",
          borderRadius: 999,
          display: "grid",
          placeItems: "center",
          fontFamily: UI,
          fontWeight: 800,
          fontSize: 24,
          color: hero ? C.ink : C.goldSoft,
          background: hero ? `linear-gradient(160deg, ${C.goldSoft}, ${C.gold})` : "rgba(236,217,154,0.12)",
          border: hero ? "none" : "1px solid rgba(236,217,154,0.3)",
        }}
      >
        {name.slice(0, 1)}
      </div>
      <div style={{ fontFamily: UI, lineHeight: 1.15, textAlign: "center", flex: 1 }}>
        <div style={{ color: C.ivory, fontWeight: 700, fontSize: 24 }}>{name}</div>
        <div style={{ color: C.goldSoft, fontWeight: 800, fontSize: 24 }}>{stack}</div>
      </div>
      {dealer && (
        <div
          style={{
            position: "absolute",
            right: -16,
            top: -16,
            width: 36,
            height: 36,
            borderRadius: 999,
            background: C.paper,
            color: C.ink,
            display: "grid",
            placeItems: "center",
            fontFamily: UI,
            fontWeight: 800,
            fontSize: 18,
            boxShadow: "0 6px 14px rgba(0,0,0,0.45)",
          }}
        >
          D
        </div>
      )}
      {clock && <ClockBar clock={clock} opacity={clockOpacity} />}
    </div>
    <div style={{ position: "absolute", left: "50%", top: "100%", marginTop: 8, transform: "translateX(-50%)", display: "flex", gap: 8, whiteSpace: "nowrap" }}>
      {tag && (
        <div
          style={{
            opacity: tagOpacity,
            transform: `scale(${0.8 + 0.2 * tagOpacity})`,
            padding: "4px 14px",
            borderRadius: 999,
            background: "rgba(4,12,8,0.9)",
            border: `2px solid ${C.mint}`,
            color: C.mint,
            fontFamily: UI,
            fontWeight: 800,
            fontSize: 18,
          }}
        >
          {tag}
        </div>
      )}
      {badge && (
        <div
          style={{
            opacity: badgeOpacity,
            transform: `scale(${0.8 + 0.2 * badgeOpacity})`,
            padding: "4px 14px",
            borderRadius: 999,
            background: badgeColor,
            color: C.ink,
            fontFamily: UI,
            fontWeight: 800,
            fontSize: 18,
            letterSpacing: 1,
            boxShadow: "0 8px 18px rgba(0,0,0,0.4)",
          }}
        >
          {badge}
        </div>
      )}
    </div>
  </div>
);
