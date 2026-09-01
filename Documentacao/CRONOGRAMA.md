# 📅 Cronograma — Plataforma de Poker Online

> **Marco S20d (2026-09-01):** Polimento total — barra felt/gold, Zero Tilt sem Full Tilt, fontes H2 fidedignas e notícias com foto oficial.

**Atualizado:** 2026-09-01 | **Status:** Roadmap; ciclo **S20d** — Polimento total; demo VPS **zerotiltpoker.net**.
**Stack:** Rust (motor + API + antifraude) + TypeScript/React (frontend canônico). Regulação: **jan/2027**.

> ⚠️ **Regra de Ouro:** Antes de codar, consultar `Arquitetura-Motor/ARQUITETURA_MOTOR.md` e `Documentacao/BUSINESS_RULES.md`.
> 📐 Specs e regras de negócio em `Documentacao/BUSINESS_RULES.md`.
> 📋 Acompanhamento tático em `DASHBOARD.md`.
> 📌 Fonte canônica de estado: `STATUS_OPERACIONAL.json` (prevalece sobre percentuais deste arquivo).
> **Importante:** os percentuais abaixo descrevem marcos de código/documentação e **não** certificam lançamento em produção. PIX mock; escala horizontal depende de ownership distribuído. Deploy demo: VPS Hostinger ou `DEPLOY_HOME_CLOUDFLARE.md` / `DEPLOY_HETZNER.md`.

---

## 🗺️ Visão Geral das Fases — Roadmap do Projeto

```
FASE 1 ████████████████████████████ 100%  Fundação (docs + regras + stack)
FASE 2 ████████████████████████████ 100%  Motor de Jogo Rust + API Axum (1.904 testes + 1M Fuzzing)
FASE 3 ██████████████████████████░░  92%  Front-end: legado Dioxus → canônico TypeScript Full Tilt (S11; polish UI)
FASE 4 ████████████████████████████ 100%  Infraestrutura (Docker + Caddy HTTPS LE em VPS + CI/CD)
FASE 5 ████████████████████████████ 100%  Segurança & Pagamentos (TLS LE + JWT + MFA + Hardening + PIX mock + Resend e-mail)
FASE 6 ████████████████████████████ 100%  IA Antifraude & Analytics (BotDetector + Collusion + Prometheus + Red Team 50w)
FASE 7 ████████████████████████░░  92%  B2B SaaS Multi-Tenant & demo pública HTTPS (sem cert. produção)
```

---

## 📊 Timeline por Fase — Marcos e Entregas

### ✅ FASE 1 — Fundação e Documentação (100% — Jun/2026)

| #    | Marco                                                          | Data       | Status                         |
|------|----------------------------------------------------------------|------------|--------------------------------|
| 1.1  | Estrutura de pastas organizada                                 | 2026-06-27 | ✅                             |
| 1.2  | `BUSINESS_RULES.md` — 45 regras documentadas                   | 2026-06-25 | ✅                             |
| 1.3  | `ARQUITETURA_MOTOR.md` v3.1 — stack definitiva                 | 2026-07-03 | ✅                             |
| 1.4  | `DASHBOARD.md` — painel de controle tático                     | 2026-06-27 | ✅                             |
| 1.5  | `DEVELOPMENT_LOG.md` — histórico de dev                        | 2026-06-25 | ✅                             |
| 1.6  | Stack motor/API Rust; frontend reavaliado (TS canônico em 2026-08) | 2026-08-04 | ✅ (v4.0)                      |
| 1.7  | Golden rules salvas em memória persistente                     | 2026-07-03 | ✅                             |
| 1.8  | Pastas legadas excluídas (01, 02, 03, 06)                      | 2026-07-03 | ✅                             |

---

### ✅ FASE 2 — Motor de Poker em Rust + API Axum (100% — 1.904 testes + 1M Fuzzing)

**Local:** `Motor-Rust/src/` + `API-Axum/`  
**Progresso:** ████████████████████████████ 100%

