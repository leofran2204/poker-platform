# Stage 1: build static SPA (TypeScript + Vite + Tailwind)
FROM node:26-alpine@sha256:aadf416b2cdce311a8811ba3f0608a61b77dbf997500e2eafe781b51f6a0b019 AS builder

WORKDIR /app

COPY Frontend-Web/package.json Frontend-Web/package-lock.json ./
RUN npm ci

COPY Frontend-Web/ ./
RUN npm run build

# Stage 2: rebuild the latest stable Caddy with patched Go/dependencies.
# The upstream v2.11.4 binary predates fixes released after the tag, so the
# source commit and every security-sensitive dependency are pinned here.
FROM golang:1.26.6-alpine@sha256:3889b425f035be855a72fb4755265311293b6d414521f0a519d819df32222d83 AS caddy-builder

RUN go install github.com/caddyserver/xcaddy/cmd/xcaddy@e0e8f68a0f4bef11bfd04c4fb3091052ad36132f \
    && CGO_ENABLED=0 xcaddy build v2.11.4 \
        --output /usr/bin/caddy \
        --replace golang.org/x/crypto=golang.org/x/crypto@v0.55.0 \
        --replace golang.org/x/net=golang.org/x/net@v0.58.0 \
        --replace golang.org/x/text=golang.org/x/text@v0.41.0 \
        --replace google.golang.org/grpc=google.golang.org/grpc@v1.83.2

# Stage 3: Caddy serves SPA + reverse_proxy API/WS
FROM caddy:2.11.4-alpine@sha256:6aeddd44c3078b0f9a35206472a11420648a79c184603ef95957d0a20044cb2b

WORKDIR /usr/share/caddy

COPY --from=caddy-builder /usr/bin/caddy /usr/bin/caddy
COPY --from=builder /app/dist .
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile

# Caddy's binary retains CAP_NET_BIND_SERVICE, so this unprivileged account can
# bind 80/443 while Compose continues to drop every other Linux capability.
RUN addgroup -S -g 10001 appgroup \
    && adduser -S -D -H -u 10001 -G appgroup appuser \
    && chown -R appuser:appgroup /usr/share/caddy /data /config \
    && setcap cap_net_bind_service=+ep /usr/bin/caddy

USER 10001:10001
