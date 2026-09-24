import { useCallback, useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router";
import { sendPresenceOffline, setWalletMode as apiSetWalletMode } from "@/api/client";
import type { MeResponse, WalletMode } from "@/api/types";
import {
  OnlinePresenceNav,
  reflectPresenceCount,
  reflectPresenceLogout,
} from "@/components/OnlinePresence";
import { BrandMark } from "@/components/BrandMark";
import { SessionConnectivity } from "@/components/SessionConnectivity";
import { clearTokens, getUsername, isAuthenticated } from "@/lib/auth";
import { clearMeCache, getMe, isAdminRole } from "@/lib/me";
import { formatBrlFromCents } from "@/lib/money";
import { SESSION_EXPIRED_EVENT, SESSION_RESTORED_EVENT } from "@/lib/sessionEvents";
import { getWalletMode, setWalletModeLocal } from "@/lib/walletMode";

const linkClass = ({ isActive }: { isActive: boolean }) =>
  `zt-nav-link ${isActive ? "zt-nav-link-active" : ""}`;

const VISITOR_HOME_PATHS = new Set(["/", "/login", "/register", "/verify-email", "/recuperar-senha", "/termos", "/rede", "/jogo-responsavel"]);

export function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const [authTick, setAuthTick] = useState(0);
  const authed = isAuthenticated();
  const username = getUsername();
  const marketingShell = !authed && VISITOR_HOME_PATHS.has(location.pathname);
  const homeBleed = location.pathname === "/";
  const wideMain =
    location.pathname.startsWith("/admin") || location.pathname.startsWith("/table");
  const [isAdmin, setIsAdmin] = useState(false);
  const [me, setMe] = useState<MeResponse | null>(null);
  const [mode, setMode] = useState<WalletMode>(getWalletMode());
  const [moreOpen, setMoreOpen] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const moreRef = useRef<HTMLDivElement>(null);
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
    setMoreOpen(false);
    setMenuOpen(false);
  }, [location.pathname]);

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

  useEffect(() => {
    if (!moreOpen) return;
    const onDoc = (e: MouseEvent) => {
      if (moreRef.current && !moreRef.current.contains(e.target as Node)) setMoreOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [moreOpen]);

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

  const learnLinks = (
    <>
      <NavLink to="/noticias" className={linkClass} onClick={() => setMenuOpen(false)}>
        Notícias
      </NavLink>
      <NavLink to="/dicas" className={linkClass} onClick={() => setMenuOpen(false)}>
        Dica do Pró
      </NavLink>
    </>
  );

  const walletToggle = authed ? (
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
  ) : null;

  return (
    <div className="zt-shell">
      <a
        href="#main-content"
        className="fixed left-3 top-3 z-[100] -translate-y-24 rounded bg-gold px-3 py-2 text-sm font-bold text-ink transition-transform focus:translate-y-0"
      >
        Pular para o conteúdo
      </a>
      <SessionConnectivity />
      <header className="zt-nav">
        <div className="zt-nav-inner">
          <div className="flex min-w-0 items-center gap-3">
            <NavLink to={authed ? "/curso" : "/"} className="zt-brand shrink-0">
              <BrandMark />
              Zero Tilt
            </NavLink>
            <OnlinePresenceNav />
          </div>
          <nav className="hidden items-center gap-3 md:flex md:gap-4">
            {authed ? (
              <>
                <NavLink to="/curso" className={linkClass}>
                  Curso
                </NavLink>
                <NavLink to="/lobby" className={linkClass}>
                  Lobby
                </NavLink>
                <NavLink to="/wallet" className={linkClass}>
                  Carteira
                </NavLink>
                <NavLink to="/estrutura" className={linkClass}>
                  Minha Rede
                </NavLink>
                <div className="relative" ref={moreRef}>
                  <button
                    type="button"
                    className="zt-nav-link"
                    aria-expanded={moreOpen}
                    onClick={() => setMoreOpen((v) => !v)}
                  >
                    Mais
                  </button>
                  {moreOpen && (
                    <div className="absolute right-0 z-50 mt-2 min-w-[11rem] rounded border border-felt-600 bg-felt-950 py-1 shadow-panel">
                      <NavLink
                        to="/noticias"
                        className="block px-3 py-1.5 text-sm text-felt-200 hover:bg-felt-800 hover:text-cream"
                        onClick={() => setMoreOpen(false)}
                      >
                        Notícias
                      </NavLink>
                      <NavLink
                        to="/dicas"
                        className="block px-3 py-1.5 text-sm text-felt-200 hover:bg-felt-800 hover:text-cream"
                        onClick={() => setMoreOpen(false)}
                      >
                        Dica do Pró
                      </NavLink>
                      <NavLink
                        to="/verificacao"
                        className="block px-3 py-1.5 text-sm text-felt-200 hover:bg-felt-800 hover:text-cream"
                        onClick={() => setMoreOpen(false)}
                      >
                        Verificação
                      </NavLink>
                      <NavLink
                        to="/suporte"
                        className="block px-3 py-1.5 text-sm text-felt-200 hover:bg-felt-800 hover:text-cream"
                        onClick={() => setMoreOpen(false)}
                      >
                        Suporte
                      </NavLink>
                      {isAdmin && (
                        <NavLink
                          to="/admin"
                          className="block px-3 py-1.5 text-sm text-felt-200 hover:bg-felt-800 hover:text-cream"
                          onClick={() => setMoreOpen(false)}
                        >
                          Admin
                        </NavLink>
                      )}
                    </div>
                  )}
                </div>
                {walletToggle}
                {mode === "real" ? (
                  <span className="zt-chip hidden font-mono lg:inline-flex">
                    Real {formatBrlFromCents(me?.balance_real ?? 0)}
                  </span>
                ) : (
                  <span className="hidden items-center gap-1 lg:inline-flex">
                    <span className="zt-chip font-mono">
                      Cash {formatBrlFromCents(me?.balance_pm_cash ?? me?.balance ?? 0)}
                    </span>
                    <span className="zt-chip font-mono">
                      MTT {formatBrlFromCents(me?.balance_pm_mtt ?? 0)}
                    </span>
                  </span>
                )}
                {username && <span className="zt-chip hidden xl:inline-flex">{username}</span>}
                <button type="button" className="zt-btn-ghost text-sm" onClick={handleLogout}>
                  Sair
                </button>
              </>
            ) : (
              <>
                <NavLink to="/curso" className={linkClass}>
                  Academy
                </NavLink>
                <NavLink to="/rede" className={linkClass}>
                  Rede
                </NavLink>
                <NavLink to="/login" className={linkClass}>
                  Entrar
                </NavLink>
                <NavLink to="/register" className="zt-btn-primary !py-1.5 !text-xs">
                  Criar conta
                </NavLink>
              </>
            )}
          </nav>
          <button
            type="button"
            className="zt-btn-ghost md:hidden"
            aria-expanded={menuOpen}
            aria-label={menuOpen ? "Fechar menu" : "Abrir menu"}
            onClick={() => setMenuOpen((v) => !v)}
          >
            {menuOpen ? "Fechar" : "Menu"}
          </button>
        </div>
        {menuOpen && (
          <div className="flex flex-col gap-3 border-t border-felt-700 px-4 py-3 md:hidden">
            {authed ? (
              <>
                <NavLink to="/curso" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Curso
                </NavLink>
                <NavLink to="/lobby" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Lobby
                </NavLink>
                <NavLink to="/wallet" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Carteira
                </NavLink>
                {learnLinks}
                <NavLink to="/estrutura" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Minha Rede
                </NavLink>
                <NavLink to="/verificacao" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Verificação
                </NavLink>
                <NavLink to="/suporte" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Suporte
                </NavLink>
                {isAdmin && (
                  <NavLink to="/admin" className={linkClass} onClick={() => setMenuOpen(false)}>
                    Admin
                  </NavLink>
                )}
                {walletToggle}
                <button type="button" className="zt-btn-ghost self-start text-sm" onClick={handleLogout}>
                  Sair
                </button>
              </>
            ) : (
              <>
                <NavLink to="/curso" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Academy
                </NavLink>
                <NavLink to="/rede" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Rede
                </NavLink>
                {learnLinks}
                <NavLink to="/login" className={linkClass} onClick={() => setMenuOpen(false)}>
                  Entrar
                </NavLink>
                <NavLink to="/register" className="zt-btn-primary !py-1.5 !text-xs self-start">
                  Criar conta
                </NavLink>
              </>
            )}
          </div>
        )}
      </header>
      <main
        id="main-content"
        className={
          homeBleed
            ? "w-full min-w-0 flex-1"
            : wideMain
              ? "w-full min-w-0 flex-1 px-4 py-4"
              : "mx-auto w-full min-w-0 max-w-6xl flex-1 px-4 py-4 sm:py-8"
        }
      >
        <Outlet />
      </main>
      <footer className="border-t border-felt-700 px-4 py-4 text-center text-xs text-felt-300">
        ZT Poker · Zero Tilt Academy
        {marketingShell ? " · Estude. Jogue. Sem tilt." : " · Demo / staging"}
        {" · "}
        <NavLink to="/curso" className="text-gold-soft hover:underline">
          Curso
        </NavLink>
        {" · "}
        <NavLink to="/noticias" className="text-gold-soft hover:underline">
          Notícias
        </NavLink>
        {" · "}
        <NavLink to="/dicas" className="text-gold-soft hover:underline">
          Dica do Pró
        </NavLink>
        {" · "}
        <NavLink to="/rede" className="text-gold-soft hover:underline">
          Rede
        </NavLink>
        {" · "}
        <NavLink to="/termos" className="text-gold-soft hover:underline">
          Termos
        </NavLink>
        {" · "}
        <NavLink to="/jogo-responsavel" className="text-gold-soft hover:underline">
          Jogo responsável
        </NavLink>
        {" · "}
        <NavLink to="/suporte" className="text-gold-soft hover:underline">
          Suporte
        </NavLink>
      </footer>
    </div>
  );
}
