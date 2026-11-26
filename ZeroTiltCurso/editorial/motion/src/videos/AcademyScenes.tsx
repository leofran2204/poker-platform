import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { Sfx } from "../components/Frame";
import { Table } from "../components/Table";
import { Panel, Pill } from "../components/Text";
import { ramp } from "../lib/anim";
import { C, DISPLAY, UI } from "../theme";
import { SceneMap, SceneProps } from "../Video";
import { Header, Note, PINEAPPLE_DRAW_SCENES, safeCue, spaced, visualOf } from "./PineappleDrawRules";

// Cenas das aulas de estratégia da Academy (Texas, Short Deck e Omaha). Somam-se às cenas
// do Pineapple (mesa animada com `variant: "holdem"`, listas, métricas, exercício e resposta):
// - "grid": grade 13 × 13 de mãos iniciais com grupos que acendem na frase de cada um;
// - "positions": mesa vista de cima com o nome de cada posição;
// - "compare": duas colunas lado a lado (por exemplo, board seco × molhado).
// Os roteiros saem de tools/build_academy.py, que expande os ranges e confere as contas.

type Group = { label: string; hands: string[]; cue?: string; detail?: string };
type SeatInfo = { label: string; value?: string; cue?: string; hero?: boolean; dim?: boolean };
type Column = { title: string; items: string[]; cues?: string[]; tone?: "gold" | "mint" | "danger" | "ok" };
type AcademyVisual = { groups?: Group[]; seats?: SeatInfo[]; button?: number; columns?: Column[]; note?: string; noteCue?: string };

const RANKS = "AKQJT98765432";
const GROUP_COLORS = [C.gold, C.mint, C.sky, C.danger];
const TONE: Record<NonNullable<Column["tone"]>, string> = { gold: C.gold, mint: C.mint, danger: C.danger, ok: C.ok };

/** Nome da célula: par na diagonal, suited acima, offsuit abaixo (convenção das tabelas de range). */
const cellName = (row: number, col: number) =>
  row === col ? RANKS[row] + RANKS[col] : row < col ? `${RANKS[row]}${RANKS[col]}s` : `${RANKS[col]}${RANKS[row]}o`;

const Grid: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const visual = visualOf(props) as AcademyVisual;
  const groups = visual.groups ?? [];
  const at = groups.map((group, i) => safeCue(props, group.cue, spaced(props, i, groups.length)));
  const owner = new Map<string, number>();
  groups.forEach((group, g) => group.hands.forEach((hand) => owner.has(hand) || owner.set(hand, g)));
  const CELL = 54;
  const LEFT = 150;
  const TOP = 262;
  return (
    <AbsoluteFill>
      <Header props={props} />
      {Array.from({ length: 13 }, (_, row) =>
        Array.from({ length: 13 }, (_, col) => {
          const name = cellName(row, col);
          const g = owner.get(name);
          const lit = g === undefined ? 0 : ramp(frame, at[g] + ((row + col) % 7), 10);
          const color = g === undefined ? C.gold : GROUP_COLORS[g % GROUP_COLORS.length];
          return (
            <div
              key={name}
              style={{
                position: "absolute",
                left: LEFT + col * CELL,
                top: TOP + row * CELL,
                width: CELL - 4,
                height: CELL - 4,
                borderRadius: 6,
                background: lit > 0 ? color : row === col ? "rgba(236,217,154,0.10)" : "rgba(255,255,255,0.05)",
                opacity: lit > 0 ? 0.25 + 0.75 * lit : ramp(frame, 2 + row, 12),
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                fontFamily: UI,
                fontWeight: 800,
                fontSize: 17,
                color: lit > 0.5 ? C.ink : C.faint,
              }}
            >
              {name}
            </div>
          );
        }),
      )}
      <div style={{ position: "absolute", left: 960, top: 300, width: 820, display: "flex", flexDirection: "column", gap: 22 }}>
        {groups.map((group, i) => (
          <Panel key={group.label} at={at[i]} accent={GROUP_COLORS[i % GROUP_COLORS.length]} style={{ padding: "22px 30px" }}>
            <div style={{ fontFamily: UI, fontWeight: 800, fontSize: 36, color: C.ivory }}>{group.label}</div>
            {group.detail && <div style={{ fontFamily: UI, fontWeight: 600, fontSize: 26, color: C.muted, marginTop: 6 }}>{group.detail}</div>}
          </Panel>
        ))}
        {visual.note && (
          <div style={{ alignSelf: "flex-start" }}>
            <Pill tone="muted" at={safeCue(props, visual.noteCue, spaced(props, 2, 3) + 12)} size={26}>
              {visual.note}
            </Pill>
          </div>
        )}
      </div>
      {groups.map((group, i) => (
        <Sfx key={group.label} at={at[i]} name="pop" volume={0.22} />
      ))}
    </AbsoluteFill>
  );
};

