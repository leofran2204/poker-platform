# Stage 1: Build the frontend static files
FROM rust:1.97.1-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97 AS builder

WORKDIR /app

# Adiciona o target wasm32 para compilação WebAssembly
RUN rustup target add wasm32-unknown-unknown

# Instala o wasm-bindgen-cli diretamente do crates.io com o lockfile publicado
RUN cargo install --locked wasm-bindgen-cli --version 0.2.126

# Copia os arquivos necessários do frontend
COPY Frontend-Dioxus ./Frontend-Dioxus

WORKDIR /app/Frontend-Dioxus

# Remove a especificação de toolchain do Windows
RUN rm -f rust-toolchain.toml rust-toolchain

# Compila o projeto para WebAssembly (feature web = default do Cargo.toml)
RUN CARGO_BUILD_JOBS=1 cargo build --release --locked --target wasm32-unknown-unknown --features web

# Gera bindings JS/WASM; nomes dependem do crate (hífen vs underscore).
# boot.js: await init + erro visível (evita tela verde silenciosa).
RUN mkdir dist && \
    wasm-bindgen --target web --out-dir dist --no-typescript target/wasm32-unknown-unknown/release/poker-frontend.wasm && \
    cp assets/index.html dist/index.html && \
    JS=""; \
    if [ -f dist/poker-frontend.js ]; then JS=poker-frontend.js; \
    elif [ -f dist/poker_frontend.js ]; then JS=poker_frontend.js; \
    else echo "wasm-bindgen JS entry not found" >&2; ls -la dist; exit 1; fi && \
    WASM_BG="${JS%.js}_bg.wasm" && \
    printf '%s\n' \
      "import init from \"./${JS}\";" \
      "async function boot() {" \
      "  try {" \
      "    await init({ module_or_path: new URL(\"./${WASM_BG}\", import.meta.url) });" \
      "  } catch (e) {" \
      "    console.error(e);" \
      "    const el = document.getElementById(\"main\");" \
      "    if (el) {" \
      "      el.innerHTML = \"<pre style='color:#ffcc66;padding:1.5rem;white-space:pre-wrap;font:14px monospace'>Falha ao carregar o app WASM:\\n\"" \
      "        + String(e) + (e && e.stack ? \"\\n\" + e.stack : \"\") + \"</pre>\";" \
      "    }" \
      "  }" \
      "}" \
      "boot();" \
      > dist/boot.js && \
    sed -i 's|</body>|<script type="module" src="./boot.js"></script></body>|' dist/index.html

# Stage 2: Servidor estático leve usando Caddy
FROM caddy:2.10.2-alpine@sha256:4c6e91c6ed0e2fa03efd5b44747b625fec79bc9cd06ac5235a779726618e530d

WORKDIR /usr/share/caddy

# Copia os arquivos compilados estáticos do Dioxus (HTML, WASM, CSS, Assets)
COPY --from=builder /app/Frontend-Dioxus/dist .

# Usa o Caddyfile canônico da infraestrutura para gerenciar rotas e HTTPS.
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile
