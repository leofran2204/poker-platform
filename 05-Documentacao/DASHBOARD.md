# 🎯 Painel de Controle — Plataforma de Poker Online

**Atualizado:** 2026-07-12 | **Sprint atual:** S03 — Motor de Poker em Rust + Front-end Dioxus

> ⚠️ **REGRA DE OURO:** Antes de codar, consultar `07-Arquitetura-Motor/ARQUITETURA_MOTOR.md` e `05-Documentacao/BUSINESS_RULES.md`.
> 📐 A spec viva (SDD) está em `STATUS.md` — lá está o que foi implementado vs regras de negócio.
> 📅 O cronograma completo está em `CRONOGRAMA.md` — veja prazos, fases e % de conclusão.
> 🦀 **Stack definitiva:** Rust para TUDO (backend, APIs, IA, dados, antifraude, autenticação, lobby, motor de jogo **e front-end com Dioxus/WebAssembly**). ❌ Sem TypeScript/React. ❌ Sem Python. ❌ Sem Go. ❌ Sem Node.js (MVP legado deletado em 2026-07-08).

---

## 📅 Cadência de Sprints — Metodologia Ágil

| # | Parâmetro | Valor |
|---|-----------|-------|
| 1 | **Duração** | 2 semanas (14 dias) |
| 2 | **Sprint atual** | S03 (2026-07-04 → 2026-07-17) |
| 3 | **Próximo sprint** | S04 (2026-07-18 → 2026-07-31) |
| 4 | **Cerimônias** | Planning (dia 1) + Review + Retrospectiva (dia 14) |
| 5 | **Retrospectivas** | Registradas em `DEVELOPMENT_LOG.md` |

### 🏁 Definition of Done (DoD) — Critério de "Pronto"

Uma tarefa só está **completa** quando TODOS os critérios abaixo são atendidos:

| # | Critério | Verificação |
|---|----------|-------------|
| 1 | **Código compila sem erros** | `cargo build` — 0 erros |
| 2 | **Zero warnings** | `cargo build` — 0 warnings |
| 3 | **Todos os testes passam** | `cargo test --lib` — 0 falhas |
| 4 | **Cobertura de testes** | Novo código tem testes unitários |
| 5 | **Documentação atualizada** | `STATUS.md` + `DASHBOARD.md` + `CRONOGRAMA.md` |
| 6 | **Regras de negócio respeitadas** | Conforme `BUSINESS_RULES.md` |
| 7 | **Padrões de qualidade** | Conforme `QUALITY.md` |
| 8 | **Sem regressões** | Testes existentes continuam passando |

> **Regra:** Se qualquer critério do DoD não for atendido, a tarefa **NÃO** está pronta. Não existe "quase pronto".

### 📊 Histórico de Sprints

| Sprint | Período | Objetivo | Entregue | Status |
|--------|---------|---------|----------|-------|
| S01 | 2026-06-25 → 2026-07-02 | Fundação + Motor Rust (4 módulos) | 10 marcos F1 + 4 módulos (47 testes) | ✅ Concluído |
| S02 | 2026-07-03 → 2026-07-07 | Stack Rust-only + 4 módulos + Dioxus | Tournament, Hand History, RNG, Auth (484 testes) + Dioxus esqueleto | ✅ Concluído |
| S03 | 2026-07-04 → 2026-07-17 | Componentes Dioxus + Lobby + Antifraude + API Axum | Lobby (2.12, 28 testes) + Antifraude (2.13, 4 submódulos) + API Axum (2.14, 12 testes) + Roteamento Dioxus (3.5, 2 testes) + Componentes de Mesa (3.6, 22 testes) + Componentes de Lobby (3.7, 34 testes) + Componentes de Auth (3.8, login/registro/MFA) concluídos. Motor + API + Frontend = 11/11 módulos, 61 testes frontend. Pendente: Integração API ↔ Front (3.9) | 🔄 Ativo |

---

## 📋 Tarefas por Status — Sprint Atual

### 🔴 Em Andamento — Desenvolvimento Ativo
| #   | Tarefa                                                                                                      | Pasta                     | Prioridade |
|-----|-------------------------------------------------------------------------------------------------------------|---------------------------|------------|
| —   | **Front-end Dioxus** — Roteamento (3.5) + Componentes de Mesa (3.6) + Componentes de Lobby (3.7) + Componentes de Auth (3.8) concluídos. Próximo: Integração API ↔ Front (3.9)                       | `09-Frontend-Dioxus/src/` | 🔴 Alta    |

