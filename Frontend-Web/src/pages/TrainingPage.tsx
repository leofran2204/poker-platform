import { useEffect, useRef, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router";
import { PlayingCard } from "@/components/PlayingCard";
import { CourseSources } from "@/components/CourseSources";
import { COURSE, findLesson } from "@/lib/course";
import { actionLabel, newStudySeed, phaseLabel, playStudy, profileLabel, readStudyRecords, saveStudyRecord, TRAINING, type StudyAction, type StudyRecord, type StudyResponse, type TrainingScenario } from "@/lib/academy";
import "./training.css";

const chips = (n: number) => n.toLocaleString("pt-BR");
const variantName: Record<string, string> = { holdem: "Texas Hold’em", short_deck: "Texas Hold’em Short Deck", omaha: "Omaha 4", brazilian_pineapple: "Brazilian Pineapple" };

export function TrainingPage() {
  const { moduleId } = useParams();
  const [params] = useSearchParams();
  const module = COURSE.modules.find(m => m.id === moduleId);
  const scenarios = TRAINING.scenarios.filter(s => s.module === moduleId);
  if (!module || !scenarios.length) return <div className="zt-panel p-6"><h1>Mesa não encontrada</h1><Link to="/curso">Voltar à Academy</Link></div>;
  return <TrainingSession key={`${moduleId}:${params.get("cenario") ?? ""}`} moduleTitle={module.title} scenarios={scenarios} initialId={params.get("cenario")} />;
}

function TrainingSession({ moduleTitle, scenarios, initialId }: { moduleTitle: string; scenarios: TrainingScenario[]; initialId: string | null }) {
  const [scenarioId, setScenarioId] = useState(scenarios.find(s => s.id === initialId)?.id ?? scenarios[0].id);
  const scenario = scenarios.find(s => s.id === scenarioId) ?? scenarios[0];
  const [seed, setSeed] = useState(newStudySeed);
  const [mode, setMode] = useState("guided");
  const [actions, setActions] = useState<StudyAction[]>([]);
  const [data, setData] = useState<StudyResponse | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState("");
  const [answer, setAnswer] = useState("");
  const [amount, setAmount] = useState(0);
  const [notes, setNotes] = useState("");
  const [saved, setSaved] = useState("");
  const [closed, setClosed] = useState(false);
  const [replayIndex, setReplayIndex] = useState<number | null>(null);
  const [records, setRecords] = useState(readStudyRecords);
  const requestId = useRef(0);
  const controller = useRef<AbortController | null>(null);
  const restoring = useRef<StudyAction[] | null>(null);

  async function load(nextActions: StudyAction[], nextScenario = scenarioId, nextSeed = seed) {
    controller.current?.abort();
    const abort = new AbortController(); controller.current = abort;
    const id = ++requestId.current;
    setBusy(true); setError(""); setSaved("");
    try {
      const result = await playStudy(nextScenario, nextSeed, nextActions, abort.signal);
      if (id !== requestId.current) return;
      setData(result); setActions(nextActions); setAnswer(""); setAmount(result.legal.min_total); setReplayIndex(null);
    } catch (e) {
      if (id === requestId.current && !abort.signal.aborted) setError(e instanceof Error ? e.message : "Falha ao carregar treino.");
    } finally { if (id === requestId.current) setBusy(false); }
  }

  useEffect(() => {
    const restoredActions = restoring.current;
    restoring.current = null;
    void load(restoredActions ?? [], scenarioId, seed);
    return () => controller.current?.abort();
    // A troca de cenário/distribuição inicia uma sessão, sem repetir ações antigas.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scenarioId, seed]);

  function reset(nextId = scenarioId, fresh = false) {
    setClosed(false); setNotes(""); setData(null); setActions([]); setReplayIndex(null);
    if (nextId !== scenarioId) setScenarioId(nextId);
    else if (fresh) setSeed(newStudySeed());
    else void load([]);
  }
  function act(action: string) { if (!data || busy || replayIndex !== null) return; void load([...actions, { action, amount, answer }]); }
  function save() {
    if (!data) return;
    const ok = saveStudyRecord({ version: TRAINING.version, id: `${scenario.id}:${seed}`, scenario: scenario.id, seed, when: new Date().toISOString(), decisions: data.reviews.filter(r => r.answer.trim()).length,
      correct: data.reviews.filter(r => r.correct).length, notes, actions });
    setSaved(ok ? "Sessão salva neste navegador." : "O navegador não permitiu salvar. Exporte o registro para guardar."); setRecords(readStudyRecords());
  }
  function restore(record: StudyRecord) {
    setClosed(false); setNotes(record.notes); setReplayIndex(null);
    if (record.scenario === scenarioId && record.seed === seed) void load(record.actions);
    else {
      restoring.current = record.actions;
      setData(null); setActions([]); setScenarioId(record.scenario); setSeed(record.seed);
    }
  }
  function download() {
    const url = URL.createObjectURL(new Blob([JSON.stringify({ version: TRAINING.version, scenario: scenarioId, seed, actions, notes, reviews: data?.reviews }, null, 2)], { type: "application/json" }));
    const anchor = document.createElement("a"); anchor.href = url; anchor.download = `zt-treino-${scenarioId}-${seed}.json`; anchor.click(); URL.revokeObjectURL(url);
  }
  const state = replayIndex !== null ? data?.events[replayIndex]?.state : data?.state;
  const lesson = findLesson(scenario.lesson);
  const latestReview = data?.reviews[data.reviews.length - 1];
  const disabled = busy || !data || data.state.finished || closed || replayIndex !== null;

  return <div className="study-page">
    <header className="study-heading"><div><Link className="text-gold-soft text-sm" to="/curso">← Zero Tilt Academy</Link><p className="study-eyebrow mt-4">MESA DO MÓDULO · {variantName[scenario.variant]}</p><h1>{moduleTitle}</h1><p>Experimente, acompanhe as respostas dos bots e reveja suas decisões.</p>{scenario.variant === "brazilian_pineapple" && <p>Todas as rodadas: aumento limitado ao pote antes do call. Ante de 1 BB pago apenas pelo big blind, inclusive heads-up.</p>}</div><span className="study-badge">Fichas de treino · sem valor</span></header>
    <div className="study-toolbar">
      <label>Cenário<select value={scenarioId} disabled={busy} onChange={e => reset(e.target.value)}>{scenarios.map(s => <option key={s.id} value={s.id}>{s.title}</option>)}</select></label>
      <label>Modo<select value={mode} onChange={e => setMode(e.target.value)}><option value="guided">Treino guiado</option><option value="challenge">Desafio sem dicas</option><option value="free">Prática variada do módulo</option></select></label>
      <button className="zt-btn-secondary" disabled={busy} onClick={() => reset(mode === "free" ? scenarios[(scenarios.findIndex(s => s.id === scenarioId) + 1) % scenarios.length].id : scenarioId, true)}>Nova distribuição</button>
      <button className="zt-btn-secondary" disabled={busy} onClick={() => reset()}>Repetir esta mão</button>
    </div>
    <p className="study-objective"><strong>Objetivo:</strong> {scenario.objective} <Link to={`/curso/${scenario.lesson}`}>Revisar a aula →</Link></p>
    <p className="text-xs text-felt-400">As cartas que definem o exercício são preservadas; rivais, cartas futuras não fixadas e respostas variam entre distribuições. Blinds de treino: 50 / 100 fichas (1 BB = 100).</p>
    {error && <div className="study-error" role="alert">{error} <button onClick={() => void load(actions)} className="underline">Tentar novamente</button></div>}
    <div className="study-workspace">
      <section className="study-main" aria-label="Mesa simuladora" aria-busy={busy}>
        <div className="study-table-head"><span>{state ? phaseLabel[state.phase] : "Preparando mesa…"}</span><span>{replayIndex !== null ? `Replay · passo ${replayIndex + 1}` : "Distribuição " + seed}</span></div>
        {state && <div className="study-felt">
          <div className="study-opponents">{state.players.slice(1).map((p, index) => <div key={p.id} className={`study-seat ${p.folded ? "is-folded" : ""}`}>
            <div className="study-seat-top"><strong>{p.name}</strong><span>{p.position}</span></div><p>{profileLabel[scenario.profiles[index]]}</p>
            <div className="study-cards">{Array.from({ length: p.card_count }, (_, i) => <PlayingCard key={i} code={p.cards[i]} faceDown={!p.cards[i]} size="sm" />)}</div>
            <p><b>{chips(p.stack)}</b> fichas {p.all_in ? "· ALL-IN" : p.folded ? "· FOLD" : ""}</p>{p.bet > 0 && <span className="study-chip">Na rodada: {chips(p.bet)}</span>}
            {state.finished && p.category && <p>{p.category}</p>}
          </div>)}</div>
          <div className="study-board"><span className="study-pot">Pote {chips(state.pot)}</span>{state.ante > 0 && <small>Ante do BB {chips(state.ante)} · pago {chips(state.ante_paid)} por {state.players.find(p => p.id === state.ante_player_id)?.name}</small>}<div className="study-cards">{Array.from({ length: 5 }, (_, i) => state.board[i] ? <PlayingCard key={i} code={state.board[i]} /> : <span key={i} className="study-card-slot" />)}</div><span className="study-watermark">ZT / ACADEMY</span></div>
          <div className="study-hero"><span className="study-hero-label">VOCÊ · {state.players[0].position}</span><div className="study-cards">{state.players[0].cards.map((c, i) => <PlayingCard key={i} code={c} highlight={state.finished && state.players[0].best_five.includes(c)} />)}</div><p><b>{chips(state.players[0].stack)}</b> fichas · investidas na rodada: {chips(state.players[0].bet)}</p>{(mode === "guided" || state.finished) && state.players[0].category && <p className="text-gold-bright">{state.players[0].category}</p>}</div>
        </div>}
        {!state && <div className="study-loading"><span className="zt-spinner" /> Carregando a distribuição…</div>}
        {data && !data.state.finished && !closed && replayIndex === null && <div className="study-controls">
          <div className="study-price"><span>Para pagar <b>{chips(data.legal.to_call)}</b></span><span>Seu stack <b>{chips(data.state.players[0].stack)}</b></span>{mode === "guided" && data.legal.to_call > 0 && <span>Preço do call <b>{data.legal.call_price_percent.toFixed(2).replace(".", ",")}%</b></span>}</div>
          {mode !== "free" && data.question && <div className="study-question"><label htmlFor="study-answer">{data.question.prompt}</label>{data.question.options.length ? <select id="study-answer" value={answer} onChange={e => setAnswer(e.target.value)}><option value="">Escolha sua leitura</option>{data.question.options.map(o => <option key={o}>{o}</option>)}</select> : <div className="flex items-center gap-2"><input id="study-answer" inputMode="decimal" autoComplete="off" value={answer} onChange={e => setAnswer(e.target.value)} placeholder="Sua resposta" /><span>{data.question.unit}</span></div>}<small>Responda antes de agir. O cálculo/leitura é avaliado separadamente do resultado da mão.</small></div>}
          <div className="study-actions"><button disabled={disabled} onClick={() => act("fold")} className="study-fold">Desistir</button><button disabled={disabled} onClick={() => act(data.legal.can_check ? "check" : "call")}>{data.legal.can_check ? "Check" : `Pagar ${chips(data.legal.to_call)}`}</button>{data.legal.can_all_in && <button disabled={disabled} onClick={() => act("all_in")}>All-in {chips(data.state.players[0].stack)}</button>}</div>
          {data.legal.can_raise && <div className="study-sizing"><label htmlFor="study-total">{data.legal.bet_exists ? "Aumentar para" : "Apostar"} · total na rodada</label><div>{<input id="study-total" type="number" min={data.legal.min_total} max={data.legal.max_total} step="1" value={amount} onChange={e => setAmount(Number(e.target.value))} />}<button disabled={disabled || !Number.isInteger(amount) || amount < data.legal.min_total || amount > data.legal.max_total} onClick={() => act(data.legal.bet_exists ? "raise" : "bet")}>{data.legal.bet_exists ? "Aumentar" : "Apostar"}</button></div><small>Mínimo {chips(data.legal.min_total)} · máximo {chips(data.legal.max_total)} fichas</small></div>}
          {busy && <p role="status">Bots respondendo…</p>}
        </div>}
        {closed && <div className="study-result"><h2>Treino encerrado</h2><p>Suas decisões continuam disponíveis no replay. Salve o caderno para revisar em outra sessão.</p><button className="zt-btn-secondary" onClick={() => setClosed(false)}>Retomar esta mão</button></div>}
        {data?.state.finished && replayIndex === null && <div className="study-result"><h2>Mão encerrada</h2><div className="study-payments">{Object.entries(data.payouts).map(([id, amount]) => <span key={id}>{id === "hero" ? "Você" : id.replace("bot-", "Bot ")}: <strong>{chips(amount)}</strong> fichas recebidas</span>)}</div>{data.pots.map((pot, i) => <p key={i} className="text-xs">{i === 0 ? "Pote principal" : `Pote paralelo ${i}`}: {chips(pot.amount)} · elegíveis: {pot.eligible_players.map(id => id === "hero" ? "você" : id.replace("bot-", "bot ")).join(", ")}</p>)}<p>O resultado da mão e a qualidade da decisão são medidas diferentes.</p></div>}
      </section>
      <aside className="study-side">
        {mode === "guided" && replayIndex === null && <section><p className="study-eyebrow">ANTES DA DECISÃO</p><h2>Um foco por vez</h2><p>{data?.hint ?? scenario.hint}</p><p className="text-xs text-felt-400">Políticas didáticas dos bots, sem promessa de estratégia ótima.</p></section>}
        {latestReview && replayIndex === null && (mode !== "free" || latestReview.answer.trim()) && <section aria-live="polite"><p className="study-eyebrow">DECISÃO {latestReview.decision}</p><h2>{latestReview.correct ? "Leitura conferida" : latestReview.answer ? "Reveja a conta/leitura" : "Resposta não registrada"}</h2><p>{latestReview.prompt}</p><p className="text-gold-bright">Resposta: {latestReview.expected}</p><p>{latestReview.explanation}</p><small>Sua ação: {actionLabel[latestReview.action]}. A nota cobre a pergunta objetiva, não certifica uma estratégia.</small></section>}
        <section><p className="study-eyebrow">CADERNO DE ESTUDO</p><label htmlFor="study-notes">Qual foi seu plano e o que o faria mudar?</label><textarea id="study-notes" rows={5} maxLength={3000} value={notes} onChange={e => setNotes(e.target.value)} placeholder="Range suposto, preço, alternativas e limite da sessão…" /><div className="flex flex-wrap gap-2"><button className="zt-btn-secondary" disabled={!data || busy} onClick={save}>Salvar sessão</button><button className="zt-btn-secondary" disabled={!data || busy} onClick={download}>Exportar</button></div>{saved && <p role="status">{saved}</p>}<small>Registro neste navegador, sem vínculo com carteira ou certificação.</small></section>
        <section><h2>Prática do módulo</h2><p>{records.filter(r => scenarios.some(s => s.id === r.scenario)).length} sessões salvas.</p><p>Refaça o desafio em outro dia e compare suas justificativas.</p><div className="study-saved">{records.filter(r => scenarios.some(s => s.id === r.scenario)).slice(0, 6).map(r => <button key={r.id} disabled={busy} onClick={() => restore(r)}><strong>{scenarios.find(s => s.id === r.scenario)?.title}</strong><small>{new Date(r.when).toLocaleDateString("pt-BR")} · {r.correct}/{r.decisions} leituras conferidas · Reabrir</small></button>)}</div><button className="zt-btn-secondary" disabled={busy || !data} onClick={() => { setClosed(true); save(); }}>Encerrar treino</button></section>
      </aside>
    </div>
    {data && <section className="study-replay"><div className="flex flex-wrap justify-between gap-2"><div><p className="study-eyebrow">REPLAY DA MÃO</p><h2>Volte à informação disponível naquele instante</h2></div><div className="flex gap-2"><button className="zt-btn-secondary" disabled={busy || actions.length === 0} onClick={() => { setClosed(false); void load(actions.slice(0, -1)); }}>Voltar uma decisão e comparar</button>{replayIndex !== null && <button className="zt-btn-primary" onClick={() => setReplayIndex(null)}>Voltar à mesa</button>}</div></div><div className="study-events">{data.events.map((event, i) => <button key={i} className={replayIndex === i ? "is-current" : ""} onClick={() => setReplayIndex(i)}><span>{String(i + 1).padStart(2,"0")} · {phaseLabel[event.state.phase]}</span><strong>{event.actor === "hero" ? "Você" : event.actor.replace("bot-", "Bot ")}: {actionLabel[event.action]}</strong><small>{event.amount > 0 ? `+ ${chips(event.amount)} fichas` : event.setup ? "Preparação do cenário" : ""}</small></button>)}</div><p className="text-xs text-felt-400">O replay preserva as cartas ocultas de cada instante. Voltar uma decisão reconstrói a mesma distribuição até esse ponto; sua nova ação pode mudar a continuação.</p></section>}
    <div className="zt-panel p-4"><Link className="text-gold-soft" to={`/curso/${scenario.lesson}`}>Teoria e exercícios: {lesson?.title} →</Link><CourseSources sourceIds={lesson?.sources} /></div>
  </div>;
}
