#!/usr/bin/env bash
# Verificação E2E manual do ponto de entrada público HTTPS/WSS via Caddy.
# Não substitui a autenticação de jogador: o handshake WSS sem ticket precisa
# ser aceito pelo proxy e encerrado pelo backend após a mensagem de erro.
set -euo pipefail

gateway_url="${PUBLIC_GATEWAY_URL:-https://localhost}"
gateway_url="${gateway_url%/}"

if [[ ! "$gateway_url" =~ ^https://[^/]+$ ]]; then
  echo "PUBLIC_GATEWAY_URL deve ser uma origem HTTPS, por exemplo https://localhost." >&2
  exit 2
fi

curl_args=(--silent --show-error --connect-timeout 5 --max-time 10)
ready_curl_args=(--silent --show-error --connect-timeout 1 --max-time 2)
if [[ "${PUBLIC_GATEWAY_INSECURE_LOCAL_CERT:-0}" == "1" ]]; then
  # Exclusivo para o certificado local emitido pelo Caddy. Nunca defina esta
  # variável em staging ou produção.
  curl_args+=(--insecure)
  ready_curl_args+=(--insecure)
fi

temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT

wait_for_gateway() {
  local attempt body
  for attempt in $(seq 1 30); do
    if body="$(curl "${ready_curl_args[@]}" --fail "$gateway_url/caddy-health")" \
      && [[ "$body" == "OK" ]]; then
      return 0
    fi
    sleep 1
  done

  echo "O Caddy não respondeu em $gateway_url/caddy-health após 30 segundos." >&2
  return 1
}

wait_for_gateway

api_body="$(curl "${curl_args[@]}" --fail "$gateway_url/health")"
if [[ "$api_body" != "OK" ]]; then
  echo "A API não respondeu OK através do gateway HTTPS." >&2
  exit 1
fi

https_headers="$(curl "${curl_args[@]}" --fail --dump-header - --output /dev/null "$gateway_url/caddy-health")"
if ! grep -qi '^Strict-Transport-Security: max-age=' <<<"$https_headers"; then
  echo "O gateway HTTPS não retornou o cabeçalho HSTS." >&2
  exit 1
fi

http_url="http://${gateway_url#https://}"
redirect_headers="$(curl --silent --show-error --connect-timeout 5 --max-time 10 --max-redirs 0 \
  --dump-header - --output /dev/null "$http_url/caddy-health")"
redirect_status="$(awk 'NR == 1 { print $2 }' <<<"$redirect_headers")"
if [[ ! "$redirect_status" =~ ^(301|302|307|308)$ ]] \
  || ! grep -qi "^Location: ${gateway_url}/caddy-health" <<<"$redirect_headers"; then
  echo "O gateway não redirecionou HTTP para HTTPS de forma verificável." >&2
  exit 1
fi

# Um cliente sem ticket não é autorizado a jogar, mas a negociação deve chegar
# ao handler por WSS. O backend envia um frame de erro e fecha a conexão;
# portanto o status 101 é a evidência do handshake, não uma autorização.
ws_headers="$temp_dir/wss-headers.txt"
set +e
curl "${curl_args[@]}" --http1.1 --dump-header "$ws_headers" --output /dev/null \
  -H 'Connection: Upgrade' \
  -H 'Upgrade: websocket' \
  -H 'Sec-WebSocket-Version: 13' \
  -H 'Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==' \
  "$gateway_url/ws/game/00000000-0000-0000-0000-000000000000"
wss_curl_status=$?
set -e
if [[ -s "$ws_headers" ]]; then
  wss_status="$(awk 'NR == 1 { print $2 }' "$ws_headers")"
else
  wss_status=""
fi
if [[ "$wss_status" != "101" ]]; then
  echo "O endpoint WSS não completou o handshake TLS/WebSocket (curl=$wss_curl_status, status=${wss_status:-ausente})." >&2
  exit 1
fi

echo "Gateway público validado: HTTPS, HSTS, redirecionamento HTTP→HTTPS e handshake WSS com guarda de ticket."
