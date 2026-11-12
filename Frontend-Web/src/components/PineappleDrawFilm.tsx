import { useId, useRef, useState } from "react";
import films from "@/data/pineappleDrawFilms.json";
import type { CourseVideoInfo } from "@/lib/course";
import { formatLessonDuration } from "@/lib/courseDuration";
import { CourseVideoPlayer } from "./CourseVideoPlayer";

type FilmId = keyof typeof films;

/** Filmes do Brazilian Pineapple produzidos em Remotion (`motion/produce.py --publish`). */
const FILMS: Record<FilmId, { label: string; heading: string; video: CourseVideoInfo }> = {
  "pineapple-draw-trailer": {
    label: "Trailer",
    heading: films["pineapple-draw-trailer"].title,
    video: media(films["pineapple-draw-trailer"]),
  },
  "pineapple-draw-rules": {
    label: "Regras completas",
    heading: films["pineapple-draw-rules"].title,
    video: media(films["pineapple-draw-rules"]),
  },
};
const ORDER: FilmId[] = ["pineapple-draw-trailer", "pineapple-draw-rules"];

function media(film: (typeof films)[FilmId]): CourseVideoInfo {
  return {
    publicationStatus: film.publicationStatus as CourseVideoInfo["publicationStatus"],
    renderer: "remotion",
    rendererVersion: film.rendererVersion,
    pronunciationVersion: film.pronunciationVersion,
    narrationHash: film.narrationHash,
    mediaRevision: film.mediaRevision,
    bettingRuleVersion: film.bettingRuleVersion,
    durationSeconds: film.durationSeconds,
    transcript: film.transcript,
    chapters: film.chapters,
    url: `/videos/${film.filename}.mp4`,
    captionsUrl: `/videos/${film.filename}.vtt`,
    posterUrl: `/videos/${film.filename}.webp`,
  };
}

/** Duração compacta para as abas (cabe numa linha no celular): 9:47. */
function clock(seconds: number): string {
  const total = Math.round(seconds);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

const rules = FILMS["pineapple-draw-rules"].video;
const trailer = FILMS["pineapple-draw-trailer"].video;

/** Home: abre no trailer. Academy: cartão do filme completo de regras. */
export function PineappleDrawFilm({ academy = false }: { academy?: boolean }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const [current, setCurrent] = useState<FilmId | null>(null);
  const film = current ? FILMS[current] : null;

  function show() {
    setCurrent(academy ? "pineapple-draw-rules" : "pineapple-draw-trailer");
    dialog.current?.showModal();
  }

  return <>
    {academy ? (
      <button ref={trigger} type="button" onClick={show} aria-haspopup="dialog"
        className="group grid w-full overflow-hidden rounded-lg border border-gold-soft/40 bg-felt-900 text-left focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-gold-soft sm:grid-cols-2">
        <img src={`${rules.posterUrl}?r=${rules.mediaRevision}`} width="1280" height="720" alt="" className="h-full w-full object-cover" />
        <span className="flex flex-col justify-center gap-2 p-5">
          <span className="text-xs font-bold uppercase tracking-wider text-gold-soft">Nova modalidade · Brazilian Pineapple</span>
          <span className="text-lg font-bold text-cream">Três cartas. Uma troca por street.</span>
          <span className="text-sm leading-relaxed text-felt-200">A ordem da mão, as trocas, o descarte do river, all-in, limite do pote e Loss Deflator, com mãos conferidas no motor do jogo e capítulos para rever cada conta.</span>
          <span className="text-xs text-gold-soft">▶ Assistir · {formatLessonDuration(rules.durationSeconds!)} · português com legendas</span>
        </span>
      </button>
    ) : (
      <button ref={trigger} type="button" onClick={show} aria-haspopup="dialog" className="hp-film-button">
        <span className="hp-play" aria-hidden="true">▶</span>
        <span>Assistir ao trailer<small>{formatLessonDuration(trailer.durationSeconds!)} · regras completas em {formatLessonDuration(rules.durationSeconds!)}</small></span>
      </button>
    )}
    <dialog ref={dialog} aria-labelledby={titleId}
      className="m-auto max-h-[94dvh] w-[min(1050px,96vw)] overflow-y-auto rounded-lg border border-gold-soft/50 bg-felt-950 p-0 text-cream backdrop:bg-black/90"
      onClose={() => { setCurrent(null); trigger.current?.focus(); }}>
      <div className="sticky top-0 z-10 flex flex-wrap items-center justify-between gap-3 border-b border-felt-700 bg-felt-950 px-4 py-2">
        {/* O título do vídeo atual já aparece no player; aqui só a modalidade. */}
        <h2 id={titleId} className="font-sans! text-base! font-bold! tracking-normal!">Brazilian Pineapple</h2>
        <div className="flex items-center gap-2">
          <div className="flex rounded-sm border border-felt-700 p-0.5" role="group" aria-label="Escolha o vídeo">
            {ORDER.map(id => (
              <button key={id} type="button" aria-pressed={current === id} onClick={() => setCurrent(id)}
                aria-label={`${FILMS[id].label}, ${formatLessonDuration(FILMS[id].video.durationSeconds!)}`}
                className={`min-h-11 whitespace-nowrap rounded-sm px-3 text-xs font-semibold focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-gold-soft ${current === id ? "bg-gold-soft text-felt-950" : "text-felt-200 hover:text-cream"}`}>
                {FILMS[id].label} · {clock(FILMS[id].video.durationSeconds!)}
              </button>
            ))}
          </div>
          <button type="button" className="min-h-11 min-w-11 rounded-sm text-xl focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-gold-soft" aria-label="Fechar vídeo" onClick={() => dialog.current?.close()}>×</button>
        </div>
      </div>
      {film && <CourseVideoPlayer key={current} video={film.video} lessonTitle={film.heading} />}
    </dialog>
  </>;
}
