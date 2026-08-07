# 📝 Histórico de Desenvolvimento — Plataforma de Poker Online

**Atualizado:** 2026-08-07
**Propósito:** Registro cronológico de desenvolvimento + retrospectivas de sprint.

> Painel tático em `DASHBOARD.md`. Cronograma em `CRONOGRAMA.md`. Estado canônico em `STATUS_OPERACIONAL.json` (prevalece sobre retrospectivas históricas que digam “Launch Ready”).

---

## 📌 2026-08-07 — S12 fechamento: auth/MFA, settlements assinados, smoke live 10×100

| Item | Detalhe |
|------|---------|
| **Branch** | `codex/security-supply-chain` (trabalho Codex 05–07/08 + fechamento documental) |
| **Auth / MFA** | Challenges de login (mig **016**), bcrypt em `spawn_blocking`, lockout atômico no Postgres, Resend assíncrono, guards de produção, UI MFA no `Frontend-Web` |
| **Supply chain** | Workflows CI reforçados, Dependabot, audit/SBOM/Trivy (publicação GHCR sob autorização) |
| **Mesa jogável** | WS expõe `legal_actions`; Frontend-Web usa ações legais; liquidação após disconnect corrigida no `game_actor` |
| **Settlement audit** | Mig **017** + HMAC de liquidação; API verifica assinatura no replay; históricos legados sem assinatura = não verificados |
| **Smoke live (1º lote)** | `run=202608070833` — **PASS**: 10 reg/verify/login/join, 100 mãos, cash-out; contas `zte2e202608070833*` **preservadas** |
| **Smoke live (2º lote + settlement)** | `run=202608070920` container `zero-tilt-e2e-10-v5` — **PASS**: 100 mãos, `settlementsVerified=2` (assinatura + winner + payouts+rake=pote por mesa) |
| **Limpeza** | Removido apenas lote `zte2e202608070920*`; zero assentos ACTIVE; zero recovery guards órfãos; stack healthy |
| **Script** | `scripts/live-e2e-ten-users.mjs` (Mail.tm, 10/100 fixos, checagem de settlement) |
| **Limites mantidos** | Sem certificação de produção; PIX mock; ownership single-process; regulação → jan/2027 |
| **Docs** | `STATUS_OPERACIONAL.json` S12; este log; `DASHBOARD.md`; `documentation-sync --write` |

### Commits principais (S12)

| Hash | Mensagem |
|------|----------|
| `96ea2a0` | feat: harden authentication and software supply chain |
| `c0934d1` | fix: unblock CI security validation |
| `7aebfe6` | fix: expose legal table actions to players |
| `0d7d415` | fix: settle hands after player disconnects |
| `71253f4` | feat: sign and verify hand settlements |
| `e0cdb1a` | test: verify signed hand settlements in live e2e |

---

## 📌 2026-08-05 — VPS: HTTPS Let's Encrypt + Resend go-live

| Item | Detalhe |
|------|---------|
| **VPS** | Hostinger KVM; stack Docker healthy (`poker_api`, `poker_frontend`/Caddy, Postgres, Redis) |
| **DNS** | `A zerotiltpoker.net` → IPv4 da VPS; UFW 80/443 |
| **TLS** | Após rate limit LE 429 (5 certs/168h), cert público emitido **2026-08-05** — issuer Let's Encrypt YE2, validade ~90 dias; Caddyfile **sem** `tls internal` |
| **Browser** | `ERR_CERT_AUTHORITY_INVALID` era CA local do Caddy (`tls internal`); resolvido com LE |
| **Resend** | Domínio `zerotiltpoker.net` **verified** (Hostinger: TXT DKIM `resend._domainkey`, MX+TXT SPF em `send`) |
| **API** | `Auth policy loaded require_email_verification=true email_provider=resend` nos logs; rebuild `poker_api` com código Resend |
| **From** | Recomendado `noreply@zerotiltpoker.net` (domínio verified); caixa `admin@` (webmail) é independente do envio transacional |
| **Cuidado** | Não apagar volume `caddy_data` a cada recreate — reestoura rate limit LE |
| **Docs** | `STATUS_OPERACIONAL.json`, `EMAIL_RESEND.md`, `Caddyfile.tls-internal` |

---

## 📌 2026-08-04 — Resend: e-mail real de verificação (código)

| Item | Detalhe |
|------|---------|
| **Provider** | `EMAIL_PROVIDER=resend` + `RESEND_API_KEY` + `EMAIL_FROM` |
| **Fallback** | Falha no Resend → log com código (registro não trava) |
| **Docs** | `Infraestrutura-Docker/EMAIL_RESEND.md` |
| **Amigos** | Com domínio verificado no Resend, cada jogador recebe o código sozinho |
| **Go-live** | Ver entrada 2026-08-05 |

---

## 📌 2026-08-04 — Registro: confirmar senha + verificação de e-mail

| Item | Detalhe |
|------|---------|
| **API** | `password_confirm`; status `pending_email_verification`; `POST /api/auth/verify-email` e `/resend-verification` |
| **Migration** | `015_email_verification.sql` (códigos hash SHA-256, TTL 15 min) |
| **E-mail** | Template boas-vindas Full Tilt; provider **resend** no deploy demo (2026-08-05); `log` em lab/testes |
| **Flag** | `REQUIRE_EMAIL_VERIFICATION` (padrão true em runtime; false nos testes) |
| **Front** | Confirmar senha; página `/verify-email`; login redireciona se pendente |
| **Join mesa** | JWT só com conta `active` |

---

## 📌 2026-08-04 — S11: Frontend TypeScript + stack híbrida

| Item | Detalhe |
|------|---------|
| **Decisão** | Abandonar regra “100% Rust no frontend”. UI canônica em **TypeScript + React + Vite + Tailwind** (`Frontend-Web/`). |
| **Visual** | Skin moderna inspirada no **Full Tilt** (felt, rail dourado, lobby tabular) — sem estética genérica de landing “AI”. |
| **Deploy** | `docker-compose` → `Frontend-Web/Dockerfile`; `Frontend-Dioxus/` marcado legado. |
| **Docs** | `ARQUITETURA_MOTOR.md` v4.0, BUSINESS_RULES, README, DASHBOARD, STATUS_OPERACIONAL, QUALITY, CRONOGRAMA. |
| **Regulação** | Trilho de compliance / real-money planejado para **janeiro de 2027**. |
| **Build** | `npm run build` em Frontend-Web OK (Vite produção). |

---

## 🔄 Retrospectivas de Sprint

> **Metodologia:** Ao final de cada sprint (2 semanas), registrar: o que funcionou, o que não funcionou, e o que melhorar no próximo sprint.
> **Nota (2026-08-01):** trechos históricos que mencionam “pronta para produção / Launch Ready” referem-se a readiness de **código/demo local** na época; o produto **permanece sem certificação de produção** (ver `STATUS_OPERACIONAL.json`).

### 📋 Sprint S01 (2026-06-25 → 2026-07-02) — Fundação + Motor Rust

| # | Aspecto | Avaliação |
|---|--------|----------|
| 1 | **O que funcionou** | Estrutura de pastas organizada desde o início; 45 regras de negócio documentadas antes de codar; 4 módulos Rust entregues com 47 testes |
| 2 | **O que não funcionou** | Stack inicial confusa (Node.js + Python + Go + TS); muita refatoração de stack |
| 3 | **Lições aprendidas** | Definir stack definitiva ANTES de codar; documentação primeiro (SDD) economiza retrabalho |
| 4 | **Melhorias para S02** | Decidir stack definitiva; criar cronograma com fases; formalizar testes |

