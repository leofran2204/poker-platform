import { useCallback, useEffect, useState } from "react";
import {
  approveDepositRequest,
  listAdminDepositRequests,
  rejectDepositRequest,
} from "@/api/client";
import type { DepositRequestResponse } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

export function AdminDepositsPage() {
  const [rows, setRows] = useState<DepositRequestResponse[]>([]);
  const [status, setStatus] = useState("pending");
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      setRows(
        await listAdminDepositRequests({
          status: status || undefined,
        }),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }, [status]);

  useEffect(() => {
    void load();
  }, [load]);

  async function onApprove(id: string) {
    if (!window.confirm("Confirmar PIX no extrato e creditar fichas?")) return;
    setMsg(null);
    try {
      await approveDepositRequest(id);
      setMsg("Pedido aprovado e saldo creditado.");
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao aprovar");
    }
  }

  async function onReject(id: string) {
    const note = window.prompt("Motivo da rejeição (opcional):") ?? "";
    if (note === null) return;
    setMsg(null);
    try {
      await rejectDepositRequest(id, note);
      setMsg("Pedido rejeitado.");
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro ao rejeitar");
    }
  }

  return (
    <div className="space-y-3">
      <div className="zt-lobby-toolbar">
        <select
          className="zt-input w-44"
          value={status}
          onChange={(e) => setStatus(e.target.value)}
        >
          <option value="pending">pending</option>
          <option value="approved">approved</option>
          <option value="rejected">rejected</option>
          <option value="">todos</option>
        </select>
        <button type="button" className="zt-btn-secondary !py-1 !text-xs" onClick={() => void load()}>
          Atualizar
        </button>
      </div>
      {error && <p className="text-sm text-red-200">{error}</p>}
      {msg && <p className="text-sm text-emerald-200">{msg}</p>}

      <div className="space-y-3">
        {rows.length === 0 ? (
          <p className="text-sm text-felt-400">Nenhum pedido.</p>
        ) : (
          rows.map((r) => (
            <div key={r.id} className="zt-panel p-4 space-y-2">
              <div className="flex flex-wrap items-start justify-between gap-2">
                <div>
                  <div className="font-semibold text-cream">
                    {r.username ?? r.user_id} · {formatBrlFromCents(r.amount_cents)}
                  </div>
                  <div className="text-xs text-felt-400">
                    {new Date(r.created_at).toLocaleString("pt-BR")} ·{" "}
                    <span className="font-mono text-gold-soft">{r.status}</span>
                  </div>
                </div>
                {r.status === "pending" && (
                  <div className="flex gap-1">
                    <button
                      type="button"
                      className="zt-btn-primary !py-1 !text-xs"
                      onClick={() => void onApprove(r.id)}
                    >
                      Aprovar
                    </button>
                    <button
                      type="button"
                      className="zt-btn-danger !py-1 !text-xs"
                      onClick={() => void onReject(r.id)}
                    >
                      Rejeitar
                    </button>
                  </div>
                )}
              </div>
              {r.player_note && (
                <p className="text-xs text-felt-300">Nota: {r.player_note}</p>
              )}
              <div>
                <div className="text-[10px] uppercase text-felt-500">Comprovante</div>
                <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-words rounded border border-felt-600 bg-felt-950 p-2 text-xs text-felt-200">
                  {r.proof_text}
                </pre>
              </div>
              {r.admin_note && (
                <p className="text-xs text-felt-400">Admin: {r.admin_note}</p>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
