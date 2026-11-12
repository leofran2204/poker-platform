import { glide, lerp, ramp } from "./anim";

export type Point = { x: number; y: number };

/** Voo de carta ou ficha do ponto de origem ao destino, com arco e giro. */
export function flight(frame: number, at: number, fps: number, from: Point, to: Point, spin = -18) {
  const t = glide(frame, at, fps, 22);
  const arc = Math.sin(Math.PI * t) * -60;
  return {
    x: lerp(from.x, to.x, t),
    y: lerp(from.y, to.y, t) + arc,
    rotate: lerp(spin, 0, t),
    opacity: ramp(frame, at, 4),
    landed: t > 0.98,
  };
}

/** Desloca um ponto em direção a outro (para cartas e fichas ficarem à frente do assento). */
export const toward = (from: Point, to: Point, amount: number): Point => ({
  x: lerp(from.x, to.x, amount),
  y: lerp(from.y, to.y, amount),
});
