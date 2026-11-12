import React from "react";
import { AbsoluteFill, Audio, Sequence, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import { ramp } from "../lib/anim";
import { C, DISPLAY, UI } from "../theme";

/** Espada da marca em vetor (marfim com contorno dourado). */
export const SpadeMark: React.FC<{ size?: number; draw?: number; withText?: boolean }> = ({ size = 220, draw = 1, withText = false }) => {
  const length = 760;
  return (
    <div style={{ position: "relative", width: size, height: size * 1.05 }}>
      <svg width={size} height={size * 1.05} viewBox="0 0 200 210" style={{ display: "block", overflow: "visible" }}>
        <defs>
          <linearGradient id="spadeGold" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stopColor={C.goldSoft} />
            <stop offset="1" stopColor={C.goldDim} />
          </linearGradient>
        </defs>
        <path
          d="M100 8 C 124 44, 190 76, 190 122 C 190 150, 170 170, 144 170 C 126 170, 114 162, 107 150 C 108 170, 116 184, 134 196 L 66 196 C 84 184, 92 170, 93 150 C 86 162, 74 170, 56 170 C 30 170, 10 150, 10 122 C 10 76, 76 44, 100 8 Z"
          fill={C.ivory}
          fillOpacity={draw}
          stroke="url(#spadeGold)"
          strokeWidth={5}
          strokeDasharray={length}
          strokeDashoffset={length * (1 - Math.min(1, draw * 1.4))}
        />
      </svg>
      {withText && (
        <div
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            top: size * 0.4,
            textAlign: "center",
            fontFamily: DISPLAY,
            fontWeight: 700,
            color: C.gold,
            fontSize: size * 0.15,
            lineHeight: 1.05,
            opacity: draw,
          }}
        >
          ZeroTilt
          <div style={{ fontSize: size * 0.12 }}>Poker</div>
        </div>
      )}
    </div>
  );
};

/** Fundo: gradiente profundo, luz lenta e vinheta. */
export const Background: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame / fps;
  const x = 50 + 18 * Math.sin(t * 0.13);
  const y = 40 + 10 * Math.cos(t * 0.11);
  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ background: `radial-gradient(ellipse at 50% 45%, ${C.bg1} 0%, ${C.bg0} 75%)` }} />
      <AbsoluteFill
        style={{
          background: `radial-gradient(circle at ${x}% ${y}%, rgba(111,208,174,0.10) 0%, rgba(111,208,174,0) 38%)`,
        }}
      />
      <AbsoluteFill style={{ background: "radial-gradient(ellipse at 50% 50%, rgba(0,0,0,0) 55%, rgba(0,0,0,0.55) 100%)" }} />
    </AbsoluteFill>
  );
};

/** Moldura persistente: marca, capítulo e barra de progresso. */
export const Chrome: React.FC<{ chapter: string; chapterStart: number; label: string; progress: number }> = ({
  chapter,
  chapterStart,
  label,
  progress,
}) => {
  const frame = useCurrentFrame();
  const t = ramp(frame, chapterStart, 24);
  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      <div style={{ position: "absolute", left: 72, top: 52, display: "flex", alignItems: "center", gap: 14 }}>
        <SpadeMark size={34} />
        <div style={{ fontFamily: UI, fontWeight: 800, letterSpacing: 5, fontSize: 20, color: C.goldSoft }}>{label}</div>
      </div>
      <div
        style={{
          position: "absolute",
          left: 72,
          bottom: 50,
          fontFamily: UI,
          fontWeight: 700,
          fontSize: 20,
          letterSpacing: 3,
          color: C.faint,
          textTransform: "uppercase",
          opacity: t,
          transform: `translateX(${(1 - t) * -20}px)`,
        }}
      >
        {chapter}
      </div>
      <div style={{ position: "absolute", left: 0, right: 0, bottom: 0, height: 5, background: "rgba(255,255,255,0.06)" }}>
        <div style={{ width: `${progress * 100}%`, height: "100%", background: `linear-gradient(90deg, ${C.goldDim}, ${C.gold})` }} />
      </div>
    </AbsoluteFill>
  );
};

/** Efeito sonoro posicionado num quadro local da cena. */
export const Sfx: React.FC<{ at: number; name: "deal" | "flip" | "chips" | "chip" | "whoosh" | "pop" | "stamp"; volume?: number }> = ({
  at,
  name,
  volume = 0.45,
}) => (
  <Sequence from={Math.max(0, Math.round(at))} durationInFrames={90} layout="none">
    <Audio src={staticFile(`sfx/${name}.wav`)} volume={volume} />
  </Sequence>
);
