/**
 * Bots de estrategia Zero Tilt (externos, via WebSocket).
 *
 * Diferenca para os scripts de carga (full-catalog, estrutura-bots):
 * aqui cada bot LE as proprias cartas (`players[].cards`), o board
 * (`community_cards`), o pote e o valor a pagar (`call_amount`) do
 * `table_state` personalizado e decide via bot/dist (decideAction +
 * avaliador real), em vez de Math.random().
 *
 * Uso:
 *   cd bot && npx tsc   (gera bot/dist; obrigatorio antes)
 *   BOT_USERS="alice:Senha1,bob:Senha1" node scripts/strategy-bots.mjs
 *
 * Env:
 *   BASE_URL     default https://localhost (local) — VPS: https://seu-dominio
 *   TABLE_NAME   trecho do nome da mesa (default "PM"): escolhe a primeira
 *                mesa play com assento livre que case; ou TABLE_ID direto.
 *   BOT_USERS    "login:senha,login:senha" (logins pre-criados, play money).
 *                Sem isso o script nao cria contas (criacao exige verificar
 *                e-mail; rode estrutura-bots-jogar.mjs antes, ou crie manual).
 *   GAME_TYPE    cash_6max (default) | cash_9max (auto se max_players > 7)
 *   HANDS_TARGET maos por bot antes de sair (default 30)
 *   TIMEOUT_MS   teto da sessao (default 20 min)
 *   SELFTEST     "1" roda testes offline (sem rede) e sai.
 *
 * Limites honestos desta versao:
 * - Sem historico de acoes (actionHistory vazio): o c-bet assume IP quando
 *   nao ha agressor previo. Suficiente para validar ranges/odds/avaliador.
 * - Cash game apenas; MTT entra quando o lobby de torneios expor blinds.
 */
import { createRequire } from 'node:module';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const WebSocket = require('ws');

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

const BASE = process.env.BASE_URL ?? 'https://localhost';
const TABLE_NAME = process.env.TABLE_NAME ?? 'PM';
const TABLE_ID_ENV = process.env.TABLE_ID ?? '';
const HANDS_TARGET = Number(process.env.HANDS_TARGET ?? 30);
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS ?? 20 * 60 * 1000);

// ---- Motor de estrategia (compilado). Falha cedo com instrucao clara.
let strategy;
try {
  strategy = await import('../bot/dist/strategy/index.js');
} catch {
  console.error('FALHOU: rode `npx tsc` dentro de bot/ primeiro (gera bot/dist).');
  process.exit(1);
}
const { decideAction, evaluate } = strategy;

// ---- Selftest offline -------------------------------------------------------
if (process.env.SELFTEST === '1') {
  const t = [];
  const eq = (name, got, want) => t.push([name, got === want ? 'ok' : `FALHOU got=${got} want=${want}`]);
  eq('flush', evaluate(['As', 'Ks'], ['Qs', 'Js', '2s', '7d', '3c']).category, 'flush');
  eq('straight', evaluate(['9d', 'Th'], ['Jc', 'Qh', 'Kd', '2s', '3c']).category, 'straight');
  eq('wheel', evaluate(['Ad', '2c'], ['3h', '4s', '5d', 'Kd', 'Qc']).category, 'straight');
  eq('trips', evaluate(['7s', '7h'], ['7d', 'Kc', '2s', '3d', '9c']).category, 'trips');
  eq('two_pair', evaluate(['As', 'Kd'], ['Ah', 'Kc', '2s', '7d', '9c']).category, 'two_pair');
  eq('top_pair', evaluate(['As', 'Qd'], ['Ah', '7c', '2s', '9d', '4c']).category, 'top_pair');
  eq('overpair', evaluate(['Qs', 'Qh'], ['9c', '7d', '2s', '4d', '5c']).category, 'overpair');
  eq('oesd_outs', evaluate(['9d', 'Th'], ['Jc', 'Qh', '2d']).drawOuts >= 8, true);
  eq('fd_outs', evaluate(['As', 'Ks'], ['Qs', 'Js', '2d']).drawOuts >= 9, true);
  eq('blocker', evaluate(['As', '2d'], ['Qs', 'Js', '7s', '9d', '4c']).nutFlushBlocker, true);
  const d1 = decideAction({ gameType: 'cash_6max', street: 'preflop', position: 'BTN', stackBB: 100, effectiveStackBB: 100, potBB: 1.5, betToCallBB: 0, myHand: ['As', 'Ks'], board: [], actionHistory: [], opponents: [] });
  eq('btn_aks_raise', d1.action, 'raise');
  const d2 = decideAction({ gameType: 'cash_6max', street: 'flop', position: 'BTN', stackBB: 100, effectiveStackBB: 100, potBB: 10, betToCallBB: 8, myHand: ['7d', '2c'], board: ['Ah', 'Kc', 'Qs'], actionHistory: [], opponents: [] });
  eq('air_fold', d2.action, 'fold');
  const d3 = decideAction({ gameType: 'cash_6max', street: 'flop', position: 'BTN', stackBB: 100, effectiveStackBB: 100, potBB: 10, betToCallBB: 2, myHand: ['As', 'Ks'], board: ['Qs', 'Js', '2d'], actionHistory: [], opponents: [] });
  eq('nfd_call', d3.action, 'call');
  let fail = 0;
  for (const [n, r] of t) { console.log(`${r === 'ok' ? 'ok' : 'FALHA'} ${n}${r === 'ok' ? '' : ' ' + r}`); if (r !== 'ok') fail++; }
  process.exit(fail ? 1 : 0);
}

