# 🏗️ Arquitetura do Motor Central da Plataforma de Poker Online

**Versão:** 4.0  
**Data:** 2026-09-22
**Status:** Documento oficial — fonte da verdade para decisões de arquitetura de **motor e stack**

> Este documento é a **fonte da verdade** sobre a arquitetura da plataforma (motor, API, frontend e camadas). Qualquer decisão de design, escolha de tecnologia ou nova pasta deve ser consultada aqui **antes** de iniciar a codificação.
>
> **Estado operacional** (ciclo, PIX, ownership, certificação, frontend TS): prevalece [`Documentacao/STATUS_OPERACIONAL.md`](../Documentacao/STATUS_OPERACIONAL.md). Transporte público: **HTTPS**.
>
> **Regulação / compliance de jogo e dinheiro real:** trilho planejado para **janeiro de 2027** (não bloqueia demo play-money).

---

## 1. 📐 Metodologia — contratos e fontes de verdade

- **SDD (Spec-Driven Development):** todas as funcionalidades começam com especificações formais (contratos de API, schemas JSON, regras de negócio).
- Specs são a "fonte da verdade" e guiam o desenvolvimento do **motor e da API em Rust** e do **frontend em TypeScript**.
- O contrato de agentes (`AGENTS.md`) e o mapa de donos impedem documentação concorrente.
- `STATUS_OPERACIONAL.json` é a fonte máquina dos fatos operacionais; `documentation-sync` valida e gera a leitura humana.

---

## 2. 🏗️ Arquitetura de Linguagens — Stack híbrida (STACK ALVO v4.0)

> **Atualizado em 2026-08-04:**  
> - **Backend crítico (motor, API, antifraude, pagamentos, ledger):** **Rust**  
> - **Frontend (UI jogador + admin B2B):** **TypeScript + React + Vite + Tailwind CSS** em `Frontend-Web/`  
> - **Direção visual:** moderno, denso, inspirado no **Full Tilt** clássico (felt, rail dourado, lobby tabular) — sem estética genérica de “site feito por IA”  
> - O antigo `Frontend-Dioxus/` (WASM) foi **removido** do monorepo; recuperável só pelo histórico git

| Camada | Linguagem | Responsabilidade |
|--------|-----------|------------------|
| **Motor de jogo, API, antifraude, auth server-side, ledger** | **Rust** | Regras de poker, RNG, side pots, rake, loss deflator, REST/WSS (Axum), persistência |
| **Front-end (SPA)** | **TypeScript (React)** | Lobby, mesa, login/registro, admin clubs; CSS/Tailwind; same-origin via Caddy |
| **Comunicação** | **JSON** | Contratos REST + mensagens WebSocket |

### 2.1 🦀 Por que Rust no backend e no motor?

- **Performance:** cálculo de mãos em tempo real sem latência perceptível.
- **Segurança de memória:** elimina classes inteiras de bugs (buffer overflow, use-after-free).
- **Concorrência:** modelo async/await nativo, ideal para milhares de conexões WebSocket simultâneas.
- **Criptografia:** TLS é terminado pelo Caddy; `aes-gcm` protege chaves PIX no payout worker e HMAC assina settlements/webhooks.
- **RNG criptograficamente seguro:** essencial para integridade do jogo.
- **Dinheiro em `u64` centavos:** precisão bancária no motor e na API.

### 2.2 🖥️ Por que TypeScript no Frontend (decisão 2026-08)?

- **Velocidade de UI/UX:** design system Full Tilt + Tailwind, iteração de layout sem rebuild WASM.
- **Ecossistema web maduro:** tooling, designers, componentes e acessibilidade.
- **Contrato estável com a API:** o frontend **não** reimplementa regras de dinheiro; só consome JSON/WSS.
- **Deploy mais leve:** build Node/Vite em minutos vs. toolchain `wasm32` + `wasm-bindgen` na VPS.
- **Regra abandonada:** “100% Rust incluindo frontend” — o motor e a API continuam 100% Rust; a UI não.

### 2.3 🔗 Comunicação entre Camadas — JSON + WebSocket

```
┌─────────────────────────────────────────────────────────────────────┐
│              🔐 CAMADA DE SEGURANÇA (transversal a tudo)            │
│   TLS 1.3 · JWT · MFA · rate limit · antifraude · Caddy headers    │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────┐      JSON/HTTPS (TLS)     ┌──────────────┐
│   Front-end  │ ◄───────────────────────► │   Backend    │
│ TypeScript   │   WebSocket Seguro (WSS)  │  Rust Axum   │
│ React + Vite │   JWT + ticket WS         │  + Motor     │
└──────┬───────┘                           └──────┬───────┘
       │                                         │
       │                                         ▼
       │                                  ┌──────────────┐
       │                                  │  PostgreSQL  │
       │                                  │  + Redis     │
       │                                  └──────────────┘
```