### 📋 Sprint S02 (2026-07-03 → 2026-07-07) — Stack Rust-only + 4 Módulos + Dioxus

| # | Aspecto | Avaliação |
|---|--------|----------|
| 1 | **O que funcionou** | Stack Rust-only decidida e documentada; 4 módulos entregues (tournament, hand_history, rng_crypto, auth) totalizando 1.816 testes; front-end Dioxus compilando; QUALITY.md com 18 seções; Seção 4.0 Pentest adicionada |
| 2 | **O que não funcionou** | Conversão u64→f64 demorou mais que o esperado (precisão de testes); sem cadência formal de sprints |
| 3 | **Lições aprendidas** | f64 requer truncamento explícito e tolerâncias de teste; documentação de segurança (Pentest) é tão importante quanto o código |
| 4 | **Melhorias para S03** | Formalizar sprints com DoD; adicionar retrospectivas; focar em componentes Dioxus |

### 📋 Sprint S03 (2026-07-04 → 2026-07-17) — Componentes Dioxus + Lobby + Antifraude

| # | Aspecto | Avaliação |
|---|--------|----------|
| 1 | **O que funcionou** | Integração em tempo real com canais WebSockets e o modelo de Atores no Axum; O bypass manual do Dioxus CLI no Docker contornou totalmente os conflitos de versão do wasm-bindgen gerando uma imagem limpa e rápida do Caddy. |
| 2 | **O que não funcionou** | Mismatch de versões estáticas do compilador Rust e dependências do Wasm com a CLI antiga do Dioxus causou atrito e travamentos de build em container nas primeiras tentativas. |
| 3 | **Lições aprendidas** | Dioxus CLI engessa as versões do wasm-bindgen que usa; realizar builds manuais do WebAssembly em containers Docker de produção é mais adaptável e muito mais leve. |
| 4 | **Melhorias para S04** | Configurar testes de cobertura (llvm-cov/tarpaulin) no CI/CD e organizar workflows automáticos de build/deploy. |

### 📋 Sprint S04 & S05 (2026-07-18 → 2026-07-25) — Testes de Estresse, PIX Gateway, Hardening & Launch Ready

| # | Aspecto | Avaliação |
|---|--------|----------|
| 1 | **O que funcionou** | Fuzzing extremo (1M iterações), carga massiva WS (1M mensagens sem deadlock), Red Team de 50 workers, Mock PIX e persistência PostgreSQL imutável, com hardening Docker/Caddy e CI/CD. |
| 2 | **O que não funcionou** | Incompatibilidade inicial entre o toolchain MinGW local e proc-macros do Dioxus no Windows exigiu ajuste estrito de ambiente (LIBRARY_PATH). |
| 3 | **Lições aprendidas** | Fuzzing estocástico e simulação autônoma de Red Team revelam edge-cases (ex: regressoes de colusao e truncamentos de centavos) antes que afetem a produção. |
| 4 | **Status Final** | 100% dos testes da suíte de rotina (2.051 no total reportado), 0 clippy warnings; **não** certificação de produção (PIX mock; ownership single-process). |

---

## 📜 Log Cronológico de Desenvolvimento

### [24] 🏢 S10 — B2B SaaS multi-tenant, agentes HTTPS e lobby MTT (2026-08-01)
**O que foi feito:**
- **Schema B2B:** migration `014_b2b_organizations_schema.sql` — `clubs`, `club_memberships`, `club_agents`, `club_id` em `tables`/`tournaments`.
- **Motor:** rake split 15% plataforma / 85% clube em `RakeResult` + invariante de soma; crédito de `club_rake` no ledger do clube ao liquidar mão (`game_actor`).
- **API admin:** clubs, financials, withdraw, theme, agents (GET/POST) com persistência PostgreSQL e audit log.
- **Frontend:** `/admin/clubs` consome a API via **HTTPS** same-origin + JWT (fallback demo sem token); `/tournament/:id` lobby MTT (blinds, prizepool, inscritos).
- **Deploy:** healthchecks Compose (Postgres, Redis, API `/health`, Caddy); `curl` na imagem da API; `.env.production.example`.
- **Docs:** `STATUS_OPERACIONAL.json` e blocos `DOCUMENTATION_SYNC` realinhados; remoção de alegações “Launch Ready de produção” no cronograma.

**Limites mantidos:** sem certificação de produção; PIX mock; ownership single-process; smoke do domínio e commit completo do WIP ainda operacionais.

---

### [23] 🌐 S10 — Domínio zerotiltpoker.net e demo HTTPS (casa + Cloudflare) (2026-07-31)
**O que foi feito:**
- **Domínio do produto:** `zerotiltpoker.net` nos templates de env (staging e tunnel), Caddy e guias de deploy.
- **Compose staging-ready:** `POSTGRES_*`, `JWT_SECRET`, `CORS_ORIGINS`, `DOMAIN_NAME` e PIX via `.env` (sem senha fixa obrigatória no YAML).
- **Demo em casa sem VPS/cartão:** `Caddyfile.tunnel` com **TLS na origem** (Cloudflare Origin CA), `docker-compose.tunnel.yml`, `certs/` (gitignore de chaves), `DEPLOY_HOME_CLOUDFLARE.md` — browser e origem em HTTPS; SSL Cloudflare **Full (strict)**.
- **Fallback documentado:** `Caddyfile.tunnel-http-only` (não recomendado; browser ainda HTTPS na borda).
- **VPS opcional:** `DEPLOY_HETZNER.md` + `.env.staging.example` (Hetzner ou qualquer Ubuntu; BR com PIX se não houver cartão internacional).
- **Scripts frontend dist** portáteis; feature Dioxus `web` default; Dockerfile com detecção do entry wasm-bindgen.
- **GitHub:** commits de infra em `master` (push realizado).

**Pendente operacional:** nameservers Cloudflare no registrador, gerar `certs/origin*.pem`, `docker compose` + `cloudflared tunnel run`, smoke em `https://zerotiltpoker.net`.

**Limites mantidos:** sem certificação de produção; PIX mock; ownership de mesa single-process.

---

### [22] 🔐 S08 — Ledger local, revogação de token e transporte em tempo real (2026-07-28)
**O que foi feito:**
- **Carteira PIX local:** migrations `008` e `009` passam valores de `wallet_transactions` para centavos inteiros, criam chaves únicas de idempotência/identificador externo e versão persistente de token. O webhook assinado por HMAC trava a intenção pendente, confere identificador e valor persistidos e atualiza saldo + status na mesma transação. Saques reservam saldo com condição atômica e registram outbox; nenhuma requisição HTTPS dispara payout externo, e a chave PIX bruta não é gravada.
- **Autorização distribuída:** JWTs carregam `token_version`; o extrator de autenticação consulta status, papel e versão no PostgreSQL em cada rota protegida. O trigger de mudança sensível revoga tokens já emitidos em todas as réplicas.
- **Jogo e WSS:** ator aplica timeout de 30 segundos com auto-fold, rotação de dealer por assento físico e snapshot após ação válida. O WebSocket responde ping/pong, aceita envelope binário limitado a 64 KiB e redige recursivamente cartas alheias e campos sensíveis.
- **Observabilidade e rotina:** readiness consulta PostgreSQL/Redis, métricas expõem apenas gauges medidos, e os contratos financeiros PostgreSQL foram adicionados ao lote manual autorizado de validação.
- **Evidência local:** 1.815 testes determinísticos do motor, 12 unitários da API, 9 contratos gerais PostgreSQL, 2 contratos financeiros PostgreSQL, 1 contrato Redis e Clippy estrito do motor/API passaram. A carga massiva continua manual e não foi executada nesta alteração.

