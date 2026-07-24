# deploy.ps1 — Script de Deploy Automatizado para Windows

Write-Host "🚀 Iniciando deploy da Plataforma de Poker..." -ForegroundColor Green

# 1. Copia .env.example para .env se não existir
if (-not (Test-Path "Infraestrutura-Docker\.env")) {
    Write-Host "⚠️ Arquivo .env não encontrado. Copiando .env.example..." -ForegroundColor Yellow
    Copy-Item "Infraestrutura-Docker\.env.example" "Infraestrutura-Docker\.env"
}

# 2. Build e Inicialização dos Containers
Set-Location "Infraestrutura-Docker"
Write-Host "📦 Compilando containers com Docker Compose..." -ForegroundColor Cyan
docker-compose up -d --build

Write-Host "✅ Deploy concluído com sucesso!" -ForegroundColor Green
Write-Host "🌐 API escutando na porta 3000 | Frontend Web no Caddy" -ForegroundColor Yellow
