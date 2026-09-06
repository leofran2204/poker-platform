"""E2E Minha Estrutura: 5 cadastros por convite, mesa, mãos, 18/12 no painel."""
from __future__ import annotations

import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request

PASS = "PokerDemo1"
RAKE_TARGET = 100
HAND_TARGET = 50
API = "http://127.0.0.1:3000"


def sh(*args: str, check: bool = True) -> str:
    r = subprocess.run(args, capture_output=True, text=True)
    if check and r.returncode != 0:
        raise RuntimeError(f"{args}\n{r.stderr or r.stdout}")
    return (r.stdout or "") + (r.stderr or "")


def docker_api(path: str, *, method: str = "GET", body: dict | None = None, token: str | None = None) -> tuple[int, dict | str]:
    cmd = [
        "docker",
        "exec",
        "poker_api",
        "curl",
        "-sS",
        "-w",
        "\n__HTTP__%{http_code}",
        "-X",
        method,
        "-H",
        "Content-Type: application/json",
    ]
    if token:
        cmd += ["-H", f"Authorization: Bearer {token}"]
    if body is not None:
        cmd += ["-d", json.dumps(body)]
    cmd.append(f"http://127.0.0.1:3000{path}")
    out = sh(*cmd)
    if "__HTTP__" not in out:
        raise RuntimeError(out)
    payload, _, code = out.rpartition("__HTTP__")
    payload = payload.strip()
    try:
        parsed: dict | str = json.loads(payload) if payload else {}
    except json.JSONDecodeError:
        parsed = payload
    return int(code.strip()), parsed


def last_code(email: str) -> str:
    logs = sh("docker", "logs", "poker_api", check=False)
    clean = re.sub(r"\x1b\[[0-9;]*m", "", logs)
    matches = re.findall(
        rf'to="{re.escape(email)}"[^\n]*verification_code=(\d{{6}})',
        clean,
    )
    if not matches:
        matches = re.findall(
            rf'to={re.escape(email)}[^\n]*code=(\d{{6}})',
            clean,
        )
    if not matches:
        raise RuntimeError(f"sem código para {email}")
    return matches[-1]


def register(username: str, email: str, invite: str | None) -> None:
    body = {
        "username": username,
        "email": email,
        "password": PASS,
        "password_confirm": PASS,
    }
    if invite:
        body["invite_code"] = invite
    status, data = docker_api("/api/auth/register", method="POST", body=body)
    if status != 200:
        raise RuntimeError(f"register {username}: {status} {data}")


def activate(email: str) -> str:
    code = last_code(email)
    status, data = docker_api(
        "/api/auth/verify-email",
        method="POST",
        body={"email": email, "code": code},
    )
    if status != 200 or not isinstance(data, dict) or not data.get("token"):
        status, data = docker_api(
            "/api/auth/login",
            method="POST",
            body={"email": email, "password": PASS},
        )
    if not isinstance(data, dict) or not data.get("token"):
        raise RuntimeError(f"login {email}: {status} {data}")
    return str(data["token"])


def bootstrap_invite() -> str | None:
    out = sh(
        "docker", "exec", "poker_postgres", "psql", "-U", "user",
        "-d", "poker_db", "-tA", "-c",
        "SELECT referral_code FROM users WHERE referral_code IS NOT NULL LIMIT 1;",
        check=False,
    ).strip()
    code = out.splitlines()[0].strip() if out.strip() else ""
    return code or None


def main() -> int:
    stamp = str(int(time.time()))[-6:]
    raiz, n1a, n1b, n2a, n2b = [f"{n}{stamp}" for n in ("raiz", "n1a", "n1b", "n2a", "n2b")]
    emails = {n: f"{n}@example.com" for n in (raiz, n1a, n1b, n2a, n2b)}

    print("1) cadastros por convite")
    try:
        register(raiz, emails[raiz], None)
    except RuntimeError as exc:
        if "convite" not in str(exc).lower():
            raise
        seed = bootstrap_invite()
        if not seed:
            raise RuntimeError("servidor exige convite e nenhum referral_code existe no banco")
        print(f"   servidor exige convite; usando semente {seed} para a raiz")
        register(raiz, emails[raiz], seed)
    time.sleep(0.4)
    tok_raiz = activate(emails[raiz])
    st, me = docker_api("/api/auth/me", token=tok_raiz)
    if st != 200 or not isinstance(me, dict):
        raise RuntimeError(me)
    code_raiz = me.get("referral_code")
    print(f"   raiz={raiz} ref={code_raiz}")

    register(n1a, emails[n1a], code_raiz)
    register(n1b, emails[n1b], code_raiz)
    time.sleep(0.4)
    tok_n1a = activate(emails[n1a])
    tok_n1b = activate(emails[n1b])
    st, me_a = docker_api("/api/auth/me", token=tok_n1a)
    st2, me_b = docker_api("/api/auth/me", token=tok_n1b)
    assert isinstance(me_a, dict) and isinstance(me_b, dict)
    register(n2a, emails[n2a], me_a.get("referral_code"))
    register(n2b, emails[n2b], me_b.get("referral_code"))
    time.sleep(0.4)
    tok_n2a = activate(emails[n2a])
    tok_n2b = activate(emails[n2b])
    tokens = {raiz: tok_raiz, n1a: tok_n1a, n1b: tok_n1b, n2a: tok_n2a, n2b: tok_n2b}

    tree = sh(
        "docker",
        "exec",
        "poker_postgres",
        "psql",
        "-U",
        "user",
        "-d",
        "poker_db",
        "-c",
        "SELECT u.username, s.username AS sponsor FROM users u LEFT JOIN users s ON s.id=u.sponsored_by ORDER BY u.created_at;",
    )
    print(tree)

    st, tables = docker_api("/api/lobby/tables?mode=play")
    if st != 200 or not isinstance(tables, list) or not tables:
        raise RuntimeError("sem mesas play")
    table = next((t for t in tables if t.get("poker_variant") == "holdem"), tables[0])
    table_id = table["id"]
    buy_in = table["min_buy_in"]
    print(f"2) join mesa {table.get('name')} {table_id} buyin={buy_in}")

    for name, tok in tokens.items():
        st, data = docker_api(
            "/api/lobby/join",
            method="POST",
            body={"table_id": table_id, "buy_in": buy_in, "wallet_mode": "play"},
            token=tok,
        )
        print(f"   join {name}: {st} {data}")
        if st != 200:
            raise RuntimeError(f"join {name}")

    print("3) estrutura antes do jogo (VP ainda falso)")
    for name, tok in tokens.items():
        st, data = docker_api("/api/estrutura", token=tok)
        if st != 200 or not isinstance(data, dict) or "eligible" not in data:
            raise RuntimeError(f"estrutura {name}: {st} {data}")
        print(
            f"   {name}: eligible={data.get('eligible')} L1={len(data.get('level1') or [])} "
            f"L2={len(data.get('level2') or [])} pts={data.get('estrutura_points')}"
        )

    print(
        f"4) mãos via WS ainda não automatizadas neste script "
        f"(alvo rake>={RAKE_TARGET} centavos e {HAND_TARGET} mãos para VP)."
    )
    print("   Use o Docker rebuild + WS bots no próximo passo se o painel estiver no ar.")
    print("OK cadastro+árvore+join+GET /api/estrutura")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print("FALHOU:", exc)
        sys.exit(1)
