import { useEffect, useState } from "react";
import { NavLink, Navigate, Outlet } from "react-router-dom";
import { getMe, isAdminRole } from "@/lib/me";
import { isAuthenticated } from "@/lib/auth";

const subLink = ({ isActive }: { isActive: boolean }) =>
  `zt-tab ${isActive ? "zt-tab-active" : ""}`;

export function AdminLayout() {
  const [ready, setReady] = useState(false);
  const [allowed, setAllowed] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isAuthenticated()) {
      setReady(true);
      setAllowed(false);
      return;
    }
    void getMe()
      .then((me) => {
        setAllowed(isAdminRole(me?.role));
        if (me && !isAdminRole(me.role)) {
          setError("Acesso restrito a administradores.");
        }
      })
      .catch((e) => {
        setAllowed(false);
        setError(e instanceof Error ? e.message : "Falha ao verificar permissão");
      })
      .finally(() => setReady(true));
  }, []);

  if (!ready) {
    return (
      <div className="flex items-center gap-3 p-8 text-sm text-felt-300">
        <span className="zt-spinner" aria-hidden />
        Verificando acesso admin…
      </div>
    );
  }

  if (!isAuthenticated()) {
    return <Navigate to="/login" replace />;
  }

  if (!allowed) {
    return (
      <div className="zt-panel p-6 text-center">
        <h1 className="text-lg font-bold text-gold-bright">Admin</h1>
        <p className="mt-2 text-sm text-red-200">{error ?? "Sem permissão"}</p>
        <NavLink to="/" className="zt-btn-secondary mt-4 inline-flex !text-xs">
          Voltar
        </NavLink>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-xl font-bold uppercase tracking-wide text-gold-bright">
          Painel administrativo
        </h1>
        <p className="text-xs text-felt-300">Operação da plataforma · role admin</p>
      </div>
      <div className="flex flex-wrap gap-1 rounded border border-felt-600 bg-felt-950/60 p-1">
        <NavLink to="/admin" end className={subLink}>
          Overview
        </NavLink>
        <NavLink to="/admin/users" className={subLink}>
          Users
        </NavLink>
        <NavLink to="/admin/tables" className={subLink}>
          Mesas
        </NavLink>
        <NavLink to="/admin/tournaments" className={subLink}>
          Torneios
        </NavLink>
        <NavLink to="/admin/presence" className={subLink}>
          Online
        </NavLink>
        <NavLink to="/admin/clubs" className={subLink}>
          Clubes
        </NavLink>
        <NavLink to="/admin/antifraud" className={subLink}>
          Antifraud
        </NavLink>
        <NavLink to="/admin/audit" className={subLink}>
          Audit
        </NavLink>
      </div>
      <Outlet />
    </div>
  );
}