- **TypeScript (React) ↔ Rust (Axum):** HTTPS REST + WSS (eventos de jogo em tempo real), same-origin atrás do Caddy.
- **Presença online:** `GET /api/presence/online` (público) + `POST /api/presence/heartbeat` (JWT); Redis ZSET com TTL 90s; UI badge no header e hero na home.
- **Formato universal:** JSON em todas as fronteiras (schemas validados no servidor).
- **🔐 Segurança:** TODA comunicação usa TLS 1.3. TODO dado sensível é criptografado.

---

## 3. 🐳 Infraestrutura — Docker Compose e caminho Kubernetes

- **Docker Compose (vigente):** PostgreSQL 15, Redis 7, API Axum e frontend/Caddy.
- **Caddy:** termina HTTPS e encaminha `/api` e `/ws` para a API pela rede interna.
- **Kubernetes (manifesto de referência):** existe para validação com uma réplica; não representa alta disponibilidade nem ownership distribuído de mesas.
- **Limite atual:** uma mesa pertence a um único processo. Escala horizontal exige ownership/roteamento explícito antes de múltiplas réplicas.

---

## 4. 💾 Dados e coordenação — PostgreSQL e Redis

- **PostgreSQL:** estado durável, ledger, catálogo, histórico e auditoria.
- **Redis:** presença, rate limiting distribuído e coordenação efêmera suportada pela API.
- **Atores Tokio em processo:** serializam o estado de cada mesa e torneio; WebSocket entrega os eventos aos clientes.
- **Kafka/RabbitMQ:** não fazem parte da stack vigente.
- **JSON:** contratos REST/WSS validados pelo servidor; valores monetários permanecem em centavos inteiros.

---

## 5. 🔐 Segurança — Camada Transversal da Plataforma

> 🔐 **Conceito:** Segurança não é "uma camada" — é uma **preocupação transversal** que aparece em **todos** os pontos da arquitetura. Pense como o cinto de segurança do carro: está em todos os assentos, não em uma "camada de cinto".

### 5.1 🔑 Criptografia — TLS, AES-256 e bcrypt

| Onde | O quê | Ferramenta (Rust) | Analogia |
|------|-------|------------------|----------|
| **Em trânsito** | HTTPS/WSS | Caddy | O gateway termina TLS e encaminha somente pela rede interna |
| **Chave PIX de saque** | AES-256-GCM | `aes-gcm` | A chave é cifrada antes da persistência |
| **Senhas** | Hash adaptativo | `bcrypt` | A senha original não é armazenada |
| **Liquidação/webhooks** | Autenticidade | HMAC-SHA256 | Detecta alteração de mensagens financeiras |

### 5.2 🛡️ Autenticação e Autorização — JWT, MFA e RBAC

| Mecanismo | O que faz | Onde |
|-----------|-----------|------|
| **JWT** (JSON Web Token) | Token que prova identidade sem reenviar senha | Backend Rust emite, Front-end armazena |
| **MFA** (Multi-Factor Auth) | Segundo fator (senha + código no celular) | Front-end pede, Backend valida |
| **RBAC** (Role-Based Access Control) | "Admin pode X, jogador pode Y" | Backend Rust valida em cada rota |

### 5.3 🕵️ Antifraude e Auditoria — Detecção de Bots e Collusion

| Componente | Responsabilidade | Linguagem |
|------------|------------------|-----------|
| **Antifraude** | Regras e sinais para bots, collusion e chip dumping | Rust |
| **Telemetria da API** | Logs estruturados e métricas medidas pelo processo | `tracing` + endpoint administrativo |
| **Saúde** | Readiness de API, PostgreSQL, Redis e gateway | healthchecks do Compose/Caddy |
| **Auditoria financeira/administrativa** | Ações e metadados persistidos | PostgreSQL `audit_logs` |

### 5.3.1 Bots jogadores e coach não são o mesmo componente

- `API-Axum/src/bots.rs` mantém 72 contas técnicas, quatro personalidades e a estratégia `lag_v2` para Play Money, testes e operação administrativa. Elas enviam comandos internos ao ator e não autenticam como usuários.
- O futuro coach trabalha somente sobre `hand_history` encerrado e verificado. Ele reutiliza tipos, ações legais e avaliadores do motor, mas não copia a política de decisão dos bots nem participa do `TableActor`.
- A separação evita que heurísticas criadas para produzir adversários variados sejam apresentadas ao aluno como estratégia ótima e mantém qualquer recomendação fora do caminho de jogo ao vivo.

