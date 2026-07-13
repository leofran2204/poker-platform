#!/usr/bin/env pwsh
# ============================================================================
# Script de Cobertura de Testes — Poker Engine (Rust)
# ============================================================================
# Gera relatório de cobertura usando grcov em container Docker oficial Rust.
# Workaround para Windows sem Visual Studio Build Tools.
#
# Uso:
#   .\scripts\coverage.ps1
#
# Saída:
#   - target/coverage/html/index.html   (relatório HTML navegável)
#   - target/coverage/cobertura.xml     (formato Cobertura, para CI)
#   - target/coverage/lcov.info         (formato LCOV)
# ============================================================================

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$RustDir = Join-Path $ProjectRoot "08-Motor-Rust"
$CoverageDir = Join-Path $RustDir "target\coverage"
$ScriptSh = Join-Path $PSScriptRoot "coverage.sh"

Write-Host "==> Limpando cobertura anterior..." -ForegroundColor Cyan
Remove-Item -Recurse -Force $CoverageDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $CoverageDir | Out-Null

Write-Host "==> Rodando testes instrumentados em container Docker..." -ForegroundColor Cyan
docker run --rm `
    -v "${ProjectRoot}:/poker" `
    -w /poker/08-Motor-Rust `
    rust:1.88-bookworm `
    bash /poker/scripts/coverage.sh

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "==> Cobertura gerada com sucesso!" -ForegroundColor Green
    Write-Host "    HTML:  $CoverageDir\html\index.html"
    Write-Host "    XML:   $CoverageDir\cobertura.xml"
    Write-Host "    LCOV:  $CoverageDir\lcov.info"
    Write-Host ""
    Write-Host "Para abrir o relatório HTML:"
    Write-Host "    start $CoverageDir\html\index.html"
} else {
    Write-Host "==> Falha ao gerar cobertura. Verifique se Docker Desktop está rodando." -ForegroundColor Red
    exit 1
}
