# 🏗️ Arquitetura do Motor Central da Plataforma de Poker Online

**Versão:** 4.0  
**Data:** 2026-08-04  
**Status:** Documento oficial — fonte da verdade para decisões de arquitetura de **motor e stack**

> Este documento é a **fonte da verdade** sobre a arquitetura da plataforma (motor, API, frontend e camadas). Qualquer decisão de design, escolha de tecnologia ou nova pasta deve ser consultada aqui **antes** de iniciar a codificação.
>
> **Estado operacional** (ciclo S10+, PIX, ownership, certificação, frontend TS): prevalece [`Documentacao/STATUS_OPERACIONAL.json`](../Documentacao/STATUS_OPERACIONAL.json). Transporte público: **HTTPS**.
>
> **Regulação / compliance de jogo e dinheiro real:** trilho planejado para **janeiro de 2027** (não bloqueia demo play-money).

---

## 1. 📐 Metodologia — SDD e MCP

- **SDD (Spec-Driven Development):** todas as funcionalidades começam com especificações formais (contratos de API, schemas JSON, regras de negócio).
- Specs são a "fonte da verdade" e guiam o desenvolvimento do **motor e da API em Rust** e do **frontend em TypeScript**.
- **MCP (Middleware Control Plane):** garante que os serviços sigam as specs, centraliza logs e políticas de segurança.
- **WebMCP:** interface web para admins configurarem specs, monitorarem serviços e acessarem relatórios.

---

## 2. 🏗️ Arquitetura de Linguagens — Stack híbrida (STACK ALVO v4.0)

> **Atualizado em 2026-08-04:**  
> - **Backend crítico (motor, API, antifraude, pagamentos, ledger):** **Rust**  
> - **Frontend (UI jogador + admin B2B):** **TypeScript + React + Vite + Tailwind CSS** em `Frontend-Web/`  
> - **Direção visual:** moderno, denso, inspirado no **Full Tilt** clássico (felt, rail dourado, lobby tabular) — sem estética genérica de “site feito por IA”  
> - `Frontend-Dioxus/` permanece como **legado** (não é o deploy canônico)

| Camada | Linguagem | Responsabilidade |
|--------|-----------|------------------|
| **Motor de jogo, API, antifraude, auth server-side, ledger** | **Rust** | Regras de poker, RNG, side pots, rake, loss deflator, REST/WSS (Axum), persistência |
| **Front-end (SPA)** | **TypeScript (React)** | Lobby, mesa, login/registro, admin clubs; CSS/Tailwind; same-origin via Caddy |
| **Comunicação** | **JSON** | Contratos REST + mensagens WebSocket |

### 2.1 🦀 Por que Rust no backend e no motor?

- **Performance:** cálculo de mãos em tempo real sem latência perceptível.
- **Segurança de memória:** elimina classes inteiras de bugs (buffer overflow, use-after-free).
- **Concorrência:** modelo async/await nativo, ideal para milhares de conexões WebSocket simultâneas.
- **Criptografia:** crates auditados (`ring`, `rustls`, `aes-gcm`) para TLS 1.3 e AES-256.
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
- **Formato universal:** JSON em todas as fronteiras (schemas validados no servidor).
- **🔐 Segurança:** TODA comunicação usa TLS 1.3. TODO dado sensível é criptografado.

---

## 3. 🐳 Infraestrutura — Docker e Kubernetes

- **Docker** → cada módulo em container isolado.
- **Kubernetes** → orquestração, escalabilidade automática, alta disponibilidade.
- **MCP** → plano de controle que valida specs, centraliza logs e políticas de segurança.
- **WebMCP** → painel web para admins configurarem specs, monitorarem serviços e acessarem relatórios.

---

## 4. 💾 Dados e Mensageria — PostgreSQL e Kafka

- **PostgreSQL** → banco de dados central, com criptografia em colunas sensíveis.
- **Kafka/RabbitMQ** → mensageria em tempo real para eventos de jogo e estatísticas.
- **JSON schemas** → validação de dados para evitar injeções e corrupção.

---

## 5. 🔐 Segurança — Camada Transversal da Plataforma

> 🔐 **Conceito:** Segurança não é "uma camada" — é uma **preocupação transversal** que aparece em **todos** os pontos da arquitetura. Pense como o cinto de segurança do carro: está em todos os assentos, não em uma "camada de cinto".

### 5.1 🔑 Criptografia — TLS, AES-256 e bcrypt