### 5.4 ⚖️ Conformidade — PCI DSS e LGPD

- A plataforma não processa cartões; PIX não elimina as obrigações legais aplicáveis.
- **LGPD:** termos, privacidade e minimização de dados estão documentados, mas isso não equivale a certificação.
- **Regulação/KYC:** trilho planejado para 2027-01; a demo atual não é certificada para produção.
- **JWT curtos** → tokens expiram rápido (15-30 min) para reduzir janela de ataque.

### 5.5 🔄 Diagrama de Fluxo Seguro — Login e Jogada

```
Jogador                    Front-end (TypeScript/React)    Backend (Rust)              Banco (PG)
   │                            │                            │                          │
   │ 1. Login (senha+MFA)      │                            │                          │
   ├───────────────────────────►│                            │                          │
   │                            │ 2. POST /login (TLS)      │                          │
   │                            ├───────────────────────────►│                          │
   │                            │                            │ 3. bcrypt.verify(senha)   │
   │                            │                            │                          │
   │                            │ 4. JWT + refresh token     │                          │
   │                            │◄───────────────────────────┤                          │
   │ 5. Token armazenado        │                            │                          │
   │◄───────────────────────────┤                            │                          │
   │                            │                            │                          │
   │ 6. Jogada (com JWT)        │                            │                          │
   ├───────────────────────────►│                            │                          │
   │                            │ 7. WSS + JWT (TLS)         │                          │
   │                            ├───────────────────────────►│                          │
   │                            │                            │ 8. Validar JWT            │
   │                            │                            │ 9. Motor calcula          │
   │                            │                            │ 10. Persistir/auditar     │
   │                            │                            ├─────────────────────────►│
   │                            │                            │                          │
   │                            │ 11. Resultado (JSON)       │                          │
   │                            │◄───────────────────────────┤                          │
   │ 12. UI atualiza            │                            │                          │
   │◄───────────────────────────┤                            │                          │
   │                            │                            │                          │
   │                            │     [LOG] tracing +        │                          │
   │                            │     audit_logs             │                          │
```

---

## 6. 🔄 Fluxo de Dados — Da Jogada ao Resultado

1. Jogador faz jogada no front-end (TypeScript/React) → envia via WebSocket.
2. Backend Rust recebe → valida → motor calcula → resultado em JSON.
3. O ator serializa a mudança, persiste o que é durável no PostgreSQL e transmite o novo estado por WebSocket.
4. Antifraude e auditoria processam sinais dentro da API; Redis sustenta estado efêmero distribuído.
5. Front-end TypeScript/React consome APIs → renderiza mesas, dashboards.
6. CI, `documentation-sync`, testes e endpoints administrativos verificam contratos e operação.

---

## 7. 📋 Governança — contratos, CI e operação

- **Contratos:** `BUSINESS_RULES.md`, `ARQUITETURA_E_APIS.md` e este documento definem regras e fronteiras.
- **CI:** fmt, clippy, testes determinísticos, frontend e `documentation-sync` formam o gate local/contínuo.
- **Operação:** fatos vigentes vivem no STATUS; deploy e carga exigem procedimentos próprios.
- **Stack:** Rust concentra motor/API/antifraude; TypeScript/React concentra a experiência web.

---

## 8. ✅ Estado Atual da Implementação — Módulos e Testes

> ⚡ **Stack v4.0:** Rust no motor/API; TypeScript no frontend (`Frontend-Web`). Node só no build da SPA.

| Pasta | Conteúdo | Status |
|-------|----------|--------|
| `Infraestrutura-Docker/` | Docker Compose + Caddy (PostgreSQL 15, Redis 7; sem Kafka) | ✅ Ativo |
| `Documentacao/` | Documentação do projeto | ✅ Ativo |
| `Arquitetura-Motor/` | Este documento | ✅ Ativo |
| `Motor-Rust/` | Motor de jogo Rust; contagem de testes deve ser obtida na execução vigente | ✅ Ativo |
| `Frontend-Web/` | Front-end TypeScript + React + Vite + Tailwind (Full Tilt skin) | ✅ Ativo (deploy canônico) |
| `API-Axum/` | API HTTPS/WSS (Axum + Tokio) | ✅ Ativo |

---

## 9. Consultar antes de mudar a stack

Antes de nova pasta ou decisão de arquitetura: este arquivo + `Documentacao/BUSINESS_RULES.md`. Fatos do dia: `STATUS_OPERACIONAL.md`. Contrato de agentes: `AGENTS.md` na raiz (não criar arquivo/pasta nova se já existe dono). Gates: `QUALITY.md` (não é enciclopédia).

---

**Próxima revisão:** antes de introduzir múltiplas réplicas da API ou mensageria externa.
