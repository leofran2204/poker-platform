# 📚 Documentação — Zero Tilt Poker

Plataforma de poker online (**Hold’em**, **Short Deck**, **Short Deck Omaha**, **Ultimate Pineapple**): **motor e API em Rust**, **UI em TypeScript** (React).

**Domínio do produto (demo/staging):** [zerotiltpoker.net](https://zerotiltpoker.net)

> **Estado (ver `STATUS_OPERACIONAL.json`):** **S18** — catálogo cash NLHE+SD+SD Omaha (PM×Real); frentes fixas; wallets isoladas; depósitos manuais; notícias com capa temática; testes 10k/mesa + e2e seeded. Isto **não** equivale a certificação de produção.

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
| **`DEMO_AMIGOS.md`** | Convite amigos: cadastro, e-mail, contador online, mín. 2 na mesa | Dev + ops + anfitrião |
| **`guia_aprendizado.md`** | Guia consolidado de aprendizado (histórico; stack UI atual é TypeScript) | Dev |
| **`TESTING_GOALS.md`** | Metas e registros históricos; perfil atual de validação | Dev + QA |
| **`SIMULACAO_RITUAL.md`** | Ritual Play Money + motor MTT até o campeão (2026-09-02) | Dev + QA |
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
| **Cache / Sessões** | Redis 7 | Sessões, rate limiting, tickets WS, presença online |
| **Mensageria** | — | Não provisionada nesta implantação |
| **Pagamentos / PIX** | Mock + Asaas Sandbox + DePix Sandbox allowlisted | DePix/PIX de produção e saque automático desabilitados |
| **Segurança** | rustls (TLS 1.3), JWT, bcrypt, Caddy headers | Criptografia, auth, MFA/TOTP, RBAC |
| **Regulação** | — | Trilho planejado para **janeiro de 2027** |

> **Motor e API em Rust** (performance e dinheiro seguro). **UI em TypeScript** para velocidade de produto e skin Full Tilt. Ver `ARQUITETURA_MOTOR.md` v4.0.

---

## 📂 Estrutura do Projeto

| Pasta | Conteúdo | Status |
|-------|----------|--------|
| `Motor-Rust/` | Motor de poker em Rust, regras financeiras em inteiros | ✅ Ativo — CI usa testes determinísticos |
| `API-Axum/` | API REST / WebSocket (Axum + Tokio + PostgreSQL/Redis) | ✅ Ativo — contratos PostgreSQL no CI |
| `Frontend-Web/` | SPA TypeScript/React (deploy canônico) + contador online | ✅ Ativo |
| `Infraestrutura-Docker/` | Docker, Caddy, deploy (casa/VPS), CI/CD | ✅ Ativo |
| `Documentacao/` | Regras de negócio, cronograma, dashboard, logs | ✅ Ativo |
| `Arquitetura-Motor/` | Arquitetura do motor e stack | ✅ Ativo |
| `scripts/` | Deploy, full-validation, live e2e, coverage | ✅ Ativo |
| `src/` + `tests/` | Pacote raiz `poker_engine` (`documentation-sync`) e testes massivos | ✅ Tooling |

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
> **Estado operacional sincronizado (2026-09-04):** S21 — Texas Hold’em rename + FT Short Deck 8-max + Omaha 5-max + Pineapple 6-max + Short Deck ranking trips>straight + torneio agendado 21:30 SP auto-start 5 + Pix Leofran + saque 24h + lobby max sempre + sim 100k/mesa + PM 150+150 sem rebuy (ilimitado com saldo, play money) **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–044 na VPS (cash Texas SD 8-max + Omaha 5-max + Pineapple 6-max + Texas rename + FT 8 + scheduled 21:30 + PM 150+150 + restore catálogo + Dockerfile cache). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor short_deck_massive + tournament_to_champion PASS (Texas/Omaha 5/Pineapple 6 até 1 campeão; flush>FH e trips>straight). VPS 2h real 100 contas: 980 mãos R$135,11 rake, 4 campeões MTT. Simulado Motor-Rust/src/bin/simulated_100.rs 100k/mesa (400k total). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT site: inscrição + horário agendado + popup FT; gameplay_ready=false (sem WS de torneio). Health público OK. Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