| Onde | O quê | Ferramenta (Rust) | Analogia |
|------|-------|------------------|----------|
| **Em trânsito** (dados voando na rede) | TLS 1.3 | `rustls`, `ring` | "Conversa sussurrada" — ninguém escuta |
| **Em repouso** (dados no banco) | AES-256 | `aes-gcm` | "Cofre trancado" — mesmo roubando o disco, não lê |
| **Senhas** | bcrypt / argon2 | `bcrypt` crate | "Senha vira sopa de letras" — irreversível |

### 5.2 🛡️ Autenticação e Autorização — JWT, MFA e RBAC

| Mecanismo | O que faz | Onde |
|-----------|-----------|------|
| **JWT** (JSON Web Token) | Token que prova identidade sem reenviar senha | Backend Rust emite, Front-end armazena |
| **MFA** (Multi-Factor Auth) | Segundo fator (senha + código no celular) | Front-end pede, Backend valida |
| **RBAC** (Role-Based Access Control) | "Admin pode X, jogador pode Y" | Backend Rust valida em cada rota |

### 5.3 🕵️ Antifraude e Auditoria — Detecção de Bots e Collusion

| Componente | Responsabilidade | Linguagem |
|------------|------------------|-----------|
| **Antifraude** | Detectar bots, collusion, chip dumping | Rust (ML) |
| **Logs centralizados** | Registrar TODAS as ações (quem, quando, onde, o quê) | ELK Stack |
| **Monitoramento** | Alertas em tempo real (ataques, falhas) | Grafana + Prometheus |
| **Auditoria** | Trilha imutável para investigação | Logs append-only |

### 5.4 ⚖️ Conformidade — PCI DSS e LGPD

- **PCI DSS** → padrão internacional para dados de cartão de crédito.
- **LGPD** → Lei Geral de Proteção de Dados (Brasil).
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
   │                            │                            │ 10. Gravar (AES-256)      │
   │                            │                            ├─────────────────────────►│
   │                            │                            │                          │
   │                            │ 11. Resultado (JSON)       │                          │
   │                            │◄───────────────────────────┤                          │
   │ 12. UI atualiza            │                            │                          │
   │◄───────────────────────────┤                            │                          │
   │                            │                            │                          │
   │                            │     [LOG] Tudo isso        │                          │
   │                            │     vai pro ELK Stack      │                          │
```

---

## 6. 🔄 Fluxo de Dados — Da Jogada ao Resultado

1. Jogador faz jogada no front-end (TypeScript/React) → envia via WebSocket.
2. Backend Rust recebe → valida → motor calcula → resultado em JSON.
3. Backend Rust registra no PostgreSQL → publica evento em Kafka.
4. Backend Rust (IA/antifraude) consome eventos → gera estatísticas, detecta fraude → resultados em JSON.
5. Front-end TypeScript/React consome APIs → renderiza mesas, dashboards.
6. MCP valida specs e segurança → WebMCP mostra status e relatórios.

---

## 7. 📋 Governança — MCP, WebMCP e SDD

- **MCP:** garante conformidade com specs, segurança e auditoria.
- **WebMCP:** interface para admins configurarem, monitorarem e ajustarem o sistema.
- **SDD:** metodologia que mantém consistência e qualidade em todas as camadas.
- **Rust unificado:** uma única linguagem para backend, IA, dados e antifraude simplifica governança, reduz pontos de falha e elimina fricção entre equipes.

---

## 8. ✅ Estado Atual da Implementação — Módulos e Testes

> ⚡ **Stack v4.0:** Rust no motor/API; TypeScript no frontend (`Frontend-Web`). Node só no build da SPA.

| Pasta | Conteúdo | Status |
|-------|----------|--------|
| `Infraestrutura-Docker/` | Docker + Deploy (PostgreSQL 15, Redis 7, Kafka) | ✅ Ativo |
| `Documentacao/` | Documentação do projeto | ✅ Ativo |
| `Arquitetura-Motor/` | Este documento | ✅ Ativo |
| `Motor-Rust/` | Motor de jogo Rust (11 módulos + 4 antifraude, 1816 testes) | ✅ Ativo |
| `Frontend-Web/` | Front-end TypeScript + React + Vite + Tailwind (Full Tilt skin) | ✅ Ativo (deploy canônico) |
| `Frontend-Dioxus/` | Legado WASM/Dioxus 0.6 | 📦 Legado (não canônico) |
| `API-Axum/` | API HTTPS/WSS (Axum + Tokio) | ✅ Ativo |

---

## 9. 🥇 Regra de Ouro — Consultar Antes de Codar

> **Antes de criar nova pasta, codar nova feature ou tomar decisão arquitetural, sempre consultar este documento, `QUALITY.md` (documento mestre) e a pasta `Documentacao/` (BUSINESS_RULES.md, DASHBOARD.md).**

---

**Próxima revisão:** Após integração Game Loop ↔ API Axum (WebSocket).
