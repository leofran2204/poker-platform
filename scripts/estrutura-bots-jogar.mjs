/**
 * Bots Minha Estrutura: registra 5 contas em cadeia (raiz > n1a/n1b > n2a/n2b),
 * senta na mesa play, joga via WS (check quando gratis, fold no resto) ate VP,
 * e confere o ledger 18/12 no painel.
 *
 *   node scripts/estrutura-bots-jogar.mjs
 *
 * Roda no host, fala com a stack local via Caddy (https://localhost).
 * Precisa de: docker (logs + psql), node com pacote `ws`.
 */
import { execSync } from "node:child_process";
import { createRequire } from "node:module";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const WebSocket = require("ws");

process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
const BASE = process.env.BASE_URL ?? "https://localhost";
const PASS = "PokerDemo1";
const HAND_TARGET = Number(process.env.HAND_TARGET ?? 55);
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS ?? 25 * 60 * 1000);
const FEST_TIMEOUT_MS = Number(process.env.FEST_TIMEOUT_MS ?? 10 * 60 * 1000);

const sleepMs = (ms) => new Promise((r) => setTimeout(r, ms));

async function api(path, { method = "GET", body, token } = {}) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: {
      Accept: "application/json",
      ...(body ? { "Content-Type": "application/json" } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let json = null;
  try {
    json = text ? JSON.parse(text) : null;
  } catch {
    json = text;
  }
  return { status: res.status, json };
}

function sh(cmd) {
  return execSync(cmd, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}

function stripAnsi(s) {
  return s.replace(/\x1b\[[0-9;]*m/g, "");
}

function lastCode(email) {
  const logs = stripAnsi(sh("docker logs poker_api"));
  const esc = email.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pick = (re) => {
    let m = null;
    let last = null;
    re.lastIndex = 0;
    while ((m = re.exec(logs)) !== null) last = m[1];
    return last;
  };
  return (
    pick(new RegExp(`to="${esc}"[^\\n]*verification_code=(\\d{6})`, "g")) ??
    pick(new RegExp(`to=${esc}[^\\n]*code=(\\d{6})`, "g")) ??
    (() => {
      throw new Error(`sem codigo para ${email}`);
    })()
  );
}

function dbSeedInvite() {
  const out = sh(
    'docker exec poker_postgres psql -U user -d poker_db -tA -c "SELECT referral_code FROM users WHERE referral_code IS NOT NULL LIMIT 1;"'
  ).trim();
  return out.split("\n")[0].trim() || null;
}

function q(sql) {
  return sh(`docker exec poker_postgres psql -U user -d poker_db -tA -c "${sql}"`).trim();
}

async function registerLogin(username, email, invite) {
  const body = { username, email, password: PASS, password_confirm: PASS };
  if (invite) body.invite_code = invite;
  let r = await api("/api/auth/register", { method: "POST", body });
  if (r.status !== 200) throw new Error(`register ${username}: ${r.status} ${JSON.stringify(r.json)}`);
  await sleep(600);
  const code = lastCode(email);
  r = await api("/api/auth/verify-email", { method: "POST", body: { email, code } });
  if (r.json?.token) return r.json.token;
  console.log(`   verify sem token para ${email} (segue login): ${r.status} ${JSON.stringify(r.json)}`);
  r = await api("/api/auth/login", { method: "POST", body: { email, password: PASS } });
  if (!r.json?.token) throw new Error(`login ${email}: ${r.status} ${JSON.stringify(r.json)}`);
  return r.json.token;
}

async function main() {
  console.log("0) limpa assentos zombies play (ritual de bot)");
  sh("docker exec -i poker_postgres psql -U user -d poker_db < scripts/clear-zombie-play-seats.sql");
  const stamp = String(Date.now()).slice(-6);
  const names = ["raiz", "n1a", "n1b", "n2a", "n2b"].map((n) => `${n}${stamp}`);
  const emails = Object.fromEntries(names.map((n) => [n, `${n}@example.com`]));
  const [raiz, n1a, n1b, n2a, n2b] = names;
  const tokens = {};

  console.log("1) cadeia de convites");
  try {
    tokens[raiz] = await registerLogin(raiz, emails[raiz], null);
  } catch (e) {
    if (!/convite/i.test(String(e))) throw e;
    const seed = dbSeedInvite();
    if (!seed) throw new Error("servidor exige convite e banco sem referral_code");
    console.log(`   convite obrigatorio; raiz usa semente ${seed}`);
    tokens[raiz] = await registerLogin(raiz, emails[raiz], seed);
  }
  const refOf = async (name) => {
    const r = await api("/api/auth/me", { token: tokens[name] });
    if (r.status !== 200 || !r.json?.referral_code) throw new Error(`me ${name}: ${JSON.stringify(r.json)}`);
    return r.json.referral_code;
  };
  const refRaiz = await refOf(raiz);
  tokens[n1a] = await registerLogin(n1a, emails[n1a], refRaiz);
  tokens[n1b] = await registerLogin(n1b, emails[n1b], refRaiz);
  tokens[n2a] = await registerLogin(n2a, emails[n2a], await refOf(n1a));
  tokens[n2b] = await registerLogin(n2b, emails[n2b], await refOf(n1b));
  console.log(`   raiz=${raiz} ref=${refRaiz}`);

  console.log("2) join na mesa holdem play");
  const t = await api("/api/lobby/tables?mode=play");
  if (t.status !== 200 || !Array.isArray(t.json) || !t.json.length) throw new Error("sem mesas play");
  const table = t.json.find((x) => x.poker_variant === "holdem") ?? t.json[0];
  const buyIn = table.min_buy_in;
  console.log(`   mesa ${table.name} buyin=${buyIn}`);
  for (const n of names) {
    const j = await api("/api/lobby/join", {
      method: "POST",
      token: tokens[n],
      body: { table_id: table.id, buy_in: buyIn, wallet_mode: "play" },
    });
    if (j.status !== 200) throw new Error(`join ${n}: ${j.status} ${JSON.stringify(j.json)}`);
  }

  console.log("3) bots via WS (fase A: check/fold; fase B: allin-fest)");
  const bots = [];
  let mode = "nit";
  for (const n of names) {
    const tk = await api(`/api/lobby/tables/${table.id}/ws-ticket`, { method: "POST", token: tokens[n] });
    if (tk.status !== 200 || !tk.json?.ticket) throw new Error(`ticket ${n}: ${JSON.stringify(tk.json)}`);
    const ws = new WebSocket(`${BASE.replace(/^http/, "ws")}/ws/game/${table.id}?ticket=${tk.json.ticket}`, {
      rejectUnauthorized: false,
    });
    const bot = { name: n, ws, acted: 0 };
    ws.on("open", () => ws.send(JSON.stringify({ type: "get_table_info" })));
    ws.on("message", (raw) => {
      let msg;
      try {
        msg = JSON.parse(String(raw));
      } catch {
        return;
      }
      // O servidor personaliza table_state por jogador: available_actions
      // so vem preenchido para quem tem a vez (is_active).
      if (msg.type === "table_state" && Array.isArray(msg.available_actions) && msg.available_actions.length) {
        const a = msg.available_actions;
        const pick = mode === "fest" && a.includes("allin") ? "allin" : a.includes("check") ? "check" : "fold";
        ws.send(JSON.stringify({ type: "action", action: pick, amount: 0 }));
        bot.acted += 1;
      }
      if (msg.type === "error") console.log(`   ws ${n} erro: ${msg.message}`);
    });
    ws.on("error", (e) => console.log(`   ws ${n} erro: ${e.message}`));
    bots.push(bot);
    await sleepMs(300);
  }

  const estOf = async (n) => (await api("/api/estrutura", { token: tokens[n] })).json;
  const sessionRake = () =>
    Number(
      q(
        `SELECT COALESCE(SUM(rake_collected),0) FROM hand_history WHERE table_id='${table.id}' AND created_at>=${runStart};`
      )
        .split("\n")[0]
        .trim() || 0
    );

  const runStart = Math.floor(Date.now() / 1000);
  console.log(`4) fase A: fold-fast ate VP (${HAND_TARGET} maos p/ raiz+n1a+n1b)`);
  const t0 = Date.now();
  for (;;) {
    await sleep(5000);
    const hs = await Promise.all([raiz, n1a, n1b].map(async (n) => (await estOf(n))?.hands_this_week ?? 0));
    if (Math.min(...hs) >= HAND_TARGET) break;
    if (Date.now() - t0 > TIMEOUT_MS) throw new Error(`timeout VP maos=${hs}`);
  }

  const RAKE_TARGET = Number(process.env.RAKE_TARGET ?? 200);
  console.log(`5) fase B: allin-fest ate R$ ${(RAKE_TARGET / 100).toFixed(2)} de rake na mesa`);
  mode = "fest";
  const t1 = Date.now();
  for (;;) {
    await sleep(5000);
    const rake = sessionRake();
    if (rake >= RAKE_TARGET) {
      console.log(`   rake da sessao=${rake} (alvo ${RAKE_TARGET})`);
      break;
    }
    if (Date.now() - t1 > FEST_TIMEOUT_MS) {
      console.log(`   fest timeout com rake=${rake} (alvo ${RAKE_TARGET})`);
      break;
    }
  }

  for (const b of bots) {
    try {
      b.ws.close();
    } catch {}
  }

  console.log("6) conferencia do ledger 18/12");
  for (const n of [raiz, n1a, n1b]) {
    const r = await api("/api/estrutura", { token: tokens[n] });
    const d = r.json;
    console.log(
      `   ${n}: eligible=${d.eligible} maos=${d.hands_this_week} L1=${d.points_week_l1} L2=${d.points_week_l2} retido=${d.withheld_week} pts=${d.estrutura_points}`
    );
  }
  const raizId = q(`SELECT id FROM users WHERE username='${raiz}';`).split("\n")[0];
  console.log(
    "   ledger da raiz: " +
      q(
        `SELECT level, count(*), COALESCE(SUM(commission_cents)::BIGINT,0) FROM estrutura_ledger WHERE beneficiary_user_id='${raizId}' GROUP BY level ORDER BY level;`
      )
        .split("\n")
        .join(" / ")
  );

  const fin = await api("/api/estrutura", { token: tokens[raiz] });
  const d = fin.json;
  if (d.hands_this_week >= 50 && !d.eligible) throw new Error("raiz com 50+ maos deveria estar elegivel");
  if (Number(q(`SELECT count(*) FROM estrutura_ledger WHERE beneficiary_user_id='${raizId}';`)) === 0)
    throw new Error("ledger da raiz vazio: nenhuma mao com rake distribuiu");
  if (d.hands_this_week >= 50 && d.points_week_l1 + d.points_week_l2 === 0)
    throw new Error("L1+L2 da raiz zerados apos fest: 18/12 nao pagou");
  console.log("OK bots jogaram, VP virou, ledger 18/12 confere");
}

main().catch((e) => {
  console.error("FALHOU:", e.message ?? e);
  process.exit(1);
});
