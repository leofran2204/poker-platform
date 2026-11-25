#!/usr/bin/env node
/**
 * Ritual Play Money no site publicado (etapa 2 do plano do proprietário de 23/11/2026).
 *
 * Fases (PHASES, padrão "cash,mtt,real"):
 *  - cash: as 4 modalidades com mesa cheia (CASH_TABLES), compra do catálogo e
 *    HANDS_PER_TABLE mãos por mesa; quem quebra dá lugar a uma reserva com saldo.
 *  - mtt: o torneio MTT_ID (rebuy e add-on ligados pelo admin), campo cheio,
 *    1 rebuy e 1 add-on por bot, até o campeão.
 *  - real: carteira Jogo Real de uma conta de teste, sem dinheiro.
 *
 * Cada mão segue um roteiro (all-in multiway, all-in heads-up, aumento e
 * reaumento, check-raise, showdown passivo, tempo esgotado, tempo extra; no
 * Brazilian Pineapple também troca, manutenção e descarte) e o relatório conta
 * só o que o servidor confirmou. Confere a carteira de cada conta antes e depois
 * (cash: variação = cashback do Loss Deflator − rake; torneio: prêmios = prize pool).
 *
 * Contas com e-mail temporário (Mail.tm) ficam em artifacts/ritual/accounts.json
 * (ignorado pelo git) e são reaproveitadas na rodada seguinte.
 * Na demo, o cash exige o IP de saída em SEAT_IP_GUARD_EXEMPT_IPS durante a janela.
 *
 *   ALLOW_TEMP_MAIL=true MTT_ID=<uuid> node scripts/live-sim-full-ritual.mjs
 */
import { randomBytes } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(join(ROOT, "scripts", "live-sim-full-ritual.mjs"));
const WebSocket = require("ws");

const BASE_URL = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const PHASES = new Set((process.env.PHASES ?? "cash,mtt,real").split(",").map((s) => s.trim()));
const HANDS_PER_TABLE = Number(process.env.HANDS_PER_TABLE ?? 150);
const POOL_SIZE = Number(process.env.POOL_SIZE ?? 52);
const MTT_ID = process.env.MTT_ID ?? "";
const CASH_TABLES = (
  process.env.CASH_TABLES ??
  "PM · NL 0,25|PM · Texas Short Deck 0,50/0,50|PM · Omaha 0,50/0,50|PM · Brazilian Pineapple 0,50"
).split("|");
// Cadastro só com convite: a primeira conta usa INVITE_CODE e as demais o convite dela.
const INVITE_CODE = process.env.INVITE_CODE ?? "";
const MAIL_API = "https://api.mail.tm";
const USE_MAIL = process.env.ALLOW_TEMP_MAIL === "true";
const RUN_ID = Date.now().toString(36).slice(-6);
const OUT_DIR = process.env.OUT_DIR ?? join(ROOT, "artifacts", "ritual");
const ACCOUNTS_PATH = join(OUT_DIR, "accounts.json");
const REPORT_PATH = process.env.REPORT_PATH ?? join(OUT_DIR, `report-${RUN_ID}.json`);
const LIMIT_GAP_MS = Number(process.env.LIMIT_GAP_MS ?? 2200);
const TURN_MS = 15_000;
const EXTRA_MS = 5_000;
// Só estes roteiros, em rodízio (ex.: SCENARIO_ONLY=allin_hu ou allin_hu,allin_mw).
const SCENARIO_ONLY = process.env.SCENARIO_ONLY ?? "";
const MTT_START_WAIT_MS = Number(process.env.MTT_START_WAIT_MIN ?? 20) * 60_000;
const MTT_MAX_MS = Number(process.env.MTT_MAX_MIN ?? 180) * 60_000;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const log = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);

// ─── HTTP ────────────────────────────────────────────────────────────────────

async function rawFetch(url, options = {}, retries = 8) {
  for (let attempt = 0; ; attempt += 1) {
    try {
      const res = await fetch(url, {
        ...options,
        signal: AbortSignal.timeout(30_000),
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
          "User-Agent": "live-sim/2.0",
          ...(options.headers ?? {}),
        },
      });
      const text = await res.text();
      let data = null;
      try {
        data = text ? JSON.parse(text) : null;
      } catch {
        data = text;
      }
      if (res.status === 429 && attempt < retries) {
        const retryAfter = Number(res.headers.get("retry-after") ?? 30);
        await sleep(Math.max(2, retryAfter) * 1000);
        continue;
      }
      return { ok: res.ok, status: res.status, data, text };
    } catch (err) {
      if (attempt >= retries) throw err;
      await sleep(3000 * (attempt + 1));
    }
  }
}

// O rate limit da API (30/min por IP) vale para autenticação, ticket do WebSocket e pagamentos.
const LIMITED = [/^\/api\/auth\/(login|register|verify-email|resend-verification)/, /\/ws-ticket$/, /^\/api\/payments\//];
let nextLimitedAt = 0;
async function throttle(path) {
  if (!LIMITED.some((r) => r.test(path))) return;
  const now = Date.now();
  const at = Math.max(now, nextLimitedAt);
  nextLimitedAt = at + LIMIT_GAP_MS;
  if (at > now) await sleep(at - now);
}

async function send(path, { method = "GET", body, token } = {}) {
  await throttle(path);
  return rawFetch(`${BASE_URL}${path}`, {
    method,
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
}

async function login(user) {
  const res = await send("/api/auth/login", {
    method: "POST",
    body: { email: user.email, password: user.password },
  });
  if (!res.ok || !res.data?.token) throw new Error(`login ${user.username} -> ${res.status}`);
  user.token = res.data.token;
}

/** Chamada autenticada; renova o token uma vez quando expira. */
async function call(path, { method = "GET", body, user } = {}) {
  let res = await send(path, { method, body, token: user?.token });
  if (res.status === 401 && user) {
    await login(user);
    res = await send(path, { method, body, token: user.token });
  }
  return res;
}

async function must(path, opts = {}) {
  const res = await call(path, opts);
  if (!res.ok) {
    throw new Error(`${opts.method ?? "GET"} ${path} -> ${res.status}: ${String(res.text).slice(0, 200)}`);
  }
  return res.data;
}

function wsUrl(tableId, ticket) {
  const url = new URL(BASE_URL);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = `/ws/game/${encodeURIComponent(tableId)}`;
  url.search = new URLSearchParams({ ticket }).toString();
  return url.toString();
}

// ─── Contas com e-mail temporário ───────────────────────────────────────────

async function createMailbox(index) {
  const domainsRes = await rawFetch(`${MAIL_API}/domains`, { method: "GET" });
  const list = Array.isArray(domainsRes.data) ? domainsRes.data : domainsRes.data?.["hydra:member"] ?? [];
  const domain = list.find((d) => d.isActive)?.domain ?? list[0]?.domain;
  if (!domain) throw new Error("mail.tm: no domain");
  const address = `tb${RUN_ID}${index}${randomBytes(2).toString("hex")}@${domain}`;
  const password = `Sim${randomBytes(6).toString("hex")}A1`;
  const created = await rawFetch(`${MAIL_API}/accounts`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  if (!created.ok) throw new Error(`mail.tm account ${created.status}`);
  const tok = await rawFetch(`${MAIL_API}/token`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  if (!tok.ok || !tok.data?.token) throw new Error(`mail.tm token ${tok.status}`);
  return { email: address, password, mailToken: tok.data.token };
}

function extractCode(content) {
  const contextual = String(content).match(/c[oó]digo(?:\s+de\s+verifica[cç][aã]o)?[\s\S]{0,400}?(\d{6})/i);
  if (contextual) return contextual[1];
  return String(content).match(/(?<!\d)(\d{6})(?!\d)/)?.[1] ?? null;
}

async function waitCode(user) {
  const deadline = Date.now() + 150_000;
  while (Date.now() < deadline) {
    const inbox = await rawFetch(`${MAIL_API}/messages?page=1`, {
      headers: { Authorization: `Bearer ${user.mailToken}` },
    });
    const members = Array.isArray(inbox.data) ? inbox.data : inbox.data?.["hydra:member"] ?? [];
    const hit = members.find((m) => /Zero Tilt|verific/i.test(`${m.subject} ${m.intro}`));
    if (hit) {
      const detail = await rawFetch(`${MAIL_API}/messages/${hit.id}`, {
        headers: { Authorization: `Bearer ${user.mailToken}` },
      });
      const d = detail.data ?? {};
      const code = extractCode([d.subject, d.text, ...(Array.isArray(d.html) ? d.html : [d.html])].join("\n"));
      if (code) return code;
    }
    await sleep(3000);
  }
  throw new Error(`no verify code for ${user.email}`);
}

function loadAccounts() {
  if (!existsSync(ACCOUNTS_PATH)) return [];
  try {
    return JSON.parse(readFileSync(ACCOUNTS_PATH, "utf8"));
  } catch {
    return [];
  }
}

function saveAccounts(accounts) {
  mkdirSync(OUT_DIR, { recursive: true });
  writeFileSync(
    ACCOUNTS_PATH,
    JSON.stringify(accounts.map(({ username, email, password, mailToken }) => ({ username, email, password, mailToken })), null, 2),
  );
}

async function registerAccount(index, invite) {
  // Sem Mail.tm só funciona contra uma API local com REQUIRE_EMAIL_VERIFICATION=false.
  const box = USE_MAIL
    ? await createMailbox(index)
    : { email: `tb${RUN_ID}${index}@example.test`, password: `Sim${randomBytes(6).toString("hex")}A1`, mailToken: null };
  const user = { ...box, username: `tb${RUN_ID}${String(index).padStart(2, "0")}` };
  let reg;
  for (let attempt = 0; attempt < 4; attempt += 1) {
    reg = await send("/api/auth/register", {
      method: "POST",
      body: {
        email: user.email,
        password: user.password,
        password_confirm: user.password,
        username: user.username,
        date_of_birth: "1990-05-10",
        over_18: true,
        ...(invite ? { invite_code: invite } : {}),
      },
    });
    if (reg.ok) break;
    if (reg.status === 409) {
      user.username = `tb${RUN_ID}${randomBytes(2).toString("hex")}`;
      continue;
    }
    throw new Error(`register -> ${reg.status}: ${String(reg.text).slice(0, 200)}`);
  }
  if (!reg.data?.token) {
    if (!USE_MAIL) throw new Error("a API exige e-mail verificado: use ALLOW_TEMP_MAIL=true");
    let code;
    try {
      code = await waitCode(user);
    } catch {
      await send("/api/auth/resend-verification", { method: "POST", body: { email: user.email } });
      code = await waitCode(user);
    }
    const ver = await send("/api/auth/verify-email", { method: "POST", body: { email: user.email, code } });
    if (!ver.ok) throw new Error(`verify -> ${ver.status}: ${String(ver.text).slice(0, 160)}`);
  }
  await login(user);
  return user;
}

async function provisionPool(n) {
  const saved = loadAccounts();
  const pool = [];
  for (const acc of saved) {
    if (pool.length >= n) break;
    try {
      await login(acc);
      pool.push(acc);
    } catch (e) {
      log(`conta salva ignorada ${acc.username}: ${e.message}`);
    }
  }
  log(`contas reaproveitadas: ${pool.length}`);
  let index = saved.length;
  let failures = 0;
  let invite = INVITE_CODE;
  const chain = saved[0] ?? null;
  if (chain) {
    try {
      if (!chain.token) await login(chain);
      invite = (await must("/api/auth/me", { user: chain })).referral_code ?? invite;
    } catch {}
  }
  const save = () => saveAccounts([...saved.filter((s) => !pool.includes(s)), ...pool]);
  const createOne = async () => {
    const my = index++;
    const user = await registerAccount(my, invite);
    pool.push(user);
    save();
    log(`conta ${pool.length}/${n} ${user.username}`);
    return user;
  };
  // A primeira conta nova vira a madrinha das demais quando não há conta salva.
  while (!chain && pool.length === 0 && failures < 4) {
    try {
      const first = await createOne();
      invite = (await must("/api/auth/me", { user: first })).referral_code ?? invite;
    } catch (e) {
      failures += 1;
      log(`cadastro falhou (${failures}): ${e.message}`);
    }
  }
  // Os e-mails demoram ~1 min: vários cadastros esperam o código ao mesmo tempo.
  let reserved = pool.length;
  const worker = async () => {
    while (reserved < n && failures < Math.min(n, 8)) {
      reserved += 1;
      try {
        await createOne();
      } catch (e) {
        reserved -= 1;
        failures += 1;
        log(`cadastro falhou (${failures}): ${e.message}`);
      }
    }
  };
  await Promise.all(Array.from({ length: Number(process.env.PROVISION_WORKERS ?? 8) }, worker));
  saveAccounts([...saved.filter((s) => !pool.includes(s)), ...pool]);
  return pool;
}

async function balances(user) {
  const me = await must("/api/auth/me", { user });
  return {
    cash: Number(me.balance_pm_cash ?? 0),
    mtt: Number(me.balance_pm_mtt ?? 0),
    real: Number(me.balance_real ?? 0),
    reset: me.last_pm_reset_date ?? null,
    mode: me.preferred_wallet_mode ?? null,
  };
}

// ─── Mesa: roteiro por mão, decisões e cobertura ────────────────────────────

const COVERAGE_KEYS = [
  "hands", "showdowns", "allin", "allin_multiway", "side_pots", "raise", "reraise", "checkraise",
  "tie", "timeout_fold", "timeout_check", "extra_time", "loss_deflator", "deflator_prevented_elimination",
  "pineapple_swap", "pineapple_keep", "pineapple_discard", "pineapple_auto_discard",
];

// allin_mw: o menor stack vai all-in, até três pagam e seguem apostando (pote paralelo);
// allin_3: três all-ins; allin_hu: um all-in e um que paga (Loss Deflator heads-up).
const CASH_CYCLE = ["mixed", "reraise", "checkraise", "showdown", "allin_mw", "mixed", "extra", "allin_hu", "showdown", "timeout", "reraise", "checkraise", "allin_3", "allin_hu"];
const CASH_QUOTAS = { allin_mw: 6, allin_3: 2, allin_hu: 4, timeout: 4, extra: 3 };
const MTT_CYCLE = ["mixed", "allin_mw", "reraise", "showdown", "allin_3", "checkraise", "allin_hu", "mixed", "timeout", "extra"];
const MTT_QUOTAS = { timeout: 2, extra: 2 };

function makeTable(table, { cycle, quotas, kind }) {
  return {
    id: table.id,
    name: table.name,
    kind,
    variant: table.poker_variant,
    max: table.max_players,
    bb: Number(table.big_blind ?? 0),
    buyIn: Number(table.min_buy_in ?? 0),
    bots: new Map(),
    seatedEver: new Set(),
    observer: null,
    hand: null,
    syntheticHand: 0,
    handsDone: 0,
    handIds: [],
    cov: Object.fromEntries(COVERAGE_KEYS.map((k) => [k, 0])),
    scen: {},
    quotas: { ...quotas },
    cycle,
    errors: [],
    deflator: [],
    aggressive: false,
    stopping: false,
    onHandEnd: null,
    lastHandAt: Date.now(),
  };
}

function pickScenario(t) {
  if (SCENARIO_ONLY) {
    const list = SCENARIO_ONLY.split(",");
    const only = list[t.handsDone % list.length];
    t.scen[only] = (t.scen[only] ?? 0) + 1;
    return only;
  }
  if (t.aggressive) return t.handsDone % 2 === 0 ? "allin_mw" : "allin_hu";
  for (let i = 0; i < t.cycle.length; i += 1) {
    const s = t.cycle[(t.handsDone + i) % t.cycle.length];
    if (t.quotas[s] === undefined || t.quotas[s] > 0) {
      if (t.quotas[s] !== undefined) t.quotas[s] -= 1;
      t.scen[s] = (t.scen[s] ?? 0) + 1;
      return s;
    }
  }
  return "mixed";
}

function startHand(t, msg, hid) {
  const scenario = pickScenario(t);
  const dealt = new Set((msg.players ?? []).filter((p) => Number(p.chips) + Number(p.bet ?? 0) > 0).map((p) => p.id));
  t.hand = {
    id: hid,
    n: t.handsDone,
    scenario,
    stage: null,
    cbm: 0,
    raises: {},
    maxPots: 0,
    allin: new Set(),
    dealt,
    draws: new Map(),
    discards: new Set(),
    actors: new Map(),
    allinsSent: 0,
    raisesSent: 0,
    cr: {},
    special: null,
    specialStreet: (t.scen[scenario] ?? 0) % 2 === 1 ? "preflop" : "river",
    timeoutUser: null,
    done: false,
  };
}

function observe(t, msg) {
  if (msg.type === "deflator_triggered") return;
  if (msg.type !== "table_state") return;
  observeState(t, msg);
}

/** Loss Deflator: conta o aviso de qualquer bot da mesa, uma vez por mão e perdedor. */
function noteDeflator(t, msg) {
  const key = `${t.hand?.id ?? "?"}:${msg.loser_name}:${msg.cashback_amount}`;
  t.deflatorSeen ??= new Set();
  if (t.deflatorSeen.has(key)) return;
  t.deflatorSeen.add(key);
  {
    t.cov.loss_deflator += 1;
    if (msg.prevented_elimination) t.cov.deflator_prevented_elimination += 1;
    t.deflator.push({
      hand: t.hand?.id ?? null,
      loser: msg.loser_name,
      winner: msg.winner_name,
      cashback: Number(msg.cashback_amount ?? 0),
      equity: msg.loser_equity_percent ?? null,
      tournament: !!msg.is_tournament,
    });
  }
}

function observeState(t, msg) {
  let hid = msg.hand_id ?? null;
  if (!hid) hid = `${t.id}:${t.syntheticHand}`;
  if (!msg.is_finished && (!t.hand || t.hand.id !== hid || t.hand.done)) {
    if (t.hand && t.hand.id === hid && t.hand.done) return;
    startHand(t, msg, hid);
  }
  const h = t.hand;
  if (!h || h.id !== hid) return;
  if (msg.is_finished) {
    if (!h.done) finishHand(t, h, msg);
    return;
  }
  const cbm = Number(msg.current_bet_to_match ?? 0);
  if (msg.stage !== h.stage) {
    h.stage = msg.stage;
    h.cbm = cbm;
    h.raises[h.stage] = 0;
  } else if (cbm > h.cbm) {
    if (h.cbm > 0) h.raises[h.stage] = (h.raises[h.stage] ?? 0) + 1;
    h.cbm = cbm;
  }
  h.maxPots = Math.max(h.maxPots, (msg.pots ?? []).filter((p) => Number(p.amount) > 0).length);
  for (const p of msg.players ?? []) {
    if (!h.dealt.has(p.id)) continue;
    if (!p.folded && Number(p.chips) === 0) h.allin.add(p.id);
    const dc = Number(p.draw_count ?? 0);
    if (dc > (h.draws.get(p.id) ?? 0)) {
      t.cov.pineapple_swap += dc - (h.draws.get(p.id) ?? 0);
      h.draws.set(p.id, dc);
    }
    if (p.river_discard && !h.discards.has(p.id)) {
      h.discards.add(p.id);
      t.cov.pineapple_discard += 1;
      if (h.timeoutUser && h.timeoutUser === h.actors.get(p.id)) t.cov.pineapple_auto_discard += 1;
    }
  }
}

function finishHand(t, h, msg) {
  h.done = true;
  t.handsDone += 1;
  t.syntheticHand += 1;
  t.lastHandAt = Date.now();
  t.cov.hands += 1;
  if (msg.hand_id) t.handIds.push({ id: msg.hand_id, viewer: t.observer?.user.username ?? null });
  h.maxPots = Math.max(h.maxPots, (msg.pots ?? []).filter((p) => Number(p.amount) > 0).length);
  const live = (msg.players ?? []).filter((p) => h.dealt.has(p.id) && !p.folded).length;
  const winners = [...new Set(msg.winners ?? [])];
  if ((msg.showdown ?? []).length >= 2) t.cov.showdowns += 1;
  if (h.allin.size >= 1) t.cov.allin += 1;
  if (h.allin.size >= 1 && live >= 3) t.cov.allin_multiway += 1;
  if (h.maxPots >= 2) t.cov.side_pots += 1;
  const raises = Object.values(h.raises);
  if (raises.some((r) => r >= 1)) t.cov.raise += 1;
  if (raises.some((r) => r >= 2)) t.cov.reraise += 1;
  if (h.cr.raised && (h.raises.flop ?? 0) >= 1) t.cov.checkraise += 1;
  if (winners.length >= 2 && h.maxPots <= 1) t.cov.tie += 1;
  if (t.handsDone <= 2 || t.handsDone % 25 === 0) log(`MÃO ${t.name} ${t.handsDone} (${h.scenario})`);
  const chips = new Map((msg.players ?? []).map((p) => [p.id, Number(p.chips)]));
  for (const bot of t.bots.values()) {
    if (bot.playerId && chips.has(bot.playerId)) bot.chips = chips.get(bot.playerId);
  }
  if (t.onHandEnd) Promise.resolve(t.onHandEnd(t, msg)).catch((e) => t.errors.push(`onHandEnd: ${e.message}`));
}

function act(action, amount = 0) {
  return { action, amount };
}

function decide(t, h, bot, msg, a, me) {
  const has = (x) => a.includes(x);
  const toCall = Number(msg.call_amount ?? 0) > 0;
  const seat = Number(me?.seat ?? 0);
  const minW = Number(msg.minimum_wager ?? 0) || Math.max(1, t.bb * 2);
  const passive = () => (has("check") ? act("check") : has("call") ? act("call") : act("fold"));
  const allin = () => (has("allin") ? act("allin") : has("raise") ? act("raise", Number(msg.maximum_wager ?? minW)) : has("bet") ? act("bet", Number(msg.maximum_wager ?? minW)) : passive());
  const u = bot.user.username;
  const st = String(msg.stage ?? "").toLowerCase();
  h.actors.set(bot.playerId, u);

  if ((h.scenario === "timeout" || h.scenario === "extra") && !h.special && st === h.specialStreet) {
    h.special = u;
    if (h.scenario === "timeout") {
      h.timeoutUser = u;
      bot.turn.timeout = true;
      return null;
    }
    bot.turn.extra = true;
  }
  // Brazilian Pineapple: troca/manutenção e descarte vêm antes da aposta na mesma vez.
  let plan = null;
  if (has("discard")) plan = act("discard", (h.n + seat) % 3);
  else if (has("keep") || has("swap")) {
    plan = (h.n + seat) % 2 === 0 && has("swap") ? act("swap", (h.n + seat) % 3) : act("keep");
  }
  if (!plan) {
    switch (h.scenario) {
      case "allin_mw": {
        // O menor stack empurra (aumento máximo até ficar all-in); até três pagam e
        // um deles aposta depois do flop, o que abre um pote paralelo.
        const live = (msg.players ?? []).filter((p) => h.dealt.has(p.id) && !p.folded);
        const total = (p) => Number(p.chips) + Number(p.bet ?? 0);
        const shortest = Math.min(...live.map(total));
        h.callers ??= new Set();
        if (h.shortAllin === u) plan = allin();
        else if (st === "preflop" && !h.shortAllin && me && total(me) === shortest) {
          h.shortAllin = u;
          plan = allin();
        } else if (st === "preflop") {
          if (h.callers.has(u) || h.callers.size < 3) {
            h.callers.add(u);
            plan = passive();
          } else plan = toCall ? act("fold") : passive();
        } else if (h.shortAllin && !h.sideBet && !toCall && has("bet")) {
          h.sideBet = u;
          plan = act("bet", minW);
        } else plan = passive();
        break;
      }
      case "allin_3":
      case "allin_hu": {
        // Dois (ou três) agressores sobem o máximo a cada vez até o all-in; os outros saem.
        const want = h.scenario === "allin_3" ? 3 : 2;
        h.aggr ??= new Set();
        if (h.aggr.has(u)) plan = allin();
        else if (h.aggr.size < want) {
          h.aggr.add(u);
          plan = allin();
        } else plan = toCall ? act("fold") : passive();
        break;
      }
      case "reraise":
        if (st === "preflop" && h.raisesSent < 3 && has("raise")) {
          h.raisesSent += 1;
          plan = act("raise", minW);
        } else plan = passive();
        break;
      case "checkraise":
        if (st === "flop") {
          if (!h.cr.checker && !toCall && has("check")) {
            h.cr.checker = u;
            plan = act("check");
          } else if (h.cr.checker && !h.cr.bettor && u !== h.cr.checker && !toCall && has("bet")) {
            h.cr.bettor = u;
            plan = act("bet", minW);
          } else if (u === h.cr.checker && toCall && !h.cr.raised && has("raise")) {
            h.cr.raised = true;
            plan = act("raise", minW);
          } else plan = passive();
        } else plan = passive();
        break;
      case "showdown":
      case "timeout":
      case "extra":
        plan = passive();
        break;
      default: {
        const roll = (h.n * 17 + seat * 31 + (Number(me?.chips ?? 0) % 97)) % 100;
        if (has("check") && roll < 60) plan = act("check");
        else if (has("bet") && roll > 85) plan = act("bet", minW);
        else if (has("raise") && roll > 90) plan = act("raise", minW);
        else if (has("allin") && roll > 98) plan = act("allin");
        else if (toCall && has("fold") && roll < 25) plan = act("fold");
        else plan = passive();
      }
    }
  }
  if (bot.turn.extra) return { ...plan, delayMs: TURN_MS + 1500 };
  return plan;
}

function sendAction(t, bot, plan) {
  if (bot.ws.readyState !== WebSocket.OPEN) return;
  bot.ws.send(JSON.stringify({ type: "action", action: plan.action, amount: plan.amount ?? 0 }));
  bot.turn.sent = true;
  bot.turn.sentAt = Date.now();
  bot.turn.last = plan.action;
  if (plan.action === "keep") bot.turn.keepPending = true;
}

function fallback(t, bot) {
  const msg = bot.last;
  if (!msg || !bot.turn || bot.turn.fallbacks >= 2) return;
  bot.turn.fallbacks = (bot.turn.fallbacks ?? 0) + 1;
  const a = (msg.available_actions ?? []).map((x) => String(x).toLowerCase());
  const pick = a.includes("keep") ? "keep" : a.includes("check") ? "check" : a.includes("call") ? "call" : a.includes("discard") ? "discard" : "fold";
  sendAction(t, bot, act(pick, 0));
}

function endTurn(t, bot, me) {
  const turn = bot.turn;
  bot.turn = null;
  if (turn.extra && turn.sent && !turn.error && turn.sentAt - turn.start >= TURN_MS && turn.sentAt - turn.start < TURN_MS + EXTRA_MS) {
    t.cov.extra_time += 1;
  }
}

function onBotState(t, bot, msg) {
  bot.last = msg;
  const me = (msg.players ?? []).find((p) => p.id === bot.playerId);
  bot.present = !!me;
  if (me) {
    bot.chips = Number(me.chips);
    bot.seenAt = Date.now();
    if (me.is_sitting === false && !t.stopping && Date.now() - (bot.sitInAt ?? 0) > 10_000) {
      bot.sitInAt = Date.now();
      bot.ws.send(JSON.stringify({ type: "sit_in" }));
    }
  }
  const actions = (msg.available_actions ?? []).map((x) => String(x).toLowerCase());
  const myTurn = !msg.is_finished && actions.length > 0 && !!me;
  if (bot.turn?.keepPending && !actions.includes("keep")) {
    bot.turn.keepPending = false;
    t.cov.pineapple_keep += 1;
  }
  const sig = JSON.stringify([msg.hand_id, msg.stage, msg.drawing === true, msg.discarding === true, msg.current_bet_to_match, me?.chips, me?.bet, me?.draw_count, me?.folded, actions.join()]);
  // Tempo esgotado: o servidor agiu pelo bot (fold com aposta a pagar; sem aposta, check; no Pineapple, mantém e descarta antes).
  const tt = bot.turn;
  if (tt?.timeout && !tt.sent && Date.now() - tt.start >= TURN_MS - 500 && (!myTurn || sig !== tt.sig0)) {
    if (me?.folded) t.cov.timeout_fold += 1;
    else t.cov.timeout_check += 1;
    tt.timeout = false;
    tt.start = Date.now();
  }
  if (bot.turn && !myTurn) endTurn(t, bot, me);
  if (myTurn && t.stopping) {
    if (bot.lastSig !== sig) {
      bot.lastSig = sig;
      bot.turn ??= { start: Date.now(), sent: false, fallbacks: 0, sig0: sig };
      sendAction(t, bot, actions.includes("check") ? act("check") : actions.includes("keep") ? act("keep") : act("fold"));
    }
    return;
  }
  if (!myTurn) return;
  if (sig === bot.lastSig) return;
  bot.lastSig = sig;
  if (!bot.turn) bot.turn = { start: Date.now(), sent: false, fallbacks: 0, sig0: sig };
  const h = t.hand;
  if (!h || h.done) {
    sendAction(t, bot, actions.includes("check") ? act("check") : act("fold"));
    return;
  }
  const plan = decide(t, h, bot, msg, actions, me);
  if (!plan) return;
  if (plan.delayMs) {
    const wait = Math.max(0, plan.delayMs - (Date.now() - bot.turn.start));
    setTimeout(() => {
      if (bot.lastSig === sig && bot.turn) sendAction(t, bot, plan);
    }, wait);
  } else sendAction(t, bot, plan);
}

async function openSocket(t, user, ticket) {
  const ws = new WebSocket(wsUrl(t.id, ticket));
  const bot = { user, ws, t, playerId: null, chips: null, turn: null, lastSig: null, present: false, openedAt: Date.now() };
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`ws open ${user.username}`)), 20_000);
    ws.once("open", () => {
      clearTimeout(timer);
      resolve();
    });
    ws.once("error", (e) => {
      clearTimeout(timer);
      reject(new Error(`ws ${user.username}: ${e.message}`));
    });
  });
  ws.on("message", (data) => {
    let msg;
    try {
      msg = JSON.parse(String(data));
    } catch {
      return;
    }
    bot.lastMsgAt = Date.now();
    if (msg.type === "welcome") bot.playerId = msg.player_id;
    if (msg.type === "deflator_triggered") noteDeflator(t, msg);
    if (t.observer === bot) observe(t, msg);
    if (msg.type === "error") {
      if (t.errors.length < 40) t.errors.push(`${user.username}: ${String(msg.message ?? "").slice(0, 120)}`);
      if (bot.turn) {
        bot.turn.error = true;
        bot.lastSig = null;
        fallback(t, bot);
      }
      return;
    }
    if (msg.type === "table_state") onBotState(t, bot, msg);
  });
  const ping = setInterval(() => {
    if (ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ type: "ping" }));
  }, 25_000);
  ws.on("close", () => {
    clearInterval(ping);
    if (t.bots.get(user.username) === bot) t.bots.delete(user.username);
    if (t.observer === bot) t.observer = [...t.bots.values()][0] ?? null;
  });
  t.bots.set(user.username, bot);
  t.seatedEver.add(user.username);
  if (!t.observer) t.observer = bot;
  ws.send(JSON.stringify({ type: "get_table_info" }));
  return bot;
}

