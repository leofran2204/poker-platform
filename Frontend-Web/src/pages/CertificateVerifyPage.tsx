import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { ApiError, verifyCertificate, type CourseCertificate } from "@/api/client";
import { BrandMark } from "@/components/BrandMark";

function longDate(iso: string): string {
  return new Date(iso).toLocaleDateString("pt-BR", { day: "numeric", month: "long", year: "numeric" });
}

/** Página pública: confere um certificado pelo código e mostra a versão para imprimir. */
export function CertificateVerifyPage() {
  const { code } = useParams();
  const navigate = useNavigate();
  const [typed, setTyped] = useState(code ?? "");
  const [found, setFound] = useState<{ code: string; certificate: CourseCertificate | null; error?: string } | null>(null);

  useEffect(() => {
    if (!code) return;
    let alive = true;
    verifyCertificate(code)
      .then((certificate) => alive && setFound({ code, certificate }))
      .catch(
        (e: unknown) =>
          alive &&
          setFound({
            code,
            certificate: null,
            error: e instanceof ApiError && e.status === 404 ? undefined : "Não foi possível consultar agora. Tente de novo.",
          }),
      );
    return () => {
      alive = false;
    };
  }, [code]);

  const result = found && found.code === code ? found : null;
  const certificate = result?.certificate ?? null;

  return (
    <div className="mx-auto max-w-4xl space-y-4 px-4 py-6">
      <div className="zt-no-print space-y-3">
        <h1 className="text-2xl font-bold text-cream">Verificar certificado</h1>
        <form
          className="flex flex-wrap gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            const clean = typed.trim();
            if (clean) void navigate(`/certificado/${encodeURIComponent(clean)}`);
          }}
        >
          <input
            value={typed}
            onChange={(e) => setTyped(e.target.value)}
            placeholder="ZT-XXXX-XXXX-XXXX"
            aria-label="Código do certificado"
            className="min-w-60 flex-1 rounded-lg border border-felt-700 bg-felt-950 px-3 py-2 font-mono text-cream uppercase"
          />
          <button type="submit" className="zt-btn-primary">
            Verificar
          </button>
        </form>
        {code && !result && <p className="text-sm text-felt-300">Consultando…</p>}
        {result && !certificate && (
          <p className="zt-panel p-4 text-sm text-red-200">
            {result.error ?? "Nenhum certificado com este código. Confira as letras e os números."}
          </p>
        )}
        {certificate && (
          <div className="flex flex-wrap items-center justify-between gap-2">
            <p className={certificate.revoked_at ? "text-sm font-bold text-red-200" : "text-sm font-bold text-emerald-300"}>
              {certificate.revoked_at
                ? `Certificado revogado em ${longDate(certificate.revoked_at)}.`
                : "Certificado verdadeiro, emitido pela Zero Tilt Academy."}
            </p>
            {!certificate.revoked_at && (
              <button type="button" className="zt-btn-secondary" onClick={() => window.print()}>
                Imprimir ou salvar em PDF
              </button>
            )}
          </div>
        )}
      </div>

      {certificate && (
        <article className="zt-certificate rounded-2xl border-4 border-double border-gold/70 bg-felt-950 p-8 text-center sm:p-12">
          <p className="zt-cert-accent text-sm tracking-[0.3em] text-gold-soft uppercase">
            <BrandMark /> Zero Tilt Academy
          </p>
          <h2 className="mt-4 text-3xl font-bold text-cream">Certificado de conclusão</h2>
          <p className="mt-6 text-sm text-felt-300">Certificamos que</p>
          <p className="zt-cert-accent mt-2 text-3xl font-semibold text-gold-soft">{certificate.holder_name}</p>
          <p className="mx-auto mt-4 max-w-xl text-sm text-felt-200">
            concluiu as {certificate.lessons} aulas obrigatórias do curso de poker da Zero Tilt Academy, com vídeo assistido e quiz de
            cada aula, e obteve aprovação na prova final com nota {certificate.score}%.
          </p>
          <p className="mt-6 text-sm text-felt-300">Emitido em {longDate(certificate.issued_at)}</p>
          <p className="mt-6 font-mono text-sm text-cream">{certificate.code}</p>
          <p className="mt-1 text-xs text-felt-400">
            Confira em {window.location.origin}/certificado/{certificate.code}
          </p>
          <p className="mt-1 text-xs text-felt-400">
            {certificate.name_source === "kyc" ? "Nome conferido pela verificação de identidade." : "Nome informado pelo aluno."}
          </p>
          <p className="mt-4 text-[11px] text-felt-500">
            Certificado de estudo. Não garante resultado nem ganho em mesas reais (Termos de Uso, cláusula 3).
          </p>
        </article>
      )}

      <p className="zt-no-print text-xs">
        <Link to="/curso" className="text-gold-soft hover:underline">
          Conhecer a Zero Tilt Academy →
        </Link>
      </p>
    </div>
  );
}
