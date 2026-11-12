import { describe, expect, it } from "vitest";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { allLessons, calculateQuizScores, COURSE, findLesson, isLessonUnlocked, lessonFormat, lessonPrerequisites } from "./course";
import sources from "@/data/courseSources.json";
import training from "@/data/courseTraining.json";
import films from "@/data/pineappleDrawFilms.json";

const lessons = allLessons();
const passed = { status: "completed", best_score: 100 };
const assetExists = (url: string) => existsSync(fileURLToPath(new URL(`../../public${url}`, import.meta.url)));
/** Aulas do Brazilian Pineapple: vídeo produzido em Remotion (ZeroTiltCurso/editorial/motion). */
const MOTION_LESSONS = ["m5l3", "m5l4", "m5l5"];
const editorial = (file: string) => fileURLToPath(new URL(`../../../ZeroTiltCurso/editorial/${file}`, import.meta.url));
const sha256 = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");

describe("integridade da Academy", () => {
  it("preserva os IDs antigos sem duplicar aulas ou perguntas", () => {
    const ids = lessons.map(l => l.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (let i = 1; i <= 8; i++) expect(ids).toContain(`m0l${i}`);
    for (let m = 1; m <= 6; m++) for (let l = 1; l <= 3; l++) expect(ids).toContain(`m${m}l${l}`);
    const questionIds = lessons.flatMap(l => l.quiz.map(q => q.id));
    expect(new Set(questionIds).size).toBe(questionIds.length);
  });

  it("tem gabaritos, alternativas e cartas coerentes com cada modalidade", () => {
    for (const lesson of lessons) {
      expect(lesson.quiz.length, lesson.id).toBeGreaterThan(0);
      const prompts = new Set<string>();
      for (const q of lesson.quiz) {
        expect(prompts.has(q.prompt), q.id).toBe(false);
        prompts.add(q.prompt);
        expect(q.options.length, q.id).toBeGreaterThan(1);
        expect(new Set(q.options).size, q.id).toBe(q.options.length);
        if (q.kind === "theory") {
          expect(Number.isInteger(q.expectedIndex), q.id).toBe(true);
          expect(q.expectedIndex, q.id).toBeGreaterThanOrEqual(0);
          expect(q.expectedIndex, q.id).toBeLessThan(q.options.length);
          continue;
        }
        expect(q.options, q.id).toContain(q.expected);
        expect([0, 3, 4, 5], q.id).toContain(q.board.length);
        const cards = [...q.hole, ...q.board];
        expect(new Set(cards).size, q.id).toBe(cards.length);
        for (const card of cards) {
          expect(card, q.id).toMatch(/^[2-9TJQKA][cdhs]$/);
          if (lesson.variant === "short_deck") expect(card, q.id).toMatch(/^[6-9TJQKA][cdhs]$/);
        }
        const count = lesson.variant === "omaha" ? 4 : lesson.variant === "pineapple_draw" ? 3 : 2;
        expect(q.hole.length, q.id).toBe(count);
      }
    }
  });

  it("mantém um grafo de pré-requisitos existente e sem ciclos", () => {
    const walk = (id: string, chain: string[] = []) => {
      expect(findLesson(id), id).toBeDefined();
      expect(chain, id).not.toContain(id);
      for (const prereq of lessonPrerequisites(id)) walk(prereq, [...chain, id]);
    };
    for (const lesson of lessons) walk(lesson.id);
  });

  it("começa em regras e permite escolher variantes sem concluir Hold’em avançado", () => {
    expect(COURSE.modules[0].lessons[0].id).toBe("m0l4");
    for (const id of ["m5l1", "m5l2", "m5l3"]) {
      expect(isLessonUnlocked(id, {})).toBe(false);
      expect(isLessonUnlocked(id, { m0l4: passed })).toBe(true);
    }
    for (const id of ["m0l1", "m0l2", "m0l3"]) expect(isLessonUnlocked(id, {})).toBe(true);
    expect(isLessonUnlocked("m5l4", { m0l4: passed })).toBe(false);
    expect(isLessonUnlocked("m5l4", { m5l3: passed })).toBe(true);
  });

  it("rejeita aula inexistente e mantém acesso ao progresso anterior", () => {
    expect(isLessonUnlocked("missing", {})).toBe(false);
    expect(isLessonUnlocked("missing", { missing: passed })).toBe(false);
    expect(isLessonUnlocked("m5l5", { m5l5: passed })).toBe(true);
    expect(isLessonUnlocked("m5l5", { m5l4: {status: "started", best_score: 69} })).toBe(false);
    expect(isLessonUnlocked("m5l5", { m5l4: {status: "started", best_score: 70} })).toBe(true);
  });

  it("não anuncia vídeo em revisão como disponível", () => {
    for (const lesson of lessons) {
      if (lesson.video?.publicationStatus === "review" || !lesson.video?.url) expect(lessonFormat(lesson)).not.toContain("Vídeo");
    }
    expect(lessonFormat(findLesson("m0l1")!)).toContain("Vídeo");
  });

  it("fontes e arquivos de vídeos publicados existem", () => {
    for (const lesson of lessons) {
      for (const id of lesson.sources ?? []) expect(sources, id).toHaveProperty(id);
      const video = lesson.video;
      if (!video?.url || video.publicationStatus === "review") continue;
      expect(video.pronunciationVersion, lesson.id).toBe(MOTION_LESSONS.includes(lesson.id) ? "pt-BR-poker-v2-lang-en" : "pt-BR-poker-v1");
      expect(video.narrationHash, lesson.id).toMatch(/^[a-f0-9]{64}$/);
      for (const asset of [video.url, video.captionsUrl, video.posterUrl]) {
        expect(asset, lesson.id).toBeTruthy();
        expect(assetExists(asset!), asset).toBe(true);
      }
      expect(video.transcript?.length, lesson.id).toBeGreaterThan(100);
      for (const chapter of video.chapters ?? []) {
        expect(chapter.start).toBeGreaterThanOrEqual(0);
        expect(chapter.start).toBeLessThan(video.durationSeconds!);
      }
    }
  });

  it("gabaritos completos passam e respostas vazias não passam", () => {
    for (const lesson of lessons) {
      const answers = Object.fromEntries(lesson.quiz.map(q => [q.id, q.kind === "theory" ? q.options[q.expectedIndex] : q.expected]));
      expect(calculateQuizScores(lesson.quiz, answers).passed, lesson.id).toBe(true);
      expect(calculateQuizScores(lesson.quiz, {}).passed, lesson.id).toBe(false);
    }
  });

  it("verifica os 25 vídeos técnicos publicados, inclusive as 3 aulas do Brazilian Pineapple em Remotion", () => {
    const technical = COURSE.modules.filter(m => m.id !== "m0").flatMap(m => m.lessons);
    expect(technical).toHaveLength(25);
    for (const lesson of technical) {
      if (MOTION_LESSONS.includes(lesson.id)) {
        const manifest = JSON.parse(readFileSync(editorial(`academy-${lesson.id}-manifest.json`), "utf8"));
        expect(lesson.video?.renderer, lesson.id).toBe("remotion");
        expect(manifest.renderer, lesson.id).toBe("remotion");
        expect(manifest.lessonId, lesson.id).toBe(lesson.id);
        expect(manifest.scriptHash, lesson.id).toMatch(/^[a-f0-9]{64}$/);
        expect(lesson.video?.bettingRuleVersion, lesson.id).toBe("pineapple_draw_v1");
        expect(manifest.bettingRuleVersion, lesson.id).toBe("pineapple_draw_v1");
        expect(lesson.video?.url, lesson.id).toBe(`/videos/zt-academy-${lesson.id}-v4.mp4`);
      }
      const payload = { id: lesson.id, title: lesson.title, body: lesson.body, quiz: lesson.quiz,
        sources: lesson.sources ?? [], training: training.scenarios.filter(s => s.lesson === lesson.id) };
      const hash = createHash("sha256").update(JSON.stringify(payload)).digest("hex");
      const manifest = JSON.parse(readFileSync(fileURLToPath(new URL(`../../../ZeroTiltCurso/editorial/academy-${lesson.id}-manifest.json`, import.meta.url)), "utf8"));
      expect(lesson.video?.publicationStatus, lesson.id).toBe("published");
      expect(lesson.video?.contentHash, lesson.id).toBe(hash);
      expect(manifest.contentHash, lesson.id).toBe(hash);
      // Vídeos em Remotion: termos em inglês lidos em inglês (pronúncia aprovada em 11/11/2026).
      expect(lesson.video?.pronunciationVersion, lesson.id).toBe(MOTION_LESSONS.includes(lesson.id) ? "pt-BR-poker-v2-lang-en" : "pt-BR-poker-v1");
      expect(manifest.pronunciationVersion, lesson.id).toBe(lesson.video?.pronunciationVersion);
      expect(manifest.narrationHash, lesson.id).toMatch(/^[a-f0-9]{64}$/);
      expect(manifest.narrationHash, lesson.id).toBe(lesson.video?.narrationHash);
      expect(lesson.video?.mediaRevision, lesson.id).toBe(manifest.mediaRevision);
      const hashes = Object.fromEntries(["mp4", "vtt", "webp"].map(ext => [ext, createHash("sha256").update(readFileSync(new URL(`../../public/videos/${manifest.filename}.${ext}`, import.meta.url))).digest("hex")]));
      expect(hashes, lesson.id).toEqual(manifest.assetHashes);
      expect(createHash("sha256").update(JSON.stringify(hashes)).digest("hex"), lesson.id).toBe(manifest.mediaRevision);
      expect(lesson.video?.rendererVersion, lesson.id).toBe(manifest.rendererVersion);
      expect(lesson.video?.renderer, lesson.id).toBe(manifest.renderer);
      const transcript = readFileSync(fileURLToPath(new URL(`../../../ZeroTiltCurso/editorial/academy-${lesson.id}-transcript.txt`, import.meta.url)), "utf8");
      expect(lesson.video?.transcript?.replace(/\r\n/g, "\n"), lesson.id).toBe(transcript.replace(/\r\n/g, "\n").trim());
      const asset = readFileSync(fileURLToPath(new URL(`../../public${lesson.video!.url}`, import.meta.url)));
      expect(createHash("sha256").update(asset).digest("hex"), lesson.id).toBe(manifest.sha256);
      expect(asset.length, lesson.id).toBe(manifest.bytes);
      expect(lesson.video?.durationSeconds, lesson.id).toBe(manifest.durationSeconds);
      expect(lesson.video?.chapters, lesson.id).toEqual(manifest.chapters);
    }
  }, 30_000); // Lê e calcula SHA-256 dos 25 MP4, inclusive em discos mais lentos.

  it("filme de regras e trailer do Brazilian Pineapple batem com manifesto, transcrição e mídia", () => {
    expect(Object.keys(films).sort()).toEqual(["pineapple-draw-rules", "pineapple-draw-trailer"]);
    for (const [id, film] of Object.entries(films)) {
      const manifest = JSON.parse(readFileSync(editorial(`${id}-manifest.json`), "utf8"));
      expect(manifest.filmId).toBe(id);
      expect(film.filename).toBe(manifest.filename);
      expect(film.filename).toBe(`zt-${id}-v1`);
      expect(film.publicationStatus).toBe("published");
      expect(film.renderer).toBe("remotion");
      expect(manifest.renderer).toBe("remotion");
      expect(film.rendererVersion).toBe(manifest.rendererVersion);
      expect(film.bettingRuleVersion).toBe("pineapple_draw_v1");
      expect(manifest.bettingRuleVersion).toBe("pineapple_draw_v1");
      expect(film.pronunciationVersion).toBe("pt-BR-poker-v2-lang-en");
      expect(film.narrationHash).toBe(manifest.narrationHash);
      expect(film.chapters).toEqual(manifest.chapters);
      expect(film.durationSeconds).toBe(manifest.durationSeconds);
      expect(film.transcript.replace(/\r\n/g, "\n")).toBe(readFileSync(editorial(`${id}-transcript.txt`), "utf8").replace(/\r\n/g, "\n").trim());
      expect(film.transcript).toContain("Brazilian Pineapple");
      expect(film.transcript.toLowerCase()).not.toMatch(/pineapple draw|exatamente duas|\brua\b|rival/);
      const hashes = Object.fromEntries(["mp4", "vtt", "webp"].map(ext => [ext, sha256(readFileSync(new URL(`../../public/videos/${film.filename}.${ext}`, import.meta.url)))]));
      expect(hashes, id).toEqual(manifest.assetHashes);
      expect(sha256(JSON.stringify(hashes))).toBe(manifest.mediaRevision);
      expect(film.mediaRevision).toBe(manifest.mediaRevision);
      expect(hashes.mp4).toBe(manifest.sha256);
    }
    // O trailer mostra uma mão inteira (apostas, trocas, descarte e showdown).
    expect(films["pineapple-draw-trailer"].durationSeconds).toBeLessThanOrEqual(160);
    expect(films["pineapple-draw-rules"].chapters).toHaveLength(13);
  }, 30_000);

  it("a pasta pública contém só a grade vigente, sem mídia de formatos retirados", () => {
    const active = lessons.flatMap(l => [l.video?.url, l.video?.captionsUrl, l.video?.posterUrl]);
    const names = new Set(active.filter(Boolean).map(url => url!.split("/").pop()));
    for (const film of Object.values(films)) for (const ext of ["mp4", "vtt", "webp"]) names.add(`${film.filename}.${ext}`);
    const files = readdirSync(fileURLToPath(new URL("../../public/videos", import.meta.url))).filter(file => /\.(mp4|vtt|webp|jpg)$/i.test(file));
    expect(files.length).toBe((28 + 2) * 3);
    expect(files.some(file => /brazilian|hybrid|apresentacao|m5l[345]-v3/.test(file))).toBe(false);
    for (const file of files) expect(names.has(file), `Mídia sem referência vigente: ${file}`).toBe(true);
  });

  it("confere as contagens exatas ensinadas no Brazilian Pineapple", () => {
    const combinations = (n: number, k: number) => {
      let result = 1;
      for (let i = 1; i <= k; i++) result = result * (n - i + 1) / i;
      return result;
    };
    // Mãos de cinco cartas com no máximo duas das três privadas: flop, turn e river.
    const candidates = (board: number) =>
      [0, 1, 2].reduce((sum, hole) => sum + combinations(3, hole) * combinations(board, 5 - hole), 0);
    expect(combinations(3, 2)).toBe(3);
    expect(candidates(3)).toBe(3);
    expect(candidates(4)).toBe(15);
    expect(candidates(5)).toBe(46);
    expect(100 * 2 / 46).toBeCloseTo(4.3478, 4);
    expect(1 - (44 / 46) * (43 / 45)).toBeCloseTo(0.0860, 4);
    expect(1 - combinations(48, 3) / combinations(50, 3)).toBeCloseTo(0.117551, 6);
  });

  it("a tabela normativa mantém sequência acima de trinca no ranking clássico", () => {
    const rules = readFileSync(fileURLToPath(new URL("../../../Documentacao/BUSINESS_RULES.md", import.meta.url)), "utf8");
    expect(rules).toMatch(/\| 5\s+\| Straight\s+\| 5/);
    expect(rules).toMatch(/\| 4\s+\| Three of a Kind\s+\| 4/);
  });
});