function closeBot(t, bot) {
  try {
    bot.ws.close();
  } catch {}
  t.bots.delete(bot.user.username);
  if (t.observer === bot) t.observer = [...t.bots.values()][0] ?? null;
}

// ─── Cash ────────────────────────────────────────────────────────────────────

async function seatCash(t, user) {
  const bal = await balances(user);
  if (bal.cash < t.buyIn) throw new Error(`saldo ${bal.cash} < ${t.buyIn}`);
  await must("/api/lobby/join", { method: "POST", user, body: { table_id: t.id, buy_in: t.buyIn, wallet_mode: "play" } });
  const tk = await must(`/api/lobby/tables/${t.id}/ws-ticket`, { method: "POST", user });
  return openSocket(t, user, tk.ticket);
}

/** Sai da mesa cash. Durante a mão o servidor recusa (400); entre mãos a saída passa. */
async function leaveWithRetry(tableId, user, maxMs = 180_000) {
  const until = Date.now() + maxMs;
  for (;;) {
    const res = await call("/api/lobby/leave", { method: "POST", user, body: { table_id: tableId } });
    if (res.ok || res.status === 404 || Date.now() > until) return res;
    await sleep(1200);
  }
}

async function leaveCash(t, bot) {
  try {
    bot.ws.send(JSON.stringify({ type: "sit_out" }));
  } catch {}
  const res = await leaveWithRetry(t.id, bot.user);
  if (!res.ok && res.status !== 404) t.errors.push(`leave ${bot.user.username}: ${res.status} ${String(res.text).slice(0, 80)}`);
  closeBot(t, bot);
}

