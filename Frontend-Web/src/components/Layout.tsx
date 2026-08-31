import { useCallback, useEffect, useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { setWalletMode as apiSetWalletMode } from "@/api/client";
import type { MeResponse, WalletMode } from "@/api/types";
import { OnlinePresenceNav } from "@/components/OnlinePresence";
import { SessionConnectivity } from "@/components/SessionConnectivity";
import { clearTokens, getUsername, isAuthenticated } from "@/lib/auth";
import { clearMeCache, getMe, isAdminRole } from "@/lib/me";
import { formatBrlFromCents } from "@/lib/money";
import { getWalletMode, setWalletModeLocal } from "@/lib/walletMode";

const linkClass = ({ isActive }: { isActive: boolean }) =>
  `zt-nav-link ${isActive ? "zt-nav-link-active" : ""}`;

export function Layout() {
  const navigate = useNavigate();
  const authed = isAuthenticated();
  const username = getUsername();
  const [isAdmin, setIsAdmin] = useState(false);
  const [me, setMe] = useState<MeResponse | null>(null);
  const [mode, setMode] = useState<WalletMode>(getWalletMode());

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

  async function switchMode(next: WalletMode) {
    setMode(next);
    setWalletModeLocal(next);
    window.dispatchEvent(new CustomEvent("wallet-mode-changed", { detail: next }));
    try {
      await apiSetWalletMode(next);
      clearMeCache();
      await refreshMe();
    } catch {
      /* keep local preference */
    }
  }

  function handleLogout() {
    clearTokens();
    clearMeCache();
    setIsAdmin(false);
    setMe(null);
    navigate("/login");
  }

  const activeBalance =
    mode === "real" ? (me?.balance_real ?? 0) : (me?.balance_pm_cash ?? me?.balance ?? 0);

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
            <NavLink to="/lobby" className={linkClass}>
              Lobby
            </NavLink>
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
                    onClick={() => void switchMode("play")}
                  >
                    Play Money
                  </button>
                  <button
                    type="button"
                    className={mode === "real" ? "zt-tab zt-tab-active !px-2 !py-1 !text-[11px]" : "zt-tab !px-2 !py-1 !text-[11px]"}
                    onClick={() => void switchMode("real")}
                  >
                    Jogo Real
                  </button>
                </div>
                <span className="zt-chip hidden font-mono sm:inline-flex">
                  {mode === "real" ? "Real" : "PM"} {formatBrlFromCents(activeBalance)}
                </span>
                {username && (
                  <span className="zt-chip hidden md:inline-flex">{username}</span>
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
      <main className="mx-auto w-full max-w-6xl flex-1 px-4 py-8">
        <Outlet />
      </main>
      <footer className="border-t border-felt-700 px-4 py-4 text-center text-xs text-felt-400">
        Zero Tilt Poker · Play Money &amp; Jogo Real · Demo / staging
      </footer>
    </div>
  );
}
