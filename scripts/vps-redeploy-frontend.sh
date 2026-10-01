#!/usr/bin/env bash
# Redeploy do frontend TypeScript e, opcionalmente, da API na VPS.
# Uso (na VPS, como root ou user com docker):
#   cd /opt/poker-platform && bash scripts/vps-redeploy-frontend.sh
# Background:
#   nohup bash scripts/vps-redeploy-frontend.sh > /tmp/redeploy-frontend.log 2>&1 &
#   tail -f /tmp/redeploy-frontend.log
#
# Env opcional:
#   POKER_ROOT=/opt/poker-platform
#   REBUILD_API=1   # também rebuilda poker_api (lento / mais RAM)

set -euo pipefail

POKER_ROOT="${POKER_ROOT:-/opt/poker-platform}"
COMPOSE_DIR="${POKER_ROOT}/Infraestrutura-Docker"
LOG_TAG="[vps-redeploy-frontend]"

echo "${LOG_TAG} $(date -u +%Y-%m-%dT%H:%M:%SZ) START root=${POKER_ROOT}"

if [[ ! -d "${POKER_ROOT}/.git" ]]; then
  echo "${LOG_TAG} FAIL: ${POKER_ROOT} não é um clone git" >&2
  exit 1
fi

if [[ ! -f "${COMPOSE_DIR}/docker-compose.yml" ]]; then
  echo "${LOG_TAG} FAIL: compose não encontrado em ${COMPOSE_DIR}" >&2
  exit 1
fi

cd "${POKER_ROOT}"
echo "${LOG_TAG} git pull --ff-only origin master"
git pull --ff-only origin master

cd "${COMPOSE_DIR}"
export DOCKER_BUILDKIT=1

if [[ ! -f .env ]]; then
  echo "${LOG_TAG} FAIL: ${COMPOSE_DIR}/.env ausente" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "${LOG_TAG} FAIL: curl é necessário para validar API e proxy" >&2
  exit 1
fi

# SKIP_BUILD=1 → só recria containers (útil após fix de Caddyfile montado em volume)
if [[ "${SKIP_BUILD:-0}" == "1" ]]; then
  echo "${LOG_TAG} SKIP_BUILD=1 — sem docker compose build"
elif [[ "${REBUILD_API:-0}" == "1" ]]; then
  echo "${LOG_TAG} docker compose build poker_api poker_frontend"
  docker compose build poker_api poker_frontend
else
  echo "${LOG_TAG} docker compose build poker_frontend (API reutilizada se já existir)"
  docker compose build poker_frontend
fi

# Volumes existentes podem ter sido criados pelo Caddy antigo (root).
# Usa apenas os dois volumes persistentes definidos no Compose; preserva os
# certificados e executa o servidor final como 10001, sem privilégios extras.
echo "${LOG_TAG} preparando ownership dos volumes Caddy para uid 10001"
docker compose run --rm --no-deps --user 0:0 --cap-add CHOWN --cap-add DAC_OVERRIDE \
  --entrypoint /bin/sh poker_frontend -c 'chown -R 10001:10001 /data /config'

echo "${LOG_TAG} docker compose up -d --wait --wait-timeout 180"
docker compose up -d --wait --wait-timeout 180
# Força a recarga de configurações montadas (ex.: Caddyfile).
docker compose up -d --force-recreate --wait --wait-timeout 180 poker_frontend

echo "${LOG_TAG} docker compose ps"
docker compose ps

echo "${LOG_TAG} health (local)"
HEALTH_OK=0
DOMAIN="${DEPLOY_DOMAIN:-zerotiltpoker.net}"
for attempt in {1..12}; do
  if curl --max-time 10 -fsS http://127.0.0.1/caddy-health | grep -q OK &&
    curl --max-time 10 -fsS -o /dev/null --resolve "${DOMAIN}:443:127.0.0.1" "https://${DOMAIN}/health"; then
    echo "${LOG_TAG} caddy-health e API via HTTPS OK"
    HEALTH_OK=1
    break
  fi
  echo "${LOG_TAG} aguardando saúde do proxy e API (${attempt}/12)"
  sleep 5
done

docker compose ps || true

if [[ "${HEALTH_OK}" -eq 1 ]]; then
  echo "${LOG_TAG} $(date -u +%Y-%m-%dT%H:%M:%SZ) DEPLOY_OK"
  echo "${LOG_TAG} Abra https://${DOMAIN} (hard refresh Ctrl+F5)"
  exit 0
fi

echo "${LOG_TAG} $(date -u +%Y-%m-%dT%H:%M:%SZ) DEPLOY_FAIL healthcheck do proxy/API falhou — conferir containers e logs"
exit 1
