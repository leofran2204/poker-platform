import { useCallback, useEffect, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router-dom";
import { sendPresenceOffline, setWalletMode as apiSetWalletMode } from "@/api/client";
import type { MeResponse, WalletMode } from "@/api/types";
import {
  OnlinePresenceNav,
  reflectPresenceCount,
  reflectPresenceLogout,
} from "@/components/OnlinePresence";
import { SessionConnectivity } from "@/components/SessionConnectivity";
import { clearTokens, getUsername, isAuthenticated } from "@/lib/auth";
import { clearMeCache, getMe, isAdminRole } from "@/lib/me";
import { formatBrlFromCents } from "@/lib/money";
import { SESSION_EXPIRED_EVENT, SESSION_RESTORED_EVENT } from "@/lib/sessionEvents";
import { getWalletMode, setWalletModeLocal } from "@/lib/walletMode";

const linkClass = ({ isActive }: { isActive: boolean }) =>
  `zt-nav-link ${isActive ? "zt-nav-link-active" : ""}`;

const MARKETING_PATHS = new Set(["/", "/login", "/register", "/verify-email"]);

export function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const [authTick, setAuthTick] = useState(0);
  const authed = isAuthenticated();
  const username = getUsername();
  const marketingShell = !authed && MARKETING_PATHS.has(location.pathname);
  const homeBleed = location.pathname === "/" || MARKETING_PATHS.has(location.pathname);
  const wideMain =
    location.pathname.startsWith("/admin") || location.pathname.startsWith("/table");
  const [isAdmin, setIsAdmin] = useState(false);
  const [me, setMe] = useState<MeResponse | null>(null);
  const [mode, setMode] = useState<WalletMode>(getWalletMode());
  void authTick;

  const refreshMe = useCallback(async () => {
    if (!authed) {
      setMe(null);
      setIsAdmin(false);
      return;
    }
    try {
      const profile = await getMe(true);
      setMe(profile);
      setIsAdmin(isAdminRole(profile?.role));
      if (profile?.preferred_wallet_mode === "real" || profile?.preferred_wallet_mode === "play") {
        const m = profile.preferred_wallet_mode as WalletMode;
        setMode(m);
        setWalletModeLocal(m);
      }
    } catch {
      setIsAdmin(false);
    }
  }, [authed]);

  useEffect(() => {
    void refreshMe();
  }, [refreshMe]);

  useEffect(() => {
    const bump = () => setAuthTick((n) => n + 1);
    const onWallet = (e: Event) => {
      const next = (e as CustomEvent<WalletMode>).detail;
      if (next === "play" || next === "real") setMode(next);
    };
    window.addEventListener("storage", bump);
    window.addEventListener(SESSION_EXPIRED_EVENT, bump);
    window.addEventListener(SESSION_RESTORED_EVENT, bump);
    window.addEventListener("wallet-mode-changed", onWallet);
    return () => {
      window.removeEventListener("storage", bump);
      window.removeEventListener(SESSION_EXPIRED_EVENT, bump);
      window.removeEventListener(SESSION_RESTORED_EVENT, bump);
      window.removeEventListener("wallet-mode-changed", onWallet);
    };
  }, []);

  async function switchMode(next: WalletMode) {
    const previous = mode;
    setMode(next);
    setWalletModeLocal(next);
    window.dispatchEvent(new CustomEvent("wallet-mode-changed", { detail: next }));
    try {
      await apiSetWalletMode(next);
      clearMeCache();
      await refreshMe();
    } catch {
      setMode(previous);
      setWalletModeLocal(previous);
      window.dispatchEvent(new CustomEvent("wallet-mode-changed", { detail: previous }));
    }
  }

  function handleLogout() {
    reflectPresenceLogout();
    void sendPresenceOffline()
      .then((presence) => reflectPresenceCount(presence.online_count))
      .catch(() => {
        /* O contador já foi reduzido localmente; o TTL cobre falhas de rede. */
      });
    clearTokens();
    clearMeCache();
    setIsAdmin(false);
    setMe(null);
    navigate("/login");
  }

  return (
    <div className="zt-shell">
      <SessionConnectivity />
      <header className="zt-nav">
        <div className="zt-nav-inner">
          <div className="flex flex-wrap items-center gap-3">
            <NavLink to="/" className="zt-brand">
              <span className="text-cream" aria-hidden>
                ♠
              </span>
              Zero Tilt
            </NavLink>
            <OnlinePresenceNav />
          </div>
          <nav className="flex flex-wrap items-center gap-3 sm:gap-4">
            {!marketingShell && (
            <>
            <NavLink to="/lobby" className={linkClass}>
              Lobby
            </NavLink>
            <NavLink to="/curso" className={linkClass}>
              Curso
            </NavLink>
            </>
            )}
            {authed && (
              <NavLink to="/estrutura" className={linkClass}>
                Minha Estrutura
              </NavLink>
            )}
            {authed && (
              <NavLink to="/wallet" className={linkClass}>
                Carteira
              </NavLink>
            )}
            {isAdmin && (
              <NavLink to="/admin" className={linkClass}>
                Admin
              </NavLink>
            )}
            {authed ? (
              <>
                <div
                  className="flex items-center gap-0.5 rounded border border-felt-600 bg-felt-950/70 p-0.5"
                  title="Play Money e Jogo Real são saldos separados — não se misturam"
                >
                  <button
                    type="button"
                    className={mode === "play" ? "zt-tab zt-tab-active !px-2 !py-1 !text-[11px]" : "zt-tab !px-2 !py-1 !text-[11px]"}
                    aria-pressed={mode === "play"}
                    onClick={() => void switchMode("play")}
                  >
                    Play Money
                  </button>
                  <button
                    type="button"
                    className={mode === "real" ? "zt-tab zt-tab-active !px-2 !py-1 !text-[11px]" : "zt-tab !px-2 !py-1 !text-[11px]"}
                    aria-pressed={mode === "real"}
                    onClick={() => void switchMode("real")}
                  >
                    Jogo Real
                  </button>
                </div>
                {mode === "real" ? (
                  <span className="zt-chip hidden font-mono sm:inline-flex">
                    Real {formatBrlFromCents(me?.balance_real ?? 0)}
                  </span>
                ) : (
                  <span className="hidden items-center gap-1 sm:inline-flex">
                    <span className="zt-chip font-mono">
                      Cash {formatBrlFromCents(me?.balance_pm_cash ?? me?.balance ?? 0)}
                    </span>
                    <span className="zt-chip font-mono">
                      MTT {formatBrlFromCents(me?.balance_pm_mtt ?? 0)}
                    </span>
                  </span>
                )}
                {username && (
                  <span className="zt-chip hidden md:inline-flex">{username}</span>
                )}
                {me?.referral_code && (
                  <button
                    type="button"
                    className="zt-chip hidden font-mono lg:inline-flex"
                    title="Copiar link de convite"
                    onClick={() => {
                      const url = `${window.location.origin}/register?ref=${me.referral_code}`;
                      void navigator.clipboard.writeText(url);
                    }}
                  >
                    convite {me.referral_code}
                  </button>
                )}
                <button type="button" className="zt-btn-ghost text-sm" onClick={handleLogout}>
                  Sair
                </button>
              </>
            ) : (
              <>
                <NavLink to="/login" className={linkClass}>
                  Entrar
                </NavLink>
                <NavLink to="/register" className="zt-btn-primary !py-1.5 !text-xs">
                  Criar conta
                </NavLink>
              </>
            )}
          </nav>
        </div>
      </header>
      <main
        className={
          homeBleed
            ? "w-full flex-1"
            : wideMain
              ? "w-full flex-1 px-4 py-4"
              : "mx-auto w-full max-w-6xl flex-1 px-4 py-4 sm:py-8"
        }
      >
        <Outlet />
      </main>
      <footer className="border-t border-felt-700 px-4 py-4 text-center text-xs text-felt-400">
        Zero Tilt Poker · Play Money e Jogo Real
        {marketingShell ? " · mesa ao vivo, rake transparente" : " · Demo / staging"}
      </footer>
    </div>
  );
}
