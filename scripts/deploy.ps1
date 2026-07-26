# deploy.ps1 — Script de Deploy Automatizado para Windows

Write-Host "🚀 [Deploy] Iniciando deploy da Plataforma de Poker..." -ForegroundColor Green

# 1. Copia .env.example para .env se não existir
if (-not (Test-Path "Infraestrutura-Docker\.env")) {
    Write-Host "⚠️ Arquivo .env não encontrado. Copiando .env.example..." -ForegroundColor Yellow
    Copy-Item "Infraestrutura-Docker\.env.example" "Infraestrutura-Docker\.env"
}

# 2. Build e Inicialização dos Containers
Set-Location "Infraestrutura-Docker"
Write-Host "📦 Compilando containers com Docker Compose..." -ForegroundColor Cyan
docker-compose up -d --build

# 3. Health Check
Write-Host "⏳ Aguardando serviços responderem ao Health Check..." -ForegroundColor Yellow
$maxRetries = 10
$counter = 0
$healthy = $false

while ($counter -lt $maxRetries -and -not $healthy) {
    $counter++
    try {
        $res = Invoke-RestMethod -Uri "http://localhost:3000/api/health" -Method Get -ErrorAction Stop
        if ($res.status -eq "ok" -or $res) {
            $healthy = $true
        }
    } catch {
        Start-Sleep -Seconds 2
    }
}

if ($healthy) {
    Write-Host "✅ API Axum respondeu com sucesso ao Health Check!" -ForegroundColor Green
} else {
    Write-Host "⚠️ API demorou a responder ao health check. Verifique os logs com: docker logs poker_api" -ForegroundColor Yellow
}

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "🎉 Deploy concluído com sucesso!" -ForegroundColor Green
Write-Host "🌐 API REST: http://localhost:3000 | HTTPS: https://localhost" -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan
