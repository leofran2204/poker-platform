import { useCallback, useEffect, useState } from "react";
import { getOnlinePresence, sendPresenceHeartbeat } from "@/api/client";
import { isAuthenticated } from "@/lib/auth";

const POLL_MS = 12_000;
const HEARTBEAT_MS = 25_000;

type PresenceState = {
  count: number | null;
  error: boolean;
};

/** Badge compacto e bem visível no header (sempre). */
export function OnlinePresenceNav() {
  const { count, error } = usePresenceLoop();
  const label =
    count === null
      ? error
        ? "— online"
        : "… online"
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
      <span className={`zt-online-dot ${count && count > 0 ? "live" : ""}`} aria-hidden />
      <span className="zt-online-count">{label}</span>
    </div>
  );
}

/** Faixa grande na home — combina mesa + online. */
export function OnlinePresenceHero() {
  const { count, error } = usePresenceLoop();
  const n = count ?? 0;
  const ready = n >= 2;

  return (
    <div className={`zt-online-hero ${ready ? "ready" : "waiting"}`}>
      <div className="flex flex-wrap items-center justify-center gap-3">
        <span className={`zt-online-dot large ${n > 0 ? "live" : ""}`} aria-hidden />
        <p className="text-lg font-bold tracking-wide text-cream sm:text-xl">
          {count === null && !error && "Checando quem está online…"}
          {error && "Não foi possível ler presença agora"}
          {count !== null && (
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

function usePresenceLoop(): PresenceState {
  const [count, setCount] = useState<number | null>(null);
  const [error, setError] = useState(false);

  const refresh = useCallback(async () => {
    try {
      if (isAuthenticated()) {
        const hb = await sendPresenceHeartbeat();
        setCount(hb.online_count);
      } else {
        const data = await getOnlinePresence();
        setCount(data.online_count);
      }
      setError(false);
    } catch {
      setError(true);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const poll = window.setInterval(() => void refresh(), POLL_MS);
    const hb = window.setInterval(() => {
      if (isAuthenticated()) void refresh();
    }, HEARTBEAT_MS);

    const onFocus = () => void refresh();
    const onVis = () => {
      if (document.visibilityState === "visible") void refresh();
    };
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVis);

    return () => {
      window.clearInterval(poll);
      window.clearInterval(hb);
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, [refresh]);

  return { count, error };
}
