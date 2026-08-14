# Stage 1: Build the binary
FROM rust:1.97.1-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97 AS builder

WORKDIR /app

# Copia os diretórios necessários do workspace
COPY Motor-Rust ./Motor-Rust
COPY API-Axum ./API-Axum

# Compila a API em modo release
WORKDIR /app/API-Axum

# Remove qualquer arquivo de toolchain que possa estar nos subdiretórios copiados
RUN find /app -name "rust-toolchain*" -delete

RUN cargo build --release --locked

# Stage 2: Imagem final leve e blindada (Hardened Non-Root Container)
FROM debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241

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
