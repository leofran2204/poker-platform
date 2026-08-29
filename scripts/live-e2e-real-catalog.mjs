/**
 * Smoke Jogo Real: cadastro → creditar (via env ADMIN ou pré-crédito) →
 * entrar em cada mesa Real OPEN e completar ≥1 mão.
 *
 * Uso:
 *   ALLOW_TEMP_MAIL=true node scripts/live-e2e-real-catalog.mjs
 * Env:
 *   BASE_URL=https://zerotiltpoker.net
 *   ADMIN_TOKEN=...   (opcional: POST /api/admin/users/:id/adjust-balance)
 *   HANDS_PER_TABLE=1
 *   MODE=real         (ou play)
 */

import { randomBytes } from "node:crypto";

const BASE_URL = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const MAIL_API = "https://api.mail.tm";
const MODE = (process.env.MODE ?? "real").toLowerCase();
const HANDS_PER_TABLE = Number(process.env.HANDS_PER_TABLE ?? 1);
const ADMIN_TOKEN = process.env.ADMIN_TOKEN ?? "";
const RUN_ID = new Date().toISOString().replace(/\D/g, "").slice(0, 12);
const REQUEST_TIMEOUT_MS = 30_000;
const PLAY_TIMEOUT_MS = 4 * 60_000;

if (process.env.ALLOW_TEMP_MAIL !== "true") {
  throw new Error("Set ALLOW_TEMP_MAIL=true to authorize Mail.tm usage");
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function collectionMembers(body) {
  return Array.isArray(body) ? body : (body?.["hydra:member"] ?? []);
}

async function fetchJson(url, options = {}, retries = 2) {
  for (let attempt = 0; ; attempt += 1) {
    const response = await fetch(url, {
      ...options,
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      headers: {
        Accept: "application/json",
        "User-Agent": "ZeroTilt-RealCatalog-E2E/1.0",
        ...(options.body ? { "Content-Type": "application/json" } : {}),
        ...(options.headers ?? {}),
      },
    });
    const text = await response.text();
    let body = null;
    if (text) {
      try {
        body = JSON.parse(text);
      } catch {
        body = text;
      }
    }
    if (response.ok) return { status: response.status, body };
    if (response.status === 429 && attempt < retries) {
      const retryAfter = Number(response.headers.get("retry-after") ?? 65);
      await sleep(Math.max(2, retryAfter) * 1_000);
      continue;
    }
    const detail =
      typeof body === "string" ? body.slice(0, 300) : JSON.stringify(body).slice(0, 300);
    throw new Error(
      `${options.method ?? "GET"} ${new URL(url).pathname} -> HTTP ${response.status}: ${detail}`,
    );
  }
}

async function api(path, { method = "GET", body, token, retries = 2 } = {}) {
  return fetchJson(
    `${BASE_URL}${path}`,
    {
      method,
      body: body === undefined ? undefined : JSON.stringify(body),
      headers: token ? { Authorization: `Bearer ${token}` } : undefined,
    },
    retries,
  );
}

async function createMailbox(domain, index) {
  const address = `ztcat-${RUN_ID}-${index}-${randomBytes(3).toString("hex")}@${domain}`;
  const password = `Cat!A9${randomBytes(20).toString("base64url")}`;
  await fetchJson(`${MAIL_API}/accounts`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  const tokenRes = await fetchJson(`${MAIL_API}/token`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  const username = `ztc${RUN_ID.slice(-6)}${index}${randomBytes(2).toString("hex")}`.slice(0, 20);
  return {
    email: address,
    password: `Pk!B7${randomBytes(18).toString("base64url")}`,
    mailPassword: password,
    mailToken: tokenRes.body.token,
    mailboxId: null,
    username,
    code: null,
    token: null,
    userId: null,
  };
}

async function receiveCodes(users) {
  const pending = new Set(users.map((u) => u.email));
  const deadline = Date.now() + 150_000;
  while (pending.size > 0 && Date.now() < deadline) {
    for (const user of users) {
      if (!pending.has(user.email)) continue;
      const msgs = await fetchJson(`${MAIL_API}/messages?page=1`, {
        headers: { Authorization: `Bearer ${user.mailToken}` },
      });
      const list = collectionMembers(msgs.body);
      for (const m of list) {
        const full = await fetchJson(`${MAIL_API}/messages/${m.id}`, {
          headers: { Authorization: `Bearer ${user.mailToken}` },
        });
        const text = `${full.body?.text ?? ""} ${full.body?.html ?? ""}`;
        const match = text.match(/\b(\d{6})\b/);
        if (match) {
          user.code = match[1];
          pending.delete(user.email);
          break;
        }
      }
    }
    if (pending.size) await sleep(4_000);
  }
  if (pending.size) throw new Error(`Email code timeout for ${[...pending].join(",")}`);
}

function wsUrl(tableId, ticket) {
  const url = new URL(BASE_URL);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = `/ws/game/${encodeURIComponent(tableId)}`;
  url.search = new URLSearchParams({ ticket }).toString();
  return url.toString();
}

async function playOneHand(table, players) {
  return new Promise(async (resolve, reject) => {
    let completed = 0;
    let handInProgress = false;
    let settled = false;
    const sockets = [];
    const timer = setTimeout(
      () => fail(new Error(`timeout playing ${table.name}`)),
      PLAY_TIMEOUT_MS,
    );
    const fail = (err) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      for (const ws of sockets) {
        try {
          ws.close();
        } catch {
          /* */
        }
      }
      reject(err);
    };
    const done = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      for (const ws of sockets) {
        try {
          ws.close();
        } catch {
          /* */
        }
      }
      resolve(completed);
    };

    try {
      const tickets = [];
      for (const p of players) {
        const t = await api(`/api/lobby/tables/${table.id}/ws-ticket`, {
          method: "POST",
          token: p.token,
          retries: 3,
        });
        tickets.push(t.body.ticket);
      }

      await Promise.all(
        players.map(
          (user, idx) =>
            new Promise((res, rej) => {
              const ws = new WebSocket(wsUrl(table.id, tickets[idx]));
              sockets.push(ws);
              user.socket = ws;
              user.playerId = null;
              user.lastSig = null;
              const isObserver = idx === 0;
              const openTimer = setTimeout(
                () => rej(new Error(`ws open timeout ${user.username}`)),
                20_000,
              );
              ws.addEventListener(
                "open",
                () => {
                  clearTimeout(openTimer);
                  ws.send(JSON.stringify({ type: "get_table_info" }));
                  res();
                },
                { once: true },
              );
              ws.addEventListener(
                "error",
                () => rej(new Error(`ws error ${user.username}`)),
                { once: true },
              );
              ws.addEventListener("message", (event) => {
                let msg;
                try {
                  msg = JSON.parse(String(event.data));
                } catch {
                  fail(new Error(`bad json ${user.username}`));
                  return;
                }
                if (msg.type === "welcome") user.playerId = msg.player_id;
                if (msg.type === "error") {
                  fail(new Error(`ws server ${user.username}: ${msg.message}`));
                  return;
                }
                if (msg.type !== "table_state") return;
                if (msg.is_finished) user.lastSig = null;
                if (isObserver) {
                  if (msg.is_finished === false) handInProgress = true;
                  if (msg.is_finished === true && handInProgress) {
                    handInProgress = false;
                    completed += 1;
                    console.log(
                      `PROGRESS table=${table.name} hands=${completed}/${HANDS_PER_TABLE}`,
                    );
                    if (completed >= HANDS_PER_TABLE) done();
                  }
                }
                if (!user.playerId || msg.is_finished || settled) return;
                const me = (msg.players ?? []).find((p) => p.id === user.playerId);
                if (!me?.is_active) return;
                const actions = (msg.available_actions ?? []).map((a) => a.toLowerCase());
                if (!actions.includes("fold") && !actions.includes("check") && !actions.includes("call")) {
                  return;
                }
                const sig = JSON.stringify([
                  msg.stage,
                  msg.current_bet_to_match,
                  me.chips,
                  me.bet,
                ]);
                if (sig === user.lastSig) return;
                user.lastSig = sig;
                // Prefer check/call to keep pots alive until someone can fold end
                if (actions.includes("check")) {
                  ws.send(JSON.stringify({ type: "action", action: "check", amount: 0 }));
                } else if (actions.includes("call")) {
                  ws.send(JSON.stringify({ type: "action", action: "call", amount: 0 }));
                } else {
                  ws.send(JSON.stringify({ type: "action", action: "fold", amount: 0 }));
                }
              });
            }),
        ),
      );
    } catch (e) {
      fail(e);
    }
  });
}

async function creditReal(user, amountCents) {
  if (!ADMIN_TOKEN) {
    console.log(`WARN no ADMIN_TOKEN — assume ${user.username} already funded (${amountCents})`);
    return;
  }
  // Resolve user id from /api/auth/me if available
  let userId = user.userId;
  if (!userId) {
    try {
      const me = await api("/api/auth/me", { token: user.token });
      userId = me.body?.id ?? me.body?.user_id;
      user.userId = userId;
    } catch {
      /* */
    }
  }
  if (!userId) throw new Error(`Cannot resolve user id for ${user.username}`);
  await api(`/api/admin/users/${userId}/adjust-balance`, {
    method: "POST",
    token: ADMIN_TOKEN,
    body: {
      delta_cents: amountCents,
      reason: `e2e-catalog-${RUN_ID}`,
      wallet: MODE === "play" ? "pm_cash" : "real",
    },
  });
  console.log(`CREDITED ${user.username} +${amountCents} (${MODE})`);
}

async function main() {
  const report = { status: "FAIL", run: RUN_ID, mode: MODE, tables: [] };

  const domains = await fetchJson(`${MAIL_API}/domains?page=1`);
  const domain = collectionMembers(domains.body).find((d) => d.isActive && !d.isPrivate)?.domain;
  if (!domain) throw new Error("No Mail.tm domain");

  const users = [];
  for (let i = 0; i < 2; i += 1) {
    users.push(await createMailbox(domain, i));
    await sleep(200);
  }
  console.log("PHASE mailboxes=2");

  for (const user of users) {
    await api("/api/auth/register", {
      method: "POST",
      body: {
        username: user.username,
        email: user.email,
        password: user.password,
        password_confirm: user.password,
      },
    });
  }
  console.log("PHASE registered=2");

  await receiveCodes(users);
  for (const user of users) {
    const verified = await api("/api/auth/verify-email", {
      method: "POST",
      body: { email: user.email, code: user.code },
    });
    user.token = verified.body.token;
    user.userId = verified.body.user_id ?? verified.body.id ?? null;
  }
  console.log("PHASE verified=2");

  for (const user of users) {
    const login = await api("/api/auth/login", {
      method: "POST",
      body: { email: user.email, password: user.password },
    });
    user.token = login.body.token;
  }
  console.log("PHASE login=2");

  // Fund enough for largest frente (Omaha R$100 = 10000) × margin
  for (const user of users) {
    await creditReal(user, 500_000);
  }

  const lobby = await api(`/api/lobby/tables?mode=${MODE}`, { token: users[0].token });
  const tables = (lobby.body ?? []).slice().sort((a, b) => a.big_blind - b.big_blind || a.name.localeCompare(b.name));
  if (tables.length === 0) throw new Error(`No OPEN tables for mode=${MODE}`);
  console.log(
    `PHASE lobby tables=${tables.length} :: ${tables.map((t) => t.name).join(" | ")}`,
  );

  for (const table of tables) {
    const entry = {
      name: table.name,
      id: table.id,
      variant: table.poker_variant,
      blinds: `${table.small_blind}/${table.big_blind}`,
      buy_in: table.min_buy_in,
      ok: false,
      hands: 0,
      error: null,
    };
    try {
      for (const user of users) {
        await api("/api/lobby/join", {
          method: "POST",
          token: user.token,
          body: {
            table_id: table.id,
            buy_in: table.min_buy_in,
            wallet_mode: MODE,
          },
        });
      }
      console.log(`JOINED ${table.name}`);
      // rate-limit gap for ws-ticket
      await sleep(2_000);
      entry.hands = await playOneHand(table, users);
      entry.ok = entry.hands >= HANDS_PER_TABLE;
      for (const user of users) {
        try {
          await api("/api/lobby/leave", {
            method: "POST",
            token: user.token,
            body: { table_id: table.id },
          });
        } catch (e) {
          console.log(`LEAVE warn ${user.username}: ${e.message}`);
        }
      }
      console.log(`PASS table=${table.name} hands=${entry.hands}`);
    } catch (e) {
      entry.error = e.message;
      console.error(`FAIL table=${table.name}: ${e.message}`);
      for (const user of users) {
        try {
          await api("/api/lobby/leave", {
            method: "POST",
            token: user.token,
            body: { table_id: table.id },
          });
        } catch {
          /* */
        }
      }
    }
    report.tables.push(entry);
    await sleep(1_500);
  }

  // Torneios: inscrição
  report.tournaments = [];
  for (const mode of ["play", "real"]) {
    try {
      const list = await api(`/api/lobby/tournaments?mode=${mode}`, {
        token: users[0].token,
      });
      for (const t of list.body ?? []) {
        try {
          await api("/api/tournament/register", {
            method: "POST",
            token: users[0].token,
            body: { tournament_id: t.id, wallet_mode: mode },
          });
          report.tournaments.push({
            id: t.id,
            name: t.name,
            mode,
            ok: true,
          });
          console.log(`TOURNEY OK ${mode} ${t.name}`);
        } catch (e) {
          report.tournaments.push({
            id: t.id,
            name: t.name,
            mode,
            ok: false,
            error: e.message,
          });
          console.log(`TOURNEY FAIL ${mode} ${t.name}: ${e.message}`);
        }
      }
    } catch (e) {
      console.log(`TOURNEY list fail ${mode}: ${e.message}`);
    }
  }

  report.status = report.tables.every((t) => t.ok) ? "PASS" : "FAIL";
  console.log(JSON.stringify(report, null, 2));
  if (report.status !== "PASS") process.exitCode = 1;
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