async function takeReserve(t, reserve) {
  for (let i = 0; i < reserve.length; i += 1) {
    const user = reserve[i];
    try {
      const bal = await balances(user);
      if (bal.cash >= t.buyIn) {
        reserve.splice(i, 1);
        return user;
      }
    } catch {}
  }
  return null;
}

async function runCashTable(t, seats, reserve) {
  for (const user of seats) {
    let u = user;
    while (u) {
      try {
        await seatCash(t, u);
        break;
      } catch (e) {
        t.errors.push(`seat ${u.username}: ${e.message}`);
        u = await takeReserve(t, reserve);
      }
    }
  }
  log(`MESA ${t.name}: ${t.bots.size}/${t.max} bots sentados`);
  t.report = { replacements: 0 };
  return new Promise((resolve) => {
    let replacing = false;
    const finish = async (why) => {
      if (t.stopping) return;
      t.stopping = true;
      clearInterval(watch);
      log(`FIM ${t.name}: ${why}`);
      await Promise.all([...t.bots.values()].map((b) => leaveCash(t, b)));
      resolve();
    };
    t.onHandEnd = async () => {
      if (t.handsDone >= HANDS_PER_TABLE) return finish(`${t.handsDone} mãos`);
      if (replacing) return;
      replacing = true;
      try {
        for (const bot of [...t.bots.values()].filter((b) => b.chips === 0)) {
          await leaveCash(t, bot);
          reserve.push(bot.user);
          const next = await takeReserve(t, reserve);
          if (!next) {
            t.errors.push("sem reserva com saldo");
            continue;
          }
          try {
            await seatCash(t, next);
            t.report.replacements += 1;
          } catch (e) {
            t.errors.push(`replace ${next.username}: ${e.message}`);
          }
        }
      } finally {
        replacing = false;
      }
    };
    const deadline = Date.now() + Math.max(60 * 60_000, HANDS_PER_TABLE * 30_000);
    const watch = setInterval(() => {
      if (Date.now() > deadline) finish("tempo máximo");
      else if (Date.now() - t.lastHandAt > 5 * 60_000) finish("mesa parada há 5 min");
    }, 10_000);
  });
}

