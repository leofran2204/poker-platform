#!/bin/bash
# ============================================================================
# Script de Cobertura de Testes — Poker Engine (Rust)
# Roda dentro do container Docker rust:1.83-bookworm
# ============================================================================
set -e

echo "==> Instalando dependências de sistema..."
apt-get update -qq
apt-get install -y -qq libssl-dev pkg-config zlib1g-dev cmake llvm > /dev/null

# Garante llvm-profdata no PATH (vem do pacote llvm)
export PATH="/usr/lib/llvm-14/bin:/usr/bin:$PATH"
LLVM_PROFDATA=$(which llvm-profdata || echo "")
if [ -z "$LLVM_PROFDATA" ]; then
    echo "ERRO: llvm-profdata não encontrado. Instalando llvm-tools via rustup..."
    rustup component add llvm-tools-preview
    LLVM_PROFDATA=$(which llvm-profdata)
fi
echo "    llvm-profdata: $LLVM_PROFDATA"

echo "==> Instalando grcov..."
cargo install grcov --version 0.10.7 --locked --quiet

echo "==> Rodando testes com instrumentação de cobertura..."
export RUSTFLAGS='-C instrument-coverage'
export LLVM_PROFILE_FILE='target/coverage/raw-%p-%m.profraw'
cargo test --lib --quiet

echo "==> Gerando relatórios de cobertura..."
mkdir -p target/coverage

# Detecta caminho do llvm-profdata (vem com pacote llvm)
LLVM_PROFDATA=$(which llvm-profdata || echo "/usr/bin/llvm-profdata")
echo "    llvm-profdata: $LLVM_PROFDATA"

grcov . \
    --binary-path ./target/debug/ \
    -s . \
    --llvm-path /usr/lib/llvm-14 \
    -t html --branch --ignore-not-existing \
    -o ./target/coverage/html

grcov . \
    --binary-path ./target/debug/ \
    -s . \
    --llvm-path /usr/lib/llvm-14 \
    -t cobertura --ignore-not-existing \
    -o ./target/coverage/cobertura.xml

grcov . \
    --binary-path ./target/debug/ \
    -s . \
    --llvm-path /usr/lib/llvm-14 \
    -t lcov --ignore-not-existing \
    -o ./target/coverage/lcov.info

echo ""
echo "=== RESUMO DE COBERTURA ==="
grcov . \
    --binary-path ./target/debug/ \
    -s . \
    --llvm-path /usr/lib/llvm-14 \
    -t markdown --ignore-not-existing | head -60
