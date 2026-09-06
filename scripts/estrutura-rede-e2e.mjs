/**
 * E2E Minha Estrutura: 5 contas por convite, mesa, mãos até rake >= 100 centavos
 * e 50 mãos (VP). API_URL=http://poker_api:3000 na rede Docker.
 */
const API = process.env.API_URL ?? "http://poker_api:3000";
const PASS = "PokerDemo1";
const RAKE_TARGET = Number(process.env.RAKE_TARGET ?? 100);
const HAND_TARGET = Number(process.env.HAND_TARGET ?? 50);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function req(path, { method = "GET", body, token } = {}) {
  const res = await fetch(`${API}${path}`, {
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

function extractCode(logs, email) {
  const clean = logs.replace(/\[[0-9;]*m/g, "");
  const re = new RegExp(
    `to="${email.replace(".", "\\.")}"[^\n]*verification_code=(\\d{6})`,
  );
  const m = clean.match(re);
  if (!m) throw new Error(`código não encontrado para ${email}`);
  return m[1];
}

async function main() {
  const stamp = Date.now().toString(36);
  const names = ["raiz", "n1a", "n1b", "n2a", "n2b"].map((n) => `${n}${stamp}`);
  const emails = names.map((n) => `${n}@example.com`);

  const r0 = await req("/api/auth/register", {
    method: "POST",
    body: {
      username: names[0],
      email: emails[0],
      password: PASS,
      password_confirm: PASS,
    },
  });
  if (r0.status !== 200) throw new Error(`raiz register ${r0.status} ${JSON.stringify(r0.json)}`);

  const logs0 = process.env.MAIL_LOGS ?? "";
  if (!logs0) throw new Error("MAIL_LOGS ausente (host injeta docker logs)");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
