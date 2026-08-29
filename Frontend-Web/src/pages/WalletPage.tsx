import { FormEvent, useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  createDepositRequest,
  fetchDepositInfo,
  fetchMe,
  listMyDepositRequests,
} from "@/api/client";
import type { DepositInfoResponse, DepositRequestResponse, MeResponse } from "@/api/types";
import { NoIndex } from "@/components/NoIndex";
import { isAuthenticated } from "@/lib/auth";
import { formatBrlFromCents } from "@/lib/money";

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

  const maskedKey = useMemo(
    () => (info?.pix_key ? maskPixKey(info.pix_key) : ""),
    [info?.pix_key],
  );

  const load = useCallback(async () => {
    if (!isAuthenticated()) return;
    setError(null);
    try {
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
          Pedido de fichas · PIX manual no banco · crédito após aprovação
        </p>
      </div>

      {error && (
        <p className="rounded border border-red-800 bg-red-950/40 px-3 py-2 text-sm text-red-200">
          {error}
        </p>
      )}
      {msg && (
        <p className="rounded border border-emerald-800 bg-emerald-950/30 px-3 py-2 text-sm text-emerald-100">
          {msg}
        </p>
      )}

      <div className="zt-panel p-4">
        <div className="text-[10px] font-bold uppercase tracking-wider text-felt-400">Saldo</div>
        <div className="mt-1 font-mono text-2xl text-gold-bright">
          {me ? formatBrlFromCents(me.balance) : "…"}
        </div>
        <p className="mt-2 text-xs text-felt-400">
          Play-money / operação manual — não é gateway automático de PSP.
        </p>
      </div>

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
                <p className="text-[11px] text-felt-500">
                  Não compartilhe print desta tela. Confira o nome do recebedor no app do banco
                  antes de confirmar o PIX.
                </p>
                <div className="rounded border border-rail/60 bg-felt-950 p-3">
                  <div className="text-[10px] uppercase text-felt-400">Recebedor</div>
                  <div className="font-semibold text-cream">{info.receiver_name}</div>
                  <div className="mt-2 text-[10px] uppercase text-felt-400">Chave PIX</div>
                  <div className="break-all font-mono text-sm text-gold-soft">
                    {revealPix ? info.pix_key : maskedKey}
                  </div>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <button
                      type="button"
                      className="zt-btn-secondary !py-1 !text-xs"
                      onClick={() => setRevealPix((v) => !v)}
                    >
                      {revealPix ? "Ocultar" : "Mostrar chave"}
                    </button>
                    <button
                      type="button"
                      className="zt-btn-secondary !py-1 !text-xs"
                      onClick={() => void copyPix()}
                    >
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
                          className={
                            amountCents === p
                              ? "zt-tab zt-tab-active !flex-none"
                              : "zt-tab !flex-none"
                          }
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
                      onChange={(e) =>
                        setAmountCents(Math.round(Number(e.target.value || 0) * 100))
                      }
                    />
                    <p className="mt-1 text-[11px] text-felt-500">
                      Máx. {formatBrlFromCents(info.max_cents)} por pedido
                    </p>
                  </div>
                  <div>
                    <label className="zt-label" htmlFor="proof">
                      Comprovante (obrigatório)
                    </label>
                    <textarea
                      id="proof"
                      className="zt-input min-h-[100px]"
                      placeholder="Cole protocolo E2E, ID da transação ou texto do comprovante PIX"
                      value={proof}
                      onChange={(e) => setProof(e.target.value)}
                      required
                      minLength={8}
                      maxLength={4000}
                    />
                  </div>
                  <div>
                    <label className="zt-label" htmlFor="note">
                      Observação (opcional)
                    </label>
                    <input
                      id="note"
                      className="zt-input"
                      value={note}
                      onChange={(e) => setNote(e.target.value)}
                      maxLength={500}
                      placeholder="Nome no extrato, horário do PIX…"
                    />
                  </div>
                  <button type="submit" className="zt-btn-primary" disabled={busy}>
                    {busy ? "Enviando…" : "Enviar pedido para verificação"}
                  </button>
                </form>
              </>
            )}
          </div>
        )}
      </div>

      <div className="zt-panel overflow-hidden">
        <div className="zt-panel-title">Meus pedidos</div>
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
                  <td colSpan={4} className="text-felt-400">
                    Nenhum pedido ainda
                  </td>
                </tr>
              ) : (
                requests.map((r) => (
                  <tr key={r.id} className="!cursor-default align-top">
                    <td className="text-xs text-felt-300">
                      {new Date(r.created_at).toLocaleString("pt-BR")}
                    </td>
                    <td className="font-mono text-gold-soft">
                      {formatBrlFromCents(r.amount_cents)}
                    </td>
                    <td className="font-mono text-xs text-cream">{r.status}</td>
                    <td className="text-xs text-felt-400">{r.admin_note || "—"}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
