import { FormEvent, useCallback, useEffect, useState } from "react";
import { adjustUserBalance, listAdminUsers, patchAdminUser } from "@/api/client";
import type { AdminUserResponse } from "@/api/types";
import { formatBrlFromCents } from "@/lib/money";

export function AdminUsersPage() {
  const [users, setUsers] = useState<AdminUserResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [q, setQ] = useState("");
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const res = await listAdminUsers({
        q: q || undefined,
        status: status || undefined,
        limit: 100,
      });
      setUsers(res.users);
      setTotal(res.total);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }, [q, status]);

  useEffect(() => {
    void load();
  }, [load]);

  async function onStatus(id: string, next: string) {
    if (!window.confirm(`Alterar status para ${next}?`)) return;
    setMsg(null);
    try {
      await patchAdminUser(id, { status: next });
      setMsg(`Status → ${next}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }

  async function onRole(id: string, next: string) {
    if (!window.confirm(`Alterar role para ${next}?`)) return;
    setMsg(null);
    try {
      await patchAdminUser(id, { role: next });
      setMsg(`Role → ${next}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erro");
    }
  }

  async function onAdjust(e: FormEvent<HTMLFormElement>, id: string) {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const reais = Number(fd.get("reais"));
    const reason = String(fd.get("reason") || "");
    if (!Number.isFinite(reais) || reais === 0) return;
    if (!window.confirm(`Ajustar saldo em R$ ${reais.toFixed(2)}?`)) return;
    setMsg(null);
    try {
      const res = await adjustUserBalance(id, Math.round(reais * 100), reason);
      setMsg(`Novo saldo: ${formatBrlFromCents(res.balance)}`);
      e.currentTarget.reset();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erro");
    }
  }

  return (
    <div className="space-y-3">
      <div className="zt-lobby-toolbar">
        <input
          className="zt-input max-w-xs"
          placeholder="Buscar user/email"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        <select className="zt-input w-44" value={status} onChange={(e) => setStatus(e.target.value)}>
          <option value="">Todos status</option>
          <option value="active">active</option>
          <option value="suspended">suspended</option>
          <option value="banned">banned</option>
          <option value="pending_email_verification">pending_email_verification</option>
        </select>
        <button type="button" className="zt-btn-secondary !py-1 !text-xs" onClick={() => void load()}>
          Filtrar ({total})
        </button>
      </div>
      {error && <p className="text-sm text-red-200">{error}</p>}
      {msg && <p className="text-sm text-emerald-200">{msg}</p>}
      <div className="zt-table-wrap zt-panel overflow-hidden">
        <table className="zt-lobby-table">
          <thead>
            <tr>
              <th>User</th>
              <th>Role</th>
              <th>Status</th>
              <th>Saldo</th>
              <th>E-mail</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {users.map((u) => (
              <tr key={u.id} className="!cursor-default align-top">
                <td>
                  <div className="font-semibold text-cream">{u.username}</div>
                  <div className="text-[11px] text-felt-400">{u.email}</div>
                </td>
                <td>
                  <select
                    className="zt-input !py-1 text-xs"
                    value={u.role}
                    onChange={(e) => void onRole(u.id, e.target.value)}
                  >
                    <option value="player">player</option>
                    <option value="moderator">moderator</option>
                    <option value="admin">admin</option>
                  </select>
                </td>
                <td className="text-xs">
                  <div className="font-mono text-cream">{u.status}</div>
                  <div className="mt-1 flex flex-wrap gap-1">
                    <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void onStatus(u.id, "active")}>
                      active
                    </button>
                    <button type="button" className="zt-btn-secondary !px-2 !py-0.5 !text-[10px]" onClick={() => void onStatus(u.id, "suspended")}>
                      suspend
                    </button>
                    <button type="button" className="zt-btn-danger !px-2 !py-0.5 !text-[10px]" onClick={() => void onStatus(u.id, "banned")}>
                      ban
                    </button>
                  </div>
                </td>
                <td className="font-mono text-gold-soft">{formatBrlFromCents(u.balance)}</td>
                <td className="text-xs text-felt-300">{u.email_verified ? "ok" : "não"}</td>
                <td>
                  <form className="flex flex-wrap items-end gap-1" onSubmit={(e) => void onAdjust(e, u.id)}>
                    <input name="reais" type="number" step="0.01" placeholder="± R$" className="zt-input !w-20 !py-1 text-xs" required />
                    <input name="reason" placeholder="motivo" className="zt-input !w-28 !py-1 text-xs" required maxLength={200} />
                    <button type="submit" className="zt-btn-primary !px-2 !py-1 !text-[10px]">
                      Ajustar
                    </button>
                  </form>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
