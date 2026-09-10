import { useCallback, useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { sendPresenceHeartbeat, sendPresenceOffline } from "@/api/client";
import { isAuthenticated } from "@/lib/auth";

const POLL_MS = 12_000;
const HEARTBEAT_MS = 25_000;
const PRESENCE_COUNT_EVENT = "poker-presence-count";

type PresenceCountEventDetail =
  | { kind: "logout" }
  | { kind: "server"; onlineCount: number };

export function reflectPresenceLogout(): void {
  window.dispatchEvent(
    new CustomEvent<PresenceCountEventDetail>(PRESENCE_COUNT_EVENT, {
      detail: { kind: "logout" },
    }),
  );
}

export function reflectPresenceCount(onlineCount: number): void {
  window.dispatchEvent(
    new CustomEvent<PresenceCountEventDetail>(PRESENCE_COUNT_EVENT, {
      detail: { kind: "server", onlineCount },
    }),
  );
}

type PresenceState = {
  count: number | null;
  error: boolean;
};

/**
 * true somente com sessão. Lê o token a cada render e força re-render
 * a cada navegação (ex.: pós-login/logout), foco ou mudança em outra aba.
 */
function useAuthed(): boolean {
  useLocation();
  const [, setTick] = useState(0);
  useEffect(() => {
    const sync = () => setTick((t) => t + 1);
    window.addEventListener("focus", sync);
    window.addEventListener("storage", sync);
    return () => {
      window.removeEventListener("focus", sync);
      window.removeEventListener("storage", sync);
    };
  }, []);
  return isAuthenticated();
}

/** Badge compacto no header — somente para logados. */
export function OnlinePresenceNav() {
  const authed = useAuthed();
  const { count, error } = usePresenceLoop(authed);
  if (!authed) return null;
  const label = error
    ? "offline"
    : count === null
      ? "… online"
      : count === 1
        ? "1 online"
        : `${count} online`;

  return (
    <div
      className="zt-online-badge"
      title="Pessoas logadas com presença ativa na plataforma (últimos ~90s)"
      role="status"
      aria-live="polite"
    >
      <span
        className={`zt-online-dot ${!error && count && count > 0 ? "live" : ""}`}
        aria-hidden
      />
      <span className="zt-online-count">{label}</span>
    </div>
  );
}

/** Faixa grande na home — combina mesa + online. Somente para logados. */
export function OnlinePresenceHero() {
  const authed = useAuthed();
  const { count, error } = usePresenceLoop(authed);
  if (!authed) return null;
  const n = count ?? 0;
  const ready = !error && n >= 2;

  return (
    <div className={`zt-online-hero ${ready ? "ready" : "waiting"}`}>
      <div className="flex flex-wrap items-center justify-center gap-3">
        <span
          className={`zt-online-dot large ${!error && n > 0 ? "live" : ""}`}
          aria-hidden
        />
        <p className="text-lg font-bold tracking-wide text-cream sm:text-xl">
          {count === null && !error && "Checando quem está online…"}
          {error && "Não foi possível ler presença agora"}
          {count !== null && !error && (
            <>
              <span className="text-gold-bright">{n}</span>
              {n === 1 ? " pessoa online" : " pessoas online"}
              <span className="font-semibold text-felt-200"> agora</span>
            </>
          )}
        </p>
      </div>
      <p className="mt-2 text-center text-sm text-felt-200">
        {ready
          ? "Tem gente o bastante — combinem a MESMA mesa no lobby (mín. 2 assentos) para rodar mão."
          : "Poker precisa de pelo menos 2 na mesma mesa. Avise o grupo e entrem juntos."}
      </p>
    </div>
  );
}

function usePresenceLoop(authed: boolean): PresenceState {
  const [count, setCount] = useState<number | null>(null);
  const [error, setError] = useState(false);

  const refresh = useCallback(async () => {
    if (!authed) {
      setCount(null);
      setError(false);
      return;
    }
    try {
      const hb = await sendPresenceHeartbeat();
      setCount(hb.online_count);
      setError(false);
    } catch {
      setCount(null);
      setError(true);
    }
  }, [authed]);

  useEffect(() => {
    if (!authed) return;
    void refresh();
    const poll = window.setInterval(() => void refresh(), POLL_MS);
    const hb = window.setInterval(() => void refresh(), HEARTBEAT_MS);

    const onFocus = () => void refresh();
    const onVis = () => {
      if (document.visibilityState === "visible") void refresh();
    };
    // Fecha a aba sem "Sair": sai da conta na hora em vez de lingerar ~90s de TTL.
    const onPageHide = () => {
      if (isAuthenticated()) void sendPresenceOffline().catch(() => {});
    };
    const onPresenceCount = (event: Event) => {
      const detail = (event as CustomEvent<PresenceCountEventDetail>).detail;
      if (detail.kind === "logout") {
        setCount((current) => Math.max(0, (current ?? 1) - 1));
      } else {
        setCount(detail.onlineCount);
      }
      setError(false);
    };
    window.addEventListener("focus", onFocus);
    window.addEventListener(PRESENCE_COUNT_EVENT, onPresenceCount);
    document.addEventListener("visibilitychange", onVis);
    window.addEventListener("pagehide", onPageHide);

    return () => {
      window.clearInterval(poll);
      window.clearInterval(hb);
      window.removeEventListener("focus", onFocus);
      window.removeEventListener(PRESENCE_COUNT_EVENT, onPresenceCount);
      document.removeEventListener("visibilitychange", onVis);
      window.removeEventListener("pagehide", onPageHide);
    };
  }, [refresh, authed]);

  return { count, error };
}
