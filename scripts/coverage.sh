#!/bin/bash
set -e

echo "==> Instalando dependências de sistema..."
apt-get update -qq
apt-get install -y -qq libssl-dev pkg-config zlib1g-dev cmake > /dev/null

echo "==> Instalando llvm-tools via rustup..."
rustup component add llvm-tools-preview

echo "==> Instalando grcov..."
cargo install grcov --version 0.10.7 --locked --quiet

echo "==> Rodando testes com instrumentação de cobertura..."
export RUSTFLAGS='-C instrument-coverage'
export LLVM_PROFILE_FILE='target/coverage/raw-%p-%m.profraw'
cargo test --lib --quiet

echo "==> Gerando relatórios de cobertura..."
mkdir -p target/coverage

grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./target/coverage/html
grcov . --binary-path ./target/debug/ -s . -t cobertura --ignore-not-existing -o ./target/coverage/cobertura.xml
grcov . --binary-path ./target/debug/ -s . -t lcov --ignore-not-existing -o ./target/coverage/lcov.info

echo ""
echo "=== RESUMO DE COBERTURA ==="
grcov . --binary-path ./target/debug/ -s . -t markdown --ignore-not-existing | head -60
