import { useId, useRef, useState } from "react";
import film from "@/data/pineappleFilm.json";
import { CourseVideoPlayer } from "./CourseVideoPlayer";
import { formatLessonDuration } from "@/lib/courseDuration";

const video = {
  ...film,
  url: `/videos/${film.filename}.mp4`,
  posterUrl: `/videos/${film.filename}.webp`,
  captionsUrl: `/videos/${film.filename}.vtt`,
};

/** One locally produced film, shared by the public home and the Academy. */
export function PineappleFilm({ academy = false }: { academy?: boolean }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const [open, setOpen] = useState(false);
  function showFilm() {
    setOpen(true);
    dialog.current?.showModal();
  }
  return <>
    <button ref={trigger} type="button" onClick={showFilm} aria-haspopup="dialog"
      className={academy ? "group grid w-full overflow-hidden rounded-lg border border-gold-soft/40 bg-felt-900 text-left focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-gold-soft sm:grid-cols-2" : "hp-button"}>
      {academy && <img src={video.posterUrl} width="1280" height="720" alt="" className="h-full w-full object-cover" />}
      <span className={academy ? "flex flex-col justify-center gap-2 p-5" : undefined}>
        <span className={academy ? "text-lg font-bold text-cream" : undefined}>Aprenda Brazilian Pineapple</span>
        {academy && <span className="text-sm leading-relaxed text-felt-200">Das primeiras cartas aos potes paralelos. Regras de apostas, exemplos e capítulos para rever cada conta.</span>}
        {academy && <span className="text-xs text-gold-soft">▶ Assistir · {formatLessonDuration(film.durationSeconds)} · Português e legendas</span>}
      </span>
      {!academy && <span aria-hidden="true">▶</span>}
    </button>
    <dialog ref={dialog} aria-labelledby={titleId}
      className="m-auto max-h-[94dvh] w-[min(1050px,96vw)] overflow-y-auto rounded-lg border border-gold-soft/50 bg-felt-950 p-0 text-cream backdrop:bg-black/90"
      onClose={() => { setOpen(false); trigger.current?.focus(); }}>
      <div className="sticky top-0 z-10 flex items-center justify-between gap-3 border-b border-felt-700 bg-felt-950 px-4 py-2">
        <h2 id={titleId} className="font-sans! text-base! font-bold! tracking-normal!">Aprenda Brazilian Pineapple</h2>
        <button type="button" className="min-h-11 min-w-11 rounded-sm text-xl focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-gold-soft" aria-label="Fechar vídeo" onClick={() => dialog.current?.close()}>×</button>
      </div>
      {open && <CourseVideoPlayer video={video} lessonTitle="Brazilian Pineapple: cartas, apostas e potes" />}
    </dialog>
  </>;
}
