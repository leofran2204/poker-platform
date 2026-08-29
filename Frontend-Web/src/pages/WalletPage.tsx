import { FormEvent, useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  createDepositRequest,
  fetchDepositInfo,
  fetchMe,
  listMyDepositRequests,
  pmRebuy,
} from "@/api/client";
import type { DepositInfoResponse, DepositRequestResponse, MeResponse } from "@/api/types";
import { NoIndex } from "@/components/NoIndex";
import { isAuthenticated } from "@/lib/auth";
import { clearMeCache } from "@/lib/me";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode } from "@/lib/walletMode";

function maskPixKey(key: string): string {
  const k = key.trim();
  if (k.length <= 10) return "••••••••";
  return `${k.slice(0, 4)}••••••••${k.slice(-4)}`;
}

export function WalletPage() {
  const [me, setMe] = useState<MeResponse | null>(null);
  const [info, setInfo] = useState<DepositInfoResponse | null>(null);
  const [requests, setRequests] = useState<DepositRequestResponse[]>([]);
  const [openForm, setOpenForm] = useState(false);
  const [revealPix, setRevealPix] = useState(false);
  const [amountCents, setAmountCents] = useState(100_000);
  const [proof, setProof] = useState("");
  const [note, setNote] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const mode = getWalletMode();

  const maskedKey = useMemo(
    () => (info?.pix_key ? maskPixKey(info.pix_key) : ""),
    [info?.pix_key],
  );

  const load = useCallback(async () => {
    if (!isAuthenticated()) return;
    setError(null);
    try {
      clearMeCache();
      const [m, i, r] = await Promise.all([
        fetchMe(),
        fetchDepositInfo(),
        listMyDepositRequests(),
      ]);
      setMe(m);
      setInfo(i);
      setRequests(r);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao carregar carteira");
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  if (!isAuthenticated()) {
    return (
      <div className="zt-panel p-8 text-center">
        <h1 className="text-xl font-bold text-gold-bright">Carteira</h1>
        <p className="mt-2 text-felt-300">Faça login para ver saldo e pedir fichas.</p>
        <Link to="/login" className="zt-btn-primary mt-4 inline-flex">
          Entrar
        </Link>
      </div>
    );
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    setMsg(null);
    try {
      await createDepositRequest({
        amount_cents: amountCents,
        proof_text: proof,
        player_note: note || undefined,
      });
      setMsg("Pedido enviado. Aguarde a verificação do comprovante.");
      setProof("");
      setNote("");
      setOpenForm(false);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha ao enviar pedido");
    } finally {
      setBusy(false);
    }
  }

  async function onRebuy(kind: "cash" | "mtt") {
    setBusy(true);
    setError(null);
    setMsg(null);
    try {
      await pmRebuy(kind);
      setMsg(kind === "cash" ? "Rebuy cash PM creditado (R$ 1.000)." : "Rebuy torneio PM creditado (R$ 15.000).");
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha no rebuy");
    } finally {
      setBusy(false);
    }
  }

  async function copyPix() {
    if (!info?.pix_key) return;
    try {
      await navigator.clipboard.writeText(info.pix_key);
      setMsg("Chave PIX copiada.");
    } catch {
      setMsg(`Chave: ${info.pix_key}`);
    }
  }

  return (
    <div className="space-y-4">
      <NoIndex />
      <div>
        <h1 className="text-xl font-bold uppercase tracking-wide text-gold-bright">Carteira</h1>
        <p className="text-xs text-felt-300">
          Modo ativo no header: <span className="text-gold-soft">{mode === "real" ? "Jogo Real" : "Play Money"}</span>
        </p>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">{error}</p>
      )}
      {msg && (
        <p className="rounded border border-emerald-800 bg-emerald-950/30 px-3 py-2 text-sm text-emerald-100">{msg}</p>
      )}

      <div className="grid gap-3 sm:grid-cols-3">
        <div className="zt-card p-4">
          <div className="text-[10px] uppercase text-felt-400">Play Money · Cash</div>
          <div className="mt-1 font-mono text-xl text-gold-bright">
            {me ? formatBrlFromCents(me.balance_pm_cash ?? me.balance) : "…"}
          </div>
          {me?.pm_cash_rebuy_available && (
            <button
              type="button"
              className="zt-btn-secondary mt-2 !py-1 !text-xs"
              disabled={busy}
              onClick={() => void onRebuy("cash")}
            >
              Rebuy R$ 1.000 (1×/dia)
            </button>
          )}
        </div>
        <div className="zt-card p-4">
          <div className="text-[10px] uppercase text-felt-400">Play Money · Torneio</div>
          <div className="mt-1 font-mono text-xl text-gold-bright">
            {me ? formatBrlFromCents(me.balance_pm_mtt ?? 0) : "…"}
          </div>
          {me?.pm_mtt_rebuy_available && (
            <button
              type="button"
              className="zt-btn-secondary mt-2 !py-1 !text-xs"
              disabled={busy}
              onClick={() => void onRebuy("mtt")}
            >
              Rebuy R$ 15.000 (1×/dia)
            </button>
          )}
        </div>
        <div className="zt-card p-4">
          <div className="text-[10px] uppercase text-felt-400">Jogo Real</div>
          <div className="mt-1 font-mono text-xl text-gold-bright">
            {me ? formatBrlFromCents(me.balance_real ?? 0) : "…"}
          </div>
        </div>
      </div>

      <p className="text-[11px] text-felt-500">
        Play Money renova todo dia à meia-noite (Brasília) para R$ 1.000 (cash) e R$ 15.000 (torneio).
        Se zerar, pode fazer 1 rebuy por carteira por dia.
      </p>

      {mode === "real" ? (
        <div className="zt-panel p-4 space-y-3">
          <button
            type="button"
            className="zt-btn-primary"
            disabled={!info}
            onClick={() => {
              setOpenForm((v) => !v);
              setRevealPix(false);
            }}
          >
            {openForm ? "Fechar" : "Pedir fichas"}
          </button>

          {openForm && info && (
            <div className="space-y-3 border-t border-felt-600 pt-3">
              {!info.available ? (
                <p className="text-sm text-amber-100">{info.instructions}</p>
              ) : (
                <>
                  <p className="text-xs text-felt-300">{info.instructions}</p>
                  <div className="rounded border border-rail/60 bg-felt-950 p-3">
                    <div className="text-[10px] uppercase text-felt-400">Recebedor</div>
                    <div className="font-semibold text-cream">{info.receiver_name}</div>
                    <div className="mt-2 text-[10px] uppercase text-felt-400">Chave PIX</div>
                    <div className="break-all font-mono text-sm text-gold-soft">
                      {revealPix ? info.pix_key : maskedKey}
                    </div>
                    <div className="mt-2 flex flex-wrap gap-1">
                      <button type="button" className="zt-btn-secondary !py-1 !text-xs" onClick={() => setRevealPix((v) => !v)}>
                        {revealPix ? "Ocultar" : "Mostrar chave"}
                      </button>
                      <button type="button" className="zt-btn-secondary !py-1 !text-xs" onClick={() => void copyPix()}>
                        Copiar chave
                      </button>
                    </div>
                  </div>
                  <form className="space-y-3" onSubmit={(e) => void onSubmit(e)}>
                    <div>
                      <div className="zt-label">Valor</div>
                      <div className="mb-2 flex flex-wrap gap-1">
                        {info.presets_cents.map((p) => (
                          <button
                            key={p}
                            type="button"
                            className={amountCents === p ? "zt-tab zt-tab-active !flex-none" : "zt-tab !flex-none"}
                            onClick={() => setAmountCents(p)}
                          >
                            {formatBrlFromCents(p)}
                          </button>
                        ))}
                      </div>
                      <input
                        type="number"
                        min={1}
                        step={1}
                        className="zt-input max-w-xs"
                        value={(amountCents / 100).toFixed(2)}
                        onChange={(e) => setAmountCents(Math.round(Number(e.target.value || 0) * 100))}
                      />
                    </div>
                    <div>
                      <label className="zt-label" htmlFor="proof">Comprovante (obrigatório)</label>
                      <textarea
                        id="proof"
                        className="zt-input min-h-[100px]"
                        placeholder="Cole protocolo E2E / texto do comprovante PIX"
                        value={proof}
                        onChange={(e) => setProof(e.target.value)}
                        required
                        minLength={8}
                        maxLength={4000}
                      />
                    </div>
                    <div>
                      <label className="zt-label" htmlFor="note">Observação (opcional)</label>
                      <input id="note" className="zt-input" value={note} onChange={(e) => setNote(e.target.value)} maxLength={500} />
                    </div>
                    <button type="submit" className="zt-btn-primary" disabled={busy}>
                      {busy ? "Enviando…" : "Enviar pedido para verificação"}
                    </button>
                  </form>
                </>
              )}
            </div>
          )}

          <div className="zt-panel overflow-hidden !border-0 !shadow-none">
            <div className="zt-panel-title">Meus pedidos (Jogo Real)</div>
            <div className="zt-table-wrap">
              <table className="zt-lobby-table">
                <thead>
                  <tr>
                    <th>Quando</th>
                    <th>Valor</th>
                    <th>Status</th>
                    <th>Admin</th>
                  </tr>
                </thead>
                <tbody>
                  {requests.length === 0 ? (
                    <tr className="!cursor-default">
                      <td colSpan={4} className="text-felt-400">Nenhum pedido ainda</td>
                    </tr>
                  ) : (
                    requests.map((r) => (
                      <tr key={r.id} className="!cursor-default">
                        <td className="text-xs text-felt-300">{new Date(r.created_at).toLocaleString("pt-BR")}</td>
                        <td className="font-mono text-gold-soft">{formatBrlFromCents(r.amount_cents)}</td>
                        <td className="font-mono text-xs">{r.status}</td>
                        <td className="text-xs text-felt-400">{r.admin_note || "—"}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      ) : (
        <div className="zt-panel p-4 text-sm text-felt-300">
          Em <strong className="text-cream">Play Money</strong> não há compra. As fichas renovam diariamente.
          Para depositar via PIX, mude o header para <strong className="text-cream">Jogo Real</strong>.
        </div>
      )}
    </div>
  );
}