| #    | Módulo                                   | Arquivo                 | Testes / Mutações | Status        | Prioridade  |
|------|------------------------------------------|-------------------------|-------------------|---------------|-------------|
| 2.1  | **Deck + Hand Evaluation**               | `deck.rs`               | 18 ✅             | ✅ Completo   | —           |
| 2.2  | **Side Pots** (all-in múltiplos)         | `side_pots.rs`          | 7 ✅              | ✅ Completo   | —           |
| 2.3  | **Loss Deflator por Equity pós-rake**     | `loss_deflator.rs`      | limites + integração ✅ | ✅ Completo | Regra 56/66/76/86 |
| 2.4  | **Rake da Casa & Regra Centavo Ímpar**   | `rake.rs` & `utils.rs`  | 14 ✅             | ✅ Completo   | —           |
| 2.5  | **evaluate_hand refatorado** (9 helpers) | `deck.rs`               | —                 | ✅ Completo   | —           |
| 2.6  | **8 warnings de dead code limpos**       | —                       | —                 | ✅ Completo   | —           |
| 2.7  | **Tournament Engine**                    | `tournament_engine.rs`  | 19 ✅             | ✅ Completo   | —           |
| 2.8  | **Hand History & Persistência SQL**      | `hand_history.rs`       | 19 ✅             | ✅ Completo   | —           |
| 2.9  | **RNG Criptográfico (CSPRNG)**           | `rng_crypto.rs`         | 20 ✅             | ✅ Completo   | —           |
| 2.10 | **Autenticação (JWT + MFA + CSPRNG UUID)**| `auth.rs`               | 153 ✅            | ✅ Completo   | —           |
| 2.11 | **Conversão monetária u64 → f64**        | `utils.rs` + todos      | —                 | ✅ Completo   | —           |
| 2.12 | **Lobby + Matchmaking**                  | `lobby.rs`              | 28 ✅             | ✅ Completo   | —           |
| 2.13 | **Antifraude Unificada (Facade)**        | `antifraud/`            | 117 ✅            | ✅ Completo   | —           |
| 2.14 | **API Axum (REST + WebSocket Atores)**   | `API-Axum/`              | 34 ✅             | ✅ Completo   | —           |
| 2.15 | **Fuzzing Extremo Massivo**              | `extreme_fuzz_tests.rs` | 1.000.000 iters ✅| ✅ Completo   | —           |
|      | **Total Motor Rust**                     |                         | **1.904 testes ✅**| **✅**        |             |

---

### ✅ FASE 3 — Frontend WebAssembly com Dioxus (100% — Roteamento + Mesa + Lobby + Auth + Modais PIX)

**Local:** `Frontend-Dioxus/`  
**Progresso:** ████████████████████████████ 100%

| #    | Marco                                            | Data       | Status      | Prioridade  |
|------|--------------------------------------------------|------------|-------------|-------------|
| 3.1  | Projeto criado + compilando (`cargo check` ✅)   | 2026-07-03 | ✅          | —           |
| 3.2  | `Cargo.toml` configurado (Dioxus 0.6 + deps)     | 2026-07-03 | ✅          | —           |
| 3.3  | `Dioxus.toml` + `assets/index.html`              | 2026-07-03 | ✅          | —           |
| 3.4  | Toolchain GNU configurado (rust-toolchain.toml)   | 2026-07-03 | ✅          | —           |
| 3.5  | **Roteamento** (dioxus-router 0.6, 4 rotas + Navbar, 2 testes) | 2026-07-11 | ✅          | —           |
| 3.6  | **Componentes de Mesa** (7 componentes, 22 testes) | 2026-07-11 | ✅          | —           |
| 3.7  | **Componentes de Lobby** (5 componentes, 34 testes, CSS Full Tilt) | 2026-07-12 | ✅          | —           |
| 3.8  | **Componentes de Auth** (login, registro, MFA, 61 testes) | 2026-07-12 | ✅          | —           |
| 3.9  | **Integração API ↔ Front** (WebSockets `WsClient` stateful) | 2026-07-17 | ✅          | —           |
| 3.10 | **Modais PIX (Depósito & Saque)** (`DepositModal` & `WithdrawModal`) | 2026-07-22 | ✅          | —           |
| 3.11 | **Fuzzing & Estresse Frontend** (10.000 mutações em rajada) | 2026-07-22 | ✅ (115 suítes)| —           |

---

### ✅ FASE 4 — Infraestrutura e Deploy (100% — Hardening + Docker Stack + CI/CD)

**Local:** `Infraestrutura-Docker/`  
**Progresso:** ████████████████████████████ 100%

