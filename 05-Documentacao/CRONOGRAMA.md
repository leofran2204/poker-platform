# 📅 Cronograma — Plataforma de Poker Online

**Atualizado:** 2026-07-12
**Stack:** Rust para TUDO (backend + APIs + IA + dados + antifraude + autenticação + lobby + front-end Dioxus/WebAssembly)

> ⚠️ **Regra de Ouro:** Antes de codar, consultar `07-Arquitetura-Motor/ARQUITETURA_MOTOR.md` e `05-Documentacao/BUSINESS_RULES.md`.
> 📐 A spec viva (SDD) está em `STATUS.md`.
> 📋 Acompanhamento tático em `DASHBOARD.md`.

---

## 🗺️ Visão Geral das Fases — Roadmap do Projeto

```
FASE 1 ████████████████████████████ 100%  Fundação (docs + regras + stack)
FASE 2 ████████████████████████████ 100%  Motor de Jogo Rust + API Axum (11/11 módulos)
FASE 3 ████████████░░░░░░░░░░░░  57%  Front-end Dioxus (Roteamento + Mesa + Lobby + Auth concluídos)
FASE 4 ░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%  Infraestrutura (Docker + K8s + CI/CD)
FASE 5 ░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%  Segurança (TLS + JWT + MFA + LGPD)
FASE 6 ░░░░░░░░░░░░░░░░░░░░░░░░░░░░   0%  IA + Analytics (estatísticas + antifraude)
```

---

## 📊 Timeline por Fase — Marcos e Entregas

### ✅ FASE 1 — Fundação e Documentação (Completa — Jun/2026)

| #    | Marco                                                          | Data       | Status                         |
|------|----------------------------------------------------------------|------------|--------------------------------|
| 1.1  | Estrutura de pastas organizada (01-09)                         | 2026-06-27 | ✅                             |
| 1.2  | `BUSINESS_RULES.md` — 45 regras documentadas                   | 2026-06-25 | ✅                             |
| 1.3  | `AUDITORIA_REGRAS.md` — auditoria completa                     | 2026-06-25 | 🗑️ Deletado (legado Node.js)   |
| 1.4  | `ARQUITETURA_MOTOR.md` v3.1 — stack definitiva                 | 2026-07-03 | ✅                             |
| 1.5  | `DASHBOARD.md` — painel de controle tático                     | 2026-06-27 | ✅                             |
| 1.6  | `STATUS.md` — spec viva SDD                                    | 2026-06-25 | ✅                             |
| 1.7  | `DEVELOPMENT_LOG.md` — histórico de dev                        | 2026-06-25 | ✅                             |
| 1.8  | **Stack definitiva:** Rust para TUDO (Python/Go/TS removidos)  | 2026-07-03 | ✅                             |
| 1.9  | Golden rules salvas em memória persistente                     | 2026-07-03 | ✅                             |
| 1.10 | Pastas legadas excluídas (01, 02, 03, 06)                      | 2026-07-03 | ✅                             |

---

### ✅ FASE 2 — Motor de Poker em Rust + API Axum (100% — 11/11 módulos)

**Local:** `08-Motor-Rust/src/` + `10-API-Axum/`
**Progresso:** ████████████████████████████ 100%

| #    | Módulo                                   | Arquivo                 | Testes      | Status        | Prioridade  |
|------|------------------------------------------|-------------------------|-------------|---------------|-------------|
| 2.1  | **Deck + Hand Evaluation**               | `deck.rs`               | 18 ✅       | ✅ Completo   | —           |
| 2.2  | **Side Pots** (all-in múltiplos)         | `side_pots.rs`          | 7 ✅        | ✅ Completo   | —           |
| 2.3  | **Loss Deflator** (cashback progressivo) | `loss_deflator.rs`      | 9 ✅        | ✅ Completo   | —           |
| 2.4  | **Rake da Casa** (2.5%, cap R$6)         | `rake.rs`               | 13 ✅       | ✅ Completo   | —           |
| 2.5  | **evaluate_hand refatorado** (9 helpers) | `deck.rs`               | —           | ✅ Completo   | —           |
| 2.6  | **8 warnings de dead code limpos**       | —                       | —           | ✅ Completo   | —           |
| 2.7  | **Tournament Engine**                    | `tournament_engine.rs`  | 19 ✅       | ✅ Completo   | —           |
| 2.8  | **Hand History**                         | `hand_history.rs`       | 19 ✅       | ✅ Completo   | —           |
| 2.9  | **RNG Criptográfico**                    | `rng_crypto.rs`         | 20 ✅       | ✅ Completo   | —           |
| 2.10 | **Autenticação (JWT + MFA)**             | `auth.rs`               | 153 ✅      | ✅ Completo   | —           |
| 2.11 | **Conversão monetária u64 → f64**        | `utils.rs` + todos      | —           | ✅ Completo   | —           |
| 2.12 | **Lobby + Matchmaking**                  | `lobby.rs`              | 28 ✅       | ✅ Completo   | —           |
| 2.13 | **Antifraude** (4 submódulos)            | `antifraud/`            | ✅          | ✅ Completo   | —           |
| 2.14 | **API Axum (REST + WebSocket)**          | `10-API-Axum/`           | 12 ✅       | ✅ Completo   | —           |
|      | **Total: 498/498 testes, 0 warnings**    |                         | **498 ✅**  | **✅**        |             |

