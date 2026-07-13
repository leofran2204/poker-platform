# 📐 SPEC — Status da Implementação da Plataforma de Poker (SDD)

**Última atualização:** 2026-07-12
**Tipo:** Spec-Driven Development — este documento é a fonte da verdade sobre o que está implementado vs especificado.

> ⚡ **REGRA PRIMORDIAL:** Este documento deve refletir a realidade do projeto. Se divergir de `QUALITY.md` (documento mestre), corrigir imediatamente. Ver auditoria contínua em `/memories/poker-project-golden-rules.md`.
> 📋 Para acompanhamento tático de tarefas, veja `DASHBOARD.md`.
> 📅 Para cronograma completo com fases e prazos, veja `CRONOGRAMA.md`.

---

## 🎯 0. VISÃO DE PRODUTO (PRD Enxuto)

> **Princípio:** "Antes de saber O QUE está implementado, precisa saber POR QUÊ existe."
> Esta seção consolida a função do PRD (Product Requirements Document) no topo da spec viva.

### 📌 0.1 Visão e Objetivo

**Visão:** Construir a plataforma de poker online **mais justa, segura e transparente** do mercado brasileiro, onde a confiança do jogador é o produto.

**Objetivo:** Uma plataforma completa (motor + API + frontend + infraestrutura + segurança + IA antifraude) escrita 100% em Rust, capaz de operar com dinheiro real sob conformidade LGPD e PCI DSS.

### 👥 0.2 Personas (Usuários-Alvo)

| # | Persona | Perfil | Necessidades Principais | Prioridade no Produto |
|---|---------|--------|-------------------------|----------------------|
| 1 | **Jogador Casual** | Joga 1-2x/semana, micro-stakes (R$1-R$10) | Interface simples, jogo justo, saque rápido | 🔴 Alta |
| 2 | **Jogador Profissional** | Joga diariamente, stakes altos (R$50+) | Estatísticas, hand history, multi-tabling, rake baixo | 🟡 Média |
| 3 | **Admin da Casa** | Gerencia a plataforma | Painel de gestão, controle de rake, monitoramento, relatórios | 🔴 Alta |
| 4 | **Auditor / Regulador** | Verifica conformidade | Logs imutáveis, RNG auditável, relatórios LGPD/PCI | 🟡 Média |
| 5 | **Especialista em Segurança** | Pentester / bug bounty | Documentação de segurança, escopo autorizado, RoE | 🟡 Média |

### 📊 0.3 KPIs de Negócio (Métricas de Sucesso do Produto)

| # | KPI | Meta (v1.0) | Meta (v2.0) | Frequência |
|---|-----|-------------|-------------|------------|
| 1 | **DAU** (Daily Active Users) | 100 | 1.000 | Diária |
| 2 | **MAU** (Monthly Active Users) | 1.000 | 10.000 | Mensal |
| 3 | **Rake mensal** | R$ 5.000 | R$ 50.000 | Mensal |
| 4 | **Taxa de retenção (D7)** | > 30% | > 50% | Semanal |
| 5 | **NPS** (Net Promoter Score) | > 40 | > 60 | Trimestral |
| 6 | **% fraudes detectadas** | > 95% | > 99% | Mensal |
| 7 | **Tempo médio de saque** | < 24h | < 6h | Contínuo |
| 8 | **Uptime** | 99.5% | 99.9% | Contínuo |
| 9 | **Latência WebSocket** | < 200ms | < 100ms | Contínuo |
| 10 | **Custo por jogador ativo/mês** | < R$ 5 | < R$ 2 | Mensal |

### 🚫 0.4 Fora de Escopo (v1.0)

| # | Item | Justificativa | Reavaliar em |
|---|------|---------------|-------------|
| 1 | Apostas esportivas | Foco em poker apenas | v3.0 |
| 2 | Casino (slots, roleta) | Foco em poker apenas | v3.0 |
| 3 | App mobile nativo (iOS/Android) | WebAssembly responsivo cobre mobile | v2.0 |
| 4 | Criptomoedas como pagamento | Conformidade regulatória complexa | v2.0 |
| 5 | Multi-idioma (i18n) | PT-BR primeiro | v2.0 |
| 6 | Poker Omaha / Stud | Texas Hold'em primeiro | v2.0 |
| 7 | Streaming de mesas (Twitch-style) | Custo de infraestrutura alto | v3.0 |
| 8 | Marketplace de skins/avatar | Não essencial para jogo | v3.0 |

### 📐 0.5 Requisitos Não-Funcionais (Resumo — detalhes em QUALITY.md)

| Categoria | Requisito | Documentação Detalhada |
|-----------|-----------|------------------------|
| Segurança | TLS 1.3, JWT, MFA, AES-256, RNG auditável | `QUALITY.md` §1-§4 |
| Performance | < 100ms latência, 99.9% uptime | `QUALITY.md` §2 |
| Conformidade | LGPD, PCI DSS | `QUALITY.md` §5-§6 |
| Observabilidade | ELK + Grafana + Prometheus | `QUALITY.md` §7 |
| Antifraude | Colusão, bots, chip dumping, multi-account | `QUALITY.md` §4 + `08-Motor-Rust/src/antifraud/` |
| Pentest | PTES, OWASP WSTG, NIST SP 800-115 | `QUALITY.md` §4.0 |

