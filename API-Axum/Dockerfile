# Stage 1: Build the binary (com cache de dependências)
FROM rust:1.97.0-bookworm@sha256:8fa55b2f3ddf97471ab6a767bfa3f37e6bad0986ba823e75fea57e2a2a5c3073 AS builder

WORKDIR /app

# ─── Camada de dependências (efeito cargo-chef, sem imagem nova) ───
# Copia SÓ manifests + lockfile e compila as dependências uma vez.
# Essa camada só invalida quando Cargo.toml/lock mudam; edição em .rs
# recompila apenas poker-engine + poker-api (~1min em vez de ~10min).
COPY Motor-Rust/Cargo.toml ./Motor-Rust/Cargo.toml
COPY API-Axum/Cargo.toml ./API-Axum/Cargo.toml
COPY API-Axum/Cargo.lock ./API-Axum/Cargo.lock

# Esqueleto mínimo com os mesmos targets dos crates reais
# (API: lib + bin poker-api; Motor: lib como dependência).
RUN mkdir -p Motor-Rust/src API-Axum/src \
    && touch Motor-Rust/src/lib.rs \
    && touch API-Axum/src/lib.rs \
    && printf 'fn main() {}\n' > API-Axum/src/main.rs \
    && find /app -name "rust-toolchain*" -delete \
    && cd /app/API-Axum && cargo build --release --locked

# ─── Código real (invalida só esta camada em diante) ───
# Copia os diretórios necessários do workspace
COPY Motor-Rust ./Motor-Rust
COPY API-Axum ./API-Axum

# Compila a API em modo release
WORKDIR /app/API-Axum

# Remove qualquer arquivo de toolchain que possa estar nos subdiretórios copiados
# ATENÇÃO: o COPY preserva o mtime original dos fontes (mais antigo que os
# artefatos do esqueleto), então o `touch` abaixo é obrigatório — sem ele o
# cargo considera tudo "fresco" e a imagem sai com o binário dummy do esqueleto.
# O `test` no final falha o build em vez de publicar imagem quebrada em silêncio.
RUN find /app -name "rust-toolchain*" -delete \
    && find /app/Motor-Rust/src /app/API-Axum/src -name "*.rs" -exec touch {} + \
    && cargo build --release --locked \
    && test $(stat -c%s /app/API-Axum/target/release/poker-api) -gt 1000000

# Stage 2: Imagem final leve e blindada (Hardened Non-Root Container)
FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171

# Cria usuário não-privilegiado dedicado
RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -s /sbin/nologin -M appuser

WORKDIR /app

# Certificados TLS + curl (somente healthcheck Compose em loopback 127.0.0.1)
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Copia o binário e as migrations garantindo permissão para o usuário appuser
COPY --from=builder --chown=appuser:appgroup /app/API-Axum/target/release/poker-api /app/poker-api
COPY --from=builder --chown=appuser:appgroup /app/API-Axum/migrations /app/migrations

# Muda para o usuário não-privilegiado (Hardening OWASP Container Security)
USER 10001:10001

EXPOSE 3000

CMD ["/app/poker-api"]
