import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { allLessons, COURSE } from "./course";

// Brazilian Pineapple desde a v3: troca no flop, no turn e no river; o descarte
// do river vem depois da troca (BUSINESS_RULES, seção do Brazilian Pineapple).
function staleRule(text: string): string[] {
  return text.split(/[.!?](?:\s|["'`<]|$)|\r?\n|\\n/).filter((sentence) => {
    const twoSwaps = /\b(?:2|duas) trocas\b/i.test(sentence);
    const swapWithoutRiver =
      /flop e (?:no |do )?turn/i.test(sentence) &&
      /\btro(?:c|qu)/i.test(sentence) &&
      !/river.*\btro(?:c|qu)/i.test(sentence);
    const discardWithoutSwap = /no river,? (?:você )?descart(?:e|a) uma/i.test(sentence);
    return twoSwaps || swapWithoutRiver || discardWithoutSwap;
  });
}

const src = fileURLToPath(new URL("../", import.meta.url));
const read = (path: string) => readFileSync(`${src}${path}`, "utf8");

describe("texto da regra do Brazilian Pineapple no site", () => {
  it("nenhum arquivo do site descreve duas trocas ou river sem troca", () => {
    const files = readdirSync(src, { recursive: true, encoding: "utf8" })
      .map((path) => path.replace(/\\/g, "/"))
      .filter((path) => /\.(?:tsx?|json)$/.test(path) && !/\.test\.tsx?$/.test(path))
      // O curso é conferido por aula, no teste seguinte.
      .filter((path) => path !== "data/courseContent.json");
    expect(files).toContain("pages/HomePage.tsx");
    expect(files).toContain("pages/TrainingPage.tsx");
    expect(files.flatMap((path) => staleRule(read(path)).map((s) => `${path}: ${s.trim().slice(0, 200)}`))).toEqual([]);
  });

  it("nenhuma aula ensina river sem troca", () => {
    const lessons = allLessons(COURSE);
    expect(lessons.flatMap((lesson) => staleRule(JSON.stringify(lesson)).map((s) => `${lesson.id}: ${s.trim().slice(0, 200)}`))).toEqual([]);
  });

  it("cartão da home e mesa de treino dizem as três trocas", () => {
    expect(read("pages/HomePage.tsx")).toContain("3 CARTAS · 3 TROCAS · 1 DESCARTE");
    expect(read("pages/TrainingPage.tsx")).toContain("Na sua vez no flop, no turn e no river");
  });
});
