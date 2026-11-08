#!/usr/bin/env bash
# Publica API + frontend na VPS a partir das imagens que o CI montou, assinou
# e atestou (workflow supply-chain.yml), sem compilar nada no servidor.
#
# Uso (na VPS, como root, a partir de qualquer diretório):
#   bash /opt/poker-platform/scripts/vps-release-attested.sh <commit-40-hex> [--verify-only]
#
#   --verify-only  baixa as imagens do commit e confere assinatura/revisão;
#                  não para, não troca e não faz backup (seguro a qualquer hora).
#
# Pré-requisitos: cosign >= 3 no PATH (versão fixa e checksum conferido, ver
# DEPLOYMENT_VALIDATION.md), Rust CI e Container Supply Chain verdes no commit.
#
# Garantias, na ordem:
#   1. checkout em fast-forward; o commit das imagens é ancestral do HEAD e
#      não há mudança de código/infra entre ele e o HEAD;
#   2. cada imagem é assinada (Sigstore keyless) pelo workflow supply-chain.yml
#      deste repositório, na master e NESTE commit — senão aborta sem tocar em nada;
#   3. label org.opencontainers.image.revision igual ao commit;
#   4. janela ociosa (zero assentos, torneios ativos/inscritos, transações
#      pendentes), conferida antes e depois de parar a API;
#   5. dois dumps validados (o segundo com a API parada) e imagens anteriores
#      retidas como before-<commit12>; rollback automático se a troca falhar;
#   6. serviços saudáveis, migrations, boot sem ERROR, health HTTPS, .env intacto.

set -euo pipefail

SHA="${1:-}"
MODE="${2:-}"
[[ "$SHA" =~ ^[0-9a-f]{40}$ ]] || { echo "uso: $0 <commit-40-hex> [--verify-only]" >&2; exit 2; }
[[ -z "$MODE" || "$MODE" == "--verify-only" ]] || { echo "opção desconhecida: $MODE" >&2; exit 2; }

POKER_ROOT="${POKER_ROOT:-/opt/poker-platform}"
REPO="leofran2204/poker-platform"
REGISTRY="ghcr.io/leofran2204"
IDENTITY="https://github.com/${REPO}/.github/workflows/supply-chain.yml@refs/heads/master"
ISSUER="https://token.actions.githubusercontent.com"
DOMAIN="${DEPLOY_DOMAIN:-zerotiltpoker.net}"
SHORT="${SHA:0:12}"
B="${POKER_ROOT}/backups/release-${SHORT}"

