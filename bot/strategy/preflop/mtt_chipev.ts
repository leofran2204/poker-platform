// RFI MTT ChipEV por profundidade (ante ~12.5% BB, 8-max).
// Valores em % de maos; conversao via topPercentRange().

const RFI_PCT: Record<string, Record<number, number>> = {
  UTG: { 20: 12, 30: 14, 40: 16, 50: 18, 75: 20, 100: 22 },
  HJ: { 20: 16, 30: 19, 40: 21, 50: 23, 75: 25, 100: 27 },
  CO: { 20: 24, 30: 28, 40: 31, 50: 33, 75: 36, 100: 38 },
  BTN: { 20: 38, 30: 44, 40: 48, 50: 50, 75: 53, 100: 55 },
  SB: { 20: 55, 30: 62, 40: 68, 50: 72, 75: 78, 100: 82 },
};

const DEPTHS = [20, 30, 40, 50, 75, 100];

export function rfiPercentMTT(position: string, stackBB: number): number {
  const row = RFI_PCT[position];
  if (!row) return 0;
  let best = DEPTHS[0];
  for (const d of DEPTHS) {
    if (Math.abs(d - stackBB) < Math.abs(best - stackBB)) best = d;
  }
  if (stackBB <= 15) return Math.round(row[20] * 0.8);
  return row[best];
}

export function openSizeMTT(stackBB: number): number {
  if (stackBB <= 20) return 2;
  if (stackBB <= 40) return 2.2;
  return 2.5;
}
