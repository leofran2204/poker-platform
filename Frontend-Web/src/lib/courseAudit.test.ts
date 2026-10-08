import { describe, expect, it } from "vitest";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { allLessons, calculateQuizScores, COURSE, findLesson, isLessonUnlocked, lessonFormat, lessonPrerequisites } from "./course";
import sources from "@/data/courseSources.json";
import homeFilm from "@/data/homeFilm.json";
import pineappleFilm from "@/data/pineappleFilm.json";
import training from "@/data/courseTraining.json";

const lessons = allLessons();
const passed = { status: "completed", best_score: 100 };
const assetExists = (url: string) => existsSync(fileURLToPath(new URL(`../../public${url}`, import.meta.url)));

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
        const count = lesson.variant === "omaha" ? 4 : lesson.variant === "brazilian_pineapple" && q.board.length > 0 ? q.board.length : 2;
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
      expect(video.pronunciationVersion, lesson.id).toBe("pt-BR-poker-v1");
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
    for (const ext of ["mp4", "vtt", "webp"]) expect(assetExists(`/videos/${homeFilm.filename}.${ext}`)).toBe(true);
    expect(homeFilm.transcript).toContain("Brazilian Pineapple");
    expect(homeFilm.transcript).toContain("Short Deck");
    expect(homeFilm.pronunciationVersion).toBe("pt-BR-poker-v1");
    expect(homeFilm.narrationHash).toMatch(/^[a-f0-9]{64}$/);
  });

  it("gabaritos completos passam e respostas vazias não passam", () => {
    for (const lesson of lessons) {
      const answers = Object.fromEntries(lesson.quiz.map(q => [q.id, q.kind === "theory" ? q.options[q.expectedIndex] : q.expected]));
      expect(calculateQuizScores(lesson.quiz, answers).passed, lesson.id).toBe(true);
      expect(calculateQuizScores(lesson.quiz, {}).passed, lesson.id).toBe(false);
    }
  });

  it("verifica os 25 vídeos técnicos, com origem preservada para a regra anterior do Pineapple", () => {
    const technical = COURSE.modules.filter(m => m.id !== "m0").flatMap(m => m.lessons);
    expect(technical).toHaveLength(25);
    for (const lesson of technical) {
      const payload = { id: lesson.id, title: lesson.title, body: lesson.body, quiz: lesson.quiz,
        sources: lesson.sources ?? [], training: training.scenarios.filter(s => s.lesson === lesson.id) };
      const hash = createHash("sha256").update(JSON.stringify(payload)).digest("hex");
      const manifest = JSON.parse(readFileSync(fileURLToPath(new URL(`../../../ZeroTiltCurso/editorial/academy-${lesson.id}-manifest.json`, import.meta.url)), "utf8"));
      if (["m5l3", "m5l4", "m5l5"].includes(lesson.id)) {
        expect(lesson.video?.publicationStatus).toBe("prior_rules");
        expect(lesson.video?.bettingRuleVersion).toBe("legacy_no_limit");
        expect(manifest.bettingRuleVersion).toBe("legacy_no_limit");
        expect(manifest.originalMediaSha256).toMatch(/^[a-f0-9]{64}$/);
        const originalHash = createHash("sha256").update(JSON.stringify(manifest.sourceSnapshot)).digest("hex");
        expect(manifest.contentHash).toBe(originalHash);
        expect(lesson.video?.contentHash).toBe(originalHash);
        expect(hash).not.toBe(originalHash);
      } else {
        expect(lesson.video?.publicationStatus, lesson.id).toBe("published");
        expect(lesson.video?.contentHash, lesson.id).toBe(hash);
        expect(manifest.contentHash, lesson.id).toBe(hash);
      }
      expect(lesson.video?.pronunciationVersion, lesson.id).toBe("pt-BR-poker-v1");
      expect(manifest.pronunciationVersion, lesson.id).toBe(lesson.video?.pronunciationVersion);
      expect(manifest.narrationHash, lesson.id).toMatch(/^[a-f0-9]{64}$/);
      expect(manifest.narrationHash, lesson.id).toBe(lesson.video?.narrationHash);
      expect(lesson.video?.rendererVersion, lesson.id).toBe(manifest.rendererVersion);
      const transcript = readFileSync(fileURLToPath(new URL(`../../../ZeroTiltCurso/editorial/academy-${lesson.id}-transcript.txt`, import.meta.url)), "utf8");
      expect(lesson.video?.transcript?.replace(/\r\n/g, "\n"), lesson.id).toBe(transcript.replace(/\r\n/g, "\n").trim());
      const asset = readFileSync(fileURLToPath(new URL(`../../public${lesson.video!.url}`, import.meta.url)));
      expect(createHash("sha256").update(asset).digest("hex"), lesson.id).toBe(manifest.sha256);
      expect(asset.length, lesson.id).toBe(manifest.bytes);
      expect(lesson.video?.durationSeconds, lesson.id).toBe(manifest.durationSeconds);
      expect(lesson.video?.chapters, lesson.id).toEqual(manifest.chapters);
    }
  }, 30_000); // Lê e calcula SHA-256 dos 25 MP4, inclusive em discos mais lentos.

  it("o destaque Pineapple tem no máximo oito minutos e corresponde à regra e ao roteiro", () => {
    const editorial = new URL("../../../ZeroTiltCurso/editorial/", import.meta.url);
    const manifest = JSON.parse(readFileSync(new URL("pineapple-manifest.json", editorial), "utf8"));
    const episode = JSON.parse(readFileSync(new URL("episodes.json", editorial), "utf8")).pineapple;
    expect(pineappleFilm.durationSeconds).toBeGreaterThan(0);
    expect(pineappleFilm.durationSeconds).toBeLessThanOrEqual(480);
    expect(pineappleFilm.bettingRuleVersion).toBe("brazilian_pineapple_hybrid_v1");
    expect(pineappleFilm.bettingRuleVersion).toBe(episode.bettingRuleVersion);
    expect(pineappleFilm.bettingRuleVersion).toBe(manifest.bettingRuleVersion);
    expect(createHash("sha256").update(JSON.stringify(episode)).digest("hex")).toBe(manifest.episodeHash);
    expect(pineappleFilm.episodeHash).toBe(manifest.episodeHash);
    expect(pineappleFilm.pronunciationVersion).toBe("pt-BR-poker-v1");
    expect(manifest.pronunciationVersion).toBe(pineappleFilm.pronunciationVersion);
    expect(manifest.narrationHash).toBe(pineappleFilm.narrationHash);
    expect(pineappleFilm.chapters).toEqual(manifest.chapters);
    expect(pineappleFilm.chapters).toHaveLength(8);
    expect(pineappleFilm.chapters[0].start).toBe(0);
    expect(pineappleFilm.durationSeconds).toBe(manifest.durationSeconds);
    const transcript = readFileSync(new URL("pineapple-transcript.txt", editorial), "utf8");
    expect(pineappleFilm.transcript).toBe(transcript.replace(/\r\n/g, "\n").trim());
    expect(pineappleFilm.transcript).toContain("O Loss Deflator não se aplica ao Brazilian Pineapple");
    expect(pineappleFilm.transcript).toContain("cinquenta e cinco");
    expect(pineappleFilm.transcript).toContain("maiores de dezoito anos");
    for (const ext of ["mp4", "vtt", "webp"] as const) {
      const asset = readFileSync(fileURLToPath(new URL(`../../public/videos/${pineappleFilm.filename}.${ext}`, import.meta.url)));
      const hash = createHash("sha256").update(asset).digest("hex");
      expect(hash).toBe(pineappleFilm.assetHashes[ext]);
      expect(hash).toBe(manifest.assetHashes[ext]);
    }
  }, 30_000);

  it("a pasta pública contém a grade, o novo destaque e o institucional preservado", () => {
    const active = lessons.flatMap(l => [l.video?.url, l.video?.captionsUrl, l.video?.posterUrl]);
    for (const ext of ["mp4", "vtt", "webp"]) active.push(`/videos/${homeFilm.filename}.${ext}`);
    for (const ext of ["mp4", "vtt", "webp"]) active.push(`/videos/${pineappleFilm.filename}.${ext}`);
    const names = new Set(active.filter(Boolean).map(url => url!.split("/").pop()));
    const files = readdirSync(fileURLToPath(new URL("../../public/videos", import.meta.url))).filter(file => /\.(mp4|vtt|webp|jpg)$/i.test(file));
    expect(files.length).toBe(30 * 3);
    for (const file of files) expect(names.has(file), `Mídia sem referência vigente: ${file}`).toBe(true);
  });

  it("confere as contagens exatas ensinadas no Pineapple", () => {
    const combinations = (n: number, k: number) => {
      let result = 1;
      for (let i = 1; i <= k; i++) result = result * (n - i + 1) / i;
      return result;
    };
    expect(combinations(3, 2) * combinations(3, 3)).toBe(3);
    expect(combinations(4, 2) * combinations(4, 3)).toBe(24);
    expect(combinations(5, 2) * combinations(5, 3)).toBe(100);
    expect(100 * 2 / 47).toBeCloseTo(4.2553, 4);
    expect(1 - combinations(48, 3) / combinations(50, 3)).toBeCloseTo(0.117551, 6);
  });

  it("a tabela normativa mantém sequência acima de trinca no ranking clássico", () => {
    const rules = readFileSync(fileURLToPath(new URL("../../../Documentacao/BUSINESS_RULES.md", import.meta.url)), "utf8");
    expect(rules).toMatch(/\| 5\s+\| Straight\s+\| 5/);
    expect(rules).toMatch(/\| 4\s+\| Three of a Kind\s+\| 4/);
  });
});
