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
FROM golang:1.26.9-alpine@sha256:cdfd4fe2da6b225d8b40c6b7a105736e548e83ff56d5d8f9394446eeb5eb84e0 AS caddy-builder

RUN go install github.com/caddyserver/xcaddy/cmd/xcaddy@e0e8f68a0f4bef11bfd04c4fb3091052ad36132f \
    && CGO_ENABLED=0 xcaddy build v2.11.4 \
        --output /usr/bin/caddy \
        --replace golang.org/x/crypto=golang.org/x/crypto@v0.57.0 \
        --replace golang.org/x/net=golang.org/x/net@v0.60.0 \
        --replace golang.org/x/text=golang.org/x/text@v0.42.0 \
        --replace google.golang.org/grpc=google.golang.org/grpc@v1.83.2

# Stage 3: Caddy serves SPA + reverse_proxy API/WS
FROM caddy:2.11.7-alpine@sha256:d8542f48d34a9cf4e4c11a478865229840e87e4c96ea3f439101f31a5d35f75f

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
