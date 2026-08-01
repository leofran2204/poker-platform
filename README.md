# Zero Tilt Poker — poker-platform

Plataforma de poker online **Texas Hold'em** em **Rust** (motor, API Axum, frontend Dioxus/WASM).

| | |
|--|--|
| **Domínio (demo)** | [https://zerotiltpoker.net](https://zerotiltpoker.net) |
| **Repositório** | https://github.com/leofran2204/poker-platform |
| **Estado** | Staging/demo — **sem** certificação de produção; PIX mock/sandbox; B2B multi-tenant em implementação local |
| **Status canônico** | [`Documentacao/STATUS_OPERACIONAL.json`](Documentacao/STATUS_OPERACIONAL.json) |
| **Transporte público** | **HTTPS** (Caddy); API e WASM same-origin |

## Pastas principais

| Pasta | Função |
|-------|--------|
| `Motor-Rust/` | Regras de jogo, rake (incl. split B2B 15/85), loss deflator, antifraude |
| `API-Axum/` | REST + WebSocket (WSS), PostgreSQL, Redis, admin B2B |
| `Frontend-Dioxus/` | UI WebAssembly (lobby, mesa, `/admin/clubs`, `/tournament/:id`) |
| `Infraestrutura-Docker/` | Compose, Caddy HTTPS, deploys casa/VPS |
| `Documentacao/` | Regras, dashboard, qualidade, status operacional |

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

### Opção B — VPS (Hetzner ou provedor BR com PIX)

Guia: [`Infraestrutura-Docker/DEPLOY_HETZNER.md`](Infraestrutura-Docker/DEPLOY_HETZNER.md)  
Env: `.env.staging.example` → `DOMAIN_NAME=zerotiltpoker.net`, `CORS_ORIGINS=https://zerotiltpoker.net`

## Desenvolvimento local

```powershell
cd Infraestrutura-Docker
copy .env.example .env
docker compose up -d --build
```

Documentação completa: [`Documentacao/README.md`](Documentacao/README.md) · Painel: [`Documentacao/DASHBOARD.md`](Documentacao/DASHBOARD.md)

## Demo com amigos (feedback)

Ver **[`Documentacao/DEMO_AMIGOS.md`](Documentacao/DEMO_AMIGOS.md)**.

- Registro público com **R$ 1.000** play-money
- Mesas demo NL2–NL25 (seed na migration `013`)
- Frontend usa **mesmo domínio** da página (API + WSS)

## Limites honestos

- PIX real e payout automático **desabilitados**
- Uma mesa = um processo (sem multi-pod de jogo)
- Demo em casa exige PC ligado e tunnel ativo
