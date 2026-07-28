#!/usr/bin/env bash
# deploy.sh — Script de Deploy Automatizado da Plataforma de Poker (Produção & Nuvem)

set -e

echo "🚀 [Deploy] Iniciando verificação e deploy da Plataforma de Poker..."

# 1. Validação de Dependências Básicas
command -v docker >/dev/null 2>&1 || { echo "❌ Docker não encontrado. Instale o Docker para continuar."; exit 1; }
command -v docker-compose >/dev/null 2>&1 || command -v docker >/dev/null 2>&1 || { echo "❌ Docker Compose não encontrado."; exit 1; }

# 2. Configuração de Variáveis de Ambiente
if [ ! -f Infraestrutura-Docker/.env ]; then
    echo "⚠️ Arquivo Infraestrutura-Docker/.env não encontrado. Gerando a partir de .env.example..."
    cp Infraestrutura-Docker/.env.example Infraestrutura-Docker/.env
fi

# 3. Compilação e Subida dos Containers
cd Infraestrutura-Docker
echo "📦 Compilando e iniciando containers Docker (PostgreSQL, Redis, Kafka, API Axum, Caddy)..."
docker compose up -d --build || docker-compose up -d --build

# 4. Aguardar Inicialização do PostgreSQL e API (Health Check)
echo "⏳ Aguardando serviços responderem ao Health Check..."
max_retries=15
counter=0
# O certificado de desenvolvimento do Caddy é local; -k só vale para este probe.
until curl --fail --silent --show-error --insecure https://localhost/health > /dev/null || [ $counter -eq $max_retries ]; do
    counter=$((counter+1))
    echo "  Aguardando API ficar pronta... ($counter/$max_retries)"
    sleep 2
done

if [ $counter -eq $max_retries ]; then
    echo "⚠️ Aviso: API demorou a responder o health check HTTPS. Verifique os logs com: docker logs poker_api"
else
    echo "✅ API Axum respondeu com sucesso ao Health Check!"
fi

echo "============================================================"
echo "🎉 Deploy concluído com sucesso!"
echo "🌐 Frontend e API REST: https://localhost"
echo "💬 WebSockets: wss://localhost/ws/game/{table_id}"
echo "============================================================"
