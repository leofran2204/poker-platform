import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { Link } from "react-router";
import {
  fetchCertificateStatus,
  startCourseExam,
  submitCourseExam,
  type CourseCertificate,
  type CourseCertificateStatus,
  type CourseExam,
  type CourseExamResult,
} from "@/api/client";
import { PlayingCard } from "@/components/PlayingCard";
import { isAuthenticated } from "@/lib/auth";
import { findLesson } from "@/lib/course";
import { clock } from "@/lib/useLessonWatch";

const errorText = (e: unknown) => (e instanceof Error ? e.message : "Não foi possível falar com o servidor. Tente de novo.");

export function localDateTime(iso: string): string {
  return new Date(iso).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" });
}

function LessonLink({ id }: { id: string }) {
  return (
    <Link to={`/curso/${id}`} className="text-gold-soft underline">
      {findLesson(id)?.title ?? id}
    </Link>
  );
}

function Shell({ children }: { children: ReactNode }) {
  return (
    <div className="mx-auto max-w-3xl space-y-4 px-4 py-6">
      <p className="text-xs">
        <Link to="/curso" className="text-gold-soft hover:underline">
          ← Voltar à Academy
        </Link>
      </p>
      <h1 className="text-2xl font-bold text-cream">Certificado Zero Tilt Academy</h1>
      {children}
    </div>
  );
}

/**
 * Certificado emitido só pelo servidor: aulas cumpridas (vídeo e quiz), prova
 * final com tempo e uma tentativa a cada 24 h. O código confere o certificado
 * em `/certificado/<código>`.
 */
export function CertificatePage() {
  const authed = isAuthenticated();
  const [status, setStatus] = useState<CourseCertificateStatus | null>(null);
  const [exam, setExam] = useState<CourseExam | null>(null);
  const [result, setResult] = useState<CourseExamResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    fetchCertificateStatus()
      .then((s) => {
        setStatus(s);
        setExam(s.open_exam);
      })
      .catch((e: unknown) => setError(errorText(e)));
  }, []);

  useEffect(() => {
    if (authed) reload();
  }, [authed, reload]);

  if (!authed) {
    return (
      <Shell>
        <p className="zt-panel p-4 text-sm text-felt-200">
          <Link to="/login" className="text-gold-soft underline">
            Entre na sua conta
          </Link>{" "}
          para acompanhar as aulas que faltam e fazer a prova final.
        </p>
      </Shell>
    );
  }
  if (!status) {
    return <Shell>{error ? <p className="text-xs text-red-200">{error}</p> : <p className="text-sm text-felt-300">Carregando…</p>}</Shell>;
  }
  if (exam) {
    return (
      <Shell>
        <ExamRunner
          exam={exam}
          onDone={(r) => {
            setExam(null);
            setResult(r);
            reload();
          }}
        />
      </Shell>
    );
  }
  const certificate = result?.certificate ?? status.certificate;
  return (
    <Shell>
      {result && !result.passed && <FailedPanel result={result} passScore={status.pass_score} />}
      {certificate ? (
        <IssuedPanel certificate={certificate} />
      ) : status.eligible ? (
        <StartPanel status={status} onStart={setExam} />
      ) : (
        <MissingPanel status={status} />
      )}
    </Shell>
  );
}

function MissingPanel({ status }: { status: CourseCertificateStatus }) {
  const done = status.total_lessons - status.missing.length;
  return (
    <div className="zt-panel space-y-3 p-4">
      <div className="zt-panel-title">
        Aulas cumpridas: {done} de {status.total_lessons}
      </div>
      <p className="text-sm text-felt-200">
        O certificado pede todas as aulas obrigatórias com o vídeo assistido (90%, com as perguntas surpresa) e o quiz aprovado
        (mínimo {status.pass_score}%). Depois vem a prova final. A trilha de história do poker é opcional.
      </p>
      <ul className="space-y-1 text-sm text-felt-200">
        {status.missing.map((m) => (
          <li key={m.lesson_id}>
            <LessonLink id={m.lesson_id} /> —{" "}
            {m.reason === "quiz"
              ? "falta concluir a aula (vídeo e quiz)"
              : "quiz aprovado antes da contagem do vídeo: assista ao vídeo e responda às perguntas surpresa"}
          </li>
        ))}
      </ul>
    </div>
  );
}

