import { randomBytes } from "node:crypto";

const BASE_URL = process.env.BASE_URL ?? "https://zerotiltpoker.net";
const MAIL_API = "https://api.mail.tm";
const USER_COUNT = Number(process.env.E2E_USERS ?? 10);
const HAND_TARGET = Number(process.env.E2E_HANDS ?? 100);
const RUN_ID = new Date().toISOString().replace(/\D/g, "").slice(0, 12);
const REQUEST_TIMEOUT_MS = 30_000;
const EMAIL_TIMEOUT_MS = 150_000;
const PLAY_TIMEOUT_MS = 9 * 60_000;

if (process.env.ALLOW_TEMP_MAIL !== "true") {
  throw new Error("Set ALLOW_TEMP_MAIL=true after explicit authorization to use Mail.tm");
}
if (USER_COUNT !== 10 || HAND_TARGET !== 100) {
  throw new Error("This production smoke test is intentionally fixed at 10 users and 100 hands");
}

const users = [];
const sockets = [];
const joinedSeats = [];
let firstAuthRequestAt = 0;

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function collectionMembers(body) {
  return Array.isArray(body) ? body : (body?.["hydra:member"] ?? []);
}

function randomPassword(prefix) {
  return `${prefix}!A9${randomBytes(24).toString("base64url")}`;
}

