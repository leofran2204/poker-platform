# 🏗️ Arquitetura do Motor Central da Plataforma de Poker Online

**Versão:** 3.2  
**Data:** 2026-07-14  
**Status:** Documento oficial — fonte da verdade para decisões de arquitetura

> Este documento é a **fonte da verdade** sobre a arquitetura da plataforma. Qualquer decisão de design, escolha de tecnologia ou nova pasta deve ser consultada aqui **antes** de iniciar a codificação.

---

## 1. 📐 Metodologia — SDD e MCP

- **SDD (Spec-Driven Development):** todas as funcionalidades começam com especificações formais (contratos de API, schemas JSON, regras de negócio).
- Specs são a "fonte da verdade" e guiam o desenvolvimento em Rust.
- **MCP (Middleware Control Plane):** garante que os serviços sigam as specs, centraliza logs e políticas de segurança.
- **WebMCP:** interface web para admins configurarem specs, monitorarem serviços e acessarem relatórios.

---

## 2. 🦀 Arquitetura de Linguagens — Stack 100% Rust (STACK ALVO v3.1)

> **Atualizado em 2026-07-03:** Stack consolidada em Rust para TUDO — backend, APIs, IA, dados, antifraude, autenticação, lobby e **front-end (Dioxus/WebAssembly)**. ❌ TypeScript/React removido. ❌ Python removido. ❌ Go removido.

| Camada                                                                                                                  | Linguagem  | Responsabilidade                                                                                                                              |
|-------------------------------------------------------------------------------------------------------------------------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------|
| **Tudo (backend + APIs + IA + dados + antifraude + autenticação + lobby + front-end)**                                  | **Rust**   | Motor crítico de jogo (cálculo de mãos, RNG, criptografia), APIs REST/WebSocket, IA de jogo, antifraude, estatísticas, relatórios, autenticação, lobby, controle de usuários, **UI para jogadores e administradores (Dioxus/WebAssembly)** |
| **Comunicação**                                                                                                         | **JSON**   | Formato universal entre módulos (Rust ↔ Rust)                                                                                                |

### 2.1 🦀 Por que Rust em TUDO?

- **Performance:** cálculo de mãos em tempo real sem latência perceptível.
- **Segurança de memória:** elimina classes inteiras de bugs (buffer overflow, use-after-free).
- **Concorrência:** modelo async/await nativo, ideal para milhares de conexões WebSocket simultâneas.
- **Criptografia:** crates auditados (`ring`, `rustls`, `aes-gcm`) para TLS 1.3 e AES-256.
- **RNG criptograficamente seguro:** essencial para integridade do jogo.
- **Ecossistema de IA:** `candle`, `burn`, `tch-rs` para ML e inferência em Rust.
- **Unificação:** uma única linguagem elimina fricção entre camadas, reduz complexidade de deploy e simplifica a equipe.

### 2.2 🖥️ Por que Rust (Dioxus) no Frontend WebAssembly?

- **Mesma linguagem do backend:** componentes, estado e lógica compartilhados sem boundary JS↔Rust.
- **WebAssembly nativo:** performance de código compilado no navegador, sem interpretação.
- **Dioxus:** framework React-like com suporte a web, desktop e mobile a partir do mesmo código.
- **Tipagem forte:** Rust elimina undefined is not a function e null pointer exceptions.
- **Comunicação direta:** WebSocket via `wasm-bindgen` + `gloo-net` para eventos em tempo real.
- **SSR nativo:** Dioxus suporta Server-Side Rendering sem ferramentas externas.

### 2.3 🔗 Comunicação entre Camadas — JSON + WebSocket

```
┌─────────────────────────────────────────────────────────────────────┐
│              🔐 CAMADA DE SEGURANÇA (transversal a tudo)            │
│   TLS 1.3 · AES-256 · JWT · MFA · bcrypt · PCI DSS · Antifraude    │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────┐      JSON/HTTP (TLS)      ┌──────────────┐
│   Front-end  │ ◄───────────────────────► │   Backend    │
│ Rust (Dioxus)│   WebSocket Seguro (WSS)  │     Rust     │
│  WebAssembly │   JWT + MFA               │  (Axum/Actix)│
└──────┬───────┘                           └──────┬───────┘
       │                                         │
       │                                         │
       │                                         ▼
       │                                  ┌──────────────┐
       │                                  │  PostgreSQL  │
       │                                  │ AES-256 em   │
       │                                  │ repouso      │
       │                                  └──────┬───────┘
       │                                         │
       │                                         ▼
       │                                  ┌──────────────┐
       │                                  │ ELK + Grafana│
       │                                  │ Logs + Audit │
       │                                  └──────────────┘
```

- **Rust (Dioxus) ↔ Rust (Axum/Actix):** HTTP REST + WebSocket (eventos de jogo em tempo real).
- **Formato universal:** JSON em todas as fronteiras (schemas validados).
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
Jogador                    Front-end (Rust/Dioxus)         Backend (Rust)              Banco (PG)
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

1. Jogador faz jogada no front-end (Rust/Dioxus WebAssembly) → envia via WebSocket.
2. Backend Rust recebe → valida → motor calcula → resultado em JSON.
3. Backend Rust registra no PostgreSQL → publica evento em Kafka.
4. Backend Rust (IA/antifraude) consome eventos → gera estatísticas, detecta fraude → resultados em JSON.
5. Front-end Rust/Dioxus consome APIs → renderiza mesas, dashboards.
6. MCP valida specs e segurança → WebMCP mostra status e relatórios.

---

## 7. 📋 Governança — MCP, WebMCP e SDD

- **MCP:** garante conformidade com specs, segurança e auditoria.
- **WebMCP:** interface para admins configurarem, monitorarem e ajustarem o sistema.
- **SDD:** metodologia que mantém consistência e qualidade em todas as camadas.
- **Rust unificado:** uma única linguagem para backend, IA, dados e antifraude simplifica governança, reduz pontos de falha e elimina fricção entre equipes.

---

## 8. ✅ Estado Atual da Implementação — Módulos e Testes

> ⚡ **Stack 100% Rust** — sem Node.js, Python, Go ou TypeScript. Pastas legadas (01, 02, 03, 06) foram deletadas em 2026-07-03.

| Pasta | Conteúdo | Status |
|-------|----------|--------|
| `Infraestrutura-Docker/` | Docker + Deploy (PostgreSQL 15, Redis 7, Kafka) | ✅ Ativo |
| `Documentacao/` | Documentação do projeto | ✅ Ativo |
| `Arquitetura-Motor/` | Este documento | ✅ Ativo |
| `Motor-Rust/` | Motor de jogo Rust (11 módulos + 4 antifraude, 1816 testes) | ✅ Ativo |
| `Frontend-Dioxus/` | Front-end WebAssembly com Dioxus 0.6 | ✅ Ativo (104 testes) |
| `API-Axum/` | API HTTP/WebSocket (Axum + Tokio) | ✅ Ativo |

---

## 9. 🥇 Regra de Ouro — Consultar Antes de Codar

> **Antes de criar nova pasta, codar nova feature ou tomar decisão arquitetural, sempre consultar este documento, `QUALITY.md` (documento mestre) e a pasta `Documentacao/` (BUSINESS_RULES.md, DASHBOARD.md).**

---

**Próxima revisão:** Após integração Game Loop ↔ API Axum (WebSocket).
