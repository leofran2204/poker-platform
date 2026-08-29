import type { WalletMode } from "@/api/types";

const KEY = "poker_wallet_mode";

export function getWalletMode(): WalletMode {
  const v = localStorage.getItem(KEY);
  return v === "real" ? "real" : "play";
}

export function setWalletModeLocal(mode: WalletMode): void {
  localStorage.setItem(KEY, mode);
}