### 🔄 0.6 Cadência de Sprints

| # | Parâmetro | Valor |
|---|-----------|-------|
| 1 | **Duração do sprint** | 2 semanas |
| 2 | **Definition of Done (DoD)** | Ver `DASHBOARD.md` §DoD |
| 3 | **Cerimônias** | Planning (início) + Review + Retrospectiva (fim) |
| 4 | **Retrospectivas** | Registradas em `DEVELOPMENT_LOG.md` |
| 5 | **Backlog** | Este documento (STATUS.md) + `DASHBOARD.md` |
| 6 | **Roadmap** | `CRONOGRAMA.md` (F1-F6) |

---

## ✅ Concluído — Módulos Implementados da Plataforma de Poker

### 🎪 Motor Rust — Lobby + Matchmaking (2026-07-10) — CRONOGRAMA.md 2.12 ✅
- [x] **lobby.rs** — Sistema completo de lobby e matchmaking para mesas cash e torneio
- [x] **Enums:** `GameType` (Cash/Tournament), `TableVisibility` (Public/Private), `PlayerLobbyStatus` (Lobby/Playing/Observing)
- [x] **Structs:** `TableInfo` (id, nome, tipo, blinds, buy-in, max_players, current_players, visibility, password_hash), `LobbyResult` (success, message, table_id)
- [x] **`LobbyManager`** com métodos: `new()`, `create_table()`, `list_tables()`, `list_tables_by_blinds()`, `list_available_tables()`, `find_table()`, `find_table_mut()`, `join_table()` (validações: existência, assento, saldo, senha), `leave_table()`, `close_table()`, `table_count()`, `total_players()`, `find_or_suggest_table()`
- [x] **28 testes unitários** cobrindo todos os fluxos (criação, listagem, filtros, entrada, saída, validações, senha, fechamento)
- [x] **Motor Rust 100% completo** — 10/10 módulos, 484/484 testes, 0 warnings

### 🌐 API Axum — REST + WebSocket (2026-07-10) — CRONOGRAMA.md 2.14 ✅
- [x] **Crate `10-API-Axum/`** — exposição HTTP/WS do motor para o frontend Dioxus
- [x] **Axum 0.7** com features `ws`, `macros` + tower-http (cors, trace) + sqlx 0.8 (postgres, uuid, chrono, migrate)
- [x] **8 endpoints REST públicos:**
  - `POST /auth/register` — registro com bcrypt + JWT
  - `POST /auth/login` — login com JWT (access + refresh)
  - `POST /auth/mfa/verify` — verificação TOTP (RFC 6238)
  - `POST /auth/refresh` — refresh token
  - `GET /lobby/tables` — listar mesas (filtros: blinds, disponibilidade)
  - `GET /lobby/tables/:id` — detalhes de mesa
  - `GET /tournament/:id` — info de torneio
  - `GET /health` — health check
- [x] **3 endpoints REST protegidos** (JWT via `RequireAuth` extractor):
  - `POST /lobby/join` — sentar em mesa
  - `POST /tournament/register` — registrar em torneio
  - `GET /hand-history/:hand_id` — replay de mão
- [x] **WebSocket `/ws/game/:table_id`** — canal de jogo em tempo real (ping/pong, get_table_info, JSON messages)
- [x] **JWT Middleware** (`middleware/auth.rs`) — `RequireAuth` extractor com `FromRequestParts`, valida token via `auth.validate_token(&token, "access")`
- [x] **Persistência PostgreSQL** — `sqlx::migrate!("./migrations")` + 6 tabelas (users, sessions, tables, hand_history, tournaments, tournament_players) + 4 índices
- [x] **AppState** (`state.rs`) — `db: PgPool`, `auth: Arc<Mutex<AuthManager>>`, `lobby: Arc<Mutex<LobbyManager>>`, `tournaments: Arc<Mutex<HashMap<String, TournamentStore>>>`, `jwt_secret: String`
- [x] **Error handling** (`error.rs`) — `ApiError` enum (BadRequest/Unauthorized/Forbidden/NotFound/Conflict/Internal) com `IntoResponse` + `From<sqlx::Error>` + `From<serde_json::Error>`
- [x] **CORS configurável** via env (`CORS_ORIGINS`)
- [x] **17 testes de integração** (`tests/api_tests.rs`) — 12 ativos passando + 5 `#[ignore]` (DB-dependent)
- [x] **Quality gates validados:** `cargo build` ✅, `cargo test` ✅ (12/12), `cargo clippy --all-targets -- -D warnings` ✅, `cargo build --release` ✅ (3m 51s, 0 warnings)
- [x] **CRONOGRAMA.md 2.14 ✅ Completo** — Motor Rust + API = 11/11 módulos, 496/496 testes

### 🛡️ Motor Rust — Antifraude (4 submódulos) (2026-07-10) — CRONOGRAMA.md 2.13 ✅
- [x] **antifraud/mod.rs** — Módulo raiz declarando 4 submódulos de detecção de fraude
- [x] **bot_detection.rs** — Detecção de bots via análise de padrões de timing, decisão e variância
- [x] **chip_dumping.rs** — Detecção de transferência ilícita de fichas entre jogadores
- [x] **collusion.rs** — Detecção de conluio entre múltiplos jogadores (padrões de aposta coordenados)
- [x] **multi_account.rs** — Detecção de múltiplas contas do mesmo jogador (device fingerprint, IP, padrões)
- [x] **Motor Rust 100% completo** — 10/10 módulos, 484/484 testes, 0 warnings

