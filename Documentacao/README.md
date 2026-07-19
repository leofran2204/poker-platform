# 📚 Documentação — Plataforma de Poker Online Texas Hold'em

Plataforma de poker online **Texas Hold'em Tradicional** (52 cartas) construída **100% em Rust** — motor de jogo, backend, frontend e antifraude.

> ⚠️ **Regra de Ouro:** Antes de codar qualquer feature, sempre consultar:
> 1. **`QUALITY.md`** — documento mestre (qualidade, segurança, negócio, arquitetura, compliance)
> 2. `Arquitetura-Motor/ARQUITETURA_MOTOR.md` — arquitetura oficial do motor
> 3. `Documentacao/BUSINESS_RULES.md` — regras de negócio
> 4. `Documentacao/DASHBOARD.md` — progresso atual e backlog

---

## 📖 Índice de Documentação

| Documento | Propósito | Público |
|-----------|-----------|---------|
| **`QUALITY.md`** | Documento mestre — qualidade, segurança, negócio, arquitetura, compliance | Toda a equipe |
| **`BUSINESS_RULES.md`** | Regras de negócio do poker (45 regras documentadas) | Negócio + Dev |
| **`DASHBOARD.md`** | Painel de controle tático — progresso, métricas, backlog | Gestão + Dev |
| **`CRONOGRAMA.md`** | Roadmap, fases, prazos e marcos | Gestão + Dev |
| **`DEVELOPMENT_LOG.md`** | Histórico cronológico de desenvolvimento | Dev |
| **`guia_aprendizado.md`** | Guia consolidado de aprendizado (Protocolo Mark + Regras Rust-only + Sprint S03 + 11 Módulos) | Dev |
| **`TESTING_GOALS.md`** | Metas de testes (2960 objetivos de teste) | Dev + QA |
| `Arquitetura-Motor/ARQUITETURA_MOTOR.md` | Arquitetura detalhada do motor Rust | Arquiteto + Dev |

---

## 🛠️ Stack Tecnológica — 100% Rust

| Camada | Tecnologia | Responsabilidade |
|--------|------------|------------------|
| **Motor de jogo** | Rust + Tokio | Cálculo de mãos, RNG, side pots, rake, loss deflator |
| **Backend / APIs** | Rust + Axum + Tokio | Auth, lobby, salas, hand history |
| **Frontend** | Rust + Dioxus 0.6 (WebAssembly) | UI no navegador via WASM |
| **Antifraude** | Rust | Colusão, chip dumping, bot detection, multi-account |
| **Banco de dados** | PostgreSQL 15 | Dados persistentes |
| **Cache / Sessões** | Redis 7 | Sessões, rate limiting, blacklist JWT |
| **Mensageria** | Kafka + Zookeeper | Eventos de jogo, hand history streaming |
| **Segurança** | rustls (TLS 1.3), aes-gcm (AES-256), bcrypt 0.16, JWT (hmac+sha2) | Criptografia, auth, MFA/TOTP, RBAC |

> **Stack 100% Rust desde 2026-07-03** — Python, TypeScript, Go e Node.js foram removidos.

---

## 📂 Estrutura do Projeto

| Pasta | Conteúdo | Status |
|-------|----------|--------|
| `Motor-Rust/` | Motor de poker em Rust (11 módulos + 4 antifraude, 1816 testes) | ✅ Ativo |
| `API-Axum/` | API HTTP/WebSocket (Axum + Tokio) | ✅ Ativo |
| `Frontend-Dioxus/` | Frontend WebAssembly (Dioxus 0.6) | ✅ Ativo |
| `Infraestrutura-Docker/` | Docker, deploy, CI/CD | ✅ Ativo |
| `Documentacao/` | Regras de negócio, cronograma, dashboard, logs | ✅ Ativo |
| `Arquitetura-Motor/` | Arquitetura do motor Rust | ✅ Ativo |
| `scripts/` | Scripts de automação (coverage, build) | ✅ Ativo |

---

## 🎲 Módulos do Motor de Jogo (`Motor-Rust/src/`)

| Módulo | Testes | Responsabilidade |
|--------|--------|-------------------|
| `deck.rs` | 18 | Baralho 52 cartas, Fisher-Yates com CSPRNG |
| `side_pots.rs` | 7 | Side pots para all-in |
| `loss_deflator.rs` | 9 | Cashback por equity no all-in (tiers 7/15/25/35%, equity ≥ 60%) |
| `rake.rs` | 13 | Rake da casa |
| `rng_crypto.rs` | 20 | CSPRNG com `OsRng` |
| `hand_history.rs` | 19 | Histórico imutável de mãos |
| `tournament_engine.rs` | 19 | Torneios (blinds crescentes, payouts) |
| `auth.rs` | 153 | JWT, bcrypt, MFA/TOTP, RBAC |
| `lobby.rs` | 28 | Lobby + matchmaking |
| `antifraud/` | — | Colusão, chip dumping, bot detection, multi-account |
| **Total** | **1816** | 0 warnings, 0 CVEs |

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
```

### 🖥️ Frontend WebAssembly (Dioxus)
```bash
cd Frontend-Dioxus
cargo install dioxus-cli --version 0.6
dx serve
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
- **Loss Deflator:** Cashback por equity no all-in (≥ 60%): 7% (60–64,9%), 15% (65–74,9%), 25% (75–84,9%), 35% (≥ 85%)
- **Side Pots:** Suporte completo a all-in com tamanhos diferentes
- **Rake:** Configurável por mesa (ver `rake.rs`)

---

## 🔗 Repositório — Código-Fonte no GitHub

https://github.com/leofran2204/poker-platform