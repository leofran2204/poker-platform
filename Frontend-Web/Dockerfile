# Stage 1: build static SPA (TypeScript + Vite + Tailwind)
FROM node:20-alpine AS builder

WORKDIR /app

COPY Frontend-Web/package.json Frontend-Web/package-lock.json* ./
RUN npm install

COPY Frontend-Web/ ./
RUN npm run build

# Stage 2: Caddy serves SPA + reverse_proxy API/WS
FROM caddy:2-alpine

WORKDIR /usr/share/caddy

COPY --from=builder /app/dist .
COPY Infraestrutura-Docker/Caddyfile /etc/caddy/Caddyfile