### 💰 Motor Rust — Conversão Monetária u64 → f64 (2026-07-07)
- [x] **Todos os campos monetários convertidos de u64 para f64** com truncamento a 2 casas decimais
- [x] **Função `truncar_2_casas(valor: f64) -> f64`** implementada em `utils.rs` (pública) e `loss_deflator.rs` (privada) — `(valor * 100.0).trunc() / 100.0`
- [x] **Tolerâncias de teste ajustadas** para precisão de f64 (0.01 em vez de f64::EPSILON)
- [x] **Valores esperados corrigidos** em `loss_deflator_tests.rs` para refletir truncamento real (ex: `122.49` em vez de `122.5`, `1.04` em vez de `1.05`)
- [x] **Compilação limpa** — 0 erros, 0 warnings
- [x] **484/484 testes passando** em `cargo test --lib` (~347s)

### 🦀 Stack Definitiva — Rust para TUDO (2026-07-03)
- [x] **Stack corrigida:** Rust para TUDO (motor de jogo + APIs + IA + dados + antifraude + autenticação + lobby + **front-end com Dioxus/WebAssembly**)
- [x] **Python removido** da stack alvo
- [x] **Go removido** da stack alvo
- [x] **TypeScript/React removido** da stack alvo — substituído por Rust (Dioxus/WebAssembly)
- [x] **MVP Node.js removido** — pasta `01-Plataforma-FullStack-NodeReact/` deletada, documentos legados (`README-Plataforma-FullStack.md`, `AUDITORIA_REGRAS.md`) deletados em 2026-07-08
- [x] `ARQUITETURA_MOTOR.md` atualizado para v3.1 refletindo stack Rust-only

### 🃏 Motor Rust — evaluate_hand Refatorado (2026-07-03)
- [x] `get_high_card()` extraída como helper dedicado (antes era inline no `evaluate_hand`)
- [x] Todas as 9 classificações de mão seguem padrão uniforme: cada uma delega a uma helper function
- [x] `evaluate_hand` agora é um orquestrador limpo com `if let Some(result) = ...` + `unreachable!()`
- [x] 18/18 testes passando em `deck.rs`

### 🧹 Motor Rust — 8 Warnings de Dead Code Limpos (2026-07-03)
- [x] `side_pots.rs`: campo `contributions` em `SidePotsResult` — `#[allow(dead_code)]`
- [x] `loss_deflator.rs`: `GamePhase` enum — `#[allow(dead_code)]` (variantes Preflop, Turn, River não usadas)
- [x] `loss_deflator.rs`: `get_heads_up_win_probability` — `#[allow(dead_code)]`
- [x] `loss_deflator.rs`: `evaluate_outcome`, `get_remaining_deck`, `create_full_deck`, `contains_card`, `combinations` — `#[allow(dead_code)]`
- [x] `cargo build` — **0 warnings**
- [x] `cargo test` — **47/47 passed**

### 🎲 Motor Rust — RNG Criptográfico (2026-07-04)
- [x] **rng_crypto.rs** — CSPRNG via `OsRng` (BCryptGenRandom no Windows, /dev/urandom no Linux)
- [x] `secure_shuffle()` — Fisher-Yates criptograficamente seguro (substitui `thread_rng` no deck.rs)
- [x] `secure_random_u32/u64` — números aleatórios com rejection sampling (sem bias de módulo)
- [x] `secure_random_f64` — float [0.0, 1.0) com 52 bits de mantissa
- [x] `secure_random_bool` — booleano com probabilidade configurável
- [x] `secure_random_bytes` — preenchimento de buffer criptográfico
- [x] **20 testes**, todos passando
- [x] **deck.rs integrado** — `shuffle_deck` agora usa `csprng()` em vez de `thread_rng()`

### 📜 Motor Rust — Hand History (2026-07-04)
- [x] **hand_history.rs** — Registro de cada mão jogada para fins de auditoria, histórico de jogador e replay
- [x] **Modelagem estruturada:** Enums e Structs para representar fases (`GamePhase`), ações (`Action`, `PlayerAction`), resultados (`PlayerResult`), configurações de mesa (`TableConfig`) e histórico consolidado (`HandHistory`)
- [x] **Serialização Universal:** Totalmente compatível com `serde` e `serde_json` (com nomes lowercase) para persistência em banco de dados ou tráfego de rede
- [x] **API robusta de consulta:** Funções para recuperar ações por jogador ou fase, calcular aposta total de um jogador na mão, identificar o vencedor da mão, formatar um resumo legível por humanos, etc.
- [x] **19 testes unitários e de integração** cobrindo todos os fluxos de jogo e serialização JSON
- [x] **Integração com deck.rs:** Reutilização das estruturas nativas `Card` e `HandResult` (agora com suporte à serialização)

