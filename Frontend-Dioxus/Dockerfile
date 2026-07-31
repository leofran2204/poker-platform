# Stage 1: Build the frontend static files
FROM rust:latest AS builder

WORKDIR /app

# Adiciona o target wasm32 para compilação WebAssembly
RUN rustup target add wasm32-unknown-unknown

# Instala o cargo-binstall para baixar binários pré-compilados e economizar tempo
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Instala o wasm-bindgen-cli na mesma versão do projeto (0.2.126)
RUN cargo binstall -y wasm-bindgen-cli@0.2.126

# Copia os arquivos necessários do frontend
COPY Frontend-Dioxus ./Frontend-Dioxus

WORKDIR /app/Frontend-Dioxus

# Remove a especificação de toolchain do Windows
RUN rm -f rust-toolchain.toml rust-toolchain

# Compila o projeto para WebAssembly (feature web = default do Cargo.toml)
RUN cargo build --release --target wasm32-unknown-unknown --features web

# Gera bindings JS/WASM; nomes dependem do crate (hífen vs underscore).
RUN mkdir dist && \
    wasm-bindgen --target web --out-dir dist --no-typescript target/wasm32-unknown-unknown/release/poker-frontend.wasm && \
    cp assets/index.html dist/index.html && \
    JS=""; \
    if [ -f dist/poker-frontend.js ]; then JS=poker-frontend.js; \
    elif [ -f dist/poker_frontend.js ]; then JS=poker_frontend.js; \
    else echo "wasm-bindgen JS entry not found" >&2; ls -la dist; exit 1; fi && \
    WASM_BG="${JS%.js}_bg.wasm" && \
    sed -i "s|</body>|<script type=\"module\">import init from \"./${JS}\"; init({ module_or_path: \"./${WASM_BG}\" });</script></body>|" dist/index.html

# Stage 2: Servidor estático leve usando Caddy
FROM caddy:2-alpine

WORKDIR /usr/share/caddy

# Copia os arquivos compilados estáticos do Dioxus (HTML, WASM, CSS, Assets)
COPY --from=builder /app/Frontend-Dioxus/dist .

# Usa o Caddyfile canônico da infraestrutura para gerenciar rotas e HTTPS.
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile
