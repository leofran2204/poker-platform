import { useState } from "react";
import { formatLessonDuration, type CourseVideoInfo } from "@/lib/course";

interface Props {
  video?: CourseVideoInfo;
  lessonTitle: string;
}

/** Player da aula: vídeo, legendas, transcrição da narração. Sem persona de coach. */
export function CourseVideoPlayer({ video, lessonTitle }: Props) {
  const [showScript, setShowScript] = useState(false);
  const transcript = video?.transcript?.trim() || video?.placeholderScript?.trim() || "";
  const durationLabel = video?.durationSeconds
    ? formatLessonDuration(video.durationSeconds)
    : null;

  return (
    <div className="zt-panel overflow-hidden border border-gold-soft/30 bg-felt-950/80 shadow-2xl">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-felt-800 bg-felt-900/90 px-4 py-2.5">
        <div>
          <span className="text-xs font-bold text-cream">{lessonTitle}</span>
          <p className="text-[11px] text-felt-400">Aula em vídeo · Zero Tilt Academy</p>
        </div>
        {durationLabel && (
          <span className="flex items-center gap-1 rounded bg-felt-800 px-2 py-1 text-xs font-mono text-gold-bright">
            {durationLabel}
          </span>
        )}
      </div>

      <div className="relative aspect-video w-full min-w-0 max-h-[420px] bg-felt-950">
        {video?.url ? (
          <video
            src={video.url}
            poster={video.posterUrl}
            controls
            playsInline
            className="h-full w-full object-contain"
            crossOrigin="anonymous"
          >
            {video.captionsUrl && (
              <track
                kind="captions"
                srcLang="pt"
                label="Português"
                src={video.captionsUrl}
                default
              />
            )}
            Seu navegador não suporta reprodução de vídeo HTML5.
          </video>
        ) : (
          <div className="flex h-full flex-col items-center justify-center space-y-3 px-4 text-center">
            <h3 className="text-sm font-bold text-gold-bright sm:text-base">
              {lessonTitle} — vídeo em produção
            </h3>
            <p className="max-w-lg text-xs text-felt-300">
              O vídeo desta aula ainda está em produção. Enquanto isso, leia a narração abaixo.
            </p>
          </div>
        )}
      </div>

      {transcript && (
        <div className="border-t border-felt-800 bg-felt-950 px-4 py-3">
          <button
            type="button"
            onClick={() => setShowScript((s) => !s)}
            className="text-xs font-semibold text-gold-soft hover:underline"
          >
            {showScript ? "Ocultar transcrição" : "Transcrição da narração"}
          </button>
          {showScript && (
            <div className="mt-2 max-h-48 overflow-y-auto whitespace-pre-line rounded border border-felt-800 bg-felt-900/60 p-3 text-xs leading-relaxed text-felt-200">
              {transcript}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
