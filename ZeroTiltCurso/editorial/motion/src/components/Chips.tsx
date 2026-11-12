import React from "react";
import { C, UI } from "../theme";

const PALETTE: Record<string, [string, string]> = {
  gold: ["#e3b94f", "#8b6914"],
  red: ["#d8443a", "#7d1a14"],
  blue: ["#3d7fd6", "#1b3f73"],
  black: ["#2a2a2a", "#0b0b0b"],
  green: ["#3fae6e", "#185a35"],
};

/** Ficha vista de cima, com faixas na borda. */
export const Chip: React.FC<{ color?: keyof typeof PALETTE; size?: number }> = ({ color = "red", size = 60 }) => {
  const [main, dark] = PALETTE[color];
  return (
    <svg width={size} height={size} viewBox="0 0 100 100" style={{ display: "block", overflow: "visible" }}>
      <circle cx="50" cy="54" r="46" fill="rgba(0,0,0,0.35)" />
      <circle cx="50" cy="50" r="46" fill={main} stroke={dark} strokeWidth="3" />
      <circle cx="50" cy="50" r="40" fill="none" stroke="#fdf6e3" strokeWidth="9" strokeDasharray="13 13.4" />
      <circle cx="50" cy="50" r="27" fill={dark} />
      <circle cx="50" cy="50" r="27" fill="none" stroke="rgba(253,246,227,0.6)" strokeWidth="2" />
    </svg>
  );
};

/** Pilha de fichas com rótulo de valor opcional. */
export const ChipStack: React.FC<{
  count?: number;
  color?: keyof typeof PALETTE;
  size?: number;
  label?: string;
  labelOpacity?: number;
}> = ({ count = 3, color = "red", size = 60, label, labelOpacity = 1 }) => (
  <div style={{ position: "relative", width: size, height: size + (count - 1) * size * 0.12 }}>
    {Array.from({ length: count }).map((_, i) => (
      <div key={i} style={{ position: "absolute", left: 0, bottom: i * size * 0.12 }}>
        <Chip color={color} size={size} />
      </div>
    ))}
    {label && (
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: size + (count - 1) * size * 0.12 + 8,
          transform: "translateX(-50%)",
          opacity: labelOpacity,
          padding: "4px 12px",
          borderRadius: 999,
          background: "rgba(5,14,9,0.85)",
          border: `1px solid ${C.line}`,
          color: C.goldSoft,
          fontFamily: UI,
          fontWeight: 800,
          fontSize: 20,
          whiteSpace: "nowrap",
        }}
      >
        {label}
      </div>
    )}
  </div>
);

/** Cápsula do pote. */
export const Pot: React.FC<{ value: string; label?: string; scale?: number; opacity?: number }> = ({
  value,
  label = "POTE",
  scale = 1,
  opacity = 1,
}) => (
  <div
    style={{
      display: "inline-flex",
      alignItems: "baseline",
      gap: 14,
      padding: "12px 26px",
      borderRadius: 999,
      background: "rgba(4,12,8,0.82)",
      border: `1px solid rgba(236,217,154,0.35)`,
      boxShadow: "0 12px 26px rgba(0,0,0,0.45)",
      transform: `scale(${scale})`,
      opacity,
      fontFamily: UI,
      whiteSpace: "nowrap",
    }}
  >
    <span style={{ color: C.muted, fontWeight: 700, fontSize: 20, letterSpacing: 3 }}>{label}</span>
    <span style={{ color: C.goldSoft, fontWeight: 800, fontSize: 38, fontVariantNumeric: "tabular-nums" }}>{value}</span>
  </div>
);