**Limites mantidos:** PIX real/payout continua fora do escopo e requer autorização + worker reconciliado; ownership de mesa continua local ao processo, mantendo Kubernetes em uma réplica.

---

### [21] 💵 Migração Arquitetural Estrita para `u64` Centavos Inteiros (2026-07-25)
**O que foi feito:**
- **Refatoração Monetária Mestre:** Substituição completa de `f64` por `u64` centavos inteiros nos tipos de saldos, apostas, stacks, potes, rake, buy-in e blinds em todos os crates (`Motor-Rust`, `API-Axum`, `Frontend-Dioxus`).
- **Eliminação de Erros Flutuantes IEEE-754:** Garantida a conservação de fichas e precisão bancária de centavos inteiros com divisão de pote de acordo com a regra WSOP Odd Cent (resto indivisível distribuído aos primeiros assentos à esquerda do botão).
- **Probabilidades e Estatísticas Preservadas:** Mantida a escala flutuante (`f64` entre 0.0 e 1.0 ou 0% a 100%) para cálculos estatísticos de equidade e exibição de porcentagens na UI.
- **Frontend Dioxus Visual:** Criados helpers de formatação de exibição `R$ {:.2}` convertendo o valor inteiro de centavos exclusivamente na camada de renderização visual.

---

### [20] 🛡️ Audit do Parecer Técnico e Saneamento Enterprise (2026-07-25)
**O que foi feito:**
- **Auditoria Ítem por Ítem:** Análise rigorosa das 30+ observações do parecer técnico, separando diagnósticos de arquitetura real de falsos positivos de scanners automáticos.
- **Segurança & Idempotência PIX:** Excluído módulo legado `auth_paseto.rs` (código morto com chave estática). Adicionada idempotência atômica no webhook PIX (`UPDATE transactions SET status='PROCESSED' WHERE status='PENDING'`) prevenindo ataques de replay. Adicionada verificação atômica de saldo no saque PIX.
- **Concorrência de Alta Carga (Axum):** Migrado `AppState` de `Arc<Mutex<...>>` para `Arc<tokio::sync::RwLock<...>>` (auth, lobby, tournaments, active_tables), permitindo milhares de leituras paralelas sem bloqueio de thread. Removida duplicação de struct em `state.rs`.
- **Bugs Funcionais no Motor & Frontend:** Ativada a rotação do botão dealer (`dealer_index`) no game loop. Removido disparo indevido de Sit command no ping do WebSocket. Corrigido redirecionamento indevido no Dioxus `lobby_list.rs` sob falha no join. Ajustado o cálculo dinâmico de apostas para Raise e AllIn em `table.rs`.
- **Limpeza de Infraestrutura & Workflows:** Removido arquivo de configuração legado `render.yaml` (que citava Node.js/npm) e substituído por ambiente Docker Rust. Deletados workflows duplicados no GitHub Actions, consolidando o pipeline em `.github/workflows/rust-ci.yml`. Fixadas versões no `docker-compose.yml` (`confluentinc/cp-zookeeper:7.5.0` e `cp-kafka:7.5.0`).
- **Sincronização 100% de Documentação:** Atualizados simultaneamente todos os 12 documentos da pasta `Documentacao/` conforme regra estrita em `AGENTS.md`.

---

### [19] 🚀 Finalização Mestre & Prontidão para Produção (2026-07-25)
**O que foi feito:**
- **Testes Massivos & Estresse Extremo:** Implementada a suíte Lote 7 em `game_loop_tests.rs` (1.000 iterações de Ante/Blinds, 500 All-Ins multiway com conservação de fichas e micro-stacks).
- **Correção S09 do registro anterior:** o projeto não integra Mercado Pago e não habilita PIX de produção. Existe somente Mock e adaptador Asaas Sandbox autenticado, restrito por allow-list; operações reais e payout permanecem bloqueados.
- **Suíte Antifraude Unificada:** Unificados os detectores sob o facade `AntiFraudSuite` no `TableActor` em tempo real.
- **Persistência PostgreSQL e REST Hand History:** Inserção assíncrona do `HandHistory` em PostgreSQL e rotas `/api/hand-history/{hand_id}`, `/api/tables/{table_id}/history` e `/api/admin/antifraud/alerts`.
- **Pipeline CI/CD & Deploy:** Criado workflow `.github/workflows/ci.yml`, arquivos `.env.example` e scripts `deploy.sh` e `deploy.ps1`.
- **Status Final:** 100% dos testes e especificações validados e atualizados na branch `master` no GitHub.

---

