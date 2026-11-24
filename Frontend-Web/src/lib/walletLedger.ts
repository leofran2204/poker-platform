import type { WalletLedgerEntry } from "@/api/types";

/** Nome de cada movimento do livro único, como o jogador lê no extrato. */
const REASON_LABELS: Record<string, string> = {
  opening_balance: "Saldo inicial",
  account_opening: "Abertura da conta",
  pm_daily_reset: "Renovação diária",
  cash_buy_in: "Entrada na mesa",
  cash_out: "Saída da mesa",
  boot_cash_out: "Saída da mesa (reinício)",
  tournament_buy_in: "Inscrição em torneio",
  tournament_fee: "Taxa de inscrição",
  tournament_unregister_refund: "Inscrição cancelada",
  tournament_rebuy: "Rebuy",
  tournament_addon: "Add-on",
  tournament_prize: "Prêmio de torneio",
  pix_deposit: "Depósito PIX",
  pix_deposit_reversal: "Estorno de depósito PIX",
  pix_withdraw: "Saque PIX",
  pix_withdraw_reversal: "Saque devolvido",
  manual_deposit: "Depósito aprovado",
  admin_adjustment: "Ajuste da plataforma",
  bot_top_up: "Banca da casa",
};

const WALLET_LABELS: Record<WalletLedgerEntry["wallet"], string> = {
  pm_cash: "Play Money · mesa",
  pm_mtt: "Play Money · torneio",
  real: "Jogo Real",
};

export function ledgerReasonLabel(reason: string): string {
  return REASON_LABELS[reason] ?? "Movimento";
}

export function ledgerWalletLabel(wallet: WalletLedgerEntry["wallet"]): string {
  return WALLET_LABELS[wallet] ?? wallet;
}

/** Linhas da carteira em uso: Jogo Real só a `real`; Play Money as duas de fichas. */
export function ledgerForMode(
  entries: WalletLedgerEntry[],
  mode: "play" | "real",
): WalletLedgerEntry[] {
  return entries.filter((entry) => (mode === "real" ? entry.wallet === "real" : entry.wallet !== "real"));
}
