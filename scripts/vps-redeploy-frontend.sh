#!/usr/bin/env bash
# Redeploy do frontend TypeScript (Full Tilt) na VPS.
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
echo "${LOG_TAG} git pull origin master"
git pull origin master

cd "${COMPOSE_DIR}"
export DOCKER_BUILDKIT=1

if [[ ! -f .env ]]; then
  echo "${LOG_TAG} WARN: ${COMPOSE_DIR}/.env ausente — compose pode falhar"
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

echo "${LOG_TAG} docker compose up -d --force-recreate poker_frontend"
docker compose up -d --force-recreate poker_frontend
# Garante postgres/redis/api se estiverem parados
docker compose up -d

echo "${LOG_TAG} docker compose ps"
docker compose ps || true

echo "${LOG_TAG} health (local)"
sleep 5
HEALTH_OK=0
if command -v curl >/dev/null 2>&1; then
  if curl -fsS http://127.0.0.1/caddy-health | grep -q OK; then
    echo "${LOG_TAG} caddy-health OK"
    HEALTH_OK=1
  else
    echo "${LOG_TAG} caddy-health FAIL (curl)"
  fi
  curl -fsS -o /dev/null -w "api-via-proxy /health %{http_code}\n" http://127.0.0.1/health || true
else
  if wget -qO- http://127.0.0.1/caddy-health 2>/dev/null | grep -q OK; then
    echo "${LOG_TAG} caddy-health OK"
    HEALTH_OK=1
  else
    echo "${LOG_TAG} caddy-health FAIL (wget)"
  fi
fi

docker compose ps || true

if [[ "${HEALTH_OK}" -eq 1 ]]; then
  echo "${LOG_TAG} $(date -u +%Y-%m-%dT%H:%M:%SZ) DEPLOY_OK"
  echo "${LOG_TAG} Abra https://zerotiltpoker.net (hard refresh Ctrl+F5)"
  exit 0
fi

echo "${LOG_TAG} $(date -u +%Y-%m-%dT%H:%M:%SZ) DEPLOY_WARN healthcheck ainda falhou — ver: docker logs poker_frontend"
exit 1
