#!/usr/bin/env pwsh
# ============================================================================
# Script de Cobertura de Testes — Poker Engine (Rust)
# ============================================================================
# Gera relatório LCOV usando cargo-llvm-cov em container Docker oficial Rust.
# Workaround para Windows sem Visual Studio Build Tools.
#
# Uso:
#   .\scripts\coverage.ps1
#
# Saída:
#   - target/coverage/lcov.info         (formato LCOV)
# ============================================================================

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$RustDir = Join-Path $ProjectRoot "Motor-Rust"
$CoverageDir = Join-Path $RustDir "target\coverage"

Write-Host "==> Limpando cobertura anterior..." -ForegroundColor Cyan
Remove-Item -Recurse -Force $CoverageDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $CoverageDir | Out-Null

Write-Host "==> Rodando testes instrumentados em container Docker..." -ForegroundColor Cyan
$coverageCommand = @'
set -euo pipefail
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
PROPTEST_CASES="${PROPTEST_CASES:-256}" cargo llvm-cov --lib --lcov --output-path target/coverage/lcov.info -- --skip extreme_fuzz_tests --skip stress_tests --skip stress_integration_tests --skip card_fairness_tests
'@
docker run --rm `
    -v "${ProjectRoot}:/poker" `
    -w /poker/Motor-Rust `
    rust:1.88-bookworm `
    bash -c $coverageCommand

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "==> Cobertura gerada com sucesso!" -ForegroundColor Green
    Write-Host "    LCOV:  $CoverageDir\lcov.info"
} else {
    Write-Host "==> Falha ao gerar cobertura. Verifique se Docker Desktop está rodando." -ForegroundColor Red
    exit 1
}
