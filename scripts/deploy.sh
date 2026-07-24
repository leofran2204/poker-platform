#!/usr/bin/env bash
# deploy.sh — Script de Deploy Automatizado da Plataforma de Poker

set -e

echo "🚀 Iniciando deploy da Plataforma de Poker..."

# 1. Copia .env.example para .env se não existir
if [ ! -f Infraestrutura-Docker/.env ]; then
    echo "⚠️ Arquivo .env não encontrado em Infraestrutura-Docker/. Copiando .env.example..."
    cp Infraestrutura-Docker/.env.example Infraestrutura-Docker/.env
fi

# 2. Build e Inicialização dos Containers
cd Infraestrutura-Docker
echo "📦 Compilando containers com Docker Compose..."
docker-compose up -d --build

echo "✅ Deploy concluído com sucesso!"
echo "🌐 API escutando na porta 3000 | Frontend Web no Caddy"
