import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { pop, ramp, rise } from "../lib/anim";
import { C, DISPLAY, UI } from "../theme";

export const Kicker: React.FC<{ children: React.ReactNode; at?: number; color?: string; style?: React.CSSProperties }> = ({
  children,
  at = 0,
  color = C.mint,
  style,
}) => {
  const frame = useCurrentFrame();
  return (
    <div
      style={{
        fontFamily: UI,
        fontWeight: 800,
        fontSize: 26,
        letterSpacing: 6,
        textTransform: "uppercase",
        color,
        ...rise(frame, at, 20),
        ...style,
      }}
    >
      {children}
    </div>
  );
};

/** Título que entra palavra por palavra, com máscara e mola. */
export const Headline: React.FC<{
  text: string;
  at?: number;
  size?: number;
  color?: string;
  accent?: string[];
  stagger?: number;
  serif?: boolean;
  style?: React.CSSProperties;
}> = ({ text, at = 0, size = 96, color = C.ivory, accent = [], stagger = 4, serif = true, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const words = text.split(" ");
  return (
    <div
      style={{
        fontFamily: serif ? DISPLAY : UI,
        fontWeight: serif ? 600 : 800,
        fontSize: size,
        lineHeight: 1.08,
        letterSpacing: serif ? -1 : 0,
        color,
        display: "flex",
        flexWrap: "wrap",
        gap: `0 ${size * 0.26}px`,
        ...style,
      }}
    >
      {words.map((word, i) => {
        const t = pop(frame, at + i * stagger, fps, 16);
        const bare = (value: string) => value.replace(/[.,?!:]/g, "");
        const isAccent = accent.map(bare).includes(bare(word));
        return (
          <span key={i} style={{ display: "inline-block", overflow: "hidden", paddingBottom: size * 0.12 }}>
            <span
              style={{
                display: "inline-block",
                transform: `translateY(${(1 - t) * 110}%)`,
                color: isAccent ? C.gold : undefined,
                fontStyle: isAccent && serif ? "italic" : undefined,
              }}
            >
              {word}
            </span>
          </span>
        );
      })}
    </div>
  );
};

export const Body: React.FC<{ children: React.ReactNode; at?: number; size?: number; color?: string; style?: React.CSSProperties }> = ({
  children,
  at = 0,
  size = 36,
  color = C.muted,
  style,
}) => {
  const frame = useCurrentFrame();
  return (
    <div style={{ fontFamily: UI, fontWeight: 500, fontSize: size, lineHeight: 1.4, color, ...rise(frame, at, 24), ...style }}>
      {children}
    </div>
  );
};

/** Cápsula de destaque (rótulos, termos, avisos). */
export const Pill: React.FC<{
  children: React.ReactNode;
  at?: number;
  tone?: "gold" | "mint" | "danger" | "ok" | "muted";
  size?: number;
  style?: React.CSSProperties;
}> = ({ children, at = 0, tone = "gold", size = 26, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = pop(frame, at, fps, 13);
  const colors = {
    gold: [C.gold, C.ink],
    mint: [C.mint, C.ink],
    danger: [C.danger, "#1a0606"],
    ok: [C.ok, "#04170c"],
    muted: ["rgba(233,240,235,0.12)", C.ivory],
  }[tone];
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 10,
        padding: `${size * 0.4}px ${size * 0.85}px`,
        borderRadius: 999,
        background: colors[0],
        color: colors[1],
        fontFamily: UI,
        fontWeight: 800,
        fontSize: size,
        letterSpacing: 1,
        whiteSpace: "nowrap",
        transform: `scale(${0.6 + 0.4 * t})`,
        opacity: Math.min(1, t * 1.4),
        boxShadow: "0 10px 24px rgba(0,0,0,0.35)",
        ...style,
      }}
    >
      {children}
    </div>
  );
};

/** Painel de vidro escuro com borda dourada à esquerda. */
export const Panel: React.FC<{ children: React.ReactNode; at?: number; accent?: string; style?: React.CSSProperties }> = ({
  children,
  at = 0,
  accent = C.gold,
  style,
}) => {
  const frame = useCurrentFrame();
  return (
    <div
      style={{
        background: C.panel,
        border: `1px solid ${C.line}`,
        borderLeft: `5px solid ${accent}`,
        borderRadius: 22,
        padding: "28px 34px",
        boxShadow: "0 24px 60px rgba(0,0,0,0.45)",
        ...rise(frame, at, 30, 24),
        ...style,
      }}
    >
      {children}
    </div>
  );
};

/** Carimbo de decisão (FOLD, CHECK-RAISE...). */
export const Stamp: React.FC<{ text: string; at: number; color?: string; size?: number; rotate?: number; style?: React.CSSProperties }> = ({
  text,
  at,
  color = C.danger,
  size = 120,
  rotate = -8,
  style,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = pop(frame, at, fps, 11);
  const visible = ramp(frame, at, 4);
  return (
    <div
      style={{
        fontFamily: UI,
        fontWeight: 800,
        fontSize: size,
        letterSpacing: size * 0.06,
        color,
        border: `${size * 0.07}px solid ${color}`,
        borderRadius: size * 0.16,
        padding: `${size * 0.04}px ${size * 0.24}px`,
        transform: `rotate(${rotate}deg) scale(${2.2 - 1.2 * t})`,
        opacity: visible,
        textShadow: `0 0 ${size * 0.25}px ${color}55`,
        whiteSpace: "nowrap",
        ...style,
      }}
    >
      {text}
    </div>
  );
};