// ---- HTTP -------------------------------------------------------------------
async function api(path, { method = 'GET', body, token } = {}) {
  const headers = { Accept: 'application/json', ...(body ? { 'Content-Type': 'application/json' } : {}), ...(token ? { Authorization: `Bearer ${token}` } : {}) };
  const res = await fetch(`${BASE}${path}`, { method, headers, body: body ? JSON.stringify(body) : undefined });
  const text = await res.text();
  let json = null;
  try { json = text ? JSON.parse(text) : null; } catch { json = text; }
  if (!res.ok) throw new Error(`${method} ${path} -> ${res.status} ${JSON.stringify(json).slice(0, 300)}`);
  return json;
}

// ---- Posicao por assento ----------------------------------------------------
// Ordem apos o dealer: SB, BB, UTG, [UTG+1], [MP/HJ], CO, BTN(dealer).
function positionBySeat(players, myId) {
  const seated = [...players].sort((a, b) => (a.seat ?? 0) - (b.seat ?? 0));
  const n = seated.length;
  const dealerIdx = Math.max(0, seated.findIndex((p) => p.is_dealer));
  const names = n <= 2 ? ['SB', 'BTN']
    : n === 3 ? ['SB', 'BB', 'BTN']
    : n === 4 ? ['SB', 'BB', 'UTG', 'BTN']
    : n === 5 ? ['SB', 'BB', 'UTG', 'CO', 'BTN']
    : n === 6 ? ['SB', 'BB', 'UTG', 'HJ', 'CO', 'BTN']
    : n === 7 ? ['SB', 'BB', 'UTG', 'MP', 'HJ', 'CO', 'BTN']
    : n === 8 ? ['SB', 'BB', 'UTG', 'UTG+1', 'MP', 'HJ', 'CO', 'BTN']
    : ['SB', 'BB', 'UTG', 'UTG+1', 'MP1', 'MP2', 'HJ', 'CO', 'BTN'];
  // i=0 e o dealer (BTN); depois: SB, BB, UTG... na ordem do jogo.
  const order = ['BTN', ...names.slice(0, n - 1)];
  for (let i = 0; i < n; i++) {
    const p = seated[(dealerIdx + i) % n];
    if (String(p.id) === String(myId)) return order[i];
  }
  return 'BTN';
}

const STAGE = { preflop: 'preflop', flop: 'flop', turn: 'turn', river: 'river', showdown: 'river' };

function toGameState(msg, me, gameType, bigBlind) {
  const street = STAGE[msg.stage] ?? 'preflop';
  const pot = (msg.pots?.[0]?.amount ?? 0) / bigBlind;
  const toCall = (msg.call_amount ?? 0) / bigBlind;
  const myChips = (me.chips ?? 0) / bigBlind;
  const oppStacks = (msg.players ?? []).filter((p) => String(p.id) !== String(me.id) && !p.folded).map((p) => (p.chips ?? 0) / bigBlind);
  return {
    gameType,
    street,
    position: positionBySeat(msg.players ?? [], me.id),
    stackBB: myChips,
    effectiveStackBB: Math.min(myChips, ...oppStacks.filter((s) => s > 0)),
    potBB: pot,
    betToCallBB: Math.min(toCall, myChips),
    myHand: me.cards ?? [],
    board: msg.community_cards ?? [],
    actionHistory: [],
    opponents: (msg.players ?? []).filter((p) => String(p.id) !== String(me.id)).map((p) => ({ position: 'BTN', stackBB: (p.chips ?? 0) / bigBlind })),
  };
}

function toWireAction(decided, msg) {
  const avail = new Set(msg.available_actions ?? []);
  const clamp = (bb, bigBlind) => Math.max(msg.minimum_wager ?? 0, Math.min(msg.maximum_wager ?? Infinity, Math.round(bb * bigBlind)));
  let { action, sizeBB } = decided;
  if (action === 'raise' && !avail.has('raise') && avail.has('bet')) action = 'bet';
  if (action === 'bet' && !avail.has('bet') && avail.has('raise')) action = 'raise';
  if ((action === 'raise' || action === 'bet') && !avail.has(action)) action = toCallPreferred(msg);
  if (action === 'call' && !avail.has('call')) action = avail.has('check') ? 'check' : 'fold';
  if (action === 'check' && !avail.has('check')) action = avail.has('call') ? 'call' : 'fold';
  if (action === 'allin' && !avail.has('allin')) action = avail.has('raise') ? 'raise' : (avail.has('call') ? 'call' : 'fold');
  if (!avail.has(action)) action = 'fold';
  const amount = action === 'raise' || action === 'bet' ? clamp(sizeBB ?? 0, this?.bigBlind ?? 0) : 0;
  return { action, amount };
}

