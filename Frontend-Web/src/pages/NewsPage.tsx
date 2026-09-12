import { Link } from "react-router-dom";
import { NewsTips } from "@/components/NewsTips";

export function NewsPage() {
  return (
    <div className="mx-auto w-full max-w-6xl space-y-4">
      <div>
        <Link to="/" className="text-xs text-gold-soft hover:underline">
          ← Início
        </Link>
        <h1 className="mt-1 text-xl font-bold text-gold-bright">Notícias do poker</h1>
        <p className="mt-1 text-sm text-felt-200">
          Cobertura do circuito — Brasil e mundo — atualizada a cada 5 minutos.
          Quer melhorar seu jogo? Veja a{" "}
          <Link to="/dicas" className="font-semibold text-gold-soft hover:underline">
            Dica do Pró
          </Link>
          .
        </p>
      </div>
      <NewsTips tab="news" />
    </div>
  );
}