---

### 🔨 FASE 3 — Frontend WebAssembly com Dioxus (57% — Roteamento + Mesa + Lobby + Auth concluídos)

**Local:** `09-Frontend-Dioxus/`
**Progresso:** ████████████░░░░░░░░ 57%

| #    | Marco                                            | Data       | Status      | Prioridade  |
|------|--------------------------------------------------|------------|-------------|-------------|
| 3.1  | Projeto criado + compilando (`cargo check` ✅)   | 2026-07-03 | ✅          | —           |
| 3.2  | `Cargo.toml` configurado (Dioxus 0.6 + deps)     | 2026-07-03 | ✅          | —           |
| 3.3  | `Dioxus.toml` + `assets/index.html`              | 2026-07-03 | ✅          | —           |
| 3.4  | Toolchain GNU configurado (rust-toolchain.toml)   | 2026-07-03 | ✅          | —           |
| 3.5  | **Roteamento** (dioxus-router 0.6, 4 rotas + Navbar, 2 testes) | 2026-07-11 | ✅          | —           |
| 3.6  | **Componentes de Mesa** (7 componentes: card, avatar, pot, community_cards, action_buttons, seat, table, 22 testes) | 2026-07-11 | ✅          | —           |
| 3.7  | **Componentes de Lobby** (5 componentes: table_card, lobby_filters, join_button, player_count, lobby_list, 34 testes, CSS puro Full Tilt Poker) | 2026-07-12 | ✅          | —           |
| 3.8  | **Componentes de Auth** (login, registro, MFA — 3 componentes + 2 páginas, CSS puro, 61 testes frontend) | 2026-07-12 | ✅          | —           |
| 3.9  | **Integração API ↔ Front** (chamadas HTTP/WS) | —          | ⏳ Pendente | 🔴 Alta     |
| 3.10 | **Tela de Lobby** (salas)                         | —          | ⏳ Pendente | 🟡 Média    |
| 3.11 | **WebSocket (gloo-net)** — conexão com backend    | —          | ⏳ Pendente | 🟡 Média    |
| 3.12 | **Integração com API Axum**                       | —          | ⏳ Pendente | 🟡 Média    |
| 3.13 | **Tema visual + Tailwind CSS**                    | —          | ⏳ Pendente | 🟢 Baixa    |
| 3.14 | **Responsividade + mobile**                       | —          | ⏳ Pendente | 🟢 Baixa    |

---

### ⏳ FASE 4 — Infraestrutura e Deploy (0%)

**Local:** `04-Infraestrutura-Docker/`
**Progresso:** ░░░░░░░░░░░░░░░░░░░░ 0%

| #   | Marco                                                            | Status       | Prioridade  |
|-----|------------------------------------------------------------------|--------------|-------------|
| 4.1 | `docker-compose.yml` existente (PostgreSQL + Redis + Kafka)      | ✅ Esqueleto | —           |
| 4.2 | Dockerfile para `08-Motor-Rust/`                                 | ⏳ Pendente  | 🟡 Média    |
| 4.3 | Dockerfile para `09-Frontend-Dioxus/`                            | ⏳ Pendente  | 🟡 Média    |
| 4.4 | CI/CD (GitHub Actions)                                           | ⏳ Pendente  | 🟡 Média    |
| 4.5 | Kubernetes (manifestos)                                          | ⏳ Pendente  | 🟢 Baixa    |
| 4.6 | WebMCP (painel admin)                                            | ⏳ Pendente  | 🟢 Baixa    |

---

### ⏳ FASE 5 — Segurança e Conformidade Regulatória (0%)