ts(){ date -u +%Y-%m-%dT%H:%M:%SZ; }
log(){ echo "[$(ts)] $*"; }
fail(){ log "FAIL: $*"; exit 1; }
psqlc(){ docker exec poker_postgres sh -c 'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tA -c "$0"' "$1"; }
busy(){ psqlc "select (select count(*) from cash_game_seats where status='ACTIVE')
  + (select count(*) from tournaments where status in ('running','paused'))
  + (select count(*) from tournament_players tp join tournaments t on t.id=tp.tournament_id
       where t.status in ('registering','running','paused'))
  + (select count(*) from wallet_transactions where status='PENDING')"; }

command -v cosign >/dev/null || fail "cosign ausente (instalar a versão fixa; ver DEPLOYMENT_VALIDATION.md)"

cd "$POKER_ROOT"
log "git pull --ff-only origin master"
git pull -q --ff-only origin master
git merge-base --is-ancestor "$SHA" HEAD || fail "commit $SHA não é ancestral do HEAD $(git rev-parse HEAD)"
# Documentação (*.md) pode avançar sem novas imagens; código, Compose e Caddyfile não.
git diff --quiet "$SHA" HEAD -- API-Axum Motor-Rust Frontend-Web Infraestrutura-Docker Cargo.toml Cargo.lock ':(exclude)*.md' \
  || fail "há mudanças de código/infra entre $SHORT e o HEAD; publique as imagens do HEAD"

declare -A REF
for comp in api web; do
  tag="${REGISTRY}/poker-${comp}:sha-${SHA:0:7}"
  log "pull ${tag}"
  docker pull -q "$tag" >/dev/null
  digest=$(docker image inspect "$tag" --format '{{range .RepoDigests}}{{println .}}{{end}}' \
    | grep "^${REGISTRY}/poker-${comp}@" | head -1)
  [[ -n "$digest" ]] || fail "digest de ${tag} não encontrado"
  log "verificando assinatura de ${digest}"
  cosign verify "$digest" \
    --certificate-identity "$IDENTITY" --certificate-oidc-issuer "$ISSUER" \
    --certificate-github-workflow-repository "$REPO" \
    --certificate-github-workflow-ref refs/heads/master \
    --certificate-github-workflow-sha "$SHA" >/dev/null 2>&1 \
    || fail "assinatura de ${digest} NÃO confere com supply-chain.yml/master/${SHORT}"
  rev=$(docker image inspect "$digest" --format '{{index .Config.Labels "org.opencontainers.image.revision"}}')
  [[ "$rev" == "$SHA" ]] || fail "label de revisão de ${comp} = ${rev}"
  REF[$comp]="$digest"
done
log "assinaturas e revisões conferidas: api=${REF[api]} web=${REF[web]}"

if [[ "$MODE" == "--verify-only" ]]; then
  log "VERIFY_OK (nada foi trocado)"
  exit 0
fi

mkdir -p -m 700 "$B"
ENV_HASH_BEFORE=$(sha256sum Infraestrutura-Docker/.env | cut -d' ' -f1)
[[ "$(busy)" == "0" ]] || fail "janela ocupada (jogadores, torneios ou transações pendentes)"

docker tag infraestrutura-docker-poker_api:latest "poker-api:before-${SHORT}"
docker tag infraestrutura-docker-poker_frontend:latest "poker-frontend:before-${SHORT}"
log "backup 1"
docker exec poker_postgres sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc' > "$B/postgres.dump"
docker exec -i poker_postgres pg_restore -l < "$B/postgres.dump" > "$B/postgres.toc"

cd Infraestrutura-Docker
rollback(){
  log "ROLLBACK para before-${SHORT}"
  docker tag "poker-api:before-${SHORT}" infraestrutura-docker-poker_api:latest
  docker tag "poker-frontend:before-${SHORT}" infraestrutura-docker-poker_frontend:latest
  docker compose up -d --no-build --no-deps --wait --wait-timeout 180 poker_api poker_frontend || true
  docker compose ps
  log "ROLLBACK_DONE"
}
trap rollback ERR

log "parando API (bloqueia mutações)"
docker compose stop poker_api >/dev/null
[[ "$(busy)" == "0" ]] || { log "janela ocupada após parar a API"; false; }
log "backup 2 (API parada)"
docker exec poker_postgres sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc' > "$B/postgres-before-switch.dump"
docker exec -i poker_postgres pg_restore -l < "$B/postgres-before-switch.dump" > "$B/postgres-before-switch.toc"
( cd "$B" && sha256sum postgres*.dump > SHA256SUMS )

docker tag "${REF[api]}" infraestrutura-docker-poker_api:latest
docker tag "${REF[web]}" infraestrutura-docker-poker_frontend:latest
log "subindo API e frontend assinados"
docker compose up -d --no-build --no-deps --wait --wait-timeout 180 poker_api poker_frontend
SWITCH_AT=$(ts)

sleep 5
docker compose ps --format '{{.Name}} {{.Status}}'
[[ "$(docker inspect poker_api --format '{{.Image}}')" == "${REF[api]#*@}" ]] || { log "poker_api não roda o digest assinado"; false; }
[[ "$(docker inspect poker_frontend --format '{{.Image}}')" == "${REF[web]#*@}" ]] || { log "poker_frontend não roda o digest assinado"; false; }
curl -fsS --max-time 10 http://127.0.0.1/caddy-health | grep -q OK
curl -fsS --max-time 10 -o /dev/null --resolve "${DOMAIN}:443:127.0.0.1" "https://${DOMAIN}/health"
trap - ERR

log "migrations: $(psqlc "select count(*)||' ok, max '||max(version) from _sqlx_migrations where success")"
log "boot: $(docker logs poker_api 2>&1 | grep -c 'Migrations applied' || true)x 'Migrations applied', $(docker logs poker_api 2>&1 | grep -c ' ERROR ' || true) linhas ERROR"
[[ "$ENV_HASH_BEFORE" == "$(sha256sum .env | cut -d' ' -f1)" ]] && log ".env preservado (hash igual)"
printf '{"commit":"%s","api":"%s","web":"%s","switched_at":"%s","backup":"%s","signature":"cosign verified: %s"}\n' \
  "$SHA" "${REF[api]}" "${REF[web]}" "$SWITCH_AT" "$B" "$IDENTITY" > "$B/published.json"
log "DEPLOY_OK switched_at=${SWITCH_AT}"
