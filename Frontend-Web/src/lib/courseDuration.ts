export function formatLessonDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const minutes = Math.floor(total / 60);
  const rest = total % 60;
  if (minutes === 0) return `${rest} s`;
  if (rest === 0) return minutes === 1 ? "1 min" : `${minutes} min`;
  return `${minutes} min ${rest} s`;
}
