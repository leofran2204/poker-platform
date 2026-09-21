import { describe, expect, it } from "vitest";
import { formatLessonDuration } from "./course";

describe("formatLessonDuration", () => {
  it("formats under a minute", () => {
    expect(formatLessonDuration(45)).toBe("45 s");
  });

  it("formats exact minutes", () => {
    expect(formatLessonDuration(60)).toBe("1 min");
    expect(formatLessonDuration(120)).toBe("2 min");
  });

  it("formats minutes and seconds without calling seconds minutes", () => {
    expect(formatLessonDuration(89)).toBe("1 min 29 s");
  });
});
