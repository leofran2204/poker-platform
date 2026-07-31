#!/usr/bin/env bash
# Gera Frontend-Dioxus/dist a partir de um .wasm já compilado (release wasm32).
# Uso típico (após cargo build --release --target wasm32-unknown-unknown):
#   CARGO_TARGET_DIR=... ./scripts/build-frontend-dist.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONT="$PROJECT_ROOT/Frontend-Dioxus"

export PATH="${HOME}/.cargo/bin:${PATH}"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"

cd "$FRONT"

DEFAULT_WASM_DIR="${CARGO_TARGET_DIR:-$FRONT/target}/wasm32-unknown-unknown/release"
WASM="${WASM_PATH:-$DEFAULT_WASM_DIR/poker-frontend.wasm}"

if [[ ! -f "$WASM" ]]; then
  echo "WASM not found at $WASM" >&2
  echo "Build first, e.g.:" >&2
  echo "  cd Frontend-Dioxus && cargo build --release --target wasm32-unknown-unknown" >&2
  exit 1
fi

rm -rf dist
mkdir -p dist
wasm-bindgen --target web --out-dir dist --no-typescript "$WASM"
cp assets/index.html dist/index.html

python3 - <<'PY'
from pathlib import Path
import re

dist = Path("dist")
html_path = dist / "index.html"
html = html_path.read_text(encoding="utf-8")

# Prefer hyphenated bindgen output (crate name poker-frontend); fall back to underscore.
js = None
for candidate in ("poker-frontend.js", "poker_frontend.js"):
    if (dist / candidate).is_file():
        js = candidate
        break
if js is None:
    raise SystemExit("No poker-frontend.js / poker_frontend.js in dist after wasm-bindgen")

wasm = js.replace(".js", "_bg.wasm")
if not (dist / wasm).is_file():
    # some bindgen versions keep hyphen in bg name
    alt = js.replace(".js", "_bg.wasm")
    wasm = alt if (dist / alt).is_file() else wasm

snippet = f'''
<div id="app-error" style="display:none;color:#ff6b6b;background:#1a1a1a;padding:16px;font-family:monospace;white-space:pre-wrap;"></div>
<script type="module">
  import init from "./{js}";
  const errBox = document.getElementById("app-error");
  try {{
    await init({{ module_or_path: "./{wasm}" }});
    console.log("poker-frontend wasm started");
  }} catch (e) {{
    console.error(e);
    errBox.style.display = "block";
    errBox.textContent = String(e && e.stack ? e.stack : e);
  }}
</script>
'''

if "poker-frontend.js" not in html and "poker_frontend.js" not in html:
    if "</body>" in html:
        html = html.replace("</body>", snippet + "\n</body>")
    else:
        html = html + snippet
    html_path.write_text(html, encoding="utf-8")
print(f"dist ready (entry {js})")
PY

ls -la dist
