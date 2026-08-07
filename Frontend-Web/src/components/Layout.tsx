import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { getUsername, isAuthenticated } from "@/lib/auth";
import { logout } from "@/api/client";
import { OnlinePresenceNav } from "@/components/OnlinePresence";

const linkClass = ({ isActive }: { isActive: boolean }) =>
  `zt-nav-link ${isActive ? "zt-nav-link-active" : ""}`;

export function Layout() {
  const navigate = useNavigate();
  const authed = isAuthenticated();
  const username = getUsername();

  function handleLogout() {
    logout();
    navigate("/login");
  }

  return (
    <div className="zt-shell">
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
          <nav className="flex flex-wrap items-center gap-4">
            <NavLink to="/lobby" className={linkClass}>
              Lobby
            </NavLink>
            <NavLink to="/admin/clubs" className={linkClass}>
              Clubes
            </NavLink>
            {authed ? (
              <>
                {username && (
                  <span className="zt-chip hidden sm:inline-flex">{username}</span>
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
        Zero Tilt Poker · Texas Hold&apos;em · Demo / staging · Play-money
      </footer>
    </div>
  );
}
