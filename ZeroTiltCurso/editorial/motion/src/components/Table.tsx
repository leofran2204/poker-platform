import React from "react";
import { C, UI } from "../theme";
import { SuitIcon } from "./Card";

/** Mesa oval com borda de couro, feltro e linha dourada interna. */
export const Table: React.FC<{ x: number; y: number; width: number; height: number; opacity?: number }> = ({
  x,
  y,
  width,
  height,
  opacity = 1,
}) => (
  <div
    style={{
      position: "absolute",
      left: x - width / 2,
      top: y - height / 2,
      width,
      height,
      borderRadius: height / 2,
      opacity,
      background: `linear-gradient(180deg, ${C.rail0} 0%, ${C.rail1} 100%)`,
      boxShadow: "0 40px 90px rgba(0,0,0,0.65), inset 0 2px 0 rgba(255,255,255,0.08)",
    }}
  >
    <div
      style={{
        position: "absolute",
        inset: 26,
        borderRadius: height / 2,
        background: `radial-gradient(ellipse at 50% 42%, ${C.felt0} 0%, ${C.felt1} 55%, ${C.felt2} 100%)`,
        boxShadow: "inset 0 0 70px rgba(0,0,0,0.55), inset 0 0 0 2px rgba(0,0,0,0.35)",
      }}
    />
    <div
      style={{
        position: "absolute",
        inset: 70,
        borderRadius: height / 2,
        border: `2px solid rgba(236,217,154,0.22)`,
      }}
    />
    <div
      style={{
        position: "absolute",
        left: "50%",
        top: "50%",
        transform: "translate(-50%, -50%)",
        opacity: 0.08,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 6,
      }}
    >
      <SuitIcon suit="s" size={130} color={C.goldSoft} />
      <div style={{ fontFamily: UI, fontWeight: 800, letterSpacing: 8, color: C.goldSoft, fontSize: 22 }}>
        ZERO TILT
      </div>
    </div>
  </div>
);

type SeatProps = {
  x: number;
  y: number;
  name: string;
  stack?: string;
  active?: number;
  hero?: boolean;
  folded?: number;
  badge?: string;
  badgeColor?: string;
  badgeOpacity?: number;
  dealer?: boolean;
  opacity?: number;
  scale?: number;
};

/** Assento: avatar, nome e stack, com realce de vez e selo de ação. */
export const Seat: React.FC<SeatProps> = ({
  x,
  y,
  name,
  stack,
  active = 0,
  hero = false,
  folded = 0,
  badge,
  badgeColor = C.gold,
  badgeOpacity = 1,
  dealer = false,
  opacity = 1,
  scale = 1,
}) => (
  <div
    style={{
      position: "absolute",
      left: x,
      top: y,
      transform: `translate(-50%, -50%) scale(${scale})`,
      opacity: opacity * (1 - 0.55 * folded),
      display: "flex",
      alignItems: "center",
      gap: 14,
      padding: "12px 22px 12px 12px",
      borderRadius: 999,
      background: "linear-gradient(180deg, rgba(14,30,21,0.96), rgba(6,15,10,0.96))",
      border: `2px solid ${active > 0 ? `rgba(236,200,95,${0.4 + 0.6 * active})` : "rgba(255,255,255,0.08)"}`,
      boxShadow: `0 14px 30px rgba(0,0,0,0.5)${active > 0 ? `, 0 0 ${28 * active}px rgba(236,200,95,${0.45 * active})` : ""}`,
      filter: folded > 0 ? `grayscale(${folded})` : undefined,
      whiteSpace: "nowrap",
    }}
  >
    <div
      style={{
        width: 52,
        height: 52,
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
    <div style={{ fontFamily: UI, lineHeight: 1.15 }}>
      <div style={{ color: C.ivory, fontWeight: 700, fontSize: 24 }}>{name}</div>
      {stack && <div style={{ color: C.muted, fontWeight: 600, fontSize: 19 }}>{stack}</div>}
    </div>
    {dealer && (
      <div
        style={{
          position: "absolute",
          right: -18,
          bottom: -14,
          width: 38,
          height: 38,
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
    {badge && (
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: -26,
          transform: `translateX(-50%) scale(${0.8 + 0.2 * badgeOpacity})`,
          opacity: badgeOpacity,
          padding: "6px 16px",
          borderRadius: 999,
          background: badgeColor,
          color: C.ink,
          fontFamily: UI,
          fontWeight: 800,
          fontSize: 18,
          letterSpacing: 1.5,
          boxShadow: "0 8px 18px rgba(0,0,0,0.4)",
        }}
      >
        {badge}
      </div>
    )}
  </div>
);