async function fetchRake(t, pool) {
  const byName = new Map(pool.map((u) => [u.username, u]));
  let rake = 0;
  let pot = 0;
  let missing = 0;
  let sidePots = 0;
  let ties = 0;
  const payoutMismatch = [];
  for (const { id, viewer } of t.handIds) {
    const candidates = [...new Set([viewer, ...t.seatedEver])].map((n) => byName.get(n)).filter(Boolean);
    let got = null;
    for (const user of candidates) {
      const res = await call(`/api/hand-history/${id}`, { user });
      if (res.ok) {
        got = res.data?.replay ?? res.data;
        break;
      }
    }
    if (!got) {
      missing += 1;
      continue;
    }
    rake += Number(got.rake_collected ?? 0);
    const settle = got.settlement;
    if (settle?.players) {
      const live = settle.players.filter((p) => !p.folded);
      const put = (p) => Number(p.chips_lost ?? 0);
      // Pote paralelo: um all-in coberto por dois ou mais jogadores que seguiram na mão.
      const side = live.some((p) => p.was_all_in && live.filter((q) => put(q) > put(p)).length >= 2);
      if (side) sidePots += 1;
      // Empate: dois ou mais levam fichas disputadas (sem contar a sobra devolvida a quem cobriu).
      const puts = live.map(put).sort((x, y) => y - x);
      const refund = (p) => (put(p) === puts[0] && puts[0] > (puts[1] ?? 0) ? puts[0] - (puts[1] ?? 0) : 0);
      const contested = live.filter((p) => Number(p.chips_won ?? 0) - refund(p) > 0).length;
      if (!side && contested >= 2) ties += 1;
      const paid = (settle.payouts ?? []).reduce((sum, p) => sum + Number(p.amount ?? 0), 0);
      if (paid + Number(got.rake_collected ?? 0) !== Number(got.pot_total ?? 0)) {
        payoutMismatch.push({ hand: id, paid, rake: Number(got.rake_collected ?? 0), pot: Number(got.pot_total ?? 0) });
      }
    }
    pot += Number(got.pot_total ?? 0);
  }
  return { rake, pot, missing, sidePots, ties, payoutMismatch };
}