### 🟡 Próximas Tarefas — Backlog Priorizado
| #   | Tarefa                              | Pasta                     | Prioridade |
|-----|-------------------------------------|---------------------------|------------|
| 3.9 | **Integração API ↔ Front** — chamadas HTTP/WS | `09-Frontend-Dioxus/src/` | 🔴 Alta    |
| 4   | **Dockerfiles** para Rust + Dioxus  | `04-Infraestrutura-Docker/` | 🟡 Média   |
| 5   | **CI/CD (GitHub Actions)**          | `04-Infraestrutura-Docker/` | 🟡 Média   |
| 6   | **HTTPS/TLS** — reverse proxy (nginx/Caddy) | `04-Infraestrutura-Docker/` | 🟡 Média   |

### ✅ Concluídas — Sprint Atual
| #   | Tarefa                                                                                                                                                              | Data       |
|-----|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------|
| 3.8 | **Componentes de Auth** — 3 componentes Dioxus em `09-Frontend-Dioxus/src/components/`: `login_form.rs` (formulário de login com username/senha, props title/submit_label/on_submit), `register_form.rs` (formulário de registro com username/email/senha/confirmar senha, props title/submit_label/on_submit), `mfa_input.rs` (input de 6 dígitos TOTP, props title/instruction/on_submit). 3 páginas: `pages/login.rs` (AuthFlow enum com Login/MfaRequired/Success/Error, validação client-side, navegação para lobby), `pages/register.rs` (validação de senha, confirmação, navegação para login), `pages/login.rs` com `mfa_required` state. CSS puro (~100 linhas) em `assets/index.html` com visual Full Tilt Poker. `cargo clippy -- -D warnings` ✅ (0 warnings), `cargo test` ✅ (61/61). | 2026-07-12 |
| 3.7 | **Componentes de Lobby** — 5 componentes Dioxus em `09-Frontend-Dioxus/src/components/`: `table_card.rs` (card de mesa com GameType, blinds, ocupação, 7 testes), `lobby_filters.rs` (filtros por tipo de jogo + range de blinds, 9 testes), `join_button.rs` (botão Entrar/Cheia/Assistir, 5 testes), `player_count.rs` (contador visual X/Y com barra de progresso, 8 testes), `lobby_list.rs` (lista combinando TableCard + PlayerCount + JoinButton, 5 testes). CSS puro (~250 linhas) em `assets/index.html` com visual Full Tilt Poker (dark felt #1a3a1a, gold #8b6914). Tailwind CDN removido. `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (57/57). Doctest `utils.rs:34` corrigido (`&Vec<f64>` → `&[Pot]`). Motor: 484/484 testes ✅ | 2026-07-12 |
| —   | **Componentes de Mesa (3.6)** — 7 componentes Dioxus em `09-Frontend-Dioxus/src/components/`: `card.rs` (Carta face/verso, Suit/Rank enums, 5 testes), `avatar.rs` (Avatar/Jogador, Position/PlayerStatus enums, 3 testes), `pot.rs` (Pote central, PotEntry struct, 3 testes), `community_cards.rs` (Cartas comunitárias, CommunityStage enum, 3 testes), `action_buttons.rs` (Fold/Check/Call/Raise/All-in, ActionKind enum, 3 testes), `seat.rs` (Assento com posição absoluta, SeatPosition struct, 2 testes), `table.rs` (Mesa oval completa integrando todos, PlayerData struct + mock helpers, 3 testes). `pages/table.rs` refatorado para usar `TableView` com mock data. `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (24/24), `cargo build --release` ✅ (0 warnings) | 2026-07-11 |
| —   | **Roteamento Dioxus (3.5)** — `dioxus-router` 0.6 com 4 rotas (`/` Home, `/login` Login, `/lobby` Lobby, `/table/:id` Table). `router.rs` com enum `Route` (Routable derive), `Root()` com Navbar persistente + `Router::<Route>`. 4 módulos de página (`pages/home.rs`, `pages/login.rs`, `pages/lobby.rs`, `pages/table.rs`) com componentes `Home`, `Login`, `Lobby`, `Table` (PascalCase + `#[allow(non_snake_case)]` para compatibilidade com macro `Routable`). 2 testes (`test_route_variants`, `test_route_clone_eq`). `cargo check` ✅, `cargo clippy -- -D warnings` ✅, `cargo test` ✅ (2/2), `cargo build --release` ✅ (0 warnings) | 2026-07-11 |
| —   | **API Axum (2.14)** — Axum 0.7 com 8 endpoints REST públicos (auth/register, auth/login, auth/mfa/verify, auth/refresh, lobby/tables, lobby/tables/:id, tournament/:id, health) + 3 endpoints protegidos (lobby/join, tournament/register, hand-history/:hand_id) + WebSocket `/ws/game/:table_id`. JWT middleware via `RequireAuth` extractor. Persistência PostgreSQL via sqlx 0.8 + migration `001_initial_schema.sql` (6 tabelas). 17 testes de integração (12 ativos passando + 5 `#[ignore]` DB-dependent). `cargo clippy --all-targets -- -D warnings` ✅, `cargo build --release` ✅ (3m 51s, 0 warnings) | 2026-07-10 |
| —   | **Lobby + Matchmaking (2.12)** — `lobby.rs` (28 testes): `LobbyManager`, `GameType`, `TableVisibility`, `PlayerLobbyStatus`, `TableInfo`, `LobbyResult` — create/list/join/leave/find_or_suggest | 2026-07-10 |
| —   | **Antifraude (2.13)** — `antifraud/` com 4 submódulos: `bot_detection`, `chip_dumping`, `collusion`, `multi_account`                                              | 2026-07-10 |
| —   | **Motor Rust 100% completo** — 10/10 módulos, 484/484 testes, 0 warnings                                                                                            | 2026-07-10 |
| —   | **Conversão monetária u64 → f64** — todos os campos monetários convertidos com truncamento a 2 casas decimais via `truncar_2_casas()`, **484/484 testes passando** | 2026-07-07 |
| —   | **Auth (JWT + MFA)** — autenticação completa (JWT manual, bcrypt, TOTP), 153 testes                                                                                | 2026-07-04 |
| —   | **Tournament Engine** — módulo `tournament_engine.rs` (blinds, prizes, rebuy, addon, late registration, 19 testes)                                                | 2026-07-04 |
| —   | **Hand History** — módulo `hand_history.rs` (JSON serializable, tracking total de jogadas/showdowns, 19 testes)                                                   | 2026-07-04 |
| —   | **RNG Criptográfico** — módulo `rng_crypto.rs` (CSPRNG via OsRng, 20 testes, integrado ao deck.rs)                                                                | 2026-07-04 |
| —   | **CRONOGRAMA.md criado** — cronograma completo com fases, prazos e % de conclusão                                                                                | 2026-07-04 |
| —   | **Front-end Dioxus criado** — projeto compilando (Dioxus 0.6, WebAssembly)                                                                                        | 2026-07-03 |
| —   | **Pastas legadas excluídas** (01, 02, 03, 06) — sem duplicidade                                                                                                    | 2026-07-03 |
| —   | **Stack corrigida:** Rust para TUDO (backend + IA + dados + antifraude + APIs + autenticação + lobby + motor + front-end Dioxus)                                   | 2026-07-03 |
| —   | **evaluate_hand refatorado** — HighCard extraído para `get_high_card()`, padrão uniforme de 9 helpers, 18/18 testes                                              | 2026-07-03 |
| —   | **8 warnings de dead code limpos** — `side_pots.rs` (1) + `loss_deflator.rs` (7)                                                                                   | 2026-07-03 |
| —   | **47/47 testes Rust passando, 0 warnings** no `cargo build`                                                                                                       | 2026-07-03 |
| 4   | **Rake da casa** — módulo `rake.rs` (2.5% default, cap R$6), 13 testes, integrado                                                                                  | 2026-07-02 |
| 3   | **Loss Deflator Progressivo** — módulo `loss_deflator.rs` (15%/25%/35%), 9 testes                                                                                 | 2026-07-02 |
| 2   | **Side Pots** — módulo `side_pots.rs` (all-in múltiplos jogadores), 7 testes                                                                                       | 2026-07-02 |
| 1   | **Deck + Hand Evaluation** — módulo `deck.rs` (criação, embaralhamento, avaliação), 18 testes                                                                     | 2026-07-02 |
| —   | **Mudança de stack alvo:** Rust (backend) + Python (IA) + TS (front)                                                                                               | 2026-06-30 |
| —   | Reorganização de pastas (01-07 sem gaps)                                                                                                                           | 2026-06-27 |
| —   | Consolidação do `ARQUITETURA_MOTOR.md`                                                                                                                             | 2026-06-27 |
| —   | Remoção de duplicatas (`client/`, `server/`, `shared/`)                                                                                                            | 2026-06-27 |
| —   | Criação do `DASHBOARD.md`                                                                                                                                          | 2026-06-27 |
| —   | Correção: Check sem validação                                                                                                                                      | 2026-06-25 |
| —   | Correção: Split Pot básico                                                                                                                                         | 2026-06-25 |

---

## 📊 Resumo Rápido — Indicadores do Projeto

| Indicador                | Valor                                                                                         |
|--------------------------|-----------------------------------------------------------------------------------------------|
| Regras documentadas      | 45                                                                                            |
| Regras implementadas    | 43 (95%)                                                                                      |
| Bugs críticos pendentes  | 0                                                                                             |
| Tarefas pendentes        | 8                                                                                             |
| Módulos Rust implementados | 10 (deck, side_pots, loss_deflator, rake, rng_crypto, hand_history, tournament_engine, auth, lobby, utils) + 4 antifraude + 1 API (10-API-Axum) |
| Testes Rust              | **554/554 passando** (484 motor + 12 API + 57 frontend), 0 warnings                                          |
| Front-end Dioxus         | ✅ Roteamento (3.5) + Componentes de Mesa (3.6) + Componentes de Lobby (3.7) concluídos — 12 componentes, 57 testes                                                |
| Progresso total          | **~55%** (ver `CRONOGRAMA.md`)                                                                        |
| Tarefa atual             | **Componentes de Auth (3.8)**                                                                        |

---

## 🗺️ Mapa das Pastas — Estrutura de Módulos

| #   | Pasta                 | O que contém                                                                                                              | Status                    |
|-----|----------------------|---------------------------------------------------------------------------------------------------------------------------|---------------------------|
| 04  | `Infraestrutura-Docker` | Docker, deploy, CI/CD                                                                                                   | ✅ Ativo                  |
| 05  | `Documentacao`       | Regras, auditoria, status, dashboard, cronograma                                                                          | ✅ Ativo                  |
| 07  | `Arquitetura-Motor`  | Arquitetura alvo (Rust puro)                                                                                              | ✅ Ativo                  |
| 08  | **`Motor-Rust`**     | **Motor de jogo em Rust (deck, side_pots, loss_deflator, rake, rng_crypto, hand_history, tournament_engine, auth, lobby + 4 antifraude)**     | **✅ Ativo — 484 testes — 100%** |
| 09  | **`Frontend-Dioxus`** | **Front-end WebAssembly com Dioxus — Roteamento (3.5) com 4 rotas + 12 componentes (Mesa + Lobby) + CSS puro Full Tilt Poker**                       | **✅ Ativo — 57 testes — 100%** |
| 10  | **`API-Axum`**       | **API HTTP/WS com Axum 0.7 (8 endpoints REST + WebSocket + JWT middleware + PostgreSQL via sqlx 0.8)**                       | **✅ Ativo — 12 testes — 100%** |

---

## 🔄 Comandos Rápidos — Build, Testes e Deploy

```bash
# Testar motor Rust (484 testes)
cd 08-Motor-Rust && cargo +stable-x86_64-pc-windows-gnu test --lib

# Build motor Rust (0 warnings)
cd 08-Motor-Rust && cargo +stable-x86_64-pc-windows-gnu build

# Testar API Axum (12 testes ativos)
cd 10-API-Axum && cargo +stable-x86_64-pc-windows-gnu test

# Build API Axum (0 warnings)
cd 10-API-Axum && cargo +stable-x86_64-pc-windows-gnu build

# Clippy API Axum (-D warnings)
cd 10-API-Axum && cargo +stable-x86_64-pc-windows-gnu clippy --all-targets -- -D warnings

# Verificar front-end Dioxus (compilando ✅, requer LIBRARY_PATH)
$gccLibPath = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\lib\gcc\x86_64-w64-mingw32\16.1.0"
$env:LIBRARY_PATH = $gccLibPath
$env:C_INCLUDE_PATH = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\include"
cd 09-Frontend-Dioxus && cargo +stable-x86_64-pc-windows-gnu check

# Testar front-end Dioxus (57 testes)
cd 09-Frontend-Dioxus && cargo +stable-x86_64-pc-windows-gnu test

# Subir infra (Docker)
cd 04-Infraestrutura-Docker && docker-compose up -d
```

---

> 💡 **Dica:** Ao voltar e dizer "vamos continuar", este painel será carregado automaticamente com o status mais recente.