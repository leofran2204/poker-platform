import { useEffect, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { ApiError } from "@/api/client";
import { isAuthenticated } from "@/lib/auth";
import { clearMeCache, getMe } from "@/lib/me";
import {
  CONNECTION_STATUS_EVENT,
  type ConnectionStatusDetail,
  SESSION_EXPIRED_EVENT,
  SESSION_RESTORED_EVENT,
} from "@/lib/sessionEvents";

type UiStatus = "online" | "offline" | "reconnecting";

function safeReturnPath(pathname: string, search: string): string {
  const path = `${pathname}${search}`;
  if (!path.startsWith("/") || path.startsWith("//") || path.startsWith("/login")) {
    return "/lobby";
  }
  return path;
}

export function SessionConnectivity() {
  const navigate = useNavigate();
  const location = useLocation();
  const mounted = useRef(true);
  const [status, setStatus] = useState<UiStatus>(() =>
    navigator.onLine ? "online" : "offline",
  );

  useEffect(() => {
    mounted.current = true;

    const markOnline = () => {
      if (mounted.current) setStatus("online");
    };

    const markOffline = () => {
      if (mounted.current) setStatus("offline");
    };

    const redirectExpiredSession = () => {
      clearMeCache();
      const returnTo = safeReturnPath(location.pathname, location.search);
      navigate(
        `/login?reason=session-expired&returnTo=${encodeURIComponent(returnTo)}`,
        { replace: true },
      );
    };

    const reconnect = async () => {
      if (!navigator.onLine) {
        markOffline();
        return;
      }

      if (!isAuthenticated()) {
        markOnline();
        return;
      }

      if (mounted.current) setStatus("reconnecting");
      try {
        clearMeCache();
        await getMe(true);
        markOnline();
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) return;
        if (isAuthenticated()) markOffline();
      }
    };

    const onNativeOnline = () => void reconnect();
    const onConnectionStatus = (event: Event) => {
      const { status: next } = (event as CustomEvent<ConnectionStatusDetail>).detail;
      if (next === "online") markOnline();
      else markOffline();
    };

    window.addEventListener("online", onNativeOnline);
    window.addEventListener("offline", markOffline);
    window.addEventListener(CONNECTION_STATUS_EVENT, onConnectionStatus);
    window.addEventListener(SESSION_EXPIRED_EVENT, redirectExpiredSession);
    window.addEventListener(SESSION_RESTORED_EVENT, markOnline);

    return () => {
      mounted.current = false;
      window.removeEventListener("online", onNativeOnline);
      window.removeEventListener("offline", markOffline);
      window.removeEventListener(CONNECTION_STATUS_EVENT, onConnectionStatus);
      window.removeEventListener(SESSION_EXPIRED_EVENT, redirectExpiredSession);
      window.removeEventListener(SESSION_RESTORED_EVENT, markOnline);
    };
  }, [location.pathname, location.search, navigate]);

  if (status === "online") return null;

  const reconnecting = status === "reconnecting";

  return (
    <div
      className="fixed inset-x-0 top-0 z-[60] border-b border-amber-500/70 bg-amber-950 px-4 py-2 text-amber-50 shadow-xl"
      role="status"
      aria-live="assertive"
    >
      <div className="mx-auto flex max-w-6xl items-center justify-between gap-3 text-sm">
        <div>
          <strong>
            {reconnecting ? "Reconectando à plataforma…" : "Sem conexão com a plataforma"}
          </strong>
          <span className="ml-2 text-amber-100/90">
            {reconnecting
              ? "Estamos validando sua sessão automaticamente."
              : "Você não precisa sair: tentaremos reconectar assim que a internet voltar."}
          </span>
        </div>
        <span className="shrink-0 text-xs font-semibold uppercase tracking-wide">
          {reconnecting ? "Validando sessão" : "Aguardando internet"}
        </span>
      </div>
      <div className="absolute inset-x-0 bottom-0 h-1 overflow-hidden bg-amber-950">
        <span className="zt-reconnect-progress absolute inset-y-0 block w-1/3 bg-gold-bright" />
      </div>
    </div>
  );
}
