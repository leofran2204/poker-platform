# Stage 1: Build the frontend static files
FROM rust:1.97.0-bookworm@sha256:8fa55b2f3ddf97471ab6a767bfa3f37e6bad0986ba823e75fea57e2a2a5c3073 AS builder

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
FROM caddy:2.11.4-alpine@sha256:5f5c8640aae01df9654968d946d8f1a56c497f1dd5c5cda4cf95ab7c14d58648

WORKDIR /usr/share/caddy

# Copia os arquivos compilados estáticos do Dioxus (HTML, WASM, CSS, Assets)
COPY --from=builder /app/Frontend-Dioxus/dist .

# Usa o Caddyfile canônico da infraestrutura para gerenciar rotas e HTTPS.
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile
