import React from "react";
import { C, UI } from "../theme";

export type Suit = "h" | "d" | "c" | "s";

const RED: Record<Suit, boolean> = { h: true, d: true, c: false, s: false };

/** Naipes em vetor (viewBox 0 0 100 100) para nitidez em qualquer tamanho. */
export const SuitIcon: React.FC<{ suit: Suit; size: number; color?: string }> = ({ suit, size, color }) => {
  const fill = color ?? (RED[suit] ? C.red : C.ink);
  return (
    <svg width={size} height={size} viewBox="0 0 100 100" style={{ display: "block" }}>
      {suit === "h" && (
        <path
          fill={fill}
          d="M50 90 C 22 68, 5 50, 5 31 C 5 15, 17 5, 31 5 C 40 5, 47 10, 50 19 C 53 10, 60 5, 69 5 C 83 5, 95 15, 95 31 C 95 50, 78 68, 50 90 Z"
        />
      )}
      {suit === "d" && <path fill={fill} d="M50 3 C 62 20, 76 36, 90 50 C 76 64, 62 80, 50 97 C 38 80, 24 64, 10 50 C 24 36, 38 20, 50 3 Z" />}
      {suit === "s" && (
        <path
          fill={fill}
          d="M50 4 C 62 22, 94 38, 94 61 C 94 75, 84 85, 71 85 C 62 85, 56 81, 53 75 C 54 85, 58 91, 66 96 L 34 96 C 42 91, 46 85, 47 75 C 44 81, 38 85, 29 85 C 16 85, 6 75, 6 61 C 6 38, 38 22, 50 4 Z"
        />
      )}
      {suit === "c" && (
        <g fill={fill}>
          <circle cx="50" cy="27" r="19" />
          <circle cx="27" cy="57" r="19" />
          <circle cx="73" cy="57" r="19" />
          <circle cx="50" cy="52" r="12" />
          <path d="M46 58 C 46 78, 40 89, 32 96 L 68 96 C 60 89, 54 78, 54 58 Z" />
        </g>
      )}
    </svg>
  );
};

export const parseCard = (code: string) => {
  const rank = code.slice(0, -1);
  const suit = code.slice(-1) as Suit;
  return { rank: rank === "T" ? "10" : rank, suit };
};

type CardProps = {
  code?: string;
  /** 0 = verso, 1 = face (anima a virada em 3D). */
  face?: number;
  width?: number;
  glow?: number;
  dim?: number;
  style?: React.CSSProperties;
};

/** Carta de baralho em vetor, com verso da marca e virada em perspectiva. */
export const Card: React.FC<CardProps> = ({ code, face = 1, width = 140, glow = 0, dim = 0, style }) => {
  const height = width * 1.42;
  const radius = width * 0.085;
  const rotation = (1 - face) * 180;
  const shadow = `0 ${width * 0.06}px ${width * 0.18}px rgba(0,0,0,0.45)`;
  const halo = glow > 0 ? `, 0 0 ${width * 0.28 * glow}px ${width * 0.06 * glow}px rgba(236,200,95,${0.75 * glow})` : "";
  const card = code ? parseCard(code) : null;
  const color = card && RED[card.suit] ? C.red : C.ink;
  const faceStyle: React.CSSProperties = {
    position: "absolute",
    inset: 0,
    borderRadius: radius,
    backfaceVisibility: "hidden",
    WebkitBackfaceVisibility: "hidden",
    overflow: "hidden",
  };
  return (
    <div style={{ width, height, perspective: width * 9, ...style }}>
      <div
        style={{
          position: "relative",
          width: "100%",
          height: "100%",
          transformStyle: "preserve-3d",
          transform: `rotateY(${rotation}deg)`,
          borderRadius: radius,
          boxShadow: shadow + halo,
          filter: dim > 0 ? `brightness(${1 - 0.55 * dim}) saturate(${1 - 0.5 * dim})` : undefined,
        }}
      >
        {/* Face */}
        <div
          style={{
            ...faceStyle,
            background: `linear-gradient(160deg, #ffffff 0%, ${C.paper} 55%, #efe7d4 100%)`,
            border: `1px solid rgba(0,0,0,0.12)`,
          }}
        >
          {card && (
            <>
              <div
                style={{
                  position: "absolute",
                  left: width * 0.09,
                  top: width * 0.06,
                  color,
                  fontFamily: UI,
                  fontWeight: 800,
                  fontSize: width * 0.3,
                  lineHeight: 1,
                  letterSpacing: card.rank === "10" ? -width * 0.02 : 0,
                }}
              >
                {card.rank}
                <div style={{ marginTop: width * 0.03 }}>
                  <SuitIcon suit={card.suit} size={width * 0.2} />
                </div>
              </div>
              <div style={{ position: "absolute", right: width * 0.1, bottom: width * 0.1 }}>
                <SuitIcon suit={card.suit} size={width * 0.5} />
              </div>
            </>
          )}
        </div>
        {/* Verso */}
        <div
          style={{
            ...faceStyle,
            transform: "rotateY(180deg)",
            background: `radial-gradient(circle at 50% 40%, #1d6b41 0%, #0f3d25 70%, #0a2a19 100%)`,
            border: `${Math.max(2, width * 0.035)}px solid ${C.paper}`,
          }}
        >
          <div
            style={{
              position: "absolute",
              inset: width * 0.06,
              borderRadius: radius * 0.6,
              border: `1px solid rgba(236,217,154,0.55)`,
              backgroundImage:
                "repeating-linear-gradient(45deg, rgba(236,217,154,0.16) 0 2px, transparent 2px 14px), repeating-linear-gradient(-45deg, rgba(236,217,154,0.16) 0 2px, transparent 2px 14px)",
            }}
          />
          <div style={{ position: "absolute", left: "50%", top: "50%", transform: "translate(-50%, -50%)" }}>
            <SuitIcon suit="s" size={width * 0.34} color={C.goldSoft} />
          </div>
        </div>
      </div>
    </div>
  );
};
