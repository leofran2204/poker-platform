# 🎰 Poker Platform — Plataforma de Poker Online Texas Hold'em

Plataforma de poker online **Texas Hold'em Tradicional** (52 cartas) construída **100% em Rust** — motor de jogo, backend, frontend e antifraude.

> ⚠️ **REGRA DE OURO:** Antes de codar qualquer feature, sempre consultar:
> 1. **`QUALITY.md`** — documento mestre (qualidade, segurança, negócio, arquitetura, compliance)
> 2. `07-Arquitetura-Motor/ARQUITETURA_MOTOR.md` — arquitetura oficial do motor
> 3. `05-Documentacao/BUSINESS_RULES.md` — regras de negócio
> 4. `05-Documentacao/STATUS.md` — status atual e backlog

---

## 🛠️ Stack Tecnológica — 100% Rust

| Camada            | Tecnologia                                                        | Responsabilidade                                                  |
|-------------------|-------------------------------------------------------------------|-------------------------------------------------------------------|
| **Motor de jogo** | Rust + Tokio                                                      | Cálculo de mãos, RNG, side pots, rake, loss deflator              |
| **Backend / APIs** | Rust + axum + Tokio                                              | Auth, lobby, salas, hand history                                   |
| **Frontend**      | Rust + Dioxus 0.6 (WebAssembly)                                   | UI no navegador via WASM                                           |
| **Antifraude**    | Rust                                                              | Colusão, chip dumping, bot detection, multi-account              |
| **Banco de dados** | PostgreSQL 15                                                    | Dados persistentes                                                |
| **Cache / Sessões** | Redis 7                                                        | Sessões, rate limiting, blacklist JWT                             |
| **Mensageria**    | Kafka + Zookeeper                                                 | Eventos de jogo, hand history streaming                          |
| **Segurança**     | rustls (TLS 1.3), aes-gcm (AES-256), bcrypt 0.16, JWT (hmac+sha2) | Criptografia, auth, MFA/TOTP, RBAC                                |

---

## 📂 Estrutura do Projeto — Organização dos Módulos

| Pasta                          | Conteúdo                                                       | Status    |
|--------------------------------|----------------------------------------------------------------|-----------|
| `08-Motor-Rust/`               | Motor de poker em Rust (8 módulos + 4 antifraude, 484+ testes) | ✅ Ativo  |
| `09-Frontend-Dioxus/`          | Frontend WebAssembly (Dioxus 0.6)                             | ✅ Ativo  |
| `04-Infraestrutura-Docker/`    | Docker, deploy, CI/CD                                         | ✅ Ativo  |
| `05-Documentacao/`             | Regras de negócio, status, cronograma, dashboard             | ✅ Ativo  |
| `07-Arquitetura-Motor/`        | Arquitetura do motor Rust                                     | ✅ Ativo  |
| `scripts/`                     | Scripts de automação (coverage)                               | ✅ Ativo  |

---

## 🎲 Módulos do Motor de Jogo (`08-Motor-Rust/src/`)

| Módulo                | Testes | Responsabilidade                                  |
|-----------------------|--------|---------------------------------------------------|
| `deck.rs`             | 18     | Baralho 52 cartas, Fisher-Yates com CSPRNG         |
| `side_pots.rs`        | 7      | Side pots para all-in                             |
| `loss_deflator.rs`    | 9      | Cashback em all-in call pré-river (equity ≥ 55%)   |
| `rake.rs`             | 13     | Rake da casa                                      |
| `rng_crypto.rs`       | 20     | CSPRNG com `OsRng`                                |
| `hand_history.rs`     | 19     | Histórico imutável de mãos                        |
| `tournament_engine.rs` | 19   | Torneios (blinds crescentes, payouts)             |
| `auth.rs`             | 153    | JWT, bcrypt, MFA/TOTP, RBAC                       |
| `antifraud/`          | —      | Colusão, chip dumping, bot detection, multi-account |

---

## 🚀 Quick Start — Como Rodar a Plataforma

### 📋 Pré-requisitos
- Rust (stable toolchain)
- Docker & Docker Compose

### 🐳 Infraestrutura — PostgreSQL, Redis e Kafka
```bash
cd 04-Infraestrutura-Docker
docker-compose up -d
```

### 🎲 Motor de Poker — Build e Testes
```bash
cd 08-Motor-Rust
cargo build
cargo test
```

### 🖥️ Frontend WebAssembly (Dioxus)
```bash
cd 09-Frontend-Dioxus
cargo install dioxus-cli --version 0.6
dx serve
```

### ✅ Validação de Qualidade — Clippy, Fmt, Audit e Cobertura
```bash
cd 08-Motor-Rust
cargo clippy -- -D warnings    # 0 warnings
cargo fmt --check             # formatado
cargo audit                   # 0 CVEs
cargo tarpaulin               # cobertura ≥ 80%
```

---

## 📚 Documentação — Mapa dos Artefatos

| Documento                                        | Propósito                                                              |
|--------------------------------------------------|------------------------------------------------------------------------|
| **`QUALITY.md`**                                 | Documento mestre — qualidade, segurança, negócio, arquitetura, compliance |
| `07-Arquitetura-Motor/ARQUITETURA_MOTOR.md`      | Arquitetura detalhada do motor Rust                                    |
| `05-Documentacao/BUSINESS_RULES.md`              | Regras de negócio do poker                                             |
| `05-Documentacao/STATUS.md`                      | Progresso atual e backlog                                              |
| `05-Documentacao/CRONOGRAMA.md`                  | Prazos e roadmap                                                       |
| `05-Documentacao/DASHBOARD.md`                   | Métricas de negócio e técnicas                                         |
| `05-Documentacao/DEVELOPMENT_LOG.md`             | Log de desenvolvimento                                                 |

---

## 🃏 Regras do Jogo — Texas Hold'em Tradicional

- **Variante:** Texas Hold'em Tradicional (52 cartas)
- **Formatos:** Cash Game e Tournament
- **Jogadores:** 2 a 9 por mesa
- **Loss Deflator:** Cashback em all-in call pré-river com equity ≥ 55%
- **Side Pots:** Suporte completo a all-in com tamanhos diferentes
- **Rake:** Configurável por mesa (ver `rake.rs`)

---

## 🔗 Repositório — Código-Fonte no GitHub

https://github.com/leofran2204/poker-platform