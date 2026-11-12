import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { Card } from "../components/Card";
import { Chip, ChipStack, Pot } from "../components/Chips";
import { Sfx } from "../components/Frame";
import { Seat, Table } from "../components/Table";
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
    | "banner";
  player?: number;
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

/** `pile` é a pilha de fichas do jogador: de lá saem as apostas e para lá vai o pote ganho. */
type SeatSpot = { plate: Point; cards: Point; bet: Point; pile: Point };
const SEATS: SeatSpot[] = [
  { plate: { x: 960, y: 952 }, cards: { x: 960, y: 800 }, bet: { x: 735, y: 772 }, pile: { x: 760, y: 952 } },
  { plate: { x: 250, y: 610 }, cards: { x: 555, y: 580 }, bet: { x: 610, y: 716 }, pile: { x: 250, y: 706 } },
  { plate: { x: 960, y: 300 }, cards: { x: 960, y: 440 }, bet: { x: 1176, y: 452 }, pile: { x: 760, y: 300 } },
  { plate: { x: 1670, y: 610 }, cards: { x: 1365, y: 580 }, bet: { x: 1310, y: 716 }, pile: { x: 1670, y: 706 } },
];
const TABLE = { x: 960, y: 610, width: 1080, height: 480 };
const BOARD: Point = { x: 960, y: 648 };
const POT: Point = { x: 960, y: 540 };
const DECK: Point = { x: 706, y: 512 };
const MUCK: Point = { x: 1214, y: 512 };
const BOARD_W = 92;
const BOARD_GAP = 102;

const MONEY = new Set(["post", "bet", "call", "raise", "allin"]);
const DEFAULT_LABEL: Record<string, string> = {
  bet: "Aposta",
  call: "Paga",
  raise: "Aumenta",
  allin: "All-in",
  check: "Check",
  fold: "Desiste",
  keep: "Mantém",
  swap: "Troca 1",
  discard: "Descarta",
};