function toCallPreferred(msg) {
  const avail = new Set(msg.available_actions ?? []);
  if (avail.has('check')) return 'check';
  return 'fold';
}

// ---- Main -------------------------------------------------------------------
async function main() {
  const raw = process.env.BOT_USERS ?? '';
  const creds = raw.split(',').map((s) => s.trim()).filter(Boolean).map((s) => {
    const i = s.indexOf(':');
    return { login: s.slice(0, i), password: s.slice(i + 1) };
  });
  if (!creds.length) throw new Error('Defina BOT_USERS="login:senha,login:senha" (contas play ja criadas).');

  const tables = await api('/api/lobby/tables?mode=play');
  const pool = tables.filter((t) => !TABLE_ID_ENV || String(t.id) === TABLE_ID_ENV)
    .filter((t) => TABLE_ID_ENV || String(t.name ?? '').includes(TABLE_NAME));
  if (!pool.length) throw new Error(`Nenhuma mesa play com "${TABLE_NAME}" (ou TABLE_ID).`);
  const table = pool[0];
  const bigBlind = Number(table.big_blind);
  if (!bigBlind) throw new Error('Mesa sem big_blind no lobby.');
  const gameType = (process.env.GAME_TYPE ?? (table.max_players > 7 ? 'cash_9max' : 'cash_6max'));
  console.log(`Mesa ${table.name} bb=${bigBlind} max=${table.max_players} modo=${gameType} bots=${creds.length}`);

  const tokens = [];
  for (const c of creds) {
    const email = c.login.includes('@') ? c.login : `${c.login}@test.local`;
    const r = await api('/api/auth/login', { method: 'POST', body: { email, password: c.password } });
    if (!r.token) throw new Error(`login ${c.login} sem token`);
    tokens.push({ login: c.login, token: r.token });
  }
  for (const u of tokens) {
    try {
      await api('/api/lobby/join', { method: 'POST', token: u.token, body: { table_id: table.id, buy_in: table.min_buy_in, wallet_mode: 'play' } });
    } catch (e) { console.log(`join ${u.login}: ${String(e.message).slice(0, 120)}`); }
  }

  const t0 = Date.now();
  let done = false;
  const stats = new Map(tokens.map((u) => [u.login, { hands: 0, actions: 0 }]));

  async function runBot(u) {
    const ticket = (await api(`/api/lobby/tables/${table.id}/ws-ticket`, { method: 'POST', token: u.token })).ticket;
    const base = new URL(BASE);
    base.protocol = base.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${base.origin}/ws/game/${table.id}?ticket=${ticket}`, { rejectUnauthorized: false });
    let playerId = null;
    let lastFinish = false;
    const st = stats.get(u.login);
    ws.on('open', () => ws.send(JSON.stringify({ type: 'get_table_info' })));
    ws.on('message', (rawMsg) => {
      if (done) return;
      let msg;
      try { msg = JSON.parse(String(rawMsg)); } catch { return; }
      if (msg.type === 'welcome') { playerId = msg.player_id; return; }
      if (msg.type !== 'table_state') return;
      if (msg.is_finished && !lastFinish) { st.hands++; lastFinish = true; }
      if (!msg.is_finished) lastFinish = false;
      if (st.hands >= HANDS_TARGET || Date.now() - t0 > TIMEOUT_MS) return;
      const me = (msg.players ?? []).find((p) => String(p.id) === String(playerId));
      if (!me || me.folded || !(msg.available_actions ?? []).length) return;
      if (!me.cards || me.cards.length < 2) return;
      try {
        const gs = toGameState(msg, me, gameType, bigBlind);
        const decided = decideAction(gs);
        const wire = toWireAction.call({ bigBlind }, decided, msg);
        ws.send(JSON.stringify({ type: 'action', action: wire.action, amount: wire.amount }));
        st.actions++;
        if (st.actions % 20 === 1) console.log(`${u.login} ${gs.street} ${gs.position} ${me.cards.join(' ')}|${(msg.community_cards ?? []).join(' ')} -> ${wire.action} ${wire.amount} (${decided.reasoning})`);
      } catch (e) { console.log(`${u.login} erro decisao: ${String(e.message).slice(0, 120)}`); }
    });
    await sleep(TIMEOUT_MS);
    try { ws.close(); } catch {}
  }

  await Promise.all(tokens.map(runBot));
  done = true;
  for (const u of tokens) {
    try { await api('/api/lobby/leave', { method: 'POST', token: u.token, body: { table_id: table.id } }); } catch {}
  }
  console.log('Resumo: ' + [...stats].map(([k, v]) => `${k} maos=${v.hands} acoes=${v.actions}`).join(' | '));
}

main().catch((e) => { console.error('FALHOU:', e.message ?? e); process.exit(1); });
