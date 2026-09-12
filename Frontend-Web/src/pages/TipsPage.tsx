import { Link } from "react-router-dom";
import { NewsTips } from "@/components/NewsTips";

export function TipsPage() {
  return (
    <div className="mx-auto w-full max-w-6xl space-y-4">
      <div>
        <Link to="/" className="text-xs text-gold-soft hover:underline">
          ← Início
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
