import React from "react";
import { AbsoluteFill, Audio, interpolate, Sequence, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import { Background, Chrome } from "./components/Frame";
import { CLAMP } from "./lib/anim";
import { Cue, makeCue, OVERLAP, plan, PlannedScene, Script, ScriptScene, Timings } from "./lib/timeline";

export type SceneProps = {
  cue: Cue;
  /** Duração total da cena em quadros (sem a sobreposição de saída). */
  duration: number;
  /** Quadro local em que a narração da cena termina. */
  speechEnd: number;
  /** Quadro local em que a narração começa. */
  audioAt: number;
  /** Dados do roteiro da cena (título e parâmetros visuais, quando houver). */
  data: ScriptScene;
};

export type SceneMap = Record<string, React.FC<SceneProps>>;

const SceneShell: React.FC<{ scene: PlannedScene; first: boolean; last: boolean; children: React.ReactNode }> = ({
  scene,
  first,
  last,
  children,
}) => {
  const frame = useCurrentFrame();
  const fadeIn = first ? 1 : interpolate(frame, [0, OVERLAP], [0, 1], CLAMP);
  const fadeOut = last ? 1 : interpolate(frame, [scene.duration, scene.duration + OVERLAP], [1, 0], CLAMP);
  return <AbsoluteFill style={{ opacity: Math.min(fadeIn, fadeOut) }}>{children}</AbsoluteFill>;
};

export const Video: React.FC<{ script: Script; timings: Timings; scenes: SceneMap; label: string; chrome?: boolean }> = ({
  script,
  timings,
  scenes,
  label,
  chrome = true,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { scenes: planned, total } = plan(script, timings, fps);
  const current = planned.reduce((found, scene) => (frame >= scene.from ? scene : found), planned[0]);
  const chapterStart = planned.find((scene) => scene.chapter === current.chapter)?.from ?? 0;

  // Trilha: mais presente sem voz, baixa sob a narração, com rampas suaves.
  const speaking = (f: number) =>
    planned.some((scene) => f >= scene.from + scene.audioAt - 8 && f <= scene.from + scene.speechEnd + 8);
  const musicVolume = (f: number) => {
    const fade = Math.min(interpolate(f, [0, 45], [0, 1], CLAMP), interpolate(f, [total - 120, total], [1, 0], CLAMP));
    let level = 0;
    const window = 14;
    for (let k = -window; k <= window; k += 2) {
      level += speaking(f + k) ? 0.055 : 0.17;
    }
    return (level / (window + 1)) * fade;
  };

  return (
    <AbsoluteFill style={{ backgroundColor: "#050e09" }}>
      <Background />
      <Audio src={staticFile("sfx/music.wav")} volume={musicVolume} />
      {planned.map((scene, index) => {
        const data = script.scenes[index];
        const mode = typeof data.visual?.mode === "string" ? data.visual.mode : undefined;
        const Scene = scenes[scene.id] ?? (mode ? scenes[`mode:${mode}`] : undefined);
        if (!Scene) throw new Error(`Cena sem componente: ${scene.id}${mode ? ` (modo ${mode})` : ""}`);
        const last = index === planned.length - 1;
        return (
          <Sequence key={scene.id} from={scene.from} durationInFrames={scene.duration + (last ? 0 : OVERLAP)} name={scene.id}>
            <SceneShell scene={scene} first={index === 0} last={last}>
              <Scene
                cue={makeCue(scene, fps)}
                duration={scene.duration}
                speechEnd={scene.speechEnd}
                audioAt={scene.audioAt}
                data={data}
              />
            </SceneShell>
            <Sequence from={scene.audioAt} layout="none">
              <Audio src={staticFile(scene.timing.audio)} />
            </Sequence>
          </Sequence>
        );
      })}
      {chrome && <Chrome chapter={current.chapter} chapterStart={chapterStart} label={label} progress={frame / total} />}
    </AbsoluteFill>
  );
};

export const totalFrames = (script: Script, timings: Timings) => plan(script, timings, script.fps).total;
