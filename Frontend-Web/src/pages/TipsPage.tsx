import { Link } from "react-router-dom";
import { NewsTips } from "@/components/NewsTips";
import { isAuthenticated } from "@/lib/auth";

export function TipsPage() {
  const authed = isAuthenticated();
  const backTo = authed ? "/curso" : "/";
  return (
    <div className="mx-auto w-full min-w-0 max-w-6xl space-y-4">
      <div>
        <Link to={backTo} className="text-xs text-gold-soft hover:underline">
          ← {authed ? "Academy" : "Início"}
        </Link>
        <h1 className="mt-1 text-xl font-bold text-gold-bright">Dica do Pró</h1>
        <p className="mt-1 text-sm text-felt-200">
          Estratégia por street — pré-flop, flop, turn e river — para estudar antes de sentar.
          Teoria completa e treino avaliado na{" "}
          <Link to="/curso" className="font-semibold text-gold-soft hover:underline">
            Zero Tilt Academy
          </Link>
          .
        </p>
      </div>
      <NewsTips tab="tips" />
    </div>
  );
}
