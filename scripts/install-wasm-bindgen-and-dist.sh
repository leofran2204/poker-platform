#!/usr/bin/env bash
# Instala wasm-bindgen-cli pinado e gera dist (requer .wasm já buildado ou use rebuild-frontend-dist.sh).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="${HOME}/.cargo/bin:${PATH}"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"

VERSION="${WASM_BINDGEN_CLI_VERSION:-0.2.126}"
echo "Installing wasm-bindgen-cli ${VERSION}..."
cargo install -f wasm-bindgen-cli --version "$VERSION"
wasm-bindgen --version

bash "$SCRIPT_DIR/build-frontend-dist.sh"