async function fetchJson(url, options = {}, retries = 2) {
  for (let attempt = 0; ; attempt += 1) {
    const response = await fetch(url, {
      ...options,
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      headers: {
        Accept: "application/json",
        "User-Agent": "ZeroTilt-Live-E2E/1.0",
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
    const detail = typeof body === "string" ? body.slice(0, 240) : JSON.stringify(body).slice(0, 240);
    throw new Error(`${options.method ?? "GET"} ${new URL(url).pathname} -> HTTP ${response.status}: ${detail}`);
  }
}

async function pokerRequest(path, { method = "GET", body, token, retries = 2 } = {}) {
  if (path.startsWith("/api/auth/") && firstAuthRequestAt === 0) firstAuthRequestAt = Date.now();
  return fetchJson(`${BASE_URL}${path}`, {
    method,
    body: body === undefined ? undefined : JSON.stringify(body),
    headers: token ? { Authorization: `Bearer ${token}` } : undefined,
  }, retries);
}

async function createMailbox(domain, index) {
  const address = `zte2e-${RUN_ID}-${String(index).padStart(2, "0")}-${randomBytes(3).toString("hex")}@${domain}`;
  const password = randomPassword("Tm");
  const account = await fetchJson(`${MAIL_API}/accounts`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  const auth = await fetchJson(`${MAIL_API}/token`, {
    method: "POST",
    body: JSON.stringify({ address, password }),
  });
  return {
    mailboxId: account.body.id,
    mailToken: auth.body.token,
    email: address,
    username: `zte2e${RUN_ID}${String(index).padStart(2, "0")}`,
    password: randomPassword("Zt"),
  };
}

function messageText(message) {
  const parts = [message.subject, message.intro, message.text];
  if (Array.isArray(message.html)) parts.push(...message.html);
  else parts.push(message.html);
  return parts.filter(Boolean).join("\n");
}

function extractCode(message) {
  const content = messageText(message);
  const contextual = content.match(/c[oó]digo(?:\s+de\s+verifica[cç][aã]o)?[\s\S]{0,500}?(\d{6})/i);
  if (contextual) return contextual[1];
  return content.match(/(?<!\d)(\d{6})(?!\d)/)?.[1] ?? null;
}

async function receiveCodes() {
  const pending = new Set(users);
  const deadline = Date.now() + EMAIL_TIMEOUT_MS;
  while (pending.size > 0 && Date.now() < deadline) {
    for (const user of [...pending]) {
      const headers = { Authorization: `Bearer ${user.mailToken}` };
      const inbox = await fetchJson(`${MAIL_API}/messages?page=1`, { headers });
      const summary = collectionMembers(inbox.body).find((item) => /Zero Tilt/i.test(item.subject ?? ""));
      if (summary) {
        const detail = await fetchJson(`${MAIL_API}/messages/${summary.id}`, { headers });
        const code = extractCode(detail.body);
        if (code) {
          user.code = code;
          pending.delete(user);
        }
      }
      await sleep(160);
    }
    if (pending.size > 0) await sleep(1_500);
  }
  if (pending.size > 0) {
    throw new Error(`Verification email timeout for ${pending.size} mailbox(es)`);
  }
}

function allocateTables(tables) {
  const eligible = tables
    .map((table) => ({ ...table, available: table.max_players - table.players }))
    .filter((table) => table.available >= 2 && table.min_buy_in <= 100_000)
    .sort((a, b) => b.available - a.available || a.big_blind - b.big_blind);
  const groups = [];
  let remaining = USER_COUNT;
  let cursor = 0;
  for (const table of eligible) {
    if (remaining === 0) break;
    let size = Math.min(table.available, remaining);
    if (remaining - size === 1) size -= 1;
    if (size < 2) continue;
    groups.push({ table, users: users.slice(cursor, cursor + size) });
    cursor += size;
    remaining -= size;
  }
  if (remaining !== 0) throw new Error("There is not enough two-or-more-player table capacity for 10 users");
  let handsLeft = HAND_TARGET;
  groups.forEach((group, index) => {
    const groupsLeft = groups.length - index;
    group.targetHands = Math.floor(handsLeft / groupsLeft);
    handsLeft -= group.targetHands;
  });
  return groups;
}

function wsUrl(tableId, ticket) {
  const url = new URL(BASE_URL);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = `/ws/game/${encodeURIComponent(tableId)}`;
  url.search = new URLSearchParams({ ticket }).toString();
  return url.toString();
}

function openSocket(user, group, ticket, observer) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(wsUrl(group.table.id, ticket));
    sockets.push(ws);
    user.socket = ws;
    user.lastActionSignature = null;
    const openTimer = setTimeout(() => reject(new Error(`WebSocket open timeout for ${user.username}`)), 20_000);
    ws.addEventListener("open", () => {
      clearTimeout(openTimer);
      ws.send(JSON.stringify({ type: "get_table_info" }));
      resolve();
    }, { once: true });
    ws.addEventListener("error", () => reject(new Error(`WebSocket error for ${user.username}`)), { once: true });
    ws.addEventListener("message", (event) => {
      let message;
      try {
        message = JSON.parse(String(event.data));
      } catch {
        group.fail(new Error(`Invalid WebSocket JSON for ${user.username}`));
        return;
      }
      if (message.type === "welcome") user.playerId = message.player_id;
      if (message.type === "error") {
        group.fail(new Error(`WebSocket server error for ${user.username}: ${message.message}`));
        return;
      }
      if (message.type !== "table_state") return;
      if (message.is_finished) user.lastActionSignature = null;
      if (observer) {
        if (message.is_finished === false) group.handInProgress = true;
        if (message.is_finished === true && group.handInProgress) {
          group.handInProgress = false;
          group.completedHands += 1;
          if (group.completedHands % 10 === 0 || group.completedHands === group.targetHands) {
            console.log(`PROGRESS table=${group.table.name} hands=${group.completedHands}/${group.targetHands}`);
          }
          if (group.completedHands >= group.targetHands) group.finish();
        }
      }
      if (!user.playerId || message.is_finished || group.finished) return;
      const me = (message.players ?? []).find((player) => player.id === user.playerId);
      if (!me?.is_active) return;
      const actions = (message.available_actions ?? []).map((action) => action.toLowerCase());
      if (!actions.includes("fold")) {
        group.fail(new Error(`Active player ${user.username} did not receive legal actions`));
        return;
      }
      const signature = JSON.stringify([
        message.stage,
        message.current_bet_to_match,
        me.chips,
        me.bet,
        (message.pots ?? []).map((pot) => pot.amount),
      ]);
      if (signature === user.lastActionSignature) return;
      user.lastActionSignature = signature;
      ws.send(JSON.stringify({ type: "action", action: "fold", amount: 0 }));
    });
  });
}

async function playGroup(group) {
  return new Promise(async (resolve, reject) => {
    group.completedHands = 0;
    group.handInProgress = false;
    group.finished = false;
    let settled = false;
    const timer = setTimeout(() => group.fail(new Error(`Gameplay timeout at ${group.table.name}`)), PLAY_TIMEOUT_MS);
    group.fail = (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      reject(error);
    };
    group.finish = () => {
      if (settled) return;
      settled = true;
      group.finished = true;
      clearTimeout(timer);
      resolve();
    };
    try {
      const tickets = [];
      for (const user of group.users) {
        const response = await pokerRequest(`/api/lobby/tables/${group.table.id}/ws-ticket`, {
          method: "POST",
          token: user.token,
          retries: 3,
        });
        tickets.push(response.body.ticket);
      }
      await openSocket(group.users[0], group, tickets[0], true);
      await Promise.all(group.users.slice(1).map((user, index) => openSocket(user, group, tickets[index + 1], false)));
    } catch (error) {
      group.fail(error);
    }
  });
}

async function cleanup() {
  for (const ws of sockets) {
    try { ws.close(); } catch { /* best effort */ }
  }
  await sleep(500);
  for (const seat of joinedSeats) {
    if (seat.left) continue;
    try {
      await pokerRequest("/api/lobby/leave", {
        method: "POST",
        body: { table_id: seat.tableId },
        token: seat.user.token,
      });
      seat.left = true;
    } catch (error) {
      seat.leaveError = error.message;
    }
  }
  for (const user of users) {
    if (!user.mailToken || !user.mailboxId) continue;
    try {
      await fetch(`${MAIL_API}/accounts/${user.mailboxId}`, {
        method: "DELETE",
        headers: { Authorization: `Bearer ${user.mailToken}` },
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      });
      user.mailboxDeleted = true;
    } catch {
      user.mailboxDeleted = false;
    }
  }
}

let result;
try {
  const domains = await fetchJson(`${MAIL_API}/domains?page=1`);
  const domain = collectionMembers(domains.body).find((item) => item.isActive && !item.isPrivate)?.domain;
  if (!domain) throw new Error("Mail.tm returned no active public domain");

  for (let index = 0; index < USER_COUNT; index += 1) {
    users.push(await createMailbox(domain, index));
    await sleep(170);
  }
  console.log("PHASE mailboxes=10");

  for (const user of users) {
    const response = await pokerRequest("/api/auth/register", {
      method: "POST",
      body: {
        username: user.username,
        email: user.email,
        password: user.password,
        password_confirm: user.password,
      },
    });
    if (!response.body.email_verification_required) throw new Error(`Email verification not required for ${user.username}`);
    user.registered = true;
  }
  console.log("PHASE registered=10");

  await receiveCodes();
  console.log("PHASE emails_received=10");

  for (const user of users) {
    const verified = await pokerRequest("/api/auth/verify-email", {
      method: "POST",
      body: { email: user.email, code: user.code },
    });
    if (!verified.body.token) throw new Error(`Verification did not issue token for ${user.username}`);
    user.code = undefined;
    user.verified = true;
  }
  console.log("PHASE verified=10");

  for (const user of users) {
    const loggedIn = await pokerRequest("/api/auth/login", {
      method: "POST",
      body: { email: user.email, password: user.password },
    });
    if (!loggedIn.body.token) throw new Error(`Login did not issue token for ${user.username}`);
    user.token = loggedIn.body.token;
    user.loggedIn = true;
    const lobby = await pokerRequest("/api/lobby/tables", { token: user.token });
    if (!Array.isArray(lobby.body)) throw new Error(`Lobby contract failed for ${user.username}`);
    user.lobbyOpened = true;
  }
  console.log("PHASE logged_in=10 lobby_opened=10");

  const tables = (await pokerRequest("/api/lobby/tables", { token: users[0].token })).body;
  const groups = allocateTables(tables);
  for (const group of groups) {
    for (const user of group.users) {
      const joined = await pokerRequest("/api/lobby/join", {
        method: "POST",
        body: { table_id: group.table.id, buy_in: group.table.min_buy_in },
        token: user.token,
      });
      if (!Number.isInteger(joined.body.seat)) throw new Error(`Join contract failed for ${user.username}`);
      user.tableId = group.table.id;
      user.seat = joined.body.seat;
      joinedSeats.push({ user, tableId: group.table.id, left: false });
    }
  }
  console.log(`PHASE joined=10 tables=${groups.length} allocation=${groups.map((group) => group.users.length).join("+")}`);

  const minimumTicketTime = firstAuthRequestAt + 62_000;
  if (Date.now() < minimumTicketTime) await sleep(minimumTicketTime - Date.now());
  const playStartedAt = Math.floor(Date.now() / 1_000) - 2;
  await Promise.all(groups.map(playGroup));
  console.log(`PHASE hands_played=${groups.reduce((sum, group) => sum + group.completedHands, 0)}`);

  for (const ws of sockets) {
    try { ws.close(); } catch { /* best effort */ }
  }
  await sleep(800);
  for (const seat of joinedSeats) {
    const left = await pokerRequest("/api/lobby/leave", {
      method: "POST",
      body: { table_id: seat.tableId },
      token: seat.user.token,
    });
    if (!Number.isFinite(left.body.chips)) throw new Error(`Cash-out contract failed for ${seat.user.username}`);
    seat.left = true;
  }
  console.log("PHASE cash_out=10");

  const handIds = new Set();
  for (const group of groups) {
    const history = await pokerRequest(`/api/tables/${group.table.id}/history`, { token: group.users[0].token });
    const currentRun = history.body.filter((hand) => hand.created_at >= playStartedAt);
    for (const hand of currentRun) handIds.add(hand.hand_id);
    group.persistedHands = currentRun.length;
  }
  if (handIds.size < HAND_TARGET) throw new Error(`Only ${handIds.size}/${HAND_TARGET} hands were observable in persisted history`);

  result = {
    status: "PASS",
    run: RUN_ID,
    registered: users.filter((user) => user.registered).length,
    emailsReceived: users.filter((user) => user.verified).length,
    verified: users.filter((user) => user.verified).length,
    loggedIn: users.filter((user) => user.loggedIn).length,
    lobbyOpened: users.filter((user) => user.lobbyOpened).length,
    joined: joinedSeats.length,
    handsPlayed: groups.reduce((sum, group) => sum + group.completedHands, 0),
    persistedHands: handIds.size,
    cashedOut: joinedSeats.filter((seat) => seat.left).length,
    tables: groups.map((group) => ({
      id: group.table.id,
      name: group.table.name,
      users: group.users.length,
      hands: group.completedHands,
      persisted: group.persistedHands,
    })),
    accounts: users.map((user) => ({ username: user.username, email: user.email })),
  };
} finally {
  await cleanup();
}

result.mailboxesDeleted = users.filter((user) => user.mailboxDeleted).length;
console.log(`E2E_RESULT ${JSON.stringify(result)}`);