async function phaseCash(pool, out) {
  const tables = await must("/api/lobby/tables?mode=play", { user: pool[0] });
  const chosen = CASH_TABLES.map((name) => tables.find((t) => t.name === name)).filter(Boolean);
  if (chosen.length !== CASH_TABLES.length) throw new Error(`mesas cash não encontradas: ${CASH_TABLES.join(" | ")}`);
  const before = new Map();
  for (const u of pool) before.set(u.username, await balances(u));
  const runs = chosen.map((tb) => makeTable(tb, { cycle: CASH_CYCLE, quotas: CASH_QUOTAS, kind: "cash" }));
  let cursor = 0;
  const plans = runs.map((t) => {
    const busy = Number(chosen.find((c) => c.id === t.id).players ?? 0);
    const seats = pool.slice(cursor, cursor + t.max - busy);
    cursor += seats.length;
    return { t, seats, busy };
  });
  const reserve = pool.slice(cursor);
  log(`CASH ${plans.map((p) => `${p.t.name} ${p.seats.length}+${p.busy}/${p.t.max}`).join(" | ")}; reservas ${reserve.length}`);
  out.cash = { tables: [], reserve_start: reserve.length };
  await Promise.all(plans.map((p) => runCashTable(p.t, p.seats, reserve)));
  await sleep(5000);
  let delta = 0;
  let resetSeen = false;
  for (const u of pool) {
    const after = await balances(u);
    const b = before.get(u.username);
    delta += after.cash - b.cash;
    if (after.reset !== b.reset) resetSeen = true;
  }
  let rake = 0;
  let cashback = 0;
  for (const t of runs) {
    const r = await fetchRake(t, pool);
    // Potes paralelos e empates vêm da liquidação (o estado da mesa só mostra o pote total).
    t.cov.side_pots = r.sidePots;
    t.cov.tie = r.ties;
    rake += r.rake;
    const cb = t.deflator.filter((d) => !d.tournament).reduce((s, d) => s + d.cashback, 0);
    cashback += cb;
    out.cash.tables.push({
      name: t.name,
      variant: t.variant,
      max_players: t.max,
      hands: t.handsDone,
      scenarios: t.scen,
      coverage: t.cov,
      replacements: t.report?.replacements ?? 0,
      rake_cents: r.rake,
      side_pots_settlement: r.sidePots,
      payout_mismatches: r.payoutMismatch,
      pot_total_cents: r.pot,
      history_missing: r.missing,
      deflator: t.deflator,
      errors: t.errors,
    });
  }
  out.cash.wallet = {
    sum_delta_cents: delta,
    rake_cents: rake,
    deflator_cashback_cents: cashback,
    // O cashback do Loss Deflator sai do vencedor (soma zero no pote): só o rake sai das contas.
    expected_delta_cents: -rake,
    daily_reset_during_run: resetSeen,
    payout_mismatches: out.cash.tables.reduce((n, t) => n + t.payout_mismatches.length, 0),
    ok: !resetSeen && delta === -rake && out.cash.tables.every((t) => t.payout_mismatches.length === 0),
  };
  log(`CASH carteira: Δ=${delta} rake=${rake} cashback=${cashback} ok=${out.cash.wallet.ok}`);
}

