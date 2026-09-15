import { useState } from "react";
import type { CourseHandExample, CourseVideoInfo } from "@/lib/course";

interface Props {
  video?: CourseVideoInfo;
  handExample?: CourseHandExample;
  lessonTitle: string;
}

/** Player neutro da aula: vídeo quando há url, ou placeholder honesto + roteiro. */
export function CourseVideoPlayer({ video, lessonTitle }: Props) {
  const [showScript, setShowScript] = useState(false);
  const duration = video?.durationSeconds ? `${Math.floor(video.durationSeconds / 60)}:${String(video.durationSeconds % 60).padStart(2, "0")}` : "02:00";

  return (
    <div className="zt-panel overflow-hidden border border-gold-soft/30 bg-felt-950/80 shadow-2xl">
      {/* Header do Player */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-felt-800 bg-felt-900/90 px-4 py-2.5">
        <div>
          <span className="text-xs font-bold text-cream">{lessonTitle}</span>
          <p className="text-[11px] text-felt-400">Aula em vídeo · Zero Tilt Academy</p>
        </div>

        <span className="flex items-center gap-1 rounded bg-felt-800 px-2 py-1 text-xs font-mono text-gold-bright">
          ⏱️ {duration} min
        </span>
      </div>

      {/* Corpo do Player */}
      <div className="relative aspect-video w-full max-h-[420px] bg-gradient-to-b from-felt-950 via-felt-900 to-black flex items-center justify-center p-4">
        {video?.url ? (
          <video
            src={video.url}
            controls
            className="h-full w-full object-contain"
          >
            Seu navegador não suporta reprodução de vídeo HTML5.
          </video>
        ) : (
          /* Placeholder honesto: sem vídeo produzido para esta aula ainda */
          <div className="flex flex-col items-center justify-center text-center space-y-3 px-4 max-w-lg">
            <div className="flex h-16 w-16 items-center justify-center rounded-full border-2 border-gold-bright/60 bg-felt-900 shadow-xl">
              <span className="text-2xl">🎬</span>
            </div>
            <h3 className="text-sm sm:text-base font-bold text-gold-bright">
              {lessonTitle} — vídeo em produção
            </h3>
            <p className="text-xs text-felt-300">
              O vídeo desta aula ainda está em produção. Enquanto isso, leia o roteiro
              da narração abaixo.
            </p>
            {video?.placeholderScript && (
              <button
                type="button"
                onClick={() => setShowScript((s) => !s)}
                className="text-xs text-gold-soft hover:underline flex items-center gap-1"
              >
                {showScript ? "Ocultar roteiro da narração" : "📜 Ler roteiro da narração"}
              </button>
            )}
          </div>
        )}
      </div>

      {/* Roteiro Expandido da Narração */}
      {showScript && video?.placeholderScript && (
        <div className="border-t border-felt-800 bg-felt-950 p-4 text-xs text-felt-200 space-y-2">
          <div className="flex items-center justify-between">
            <span className="font-bold text-gold-soft">🎙️ Roteiro da narração:</span>
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
