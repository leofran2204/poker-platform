import { FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { applyAuthTokens, login } from "@/api/client";
import { saveUsername } from "@/lib/auth";

export function LoginPage() {
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      const tokens = await login(email.trim(), password);
      applyAuthTokens(tokens);
      saveUsername(email.trim().split("@")[0] || "jogador");
      navigate("/lobby");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Falha no login");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="mx-auto max-w-md">
      <div className="zt-panel">
        <div className="zt-panel-title">Entrar</div>
        <form className="space-y-4 p-5" onSubmit={onSubmit}>
          <div>
            <label className="zt-label" htmlFor="email">
              E-mail
            </label>
            <input
              id="email"
              type="email"
              className="zt-input"
              autoComplete="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          <div>
            <label className="zt-label" htmlFor="password">
              Senha
            </label>
            <input
              id="password"
              type="password"
              className="zt-input"
              autoComplete="current-password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>
          {error && (
            <p className="rounded border border-red-800 bg-red-950/50 px-3 py-2 text-sm text-red-200">
              {error}
            </p>
          )}
          <button type="submit" className="zt-btn-primary w-full" disabled={loading}>
            {loading ? "Entrando…" : "Entrar"}
          </button>
          <p className="text-center text-sm text-felt-300">
            Novo por aqui?{" "}
            <Link to="/register" className="font-semibold text-gold-bright hover:underline">
              Criar conta
            </Link>
          </p>
        </form>
      </div>
    </div>
  );
}
