import React from "react";
import { Composition } from "remotion";
import m2l2Script from "./data/m2l2.json";
import m5l3Script from "./data/academy-m5l3.json";
import m5l4Script from "./data/academy-m5l4.json";
import m5l5Script from "./data/academy-m5l5.json";
import rulesScript from "./data/pineapple-draw-rules.json";
import trailerScript from "./data/pineapple-draw-trailer.json";
import m2l2Timings from "./generated/academy-m2l2.timings.json";
import m5l3Timings from "./generated/academy-m5l3.timings.json";
import m5l4Timings from "./generated/academy-m5l4.timings.json";
import m5l5Timings from "./generated/academy-m5l5.timings.json";
import rulesTimings from "./generated/pineapple-draw-rules.timings.json";
import trailerTimings from "./generated/pineapple-draw-trailer.timings.json";
import { Script, Timings } from "./lib/timeline";
import { H, W } from "./theme";
import { totalFrames, Video } from "./Video";
import { M2L2_SCENES } from "./videos/AcademyM2L2";
import { PINEAPPLE_DRAW_SCENES } from "./videos/PineappleDrawRules";
import type { SceneMap } from "./Video";

type Entry = { id: string; script: Script; timings: Timings; scenes: SceneMap; label: string; chrome?: boolean };

/* Um vídeo = roteiro (src/data) + tempos da narração (src/generated) + cenas. */
const VIDEOS: Entry[] = [
  { id: "AcademyM2L2", script: m2l2Script as Script, timings: m2l2Timings as Timings, scenes: M2L2_SCENES, label: "ZT ACADEMY · M2 · AULA 2" },
  {
    id: "PineappleDrawRules",
    script: rulesScript as Script,
    timings: rulesTimings as Timings,
    scenes: PINEAPPLE_DRAW_SCENES,
    label: "ZT ACADEMY · BRAZILIAN PINEAPPLE",
  },
  {
    id: "PineappleDrawTrailer",
    script: trailerScript as Script,
    timings: trailerTimings as Timings,
    scenes: PINEAPPLE_DRAW_SCENES,
    label: "ZT POKER · BRAZILIAN PINEAPPLE",
    chrome: false,
  },
  { id: "AcademyM5L3", script: m5l3Script as Script, timings: m5l3Timings as Timings, scenes: PINEAPPLE_DRAW_SCENES, label: "ZT ACADEMY · BRAZILIAN PINEAPPLE · AULA 1" },
  { id: "AcademyM5L4", script: m5l4Script as Script, timings: m5l4Timings as Timings, scenes: PINEAPPLE_DRAW_SCENES, label: "ZT ACADEMY · BRAZILIAN PINEAPPLE · AULA 2" },
  { id: "AcademyM5L5", script: m5l5Script as Script, timings: m5l5Timings as Timings, scenes: PINEAPPLE_DRAW_SCENES, label: "ZT ACADEMY · BRAZILIAN PINEAPPLE · AULA 3" },
];

const COMPONENTS = VIDEOS.map((entry) => {
  const Component: React.FC = () => (
    <Video script={entry.script} timings={entry.timings} scenes={entry.scenes} label={entry.label} chrome={entry.chrome} />
  );
  Component.displayName = entry.id;
  return Component;
});

export const Root: React.FC = () => (
  <>
    {VIDEOS.map((entry, i) => (
      <Composition
        key={entry.id}
        id={entry.id}
        component={COMPONENTS[i]}
        durationInFrames={totalFrames(entry.script, entry.timings)}
        fps={entry.script.fps}
        width={W}
        height={H}
      />
    ))}
  </>
);