function StartPanel({ status, onStart }: { status: CourseCertificateStatus; onStart: (exam: CourseExam) => void }) {
  const [name, setName] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [now, setNow] = useState(() => Date.now());
  const waitUntil = status.next_attempt_at ? Date.parse(status.next_attempt_at) : 0;
  const waiting = waitUntil > now;

  useEffect(() => {
    if (!waitUntil) return;
    const timer = window.setInterval(() => setNow(Date.now()), 30_000);
    return () => window.clearInterval(timer);
  }, [waitUntil]);

  async function start() {
    setSending(true);
    setError(null);
    try {
      onStart(await startCourseExam(status.verified_name ? null : name));
    } catch (e) {
      setError(errorText(e));
    } finally {
      setSending(false);
    }
  }

  return (
    <div className="zt-panel space-y-3 p-4">
      <div className="zt-panel-title">Prova final</div>
      <p className="text-sm text-felt-200">Todas as {status.total_lessons} aulas obrigatórias foram cumpridas. Regras da prova:</p>
      <ul className="list-disc space-y-1 pl-5 text-sm text-felt-200">
        <li>{status.exam_questions} perguntas sorteadas, cada uma de uma aula diferente, com as alternativas em ordem sorteada.</li>
        <li>
          {status.exam_minutes} minutos, contados pelo servidor. Fechar a página não para o relógio; no fim do tempo, a prova vale como
          reprovada se não tiver sido entregue.
        </li>
        <li>Nota mínima: {status.pass_score}%.</li>
        <li>Uma tentativa a cada 24 horas. Na reprovação aparecem a nota e as aulas a rever, sem o gabarito.</li>
      </ul>
      {status.verified_name ? (
        <p className="text-sm text-felt-200">
          O certificado sai com o nome da sua verificação de identidade: <strong className="text-cream">{status.verified_name}</strong>.
        </p>
      ) : (
        <label className="block space-y-1 text-sm text-felt-200">
          <span>Nome completo para o certificado (não muda depois da emissão)</span>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            maxLength={80}
            autoComplete="name"
            className="w-full rounded-lg border border-felt-700 bg-felt-950 px-3 py-2 text-cream"
          />
          <span className="block text-xs text-felt-400">
            Sem a <Link to="/verificacao" className="text-gold-soft underline">verificação de identidade</Link>, o certificado informa que o nome foi
            declarado pelo aluno.
          </span>
        </label>
      )}
      {status.last_attempt && !status.last_attempt.passed && (
        <p className="text-xs text-felt-300">
          Última tentativa: {status.last_attempt.timed_out ? "tempo esgotado" : `${status.last_attempt.score}%`}.
        </p>
      )}
      {waiting && status.next_attempt_at ? (
        <p className="text-sm text-amber-300">Nova tentativa liberada em {localDateTime(status.next_attempt_at)}.</p>
      ) : (
        <button
          type="button"
          className="zt-btn-primary"
          disabled={sending || (!status.verified_name && name.trim().length < 5)}
          onClick={() => void start()}
        >
          {sending ? "Abrindo…" : "Começar a prova"}
        </button>
      )}
      {error && <p className="text-xs text-red-200">{error}</p>}
    </div>
  );
}

