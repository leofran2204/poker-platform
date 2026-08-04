/** Valores da API são centavos inteiros (u64 no backend). */

export function formatBrlFromCents(cents: number | bigint): string {
  const n = typeof cents === "bigint" ? Number(cents) : cents;
  const sign = n < 0 ? "-" : "";
  const abs = Math.abs(Math.trunc(n));
  const whole = Math.floor(abs / 100);
  const frac = abs % 100;
  return `${sign}R$ ${whole.toLocaleString("pt-BR")},${frac.toString().padStart(2, "0")}`;
}

export function formatChips(cents: number): string {
  return formatBrlFromCents(cents);
}

export function parseBrlToCents(input: string): number | null {
  const cleaned = input.replace(/\s/g, "").replace("R$", "").replace(/\./g, "").replace(",", ".");
  const value = Number(cleaned);
  if (!Number.isFinite(value) || value < 0) return null;
  return Math.round(value * 100);
}
