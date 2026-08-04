# Zero Tilt Poker — poker-platform

Plataforma de poker online **Texas Hold'em**: **motor e API em Rust**, **frontend em TypeScript** (React + Vite + Tailwind), skin inspirada no **Full Tilt** clássico.

| | |
|--|--|
| **Domínio (demo)** | [https://zerotiltpoker.net](https://zerotiltpoker.net) |
| **Repositório** | https://github.com/leofran2204/poker-platform |
| **Estado** | Staging/demo — **sem** certificação de produção; PIX mock/sandbox; B2B multi-tenant |
| **Status canônico** | [`Documentacao/STATUS_OPERACIONAL.json`](Documentacao/STATUS_OPERACIONAL.json) |
| **Transporte público** | **HTTPS** (Caddy); API + SPA same-origin |
| **Regulação** | Trilho de compliance planejado para **janeiro de 2027** |

## Pastas principais

| Pasta | Função |
|-------|--------|
| `Motor-Rust/` | Regras de jogo, rake (incl. split B2B 15/85), loss deflator, antifraude |
| `API-Axum/` | REST + WebSocket (WSS), PostgreSQL, Redis, admin B2B |
| `Frontend-Web/` | **UI canônica** — TypeScript, React, Vite, Tailwind (lobby, mesa, admin clubs) |
| `Frontend-Dioxus/` | **Legado** WASM (não usado no deploy canônico) |
| `Infraestrutura-Docker/` | Compose, Caddy HTTPS, deploys casa/VPS |
| `Documentacao/` | Regras, dashboard, qualidade, status operacional |

## Stack (v4.0)

| Camada | Tecnologia |
|-------|------------|
| Motor + API | Rust (Axum, Tokio) |
| Frontend | TypeScript + React + Vite + Tailwind |
| Edge | Caddy (TLS, reverse_proxy `/api` `/ws`) |
| Dados | PostgreSQL 15, Redis 7 |

## Como publicar a demo (HTTPS)

### Opção A — Em casa + Cloudflare (sem cartão / sem VPS)

1. Guia: [`Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md`](Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md)
2. Certificados Origin CA: [`Infraestrutura-Docker/certs/README.md`](Infraestrutura-Docker/certs/README.md)
3. Subir:

```powershell
cd Infraestrutura-Docker
copy .env.tunnel.example .env
# edite JWT_SECRET; coloque origin.pem e origin-key.pem em certs\
docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up -d --build
# em outro terminal: cloudflared tunnel run zerotilt-poker
```

Browser: **https://zerotiltpoker.net** (TLS na Cloudflare + TLS na origem).

### Opção B — VPS (Hostinger KVM 2 / Hetzner / etc.)

Guia: [`Infraestrutura-Docker/DEPLOY_HETZNER.md`](Infraestrutura-Docker/DEPLOY_HETZNER.md)  
Env: `.env.staging.example` → `DOMAIN_NAME=zerotiltpoker.net`, `CORS_ORIGINS=https://zerotiltpoker.net`

## Desenvolvimento local

```powershell
# Stack completa
cd Infraestrutura-Docker
copy .env.example .env
docker compose up -d --build

# Só frontend (API em :3000)
cd Frontend-Web
npm install
npm run dev
```

Documentação completa: [`Documentacao/README.md`](Documentacao/README.md) · Painel: [`Documentacao/DASHBOARD.md`](Documentacao/DASHBOARD.md) · Arquitetura: [`Arquitetura-Motor/ARQUITETURA_MOTOR.md`](Arquitetura-Motor/ARQUITETURA_MOTOR.md)

## Demo com amigos (feedback)

Ver **[`Documentacao/DEMO_AMIGOS.md`](Documentacao/DEMO_AMIGOS.md)**.

- Registro público com **R$ 1.000** play-money
- Mesas demo NL2–NL25 (seed na migration `013`)
- Frontend usa **mesmo domínio** da página (API + WSS)

## Limites honestos

- PIX real e payout automático **desabilitados**
- Uma mesa = um processo (sem multi-pod de jogo)
- Sem certificação de produção; regulação em **2027-01**
- Demo em casa exige PC ligado e tunnel ativo
