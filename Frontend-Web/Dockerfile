# Stage 1: build static SPA (TypeScript + Vite + Tailwind)
FROM node:26-alpine@sha256:aadf416b2cdce311a8811ba3f0608a61b77dbf997500e2eafe781b51f6a0b019 AS builder

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
