# deploy.ps1 — Script de inicialização local via Docker Compose

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$infraDir = Join-Path $projectRoot "Infraestrutura-Docker"

Write-Host "🚀 [Deploy] Iniciando deploy da Plataforma de Poker..." -ForegroundColor Green

# 1. Copia .env.example para .env se não existir
if (-not (Test-Path (Join-Path $infraDir ".env"))) {
    Write-Host "⚠️ Arquivo .env não encontrado. Copiando .env.example..." -ForegroundColor Yellow
    Copy-Item (Join-Path $infraDir ".env.example") (Join-Path $infraDir ".env")
}

# 2. Build e Inicialização dos Containers
Push-Location $infraDir
Write-Host "📦 Compilando containers com Docker Compose..." -ForegroundColor Cyan
docker compose up -d --build

# 3. Health Check
Write-Host "⏳ Aguardando serviços responderem ao Health Check..." -ForegroundColor Yellow
$maxRetries = 10
$counter = 0
$healthy = $false

while ($counter -lt $maxRetries -and -not $healthy) {
    $counter++
    try {
        $res = Invoke-WebRequest -Uri "http://127.0.0.1:3000/health" -Method Get -ErrorAction Stop
        if ($res.StatusCode -eq 200) {
            $healthy = $true
        }
    } catch {
        Start-Sleep -Seconds 2
    }
}

if ($healthy) {
    Write-Host "✅ API Axum respondeu com sucesso ao Health Check!" -ForegroundColor Green
} else {
    Write-Host "⚠️ API demorou a responder ao health check. Verifique os logs com: docker compose logs poker_api" -ForegroundColor Yellow
}

Pop-Location

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "🎉 Deploy concluído com sucesso!" -ForegroundColor Green
Write-Host "🌐 Frontend: https://localhost | API local: http://127.0.0.1:3000" -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan
