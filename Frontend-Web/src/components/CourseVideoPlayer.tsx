import { useRef, useState } from "react";
import { formatLessonDuration, type CourseVideoInfo } from "@/lib/course";

interface Props {
  video?: CourseVideoInfo;
  lessonTitle: string;
}

/** Player da aula: vídeo, legenda em pt-BR e transcrição completa da narração. */
export function CourseVideoPlayer({ video, lessonTitle }: Props) {
  const [showScript, setShowScript] = useState(false);
  const player = useRef<HTMLVideoElement>(null);
  const transcript = video?.transcript?.trim() || video?.placeholderScript?.trim() || "";
  const durationLabel = video?.durationSeconds
    ? formatLessonDuration(video.durationSeconds)
    : null;
  const captionsUrl = video?.captionsUrl ?? video?.url?.replace(/\.mp4(?=$|\?)/i, ".vtt");

  if (!video?.url || video.publicationStatus === "review") {
    return <p className="rounded border border-felt-700 bg-felt-900 px-4 py-3 text-xs text-felt-300">
      {video?.publicationStatus === "review" ? "Vídeo em revisão editorial. " : "Vídeo em produção. "}
      Estude pelo material escrito e pratique com os exercícios abaixo.
    </p>;
  }

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
            key={video.url}
            ref={player}
            src={video.url}
            poster={video.posterUrl}
            preload="metadata"
            controls
            playsInline
            className="h-full w-full object-contain"
          >
            {captionsUrl && (
              <track
                kind="captions"
                src={captionsUrl}
                srcLang="pt-BR"
                label="Português"
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

      {video?.chapters && video.url && (
        <nav aria-label="Capítulos do vídeo" className="flex flex-wrap gap-2 border-t border-felt-800 px-4 py-3">
          {video.chapters.map(chapter => (
            <button key={chapter.start} type="button" className="rounded border border-felt-700 px-2 py-1 text-left text-xs text-gold-soft hover:bg-felt-800" onClick={() => {
              if (!player.current) return;
              player.current.currentTime = chapter.start;
              void player.current.play().catch(() => { /* Os controles nativos continuam disponíveis. */ });
            }}>
              {Math.floor(chapter.start / 60)}:{String(Math.floor(chapter.start % 60)).padStart(2, "0")} · {chapter.title}
            </button>
          ))}
        </nav>
      )}

      {transcript && (
        <div className="border-t border-felt-800 bg-felt-950 px-4 py-3">
          <button
            type="button"
            aria-expanded={showScript}
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
