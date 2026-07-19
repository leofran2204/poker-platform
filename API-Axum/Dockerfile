# Stage 1: Build the binary
FROM rust:latest AS builder

WORKDIR /app

# Copia os diretórios necessários do workspace
COPY 08-Motor-Rust ./08-Motor-Rust
COPY 10-API-Axum ./10-API-Axum

# Compila a API em modo release
WORKDIR /app/10-API-Axum

# Remove qualquer arquivo de toolchain do Windows que possa estar nos subdiretórios copiados
RUN find /app -name "rust-toolchain*" -delete

RUN cargo build --release

# Stage 2: Imagem final leve para execução
FROM debian:bookworm-slim

WORKDIR /app

# Instala certificados SSL e dependências básicas de sistema
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copia o binário e as migrations do stage anterior
COPY --from=builder /app/10-API-Axum/target/release/poker-api /app/poker-api
COPY --from=builder /app/10-API-Axum/migrations /app/migrations

EXPOSE 3000

CMD ["/app/poker-api"]