/** Tira das mesas cash qualquer conta do grupo que tenha ficado sentada. */
async function phaseCleanup(pool, out) {
  const tables = await must("/api/lobby/tables?mode=play", { user: pool[0] });
  const ids = CASH_TABLES.map((name) => tables.find((t) => t.name === name)?.id).filter(Boolean);
  const results = await Promise.all(
    pool.flatMap((user) => ids.map(async (id) => ({ user: user.username, status: (await leaveWithRetry(id, user)).status }))),
  );
  out.cleanup = { left: results.filter((r) => r.status === 200).length, failed: results.filter((r) => r.status !== 200 && r.status !== 404) };
  out.cleanup.ok = out.cleanup.failed.length === 0;
  log(`LIMPEZA: ${out.cleanup.left} saídas, ${out.cleanup.failed.length} falhas`);
}

// ─── Torneio ─────────────────────────────────────────────────────────────────

async function phaseMtt(pool, out) {
  if (!MTT_ID) throw new Error("MTT_ID é obrigatório para a fase mtt");
  let info = await must(`/api/tournament/${MTT_ID}`);
  out.mtt = { id: MTT_ID, name: info.name, purchases: [], tables: [], errors: [] };
  if (!info.allow_rebuy || !info.allow_addon) throw new Error(`torneio sem rebuy/add-on: rebuy=${info.allow_rebuy} addon=${info.allow_addon}`);
  // MTT_RESUME=1 retoma um torneio já em andamento com os bots inscritos; o saldo
  // de torneio antes da inscrição vem de MTT_BEFORE_CENTS (contas novas: 15.000).
  const resume = process.env.MTT_RESUME === "1" && info.status === "running";
  if (info.status !== "registering" && !resume) throw new Error(`torneio em ${info.status}`);
  const cost = Number(info.buy_in) + Number(info.rebuy_cost) + Number(info.addon_cost);
  const field = [];
  const before = new Map();
  for (const u of pool) {
    if (resume) {
      const reg = await call(`/api/tournament/${MTT_ID}/registration`, { user: u });
      if (reg.ok && reg.data?.registered) {
        field.push(u);
        before.set(u.username, { mtt: Number(process.env.MTT_BEFORE_CENTS ?? 15_000) });
      }
      continue;
    }
    if (field.length >= info.max_players - info.registered_players) break;
    const b = await balances(u);
    if (b.mtt >= cost) {
      field.push(u);
      before.set(u.username, b);
    }
  }
  out.mtt.resumed = resume;
  log(`MTT ${info.name}: ${resume ? "retomando" : "inscrevendo"} ${field.length} (custo máximo ${cost} por bot)`);
  for (const u of resume ? [] : field) {
    const res = await call("/api/tournament/register", { method: "POST", user: u, body: { tournament_id: MTT_ID, wallet_mode: "play" } });
    if (!res.ok) out.mtt.errors.push(`register ${u.username}: ${res.status} ${String(res.text).slice(0, 120)}`);
  }
  const startBy = Date.now() + MTT_START_WAIT_MS;
  while (Date.now() < startBy) {
    info = await must(`/api/tournament/${MTT_ID}`);
    if (info.status !== "registering") break;
    await sleep(10_000);
  }
  if (info.status !== "running") throw new Error(`torneio não começou: ${info.status}`);
  log(`MTT começou: ${info.registered_players} inscritos, mesas ${(info.live_table_ids ?? []).length}`);
  const tables = new Map();
  const state = new Map(field.map((u) => [u.username, { user: u, bot: null, eliminated: false, rebuys: 0, addon: false, tables: new Set() }]));
  const tableFor = (id) => {
    if (!tables.has(id)) {
      const t = makeTable(
        { id, name: `${info.name} · mesa ${tables.size + 1}`, poker_variant: info.poker_variant, max_players: info.table_max_players },
        { cycle: MTT_CYCLE, quotas: MTT_QUOTAS, kind: "mtt" },
      );
      tables.set(id, t);
    }
    return tables.get(id);
  };
  const locate = async (s, liveIds) => {
    for (const id of liveIds) {
      const res = await call(`/api/lobby/tables/${id}/ws-ticket`, { method: "POST", user: s.user });
      if (!res.ok) continue;
      if (s.bot) closeBot(s.bot.t, s.bot);
      const t = tableFor(id);
      s.bot = await openSocket(t, s.user, res.data.ticket);
      s.tables.add(id);
      return true;
    }
    return false;
  };
  const deadline = Date.now() + MTT_MAX_MS;
  let lastPurchase = 0;
  while (Date.now() < deadline) {
    info = await must(`/api/tournament/${MTT_ID}`);
    if (info.status !== "running") break;
    const liveIds = info.live_table_ids?.length ? info.live_table_ids : info.live_table_id ? [info.live_table_id] : [];
    const level = Number(info.current_level ?? 1);
    const buyWindow = level <= Math.max(Number(info.rebuy_max_level), Number(info.addon_max_level));
    for (const t of tables.values()) t.aggressive = !buyWindow;
    for (const s of state.values()) {
      if (s.eliminated) continue;
      // Na fusão de mesas a antiga para de mandar estado: o socket segue aberto, mas
      // o bot precisa procurar a mesa nova (lista de mesas ativas ou silêncio de 45 s).
      const quiet = Date.now() - (s.bot?.lastMsgAt ?? s.bot?.openedAt ?? 0) > 45_000;
      const stale =
        !s.bot ||
        s.bot.ws.readyState !== WebSocket.OPEN ||
        !liveIds.includes(s.bot.t.id) ||
        quiet ||
        (!s.bot.present && Date.now() - s.bot.openedAt > 8_000);
      if (stale && (s.nextLocate ?? 0) <= Date.now()) {
        const found = await locate(s, liveIds);
        if (!found) {
          s.nextLocate = Date.now() + 30_000;
          if (s.bot) {
            closeBot(s.bot.t, s.bot);
            s.bot = null;
          }
        }
      }
    }
    if (Date.now() - lastPurchase > 20_000) {
      lastPurchase = Date.now();
      for (const s of state.values()) {
        if (s.eliminated && !buyWindow) continue;
        const reg = await call(`/api/tournament/${MTT_ID}/registration`, { user: s.user });
        if (!reg.ok) continue;
        const r = reg.data;
        s.rebuys = Number(r.rebuys ?? s.rebuys);
        s.addon = !!r.addon_done;
        for (const kind of ["rebuy", "addon"]) {
          const can = kind === "rebuy" ? r.can_rebuy && s.rebuys < 1 : r.can_addon && !s.addon;
          if (!can) continue;
          const res = await call(`/api/tournament/${kind}`, { method: "POST", user: s.user, body: { tournament_id: MTT_ID } });
          out.mtt.purchases.push({ user: s.user.username, kind, level, status: res.status, delivery: res.data?.delivery ?? null, cost: res.data?.cost_cents ?? null, chips: res.data?.chips ?? null, error: res.ok ? null : String(res.text).slice(0, 120) });
          if (res.ok) {
            if (kind === "rebuy") s.rebuys += 1;
            else s.addon = true;
            if (res.data?.delivery === "seated" && res.data?.table_id) {
              s.eliminated = false;
              await locate(s, [res.data.table_id]);
            }
          }
        }
        if (r.eliminated && !r.can_rebuy) {
          s.eliminated = true;
          if (s.bot) {
            closeBot(s.bot.t, s.bot);
            s.bot = null;
          }
        }
      }
    }
    await sleep(8_000);
  }
  for (const s of state.values()) if (s.bot) closeBot(s.bot.t, s.bot);
  info = await must(`/api/tournament/${MTT_ID}`);
  await sleep(5000);
  let prizes = 0;
  const paid = [];
  for (const s of state.values()) {
    const b0 = before.get(s.user.username);
    const b1 = await balances(s.user);
    const reg = await call(`/api/tournament/${MTT_ID}/registration`, { user: s.user });
    const rebuys = Number(reg.data?.rebuys ?? s.rebuys);
    const addon = reg.data?.addon_done ?? s.addon;
    const spent = Number(info.buy_in) + rebuys * Number(info.rebuy_cost) + (addon ? Number(info.addon_cost) : 0);
    const prize = b1.mtt - b0.mtt + spent;
    prizes += prize;
    if (prize !== 0) paid.push({ user: s.user.username, prize_cents: prize });
    s.final = { rebuys, addon, spent, prize };
  }
  const states = [...state.values()];
  out.mtt.status = info.status;
  out.mtt.registered = info.registered_players;
  out.mtt.prize_pool_cents = Number(info.prize_pool);
  out.mtt.prizes_paid_cents = prizes;
  out.mtt.paid = paid.sort((a, b) => b.prize_cents - a.prize_cents);
  out.mtt.rebuys = states.filter((s) => s.final.rebuys >= 1).length;
  out.mtt.addons = states.filter((s) => s.final.addon).length;
  out.mtt.field = states.length;
  out.mtt.moved_tables = states.filter((s) => s.tables.size >= 2).length;
  out.mtt.tables = [...tables.values()].map((t) => ({ name: t.name, hands: t.handsDone, scenarios: t.scen, coverage: t.cov, deflator: t.deflator, errors: t.errors }));
  out.mtt.ok = info.status === "finished" && prizes === Number(info.prize_pool);
  log(`MTT ${info.status}: prêmios ${prizes} / pool ${info.prize_pool}; rebuys ${out.mtt.rebuys}/${states.length}; add-ons ${out.mtt.addons}/${states.length}`);
}

