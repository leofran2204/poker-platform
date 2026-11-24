import { describe, expect, it } from "vitest";
import type { WalletLedgerEntry } from "@/api/types";
import { ledgerForMode, ledgerReasonLabel, ledgerWalletLabel } from "./walletLedger";

const entry = (id: number, wallet: WalletLedgerEntry["wallet"], reason: string): WalletLedgerEntry => ({
  id,
  wallet,
  amount_cents: 100,
  balance_after_cents: 100,
  reason,
  created_at: "2026-11-23T21:30:00Z",
});

describe("extrato do livro único", () => {
  it("nomeia os movimentos e as carteiras em português", () => {
    expect(ledgerReasonLabel("tournament_prize")).toBe("Prêmio de torneio");
    expect(ledgerReasonLabel("pix_withdraw_reversal")).toBe("Saque devolvido");
    expect(ledgerReasonLabel("algo_novo")).toBe("Movimento");
    expect(ledgerWalletLabel("pm_mtt")).toBe("Play Money · torneio");
  });

  it("mostra só as carteiras do modo em uso", () => {
    const rows = [entry(1, "real", "pix_deposit"), entry(2, "pm_cash", "cash_out"), entry(3, "pm_mtt", "tournament_fee")];
    expect(ledgerForMode(rows, "real").map((row) => row.id)).toEqual([1]);
    expect(ledgerForMode(rows, "play").map((row) => row.id)).toEqual([2, 3]);
  });
});
