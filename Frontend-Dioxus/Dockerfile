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

# Compila o projeto para WebAssembly
RUN cargo build --release --target wasm32-unknown-unknown

# Gera os bindings JS/WASM na pasta dist, copia o index.html e injeta o script de inicialização
RUN mkdir dist && \
    wasm-bindgen --target web --out-dir dist --no-typescript target/wasm32-unknown-unknown/release/poker-frontend.wasm && \
    cp assets/index.html dist/index.html && \
    sed -i 's|</body>|<script type="module">import init from "./poker_frontend.js"; init("./poker_frontend_bg.wasm");</script></body>|' dist/index.html

# Stage 2: Servidor estático leve usando Caddy
FROM caddy:2-alpine

WORKDIR /usr/share/caddy

# Copia os arquivos compilados estáticos do Dioxus (HTML, WASM, CSS, Assets)
COPY --from=builder /app/Frontend-Dioxus/dist .

# Copia o Caddyfile para gerenciar as rotas e HTTPS
COPY Frontend-Dioxus/Caddyfile /etc/caddy/Caddyfile
