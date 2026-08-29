/**
 * Smoke seeded users against Real (or Play) catalog — one table at a time.
 *
 *   node scripts/live-e2e-seeded-catalog.mjs
 *   MODE=real HANDS_PER_TABLE=1
 */

import WebSocket from "ws";

const BASE_URL = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const MODE = (process.env.MODE ?? "real").toLowerCase();
const HANDS_PER_TABLE = Number(process.env.HANDS_PER_TABLE ?? 1);
const USER1 = {
  email: process.env.E2E_EMAIL1 ?? "e2ecat01@zerotilt.local",
  password: process.env.E2E_PASS ?? "TestPass1!",
  username: "e2ecat01",
};
const USER2 = {
  email: process.env.E2E_EMAIL2 ?? "e2ecat02@zerotilt.local",
  password: process.env.E2E_PASS ?? "TestPass1!",
  username: "e2ecat02",
};
const PLAY_TIMEOUT_MS = 4 * 60_000;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function api(path, { method = "GET", body, token } = {}) {
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    signal: AbortSignal.timeout(30_000),
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      "User-Agent": "ZeroTilt-SeededCatalog/1.0",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let data = null;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    data = text;
  }
  if (!res.ok) {
    throw new Error(`${method} ${path} -> ${res.status}: ${String(text).slice(0, 280)}`);
  }
  return data;
}

function wsUrl(tableId, ticket) {
  const url = new URL(BASE_URL);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = `/ws/game/${encodeURIComponent(tableId)}`;
  url.search = new URLSearchParams({ ticket }).toString();
  return url.toString();
}

async function login(user) {
  const data = await api("/api/auth/login", {
    method: "POST",
    body: { email: user.email, password: user.password },
  });
  if (!data.token) throw new Error(`login failed for ${user.email}`);
  user.token = data.token;
  user.userId = data.user_id ?? data.id ?? null;
  console.log(`LOGIN ok ${user.username}`);
}

async function playOneHand(table, players) {
  return new Promise(async (resolve, reject) => {
    let completed = 0;
    let handInProgress = false;
    let settled = false;
    const sockets = [];
    const timer = setTimeout(() => fail(new Error(`timeout ${table.name}`)), PLAY_TIMEOUT_MS);
    const fail = (err) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      for (const ws of sockets) try { ws.close(); } catch {}
      reject(err);
    };
    const done = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      for (const ws of sockets) try { ws.close(); } catch {}
      resolve(completed);
    };
    try {
      const tickets = [];
      for (const p of players) {
        const t = await api(`/api/lobby/tables/${table.id}/ws-ticket`, {
          method: "POST",
          token: p.token,
        });
        tickets.push(t.ticket);
      }
      await Promise.all(
        players.map(
          (user, idx) =>
            new Promise((res, rej) => {
              const ws = new WebSocket(wsUrl(table.id, tickets[idx]));
              sockets.push(ws);
              user.playerId = null;
              user.lastSig = null;
              const observe = idx === 0;
              const ot = setTimeout(() => rej(new Error(`ws open ${user.username}`)), 20_000);
              ws.addEventListener("open", () => {
                clearTimeout(ot);
                ws.send(JSON.stringify({ type: "get_table_info" }));
                res();
              }, { once: true });
              ws.addEventListener("error", () => rej(new Error(`ws err ${user.username}`)), {
                once: true,
              });
              ws.addEventListener("message", (ev) => {
                let msg;
                try {
                  msg = JSON.parse(String(ev.data));
                } catch {
                  return;
                }
                if (msg.type === "welcome") user.playerId = msg.player_id;
                if (msg.type === "error") {
                  fail(new Error(`${table.name} ws: ${msg.message}`));
                  return;
                }
                if (msg.type !== "table_state") return;
                if (msg.is_finished) user.lastSig = null;
                if (observe) {
                  if (msg.is_finished === false) handInProgress = true;
                  if (msg.is_finished === true && handInProgress) {
                    handInProgress = false;
                    completed += 1;
                    console.log(`HAND ${table.name} ${completed}/${HANDS_PER_TABLE}`);
                    if (completed >= HANDS_PER_TABLE) done();
                  }
                }
                if (!user.playerId || msg.is_finished || settled) return;
                const me = (msg.players ?? []).find((p) => p.id === user.playerId);
                if (!me?.is_active) return;
                const actions = (msg.available_actions ?? []).map((a) => String(a).toLowerCase());
                const sig = JSON.stringify([msg.stage, msg.current_bet_to_match, me.chips, me.bet]);
                if (sig === user.lastSig) return;
                user.lastSig = sig;
                if (actions.includes("check")) {
                  ws.send(JSON.stringify({ type: "action", action: "check", amount: 0 }));
                } else if (actions.includes("call")) {
                  ws.send(JSON.stringify({ type: "action", action: "call", amount: 0 }));
                } else if (actions.includes("fold")) {
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

async function main() {
  const users = [USER1, USER2];
  for (const u of users) await login(u);

  const tables = await api(`/api/lobby/tables?mode=${MODE}`, { token: users[0].token });
  if (!Array.isArray(tables) || tables.length === 0) {
    throw new Error(`no tables mode=${MODE}`);
  }
  tables.sort((a, b) => a.big_blind - b.big_blind || a.name.localeCompare(b.name));
  console.log(`TABLES ${tables.map((t) => t.name).join(" | ")}`);

  const report = { mode: MODE, tables: [] };

  for (const table of tables) {
    const row = { name: table.name, variant: table.poker_variant, ok: false, hands: 0, error: null };
    try {
      for (const u of users) {
        await api("/api/lobby/join", {
          method: "POST",
          token: u.token,
          body: { table_id: table.id, buy_in: table.min_buy_in, wallet_mode: MODE },
        });
      }
      console.log(`JOINED ${table.name}`);
      await sleep(2000);
      row.hands = await playOneHand(table, users);
      row.ok = row.hands >= HANDS_PER_TABLE;
      for (const u of users) {
        try {
          await api("/api/lobby/leave", {
            method: "POST",
            token: u.token,
            body: { table_id: table.id },
          });
        } catch (e) {
          console.log(`leave warn: ${e.message}`);
        }
      }
      console.log(`PASS ${table.name}`);
    } catch (e) {
      row.error = e.message;
      console.error(`FAIL ${table.name}: ${e.message}`);
      for (const u of users) {
        try {
          await api("/api/lobby/leave", {
            method: "POST",
            token: u.token,
            body: { table_id: table.id },
          });
        } catch {}
      }
    }
    report.tables.push(row);
    await sleep(1000);
  }

  // Tournament registration smoke
  report.tournaments = [];
  for (const mode of ["real", "play"]) {
    try {
      const list = await api(`/api/lobby/tournaments?mode=${mode}`, { token: users[0].token });
      for (const t of list ?? []) {
        try {
          await api("/api/tournament/register", {
            method: "POST",
            token: users[0].token,
            body: { tournament_id: t.id, wallet_mode: mode },
          });
          report.tournaments.push({ name: t.name, mode, ok: true });
          console.log(`TOURNEY OK ${mode} ${t.name}`);
        } catch (e) {
          const already = /já registrado|already registered/i.test(e.message);
          report.tournaments.push({
            name: t.name,
            mode,
            ok: already,
            error: already ? null : e.message,
          });
          console.log(
            already
              ? `TOURNEY OK ${mode} ${t.name} (already registered)`
              : `TOURNEY FAIL ${mode} ${t.name}: ${e.message}`,
          );
        }
      }
    } catch (e) {
      console.log(`TOURNEY list ${mode}: ${e.message}`);
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