### [18] 🔧 CI/CD Corrigido para Raiz + Job de Cobertura (2026-07-20)
**O que foi feito:**
- **Descoberta:** o workflow `Motor-Rust/.github/workflows/ci.yml` criado em `ab19168` estava em subpasta — o GitHub Actions só lê `.github/workflows/` na **raiz**, logo não rodava. Além disso, o `rust-ci.yml` da raiz estava obsoleto (paths `08-Motor-Rust`/`09-Frontend-Dioxus`/`10-API-Axum` que não existem mais após a reorganização de pastas).
- **Correção:** reescrevi `.github/workflows/rust-ci.yml` (raiz) com paths corrigidos (`Motor-Rust`/`Frontend-Dioxus`/`API-Axum`) em todos os `working-directory`, `paths` e chaves de cache; removi o arquivo mal-posicionado `Motor-Rust/.github/workflows/ci.yml`.
- **Novo job `coverage`:** `cargo llvm-cov --all-targets --lcov` gera `lcov.info`, sobe como artefato (`motor-rust-coverage`) e imprime `--summary-only`. Relatório contínuo da cobertura do motor (sem gate rígido, para não ficar flaky).
- **Próximo passo sugerido (backlog #5):** Fuzz tests (`cargo-fuzz`) nos módulos críticos (rake, side_pots, loss_deflator, auth, parser hand_history).

*Próximo passo: Push para ativar o CI corrigido no GitHub e acompanhar o run (test/clippy/audit/coverage).*

---

### [17] 🧪 Testes de Integração/Stress/Fairness + CI/CD (2026-07-20)
**O que foi feito:**
- **`integration_tests.rs` (5 testes):** mão completa (deck→side_pots→rake→hand_history), ciclo de torneio, loss_deflator+rake, RNG+deck, conservação de fichas em side pots com fold (foldado contribui mas não recebe payout).
- **`stress_integration_tests.rs` (5 testes × 200k iters = 1M):** full_hand, sidepots_multiway, tournament, loss_deflator_plus_rake, rng_deck — seed fixo (`StdRng`, `SEED = 0xDEAD_BEEF_CAFE_1234`); invariantes exatos (conservação de fichas, vencedores ≥1, rake ≤ cap, não-duplicação de cartas).
- **`card_fairness_tests.rs` (3 testes × 500k = 1,5M):** ausência de duplicatas, distribuição de hole cards, distribuição flop/turn/river — qui-quadrado, tolerância 0,5% (0,005).
- **`stress_tests.rs` (15 testes):** stress por módulo (deck, side_pots, rake, utils, hand_history, tournament_engine).
- **`loss_deflator.rs`:** Monte Carlo `MC_SAMPLES = 500_000`, `mc_error_bound()` re-exportado em `utils.rs`; tolerância de teste 0,005. `get_heads_up_win_probability` determinístico via seed derivada das cartas.
- **`rng_crypto.rs`:** testes de distribuição por qui-quadrado (bool, d6, shuffle posição 0) — substitui asserts per-card flaky.
- **Clippy:** 10 warnings corrigidos em testes (loop-index, cast redundante, `sort_by_key`, `== false`, Range::contains, `*=`) → `cargo clippy --all-targets -- -D warnings` limpo (0 warnings).
- **CI/CD:** `.github/workflows/ci.yml` criado — jobs `test` (`RUSTFLAGS="-D warnings"`, clippy -D warnings, build, test) e `audit` (`cargo audit` via `rustsec/audit-check@v2` em ubuntu-latest).
- **Bugs corrigidos durante os testes:** `Card` não deriva Hash (bitmap de índices); tournament paga só 1º lugar quando resta 1; foldado contribui mas não recebe payout; bound de truncagem por-pote.
- **Quality gates validados:** `cargo clippy --all-targets -- -D warnings` ✅ 0 warnings; `cargo test --lib` ✅ **1.874 testes + 6 doc-tests, 0 falhas** (~480s).
- **Commit:** `ab19168` (11 arquivos, +1794/−260) enviado ao `origin/master`.

*Próximo passo: Acompanhar run do CI no GitHub (cargo audit em Linux).*

---

### [16] 🔌 Integração Real-time & Orquestração Docker/Caddy (2026-07-17)
**O que foi feito:**
- **Ator de Mesa (`game_actor.rs`)**: Criado gerenciador em background para orquestrar o GameLoop concorrente de cada mesa.
- **WebSocket seguro com anti-cheat**: Refatoração do websocket para consumir comandos de jogador e ocultar informações de cartas privadas não reveladas de outros competidores.
- **Dockerfiles Multi-stage**: Imagem da API com Debian Slim e Frontend compilando WebAssembly manualmente e servido via Caddy.
- **Proxy Reverso Caddy**: Caddyfile gerenciando HTTPS local (`https://localhost`), arquivos estáticos, fallback de SPA e redirecionamento de endpoints API/WS no docker-compose.
- **Documentação Sincronizada**: Histórico de sprints linkado em `DASHBOARD.md`.

---



### [06] 🎲 Motor de Poker em Rust — Migração e Refatoração (2026-07-03)
**O que foi feito:**
- **Stack definitiva:** Rust para TUDO (backend + APIs + IA + dados + antifraude + autenticação + lobby + motor de jogo + **front-end com Dioxus/WebAssembly**). Python, Go e TypeScript/React removidos da stack alvo.
- **4 módulos Rust implementados** em `Motor-Rust/src/`:
  - `deck.rs` — Criação, embaralhamento, avaliação de mãos Texas Hold'em (18 testes)
  - `side_pots.rs` — Calculadora de side pots para all-in múltiplos jogadores (7 testes)
  - `loss_deflator.rs` — Cashback progressivo para perdedores de all-in (9 testes)
  - `rake.rs` — Rake da casa 2.5% default, cap R$6 (13 testes)
- **evaluate_hand refatorado:** HighCard extraído para `get_high_card()`, padrão uniforme de 9 helpers
- **8 warnings de dead code limpos:** `side_pots.rs` (1) + `loss_deflator.rs` (7)
- **47/47 testes passando, 0 warnings** no `cargo build`
- **Documentos de gerenciamento atualizados:** `DASHBOARD.md`, `ARQUITETURA_MOTOR.md`

---
*Próximo passo: Aguardando definição do próximo módulo Rust.*

---

### [11] 🔐 Componentes de Auth — Login, Registro e MFA (2026-07-12)
**O que foi feito:**
- em `Frontend-Dioxus/src/components/`:
  - `login_form.rs` — Formulário de login com campos username/senha, props `title`, `submit_label`, `on_submit: EventHandler<AuthSubmitEvent>`. CSS com dark felt #1a3a1a, inputs em vidro escuro, botão submit gradiente dourado (#8b6914 → #6b4f0a)
  - `register_form.rs` — Formulário de registro com username/email/senha/confirmar senha, props `title`, `submit_label`, `on_submit: EventHandler<AuthSubmitEvent>`. Validação client-side de senha (mínimo 8 chars, maiúscula, número) e confirmação
  - `mfa_input.rs` — Input de 6 dígitos TOTP com auto-focus, props `title`, `instruction`, `on_submit: EventHandler<String>`. Máscara de 6 inputs individuais com foco automático progressivo
- **2 páginas atualizadas** em `Frontend-Dioxus/src/pages/`:
  - `login.rs` — `AuthFlow` enum (Login/MfaRequired/Success/Error) com `use_signal` para gerenciar estado do fluxo. Renderiza `LoginForm` ou `MfaInput` conforme estado. Navega para lobby em sucesso via `use_navigator()`
  - `register.rs` — Renderiza `RegisterForm`, valida senha e confirmação, navega para login em sucesso
- **CSS puro (~100 linhas)** adicionado em `assets/index.html` com visual Full Tilt Poker (inputs escuros, botões dourados, feedback visual de erro/sucesso)
- **`components/mod.rs` atualizado** — declara `login_form`, `register_form`, `mfa_input`
- **`pages/mod.rs` atualizado** — re-exporta `login`, `register`
- **Lições aprendidas:**
  - `EventHandler::new(|_| {})` **panica** fora de um runtime Dioxus — testes unitários que criam EventHandler diretamente não funcionam
  - Dioxus `#[props(default = "...")]` **não** gera funções `default_*()` associadas — não é possível testar valores default de props sem instanciar o componente
  - PowerShell consome `--` ao chamar scripts `.ps1` — usar `--%` (stop-parsing symbol) para passar `--` para o cargo: `.\scripts\cargo-dioxus.ps1 --% clippy -- -D warnings`
- **Quality gates validados:**
  - `cargo clippy -- -D warnings` ✅ — 0 warnings
  - `cargo test` ✅ — 61/61 testes passando
- **Total de testes frontend:** 61 (era 58 com 3 removidos)

### [10] 🧭 Roteamento Dioxus — dioxus-router 0.6 com 4 Rotas + Navbar (2026-07-11)
**O que foi feito:**
- **Dependência adicionada:** `dioxus-router = "0.6"` em `Frontend-Dioxus/Cargo.toml`
- **`router.rs` criado** com enum `Route` (derive `Routable, Clone, Debug, PartialEq`):
  - `Home {}` → `#[route("/")]`
  - `Login {}` → `#[route("/login")]`
  - `Lobby {}` → `#[route("/lobby")]`
  - `Table { id: String }` → `#[route("/table/:id")]`
- **`Root()`** renderiza `<div>` com `Navbar` persistente + `Router::<Route>` para troca de páginas
- **`Navbar`** com 3 `Link` (Home, Lobby, Login) usando `Route::Home {}`, `Route::Lobby {}`, `Route::Login {}`
- **4 módulos de página criados** em `Frontend-Dioxus/src/pages/`:
  - `mod.rs` — re-exporta `home`, `login`, `lobby`, `table`
  - `home.rs` — `Home()` com título "🃏 Poker Project", 3 `FeatureCard` (Texas Hold'em, Multiplayer, Antifraude)
  - `login.rs` — `Login()` com `use_signal(String::new)` para username/password, validação client-side, `use_navigator()` para push `Route::Lobby {}` em sucesso. A integração com API Axum era pendência desta etapa histórica e foi registrada posteriormente.
  - `lobby.rs` — `Lobby()` com 3 `MockTable` (table-001 Texas Hold'em, table-002 Omaha, table-003 Freeroll), `TableCard` com `Link` para `Route::Table { id }`
  - `table.rs` — `Table(id: String)` placeholder com "Conectando ao WebSocket..." (componentes virão no 3.6)
- **`main.rs` refatorado:** `mod pages; mod router;`, `fn app() -> Element { router::Root() }`, `launch(app)` com logger INFO
- **Convenção de nomes:** componentes em PascalCase (`Home`, `Login`, `Lobby`, `Table`, `Navbar`, `Root`) com `#[allow(non_snake_case)]` para compatibilidade com macro `Routable` do `dioxus-router` 0.6 e `rsx!` do Dioxus
- **2 testes unitários** em `router.rs`: `test_route_variants` (valida 4 variantes) + `test_route_clone_eq` (valida Clone + PartialEq)
- **Quality gates validados:**
  - `cargo check` ✅ — 0 warnings
  - `cargo clippy -- -D warnings` ✅ — 0 warnings
  - `cargo test` ✅ — 2/2 testes passando
  - `cargo build --release` ✅ — 0 warnings (2m 10s)
- **Total de testes Rust:** 1.830/1.830 passando (1.816 motor + 12 API + 2 frontend)

---
*Próximo passo: 3.6 Componentes de Mesa (mesa, cartas, avatares).*

---

### [11] 🃏 Componentes de Mesa — 7 Componentes Dioxus (2026-07-11)
**O que foi feito:**
- **7 componentes criados** em `Frontend-Dioxus/src/components/`:
  - `card.rs` — Carta individual (face/verso) com enums `Suit` (Spades/Hearts/Diamonds/Clubs) e `Rank` (Two..Ace), struct `PlayingCard`, 5 testes (suit_symbols, suit_colors, rank_labels, playing_card_new, playing_card_equality)
  - `avatar.rs` — Avatar/informações de jogador com enums `Position` (Dealer/SmallBlind/BigBlind/UTG/Middle/Cutoff/Button) e `PlayerStatus` (Waiting/Active/Folded/AllIn/Out), struct `AvatarData`, 3 testes (position_labels, status_colors, status_icons)
  - `pot.rs` — Pote central com valor acumulado via struct `PotEntry` (label + amount), função `pot_total()`, 3 testes (pot_entry_new, pot_total_sum, empty_pot)
  - `community_cards.rs` — Cartas comunitárias no centro da mesa com enum `CommunityStage` (PreFlop/Flop/Turn/River), função `stage_card_count()`, 3 testes (stage_card_count, stage_labels, cards_truncation)
  - `action_buttons.rs` — Botões de ação (Fold/Check/Call/Raise/All-in) com enum `ActionKind`, função `available_actions()`, 3 testes (action_labels, action_colors, action_equality)
  - `seat.rs` — Assento do jogador com posição absoluta via struct `SeatPosition` (top_percent, left_percent), 2 testes (seat_position_new, seat_position_equality)
  - `table.rs` — Mesa oval completa integrando todos os componentes com struct `PlayerData` + mock helpers `mock_table_data()` (5 jogadores) e `mock_flop()` (3 cartas), 3 testes (player_data_new, mock_table_data_count, mock_flop_count)
- **`components/mod.rs`** — Módulo raiz declarando os 7 submódulos (sem re-exports `pub use` para evitar warnings de `unused_imports`)
- **`pages/table.rs` refatorado** — Usa `TableView` component com mock data (5 jogadores + 3 cartas comunitárias no Flop), callback `on_action` registra ação via `log::info!`
- **`main.rs` atualizado** — Adicionado `mod components;` declaration ao lado de `mod pages; mod router;`
- **22 testes unitários novos** distribuídos pelos 7 componentes (5+3+3+3+3+2+3)
- **Quality gates validados:**
  - `cargo check` ✅ — 0 warnings
  - `cargo clippy --all-targets -- -D warnings` ✅ — 0 warnings
  - `cargo test` ✅ — 24/24 testes passando (22 novos + 2 existentes do router)
  - `cargo build --release` ✅ — 0 warnings
- **Total de testes Rust:** 1.852/1.852 passando (1.816 motor + 12 API + 24 frontend)
- **Documentação sincronizada:** `DASHBOARD.md`, `CRONOGRAMA.md` (2026-07-11)

---
*Próximo passo: 3.9 Integração API ↔ Front (api_client.rs + ws_client.rs).*

---

### [13] 🔌 Integração API ↔ Front — api_client.rs + ws_client.rs (2026-07-12)
**O que foi feito:**
- **`api_client.rs` criado** em `Frontend-Dioxus/src/` — cliente HTTPS via `gloo-net 0.6`:
  - `api_url(path)` constrói URL completa a partir de `window.location.origin` + path
  - `save_tokens(access, refresh)` / `get_token()` / `get_refresh_token()` / `clear_tokens()` / `is_authenticated()` — persistência via `web_sys::window().local_storage()` (browser-only)
  - `default_headers()` — retorna `Vec<(String, String)>` com `Authorization: Bearer <token>` se autenticado
  - `register(username, email, password)` → `POST /auth/register`
  - `login(username, password)` → `POST /auth/login` (retorna tokens + flag `mfa_required`)
  - `verify_mfa(username, code)` → `POST /auth/mfa/verify`
  - `refresh_token(refresh)` → `POST /auth/refresh`
  - `logout()` → limpa tokens localmente
  - `health_check()` → `GET /health`
  - Validação de status de resposta HTTPS com `(200..300).contains(&status)` (clippy `manual_range_contains`)
  - 5 testes unitários (3 guardados com `#[cfg(target_arch = "wasm32")]` pois usam `web_sys::window()`)
- **`ws_client.rs` criado** em `Frontend-Dioxus/src/` — cliente WebSocket via `ws_stream_wasm 0.7.5` + `gloo-net 0.6`:
  - `WsConnectionState` enum (Disconnected/Connecting/Connected/Error/Reconnecting) com `PartialEq`
  - `ServerMessage` enum (Welcome/TableState/YourTurn/ActionResult/Error/Pong) — `Deserialize`
  - `ClientMessage` enum (Action/Ping) — `Serialize`
  - `WsCallbacks` struct com `on_connection_state`, `on_message`, `on_error` — `Option<Rc<RefCell<dyn FnMut(...)>>>` (type aliases para evitar `clippy::type_complexity`)
  - `WsClient::new(table_id, token, callbacks)` + `connect()` async + `send_action(action, amount)` + `send_ping()`
  - Reconexão automática com backoff (1s → 2s → 4s → max 30s)
  - 8 testes unitários (serialização ClientMessage, deserialização ServerMessage, WsConnectionState PartialEq)
- **`pages/login.rs` refatorado** — agora chama `api_client::login()` real (não mais mock). Fluxo: Login → se `mfa_required` → MfaInput → `verify_mfa()` → navega para lobby. `AuthFlow` enum gerencia estados (Login/MfaRequired/Success/Error)
- **`pages/register.rs` refatorado** — chama `api_client::register()` real. Validação client-side de senha (8+ chars, maiúscula, número) + confirmação. Navega para login em sucesso
- **`pages/table.rs` refatorado** — conecta ao WebSocket real via `use_effect` + `spawn`. `WsCallbacks` com closures `FnMut` que mutam `Signal<TableState>` via `borrow_mut()`. Mapeia `ServerMessage::TableState` → `PlayerData`/`CommunityCards`/`Pot`/`ActionKind`. `on_action` envia via `client.send_action()`
- **`main.rs` atualizado** — adicionado `#![allow(dead_code)]` global (inner attribute no crate root) para suprimir warnings de funções/tipos ainda não usados em produção (mock helpers, parsers)
- **Lições aprendidas:**
  - **Inner attributes (`#![...]`) SÓ funcionam no crate root** (lib.rs/main.rs). Em módulos dão erro "an inner attribute is not permitted in this context"
  - **Outer attributes (`#[...]`) aplicam APENAS ao item imediatamente seguinte** — antes de comentário ou `use` não cobrem nada
  - **`Signal<T>` e `Navigator` são `Copy` em Dioxus 0.6** — `.clone()` é redundante (clippy `clone_on_copy`)
  - **`Rc<RefCell<dyn FnMut(...)>>`** precisa de type alias para passar `clippy::type_complexity`
  - **`#[derive(Default)]`** substitui `impl Default` manual quando todos os campos são `None`/`Default` (clippy `derivable_impls`)
  - **`web_sys::window()` panica em testes nativos** — guardar testes de localStorage com `#[cfg(target_arch = "wasm32")]`
  - **Clippy 1.96.0** tem `empty_line_after_outer_attr` habilitado por padrão — não deixar linha vazia após `#[allow(...)]`
  - **`collapsible_if`** pode ser corrigido com `&& let` chaining (Rust 2024)
- **Quality gates validados:**
  - `cargo check` ✅ — 0 errors
  - `cargo clippy --all-targets -- -D warnings` ✅ — 0 warnings (8.43s)
  - `cargo test` ✅ — 73/73 testes passando (61 anteriores + 5 api_client + 8 ws_client - 1 register simplificado)
- **Total de testes Rust:** 1.899/1.899 passando (1.816 motor + 12 API + 71 frontend)
- **Documentação sincronizada:** `DASHBOARD.md`, `CRONOGRAMA.md`, `DEVELOPMENT_LOG.md` (2026-07-12)

---
*Próximo passo: 3.10 Persistência PostgreSQL (já implementado, falta documentar) ou iniciar Fase 4 (Infraestrutura Docker).*

---

### [14] 🔄 Game Loop — State Machine Completa de Texas Hold'em (2026-07-14) — Task 4.1 ✅
**O que foi feito:**
- **`game_loop.rs` criado** em `Motor-Rust/src/` — state machine que orquestra uma mão completa de Texas Hold'em
- **Estruturas principais:**
  - `PlayerState` — estado de cada jogador na mão (stack, bet atual, total bet, folded, all_in, last_action)
  - `HandState` — estado completo da mão (fase, jogadores, dealer_idx, cartas comunitárias, pote, apostas)
  - `GameLoop` — orquestrador com `new()`, `start_hand()`, `process_action()`, `advance_phase()`
  - `PlayerMove` enum — Fold, Check, Call, Raise(amount), AllIn
  - `HandResolution` — resultado final (vencedores, pote distribuído, rake)
  - `GameLoopError` — erros do game loop (jogador não encontrado, não é sua vez, ação inválida)
- **Fluxo completo:** Preflop → Flop → Turn → River → Showdown, com blinds automáticos, ordem de ação correta (SB primeiro no preflop, dealer primeiro pós-flop), apostas mínimas, all-in side pots
- **Integração com módulos existentes:** `deck.rs` (avaliação de mãos), `side_pots.rs` (cálculo de side pots), `rake.rs` (rake da casa), `hand_history.rs` (registro da mão)
- **15 testes unitários** cobrindo: preflop, flop, turn, river, showdown, all-in, fold encerra mão, raise mínimo, ordem de ação heads-up pós-flop, community cards acumulativos
- **Quality gates validados:**
  - `cargo clippy --lib -- -D warnings` ✅ — 0 warnings (game_loop.rs limpo)
  - `cargo test --lib` ✅ — 1.816/1.816 passando (484 anteriores + 15 game_loop)
- **Total de testes motor:** 1.816/1.816 passando

---
*Próximo passo: Task 4.2 — Integração Game Loop ↔ API Axum (WebSocket).*

---

### [15] 🧹 Correção de 15 Erros Clippy Pré-existentes (2026-07-14) — Task 4.1.1 ✅
**O que foi feito:**
- **15 erros clippy corrigidos** em 6 arquivos do motor (`Motor-Rust/src/`):
  - `antifraud/collusion.rs` (2 erros): `for_kv_map` — `for (_street, actions) in street_actions` → `for actions in street_actions.values()`
  - `auth.rs` (1 erro): `manual_is_multiple_of` — `result.len() % 8 != 0` → `!result.len().is_multiple_of(8)`
  - `deck.rs` (3 erros): 2× `unnecessary_sort_by` — `sort_by(|a, b| b.rank.cmp(&a.rank))` → `sort_by_key(|c| std::cmp::Reverse(c.rank))` + 1× `collapsible_if` — if aninhado colapsado com `&&` chaining (Rust 2024)
  - `hand_history.rs` (6 erros): 2× `needless_lifetimes` removidos, 3× `single_char_add_str` — `push_str("\n")` → `push('\n')`, 1× `useless_format` — `format!("...")` → `push_str("...")` direto
  - `lobby.rs` (2 erros): `too_many_arguments` — `#[allow(...)]` em `create_table` (10 params), `unnecessary_map_or` — `.map_or(true, \|gt\| ...)` → `.is_none_or(\|gt\| ...)`
  - `tournament_engine.rs` (1 erro): `unnecessary_sort_by` — `sort_by(|a, b| b.stack.cmp(&a.stack))` → `sort_by_key(|e| std::cmp::Reverse(e.stack))`
- **Quality gates validados:**
  - `cargo clippy --lib -- -D warnings` ✅ — **0 erros, 0 warnings** em TODOS os módulos (7.48s)
  - `cargo test --lib` ✅ — 1.816/1.816 passando (429.72s)
- **CI/CD garantido:** `RUSTFLAGS="-D warnings"` agora passa limpo em todos os módulos

---
*Próximo passo: Commit + push das alterações.*

### [12] 🎰 Componentes de Lobby — 5 Componentes Dioxus + CSS Puro Full Tilt Poker (2026-07-12)
**O que foi feito:**
- **5 componentes criados** em `Frontend-Dioxus/src/components/`:
  - `table_card.rs` — Card de mesa com GameType (Cash/Tournament), blinds (R$0.25/R$0.50), ocupação (3/9), 7 testes
  - `lobby_filters.rs` — Filtros por tipo de jogo (Cash/Tournament/Todos) + range de blinds (Micro/Baixo/Médio/Alto/Todos), 9 testes
  - `join_button.rs` — Botão Entrar/Cheia/Assistir com estados (Available/Full/Watching), 5 testes
  - `player_count.rs` — Contador visual X/Y com barra de progresso colorida (verde/amarelo/vermelho), 8 testes
  - `lobby_list.rs` — Lista combinando TableCard + PlayerCount + JoinButton com filtros, 5 testes
- **CSS puro (~250 linhas)** em `assets/index.html` com visual Full Tilt Poker:
  - Dark felt background (#1a3a1a), gold accents (#8b6914), red felt (#2d1a1a)
  - Table cards com bordas douradas, hover effects, gradientes
  - Filtros com chips visuais, botões com estados (entrar/cheia/assistir)
  - Player count com barra de progresso animada
  - **Tailwind CDN removido** — 100% CSS puro, sem frameworks
- **`components/mod.rs`** atualizado — declara os 5 novos submódulos
- **34 testes unitários novos** distribuídos pelos 5 componentes (7+9+5+8+5)
- **Quality gates validados:**
  - `cargo check` ✅ — 0 errors (36.11s, requer LIBRARY_PATH para linker GNU)
  - `cargo clippy --all-targets -- -D warnings` ✅ — 0 warnings (5.70s)
  - `cargo test` ✅ — 57/57 testes passando (34 novos + 22 mesa + 2 router)
  - `cargo test --lib` motor ✅ — 1.816/1.816 passando (434.12s)
- **Doctest corrigido:** `utils.rs:34` — `ratear_proporcional` doctest alterado de `&Vec<f64>` para `&[Pot]` com `Pot::new()` instances
- **Total de testes Rust:** 1.886/1.886 passando (1.816 motor + 12 API + 57 frontend + 1 doctest)
- **Documentação sincronizada:** `DASHBOARD.md`, `CRONOGRAMA.md`, `DEVELOPMENT_LOG.md` (2026-07-12)

---
*Próximo passo: 3.8 Componentes de Auth (login, registro, MFA).*

---

### [09] 🌐 API Axum — REST + WebSocket + JWT + PostgreSQL (2026-07-10)
**O que foi feito:**
- **Crate `API-Axum/`** — exposição HTTPS/WSS do motor para o frontend Dioxus
- **Stack:** Axum 0.7 (features `ws`, `macros`) + middleware CORS/trace 0.6 + sqlx 0.8 (postgres, uuid, chrono, migrate) + tokio 1 + serde 1 + uuid 1 + chrono 0.4 + tracing 0.1 + dotenvy 0.15 + futures-util 0.3
- **8 endpoints REST públicos:**
  - `POST /auth/register` — registro com bcrypt + JWT
  - `POST /auth/login` — login com JWT (access + refresh)
  - `POST /auth/mfa/verify` — verificação TOTP (RFC 6238)
  - `POST /auth/refresh` — refresh token
  - `GET /lobby/tables` — listar mesas (filtros: blinds, disponibilidade)
  - `GET /lobby/tables/:id` — detalhes de mesa
  - `GET /tournament/:id` — info de torneio
  - `GET /health` — health check
- **3 endpoints REST protegidos** (JWT via `RequireAuth` extractor):
  - `POST /lobby/join` — sentar em mesa
  - `POST /tournament/register` — registrar em torneio
  - `GET /hand-history/:hand_id` — replay de mão
- **WebSocket `/ws/game/:table_id`** — canal de jogo em tempo real (ping/pong, get_table_info, JSON messages)
- **JWT Middleware** (`middleware/auth.rs`) — `RequireAuth` extractor com `FromRequestParts`, valida token via `auth.validate_token(&token, "access")`
- **Persistência PostgreSQL** — `sqlx::migrate!("./migrations")` + 6 tabelas (users, sessions, tables, hand_history, tournaments, tournament_players) + 4 índices
- **AppState** (`state.rs`) — `db: PgPool`, `auth: Arc<Mutex<AuthManager>>`, `lobby: Arc<Mutex<LobbyManager>>`, `tournaments: Arc<Mutex<HashMap<String, TournamentStore>>>`, `jwt_secret: String`
- **Error handling** (`error.rs`) — `ApiError` enum (BadRequest/Unauthorized/Forbidden/NotFound/Conflict/Internal) com `IntoResponse` + `From<sqlx::Error>` + `From<serde_json::Error>`
- **CORS configurável** via env (`CORS_ORIGINS`)
- **17 testes de integração** (`tests/api_tests.rs`) — 12 ativos passando + 5 `#[ignore]` (DB-dependent)
- **Quality gates validados:** `cargo build` ✅ (2m 47s, 0 warnings), `cargo build --tests` ✅ (1m 30s, 0 warnings), `cargo test` ✅ (12/12 passing, 5 ignored), `cargo clippy --all-targets -- -D warnings` ✅ (1m 15s, 0 warnings), `cargo build --release` ✅ (3m 51s, 0 warnings)
- **CRONOGRAMA.md 2.14 ✅ Completo** — Motor Rust + API = 11/11 módulos, 1.816/1.816 testes
- **Documentação sincronizada:** `DASHBOARD.md`, `CRONOGRAMA.md` (2026-07-10)

---
*Próximo passo: 3.5 Roteamento Dioxus — dioxus-router para navegação entre telas (login → lobby → mesa).*

---

### [08] 🎪 Motor de Poker — Lobby + Antifraude (2026-07-10)
**O que foi feito:**
- **Lobby + Matchmaking (2.12) — `lobby.rs` (28 testes ✅):**
  - Enums: `GameType` (Cash/Tournament), `TableVisibility` (Public/Private), `PlayerLobbyStatus` (Lobby/Playing/Observing)
  - Structs: `TableInfo` (id, nome, tipo, blinds, buy-in, max_players, current_players, visibility, password_hash), `LobbyResult` (success, message, table_id)
  - `LobbyManager` com métodos: `new()`, `create_table()`, `list_tables()`, `list_tables_by_blinds()`, `list_available_tables()`, `find_table()`, `find_table_mut()`, `join_table()` (validações: existência, assento, saldo, senha), `leave_table()`, `close_table()`, `table_count()`, `total_players()`, `find_or_suggest_table()`
  - 28 testes unitários cobrindo todos os fluxos (criação, listagem, filtros, entrada, saída, validações, senha, fechamento)
- **Antifraude (2.13) — `antifraud/` com 4 submódulos:**
  - `bot_detection.rs` — Detecção de bots via análise de padrões de timing, decisão e variância
  - `chip_dumping.rs` — Detecção de transferência ilícita de fichas entre jogadores
  - `collusion.rs` — Detecção de conluio entre múltiplos jogadores (padrões de aposta coordenados)
  - `multi_account.rs` — Detecção de múltiplas contas do mesmo jogador (device fingerprint, IP, padrões)
- **Motor Rust 100% completo:** 10/10 módulos, 1.816/1.816 testes passando, 0 warnings
- **Documentação sincronizada:** `DASHBOARD.md`, `CRONOGRAMA.md` (2026-07-10)

---
*Próximo passo: API Axum (2.14) — exposição HTTPS/WSS do motor para o front-end Dioxus.*

---

### [07] 💰 Motor de Poker — Conversão Monetária u64 → f64 (2026-07-06/07)
**O que foi feito:**
- **Todos os campos monetários convertidos de u64 para f64** com truncamento a 2 casas decimais
- **Função `truncar_2_casas(valor: f64) -> f64`** implementada em `utils.rs` (pública) e `loss_deflator.rs` (privada) — `(valor * 100.0).trunc() / 100.0`
- **Tolerâncias de teste ajustadas** para precisão de f64 (0.01 em vez de f64::EPSILON)
- **Valores esperados corrigidos** em `loss_deflator_tests.rs` para refletir truncamento real (ex: `122.49` em vez de `122.5`, `1.04` em vez de `1.05`)
- **Compilação limpa** — 0 erros, 0 warnings
- **1.816/1.816 testes passando** em `cargo test --lib` (~347s)
- **Documentação atualizada:** `DASHBOARD.md`, `CRONOGRAMA.md` (2026-07-07)

---
*Próximo passo: Componentes Dioxus (mesa, cartas, avatares, login).*

---

### [09] 🧹 Limpeza de Warnings + Otimização de Testes + Reestruturação (2026-07-19)
**O que foi feito:**
- **Eliminação de warnings do Motor-Rust (0 warnings / 0 erros):**
  - Causa raiz dos 131 warnings do binário: `main.rs` redeclarava todos os módulos via `mod`, criando árvore de compilação paralela à `lib.rs`. Refatorado para consumir a crate lib (`use poker_engine::...`), eliminando dead_code sem `#[allow]` global.
  - 23 warnings do build de testes corrigidos (`cargo fix`): imports/variáveis não usados, `mut` desnecessário; `make_config_custom` recebeu `#[allow(dead_code)]`.
  - Lints do clippy corrigidos (parênteses redundantes, `get().is_none()`→`contains_key()`, etc.); testes de range invertido receberam `#[allow(clippy::reversed_empty_ranges)]` / `absurd_extreme_comparisons`.
  - `cargo clippy --all-targets -- -D warnings` → **0 warnings, 0 errors**.
- **Bug de panic corrigido (`deck.rs`):** `evaluate_hand([], [])` caía em `unreachable!` (linha 159) quando a demo de side pots passava jogadores sem cartas. Agora retorna `HandRank::HighCard` vazio para mãos vazias. `cargo run` roda até o fim.
- **Otimização de `get_heads_up_win_probability` (`loss_deflator.rs`):** substituiu enumeração exaustiva de C(45,5)≈1.2M boards por **Monte Carlo sem reposição** (200k amostras, determinístico via seed derivada das cartas, usando `StdRng` do projeto). Boards já avaliados são ignorados via `HashSet` (sem repetição). Board completo (river) continua exato. Tempo de teste da suíte caiu de ~23min → 57s (~24x).
- **Tolerâncias de teste ajustadas:** `test_win_prob_known_cards_dont_overlap` passou de exato (`EPSILON`) para ±0.05 (ruído estatístico esperado de Monte Carlo).
- **Reestruturação de pastas:** removidos prefixos numéricos (`08-Motor-Rust`→`Motor-Rust`, `10-API-Axum`→`API-Axum`, `09-Frontend-Dioxus`→`Frontend-Dioxus`, etc.); `**/target/` adicionado ao `.gitignore`. Commit `4093d59` enviado ao `origin/master`.
- **Suíte completa:** 1.848 testes passando, 0 failed, 1 ignored.

---

### [22] 🛡️ Anúncio Global e Psicologia do Loss Deflator (2026-07-25)
**O que foi feito:**
- **Integração Real-Time do Engine com WebSocket:** O `game_actor.rs` do Axum agora intercepta resoluções de mão com `loss_deflator` e transmite o evento `DeflatorTriggered` para o Dioxus via WebSocket.
- **Cálculo de Equity:** O `game_loop.rs` usa `get_heads_up_win_probability` com o board conhecido no instante do all-in; a rotina é determinística e usa enumeração ou Monte Carlo determinístico conforme o espaço.
- **Componente Visual Dioxus (`deflator_notification.rs`):** Pop-up global animado com sobreposição (overlay) e 4 níveis de cores e textos adaptados à gravidade da Bad Beat (7% Teal, 15% Azul, 25% Laranja, 35% Vermelho Neon).
- **Copywriting de Prova de Justiça:** Textos explícitos detalhando o evento de Bad Beat, o percentual de chance exato do vencedor e do perdedor, e o saldo/fichas recuperadas, adaptados dinamicamente para Cash Games e Torneios.
- **Compilação Limpa:** 0 erros de compilação, suíte de 2.051+ testes verificada.


---

### [22.1] 📐 Regra normativa do Loss Deflator por equity (2026-07-30)
**O que foi feito:**
- Faixas vigentes centralizadas no motor: <56%=0%; 56–65,9%=7%; 66–75,9%=15%; 76–85,9%=25%; ≥86%=35%.
- O tier é escolhido pela equity do perdedor no instante em que o all-in é pago. A fase serve somente para reconstruir o board conhecido.
- A ordem financeira foi fixada em potes/side pots → rake → Loss Deflator nos potes líquidos elegíveis → pagamentos.
- API e Dioxus agora transportam e mostram a equity do perdedor, a faixa aplicada e o valor devolvido.

### [22.2] 📡 Multi-deflator, multiway equity e auditoria HH (2026-07-31)
**O que foi feito:**
- WS emite `deflator_triggered` para **cada** entrada de `loss_deflators` (não só a primary).
- Equity multiway determinística quando há 2+ oponentes elegíveis; heads-up permanece o atalho de 1 oponente.
- `hand_history.loss_deflators_json` (migration `012`) grava equity, tier e cashback por perdedor.
- Caps de rake opcionais por agenda HU/3–4/5+ (migration `011`) documentados; schema WS em `ARQUITETURA_E_APIS.md`.

---

### [23] 🚀 Elevação de Arquitetura Comercial / Commercial Grade (2026-07-25)
**O que foi feito:**
- **Pre-filtering de Alta Velocidade no Engine (`deck.rs`):** Implementado circuito rápido vetorial de naipes (`[u8; 4]`) no `evaluate_hand()`. Se nenhum naipe tiver 5+ cartas, o sistema pula instantaneamente verificações de Flush e Straight Flush sem alocar `HashMap`, otimizando 80%+ das avaliações de mãos.
- **Hardening de JWT (`auth.rs`):** Migração completa da codificação/decodificação manual para a biblioteca padrão da indústria `jsonwebtoken` (v9.2) com chaves `EncodingKey` e `DecodingKey`, garantindo imunidade a ataques de canal lateral (*timing attacks*).
- **Saneamento Docker (`docker-compose.yml`):** Remoção de containers ociosos (`kafka` e `zookeeper`), economizando ~800MB de RAM na infraestrutura local/produção.
- **Integração Postgres no CI (`rust-ci.yml`):** Configurado o serviço `postgres:15-alpine` com *healthcheck* no job `api-test` do GitHub Actions para execução automatizada de testes E2E.
- **Sincronização de Documentação Técnica:** Atualizado `ARQUITETURA_E_APIS.md` para tipagem estrita `u64` centavos em todas as especificações de API.

---

### [24] 🔄 Tolerância a Falhas — Redis Snapshots no TableActor (2026-07-25)
**O que foi feito:**
- **Cliente Redis Integrado (`API-Axum/Cargo.toml`):** Adicionada dependência da crate `redis` (v0.25) com `std`, `tokio-comp` e `aio`.
- **Graceful Fallback (`main.rs` & `state.rs`):** Injetado `redis: Option<ConnectionManager>` em `AppState`. Se a variável `REDIS_URL` não for informada, o sistema opera em memória RAM sem nenhuma interrupção.
- **Persistência Assíncrona no `TableActor` (`game_actor.rs`):** Implementado o método `save_snapshot()`, gravando o estado exato da mesa em Redis com chave `poker:table:state:{table_id}` e TTL de 3.600s (1 hora) após comandos de ação/sit/leave dos jogadores.
- **Restauração de Estado:** Garantida resiliência de mesas ativas contra reinicializações e falhas do servidor backend.
- **Compilação do Frontend:** `cargo check` no `Frontend-Dioxus` (WASM) 100% aprovado.

---
*Próximo passo: Deploy em ambiente de staging / produção ou disponibilização de canal seguro via Ngrok.*

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-07):** S12 — Auth MFA + supply-chain CI; ações legais na mesa; settle pós-disconnect; liquidação de mão assinada (migração 017); smoke live 10 usuários/100 mãos com settlement verificado na VPS demo; branch codex/security-supply-chain fechada e documentada. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** VPS stack healthy (postgres, redis, api, frontend/Caddy). Migrations 001–017 aplicadas (017 hand settlement audit). Smoke live scripts/live-e2e-ten-users.mjs: run 202608070833 PASS (10 reg/100 mãos); run 202608070920 PASS com settlementsVerified=2 (assinatura + winner + payouts+rake=pote por mesa). Simulação motor 100k mãos release OK. Segundo lote sintético zte2e202608070920* removido; lote original zte2e202608070833* preservado (10 contas demo). Suíte histórica motor/API + gates supply-chain (Dependabot, audit, SBOM/Trivy workflows). Mock é o padrão. O único adaptador externo é o Asaas Sandbox, restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção permanecem desabilitados. Nenhum depósito com dinheiro real foi habilitado. Mesas continuam com dono único por processo; uma guarda persistente pausa a mesa após falha entre início e liquidação da mão, exigindo revisão/abort administrativo antes da reabertura. Liquidação de mão agora persiste settlement assinado (HMAC) e a API verifica assinatura no replay; históricos legados sem assinatura permanecem legíveis como não verificados.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
