"""Para os bots da casa numa mesa via endpoint admin (uso unico, remoto).
Uso: python3 stop_bots.py <admin_id> <username> <token_version> <table_id>
Le JWT_SECRET do .env local da VPS. Imprime so o resultado (nunca o segredo).
"""
import base64
import hashlib
import hmac
import json
import sys
import time
import urllib.request

admin_id, username, token_version, table_id = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]

secret = None
with open("/opt/poker-platform/Infraestrutura-Docker/.env", encoding="utf-8") as f:
    for line in f:
        if line.startswith("JWT_SECRET="):
            secret = line.strip().split("=", 1)[1]
            break
if not secret:
    print("FALHOU: JWT_SECRET ausente");
    sys.exit(1)

def b64url(b: bytes) -> str:
    return base64.urlsafe_b64encode(b).rstrip(b"=").decode()

now = int(time.time())
claims = {"sub": admin_id, "username": username, "role": "admin",
          "token_version": token_version, "iat": now, "exp": now + 300, "type": "access"}
signing = f"{b64url(json.dumps({'alg': 'HS256', 'typ': 'JWT'}).encode())}.{b64url(json.dumps(claims).encode())}"
jwt = f"{signing}.{b64url(hmac.new(secret.encode(), signing.encode(), hashlib.sha256).digest())}"

def call(method, path, body=None):
    req = urllib.request.Request(
        f"https://zerotiltpoker.net{path}", method=method,
        headers={"Authorization": f"Bearer {jwt}", "Content-Type": "application/json"},
        data=json.dumps(body).encode() if body is not None else None)
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return r.status, r.read().decode()[:500]
    except Exception as e:  # noqa: BLE001 - diagnostico remoto enxuto
        return -1, str(e)[:200]

# A API escuta em 3000 no compose; via Caddy local tambem vale. Tenta direto.
s, b = call("GET", "/api/admin/bots/status")
print("status:", s, b[:300])
s, b = call("POST", "/api/admin/bots/stop", {"table_id": table_id})
print("stop:", s, b[:300])