| #   | Marco                                                            | Status       | Prioridade  |
|-----|------------------------------------------------------------------|--------------|-------------|
| 4.1 | `docker-compose.yml` (PostgreSQL 15 + Redis 7 + Kafka + Caddy)   | ✅ Completo  | —           |
| 4.2 | Dockerfile Multi-stage para `Motor-Rust/` & `API-Axum/` (USER 10001)| ✅ Completo  | —           |
| 4.3 | Dockerfile Multi-stage para `Frontend-Dioxus/` + Caddy HTTPS     | ✅ Completo  | —           |
| 4.4 | Pipeline CI/CD GitHub Actions (`.github/workflows/rust-ci.yml`)   | ✅ Completo  | —           |
| 4.5 | Scripts de Deploy Autônomo (`scripts/deploy.sh` & `deploy.ps1`)  | ✅ Completo  | —           |

---

### ✅ FASE 5 — Segurança & Conformidade Regulatória (100% — Enterprise Hardening + Gateway PIX)

| #   | Marco                                  | Status      | Prioridade  |
|-----|----------------------------------------|-------------|-------------|
| 5.1 | TLS 1.3 & HTTPS Caddy Proxy            | ✅ Completo | —           |
| 5.2 | JWT + Refresh Tokens & Rate Limiting IP| ✅ Completo | —           |
| 5.3 | MFA / TOTP (RFC 6238)                  | ✅ Completo | —           |
| 5.4 | bcrypt 0.16 para senhas                | ✅ Completo | —           |
| 5.5 | Container Hardening (read_only, cap_drop)| ✅ Completo | —           |
| 5.6 | PIX para desenvolvimento: Mock + Asaas Sandbox autenticado | ⏸️ Produção bloqueada | Requer PSP compatível, conformidade e operação reconciliada |
| 5.7 | Validação Webhook PIX HMAC-SHA256      | ✅ Completo | —           |
| 5.8 | Audit de Segurança (DevSecOps Trivy)   | ✅ Completo | —           |
| 5.9 | Prometheus & Health Security Endpoints | ✅ Completo | —           |

---

### ✅ FASE 6 — IA Antifraude, Red Team & Estresse (100% — 1M WS Stress + Red Team 50w)

| #   | Marco                   | Status      | Prioridade  |
|-----|-------------------------|-------------|-------------|
| 6.1 | `BotDetector` (análise de variância de tempo de reação) | ✅ Completo | — |
| 6.2 | `CollusionDetector` & `RiskScore` unificados no `TableActor` | ✅ Completo | — |
| 6.3 | Simulação Red Team Autônoma (50 workers simultâneos) | ✅ Completo | — |
| 6.4 | Carga Massiva WebSocket (1.000.800 msgs em 100 mesas) | ✅ Completo | — |

---

### 🟡 FASE 7 — B2B SaaS Multi-Tenant & Dashboard (~90% — código local; sem cert. produção)

| #   | Marco | Status | Notas |
|-----|-------|--------|-------|
| 7.1 | Schema `014` clubs / memberships / `club_id` | ✅ Local | Untracked até commit |
| 7.2 | Rake split 15/85 no motor + crédito `clubs.balance` | ✅ Local | Invariante testada |
| 7.3 | Admin API clubs + agents + financials/theme/withdraw | ✅ Local | Role `admin` + JWT |
| 7.4 | Dashboard Dioxus `/admin/clubs` via HTTPS | ✅ Local | Fallback demo sem JWT |
| 7.5 | Lobby MTT `/tournament/:id` | ✅ Local | UI demo + estrutura blinds/prizepool |
| 7.6 | Smoke público `https://zerotiltpoker.net` | 🟡 Pendente | Tunnel/VPS operacional |
| 7.7 | Certificação produção / PIX real | ⏸️ Fora de escopo | Ver STATUS_OPERACIONAL |

---

## 📈 Resumo do Progresso — Visão Consolidada

