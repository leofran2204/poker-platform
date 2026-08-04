# 📚 Documentação — Plataforma de Poker Online Texas Hold'em

Plataforma de poker online **Texas Hold'em Tradicional** (52 cartas) construída **100% em Rust** — motor de jogo, backend, frontend e antifraude.

**Domínio do produto (demo/staging):** [zerotiltpoker.net](https://zerotiltpoker.net)

> **Estado (ver `STATUS_OPERACIONAL.json`):** S10 — demo HTTPS + B2B SaaS multi-tenant (clubs, rake 15/85, agentes, dashboard admin via **HTTPS**). Isto **não** equivale a certificação de produção: multi-pod de mesas ainda requer ownership distribuído; PIX de produção desabilitado; site público depende de Cloudflare Tunnel (casa) ou VPS.

## Estado operacional e sincronização

Os fatos operacionais transversais — ciclo atual, escopo validado, limitações de PIX e de ownership de mesas — têm uma fonte canônica em [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). O comando abaixo atualiza somente os blocos marcados nos documentos; ele não altera registros históricos, termos legais, exemplos ou texto didático.

```bash
cargo run --bin documentation-sync -- --write
cargo run --bin documentation-sync -- --check
```

O segundo comando é obrigatório na CI. Se ele falhar, a mudança deve atualizar a fonte canônica e versionar todos os blocos gerados antes do merge.

> ⚠️ **Regra de Ouro:** Antes de codar qualquer feature, sempre consultar:
> 1. **`QUALITY.md`** — documento mestre (qualidade, segurança, negócio, arquitetura, compliance)
> 2. `Arquitetura-Motor/ARQUITETURA_MOTOR.md` — arquitetura oficial do motor
> 3. `Documentacao/BUSINESS_RULES.md` — regras de negócio
> 4. `Documentacao/DASHBOARD.md` — progresso atual e backlog

---

## 📖 Índice de Documentação

| Documento | Propósito | Público |
|-----------|-----------|---------|
| **`STATUS_OPERACIONAL.json`** | **Fonte canônica** de ciclo, limites PIX, ownership e validação | Toda a equipe |
| **`QUALITY.md`** | Documento mestre — qualidade, segurança, negócio, arquitetura, compliance | Toda a equipe |
| **`BUSINESS_RULES.md`** | Regras de negócio do poker (45+ regras documentadas, incl. B2B 15/85) | Negócio + Dev |
| **`ARQUITETURA_E_APIS.md`** | Contratos de API REST/WSS, admin B2B, protocolo de jogo | Arquiteto + Dev |
| **`DASHBOARD.md`** | Painel de controle tático — progresso, métricas, backlog | Gestão + Dev |
| **`CRONOGRAMA.md`** | Roadmap, fases, prazos e marcos (não certifica produção) | Gestão + Dev |
| **`DEVELOPMENT_LOG.md`** | Histórico cronológico de desenvolvimento | Dev |
| **`DEMO_AMIGOS.md`** | Como rodar demo play-money para feedback | Dev + ops |
| **`guia_aprendizado.md`** | Guia consolidado de aprendizado (Protocolo Mark + Regras Rust-only + Sprint S03 + 11 Módulos) | Dev |
| **`TESTING_GOALS.md`** | Metas e registros históricos; perfil atual de validação | Dev + QA |
| `Arquitetura-Motor/ARQUITETURA_MOTOR.md` | Arquitetura detalhada do motor Rust | Arquiteto + Dev |

---

## 🛠️ Stack Tecnológica — Rust (motor/API) + TypeScript (UI)

| Camada | Tecnologia | Responsabilidade |
|--------|------------|------------------|
| **Motor de jogo** | Rust + Tokio | Cálculo de mãos, RNG, side pots, rake, loss deflator |
| **Backend / APIs** | Rust + Axum + Tokio | Auth, lobby, salas, hand history |
| **Frontend** | TypeScript + React + Vite + Tailwind (`Frontend-Web/`) | UI Full Tilt moderna no navegador |
| **Antifraude** | Rust | Colusão, chip dumping, bot detection, multi-account |
| **Banco de dados** | PostgreSQL 15 | Dados persistentes |
| **Cache / Sessões** | Redis 7 | Sessões, rate limiting, blacklist JWT |
| **Mensageria** | — | Não provisionada nesta implantação |
| **Pagamentos / PIX** | Mock + Asaas Sandbox autenticado | Mercado Pago e PIX de produção desabilitados |
| **Segurança** | rustls (TLS 1.3), JWT, bcrypt, Caddy headers | Criptografia, auth, MFA/TOTP, RBAC |
| **Regulação** | — | Trilho planejado para **janeiro de 2027** |

> **Motor e API em Rust** (performance e dinheiro seguro). **UI em TypeScript** para velocidade de produto e skin Full Tilt. Ver `ARQUITETURA_MOTOR.md` v4.0.

---

## 📂 Estrutura do Projeto

| Pasta | Conteúdo | Status |
|-------|----------|--------|
| `Motor-Rust/` | Motor de poker em Rust, regras financeiras em inteiros | ✅ Ativo — CI usa testes determinísticos |
| `API-Axum/` | API REST / WebSocket (Axum + Tokio + PostgreSQL/Redis) | ✅ Ativo — contratos PostgreSQL no CI |
| `Frontend-Web/` | SPA TypeScript/React (deploy canônico) | ✅ Ativo |
| `Frontend-Dioxus/` | WASM legado | 📦 Legado |
| `Infraestrutura-Docker/` | Docker, Caddy, deploy (casa/VPS), CI/CD | ✅ Ativo |
| `Documentacao/` | Regras de negócio, cronograma, dashboard, logs | ✅ Ativo |
| `Arquitetura-Motor/` | Arquitetura do motor e stack | ✅ Ativo |
| `scripts/` | Scripts de automação (coverage, build, deploy) | ✅ Ativo |

### Deploy e domínio

| Documento | Uso |
|-----------|-----|
| [`Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md`](../Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md) | **Preferido sem VPS:** PC + Cloudflare Tunnel, **HTTPS E2E** (Origin CA) |
| [`Infraestrutura-Docker/DEPLOY_HETZNER.md`](../Infraestrutura-Docker/DEPLOY_HETZNER.md) | VPS Ubuntu (Hetzner ou similar) + Let's Encrypt |
| [`Infraestrutura-Docker/.env.tunnel.example`](../Infraestrutura-Docker/.env.tunnel.example) | Env da demo em casa |
| [`Infraestrutura-Docker/.env.staging.example`](../Infraestrutura-Docker/.env.staging.example) | Env staging VPS (`zerotiltpoker.net`) |
| [`Infraestrutura-Docker/certs/README.md`](../Infraestrutura-Docker/certs/README.md) | Como gerar Origin CA |

---

## 🎲 Módulos do Motor de Jogo (`Motor-Rust/src/`)

| Módulo | Testes | Responsabilidade |
|--------|--------|-------------------|
| `deck.rs` | 18 | Baralho 52 cartas, Fisher-Yates com CSPRNG |
| `side_pots.rs` | 7 | Side pots para all-in |
| `loss_deflator.rs` | 9+ | Cashback pós-rake por equity no all-in (56–65,9%: 7%; 66–75,9%: 15%; 76–85,9%: 25%; ≥86%: 35%) |
| `rake.rs` | 13 | Rake da casa (cap R$6, Regra Centavo Ímpar WSOP 68) |
| `rng_crypto.rs` | 20 | CSPRNG com `OsRng` |
| `hand_history.rs` | 19 | Histórico imutável de mãos e gravação no PostgreSQL |
| `tournament_engine.rs` | 19 | Torneios (blinds crescentes, payouts) |
| `auth.rs` | 153 | JWT, bcrypt, MFA/TOTP, RBAC |
| `lobby.rs` | 28 | Lobby + matchmaking |
| `antifraud/` | 117 | Bot detection, collusion, chip dumping, multi-account |
| `extreme_fuzz_tests.rs` | 8 | 1.000.000 iterações de fuzzing estocástico |
| **Validação do Motor** | **1.814 rotina / perfil autorizado sob demanda** | 79 cenários massivos adicionais; métricas por execução |

---

## 🚀 Quick Start — Como Rodar a Plataforma

### 📋 Pré-requisitos
- Rust (stable toolchain)
- Docker & Docker Compose

### 🐳 Infraestrutura — PostgreSQL, Redis e Kafka
```bash
cd Infraestrutura-Docker
docker-compose up -d
```

### 🎲 Motor de Poker — Build e Testes
```bash
cd Motor-Rust
cargo build
cargo test
# Cargas de plataforma: exigem autorização explícita.
.\scripts\full-validation.ps1 -Approved
```

### 🖥️ Frontend TypeScript (canônico)
```bash
cd Frontend-Web
npm install
npm run dev
# build produção: npm run build
```

### ✅ Validação de Qualidade — Clippy, Fmt, Audit e Cobertura
```bash
cd Motor-Rust
cargo clippy -- -D warnings    # 0 warnings
cargo fmt --check             # formatado
cargo audit                   # 0 CVEs
cargo tarpaulin               # cobertura ≥ 80%
```

---

## 🃏 Regras do Jogo — Texas Hold'em Tradicional

- **Variante:** Texas Hold'em Tradicional (52 cartas)
- **Formatos:** Cash Game e Tournament
- **Jogadores:** 2 a 9 por mesa
- **Loss Deflator:** calculado após o rake sobre os potes elegíveis; <56%: 0%, 56–65,9%: 7%, 66–75,9%: 15%, 76–85,9%: 25%, ≥86%: 35%. A fase do all-in apenas define o board do snapshot.
- **Side Pots:** Suporte completo a all-in com tamanhos diferentes
- **Rake:** Configurável por mesa (ver `rake.rs`)

---

## 🔗 Repositório — Código-Fonte no GitHub

https://github.com/leofran2204/poker-platform

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-01):** S10 — B2B SaaS Multi-Tenant White-Label (clubs, rake split 15/85, agentes/rakeback, dashboard admin via HTTPS, lobby MTT); domínio público zerotiltpoker.net; demo em casa com Cloudflare Tunnel e HTTPS de ponta a ponta; VPS/Hetzner opcional. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público previsto: demo residencial (Cloudflare Tunnel) ou VPS opcional — não multi-AZ. Staging/demo apenas; não alegar Launch Ready de produção.** Infra e docs de deploy (compose com healthchecks Postgres/Redis/API/Caddy, Caddyfile.tunnel HTTPS, DEPLOY_HOME_CLOUDFLARE, DEPLOY_HETZNER, .env.staging/.env.tunnel/.env.production.example) versionados no working tree. Suíte histórica reportada: ~2.051 testes (1.904 Motor Rust + 115 Dioxus + 32 API); full-validation e smoke em https://zerotiltpoker.net ainda pendem de execução. Migration 014 (clubs, memberships, club_agents, club_id em mesas/torneios) e endpoints admin B2B implementados localmente. Mock é o padrão. O único adaptador externo é o Asaas Sandbox, restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção permanecem desabilitados. Nenhum depósito com dinheiro real foi habilitado. Mesas continuam com dono único por processo; uma guarda persistente pausa a mesa após falha entre início e liquidação da mão, exigindo revisão/abort administrativo antes da reabertura.
>
> Fonte canônica: [STATUS_OPERACIONAL.json](STATUS_OPERACIONAL.json). Verificação: cargo run --bin documentation-sync -- --check.
<!-- DOCUMENTATION_SYNC:END -->