type Timed = HandEvent & { at: number };
type Occupant = { code: string; from: number; to: number; arrive: "start" | "deal" | "swap"; leave: "none" | "muck" };

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

  /* ── Fichas: stacks, apostas da rodada e pote ── */
  const stacks = players.map((p) => p.stack);
  const bets = players.map((p) => p.bet ?? 0);
  const folded: number[] = players.map((p) => (p.folded ? 1 : 0));
  let pot = visual.pot ?? 0;
  const flying: { from: Point; to: Point; at: number; key: string; color: "red" | "gold" }[] = [];
  const badges: ({ text: string; at: number; tone: string } | null)[] = players.map(() => null);
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
      else flying.push({ from: SEATS[who].pile, to: SEATS[who].bet, at: event.at, key: `b${event.at}`, color: "red" });
      badges[who] = { text: event.label ?? DEFAULT_LABEL[event.type] ?? "", at: event.at, tone: C.gold };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "ante" && who >= 0) {
      // Ante do big blind: vai direto para o pote, sem contar como aposta da rodada.
      const amount = event.to ?? 0;
      stacks[who] -= amount;
      if (t > 0.9) pot += amount;
      else flying.push({ from: SEATS[who].pile, to: POT, at: event.at, key: `n${event.at}`, color: "red" });
      badges[who] = { text: event.label ?? "Ante", at: event.at, tone: C.gold };
      actor = who;
      actorAt = event.at;
    } else if (event.type === "check" || event.type === "keep" || event.type === "swap" || event.type === "discard") {
      badges[who] = { text: event.label ?? DEFAULT_LABEL[event.type], at: event.at, tone: event.type === "check" ? C.goldSoft : C.mint };
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
        else flying.push({ from: POT, to: SEATS[player].pile, at: event.at, key: `a${event.at}-${player}`, color: "gold" });
        badges[player] = { text: event.label ?? "Vence o pote", at: event.at, tone: C.ok };
      });
      if (t > 0.35) pot = Math.max(0, pot - shares.reduce((sum, share) => sum + share, 0));
      actor = -1;
    } else if (event.type === "banner") {
      banner = event.text;
      bannerAt = event.at;
    } else if (event.type === "board") {
      badges.fill(null);
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
      current.leave = "muck";
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

  const cardWidth = (player: number) => (player === hero ? 104 : 68);
  const cardGap = (player: number) => (player === hero ? 112 : 73);
  /** Posição horizontal de uma carta: as que restam se reagrupam depois do descarte. */
  const slotX = (player: number, slot: number) => {
    const total = slots[player].length;
    let index = slot;
    let size = total;
    for (const item of removed.filter((r) => r.player === player)) {
      const t = progress(frame, item.at + 8, fps, 18);
      if (item.slot < slot) index -= t;
      size -= t;
    }
    return SEATS[player].cards.x + (index - (size - 1) / 2) * cardGap(player);
  };

  const title = props.data.title;
  const picked = pick as { player?: number; hole: number[]; board: number[]; at: number; text?: string } | null;

  return (
    <>
      <div style={{ position: "absolute", left: 140, top: 104, width: 1180 }}>
        <Kicker at={0}>{props.data.chapter}</Kicker>
        {title && <Headline text={title} at={3} size={46} style={{ marginTop: 8 }} />}
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

      {/* Pote */}
      {pot > 0 && (
        <div style={{ position: "absolute", left: POT.x, top: POT.y, transform: "translate(-50%, -50%)", display: "flex", alignItems: "center", gap: 14 }}>
          <ChipStack count={chipCount(pot, big * 2)} color="gold" size={40} />
          <Pot value={brl(pot / 100)} scale={0.78} />
        </div>
      )}

      {/* Comunitárias */}
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
            const width = cardWidth(player);
            const home = { x: slotX(player, k), y: SEATS[player].cards.y };
            let x = home.x;
            let y = home.y;
            let opacity = 1;
            if (card.arrive !== "start") {
              const move = flight(frame, card.from, fps, DECK, home, -14);
              x = move.x;
              y = move.y;
              opacity = move.opacity;
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
                style={{ position: "absolute", left: x - width / 2, top: y - (width * 1.42) / 2 - 12 * glow, opacity, zIndex: gone > 0 ? 3 : 2 }}
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
                <div key={n} style={{ position: "absolute", left: lag.x - 19, top: lag.y - 19 - n * 3, opacity: move.opacity, zIndex: 5 }}>
                  <Chip color={chip.color} size={38} />
                </div>
              );
            })}
          </React.Fragment>
        );
      })}

      {/* Pilha de fichas de cada jogador */}
      {players.map((player, i) =>
        stacks[i] > 0 ? (
          <div
            key={`pile-${player.name}`}
            style={{
              position: "absolute",
              left: SEATS[i].pile.x,
              top: SEATS[i].pile.y,
              transform: "translate(-50%, -50%)",
              display: "flex",
              alignItems: "flex-end",
              gap: 4,
              opacity: ramp(frame, 6 + i * 3, 12) * (1 - 0.6 * (folded[i] ?? 0)),
            }}
          >
            <ChipStack count={Math.max(1, Math.min(7, Math.round(stacks[i] / 2200)))} color="blue" size={34} />
            <ChipStack count={Math.max(1, Math.min(5, Math.round(stacks[i] / 4000)))} color="green" size={34} />
          </div>
        ) : null,
      )}

      {/* Assentos */}
      {players.map((player, i) => {
        const badge = badges[i];
        const win = winners.find((winner) => winner.player === i);
        const acting = i === actor ? 1 - ramp(frame, actorAt + 24, 20) * 0.6 : 0;
        return (
          <Seat
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
            opacity={ramp(frame, 4 + i * 3, 12)}
          />
        );
      })}

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
    </>
  );
};
