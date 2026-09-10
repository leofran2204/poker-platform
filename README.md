# Zero Tilt Poker — poker-platform

Motor e API em **Rust**, UI em **TypeScript** (React + Vite + Tailwind), skin **Full Tilt**. Demo: [zerotiltpoker.net](https://zerotiltpoker.net). **Sem** certificação de produção.

| | |
|--|--|
| **Estado e catálogo** | [`Documentacao/STATUS_OPERACIONAL.md`](Documentacao/STATUS_OPERACIONAL.md) |
| **Contrato para qualquer LLM** | [`AGENTS.md`](AGENTS.md) |
| **Convite / como jogar** | [`Documentacao/DEMO_AMIGOS.md`](Documentacao/DEMO_AMIGOS.md) (mín. 2 na mesma mesa) |
| **Arquitetura** | [`Arquitetura-Motor/ARQUITETURA_MOTOR.md`](Arquitetura-Motor/ARQUITETURA_MOTOR.md) |
| **Gates CI** | [`Documentacao/QUALITY.md`](Documentacao/QUALITY.md) |
| **Painel** | [`Documentacao/DASHBOARD.md`](Documentacao/DASHBOARD.md) |
| **Índice de docs** | [`Documentacao/README.md`](Documentacao/README.md) |
| **Repositório** | https://github.com/leofran2204/poker-platform |

## Pastas

| Pasta | Função |
|-------|--------|
| `Motor-Rust/` | Regras, rake, deflator, antifraude |
| `API-Axum/` | REST + WebSocket, PostgreSQL, Redis |
| `Frontend-Web/` | UI canônica |
| `Infraestrutura-Docker/` | Compose, Caddy, deploy |
| `Documentacao/` | Donos por assunto; `historico/` = snapshots |
| `Arquitetura-Motor/` | Spec do motor/stack |
| `scripts/` | Deploy, e2e, full-validation |
| `src/bin/documentation_sync.rs` | Sync STATUS (CI `--check`) |
| `legacy/engine-v1/` | Arquivo git |

`Frontend-Dioxus/` foi removido do monorepo.

## Deploy e lab

- Casa + tunnel: [`Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md`](Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md)
- VPS: [`Infraestrutura-Docker/DEPLOY_HETZNER.md`](Infraestrutura-Docker/DEPLOY_HETZNER.md)
- Índice: [`DEPLOYMENT_VALIDATION.md`](Infraestrutura-Docker/DEPLOYMENT_VALIDATION.md)
- Lab: `cd Infraestrutura-Docker && docker compose up -d --build`

Ambiente de build (WSL, Node): [`AGENTS.md`](AGENTS.md).

## Limites

PIX automático e payout **desligados**. Uma mesa = um processo. Regulação planejada **2027-01**.
