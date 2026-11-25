import { useCallback, useEffect, useRef, useState } from "react";
import {
  answerCourseCheck,
  fetchCourseLessonState,
  sendCourseWatch,
  type CourseLessonState,
} from "@/api/client";

/** Um trecho a cada 10 s de vídeo contínuo (o servidor aceita até 30 s). */
const SEGMENT_SECONDS = 10;
/** Mesmo teto da API: acima de 2x o trecho não conta. */
export const MAX_COUNTED_RATE = 2;

export function clock(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/**
 * Estado da aula no servidor e contagem do vídeo: manda ao servidor só os
 * trechos tocados sem pulo, com a aba visível e em até 2x. O servidor confere
 * de novo cada trecho contra o relógio e decide o que conta.
 */
export function useLessonWatch(lessonId: string, enabled: boolean) {
  const [state, setState] = useState<CourseLessonState | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const apply = useCallback((next: CourseLessonState) => {
    setState(next);
    const video = videoRef.current;
    if (next.check_result === "wrong" || next.check_result === "expired") {
      video?.pause();
      const back = next.rewind_to ?? 0;
      if (video) video.currentTime = back;
      setNotice(
        next.check_result === "wrong"
          ? `Resposta errada. O vídeo voltou para ${clock(back)}: esse trecho precisa ser assistido de novo.`
          : `Sem resposta em 60 s: o vídeo pausou e voltou para ${clock(back)}. Esse trecho precisa ser assistido de novo.`,
      );
    } else if (next.check_result === "right") {
      setNotice("Certo! Pode seguir com o vídeo.");
    }
    if (next.pending_check && document.fullscreenElement) {
      // Em tela cheia a pergunta não apareceria.
      void document.exitFullscreen().catch(() => undefined);
    }
  }, []);

  const refresh = useCallback(async () => {
    if (!enabled || !lessonId) return;
    try {
      setState(await fetchCourseLessonState(lessonId));
    } catch {
      // Sem rede: a página segue com o último estado.
    }
  }, [enabled, lessonId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const current = state?.lesson_id === lessonId ? state : null;
  const tracking = enabled && current !== null && current.has_video && current.review === null;

  useEffect(() => {
    const video = videoRef.current;
    if (!tracking || !video) return;
    let start: number | null = null;
    let lastT = 0;
    let lastWall = 0;
    const send = (from: number, to: number) => {
      const rate = video.playbackRate;
      queue.current = queue.current.then(async () => {
        try {
          apply(await sendCourseWatch(lessonId, from, to, rate));
        } catch {
          // Sem rede: o trecho não conta.
        }
      });
    };
    const eligible = () =>
      !video.paused &&
      !video.ended &&
      document.visibilityState === "visible" &&
      video.playbackRate <= MAX_COUNTED_RATE;
    const flush = () => {
      if (start !== null && lastT - start >= 0.5) send(start, lastT);
      start = null;
    };
    const begin = () => {
      flush();
      if (!eligible()) return;
      start = video.currentTime;
      lastT = start;
      lastWall = performance.now();
      send(start, start);
    };
    const onTime = () => {
      if (start === null) {
        if (eligible()) begin();
        return;
      }
      if (!eligible()) {
        flush();
        return;
      }
      const t = video.currentTime;
      const now = performance.now();
      const wall = (now - lastWall) / 1000;
      if (t < lastT - 0.5 || t - lastT > MAX_COUNTED_RATE * 1.25 * wall + 1) {
        begin(); // Pulo: o trecho anterior fecha onde estava.
        return;
      }
      lastT = t;
      lastWall = now;
      if (t - start >= SEGMENT_SECONDS) {
        send(start, t);
        start = t;
      }
    };
    const onStop = () => {
      const t = video.currentTime;
      if (start !== null && t >= lastT && t - lastT < 1) lastT = t;
      flush();
    };
    const onVisibility = () => (document.visibilityState === "visible" ? begin() : flush());
    video.addEventListener("playing", begin);
    video.addEventListener("timeupdate", onTime);
    video.addEventListener("pause", onStop);
    video.addEventListener("ended", onStop);
    video.addEventListener("seeking", flush);
    video.addEventListener("ratechange", begin);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      flush();
      video.removeEventListener("playing", begin);
      video.removeEventListener("timeupdate", onTime);
      video.removeEventListener("pause", onStop);
      video.removeEventListener("ended", onStop);
      video.removeEventListener("seeking", flush);
      video.removeEventListener("ratechange", begin);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [tracking, lessonId, apply]);

  const answer = useCallback(
    async (checkId: string, option: string | null) => {
      try {
        apply(await answerCourseCheck(lessonId, checkId, option));
      } catch {
        setNotice("Não foi possível enviar a resposta. Tente de novo.");
      }
    },
    [lessonId, apply],
  );

  return {
    state: current,
    notice,
    clearNotice: () => setNotice(null),
    videoRef,
    refresh,
    answer,
  };
}
