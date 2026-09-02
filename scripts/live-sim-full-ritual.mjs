/**
 * Ritual Play Money: enche cada mesa cash (1 por assento + 2 reservas),
 * joga HANDS_PER_TABLE mãos; reservas sentam no lugar de quem quebrou.
 * Torneios: inscreve campo (table_max×3) + 2 reservas por mesa (API).
 * Addon/rebuy MTT tentados; campeão/duração no teste Motor-Rust.
 *
 *   ALLOW_TEMP_MAIL=true HANDS_PER_TABLE=2000 node scripts/live-sim-full-ritual.mjs
 */
import { randomBytes } from "node:crypto";
import { writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(join(ROOT, "scripts", "live-sim-full-ritual.mjs"));
const WebSocket = require("ws");

const BASE_URL = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const MODE = "play";
const HANDS_PER_TABLE = Number(process.env.HANDS_PER_TABLE ?? 2000);
const WAITERS_PER_TABLE = 2;
const MTT_TABLES = 3;
const AUTH_GAP_MS = Number(process.env.AUTH_GAP_MS ?? 2500);
const MAIL_API = "https://api.mail.tm";
const USE_MAIL = process.env.ALLOW_TEMP_MAIL === "true";
const PASS = process.env.E2E_PASS ?? "PokerSim1A";
const RUN_ID = Date.now().toString(36).slice(-6);
const PLAY_TIMEOUT_MS = Math.max(45 * 60_000, HANDS_PER_TABLE * 10_000);
const REPORT_PATH =
  process.env.REPORT_PATH ?? join(ROOT, "Documentacao", "SIMULACAO_RITUAL.json");

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function rawFetch(url, options, retries = 8) {
  for (let attempt = 0; ; attempt += 1) {
    try {
      const res = await fetch(url, {
        ...options,
        signal: AbortSignal.timeout(30_000),
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
          "User-Agent": "ZeroTilt-LiveSim/1.0",
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
        const retryAfter = Number(res.headers.get("retry-after") ?? 65);
        await sleep(Math.max(2, retryAfter) * 1000);
        continue;
      }
      return { ok: res.ok, status: res.status, data, text };
    } catch (err) {
      if (attempt >= retries) throw err;
      await sleep(4000 * (attempt + 1));
    }
  }
}

async function apiSoft(path, { method = "GET", body, token } = {}) {
  return rawFetch(`${BASE_URL}${path}`, {
    method,
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
}

async function api(path, opts) {
  const res = await apiSoft(path, opts);
  if (!res.ok) {
    throw new Error(
      `${opts?.method ?? "GET"} ${path} -> ${res.status}: ${String(res.text).slice(0, 240)}`,
    );
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

async function registerOrLogin(user) {
  const login = await apiSoft("/api/auth/login", {
    method: "POST",
    body: { email: user.email, password: user.password },
  });
  if (login.ok && login.data?.token) {
    user.token = login.data.token;
    return "login";
  }
  let data;
  for (let attempt = 0; attempt < 4; attempt += 1) {
    const username = attempt === 0 ? user.username : `${user.username}${randomBytes(1).toString("hex")}`;
    const reg = await apiSoft("/api/auth/register", {
      method: "POST",
      body: {
        email: user.email,
        password: user.password,
        password_confirm: user.password,
        username,
      },
    });
    if (reg.ok) {
      user.username = username;
      data = reg.data;
      break;
    }
    if (reg.status === 409 && attempt < 3) continue;
    throw new Error(`POST /api/auth/register -> ${reg.status}: ${String(reg.text).slice(0, 240)}`);
  }
  if (data.token) {
    user.token = data.token;
    return "register";
  }
  if (data.pending_email_verification || /verific/i.test(JSON.stringify(data))) {
    user.needsVerify = true;
    return "pending";
  }
  throw new Error(`auth failed ${user.email}: ${JSON.stringify(data).slice(0, 200)}`);
}

async function createMailbox(index) {
  const domainsRes = await rawFetch(`${MAIL_API}/domains`, { method: "GET" });
  const domains = domainsRes.data;
  const list = Array.isArray(domains) ? domains : domains["hydra:member"] ?? [];
  const domain = list.find((d) => d.isActive)?.domain ?? list[0]?.domain;
  if (!domain) throw new Error("mail.tm: no domain");
  const address = `ztsim-${Date.now().toString(36)}-${index}-${randomBytes(2).toString("hex")}@${domain}`;
  const password = `${PASS}${randomBytes(4).toString("hex")}`;
  const created = await rawFetch(`${MAIL_API}/accounts`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  if (!created.ok) {
    throw new Error(`mail.tm account ${created.status}: ${String(created.text).slice(0, 160)}`);
  }
  const tok = await rawFetch(`${MAIL_API}/token`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  if (!tok.ok || !tok.data?.token) {
    throw new Error(`mail.tm token ${tok.status}`);
  }
  return { email: address, mailPassword: password, mailToken: tok.data.token };
}

function extractCode(content) {
  const contextual = String(content).match(
    /c[oó]digo(?:\s+de\s+verifica[cç][aã]o)?[\s\S]{0,400}?(\d{6})/i,
  );
  if (contextual) return contextual[1];
  return String(content).match(/(?<!\d)(\d{6})(?!\d)/)?.[1] ?? null;
}

async function waitCode(user) {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const inbox = await fetch(`${MAIL_API}/messages?page=1`, {
      headers: { Authorization: `Bearer ${user.mailToken}` },
    }).then((r) => r.json());
    const members = Array.isArray(inbox) ? inbox : inbox["hydra:member"] ?? [];
    const hit = members.find((m) => /Zero Tilt|verific/i.test(`${m.subject} ${m.intro}`));
    if (hit) {
      const detail = await fetch(`${MAIL_API}/messages/${hit.id}`, {
        headers: { Authorization: `Bearer ${user.mailToken}` },
      }).then((r) => r.json());
      const code = extractCode([detail.subject, detail.text, detail.html].join("\n"));
      if (code) return code;
    }
    await sleep(2000);
  }
  throw new Error(`no verify code for ${user.email}`);
}

async function provisionUsers(n, startIndex = 0) {
  const users = [];
  let idx = startIndex;
  let attempts = 0;
  while (users.length < n && attempts < n * 3) {
    attempts += 1;
    const username = `s${RUN_ID}${String(idx).padStart(2, "0")}`;
    let user = {
      username,
      password: PASS,
      email: `${username}@zerotilt.local`,
    };
    try {
      if (USE_MAIL) {
        const box = await createMailbox(idx);
        user.email = box.email;
        user.password = box.mailPassword;
        user.mailToken = box.mailToken;
      }
      await sleep(AUTH_GAP_MS);
      const how = await registerOrLogin(user);
      console.log(`AUTH ${user.username} ${how} ${user.email}`);
      if (how === "pending" && USE_MAIL) {
        let code = null;
        try {
          code = await waitCode(user);
        } catch {
          await apiSoft("/api/auth/resend-verification", {
            method: "POST",
            body: { email: user.email },
          });
          code = await waitCode(user);
        }
        await api("/api/auth/verify-email", {
          method: "POST",
          body: { email: user.email, code },
        });
        const data = await api("/api/auth/login", {
          method: "POST",
          body: { email: user.email, password: user.password },
        });
        user.token = data.token;
        console.log(`VERIFY ok ${user.username}`);
      }
      if (!user.token) {
        const data = await api("/api/auth/login", {
          method: "POST",
          body: { email: user.email, password: user.password },
        });
        user.token = data.token;
      }
      users.push(user);
    } catch (e) {
      console.error(`AUTH skip ${user.username}: ${e.message}`);
    }
    idx += 1;
  }
  return users;
}

async function refreshAuth(user) {
  const data = await api("/api/auth/login", {
    method: "POST",
    body: { email: user.email, password: user.password },
  });
  if (!data.token) throw new Error(`relogin failed ${user.email}`);
  user.token = data.token;
}

async function ensureFunds(user) {
  const cash = await apiSoft("/api/wallet/pm-rebuy", {
    method: "POST",
    token: user.token,
    body: { kind: "cash" },
  });
  const mtt = await apiSoft("/api/wallet/pm-rebuy", {
    method: "POST",
    token: user.token,
    body: { kind: "mtt" },
  });
  return {
    cash: cash.ok ? "rebuy" : `${cash.status}`,
    mtt: mtt.ok ? "rebuy" : `${mtt.status}`,
  };
}

function pickAction(actions, roll, msg, table) {
  const a = actions.map((x) => String(x).toLowerCase());
  const minW = Number(msg.minimum_wager ?? 0) || table.big_blind * 2;
  if (a.includes("check") && roll < 64) return { action: "check", amount: 0 };
  if (a.includes("bet") && roll > 88) return { action: "bet", amount: minW };
  if (a.includes("raise") && roll > 90) return { action: "raise", amount: minW };
  if (a.includes("allin") && roll > 97) return { action: "allin", amount: 0 };
  if (a.includes("fold") && roll < 22) return { action: "fold", amount: 0 };
  if (a.includes("call")) return { action: "call", amount: 0 };
  if (a.includes("check")) return { action: "check", amount: 0 };
  if (a.includes("fold")) return { action: "fold", amount: 0 };
  return null;
}

async function playTable(table, seated, waiters) {
  const report = {
    name: table.name,
    variant: table.poker_variant,
    seats: seated.length,
    waiters: waiters.length,
    hands: 0,
    replacements: 0,
    errors: [],
  };
  const sockets = new Map();
  let completed = 0;
  let inHand = false;
  let settled = false;
  let replacing = false;

  const closeAll = () => {
    for (const ws of sockets.values()) {
      try {
        ws.close();
      } catch {}
    }
    sockets.clear();
  };

  const attach = (user, ws) => {
    user.lastSig = null;
    user.playerId = null;
    user.chips = table.min_buy_in;
    ws.addEventListener("message", (ev) => {
      let msg;
      try {
        msg = JSON.parse(String(ev.data));
      } catch {
        return;
      }
      if (msg.type === "welcome") user.playerId = msg.player_id;
      if (msg.type === "error") {
        report.errors.push(String(msg.message ?? msg));
        return;
      }
      if (msg.type !== "table_state") return;
      const me = (msg.players ?? []).find((p) => p.id === user.playerId);
      if (me) user.chips = me.chips;
      if (msg.is_finished === false) inHand = true;
      if (msg.is_finished === true && inHand) {
        inHand = false;
        completed += 1;
        if (completed % 25 === 0 || completed <= 3) {
          console.log(`HAND ${table.name} ${completed}/${HANDS_PER_TABLE}`);
        }
      }
      if (!user.playerId || msg.is_finished || settled) return;
      if (!me?.is_active) return;
      const actions = msg.available_actions ?? [];
      const sig = JSON.stringify([msg.stage, msg.current_bet_to_match, me.chips, me.bet]);
      if (sig === user.lastSig) return;
      user.lastSig = sig;
      const act = pickAction(
        actions,
        (completed * 17 + user.username.length + (me.chips % 97)) % 100,
        msg,
        table,
      );
      if (act) ws.send(JSON.stringify({ type: "action", ...act }));
    });
  };

  async function openSeat(user) {
    await refreshAuth(user);
    await ensureFunds(user);
    await api("/api/lobby/join", {
      method: "POST",
      token: user.token,
      body: { table_id: table.id, buy_in: table.min_buy_in, wallet_mode: MODE },
    });
    const t = await api(`/api/lobby/tables/${table.id}/ws-ticket`, {
      method: "POST",
      token: user.token,
    });
    await new Promise((resolve, reject) => {
      const ws = new WebSocket(wsUrl(table.id, t.ticket));
      const ot = setTimeout(() => reject(new Error(`ws open ${user.username}`)), 20_000);
      ws.addEventListener(
        "open",
        () => {
          clearTimeout(ot);
          ws.send(JSON.stringify({ type: "get_table_info" }));
          sockets.set(user.username, ws);
          attach(user, ws);
          resolve();
        },
        { once: true },
      );
      ws.addEventListener("error", () => reject(new Error(`ws err ${user.username}`)), {
        once: true,
      });
    });
  }

  for (const u of seated) await openSeat(u);

  const t0 = Date.now();
  await new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(`timeout ${table.name} hands=${completed}`)),
      PLAY_TIMEOUT_MS,
    );
    const tick = setInterval(async () => {
      if (settled || replacing) return;
      if (completed >= HANDS_PER_TABLE) {
        settled = true;
        clearInterval(tick);
        clearTimeout(timer);
        resolve();
        return;
      }
      if (inHand) return;
      const busted = seated.filter((u) => u.chips === 0);
      if (!busted.length) return;
      replacing = true;
      try {
        for (const u of busted) {
          const waiter = waiters.shift();
          if (!waiter) continue;
          const oldWs = sockets.get(u.username);
          if (oldWs) {
            try {
              oldWs.close();
            } catch {}
            sockets.delete(u.username);
          }
          try {
            await api("/api/lobby/leave", {
              method: "POST",
              token: u.token,
              body: { table_id: table.id },
            });
          } catch (e) {
            report.errors.push(`leave busted ${u.username}: ${e.message}`);
          }
          waiters.push(u);
          u.chips = -1;
          const idx = seated.indexOf(u);
          if (idx >= 0) seated.splice(idx, 1);
          seated.push(waiter);
          await openSeat(waiter);
          report.replacements += 1;
          console.log(`REPLACE ${table.name} ${u.username} -> ${waiter.username}`);
        }
      } catch (e) {
        report.errors.push(`replace: ${e.message}`);
      } finally {
        replacing = false;
      }
    }, 1500);
  });

  closeAll();
  for (const u of [...seated, ...waiters]) {
    try {
      await api("/api/lobby/leave", {
        method: "POST",
        token: u.token,
        body: { table_id: table.id },
      });
    } catch {}
  }
  report.hands = completed;
  report.elapsed_ms = Date.now() - t0;
  report.ok = completed >= HANDS_PER_TABLE;
  return report;
}

function saveReport(out) {
  try {
    writeFileSync(REPORT_PATH, JSON.stringify(out, null, 2));
  } catch (e) {
    console.error(`report write failed: ${e.message}`);
  }
}

async function main() {
  console.log(
    `BASE ${BASE_URL} MODE=${MODE} HANDS=${HANDS_PER_TABLE} WAITERS/TABLE=${WAITERS_PER_TABLE} RUN=${RUN_ID}`,
  );
  if (!USE_MAIL) {
    console.warn("ALLOW_TEMP_MAIL is not true — VPS may reject unverified @zerotilt.local");
  }

  const probe = await provisionUsers(1);
  const tables = await api(`/api/lobby/tables?mode=${MODE}`, { token: probe[0].token });
  const playTables = (tables ?? []).filter((t) => (t.money_mode ?? MODE) === MODE);
  playTables.sort((a, b) => a.max_players - b.max_players || a.name.localeCompare(b.name));
  console.log(`CASH TABLES ${playTables.map((t) => `${t.name} ${t.max_players}`).join(" | ")}`);

  const list = await api(`/api/lobby/tournaments?mode=${MODE}`, { token: probe[0].token });
  console.log(
    `MTT ${(list ?? []).map((t) => `${t.name} maxT=${t.table_max_players} ready=${t.gameplay_ready}`).join(" | ")}`,
  );

  const cashNeed = playTables.reduce(
    (n, t) => n + t.max_players + WAITERS_PER_TABLE,
    0,
  );
  const maxMtt = Math.max(0, ...(list ?? []).map((t) => t.table_max_players ?? 9));
  const computed = Math.max(
    cashNeed,
    maxMtt * MTT_TABLES + WAITERS_PER_TABLE * MTT_TABLES,
  );
  const need = Math.min(computed, Number(process.env.MAX_POOL ?? computed));
  const extra = await provisionUsers(Math.max(0, need - probe.length), probe.length);
  const pool = [...probe, ...extra];
  console.log(`POOL ${pool.length} (need=${need})`);

  const cashHero = pool[0];
  const mttHero = pool[1] ?? pool[0];
  const walletNotes = [];

  const meCash = await apiSoft("/api/auth/me", { token: cashHero.token });
  walletNotes.push(`cashHero me: ${meCash.status} ${JSON.stringify(meCash.data ?? {}).slice(0, 180)}`);
  const depInfo = await apiSoft("/api/wallet/deposit-info", { token: cashHero.token });
  walletNotes.push(`deposit-info: ${depInfo.status} ${JSON.stringify(depInfo.data ?? {}).slice(0, 180)}`);
  const dep = await apiSoft("/api/wallet/deposit-requests", {
    method: "POST",
    token: cashHero.token,
    body: {
      amount_cents: 5000,
      player_note: "sim ritual play pedido de fichas",
      proof_text: "simulação ritual — comprovante mock",
    },
  });
  walletNotes.push(`deposit-request: ${dep.status} ${JSON.stringify(dep.data ?? dep.text).slice(0, 180)}`);
  const wd = await apiSoft("/api/payments/pix/withdraw", {
    method: "POST",
    token: cashHero.token,
    body: { amount: 1000, pix_key_type: "email", pix_key: "sim@zerotilt.local" },
  });
  walletNotes.push(`withdraw: ${wd.status} ${JSON.stringify(wd.data ?? wd.text).slice(0, 180)}`);
  walletNotes.push(`cashHero funds: ${JSON.stringify(await ensureFunds(cashHero))}`);
  walletNotes.push(`mttHero funds: ${JSON.stringify(await ensureFunds(mttHero))}`);

  const out = {
    mode: MODE,
    hands_target: HANDS_PER_TABLE,
    pool: pool.length,
    wallet: walletNotes,
    cash: [],
    tournaments: [],
    note: "MTT WS gameplay_ready=false — campeão/duração no teste Motor-Rust tournament_to_champion",
  };
  saveReport(out);

  let cursor = 0;
  const sessions = [];
  for (const table of playTables) {
    for (const u of pool) {
      try {
        await api("/api/lobby/leave", {
          method: "POST",
          token: u.token,
          body: { table_id: table.id },
        });
      } catch {}
    }
    const n = table.max_players;
    const seated = pool.slice(cursor, cursor + n).map((u) => ({ ...u, chips: table.min_buy_in }));
    const waiters = pool
      .slice(cursor + n, cursor + n + WAITERS_PER_TABLE)
      .map((u) => ({ ...u, chips: -1 }));
    cursor += n + WAITERS_PER_TABLE;
    if (seated.length < n) {
      console.error(`POOL curto para ${table.name}: ${seated.length}/${n}`);
    }
    sessions.push({ table, seated, waiters });
    console.log(
      `FILL ${table.name} seats=${seated.length} waiters=${waiters.length} emails=${seated.map((u) => u.email).join(",")}`,
    );
  }
  out.occupancy = sessions.map((s) => ({
    table: s.table.name,
    seated: s.seated.map((u) => ({ username: u.username, email: u.email })),
    waiters: s.waiters.map((u) => ({ username: u.username, email: u.email })),
  }));
  saveReport(out);

  let mttCursor = 0;
  for (const t of list ?? []) {
    const perTable = t.table_max_players ?? 9;
    const field = Math.min(
      perTable,
      Math.max(0, pool.length - mttCursor - WAITERS_PER_TABLE),
    );
    const waiterN = Math.min(
      WAITERS_PER_TABLE,
      Math.max(0, pool.length - mttCursor - field),
    );
    const row = {
      name: t.name,
      id: t.id,
      gameplay_ready: t.gameplay_ready,
      allow_rebuy: t.allow_rebuy,
      rebuy_max_level: t.rebuy_max_level,
      registered: 0,
      waiters: 0,
      emails: [],
      rebuy: null,
      addon: null,
    };
    for (let i = 0; i < field; i++) {
      const user = pool[mttCursor + i];
      try {
        await refreshAuth(user);
      } catch {}
      const res = await apiSoft("/api/tournament/register", {
        method: "POST",
        token: user.token,
        body: { tournament_id: t.id, wallet_mode: MODE },
      });
      if (res.ok || /já registrado|already/i.test(String(res.text))) {
        row.registered += 1;
        row.emails.push(user.email);
      } else row.error = `${res.status} ${String(res.text).slice(0, 160)}`;
      await sleep(150);
    }
    const rebuyTry = await apiSoft("/api/tournament/rebuy", {
      method: "POST",
      token: mttHero.token,
      body: { tournament_id: t.id, wallet_mode: MODE },
    });
    row.rebuy = `${rebuyTry.status} ${String(rebuyTry.text).slice(0, 160)}`;
    const addonTry = await apiSoft("/api/tournament/addon", {
      method: "POST",
      token: mttHero.token,
      body: { tournament_id: t.id, wallet_mode: MODE },
    });
    row.addon = `${addonTry.status} ${String(addonTry.text).slice(0, 160)}`;
    for (let i = 0; i < waiterN; i++) {
      const user = pool[mttCursor + field + i];
      const res = await apiSoft("/api/tournament/register", {
        method: "POST",
        token: user.token,
        body: { tournament_id: t.id, wallet_mode: MODE },
      });
      if (res.ok || /já registrado|already/i.test(String(res.text))) {
        row.waiters += 1;
        row.emails.push(user.email);
      } else row.waiter_error = `${res.status} ${String(res.text).slice(0, 160)}`;
      await sleep(150);
    }
    mttCursor += field + waiterN;
    out.tournaments.push(row);
    console.log(
      `MTT ${t.name} registered=${row.registered} waiters=${row.waiters} emails=${row.emails.join(",")}`,
    );
    saveReport(out);
  }

  console.log("TODAS as mesas cash + inscrições MTT com pessoas distintas. Abrindo WS em paralelo.");

  const cashRows = await Promise.all(
    sessions.map(async ({ table, seated, waiters }) => {
      console.log(`\n=== CASH ${table.name} seats=${seated.length} waiters=${waiters.length} ===`);
      try {
        return await playTable(table, seated, waiters);
      } catch (e) {
        console.error(`FAIL ${table.name}: ${e.message}`);
        for (const u of [...seated, ...waiters]) {
          try {
            await api("/api/lobby/leave", {
              method: "POST",
              token: u.token,
              body: { table_id: table.id },
            });
          } catch {}
        }
        return { name: table.name, ok: false, error: e.message, hands: 0 };
      }
    }),
  );
  out.cash = cashRows;
  saveReport(out);

  out.status = out.cash.length && out.cash.every((c) => c.ok) ? "PASS" : "PARTIAL";
  saveReport(out);
  console.log(JSON.stringify(out, null, 2));
  if (out.status !== "PASS") process.exitCode = 1;
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
