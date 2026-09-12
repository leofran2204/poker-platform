import { useState } from "react";
import type { CourseHandExample, CourseVideoInfo } from "@/lib/course";
import { PlayingCard } from "./PlayingCard";

interface Props {
  video?: CourseVideoInfo;
  handExample?: CourseHandExample;
  lessonTitle: string;
}

export function CourseVideoPlayer({ video, handExample, lessonTitle }: Props) {
  const [viewMode, setViewMode] = useState<"video" | "hand">("video");
  const [showScript, setShowScript] = useState(false);
  const hostName = video?.hostName ?? "Vitor 'Zero Tilt' Brandão";
  const hostRole = video?.hostRole ?? "Coach Virtual IA";
  const duration = video?.durationSeconds ? `${Math.floor(video.durationSeconds / 60)}:${String(video.durationSeconds % 60).padStart(2, "0")}` : "02:00";

  return (
    <div className="zt-panel overflow-hidden border border-gold-soft/30 bg-felt-950/80 shadow-2xl">
      {/* Header do Player com Avatar e Duração */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-felt-800 bg-felt-900/90 px-4 py-2.5">
        <div className="flex items-center gap-3">
          <div className="relative flex h-10 w-10 items-center justify-center rounded-full border border-gold-soft/60 bg-gradient-to-br from-gold-soft/20 via-felt-900 to-black text-xs font-bold text-gold-bright shadow">
            {video?.hostAvatarUrl ? (
              <img
                src={video.hostAvatarUrl}
                alt={hostName}
                className="h-full w-full rounded-full object-cover"
              />
            ) : (
              <span>IA</span>
            )}
            <span className="absolute -bottom-0.5 -right-0.5 flex h-3 w-3">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
              <span className="relative inline-flex h-3 w-3 rounded-full bg-emerald-500" />
            </span>
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-cream">{hostName}</span>
              <span className="rounded bg-gold-bright/10 px-1.5 py-0.5 text-[10px] font-semibold text-gold-soft border border-gold-soft/20">
                {hostRole}
              </span>
            </div>
            <p className="text-[11px] text-felt-400">Vídeo Explicativo • Análise de Mãos</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <span className="flex items-center gap-1 rounded bg-felt-800 px-2 py-1 text-xs font-mono text-gold-bright">
            ⏱️ {duration} min
          </span>
          <div className="flex rounded border border-felt-700 bg-felt-950/90 p-0.5">
            <button
              type="button"
              onClick={() => setViewMode("video")}
              className={`rounded px-2.5 py-1 text-xs font-semibold transition-colors ${
                viewMode === "video" ? "bg-gold-bright text-felt-950 shadow" : "text-felt-300 hover:text-white"
              }`}
            >
              🎬 Vídeo IA
            </button>
            <button
              type="button"
              onClick={() => setViewMode("hand")}
              className={`rounded px-2.5 py-1 text-xs font-semibold transition-colors ${
                viewMode === "hand" ? "bg-gold-bright text-felt-950 shadow" : "text-felt-300 hover:text-white"
              }`}
            >
              🃏 Mão na Mesa
            </button>
          </div>
        </div>
      </div>

      {/* Corpo do Player */}
      <div className="relative aspect-video w-full max-h-[420px] bg-gradient-to-b from-felt-950 via-felt-900 to-black flex items-center justify-center p-4">
        {viewMode === "video" && video?.url ? (
          <video
            src={video.url}
            controls
            className="h-full w-full object-contain"
            poster={video.hostAvatarUrl}
          >
            Seu navegador não suporta reprodução de vídeo HTML5.
          </video>
        ) : viewMode === "video" ? (
          /* Placeholder de Vídeo Interativo com visual da Mesa e IA */
          <div className="flex flex-col items-center justify-center text-center space-y-3 px-4 max-w-lg">
            <div className="flex h-16 w-16 items-center justify-center rounded-full border-2 border-gold-bright/60 bg-felt-900 shadow-xl">
              <span className="text-2xl">▶️</span>
            </div>
            <h3 className="text-sm sm:text-base font-bold text-gold-bright">
              {lessonTitle} — Aula em Vídeo de 2 Minutos
            </h3>
            <p className="text-xs text-felt-300">
              Apresentado pelo jogador virtual <strong className="text-cream">{hostName}</strong> com
              demonstração gráfica de mãos e explicação de ranges.
            </p>
            {video?.placeholderScript && (
              <button
                type="button"
                onClick={() => setShowScript((s) => !s)}
                className="text-xs text-gold-soft hover:underline flex items-center gap-1"
              >
                {showScript ? "Ocultar roteiro da narração" : "📜 Ler roteiro da narração do Coach IA"}
              </button>
            )}
            {handExample && (
              <button
                type="button"
                onClick={() => setViewMode("hand")}
                className="zt-btn-secondary !text-xs !py-1 !px-3"
              >
                Ver demonstração da mão na mesa →
              </button>
            )}
          </div>
        ) : (
          /* Visualização da Mão na Mesa */
          <div className="w-full max-w-xl rounded-xl border border-felt-700/80 bg-felt-900/90 p-4 sm:p-5 shadow-2xl space-y-4">
            <div className="flex items-center justify-between border-b border-felt-800 pb-2">
              <div>
                <span className="text-[11px] font-mono uppercase tracking-wider text-gold-soft">
                  {handExample?.title ?? "Exemplo Prático na Mesa"}
                </span>
                <h4 className="text-sm font-bold text-cream">
                  Posição: <span className="text-gold-bright">{handExample?.heroPosition}</span> vs{" "}
                  <span className="text-felt-300">{handExample?.villainPosition}</span>
                </h4>
              </div>
              {handExample?.potBb && (
                <div className="rounded bg-felt-950 px-2 py-1 text-right">
                  <span className="block text-[10px] text-felt-400">Pote Atual</span>
                  <span className="text-xs font-mono font-bold text-gold-bright">{handExample.potBb} BB</span>
                </div>
              )}
            </div>

            {/* Mesa com Cartas do Jogador e Comunitárias */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 items-center bg-felt-950/60 p-3 rounded-lg border border-felt-800">
              <div className="space-y-1 text-center sm:text-left">
                <span className="text-[11px] text-felt-300 font-medium">Sua Mão (Hero)</span>
                <div className="flex justify-center sm:justify-start gap-1.5">
                  {handExample?.heroCards.map((c) => (
                    <PlayingCard key={c} code={c} size="sm" highlight />
                  ))}
                </div>
              </div>

              <div className="space-y-1 text-center sm:text-left">
                <span className="text-[11px] text-felt-300 font-medium">Cartas na Mesa (Board)</span>
                <div className="flex justify-center sm:justify-start gap-1.5 min-h-[52px] items-center">
                  {handExample?.boardCards && handExample.boardCards.length > 0 ? (
                    handExample.boardCards.map((c) => (
                      <PlayingCard key={c} code={c} size="sm" />
                    ))
                  ) : (
                    <span className="text-xs italic text-felt-500">Rodada Pré-flop (sem board)</span>
                  )}
                </div>
              </div>
            </div>

            {/* Dinâmica da Ação e Decisão */}
            <div className="space-y-2 text-xs">
              <p className="text-felt-200">
                <strong className="text-gold-soft">Ação da Mesa:</strong> {handExample?.actionText}
              </p>
              <div className="rounded border-l-2 border-gold-bright bg-felt-950/80 p-2 text-emerald-300">
                <strong>Decisão Correta:</strong> {handExample?.heroDecision}
              </div>
              <p className="text-[11px] text-felt-400 italic">
                💡 <strong>Dica Zero Tilt:</strong> {handExample?.keyTakeaway}
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Roteiro Expandido da Narração */}
      {showScript && video?.placeholderScript && (
        <div className="border-t border-felt-800 bg-felt-950 p-4 text-xs text-felt-200 space-y-2">
          <div className="flex items-center justify-between">
            <span className="font-bold text-gold-soft">🎙️ Roteiro do Vídeo (Narração de 2 Minutos):</span>
            <button
              type="button"
              onClick={() => setShowScript(false)}
              className="text-felt-400 hover:text-white text-[11px]"
            >
              Fechar ✕
            </button>
          </div>
          <div className="max-h-48 overflow-y-auto whitespace-pre-line rounded border border-felt-800 bg-felt-900/60 p-3 font-sans leading-relaxed">
            {video.placeholderScript}
          </div>
        </div>
      )}
    </div>
  );
}
