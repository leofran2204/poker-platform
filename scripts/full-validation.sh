#!/usr/bin/env bash
# Execução manual autorizada da validação completa da plataforma.
# PIX real permanece adiado. Os contratos locais do ledger (sem payout externo)
# fazem parte desta rotina autorizada para impedir regressões financeiras.
set -euo pipefail

phase="${1:-all}"
case "$phase" in
  all|motor|api|frontend|gateway) ;;
  *) echo "Uso: FULL_VALIDATION_APPROVED=1 $0 [all|motor|api|frontend|gateway]" >&2; exit 2 ;;
esac

if [[ "${FULL_VALIDATION_APPROVED:-}" != "1" ]]; then
  echo "Esta rotina executa carga intensa. Exige autorização explícita: FULL_VALIDATION_APPROVED=1." >&2
  exit 2
fi

# O WSL instalado pelo rustup normalmente expõe o Cargo por este arquivo. Em
# terminais que já o adicionam ao PATH, o bloco é inócuo; nos demais, evita
# que uma execução manual autorizada morra antes de iniciar os testes.
if ! command -v cargo >/dev/null 2>&1 && [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo não foi encontrado. Instale Rust ou carregue o ambiente do rustup antes de executar." >&2
  exit 127
fi

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report_dir="${FULL_VALIDATION_REPORT_DIR:-$project_root/artifacts/full-validation}"
mkdir -p "$report_dir"
report_file="$report_dir/metrics-$(date -u +%Y%m%dT%H%M%SZ).tsv"
printf 'phase\tstatus\tduration_seconds\tworkload\n' > "$report_file"

record_metric() {
  local phase_name="$1"
  local status="$2"
  local duration_seconds="$3"
  local workload="$4"
  printf '%s\t%s\t%s\t%s\n' "$phase_name" "$status" "$duration_seconds" "$workload" >> "$report_file"
  echo "==> Métrica [$phase_name]: status=$status duração=${duration_seconds}s carga=$workload"
}

run_timed() {
  local phase_name="$1"
  local workload="$2"
  shift 2
  local started_at
  local duration_seconds
  local status="passed"
  started_at="$(date +%s)"

  if "$@"; then
    :
  else
    status="failed"
  fi

  duration_seconds=$(( $(date +%s) - started_at ))
  record_metric "$phase_name" "$status" "$duration_seconds" "$workload"
  [[ "$status" == "passed" ]]
}

if grep -qi microsoft /proc/version 2>/dev/null; then
  # Fases disparadas em paralelo não podem disputar o mesmo lock de compilação.
  # `all` continua usando um diretório único para sua execução sequencial.
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/poker-project-target/full-validation-$phase}"
fi

run_motor() {
  echo "==> Motor: rotina + 79 cenários de carga"
  (cd "$project_root/Motor-Rust" && cargo test --lib --features massive-tests -- --nocapture) || return 1
}

run_api() {
  # Valor padrão compatível com o docker-compose local. Uma URL já exportada
  # pelo operador (por exemplo, em staging) sempre tem precedência.
  export DATABASE_URL="${DATABASE_URL:-postgres://user:password@localhost:5433/poker_db}"

  echo "==> API HTTPS: contratos e segurança"
  (cd "$project_root/API-Axum" && cargo test --lib --bin poker-api --test api_tests --test payments_tests --test red_team_simulation_tests -- --nocapture) || return 1

  echo "==> API HTTPS: contratos funcionais PostgreSQL"
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test api_tests -- --ignored --nocapture) || return 1

  echo "==> API HTTPS: contratos financeiros PostgreSQL"
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test payments_tests -- --ignored --nocapture) || return 1

  echo "==> API HTTPS: contrato de limite compartilhado Redis"
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test rate_limit_tests -- --ignored --nocapture) || return 1

  echo "==> API HTTPS: 10 cenários de fuzz (2.000 casos por cenário)"
  (cd "$project_root/API-Axum" && PROPTEST_CASES="${API_FUZZ_CASES:-2000}" cargo test --features full-validation --test api_fuzz_tests -- --nocapture) || return 1

  echo "==> API: WebSocket, concorrência, jitter e desconexão"
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test ws_stress_tests -- --nocapture) || return 1
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test ws_network_jitter_tests -- --nocapture) || return 1
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test concurrency_ws_tests -- --nocapture) || return 1
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test actor_disconnect_stress_tests -- --nocapture) || return 1

  echo "==> API: persistência PostgreSQL concorrente"
  (cd "$project_root/API-Axum" && cargo test --features full-validation --test db_pool_stress_tests -- --ignored --nocapture) || return 1
}

run_frontend() {
  echo "==> Frontend: rotina, 10 fuzzes visuais e stress de estado"
  # O projeto fixa a toolchain GNU do Windows para desenvolvimento local.
  # No Linux/WSL, selecionamos explicitamente a toolchain nativa e o target
  # Linux para executar os testes host.
  (cd "$project_root/Frontend-Dioxus" && cargo +stable test --lib --target x86_64-unknown-linux-gnu --features full-validation -- --nocapture) || return 1
}

run_gateway() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "Docker é necessário para a fase gateway HTTPS/WSS." >&2
    return 127
  fi

  echo "==> Gateway: reconstruindo a API e o Caddy para a verificação E2E"
  (
    cd "$project_root"
    docker compose -f Infraestrutura-Docker/docker-compose.yml up -d --build poker_api poker_frontend
  ) || return 1

  echo "==> Gateway: HTTPS, HSTS, redirecionamento e handshake WSS"
  (
    cd "$project_root"
    local_gateway_url="${PUBLIC_GATEWAY_URL:-https://localhost}"
    # O certificado emitido pelo Caddy em localhost é deliberadamente local.
    # Para qualquer outra origem, a validação da cadeia TLS é obrigatória
    # salvo escolha explícita do operador (variável já definida).
    local_insecure_cert="${PUBLIC_GATEWAY_INSECURE_LOCAL_CERT:-}"
    if [[ -z "$local_insecure_cert" && "$local_gateway_url" == "https://localhost" ]]; then
      local_insecure_cert=1
    fi
    PUBLIC_GATEWAY_URL="$local_gateway_url" \
      PUBLIC_GATEWAY_INSECURE_LOCAL_CERT="$local_insecure_cert" \
      bash scripts/verify-public-https.sh
  ) || return 1
}

if [[ "$phase" == "all" || "$phase" == "motor" ]]; then
  run_timed \
    "motor" \
    "1814 rotina + 79 carga; Monte Carlo, CSPRNG, fairness e invariantes" \
    run_motor
fi

if [[ "$phase" == "all" || "$phase" == "api" ]]; then
  run_timed \
    "api" \
    "10 fuzzes de API x 2000 = 20000 entradas; WebSocket = 1000800 mensagens; testes funcionais" \
    run_api
fi

if [[ "$phase" == "all" || "$phase" == "frontend" ]]; then
  run_timed \
    "frontend" \
    "76 funcionais + 10 fuzzes x 200000 = 2000000 entradas + 2 stresses" \
    run_frontend
fi

if [[ "$phase" == "all" || "$phase" == "gateway" ]]; then
  run_timed \
    "gateway" \
    "Caddy local: API HTTPS, HSTS, redirecionamento HTTP→HTTPS e handshake WSS" \
    run_gateway
fi

echo "==> Validação completa concluída."
echo "==> Relatório de métricas: $report_file"