// ─── Jogo Real (sem dinheiro) ───────────────────────────────────────────────

async function phaseReal(user, out) {
  const r = {};
  const mode = await call("/api/wallet/mode", { method: "POST", user, body: { mode: "real" } });
  r.set_mode = { status: mode.status, balance_real: mode.data?.balance_real ?? null, mode: mode.data?.preferred_wallet_mode ?? null };
  const me = await balances(user);
  r.me = me;
  const dep = await call("/api/wallet/deposit-info", { user });
  r.deposit_info = { status: dep.status, keys: dep.data && typeof dep.data === "object" ? Object.keys(dep.data) : [] };
  const tx = await call("/api/wallet/transactions", { user });
  r.transactions = { status: tx.status, count: Array.isArray(tx.data) ? tx.data.length : null };
  const tables = await call("/api/lobby/tables?mode=real", { user });
  const realTables = Array.isArray(tables.data) ? tables.data.filter((t) => t.money_mode === "real") : [];
  r.real_tables = realTables.length;
  if (realTables[0]) {
    const join = await call("/api/lobby/join", { method: "POST", user, body: { table_id: realTables[0].id, buy_in: realTables[0].min_buy_in, wallet_mode: "real" } });
    r.join_real = { table: realTables[0].name, status: join.status, message: String(join.text).slice(0, 160) };
  }
  const tours = await call("/api/lobby/tournaments?mode=real", { user });
  const realTour = Array.isArray(tours.data) ? tours.data.find((t) => t.money_mode === "real" && t.status === "registering" && Number(t.buy_in) > 0) : null;
  if (realTour) {
    const reg = await call("/api/tournament/register", { method: "POST", user, body: { tournament_id: realTour.id, wallet_mode: "real" } });
    r.register_real = { tournament: realTour.name, status: reg.status, message: String(reg.text).slice(0, 160) };
  }
  const wd = await call("/api/payments/pix/withdraw", { method: "POST", user, body: { amount: 1000, pix_key_type: "email", pix_key: user.email } });
  r.withdraw = { status: wd.status, message: String(wd.text).slice(0, 160) };
  const back = await call("/api/wallet/mode", { method: "POST", user, body: { mode: "play" } });
  r.back_to_play = back.status;
  r.ok =
    mode.ok &&
    me.real === 0 &&
    me.mode === "real" &&
    (!r.join_real || r.join_real.status >= 400) &&
    (!r.register_real || r.register_real.status >= 400) &&
    wd.status >= 400 &&
    back.ok;
  out.real = r;
  log(`REAL: saldo ${me.real}, mesa ${r.join_real?.status ?? "-"}, torneio ${r.register_real?.status ?? "-"}, saque ${wd.status}, ok=${r.ok}`);
}

// ─── Principal ──────────────────────────────────────────────────────────────

function saveReport(out) {
  mkdirSync(dirname(REPORT_PATH), { recursive: true });
  writeFileSync(REPORT_PATH, JSON.stringify(out, null, 2));
}

async function main() {
  log(`BASE ${BASE_URL} fases=${[...PHASES].join(",")} mãos/mesa=${HANDS_PER_TABLE} contas=${POOL_SIZE} run=${RUN_ID}`);
  const out = { run: RUN_ID, base_url: BASE_URL, started_at: new Date().toISOString(), phases: [...PHASES] };
  const pool = await provisionPool(POOL_SIZE);
  out.pool = pool.length;
  saveReport(out);
  const steps = [
    ["cleanup", () => phaseCleanup(pool, out)],
    ["cash", () => phaseCash(pool, out)],
    ["mtt", () => phaseMtt(pool, out)],
    ["real", () => phaseReal(pool[0], out)],
  ];
  for (const [name, run] of steps) {
    if (!PHASES.has(name)) continue;
    try {
      await run();
    } catch (e) {
      out[`${name}_error`] = e.message;
      log(`FASE ${name} falhou: ${e.message}`);
    }
    saveReport(out);
  }
  out.finished_at = new Date().toISOString();
  out.ok = [...PHASES].every((p) => (p === "cash" ? out.cash?.wallet?.ok : out[p]?.ok));
  saveReport(out);
  log(`RELATÓRIO ${REPORT_PATH} ok=${out.ok}`);
  if (!out.ok) process.exitCode = 1;
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
