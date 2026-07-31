#!/usr/bin/env bash
# Build completo WASM release + wasm-bindgen → Frontend-Dioxus/dist
# Funciona em WSL/Linux; no Windows prefira Docker (Frontend-Dioxus/Dockerfile).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONT="$PROJECT_ROOT/Frontend-Dioxus"

export PATH="${HOME}/.cargo/bin:${PATH}"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$FRONT/target}"

cd "$FRONT"

# Toolchain file do Windows (gnu) quebra build wasm em WSL — isola temporariamente
TOOLCHAIN_BAK=""
if [[ -f rust-toolchain.toml ]]; then
  TOOLCHAIN_BAK="rust-toolchain.toml.bak.$$"
  mv -f rust-toolchain.toml "$TOOLCHAIN_BAK"
  trap 'if [[ -n "$TOOLCHAIN_BAK" && -f "$TOOLCHAIN_BAK" ]]; then mv -f "$TOOLCHAIN_BAK" rust-toolchain.toml; fi' EXIT
fi

echo "==> cargo build wasm32 release (dioxus web)"
cargo build --release --target wasm32-unknown-unknown --features web

WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/poker-frontend.wasm"
test -f "$WASM"

bash "$SCRIPT_DIR/build-frontend-dist.sh"