### 🧩 Motor Rust — Módulos Implementados (2026-07-02)
- [x] **deck.rs** — Criação, embaralhamento, avaliação de mãos Texas Hold'em (18 testes)
- [x] **side_pots.rs** — Calculadora de side pots para all-in múltiplos jogadores (7 testes)
- [x] **loss_deflator.rs** — Cashback progressivo para perdedores de all-in (9 testes)
- [x] **rake.rs** — Rake da casa 2.5% default, cap R$6 (13 testes)
- [x] **main.rs** — Demo integrando todos os módulos
- [x] **Total na época: 47 testes** (posteriormente expandido para 484 com tournament_engine, auth, rng_crypto, hand_history, antifraude)

---

### 🛡️ Motor Rust — Antifraude (4 submódulos) (2026-07-10)
- [x] **antifraud/mod.rs** — Módulo raiz declarando 4 submódulos de detecção de fraude
- [x] **bot_detection.rs** — Detecção de bots via análise de padrões de timing, decisão e variância
- [x] **chip_dumping.rs** — Detecção de transferência ilícita de fichas entre jogadores
- [x] **collusion.rs** — Detecção de conluio entre múltiplos jogadores (padrões de aposta coordenados)
- [x] **multi_account.rs** — Detecção de múltiplas contas do mesmo jogador (device fingerprint, IP, padrões)
- [x] **CRONOGRAMA.md 2.13 ✅ Completo** — Motor Rust 100% (10/10 módulos)

### 🎪 Motor Rust — Lobby + Matchmaking (2026-07-10)
- [x] **lobby.rs** — Sistema completo de lobby e matchmaking para mesas cash e torneio
- [x] **Enums:** `GameType` (Cash/Tournament), `TableVisibility` (Public/Private), `PlayerLobbyStatus` (Lobby/Playing/Observing)
- [x] **Structs:** `TableInfo` (id, nome, tipo, blinds, buy-in, max_players, current_players, visibility, password_hash), `LobbyResult` (success, message, table_id)
- [x] **`LobbyManager`** com métodos: `new()`, `create_table()`, `list_tables()`, `list_tables_by_blinds()`, `list_available_tables()`, `find_table()`, `find_table_mut()`, `join_table()` (validações: existência, assento, saldo, senha), `leave_table()`, `close_table()`, `table_count()`, `total_players()`, `find_or_suggest_table()`
- [x] **28 testes unitários** cobrindo todos os fluxos (criação, listagem, filtros, entrada, saída, validações, senha, fechamento)
- [x] **CRONOGRAMA.md 2.12 ✅ Completo** — Motor Rust 100% (10/10 módulos)

### 🏆 Motor Rust — Tournament Engine (2026-07-04)
- [x] **tournament_engine.rs** — Engine completo de torneios com blinds, prizes, rebuy, addon, late registration
- [x] **Estruturas:** `TournamentConfig`, `BlindLevel`, `TournamentState`, `PlayerTournamentEntry`, `TournamentResult`, `TournamentStats`
- [x] **Enums:** `TournamentStatus` (Registering/Running/Paused/Finished/Cancelled), `TournamentSpeed` (Turbo/Normal/Slow)
- [x] **Funções:** `create_tournament`, `register_player`, `start_tournament`, `advance_blinds`, `eliminate_player`, `finish_tournament`, `cancel_tournament`, `process_rebuy`, `process_addon`, `pause_tournament`, `resume_tournament`, `get_tournament_stats`
- [x] **19 testes unitários** cobrindo todos os fluxos (registro, late registration, blinds, eliminação, premiação, rebuy, addon, pausa, cancelamento, JSON)
- [x] **Serialização JSON** completa via serde (rename_all = lowercase)

### 🔐 Motor Rust — Autenticação (JWT + MFA) (2026-07-04)
- [x] **auth.rs** — Sistema completo de autenticação e gestão de sessões
- [x] **JWT Manual** — Implementação de Encode/Decode/Verify usando `hmac` e `sha2` (evitando dependências de C complexas no Windows)
- [x] **Segurança de Senhas** — Hash com `bcrypt` (cost 12)
- [x] **MFA/TOTP** — Implementação completa do RFC 6238 para autenticação de dois fatores
- [x] **Gestão de Sessões** — Fluxos de registro, login, refresh token, lockout de conta e invalidação de sessão
- [x] **153 testes unitários e de integração** cobrindo todos os cenários de segurança e expiração de tokens
- [x] **Serialização JSON** via `serde` com `rename_all = "snake_case"`

