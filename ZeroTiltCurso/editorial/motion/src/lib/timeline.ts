export type Word = { text: string; startMs: number; endMs: number };
export type SceneTiming = { id: string; audio: string; durationMs: number; words: Word[] };
export type Timings = { voice: string; rate: string; scenes: SceneTiming[] };
export type ScriptScene = {
  id: string;
  chapter: string;
  voice: string;
  padAfter?: number;
  title?: string;
  /** Parâmetros visuais de cenas guiadas por dados (filme de regras). */
  visual?: Record<string, unknown>;
};
export type Script = { id: string; title: string; fps: number; scenes: ScriptScene[] };

/** Respiro antes da voz e depois dela, em segundos. */
export const LEAD = 0.45;
export const TAIL = 0.65;
/** Quadros de transição cruzada entre cenas. */
export const OVERLAP = 14;

export type PlannedScene = {
  id: string;
  chapter: string;
  index: number;
  from: number;
  duration: number;
  audioAt: number;
  speechEnd: number;
  timing: SceneTiming;
};

export const normalize = (value: string) =>
  value
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "");

export function plan(script: Script, timings: Timings, fps: number) {
  let cursor = 0;
  const scenes: PlannedScene[] = script.scenes.map((scene, index) => {
    const timing = timings.scenes.find((item) => item.id === scene.id);
    if (!timing) {
      throw new Error(`Sem narração para a cena ${scene.id}; rode narrate.py`);
    }
    const audioAt = Math.round(LEAD * fps);
    const speech = Math.round((timing.durationMs / 1000) * fps);
    const duration = audioAt + speech + Math.round((scene.padAfter ?? TAIL) * fps);
    const planned = {
      id: scene.id,
      chapter: scene.chapter,
      index,
      from: cursor,
      duration,
      audioAt,
      speechEnd: audioAt + speech,
      timing,
    };
    cursor += duration;
    return planned;
  });
  return { scenes, total: cursor };
}

export type Cue = (token: string, occurrence?: number, edge?: "start" | "end") => number;

/**
 * Quadro local (a partir do início da cena) em que a palavra ou a frase é dita.
 * Frases ("Beto paga") casam palavras consecutivas; o quadro é o da primeira.
 */
export function makeCue(scene: PlannedScene, fps: number): Cue {
  const words = scene.timing.words.map((word) => ({ ...word, key: normalize(word.text) }));
  return (token, occurrence = 1, edge = "start") => {
    const parts = token.split(/\s+/).map(normalize).filter(Boolean);
    let seen = 0;
    for (let i = 0; i + parts.length <= words.length; i++) {
      if (parts.every((part, k) => words[i + k].key === part)) {
        seen += 1;
        if (seen === occurrence) {
          const ms = edge === "start" ? words[i].startMs : words[i + parts.length - 1].endMs;
          return scene.audioAt + Math.round((ms / 1000) * fps);
        }
      }
    }
    throw new Error(`Cena ${scene.id}: "${token}" (${occurrence}ª) não está na narração`);
  };
}
