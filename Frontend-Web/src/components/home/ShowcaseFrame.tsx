import { useEffect, useRef, useState, type ReactNode } from "react";
import "./live-hand.css";

/**
 * Moldura das mesas de vitrine: a mesa do jogo é desenhada numa largura fixa e
 * reduzida por escala para caber no espaço disponível. Em tela estreita a mesa
 * usa o formato de celular do próprio jogo, sem escala.
 */
export function ShowcaseFrame({
  designWidth,
  designHeight,
  children,
}: {
  designWidth: number;
  designHeight: number;
  children: ReactNode;
}) {
  const frame = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState<number | null>(0.6);
  useEffect(() => {
    const node = frame.current;
    if (!node || typeof ResizeObserver === "undefined") return;
    // O formato de celular da mesa depende da largura da tela (mesmo critério do CSS do jogo).
    const fit = () => setScale(window.innerWidth < 640 ? null : Math.min(1, node.clientWidth / designWidth));
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(node);
    window.addEventListener("resize", fit);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", fit);
    };
  }, [designWidth]);
  return (
    <div
      ref={frame}
      className="hp-live-frame"
      style={scale === null ? undefined : { height: designHeight * scale }}
    >
      <div
        className={scale === null ? "hp-live-stage native" : "hp-live-stage"}
        style={scale === null ? undefined : { width: designWidth, transform: `scale(${scale})` }}
      >
        {children}
      </div>
    </div>
  );
}
