import { describe, expect, it } from "vitest";
import { formatCountdown, liveTableIds, nextStartEpoch } from "./countdown";

describe("formatCountdown", () => {
  it("formats hours and minutes", () => {
    expect(formatCountdown(3_600 + 120, 0)).toBe("1h 2min");
  });

  it("formats minutes and seconds", () => {
    expect(formatCountdown(125, 0)).toBe("2 min 05s");
  });

  it("marks the start when due", () => {
    expect(formatCountdown(1, 5_000)).toBe("começando");
  });
});

describe("liveTableIds", () => {
  it("prefers the list", () => {
    expect(liveTableIds({ live_table_ids: ["a", "b"], live_table_id: "z" })).toEqual(["a", "b"]);
  });
});

describe("nextStartEpoch", () => {
  it("picks the soonest future start", () => {
    expect(nextStartEpoch([{ scheduled_start_at: 50 }, { scheduled_start_at: 20 }], 10)).toBe(20);
  });
});