| Fase                      | %         | Concluído                                     | Pendente      |
|---------------------------|-----------|-----------------------------------------------|---------------|
| **F1 — Fundação**         | **100%**  | 8/8 marcos                                    | 0             |
| **F2 — Motor Rust + API** | **100%**  | 15/15 módulos (1.904 testes + 1M Fuzzing)     | 0             |
| **F3 — Front-end Dioxus** | **100%**  | 11/11 marcos (115 suítes de teste)            | 0             |
| **F4 — Infraestrutura**   | **100%**  | 5/5 marcos (Docker + Caddy + CI/CD)           | 0             |
| **F5 — Segurança & PIX**  | **100%**  | 9/9 marcos (HTTPS + Rate Limit + PIX)         | 0             |
| **F6 — IA & Red Team**    | **100%**  | 4/4 marcos (1M WS Stress + 50 Red Team workers)| 0             |
| **F7 — B2B SaaS & Admin** | **~90%**  | Clubs, rake 15/85, agentes, dashboard HTTPS, MTT UI | Go-live demo + cert. produção fora de escopo |
|                           |           |                                               |               |
| **Total da plataforma**   | **~98%**  | Marcos de código/demo entregues               | Smoke domínio + PIX real + multi-pod |

---

## 📅 Linha do Tempo Final — Entregas Executadas

```
2026-06-25 ██  Fundação inicial (regras, docs, estrutura)
2026-06-27 ██  Reorganização + DASHBOARD.md
2026-07-02 ██  Motor Rust: deck, side_pots, loss_deflator, rake
2026-07-03 ██  Stack Rust-only + Dioxus + docs atualizados
2026-07-10 ██  Lobby (2.12) + Antifraude (2.13) concluídos — Motor 100%
2026-07-10 ██  API Axum (2.14) concluída — 8 endpoints REST + WebSocket + JWT
2026-07-12 ██  Componentes de Lobby e Auth (Dioxus 0.6)
2026-07-17 ██  Integração Real-time WebSockets ↔ Dioxus & Docker Caddy
2026-07-20 ██  CI/CD GitHub Actions + Job de Cobertura (llvm-cov)
2026-07-22 ██  Hardening Enterprise + Gateway PIX + Red Team + Fuzzing 1M
2026-07-23 ██  Deep Audit Fixes + Odd Cent Rule (WSOP 68) + snapshots multi-all-in
2026-07-30 ██  Loss Deflator por equity 56/66/76/86, sempre após o rake
2026-08-01 ██  B2B SaaS Multi-Tenant (014: clubs, agents), Rake Split 15/85, dashboard HTTPS, lobby MTT
2026-08-31 ██  S19 — Sessão resiliente + DePix Sandbox (030) + presence TTL 90s
2026-09-01 ██  S20 — Big Blind Ante 26 níveis (032) ante morto + 1828 Motor + 35 API + VPS 4/4 healthy
2026-09-01 ██  S20b — História completa (8+7 blocos) + PT-BR normalizado + Dica do Pró
2026-09-01 ██  S20c — UI polimento: laterais com história, sem vazios login, sem painel duplicado, sem A♠, fontes H2
2026-09-01 ██  S20d — Polimento total: barra felt/gold, Zero Tilt sem Full Tilt, H2 revisado, notícias com foto oficial
```

---

> 💡 **Nota (2026-09-01):** S20 validado com 1828 Motor + 35 API + 3 BBA, clippy estrito e Vite — 0 falhas. VPS Hostinger 4/4 healthy (migrations 032, ante=big_blind 26/26). Isso **não** equivale a certificação de produção: PIX real desabilitado, ownership de mesa single-process. Ver `STATUS_OPERACIONAL.json`.

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-01):** S20d — Polimento total: barra felt/gold, bloco Zero Tilt sem Full Tilt, H2 fontes e sem A♠; catálogo cash canônico NLHE 0,25/0,25, Hold'em Short Deck 0,25/0,50 e Omaha Short Deck 0,50/0,50 (Play Money e Jogo Real). **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy; migrations 001–032 aplicadas. Gate S20d: cargo fmt, Clippy estrito, tsc -b + Vite 60 módulos — todos sem falhas; VPS 4/4 healthy, 26 níveis ante=big_blind (invalid_BBA=0), backup verificável, rebuilds e health público OK. Frontend: PT-BR + Dica do Pró + história 8+7 com fontes H2 2006 + vazios com história + sem painel duplicado + sem A♠ (case-sensitive) + scrollbar felt/gold + bloco Zero Tilt sem Full Tilt + notícias com foto oficial (sem placeholder). A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