## 🔴 Em Andamento — Desenvolvimento Ativo da Plataforma
- [x] **3.5 Roteamento Dioxus** ✅ 2026-07-11 — `dioxus-router` 0.6, 4 rotas (Home/Login/Lobby/Table), Navbar persistente, 2 testes, 0 warnings
- [x] **3.6 Componentes de Mesa** ✅ 2026-07-11 — 7 componentes Dioxus (card, avatar, pot, community_cards, action_buttons, seat, table) + refatoração de `pages/table.rs` usando `TableView` com mock data. 22 testes unitários novos. `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (24/24), `cargo build --release` ✅ (0 warnings)
- [x] **3.7 Componentes de Lobby** ✅ 2026-07-12 — 5 componentes Dioxus (table_card, lobby_filters, join_button, player_count, lobby_list) + refatoração de `pages/lobby.rs` usando `LobbyFilters` + `LobbyList` com mock data. 34 testes unitários novos (7+9+5+8+5). `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (57/57 frontend), `cargo build --release` ✅ (0 warnings)
- [x] **3.8 Componentes de Auth** ✅ 2026-07-12 — 3 componentes Dioxus (login_form, register_form, mfa_input) + 2 páginas atualizadas (login.rs com AuthFlow enum, register.rs). CSS puro Full Tilt Poker (~100 linhas). `cargo clippy -- -D warnings` ✅ (0 warnings), `cargo test` ✅ (61/61 frontend)
- [x] **3.9 Integração API ↔ Front** ✅ 2026-07-12 — `api_client.rs` (HTTP via gloo-net: register/login/refresh/logout/health + token storage via web-sys localStorage) + `ws_client.rs` (WebSocket via ws_stream_wasm + gloo-net, com WsCallbacks FnMut + WsConnectionState + ClientMessage/ServerMessage serde). `pages/login.rs` e `pages/register.rs` agora chamam API real (não mais mock). `pages/table.rs` conecta ao WebSocket real. Quality gate do CI: `cargo clippy --all-targets -- -D warnings` ✅ (0 warnings), `cargo test` ✅ (73/73 frontend). 3 testes de localStorage guardados com `#[cfg(target_arch = "wasm32")]` (web-sys não funciona em testes nativos).

---

## ⏳ Próximas Tarefas — Backlog da Plataforma de Poker

