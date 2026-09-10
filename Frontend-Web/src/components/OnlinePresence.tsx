import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { getOnlinePresence, sendPresenceHeartbeat, sendPresenceOffline } from "@/api/client";
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

/** Badge compacto no header — visitante (GET público) e logado (heartbeat). */
export function OnlinePresenceNav() {
  const authed = useAuthed();
  const { count, error } = usePresenceLoop(authed);
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

/** Faixa na home — GET público para visitante, heartbeat se logado. */
export function OnlinePresenceHero() {
  const authed = useAuthed();
  const { count, error } = usePresenceLoop(authed);
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

type PresenceListener = (state: PresenceState) => void;

let shared: PresenceState = { count: null, error: false };
const listeners = new Set<PresenceListener>();
let subscribers = 0;
let pollTimer: number | null = null;
let hbTimer: number | null = null;
let lastAuthed = false;

function emitPresence(next: PresenceState): void {
  shared = next;
  listeners.forEach((fn) => fn(next));
}

async function refreshPresence(authed: boolean): Promise<void> {
  try {
    if (authed) {
      const hb = await sendPresenceHeartbeat();
      emitPresence({ count: hb.online_count, error: false });
    } else {
      const pub = await getOnlinePresence();
      emitPresence({ count: pub.online_count, error: false });
    }
  } catch {
    emitPresence({ count: null, error: true });
  }
}

function onFocus(): void {
  void refreshPresence(lastAuthed);
}
function onVis(): void {
  if (document.visibilityState === "visible") void refreshPresence(lastAuthed);
}
function onPageHide(): void {
  if (isAuthenticated()) void sendPresenceOffline().catch(() => {});
}
function onPresenceCount(event: Event): void {
  const detail = (event as CustomEvent<PresenceCountEventDetail>).detail;
  if (detail.kind === "logout") {
    emitPresence({
      count: Math.max(0, (shared.count ?? 1) - 1),
      error: false,
    });
  } else {
    emitPresence({ count: detail.onlineCount, error: false });
  }
}

function startPresenceEngine(authed: boolean): void {
  lastAuthed = authed;
  void refreshPresence(authed);
  if (pollTimer == null) {
    pollTimer = window.setInterval(() => void refreshPresence(lastAuthed), POLL_MS);
    window.addEventListener("focus", onFocus);
    window.addEventListener(PRESENCE_COUNT_EVENT, onPresenceCount);
    document.addEventListener("visibilitychange", onVis);
    window.addEventListener("pagehide", onPageHide);
  }
  if (authed && hbTimer == null) {
    hbTimer = window.setInterval(() => void refreshPresence(true), HEARTBEAT_MS);
  }
  if (!authed && hbTimer != null) {
    window.clearInterval(hbTimer);
    hbTimer = null;
  }
}

function stopPresenceEngine(): void {
  if (pollTimer != null) {
    window.clearInterval(pollTimer);
    pollTimer = null;
    window.removeEventListener("focus", onFocus);
    window.removeEventListener(PRESENCE_COUNT_EVENT, onPresenceCount);
    document.removeEventListener("visibilitychange", onVis);
    window.removeEventListener("pagehide", onPageHide);
  }
  if (hbTimer != null) {
    window.clearInterval(hbTimer);
    hbTimer = null;
  }
}

function usePresenceLoop(authed: boolean): PresenceState {
  const [state, setState] = useState<PresenceState>(shared);

  useEffect(() => {
    const onState: PresenceListener = (next) => setState(next);
    listeners.add(onState);
    subscribers += 1;
    startPresenceEngine(authed);

    return () => {
      listeners.delete(onState);
      subscribers -= 1;
      if (subscribers <= 0) {
        subscribers = 0;
        stopPresenceEngine();
      }
    };
  }, [authed]);

  return state;
}
