# 🎯 Painel de Controle — Plataforma de Poker Online

**Atualizado:** 2026-09-01 | **Status:** **S20c** — UI iniciantes: história nas laterais, Dica do Pró, PT-BR futuro, sem painel duplicado, sem A♠ fantasma; demo/staging; sem certificação de produção.

> ⚠️ **REGRA DE OURO:** Antes de codar, consultar `Arquitetura-Motor/ARQUITETURA_MOTOR.md` e `Documentacao/BUSINESS_RULES.md`.
> 📌 **Fonte canônica de estado:** [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json) — prevalece sobre qualquer texto datado abaixo.
> 📅 O cronograma completo está em `Documentacao/CRONOGRAMA.md` — veja prazos, fases e % de conclusão.
> 🏗️ **Stack v4.0:** Rust no motor/API; TypeScript + React + Vite + Tailwind no frontend (`Frontend-Web/`). `Frontend-Dioxus/` removido do monorepo.
> **Domínio do produto:** [`zerotiltpoker.net`](https://zerotiltpoker.net) (demo/staging — VPS Hostinger).
> **Transporte público:** **HTTPS** (Caddy + **Let's Encrypt**); SPA same-origin.
> **E-mail:** Resend domínio **verified**; `EMAIL_PROVIDER=resend` na API (ver `EMAIL_RESEND.md`).
> **Presença online:** badge no header + hero na home; `GET /api/presence/online`; heartbeat JWT ~25s; TTL 90s.
> **Live E2E:** `scripts/live-e2e-ten-users.mjs` — 10 users / 100 hands + settlement assinado.
> **Demo amigos:** [`DEMO_AMIGOS.md`](DEMO_AMIGOS.md) — mín. **2 na mesma mesa**.
> **Limites conhecidos:** PIX mock/sandbox; mesas com dono único por processo; VPS Hostinger KVM 2 ok para ~40 concurrent; LE rate limit 5 certs/168h se recriar `caddy_data`.
> ⚖️ **Regulação / KYC / real-money compliance:** planejado para **janeiro de 2027**.

---

## 📅 Cadência de Sprints — Metodologia Ágil

| # | Parâmetro | Valor |
|---|-----------|-------|
| 1 | **Duração** | 2 semanas (14 dias) |
| 2 | **Sprint atual** | S13 — presença online + demo amigos |
| 3 | **Status** | 🟢 Contador online no ar; convites play-money liberados |
| 4 | **Cerimônias** | Planning + Review + Retrospectiva |
| 5 | **Retrospectivas** | Registradas em `DEVELOPMENT_LOG.md` |

### 🏁 Definition of Done (DoD) — Critério de "Pronto"

Uma tarefa só está **completa** quando TODOS os critérios abaixo são atendidos:

| # | Critério | Verificação |
|---|----------|-------------|
| 1 | **Código compila sem erros** | `cargo check` — 0 erros |
| 2 | **Zero warnings** | `cargo check` — 0 warnings |
| 3 | **Testes de rotina passam** | `cargo test` — apenas a suíte determinística e rápida, sem carga probabilística implícita |
| 4 | **Cargas de validação** | `scripts/full-validation.*` — 100 cenários centrais, somente após autorização explícita e com relatório de duração/carga/status; gateway Caddy validado por `verify-public-https.sh` |
| 5 | **Documentação atualizada** | `DASHBOARD.md` + `README.md` + `DEVELOPMENT_LOG.md` |
| 6 | **Regras de negócio respeitadas** | Conforme `BUSINESS_RULES.md` |
| 7 | **Padrões de qualidade** | Conforme `QUALITY.md` |
| 8 | **Sem regressões** | Sincronizado no GitHub |

### 📊 Histórico de Sprints

> Estes são registros datados. O status operacional atual prevalece: a
> integração PIX real continua adiada, embora os adaptadores permaneçam no
> código para uma futura retomada autorizada.

| Sprint | Período | Objetivo | Entregue | Status |
|--------|---------|---------|----------|-------|
| S01 | 2026-06-01 a 2026-06-14 | Infraestrutura, Tipos Core, Baralho e Avaliador de Mãos | 11 módulos, 169 testes | ✅ Concluído |
| S02 | 2026-06-15 a 2026-06-28 | Game Loop, Side Pots, Rake, Deflator, Antifraude e Auth | 43 testes adicionais, 0 erros | ✅ Concluído |
| S03 | 2026-06-29 a 2026-07-12 | Motor Financeiro, Torneios e Validação Extrema | 1.816 testes unitários | ✅ Concluído |
| S04 | 2026-07-13 a 2026-07-25 | Axum, Dioxus WASM, Fuzzing Extremo e Estresse | 2.051 testes + 1M Fuzzing | ✅ Concluído |
| S05 | 2026-07-25 | Auditar Parecer Técnico, Idempotência PIX, RwLock & Saneamento | Saneamento Completo de Segurança | ✅ Concluído |
| S06 | 2026-07-25 | Migração de Tipagem Monetária f64 -> u64 Centavos Inteiros | Precisão Monetária Bancária B3 | ✅ Concluído |
| S07 | 2026-07-26 | Commercial Grade & Redis Snapshots Fault Tolerance | Otimização Hand Evaluator + JWT + Redis Recovery | ✅ Concluído |
| S08 | 2026-07-28 | Correções críticas de segurança e arquitetura | Ledger PIX local, token version, timeout de turno, WebSocket e contratos PostgreSQL | ✅ Concluído localmente |
| S09 | 2026-07-29 | Recovery de mão + PIX sandbox | Guarda transacional de liquidação; Asaas sandbox restrito | ✅ Concluído localmente |
| S10 | 2026-07-31 → 2026-08-01 | Domínio/demo HTTPS + B2B SaaS | `zerotiltpoker.net`; tunnel/VPS; migration 014 clubs/agents; rake 15/85; dashboard HTTPS; lobby MTT | 🟡 Local pronto — go-live tunnel e commit full pendentes |
| S11 | 2026-08-04 → 2026-08-05 | Frontend TypeScript + demo VPS | `Frontend-Web` React/Vite/Tailwind; docs v4.0; regulação jan/2027; VPS Hostinger LE (2026-08-05); Resend `zerotiltpoker.net` verified; auth e-mail + Resend na API | 🟢 Demo HTTPS + e-mail operacionais |
| S12 | 2026-08-05 → 2026-08-07 | Auth MFA + supply chain + prova live | MFA challenges (016); CI/Dependabot/SBOM; legal_actions; settle pós-disconnect; settlement HMAC (017); smoke live 10×100 PASS com settlementsVerified; lote sintético 2º limpo | 🟢 Fechado (demo); sem cert. produção |
| S13 | 2026-08-07 | Presença online + docs/higiene | Badge + hero online; API presence; deploy VPS; DEMO_AMIGOS; sync docs; limpeza de artefatos locais | 🟢 Fechado |
| S14–S16 | 2026-08 | Admin, depósitos manuais, privacidade | Painel admin; Pedir fichas PIX; WHOIS/noindex | 🟢 Fechado |
| S17 | 2026-08 | Play Money × Jogo Real | Wallets PM cash/MTT + Real; mesas/torneios isolados por `money_mode` | 🟢 Fechado |
| S18 | 2026-08-29 | Variantes + catálogo + validação | Short Deck + SD Omaha; frentes/blinds oficiais; 10k mãos/config PASS; e2e Real/PM mesa a mesa PASS; migrations 025 | 🟢 Fechado (demo) |
| S19 | 2026-08-31 | Sessão resiliente + DePix Sandbox protegida | Heartbeat + presence TTL 90s; DePix sk_test + allowlist + HMAC + dedup; migrations 030 | 🟢 Fechado (demo) |
| S20 | 2026-09-01 | Big Blind Ante 26 níveis + potes laterais | Ante = big_blind 26/26 (BBA); ante morto só no main pot; cash sem ante; `HoldemFTShortDeck` (VARCHAR 30); 1828 Motor + 35 API PASS; VPS 4/4 healthy | 🟢 Fechado (demo) |
| S20b | 2026-09-01 | História completa + PT-BR normalizado | Mundo 8 blocos (1829→Triton, modalidades, lendas, eventos) + Brasil 7 blocos (BSOP/CPH/H2 2011‑2014, Akkari/Yuri, Trafane/H2); correctPtOrthography + ProseRichText + Dica do Pró; Vite 60 módulos | 🟢 Fechado (demo) |
| S20c | 2026-09-01 | UI polimento iniciante | Vazios laterais com história (360|1fr|360), login volta ao centro, sem painel duplicado (OnlinePresenceHero), sem A♠ fantasma (case-sensitive cards), fontes por bloco + disclaimer H2 2006; Vite 324KB | 🟢 Fechado (demo) |

**Catálogo cash vigente:** NL 0,25/0,25 · NL 0,25/0,50 · SD 0,50/0,50 · SD Omaha 0,50/1 (cada um em PM e Real).

---

## 🌐 Deploy da demo (escolha um caminho)

| Caminho | Quando usar | Guia |
|---------|-------------|------|
| **Casa + Cloudflare Tunnel (HTTPS E2E)** | Sem cartão / sem VPS; PC ligado | `Infraestrutura-Docker/DEPLOY_HOME_CLOUDFLARE.md` |
| **VPS (Hetzner ou BR com PIX)** | Servidor 24/7 na nuvem | `Infraestrutura-Docker/DEPLOY_HETZNER.md` + `.env.staging.example` |
| **Lab local só na máquina** | Dev sem domínio | `docker compose up` + `DOMAIN_NAME=localhost` |

**HTTPS obrigatório no browser:** tunnel com Origin CA + Cloudflare **Full (strict)**; VPS com Caddy + Let's Encrypt.  
**CORS:** `https://zerotiltpoker.net` · **PIX:** mock.

---

## 📋 Tarefas por Status — Conclusão Geral

### 🟢 Tarefas Concluídas (100% Complete)
| #   | Tarefa                                                                                                      | Pasta                     | Status     |
|-----|-------------------------------------------------------------------------------------------------------------|---------------------------|------------|
| 1   | **Testes Massivos e de Estresse Extremo** — 1.000 iterações Ante/Blinds, 500 All-in multiway e desconexão  | `Motor-Rust/` & `API-Axum/`| ✅ Concluído |
| 3   | **Suíte Antifraude Unificada & Avaliação no Ator** — Real-time risk scoring no TableActor                     | `Motor-Rust/` & `API-Axum/`| ✅ Concluído |
| 4   | **Persistência Assíncrona do Hand History** — Gravação no PostgreSQL e endpoints REST                      | `API-Axum/`               | ✅ Concluído |
| 5   | **Pipeline CI/CD GitHub Actions & Scripts de Deploy** — `.github/workflows/ci.yml` e `scripts/deploy.sh`     | `Infraestrutura-Docker/`  | ✅ Concluído |

### ⏸️ Escopo deliberadamente adiado

| # | Tarefa | Pasta | Status |
|---|--------|-------|--------|
| PIX | Adaptadores e testes locais preservados; integração real de depósitos, saques e webhooks requer nova autorização de escopo | `API-Axum/` & `Frontend-Web/` | ⏸️ Adiado |

### ✅ Concluídas — Sprint Atual (S13 + S12)
| #   | Tarefa                                                                                                                                                              | Data       |
|-----|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------|
| ONL | **Contador de online** — `presence` API + badge header + hero home; heartbeat autenticado; TTL 90s Redis | 2026-08-07 |
| AUTH| **MFA + harden auth** — challenges opacos (mig 016), bcrypt non-blocking, lockout atômico, Resend async, guards de produção, LoginPage 2 passos | 2026-08-06 |
| SEC | **Supply-chain CI** — rust-ci reforçado, supply-chain workflow, Dependabot, audit.toml | 2026-08-06 |
| PLAY| **Mesa jogável no e2e** — `legal_actions` no WS/Frontend; settle após disconnect no `game_actor` | 2026-08-07 |
| SETL| **Settlement assinado** — mig 017, HMAC na liquidação, verify no hand-history replay | 2026-08-07 |
| E2E | **Smoke live 10×100** — `live-e2e-ten-users.mjs`; PASS `0833` (jornada) + PASS `0920` (`settlementsVerified=2`); 2º lote removido, 10 originais preservados | 2026-08-07 |
| TLS | **HTTPS público Let's Encrypt** — VPS Hostinger; DNS A `zerotiltpoker.net` → IP da VPS; Caddyfile canônico (sem `tls internal`); volume `caddy_data` persistido; cert LE emitido 2026-08-05 (issuer Let's Encrypt YE2, validade ~90d). Rate limit 429 documentado em `Caddyfile.tls-internal` e `STATUS_OPERACIONAL.json`. | 2026-08-05 |
| MAIL| **Resend go-live** — domínio `zerotiltpoker.net` **verified** (DKIM `resend._domainkey`, SPF+MX `send` na Hostinger); API `email_provider=resend` + `require_email_verification=true`; rebuild `poker_api` com código Resend; `EMAIL_FROM` `noreply@` recomendado. Guia: `EMAIL_RESEND.md`. | 2026-08-05 |
| B2B | **SaaS multi-tenant (local)** — migration `014` (`clubs`, `club_memberships`, `club_agents`, `club_id` em tables/tournaments); rake split 15/85 no motor + crédito de `club_rake` no `TableActor`; admin API clubs/financials/withdraw/theme/agents; dashboard Dioxus `/admin/clubs` via **HTTPS** (JWT + fallback demo); lobby MTT `/tournament/:id`; compose com healthchecks API/Caddy; `.env.production.example`. | 2026-08-01 |
| S10 | **Domínio e demo HTTPS** — `zerotiltpoker.net`; compose `.env`; `Caddyfile.tunnel` + Origin CA; `DEPLOY_HOME_CLOUDFLARE.md`; Hetzner opcional; demo amigos (play-money + mesas seed). | 2026-07-31 |
| S08 | **Correções críticas validadas localmente** — `wallet_transactions` ganhou chave de idempotência, identificador externo único, status de provedor e impressão de chave PIX; webhook liquida saldo e status na mesma transação; saque reserva saldo e emite outbox sem payout externo. `token_version` persistente revoga JWTs após mudança sensível; Redis limita IPs de forma compartilhada; timeout de turno aplica fold automático; WSS responde ping/pong e remove campos sensíveis. Passaram 1.814 testes determinísticos do motor, 12 unitários da API, 11 contratos PostgreSQL, 1 contrato Redis e Clippy estrito. A carga manual não foi acionada. | 2026-07-28 |
| AUD | **Deep Audit Fixes, Rate Limiting & Regra do Centavo Ímpar** — 1) Validação HMAC-SHA256 em Webhook PIX (`payments_routes.rs`); 2) CSPRNG para UUID v4, TOTP e backup codes (`auth.rs`); 3) Middleware de Rate Limiting `RateLimiter` em memória por IP para endpoints sensíveis de Auth e Pagamentos (`rate_limit.rs`); 4) Suíte de 6 testes unitários para Flush e 14 proptests de Fuzzing em `fuzz_tests.rs`; 5) Implementação da Regra do Centavo Ímpar (*Odd Cent Rule* — WSOP/TDA Regra 68) em `utils::dividir_pote_empatado` e documentação na Seção 4.4 do `BUSINESS_RULES.md`. Todos os testes passando! | 2026-07-23 |
| LD56| **Loss Deflator por Equity, após o Rake** — Regra vigente: <56%=0%; 56–65,9%=7%; 66–75,9%=15%; 76–85,9%=25%; ≥86%=35%. A fase apenas reconstrói o board no instante do all-in; não determina o tier. Main pot e side pots são taxados antes do cashback. Multi-WS, multiway equity e `loss_deflators_json` (2026-07-31). | 2026-07-31 |
| PIX | **Módulo de Pagamentos PIX** — ledger PostgreSQL e endpoints preservados; `payment_gateway.rs` usa Mock ou Asaas Sandbox autenticado. Mercado Pago, payout automático e PIX de produção estão desabilitados. | 2026-07-29 |
| EXFZ| **Fuzzing Extremo Massivo (1.000.000 de Iterações no Motor Rust)** — Implementado `extreme_fuzz_tests.rs` cobrindo 8 módulos críticos (`rake`, `side_pots`, `loss_deflator`, `hand_history`, `auth`, `antifraud`, `tournament`, `deck`). **1 MILHÃO de mutações estocásticas executadas sem falhas, leaks ou panics** (25.95s). Total Motor Rust: **1.904 testes passando**. | 2026-07-22 |
| ST2 | **Estresse Massivo 1M WS, Red Team 50 Workers & Fix Antifraude** — 1) Correção na fórmula de pontuação de colusão (`CollusionDetector::calculate_score()`) em `antifraud_engine.rs` (100% verde no Motor); 2) Carga massiva de **1.000.800 mensagens WebSockets** em 100 mesas ativas sem deadlock (`ws_stress_tests.rs`, 25.81s); 3) Red Team com **50 workers simultâneos** de ataque (1.000 brute-force, 1.000 JWT tampering, 1.000 WS injection). | 2026-07-22 |
| Cov | **Fuzzing Massivo MTT, Network Jitter WS & Carga no PostgreSQL Pool** — 1) Fuzzing de Rebalanceamento MTT no Motor Rust (`tournament_fuzz_tests.rs`, 200.000 iters); 2) Estresse de Instabilidade de Rede e Lag em WebSockets (`ws_network_jitter_tests.rs`); 3) Estresse de Transações em Massa no PostgreSQL Pool (`db_pool_stress_tests.rs`). `cargo clippy` ✅ (0 warnings), **2.051 testes passando** (0 falhas). | 2026-07-22 |
| RT  | **Módulo Antifraude (IA/ML), Métricas Prometheus & Simulação Red Team** — 1) Módulo `antifraud_engine.rs` no Motor Rust com `BotDetector` (análise de variância de tempo de reação) + `CollusionDetector` + `RiskScore`; 2) Endpoints `/api/metrics` (Prometheus) e `/api/health/security` no Axum; 3) Suíte autônoma de Red Team (`red_team_simulation_tests.rs`) validando repulsa a brute-force, JWT tampering e WS injection (4/4 testes ✅). Total plataforma: **2.044 testes passando** (0 falhas, 0 clippy warnings). | 2026-07-22 |
| SEC | **Hardening de Segurança Enterprise ("Fortaleza Híbrida")** — 1) Security Headers OWASP no Caddyfile (HSTS, CSP, X-Frame-Options, X-Content-Type-Options); 2) Container Hardening (`docker-compose.yml` e `API-Axum/Dockerfile` com `USER 10001`, `cap_drop: ALL`, `read_only: true`, `no-new-privileges:true`, `tmpfs`); 3) DevSecOps Trivy Vulnerability Scanner no CI/CD (`rust-ci.yml`). `cargo clippy` ✅ (0 warnings), **2.036 testes passando** ✅. | 2026-07-22 |
| FZ  | **Fuzzing Dinâmico & Estresse no Frontend Dioxus (`Frontend-Dioxus`)** — 10 funções de Fuzzing baseadas em propriedades (`fuzz_tests.rs`) + 10.000 mutações de estado em rajada (`state_stress_tests.rs`). Descoberto e corrigido caso limite Unicode com expansão de caracteres turcos `İ` (`to_lowercase()`). Total Frontend: **115 suítes de testes passando** (0 falhas, 0 clippy warnings). | 2026-07-22 |
| ST  | **Fuzzing HTTPS Massivo & Estresse da API Axum (`API-Axum`)** — `api_fuzz_tests.rs` expandido para 10 funções de fuzzing cobrindo 100% dos endpoints REST expostos via HTTPS sob 2.000 iterações por função (`552.17s`). 600 operações WSS simultâneas sem deadlock e 30 cadastros/transações paralelas com bcrypt/JWT no PostgreSQL. Total API: **34 suítes de testes passando** (0 falhas, 0 clippy warnings). | 2026-07-22 |
| DB  | **Testes de Integração PostgreSQL (Opção 2)** — `api_tests.rs` ativado com banco PostgreSQL real (container `poker_postgres`). 5/5 testes de contrato passando (`register_login_flow`, `duplicate_409`, `invalid_credentials_401`, `lobby_join`, `hand_history_404`). Fix no gerador `generate_uuid_v4()` (`4{:01x}`). | 2026-07-22 |
| DOC | **Orquestração Docker Stack & Proxy HTTPS Caddy (Opção 3)** — Stack completa com 6 containers (PostgreSQL, Redis, Zookeeper, Kafka, API Axum, Frontend Dioxus Caddy) rodando e comunicando via HTTPS/TLS auto-assinado (`https://localhost`). Teste E2E de registro com emissão de JWT passando via proxy (`200 OK`). | 2026-07-22 |
| 5   | **Fuzz & Property Tests (`fuzz_tests.rs`)** — Estratégias `proptest` para `rake`, `side_pots`, `loss_deflator`, `auth` JWT e `hand_history` JSON. 6 suítes adicionadas. `cargo clippy` ✅ (0 warnings), `cargo test` ✅ (**1.880 testes no Motor-Rust**). | 2026-07-21 |
| 3.9 | **Integração API ↔ Front (WebSockets)** — Conexão do Dioxus via WsClient com o TableActor e GameLoop do Axum, sincronizando apostas e turnos com anti-cheat nativo. | 2026-07-17 |
| I2  | **Testes de Integração + Stress + Fairness + CI/CD** — `integration_tests.rs` (5 det.), `stress_integration_tests.rs` (5×200k iters, seed fixo), `card_fairness_tests.rs` (3×500k qui-quadrado), `stress_tests.rs` (15); `loss_deflator.rs` MC_SAMPLES=500k (tolerância 0,005); `rng_crypto.rs` qui-quadrado; 10 warnings clippy corrigidos; CI/CD em `.github/workflows/rust-ci.yml` (clippy -D warnings + test + cargo audit). Total Motor: **1.874 testes, 0 falhas**. Commit `ab19168`. | 2026-07-20 |
| I3  | **CI corrigido para raiz + Job de Cobertura** — workflow movido para `.github/workflows/` (GitHub só lê a raiz); paths antigos `08/09/10-` corrigidos para `Motor-Rust`/`Frontend-Dioxus`/`API-Axum`; adicionado job `coverage` (`cargo llvm-cov`, artefato lcov + summary). | 2026-07-20 |
| 4 & 6| **Dockerfiles + HTTPS/TLS** — Dockerfiles multi-stage unificados para API e Frontend. Caddyfile de proxy reverso e local HTTPS auto-assinado adicionado e testado. | 2026-07-17 |
| 3.8 | **Componentes de Auth** — 3 componentes Dioxus em `Frontend-Dioxus/src/components/`: `login_form.rs` (formulário de login com username/senha, props title/submit_label/on_submit), `register_form.rs` (formulário de registro com username/email/senha/confirmar senha, props title/submit_label/on_submit), `mfa_input.rs` (input de 6 dígitos TOTP, props title/instruction/on_submit). 3 páginas: `pages/login.rs` (AuthFlow enum com Login/MfaRequired/Success/Error, validação client-side, navegação para lobby), `pages/register.rs` (validação de senha, confirmação, navegação para login), `pages/login.rs` com `mfa_required` state. CSS puro (~100 linhas) em `assets/index.html` com visual Full Tilt Poker. `cargo clippy -- -D warnings` ✅ (0 warnings), `cargo test` ✅ (61/61). | 2026-07-12 |
| 3.7 | **Componentes de Lobby** — 5 componentes Dioxus em `Frontend-Dioxus/src/components/`: `table_card.rs` (card de mesa com GameType, blinds, ocupação, 7 testes), `lobby_filters.rs` (filtros por tipo de jogo + range de blinds, 9 testes), `join_button.rs` (botão Entrar/Cheia/Assistir, 5 testes), `player_count.rs` (contador visual X/Y com barra de progresso, 8 testes), `lobby_list.rs` (lista combinando TableCard + PlayerCount + JoinButton, 5 testes). CSS puro (~250 linhas) em `assets/index.html` com visual Full Tilt Poker (dark felt #1a3a1a, gold #8b6914). Tailwind CDN removido. `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (57/57). Doctest `utils.rs:34` corrigido (`&Vec<f64>` → `&[Pot]`). Motor: 484/484 testes ✅ | 2026-07-12 |
| —   | **Componentes de Mesa (3.6)** — 7 componentes Dioxus em `Frontend-Dioxus/src/components/`: `card.rs` (Carta face/verso, Suit/Rank enums, 5 testes), `avatar.rs` (Avatar/Jogador, Position/PlayerStatus enums, 3 testes), `pot.rs` (Pote central, PotEntry struct, 3 testes), `community_cards.rs` (Cartas comunitárias, CommunityStage enum, 3 testes), `action_buttons.rs` (Fold/Check/Call/Raise/All-in, ActionKind enum, 3 testes), `seat.rs` (Assento com posição absoluta, SeatPosition struct, 2 testes), `table.rs` (Mesa oval completa integrando todos, PlayerData struct + mock helpers, 3 testes). `pages/table.rs` refatorado para usar `TableView` com mock data. `cargo check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅ (24/24), `cargo build --release` ✅ (0 warnings) | 2026-07-11 |
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
| 3   | **Loss Deflator Progressivo** — módulo `loss_deflator.rs` (7%/15%/25%/35% por equity), 9 testes                                                                | 2026-07-02 |
| 2   | **Side Pots** — módulo `side_pots.rs` (all-in múltiplos jogadores), 7 testes                                                                                       | 2026-07-02 |
## 🗺️ Mapa das Pastas — Estrutura de Módulos

| #   | Pasta                 | O que contém                                                                                                              | Status                    |
|-----|----------------------|---------------------------------------------------------------------------------------------------------------------------|---------------------------|
| —   | `Infraestrutura-Docker` | Docker, Caddy HTTPS, deploy casa/VPS, CI/CD                                                                              | ✅ Ativo                  |
| —   | `Documentacao`       | Regras, STATUS_OPERACIONAL, dashboard, cronograma, logs                                                                   | ✅ Ativo                  |
| —   | `Arquitetura-Motor`  | Arquitetura alvo (Rust puro)                                                                                              | ✅ Ativo                  |
| —   | **`Motor-Rust`**     | **Motor (deck, side_pots, loss_deflator, rake+B2B 15/85, rng, hand_history, tournament, auth, lobby, antifraude)**      | **✅ Ativo — suíte histórica ~1.904 (`--lib`)** |
| —   | **`Frontend-Dioxus`** | **WASM: rotas Home/Login/Register/Lobby/Table + `/admin/clubs` + `/tournament/:id`; WSS; PIX modals**                   | **✅ Ativo — ~115 suítes reportadas** |
| —   | **`API-Axum`**       | **API HTTPS/WSS Axum; TableActor; admin B2B; migrations até 014**                                                         | **✅ Ativo — ~32–34 suítes reportadas** |

> Contagens de testes: valores **históricos reportados** em logs/CI; revalidar com `cargo test` no ambiente atual (toolchain GNU no Windows).

---

## 🔄 Comandos Rápidos — Build, Testes e Deploy

```bash
# Testar motor Rust (suíte lib; contagem histórica ~1.904)
cd Motor-Rust && cargo +stable-x86_64-pc-windows-gnu test --lib

# Build motor Rust (0 warnings)
cd Motor-Rust && cargo +stable-x86_64-pc-windows-gnu build

# Testar API Axum (suíte reportada ~32–34)
cd API-Axum && cargo +stable-x86_64-pc-windows-gnu test

# Build API Axum (0 warnings)
cd API-Axum && cargo +stable-x86_64-pc-windows-gnu build

# Clippy API Axum (-D warnings)
cd API-Axum && cargo +stable-x86_64-pc-windows-gnu clippy --all-targets -- -D warnings

# Verificar front-end Dioxus (compilando ✅, requer LIBRARY_PATH)
$gccLibPath = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\lib\gcc\x86_64-w64-mingw32\16.1.0"
$env:LIBRARY_PATH = $gccLibPath
$env:C_INCLUDE_PATH = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\include"
cd Frontend-Dioxus && cargo +stable-x86_64-pc-windows-gnu check

# Testar front-end Dioxus (115 testes)
cd Frontend-Dioxus && cargo +stable-x86_64-pc-windows-gnu test

# Subir infra (Docker)
cd Infraestrutura-Docker && docker-compose up -d
```

---

> 💡 **Dica:** Ao voltar e dizer "vamos continuar", este painel será carregado automaticamente com o status mais recente.

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-01):** S20c — UI para iniciantes: vazios laterais com história (360px|1fr|360px), Dica do Pró, correção PT-BR futura, sem painel duplicado e sem A♠ fantasma; catálogo cash canônico NLHE 0,25/0,25, Hold'em Short Deck 0,25/0,50 e Omaha Short Deck 0,50/0,50 (Play Money e Jogo Real). **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy; migrations 001–032 aplicadas. Gate S20c: cargo fmt, Clippy estrito, tsc -b + Vite 60 módulos 324KB — todos sem falhas; VPS 4/4 healthy, 26 níveis ante=big_blind (invalid_BBA=0), backup verificável, rebuilds 4m13s + 18s e health público OK. Frontend: PT-BR normalizado (correctPtOrthography + htmlToStructuredMarkdown + ProseRichText), Dica do Pró, história 8+7 com fontes e H2 2006 fidedigno (disclaimer), sem A♠ fantasma (case-sensitive cards), vazios laterais preenchidos com história e sem painel duplicado. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