### 🔴 Prioridade Crítica — Gaps da Auditoria QUALITY.md (2026-07-08)
- [x] **CI/CD GitHub Actions** — workflow com `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, `cargo audit` (QUALITY.md Seção 5.5, Fase 1) ✅ 2026-07-08
- [x] **API Axum (HTTP)** — expor o motor via REST/WebSocket para o frontend (QUALITY.md Seção 2) ✅ 2026-07-08
- [x] **Persistência PostgreSQL** — schema, migrations, `sqlx` (QUALITY.md Seção 6) ✅ 2026-07-08
- [x] **Integração motor ↔ frontend** — frontend Dioxus chamando API do motor ✅ 2026-07-12 (Task 3.9)
- [x] **`cargo audit`** — instalar e rodar, 0 CVEs (QUALITY.md Seção 18.5.1) ✅ 2026-07-08 (job `audit` no CI)
- [ ] **Cobertura diferenciada por criticidade** — `cargo-tarpaulin` ou `cargo-llvm-cov` (QUALITY.md Seção 3.1). Metas: ≥ 98% módulos críticos (financeiro, segurança, motor de jogo) / ≥ 95% API / ≥ 90% frontend

### 🟡 Prioridade Alta — Infraestrutura e Integração
- [ ] **Lobby** — `lobby.rs` existe mas sem integração com API HTTP
- [ ] **Frontend Dioxus** — telas reais (login, lobby, mesa, torneio)
- [ ] **HTTPS/TLS na infraestrutura** — `docker-compose.yml` não tem reverse proxy (nginx/Caddy) com TLS termination. `render.yaml` desatualizado (ainda referencia Node.js). QUALITY.md §2.4 prevê rustls (TLS 1.3) como planejado. Para poker online com senhas/JWT/saldos, HTTPS é obrigatório, não opcional. Gap de infraestrutura (não de código de aplicação — Axum escuta HTTP corretamente, TLS fica no reverse proxy).

### 🟠 Prioridade Média — Segurança e Compliance
- [ ] OWASP ZAP scan — 0 alertas high/critical (QUALITY.md Seção 18.5.1)
- [ ] Trivy container scan — 0 vulnerabilidades high
- [ ] gitleaks — 0 secrets no código

### 🔵 Prioridade Baixa — Fase 2+ (KYC, PIX, Jogo Responsável)
- [ ] KYC obrigatório antes de depositar
- [ ] Depósito/saque via PIX
- [ ] Self-exclusion e limites de depósito
- [ ] Reality check pop-up (jogo responsável)

---

## 📊 Métricas — Indicadores da Plataforma de Poker

| Item                          | Valor                                                                                                                              |
|-------------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| Regras de negócio documentadas | 45                                                                                                                                 |
| Regras implementadas corretamente | 43 (95%)                                                                                                                       |
| Bugs corrigidos               | 2/2 críticos                                                                                                                       |
| Módulos Rust implementados    | **10** (deck, side_pots, loss_deflator, rake, rng_crypto, hand_history, tournament_engine, auth, lobby, utils) + 4 antifraude + 1 API + 15 componentes frontend (7 mesa + 5 lobby + 3 auth) |
| Módulos antifraude            | **4** (collusion, chip_dumping, bot_detection, multi_account) — 2.13 ✅ Completo                                                   |
| API Axum                      | **✅ Implementada** (Axum 0.7, 8 endpoints REST + WebSocket + JWT middleware, 12 testes ativos) — 2.14 ✅ Completo                |
| Testes Rust                   | **567/567 passando** (484 motor + 12 API + 71 frontend), 0 warnings
| Funcionalidades pendentes (Rust) | 0 (Motor + API 100%)                                                                                                            |
| Stack atual                   | **Rust puro** (Dioxus no front-end, Axum/Rust no back-end)                                                                         |
| CI/CD                         | ✅ Implementado (10 jobs: check, test, clippy, fmt, audit, frontend-check + api-check, api-test, api-clippy, api-fmt)              |
| Persistência                  | ✅ Implementada (PostgreSQL 15 + sqlx 0.8, 6 tabelas, migrations)                                                                  |
| API HTTP                      | ✅ Implementada (Axum 0.7, 8 endpoints REST + WebSocket)                                                                          |
| HTTPS/TLS                     | ❌ Não implementado (gap de infraestrutura — sem reverse proxy no docker-compose, render.yaml desatualizado)                       |

---

## 📁 Estrutura Atual — Árvore de Módulos da Plataforma

```
OneDrive/Projetos/Poker_Project/
├── QUALITY.md                           ✅ Documento mestre (17 seções + 8-BIS)
├── README.md                            ✅ Atualizado (2026-07-08) — stack 100% Rust
├── 04-Infraestrutura-Docker/            ✅
│   ├── docker-compose.yml               (PostgreSQL 15 + Redis 7 + Kafka + Zookeeper)
│   ├── render.yaml
│   └── ...
├── 05-Documentacao/                     ✅
│   ├── BUSINESS_RULES.md
│   ├── DEVELOPMENT_LOG.md
│   ├── STATUS.md                        ← ATUALIZADO (2026-07-08)
│   ├── DASHBOARD.md
│   ├── CRONOGRAMA.md
│   ├── ESTRATEGIA_APRENDIZADO.md
│   └── PARAMETROS_ESTUDO.md
├── 07-Arquitetura-Motor/                ✅ (ARQUITETURA_MOTOR.md)
├── 08-Motor-Rust/                       ✅ ATIVO — Motor de jogo (10 módulos + 4 antifraude, 484 testes)
│   └── src/
│       ├── deck.rs                      (18 testes) ✅
│       ├── side_pots.rs                 (7 testes) ✅
│       ├── loss_deflator.rs             (9 testes) ✅
│       ├── rake.rs                       (13 testes) ✅
│       ├── rng_crypto.rs                (20 testes) ✅
│       ├── hand_history.rs              (19 testes) ✅
│       ├── tournament_engine.rs         (19 testes) ✅
│       ├── auth.rs                      (153 testes) ✅
│       ├── lobby.rs                     (28 testes) ✅ — 2.12 Completo
│       ├── antifraud/
│       │   ├── collusion.rs             ✅ — 2.13 Completo
│       │   ├── chip_dumping.rs          ✅ — 2.13 Completo
│       │   ├── bot_detection.rs         ✅ — 2.13 Completo
│       │   ├── multi_account.rs         ✅ — 2.13 Completo
│       │   └── mod.rs
│       ├── utils.rs                     ✅ (truncar_2_casas)
│       ├── types.rs                     ✅
│       ├── main.rs                      (demo integrado)
│       └── tests/                       (integration, property, motor, antifraud, loss_deflator)
├── 09-Frontend-Dioxus/                  ✅ ATIVO — Front-end WebAssembly (Dioxus 0.6)
│   └── src/
│       ├── main.rs                      (entry point + launch(app))
│       ├── router.rs                    (4 rotas: Home/Login/Lobby/Table + Navbar)
│       ├── components/                  ✅ 15 componentes (7 Mesa 3.6 + 5 Lobby 3.7 + 3 Auth 3.8)
│       │   ├── mod.rs                   (declaração dos 15 submódulos)
│       │   ├── card.rs                  (Carta face/verso, 5 testes)
│       │   ├── avatar.rs                (Avatar/Jogador, 3 testes)
│       │   ├── pot.rs                   (Pote central, 3 testes)
│       │   ├── community_cards.rs       (Cartas comunitárias, 3 testes)
│       │   ├── action_buttons.rs        (Fold/Check/Call/Raise/All-in, 3 testes)
│       │   ├── seat.rs                  (Assento com posição absoluta, 2 testes)
│       │   ├── table.rs                 (Mesa oval completa, 3 testes)
│       │   ├── table_card.rs            (Card de mesa individual, 7 testes) — 3.7
│       │   ├── lobby_filters.rs         (Filtros tipo de jogo + blinds, 9 testes) — 3.7
│       │   ├── join_button.rs           (Botão entrar Available/Full/Spectating, 5 testes) — 3.7
│       │   ├── player_count.rs          (Indicador X/Y + barra progresso, 8 testes) — 3.7
│       │   └── lobby_list.rs            (Lista de mesas combinando componentes, 5 testes) — 3.7
│       └── pages/
│           ├── home.rs                  (Home)
│           ├── login.rs                 (Login — AuthFlow enum com Login/MfaRequired/Success/Error)
│           ├── register.rs              (Register — formulário de registro com validação)
│           ├── lobby.rs                 (Lobby)
│           └── table.rs                 (Table — usa TableView com mock data)
├── 10-API-Axum/                         ✅ ATIVO — API HTTP/WS (Axum 0.7, 8 endpoints REST + WebSocket + JWT)
│   ├── Cargo.toml                       (axum 0.7, sqlx 0.8, tower-http 0.6, tokio 1)
│   ├── src/
│   │   ├── lib.rs                       (build_router com 11 rotas)
│   │   ├── main.rs                      (entry point + PgPool + migrations + CORS)
│   │   ├── state.rs                     (AppState: db, auth, lobby, tournaments, jwt_secret)
│   │   ├── error.rs                     (ApiError + IntoResponse)
│   │   ├── tournament_store.rs          (TournamentStore wrapper)
│   │   ├── handlers/
│   │   │   ├── auth.rs                  (register, login, mfa_verify, refresh)
│   │   │   ├── lobby.rs                 (list_tables, join_table, get_table)
│   │   │   ├── tournament.rs            (register_player, get_tournament)
│   │   │   ├── hand_history.rs          (get_hand_history)
│   │   │   ├── websocket.rs             (game_websocket + handle_game_socket)
│   │   │   └── mod.rs
│   │   └── middleware/
│   │       ├── auth.rs                  (RequireAuth extractor + AuthUser)
│   │       └── mod.rs
│   ├── migrations/
│   │   └── 001_initial_schema.sql       (6 tabelas + 4 índices)
│   └── tests/
│       └── api_tests.rs                 (17 testes: 12 ativos + 5 #[ignore] DB)
├── scripts/
│   ├── coverage.ps1
│   └── coverage.sh
└── .git/
```

---

## 🔄 Histórico de Atualizações — Evolução da Plataforma

| Data       | Mudança                                                                                                                                                                                                                                                                                                                                 |
|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 2026-07-11 | **3.6 Componentes de Mesa CONCLUÍDO:** 7 componentes Dioxus criados em `09-Frontend-Dioxus/src/components/`: `card.rs` (Carta face/verso com Suit/Rank enums, 5 testes), `avatar.rs` (Avatar/Jogador com Position/PlayerStatus enums, 3 testes), `pot.rs` (Pote central com PotEntry struct, 3 testes), `community_cards.rs` (Cartas comunitárias com CommunityStage enum, 3 testes), `action_buttons.rs` (Fold/Check/Call/Raise/All-in com ActionKind enum, 3 testes), `seat.rs` (Assento com posição absoluta via SeatPosition struct, 2 testes), `table.rs` (Mesa oval completa integrando todos os componentes com PlayerData struct + mock helpers, 3 testes). `pages/table.rs` refatorado para usar `TableView` com mock data (5 jogadores + 3 cartas comunitárias). `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (24/24), `cargo build --release` ✅ (0 warnings). **Total: 520/520 testes (484 motor + 12 API + 24 frontend).** Próximo: 3.7 Componentes de Lobby. |
| 2026-07-11 | **3.5 Roteamento Dioxus CONCLUÍDO:** `dioxus-router` 0.6 com 4 rotas (`/` Home, `/login` Login, `/lobby` Lobby, `/table/:id` Table). `router.rs` com enum `Route` (Routable derive), `Root()` com Navbar persistente + `Router::<Route>`. 4 módulos de página (`pages/home.rs`, `pages/login.rs`, `pages/lobby.rs`, `pages/table.rs`) com componentes `Home`, `Login`, `Lobby`, `Table` (PascalCase + `#[allow(non_snake_case)]` para compatibilidade com macro `Routable`). 2 testes (`test_route_variants`, `test_route_clone_eq`). `cargo check` ✅, `cargo clippy -- -D warnings` ✅, `cargo test` ✅ (2/2), `cargo build --release` ✅ (0 warnings). Próximo: 3.6 Componentes de Mesa. |
| 2026-07-10 | **API Axum concluída (2.14):** Crate `10-API-Axum/` com Axum 0.7 — 8 endpoints REST públicos (auth/register, auth/login, auth/mfa/verify, auth/refresh, lobby/tables, lobby/tables/:id, tournament/:id, health) + 3 endpoints protegidos (lobby/join, tournament/register, hand-history/:hand_id) + WebSocket `/ws/game/:table_id`. JWT middleware via `RequireAuth` extractor. Persistência PostgreSQL via sqlx 0.8 + migration `001_initial_schema.sql` (6 tabelas). 17 testes de integração (12 ativos passando + 5 `#[ignore]` DB-dependent). `cargo clippy --all-targets -- -D warnings` ✅, `cargo build --release` ✅ (3m 51s, 0 warnings). **Motor Rust + API = 11/11 módulos, 496/496 testes, 0 warnings.** Próximo: 3.5 Roteamento Dioxus. |
| 2026-07-10 | **Lobby + Antifraude concluídos (2.12 + 2.13):** `lobby.rs` (28 testes) com `LobbyManager`, `GameType`, `TableVisibility`, `PlayerLobbyStatus`, `TableInfo`, `LobbyResult` — create_table, list_tables, join_table (validações: existência, assento, saldo, senha), leave_table, find_or_suggest_table. `antifraud/` com 4 submódulos (bot_detection, chip_dumping, collusion, multi_account). **Motor Rust 100% completo (10/10 módulos, 484/484 testes, 0 warnings).** Próximo: 2.14 API Axum. |
| 2026-07-09 | **Modernização completa de QUALITY.md:** Todos os 259 headers modernizados com emojis + contexto de poker — 18 títulos top-level (#), 112 sub-seções (## N.M), 129 sub-sub-seções (### N.M.K). Anomalia estrutural 8-BIS resolvida (114 renumerações), ## 11.1 duplicado corrigido, referências cruzadas atualizadas. Documento mestre agora reflete 100% a essência do negócio de poker online. |
| 2026-07-08 | **Passo 3 — API Axum + Persistência PostgreSQL CONCLUÍDO:** Crate `10-API-Axum` implementado com Axum 0.7 (8 endpoints REST: auth/register, auth/login, lobby/tables, lobby/join, tournament/create, tournament/list, hand-history, WebSocket + JWT middleware). Persistência via sqlx 0.8 + PostgreSQL 15 com migration `001_initial_schema.sql` (6 tabelas: users, sessions, tables, table_players, tournaments, tournament_entries). 17 testes de integração (12 não-DB passando, 5 DB `#[ignore]`). CI atualizado com 4 jobs adicionais (api-check, api-test, api-clippy, api-fmt). `cargo clippy -D warnings` limpo, `cargo fmt` limpo. |
| 2026-07-08 | **Gap de HTTPS/TLS registrado:** `docker-compose.yml` sem reverse proxy (nginx/Caddy) com TLS termination. `render.yaml` desatualizado (referencia Node.js, não Rust). QUALITY.md §2.4 prevê rustls (TLS 1.3) como planejado. Axum escuta HTTP corretamente (TLS fica na camada de infraestrutura). Gap classificado como Prioridade Alta no backlog. |
| 2026-07-08 | **CI/CD GitHub Actions implementado:** Workflow `.github/workflows/rust-ci.yml` com 6 jobs — `check`, `test` (484 testes), `clippy` (-D warnings), `fmt` (--check), `audit` (cargo audit --deny warnings, CVE scan), `frontend-check` (Dioxus/wasm32-unknown-unknown: check + clippy + fmt). Triggers em push/PR para main/master quando `08-Motor-Rust/**` ou `09-Frontend-Dioxus/**` mudam. `cargo fmt` executado em todo o código do motor para alinhar com o CI. |
| 2026-07-08 | **Auditoria QUALITY.md:** STATUS.md corrigido — removidas referências a arquivos deletados (`AUDITORIA_REGRAS.md`, `README-Plataforma-FullStack.md`), contagem de testes unificada (484), estrutura de pastas atualizada (8 módulos + 4 antifraude + tests/), backlog reescrito com gaps críticos da auditoria (CI/CD, API Axum, persistência), métricas atualizadas com indicadores ❌ para gaps |
| 2026-07-07 | **Conversão monetária u64 → f64:** Todos os campos monetários convertidos com truncamento a 2 casas decimais. **484/484 testes passando**                                                                                                                                                                                              |
| 2026-07-04 | **Hand History em Rust:** Módulo `hand_history.rs` criado e integrado com serialização JSON e 19 testes (87/87 passando)                                                                                                                                                                                                              |
| 2026-07-04 | **RNG Criptográfico em Rust:** Módulo `rng_crypto.rs` com CSPRNG seguro (20 testes ✅), integrado ao deck                                                                                                                                                                                                                              |
| 2026-07-03 | **Stack definitiva:** Rust para TUDO (Python/Go removidos). Motor Rust com 47/47 testes, 0 warnings                                                                                                                                                                                                                                   |
| 2026-07-03 | **evaluate_hand refatorado** — HighCard extraído para helper, padrão uniforme                                                                                                                                                                                                                                                         |
| 2026-07-03 | **8 warnings de dead code limpos** — side_pots.rs + loss_deflator.rs                                                                                                                                                                                                                                                                  |
| 2026-07-02 | Motor Rust: 4 módulos implementados (deck, side_pots, loss_deflator, rake) — 47 testes                                                                                                                                                                                                                                                |
| 2026-06-30 | **Mudança de stack alvo:** Rust (backend) + Python (IA/dados) + TypeScript (front)                                                                                                                                                                                                                                                   |
| 2026-06-30 | Loss Deflator Progressivo implementado (15% / 25% / 35%)                                                                                                                                                                                                                                                                               |
| 2026-06-28 | Side Pots implementado (all-in múltiplos jogadores)                                                                                                                                                                                                                                                                                   |
| 2026-06-27 | Reorganização completa: pastas renomeadas, duplicatas removidas, docs consolidados                                                                                                                                                                                                                                                    |
| 2026-06-27 | Criação do `ARQUITETURA_MOTOR.md` (stack Rust/Go/Python/TS)                                                                                                                                                                                                                                                                            |
| 2026-06-27 | Criação do `DASHBOARD.md` (painel de controle)                                                                                                                                                                                                                                                                                         |
| 2026-06-27 | Regra de ouro estabelecida e salva em memória persistente                                                                                                                                                                                                                                                                              |
| 2026-06-25 | Deflator de perda definido (fórmula baseada em odds)                                                                                                                                                                                                                                                                                  |
| 2026-06-25 | Criação do documento de status                                                                                                                                                                                                                                                                                                         |
| 2026-06-25 | Correção dos bugs críticos (Check + Split Pot)                                                                                                                                                                                                                                                                                        |
| 2026-06-25 | Criação da auditoria de regras                                                                                                                                                                                                                                                                                                         |
| 2026-06-25 | Criação do BUSINESS_RULES.md                                                                                                                                                                                                                                                                                                          |
| 2026-06-25 | Organização completa das pastas                                                                                                                                                                                                                                                                                                        |