| #   | Marco                                  | Status      | Prioridade  |
|-----|----------------------------------------|-------------|-------------|
| 5.1 | TLS 1.3 (rustls)                       | ⏳ Pendente | 🔴 Alta     |
| 5.2 | JWT + Refresh Tokens                   | ⏳ Pendente | 🔴 Alta     |
| 5.3 | MFA (2FA)                              | ⏳ Pendente | 🟡 Média    |
| 5.4 | bcrypt/argon2 para senhas              | ⏳ Pendente | 🔴 Alta     |
| 5.5 | AES-256 em repouso (dados sensíveis)   | ⏳ Pendente | 🟡 Média    |
| 5.6 | LGPD (proteção de dados)               | ⏳ Pendente | 🟡 Média    |
| 5.7 | PCI DSS (cartão de crédito)            | ⏳ Pendente | 🟢 Baixa    |
| 5.8 | ELK Stack (logs centralizados)         | ⏳ Pendente | 🟢 Baixa    |
| 5.9 | Grafana + Prometheus (monitoramento)   | ⏳ Pendente | 🟢 Baixa    |

---

### ⏳ FASE 6 — IA Antifraude e Analytics de Jogo (0%)

| #   | Marco                   | Status      | Prioridade  |
|-----|-------------------------|-------------|-------------|
| 6.1 | Estatísticas de jogo    | ⏳ Pendente | 🟢 Baixa    |
| 6.2 | Detecção de fraude (ML) | ⏳ Pendente | 🟢 Baixa    |
| 6.3 | Relatórios para admins  | ⏳ Pendente | 🟢 Baixa    |
| 6.4 | Dashboard de desempenho | ⏳ Pendente | 🟢 Baixa    |

---

## 📈 Resumo do Progresso — Visão Consolidada

| Fase                      | %         | Concluído                     | Pendente      |
|---------------------------|-----------|-------------------------------|---------------|
| **F1 — Fundação**         | **100%**  | 10/10                         | 0             |
| **F2 — Motor Rust + API** | **100%** | 11/11 módulos (496 testes)    | 0             |
| **F3 — Front-end Dioxus** | **40%**   | Esqueleto + Roteamento (3.5) + Componentes de Mesa (3.6) + Componentes de Lobby (3.7) | 6 marcos       |
| **F4 — Infraestrutura**   | **5%**    | docker-compose.yml esqueleto  | 5 marcos      |
| **F5 — Segurança**        | **0%**    | 0                             | 9 marcos      |
| **F6 — IA + Analytics**   | **0%**    | 0                             | 4 marcos      |
|                           |           |                               |               |
| **Total do Projeto**      | **~42%**  | **23 marcos**                 | **26 marcos** |

---

## 🎯 Próximos Passos Imediatos — Prioridade 🔴 Alta

```
┌─────────────────────────────────────────────────────────────┐
│  PRÓXIMOS MÓDULOS (escolher 1):                            │
│                                                             │
│  🔴 3.8  Componentes de Auth (login, registro, MFA)        │
│  🔴 3.9  Integração API ↔ Front (chamadas HTTP/WS)        │
└─────────────────────────────────────────────────────────────┘
```

---

## 📅 Linha do Tempo Estimada — Calendário de Entregas

```
2026-06-25 ██  Fundação inicial (regras, docs, estrutura)
2026-06-27 ██  Reorganização + DASHBOARD.md
2026-07-02 ██  Motor Rust: deck, side_pots, loss_deflator, rake
2026-07-03 ██  Stack Rust-only + Dioxus + docs atualizados
2026-07-10 ██  Lobby (2.12) + Antifraude (2.13) concluídos — Motor 100%
2026-07-10 ██  API Axum (2.14) concluída — 8 endpoints REST + WebSocket + JWT
2026-07-11 ██  Roteamento Dioxus (3.5) concluído — 4 rotas + Navbar + 2 testes
2026-07-11 ██  Componentes de Mesa (3.6) concluído — 7 componentes + 22 testes
2026-07-12 ██  Componentes de Lobby (3.7) concluído — 5 componentes + 34 testes + CSS puro Full Tilt Poker
══════════════════════════════════════════════════════════════
2026-07-xx ░░  PRÓXIMO: Componentes de Auth (3.8) + Integração API (3.9)
2026-07-xx ░░  Integração Front-end Dioxus ↔ API Axum (3.10–3.12)
2026-08-xx ░░  Infraestrutura + Deploy
2026-08-xx ░░  Segurança + Conformidade
2026-09-xx ░░  IA + Analytics
```

---

> 💡 **Dica:** Digite "vamos continuar" para carregar o `DASHBOARD.md` automaticamente e retomar o contexto.