const Positions: React.FC<SceneProps> = (props) => {
  const frame = useCurrentFrame();
  const visual = visualOf(props) as AcademyVisual;
  const seats = visual.seats ?? [];
  const n = Math.max(2, seats.length);
  const center = { x: 960, y: 600 };
  return (
    <AbsoluteFill>
      <Header props={props} />
      <Table x={center.x} y={center.y} width={1040} height={430} opacity={ramp(frame, 0, 16)} />
      {seats.map((seat, i) => {
        // Você (ou o primeiro lugar) embaixo; os demais em sentido horário, como a ação anda.
        const angle = Math.PI / 2 + (2 * Math.PI * i) / n;
        const x = center.x + 640 * Math.cos(angle);
        const y = center.y + 290 * Math.sin(angle);
        const at = safeCue(props, seat.cue, 6 + i * 3);
        const lit = seat.cue ? ramp(frame, at, 12) : 0;
        const accent = seat.hero ? C.gold : C.mint;
        return (
          <React.Fragment key={seat.label + i}>
            <div
              style={{
                position: "absolute",
                left: x,
                top: y,
                transform: `translate(-50%, -50%) scale(${1 + 0.08 * lit})`,
                minWidth: 170,
                padding: "14px 22px",
                borderRadius: 18,
                textAlign: "center",
                background: "linear-gradient(180deg, rgba(16,24,20,0.97), rgba(5,10,8,0.97))",
                border: `3px solid ${lit > 0 ? accent : "rgba(236,217,154,0.3)"}`,
                boxShadow: lit > 0 ? `0 0 ${36 * lit}px ${accent}` : "none",
                opacity: ramp(frame, 3 + i * 2, 12) * (seat.dim ? 0.45 : 1),
              }}
            >
              <div style={{ fontFamily: UI, fontWeight: 800, fontSize: 32, color: seat.hero ? C.gold : C.ivory }}>{seat.label}</div>
              {seat.value && (
                <div style={{ fontFamily: DISPLAY, fontWeight: 700, fontSize: 30, color: C.goldSoft, marginTop: 4, opacity: seat.cue ? lit : 1 }}>{seat.value}</div>
              )}
            </div>
            {visual.button === i && (
              <div
                style={{
                  position: "absolute",
                  left: x + (center.x - x) * 0.22,
                  top: y + (center.y - y) * 0.22,
                  transform: "translate(-50%, -50%)",
                  width: 46,
                  height: 46,
                  borderRadius: 23,
                  background: C.paper,
                  color: C.ink,
                  fontFamily: UI,
                  fontWeight: 800,
                  fontSize: 24,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  opacity: ramp(frame, 10, 12),
                }}
              >
                D
              </div>
            )}
            {seat.cue && <Sfx at={at} name="pop" volume={0.18} />}
          </React.Fragment>
        );
      })}
      <Note text={visual.note} at={safeCue(props, visual.noteCue, spaced(props, 2, 3) + 12)} tone="muted" />
    </AbsoluteFill>
  );
};

const Compare: React.FC<SceneProps> = (props) => {
  const visual = visualOf(props) as AcademyVisual;
  const columns = visual.columns ?? [];
  return (
    <AbsoluteFill>
      <Header props={props} />
      <div style={{ position: "absolute", left: 140, top: 300, width: 1640, display: "grid", gridTemplateColumns: `repeat(${columns.length}, 1fr)`, gap: 40 }}>
        {columns.map((column, c) => {
          const accent = TONE[column.tone ?? (c === 0 ? "mint" : "gold")];
          const first = safeCue(props, column.cues?.[0], spaced(props, c * 2, columns.length * 2));
          return (
            <div key={column.title} style={{ display: "flex", flexDirection: "column", gap: 18 }}>
              <Pill tone={column.tone ?? (c === 0 ? "mint" : "gold")} at={Math.max(4, first - 8)} size={30}>
                {column.title}
              </Pill>
              {column.items.map((item, i) => {
                const at = safeCue(props, column.cues?.[i], spaced(props, c * column.items.length + i, columns.length * column.items.length));
                return (
                  <Panel key={item} at={at} accent={accent} style={{ padding: "22px 28px" }}>
                    <div style={{ fontFamily: UI, fontWeight: 700, fontSize: 34, lineHeight: 1.25, color: C.ivory }}>{item}</div>
                    <Sfx at={at} name="pop" volume={0.2} />
                  </Panel>
                );
              })}
            </div>
          );
        })}
      </div>
      <Note text={visual.note} at={safeCue(props, visual.noteCue, spaced(props, 2, 3) + 12)} tone="muted" />
    </AbsoluteFill>
  );
};

export const ACADEMY_SCENES: SceneMap = {
  ...PINEAPPLE_DRAW_SCENES,
  "mode:grid": Grid,
  "mode:positions": Positions,
  "mode:compare": Compare,
};
