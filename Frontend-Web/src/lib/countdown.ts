/** Countdown until a unix-seconds instant. Empty string if already past. */
export function formatCountdown(epochSeconds: number, nowMs: number = Date.now()): string {
  const diff = Math.floor(epochSeconds * 1000 - nowMs);
  if (diff <= 0) return "começando";
  const total = Math.floor(diff / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  if (hours > 0) return `${hours}h ${minutes}min`;
  if (minutes > 0) return `${minutes} min ${seconds.toString().padStart(2, "0")}s`;
  return `${seconds}s`;
}

export function liveTableIds(t: {
  live_table_ids?: string[] | null;
  live_table_id?: string | null;
}): string[] {
  if (t.live_table_ids && t.live_table_ids.length > 0) return t.live_table_ids;
  if (t.live_table_id) return [t.live_table_id];
  return [];
}

export function nextStartEpoch(
  events: { scheduled_start_at?: number | null }[],
  nowSec: number = Math.floor(Date.now() / 1000),
): number | null {
  const future = events
    .map((e) => e.scheduled_start_at)
    .filter((n): n is number => typeof n === "number" && n > nowSec);
  if (future.length === 0) return null;
  return Math.min(...future);
}
