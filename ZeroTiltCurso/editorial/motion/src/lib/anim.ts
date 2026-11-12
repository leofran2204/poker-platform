import { Easing, interpolate, spring } from "remotion";

export const CLAMP = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
export const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);
export const EASE_IN_OUT = Easing.bezier(0.65, 0, 0.35, 1);

/** 0→1 suave entre `start` e `start + length`. */
export const ramp = (frame: number, start: number, length = 18, easing = EASE_OUT) =>
  interpolate(frame, [start, start + length], [0, 1], { ...CLAMP, easing });

/** Mola com leve ultrapassagem, para entradas com peso físico. */
export const pop = (frame: number, start: number, fps: number, damping = 14) =>
  spring({ frame: frame - start, fps, config: { damping, stiffness: 140, mass: 0.7 } });

/** Mola sem ultrapassagem, para movimentos de câmera e deslocamentos longos. */
export const glide = (frame: number, start: number, fps: number, duration = 30) =>
  spring({ frame: frame - start, fps, durationInFrames: duration, config: { damping: 200 } });

export const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

export const brl = (value: number) =>
  `R$ ${value.toFixed(2).replace(".", ",")}`;

/** Entrada padrão de elementos: sobe e aparece. */
export const rise = (frame: number, start: number, distance = 34, length = 22) => {
  const t = ramp(frame, start, length);
  return { opacity: t, transform: `translateY(${(1 - t) * distance}px)` };
};
