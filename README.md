# Zero Tilt Poker — poker-platform

Plataforma de poker online (**Texas Hold’em**, **Texas Short Deck**, **Short Deck Omaha 5-max**, **Ultimate Pineapple 6-max**): **motor e API em Rust**, **frontend em TypeScript** (React + Vite + Tailwind), skin inspirada no **Full Tilt** clássico.

| | |
|--|--|
| **Domínio (demo)** | [https://zerotiltpoker.net](https://zerotiltpoker.net) |
| **Repositório** | https://github.com/leofran2204/poker-platform |
| **Estado** | Staging/demo (**S21**) — **sem** certificação de produção; Big Blind Ante 26 níveis (torneios) + potes laterais com ante morto; cash sem ante; wallets PM × Real; settlements assinados; contador **online**; FT Short Deck **8-max**; torneio agendado **21:30 SP** (auto com 5+) |
| **Status canônico** | [`Documentacao/STATUS_OPERACIONAL.json`](Documentacao/STATUS_OPERACIONAL.json) |
| **Cash (PM + Real)** | NL 0,25/0,25 9-max · SD Texas 0,25/0,50 8-max · SD Omaha 0,50/0,50 5-max · Pineapple 0,50/0,50 6-max — frentes fixas |
| **Torneios (PM + Real)** | Texas Hold’em · Texas Freeroll (FT Short Deck 8-max) · Omaha 4 Cartas 5-max · Ultimate Pineapple 6-max — início agendado, BBA 26 níveis |
| **Pix / Saque** | Recebedor **Leofran** (`6eefcd53-686e-42d4-a062-03751336251c`); saque com chave própria, **recebimento em até 24h** |
| **Transporte público** | **HTTPS** (Caddy + Let's Encrypt na VPS); API + SPA same-origin |
| **E-mail (demo)** | Resend — domínio `zerotiltpoker.net` verified; ver [`EMAIL_RESEND.md`](Infraestrutura-Docker/EMAIL_RESEND.md) |
| **Presença** | Badge no header + hero na home; `GET /api/presence/online` |
| **Live smoke** | `scripts/live-e2e-ten-users.mjs` (10×100) · `scripts/full-catalog-100.mjs` (100 contas 72 cash + 28 MTT) · `scripts/full-catalog-100-vps.mjs` (VPS 2h real) |
| **Stress motor** | `Motor-Rust/tests/cash_catalog_10k_hands.rs` — 10k mãos por config (catálogo atual) · `Motor-Rust/src/bin/simulated_100.rs` — 100k/mesa (400k total) |
| **Demo amigos** | [`Documentacao/DEMO_AMIGOS.md`](Documentacao/DEMO_AMIGOS.md) — mín. **2 na mesma mesa**; torneios **21:30 SP** auto com **5+** |
| **Regulação** | Trilho de compliance planejado para **janeiro de 2027** |

## Mapa de pastas (canônico)

| Pasta | Função | Status |
|-------|--------|--------|
| `Motor-Rust/` | Regras de jogo, rake, loss deflator, antifraude, auth helpers | ✅ Ativo (dependência da API) |
| `API-Axum/` | REST + WebSocket, PostgreSQL, Redis, presence, payments, admin B2B | ✅ Ativo |
| `Frontend-Web/` | **UI canônica** — React/Vite/Tailwind (lobby, mesa, auth, admin) | ✅ Ativo (deploy) |
| `Infraestrutura-Docker/` | Compose, Caddy HTTPS, deploys casa/VPS | ✅ Ativo |
| `Documentacao/` | Regras, dashboard, status operacional, demo amigos | ✅ Ativo |
| `Arquitetura-Motor/` | Spec de arquitetura do motor/stack | ✅ Ativo |
| `scripts/` | Deploy, full-validation, live e2e, coverage | ✅ Ativo |
| `src/` + `tests/` + `benches/` | Pacote raiz `poker_engine` (incl. `documentation-sync`) e testes massivos | ✅ Ativo (tooling/CI histórico) |

> O antigo `Frontend-Dioxus/` (WASM) e scripts `*wasm*` / `cargo-dioxus*` foram **removidos** do monorepo (histórico git).

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

Documentação: [`Documentacao/README.md`](Documentacao/README.md) · Painel: [`Documentacao/DASHBOARD.md`](Documentacao/DASHBOARD.md) · Arquitetura: [`Arquitetura-Motor/ARQUITETURA_MOTOR.md`](Arquitetura-Motor/ARQUITETURA_MOTOR.md)

## Demo com amigos (feedback)

Ver **[`Documentacao/DEMO_AMIGOS.md`](Documentacao/DEMO_AMIGOS.md)**.

- Registro público com **R$ 1.000** play-money
- Contador **online** no topo e na home
- Mesas cash PM+Real: NL 9-max · SD Texas 8-max · SD Omaha 5-max · Pineapple 6-max (migrations `037`/`039`)
- Torneios agendados **21:30 SP**, auto-start com **5+**; FT Texas Freeroll Short Deck **8-max**
- Ranking Short Deck: **trinca > sequência** e **flush > full house**
- **Mínimo 2 pessoas na mesma mesa** para iniciar mão
- Pix manual: recebedor **Leofran**; saque com chave própria, **recebimento em até 24h**
- Frontend same-origin (API + WSS)

## Limites honestos

- PIX real e payout automático **desabilitados**
- Uma mesa = um processo (sem multi-pod de jogo)
- Sem certificação de produção; regulação em **2027-01**
- Demo em casa exige PC ligado e tunnel ativo