function ExamRunner({ exam, onDone }: { exam: CourseExam; onDone: (result: CourseExamResult) => void }) {
  const storageKey = `zt-exam-${exam.attempt_id}`;
  const [answers, setAnswers] = useState<Record<string, string>>(() => {
    try {
      return JSON.parse(sessionStorage.getItem(storageKey) ?? "{}") as Record<string, string>;
    } catch {
      return {};
    }
  });
  const endAt = useRef(Date.now() + exam.seconds_left * 1000);
  const [left, setLeft] = useState(exam.seconds_left);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const sent = useRef(false);
  const answered = exam.questions.filter((q) => answers[q.id] !== undefined).length;

  const submit = useCallback(async () => {
    if (sent.current) return;
    sent.current = true;
    setSending(true);
    setError(null);
    try {
      const result = await submitCourseExam(exam.attempt_id, answers);
      sessionStorage.removeItem(storageKey);
      onDone(result);
    } catch (e) {
      sent.current = false;
      setError(errorText(e));
    } finally {
      setSending(false);
    }
  }, [answers, exam.attempt_id, onDone, storageKey]);

  useEffect(() => {
    sessionStorage.setItem(storageKey, JSON.stringify(answers));
  }, [answers, storageKey]);

  useEffect(() => {
    const timer = window.setInterval(() => setLeft(Math.max(0, (endAt.current - Date.now()) / 1000)), 1000);
    return () => window.clearInterval(timer);
  }, []);

  // No fim do tempo, entrega o que estiver marcado.
  useEffect(() => {
    if (left <= 0) void submit();
  }, [left, submit]);

  return (
    <div className="space-y-4">
      <div className="zt-panel sticky top-2 z-10 flex flex-wrap items-center justify-between gap-2 p-3 text-sm">
        <span className="text-cream">
          Prova final · {answered}/{exam.questions.length} respondidas
        </span>
        <span className={left < 120 ? "font-bold text-amber-300" : "text-felt-200"}>Tempo: {clock(left)}</span>
      </div>
      {exam.questions.map((q, idx) => (
        <div key={q.id} className="zt-panel space-y-2 p-4">
          <p className="text-sm font-semibold text-cream">
            {idx + 1}. {q.prompt}
          </p>
          {q.kind === "engine" && (
            <div className="flex flex-wrap items-center gap-4 text-xs text-felt-300">
              <span className="flex items-center gap-1">
                Mão:
                {q.hole.map((c) => (
                  <PlayingCard key={c} code={c} size="xs" inline />
                ))}
              </span>
              {q.board.length > 0 && (
                <span className="flex items-center gap-1">
                  Mesa:
                  {q.board.map((c) => (
                    <PlayingCard key={c} code={c} size="xs" inline />
                  ))}
                </span>
              )}
            </div>
          )}
          <div className="flex flex-wrap gap-2">
            {q.options.map((opt) => (
              <button
                key={opt}
                type="button"
                aria-pressed={answers[q.id] === opt}
                disabled={sending}
                onClick={() => setAnswers((a) => ({ ...a, [q.id]: opt }))}
                className={answers[q.id] === opt ? "zt-btn-primary text-xs!" : "zt-btn-secondary text-xs!"}
              >
                {opt}
              </button>
            ))}
          </div>
        </div>
      ))}
      <button
        type="button"
        className="zt-btn-primary"
        disabled={sending || answered < exam.questions.length}
        onClick={() => void submit()}
      >
        {sending ? "Enviando…" : "Entregar a prova"}
      </button>
      {error && <p className="text-xs text-red-200">{error}</p>}
    </div>
  );
}

function FailedPanel({ result, passScore }: { result: CourseExamResult; passScore: number }) {
  return (
    <div className="zt-panel space-y-2 p-4">
      <div className="zt-panel-title">Prova final não aprovada</div>
      <p className="text-sm text-cream">
        {result.timed_out ? "A prova foi entregue depois do tempo." : `Nota: ${result.score}% (mínimo ${passScore}%).`}
      </p>
      {result.review_lessons.length > 0 && (
        <p className="text-sm text-felt-200">
          Aulas a rever:{" "}
          {result.review_lessons.map((id, i) => (
            <span key={id}>
              {i > 0 && ", "}
              <LessonLink id={id} />
            </span>
          ))}
          .
        </p>
      )}
      {result.next_attempt_at && <p className="text-xs text-felt-300">Nova tentativa em {localDateTime(result.next_attempt_at)}.</p>}
    </div>
  );
}

function IssuedPanel({ certificate }: { certificate: CourseCertificate }) {
  const link = `${window.location.origin}/certificado/${certificate.code}`;
  const [copied, setCopied] = useState(false);
  return (
    <div className="zt-panel space-y-3 p-4">
      <div className="zt-panel-title">{certificate.revoked_at ? "Certificado revogado" : "Certificado emitido"}</div>
      <p className="text-sm text-felt-200">
        {certificate.holder_name} · nota {certificate.score}% · emitido em {localDateTime(certificate.issued_at)}
      </p>
      <p className="text-sm text-cream">
        Código: <strong className="font-mono">{certificate.code}</strong>
      </p>
      <div className="flex flex-wrap gap-2">
        <Link to={`/certificado/${certificate.code}`} className="zt-btn-primary">
          Ver e imprimir
        </Link>
        <button
          type="button"
          className="zt-btn-secondary"
          onClick={() => {
            void navigator.clipboard?.writeText(link).then(() => setCopied(true));
          }}
        >
          {copied ? "Link copiado" : "Copiar link de verificação"}
        </button>
      </div>
      <p className="text-xs text-felt-400">Qualquer pessoa com o código confere o certificado em {link}.</p>
    </div>
  );
}
