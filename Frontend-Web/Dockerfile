# Stage 1: build static SPA (TypeScript + Vite + Tailwind)
FROM node:24-alpine@sha256:d32cdf619f63fe0471182d08996dd516c6275bb5fd31ae06e55a570bd9e1ad43 AS builder

WORKDIR /app

COPY Frontend-Web/package.json Frontend-Web/package-lock.json ./
RUN npm ci

COPY Frontend-Web/ ./
RUN npm run build

# Stage 2: Caddy serves SPA + reverse_proxy API/WS
FROM caddy:2.10.2-alpine@sha256:4c6e91c6ed0e2fa03efd5b44747b625fec79bc9cd06ac5235a779726618e530d

WORKDIR /usr/share/caddy

COPY --from=builder /app/dist .
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile
