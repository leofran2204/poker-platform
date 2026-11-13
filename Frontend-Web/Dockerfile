# Stage 1: build static SPA (TypeScript + Vite + Tailwind)
FROM node:24-alpine@sha256:d32cdf619f63fe0471182d08996dd516c6275bb5fd31ae06e55a570bd9e1ad43 AS builder

WORKDIR /app

COPY Frontend-Web/package.json Frontend-Web/package-lock.json ./
RUN npm ci

COPY Frontend-Web/ ./
RUN npm run build

# Stage 2: rebuild the latest stable Caddy with patched Go/dependencies.
# The upstream v2.11.4 binary predates fixes released after the tag, so the
# source commit and every security-sensitive dependency are pinned here.
FROM golang:1.26.10-alpine@sha256:87c65a688975c0b1d4585d204874c45ca78354670e3a5f11368c5a492303d696 AS caddy-builder

RUN go install github.com/caddyserver/xcaddy/cmd/xcaddy@e0e8f68a0f4bef11bfd04c4fb3091052ad36132f \
    && CGO_ENABLED=0 xcaddy build v2.11.4 \
        --output /usr/bin/caddy \
        --replace golang.org/x/crypto=golang.org/x/crypto@v0.57.0 \
        --replace golang.org/x/net=golang.org/x/net@v0.62.0 \
        --replace golang.org/x/text=golang.org/x/text@v0.42.0 \
        --replace google.golang.org/grpc=google.golang.org/grpc@v1.83.2 \
        --replace github.com/go-jose/go-jose/v4=github.com/go-jose/go-jose/v4@v4.1.6 \
        --replace github.com/klauspost/compress=github.com/klauspost/compress@v1.18.7

# Stage 3: Caddy serves SPA + reverse_proxy API/WS
FROM caddy:2.11.8-alpine@sha256:f6228111e5fafa1a5238258d0858e103f10599b8d1659a146e7726681a002bb9

WORKDIR /usr/share/caddy

COPY --from=caddy-builder /usr/bin/caddy /usr/bin/caddy
COPY --from=builder /app/dist .
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile

# Caddy's binary retains CAP_NET_BIND_SERVICE, so this unprivileged account can
# bind 80/443 while Compose continues to drop every other Linux capability.
# A imagem base é fixada por digest; pacotes do Alpine com correção de
# segurança publicada depois dela são atualizados aqui (Trivy HIGH/CRITICAL).
RUN apk upgrade --no-cache brotli-libs curl libcurl musl musl-utils zlib

RUN addgroup -S -g 10001 appgroup \
    && adduser -S -D -H -u 10001 -G appgroup appuser \
    && chown -R appuser:appgroup /usr/share/caddy /data /config \
    && setcap cap_net_bind_service=+ep /usr/bin/caddy

USER 10001:10001
