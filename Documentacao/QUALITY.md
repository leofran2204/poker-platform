# 🃏 QUALITY.md — Documento Mestre do Poker Project

**Atualizado:** 2026-08-07 | **Versão:** 5.4 (baseline de qualidade, segurança e operação)
**Stack Definitiva (v4.0):** Rust no backend/motor/API/antifraude; TypeScript + React + Vite + Tailwind no frontend (`Frontend-Web`). O antigo `Frontend-Dioxus/` foi removido do monorepo. Regulação planejada jan/2027.
**Decisão de Stack:** Motor/API 100% Rust; UI canônica TypeScript desde 2026-08-04.

> **Estado atual verificável:** este documento contém metas e registros históricos; não deve ser lido como certificado de produção ou como contagem fixa de testes. O gate atual executa lint estrito, testes determinísticos e contratos PostgreSQL. DePix foi incorporada somente como Sandbox não produtiva, com allowlist, idempotência e webhook autenticado; PIX produtivo e saque automático continuam fora do escopo. A operação multi-réplica aguarda ownership distribuído de mesas. Fonte canônica de ciclo: `STATUS_OPERACIONAL.json` (**S13** — presença online + demo).


---

> ## ⚠️ REGRA DE OURO — LEIA ANTES DE QUALQUER AÇÃO
>
> **Este documento é a fonte normativa de qualidade do Poker Project.**
> O estado operacional transversal é mantido em `Documentacao/STATUS_OPERACIONAL.json` e sincronizado automaticamente nos documentos marcados; os registros históricos deste arquivo continuam sob revisão humana.
> Antes de escrever uma linha de código, criar uma tarefa, tomar uma decisão técnica,
> de negócio, de segurança ou de marketing — **CONSULTE ESTE DOCUMENTO**.
>
> Documentos complementares obrigatórios:
> - `Arquitetura-Motor/ARQUITETURA_MOTOR.md` — arquitetura técnica do motor
> - `Documentacao/BUSINESS_RULES.md` — 45 regras de negócio do poker
> - `Documentacao/CRONOGRAMA.md` — fases e timeline
> - `Documentacao/DASHBOARD.md` — painel tático de tarefas
> - `Documentacao/DEVELOPMENT_LOG.md` — log de desenvolvimento
> - `Documentacao/guia_aprendizado.md` — guia de aprendizado consolidado
>
> **Princípio:** "A qualidade não é um ato, é um hábito." — Aristóteles
> **Missão:** Construir a plataforma de poker online **mais segura, confiável, ágil
> e bem-sucedida do mundo**.

---

# 📑 ÍNDICE GERAL

| #   | Seção                                            | Descrição                                                                          |
|-----|--------------------------------------------------|------------------------------------------------------------------------------------|
| 1   | Protocolo de Aprendizagem                        | Didática Mark↔Leofran, SDD, regras de símbolos                                     |
| 2   | Estado Atual do Projeto                          | 1.814 testes determinísticos do motor; perfil manual com 79 cenários extras, 20 mil entradas API HTTPS, 1.000.800 mensagens WSS e 2 milhões de entradas frontend |
| 3   | Pirâmide de Testes — Estratégia Completa         | Unit, integration, property, E2E, load, stress, fuzz, mutation, chaos              |
| 4   | Hacker Ético — Segurança Específica para Poker    | OWASP WSTG, pentests, ataques específicos de poker                                 |
| 5   | Arquitetura de Software                          | Martin Fowler, microservices, padrões distribuídos                                 |
| 6   | DevSecOps & Pipeline de Segurança                | OWASP DSOMM, shift-left, SAST/DAST/SCA, SBOM                                        |
| 7   | Chaos Engineering                                | Princípios, GameDays, métricas, blast radius                                        |
| 8   | Observabilidade com Rust                          | tracing crate, spans, eventos, #[instrument]                                       |
| 9   | Práticas de IA no Desenvolvimento                | Agentes, token efficiency, validação agêntica                                      |
| 10  | Plano de Negócio — Estilo Elon Musk              | Visão, mercado, monetização, milestones, equipe                                    |
| 11  | Gestão Financeira                                | Burn rate, runway, CAC/LTV, unit economics, cash flow                               |
| 12  | Marketing & Growth                               | AARRR, SEO, conteúdo, aquisição, retenção, comunidade                              |
| 13  | Metas & OKRs                                     | Objectives & Key Results, SMART, KPIs, Balanced Scorecard                          |
| 14  | Regulamentação & Compliance                      | UKGC, LGPD, PCI DSS, RGPD, jurisdições                                             |
| 15  | Próximas Ações & Roadmap                         | Plano de execução por fase                                                         |

---

# 🎓 1. PROTOCOLO DE APRENDIZAGEM — Formação Full Cycle do Arquiteto de Poker

## 👤 1.1 IDENTIDADE E CONTEXTO — Quem é Mark e o Papel do Arquiteto de Poker

*   **Instrutor:** Mark — especialista em Lógica, Spec-Driven Development (SDD),
    Harness Engineering, **Rust (Linguagem Principal — Backend, Motor, Frontend Dioxus)**,
    CRMs, APIs REST, MCP, MCP Server, WebMCP, UCP do Google,
    Segurança Pentest e Infraestrutura.
*   **Personalidade:** Um amigo com postura firme, direto e persistente para manter
    o foco. Comunicação estritamente livre de jargões ou analogias militares.
*   **Aluno:** Leofran — iniciante absoluto. A linguagem deve ser simples e clara
    (nível ensino médio).

> "Se não consegues explicar de forma simples, é porque ainda não entendeste bem
> o suficiente." — Feynman

## 🎯 1.2 OBJETIVO — Dominar a Engenharia da Plataforma de Poker

Formar Leofran como **Desenvolvedor Full Cycle e Engenheiro de IA**, tendo o
**Rust como o coração das suas habilidades técnicas**. Objetivo duplo:
1.  Ensiná-lo a comandar a IA (via prompt e agentes) para gerar código rápido
    através da metodologia SDD.
2.  **Obrigatoriamente** ensiná-lo a programar e auditar manualmente — dominar a
    sintaxe rigorosa do Rust e o significado de cada símbolo para revisar, otimizar
    e corrigir código gerado pela máquina.

## 🎰 1.3 PROJETO CENTRAL — Plataforma de Poker Online com Motor em Rust

Construção de uma **ÚNICA** plataforma de poker online (similar ao
PokerStars/GGPoker) altamente integrada:
*   **Rust (Foco Principal):** Backend, motor do jogo, validação de regras,
    gerenciamento de estado, processamento em tempo real, segurança de memória.
*   **Dioxus/WebAssembly:** Frontend, interface visual das mesas para o jogador.
*   **IA e Dados:** Implementados em Rust (antifraude, estatísticas, relatórios) —
    Python/TypeScript/Go foram removidos da stack em 2026-07-03.

## 📋 1.4 ORDEM DOS TÓPICOS — Roteiro de Estudo do Motor de Poker

1.  **Interpretação e Especificação (Fase 1 do SDD)** — ler problemas e escrever
    regras de negócio sem ambiguidades.
2.  **Lógica Proposicional e Matemática Aplicada** — condições, probabilidades,
    potes do jogo.
3.  **Planejamento e Tarefas (Fases 2 e 3 do SDD)** — quebrar o problema em pedaços
    lógicos e pseudocódigo.
4.  **Harness Engineering** — criar "pistas de teste" para validar o código.
5.  **Rust & O Motor do Jogo (Imersão Principal)** — sintaxe, ownership, borrowing,
    alta concorrência e APIs ultrarrápidas.
6.  **Dioxus & Interface** — gerar e revisar componentes visuais e painéis.
7.  **Programação Agêntica & IA** — orquestração com agentes, WebMCP, UCP.
8.  **Banco de Dados e Persistência** — salvar históricos e saldos integrados ao Rust.
9.  **Git/GitHub e CI/CD** — versionamento e automação de entregas.
10. **Segurança Pentest & Pagamentos** — depósitos, saques, blindagem de transações.
11. **Deploy e Infraestrutura** — Docker e Kubernetes.

## 🔤 1.5 REGRA DE TERMOS TÉCNICOS — Vocabulário do Poker e Engenharia

A cada termo técnico novo, abrir parêntese com definição simples + exemplo prático.
*   *Exemplo:* "O Rust usa o conceito de Ownership (Ownership = o sistema que
    garante que apenas uma parte do programa seja 'dona' de um dado por vez,
    evitando que o programa trave ou vaze memória)."

## ⌨️ 1.6 REGRA DO SIGNIFICADO DOS SÍMBOLOS (A Pontuação da Máquina)

É expressamente obrigatório explicar o que significa cada caractere especial no
código, com atenção redobrada aos símbolos do Rust (`&`, `*`, `mut`, `!`, `<T>`,
`||`, `::`, etc.). O aluno precisa saber o que está "falando" ao digitar aquele
símbolo.
*   *Exemplo:* "No Rust, quando usamos o e-comercial `&` antes de uma palavra,
    estamos dizendo: 'Estou apenas emprestando essa informação para você ler, não
    estou te dando ela em definitivo'. E o ponto de exclamação `!` em `println!`
    avisa que isso não é uma função comum, mas sim uma 'macro' (um atalho de código)."

## 📐 1.7 REGRA DE METODOLOGIA SDD (Spec-Driven Development)

1.  **Spec (Especificação):** O que o sistema tem que fazer e quais as regras.
2.  **Plan (Plano):** Como as partes vão se conectar.
3.  **Tasks (Tarefas):** O passo a passo exato para a IA programadora.

## 🔍 1.8 REGRA DE DISSECAÇÃO DE CÓDIGO — Anatomia do Motor de Poker

A lógica de cada comando manual ou gerado pela IA deve ser dissecada linha por
linha, caractere por caractere.
*   *Exemplo (Comando: `Get-ChildItem -Path C:\`):*
    **📖 Explicação detalhada:**
    *   `Get-ChildItem`: Cmdlet que lista arquivos.
    *   `-` (Traço): O traço avisa ao sistema que a próxima palavra é uma
        configuração (parâmetro).
    *   `Path`: A configuração que indica o "caminho" de onde vamos começar.
    *   `C:\`: A raiz do disco. Os dois pontos `:` separam o nome do disco da
        barra `\`, que indica "entrar na pasta".

## 🔄 1.9 REGRA DE REVISÃO ESPAÇADA — Consolidação do Conhecimento de Poker

Toda aula começa com a revisão dos 3 conceitos anteriores, 2 mini exercícios
integrados e explicação do aluno.

## 📏 1.10 REGRA DE GESTÃO DE ESPAÇO E CONTINUIDADE — Cadência do Estudo

**Estratégia revisada em 2026-07-02 (a pedido do aluno):**
*   **1 exemplo avançado por tópico** (em vez de 6 exercícios resolvidos).
*   O código do exemplo vem **logo abaixo** da explicação.
*   **Dissecação detalhada** de cada parte do código, caractere por caractere.
*   Foco em **mesclar construção do projeto + aprendizado** (aprender fazendo).
*   Cada exemplo é um **trecho real do código em produção** (não pseudocódigo).
*   Após o exemplo, o aluno pode pedir variações ou o próximo tópico.

## ⚡ 1.11 REGRA DE MODO CONSTRUÇÃO — Construir o Motor de Poker em Código

**Quando o aluno diz "vamos dar sequência na construção da plataforma", ativar o
MODO CONSTRUÇÃO:**

| Elemento           | Regra                                                                              |
|--------------------|------------------------------------------------------------------------------------|
| **Código**         | Bloco de código real do projeto (não pseudocódigo).                                |
| **Explicação**     | O que o BLOCO inteiro faz (intenção, não linha por linha).                         |
| **Analogia**       | 1 analogia do mundo real para cada bloco de código.                                |
| **Sem dissecação** | NÃO dissecar símbolos durante a construção — o aluno estuda isso por fora.        |
| **Ritmo**          | Código → Explicação do bloco → Analogia → Próximo bloco.                          |
| **Objetivo**       | Avançar a plataforma rapidamente, mantendo o entendimento conceitual.             |

**Contraste com o MODO ESTUDO:**
- Modo Estudo = Teoria + Exemplo + Dissecação + Exercício (tópicos NOVOS e COMPLEXOS)
- Modo Construção = Código + Explicação do bloco + Analogia (AVANÇAR o projeto)

## 🧭 1.12 SEQUÊNCIA OBRIGATÓRIA POR TÓPICO (4 etapas)

### Etapa 1 — Teoria
O que é, analogia, como funciona no modelo SDD e explicação clara dos símbolos
usados na sintaxe (com destaque para o Rust).

### Etapa 2 — 1 Exemplo Avançado
*   **Enunciado:** Problema real do projeto (trecho de código em produção).
*   **Spec & Plan (SDD):** A especificação lógica e a arquitetura visual/fluxo.
*   **O Código Real:** O trecho de código **real** do projeto (não pseudocódigo).
*   **Dissecação Detalhada:** Cada linha, cada caractere especial, cada símbolo
    explicado.
*   **Erro de IA Comum:** Uma alucinação típica da IA e como revisar e corrigir.
*   **🧩 Exercício de Fixação:** Logo abaixo da dissecação, um exercício completo:
    1. Enunciado do exercício (variação do exemplo principal)
    2. Dica de abordagem (por onde começar)
    3. Onde aplicar no projeto (arquivo real para modificar)
    4. Resultado esperado (o que deve acontecer quando estiver correto)
    5. Checkpoint de validação (como rodar o teste e confirmar que funcionou)

### Etapa 3 — Integração no Projeto
Após entender o exemplo, o aluno aplica o conceito em uma tarefa real do backlog
(`DASHBOARD.md`).

### Etapa 4 — Revisão Espaçada
Na próxima aula, revisão rápida dos conceitos do tópico anterior antes de avançar.

## 📡 1.13 REGRAS DE COMUNICAÇÃO — Tom de Voz do Arquiteto de Poker

✅ **SEMPRE:**
*   Firmeza, clareza e persistência no foco.
*   Dissecar o código e os símbolos, especialmente as peculiaridades do Rust.
*   Ensinar a auditar a IA manualmente.
*   Usar marcadores para parágrafos maiores.

❌ **NUNCA:**
*   Usar jargões ou expressões militares em nenhuma hipótese.
*   Usar jargão técnico sem definição imediata.
*   Pular a fase de Especificação (Spec).
*   Permitir que o aluno mude de assunto.

---

# 📊 2. ESTADO ATUAL DO PROJETO — Snapshot da Plataforma de Poker Online

## 🗺️ 2.1 VISÃO GERAL — Panorama da Plataforma de Poker Online

| Métrica               | Valor                                                                  |
|-----------------------|------------------------------------------------------------------------|
| **Linguagem**         | Rust (motor + API) + TypeScript (SPA)                                 |
| **Frontend**          | React 18 + Vite + Tailwind (`Frontend-Web`) — Full Tilt skin          |
| **Testes totais**     | 1.813 testes determinísticos no Motor-Rust; perfil autorizado com 79 cenários de carga adicionais |
| **Estresse API/WS**   | **1.000.800 mensagens WebSockets** em 100 mesas + 50 Red Team workers  |
| **Módulos do motor**  | 10 módulos principais                                                  |
| **Módulos antifraude**| 4 módulos (bot detection, collusion, chip dumping, multi-account)       |
| **Pagamentos**        | Mock local e Asaas Sandbox autenticado com allow-list; Mercado Pago e PIX de produção desabilitados |
| **Infraestrutura**    | docker-compose.yml (PostgreSQL 15, Redis 7, Kafka+Zookeeper, Caddy HTTPS) |

## 🧩 2.2 MÓDULOS DO MOTOR (10 módulos engine + módulos de teste)

| Módulo               | Arquivo                  | Testes | Função                                              |
|----------------------|--------------------------|--------|-----------------------------------------------------|
| **Deck**             | `deck.rs`                | 18     | Baralho de 52 cartas, embaralhamento, distribuição  |
| **Side Pots**        | `side_pots.rs`           | 7      | Potes laterais (all-in parcial)                     |
| **Loss Deflator**    | `loss_deflator.rs`       | 9      | Mecânica de cashback por perdas                      |
| **Rake**             | `rake.rs`                | 13     | Taxa da casa (cap R$6, tiers)                       |
| **RNG Crypto**       | `rng_crypto.rs`          | 20     | Gerador criptográfico de aleatoriedade              |
| **Hand History**     | `hand_history.rs`        | 19     | Histórico de mãos jogadas                           |
| **Tournament Engine**| `tournament_engine.rs`   | 19     | Motor de torneios (MTT/SNG)                         |
| **Auth**             | `auth.rs`                | 153    | Autenticação, JWT, MFA, RBAC                        |
| **Lobby**            | `lobby.rs`               | 28     | GameType, TableVisibility, TableInfo                |
| **Fuzzing Extremo**  | `extreme_fuzz_tests.rs`  | 8 (1M) | 1.000.000 de iterações de Fuzzing estocástico em 8 módulos |
| **Pagamentos PIX**   | `payments_routes.rs`     | 5      | Adaptadores e contratos locais; não habilitado para operação real         |
| **Integração**       | `integration_tests.rs`  | 5      | Fluxo completo entre módulos (deck→side_pots→rake→hand_history, torneio, loss_deflator+rake, RNG+deck) |
| **Stress Integração**| `stress_integration_tests.rs` | 5 | Stress massivo de integração (200k iters/cenário, seed fixo `StdRng`, invariantes exatos) |
| **Fairness Cartas**  | `card_fairness_tests.rs`| 3      | Fairness estatística de cartas (qui-quadrado, 500k iters/teste, tolerância 0,5%) |
| **Stress Módulos**   | `stress_tests.rs`       | 15     | Stress de cada módulo (deck, side_pots, rake, utils, hand_history, tournament) |

## 🛡️ 2.3 MÓDULOS ANTIFRAUDE (4 módulos)

| Módulo            | Arquivo                       | Estrutura                                | Função                              |
|-------------------|-------------------------------|------------------------------------------|-------------------------------------|
| **Collusion**     | `antifraud/collusion.rs`      | `PlayerAction`, `HandStrength`           | Detecta colusão entre jogadores     |
| **Chip Dumping**  | `antifraud/chip_dumping.rs`   | `ChipDumpRecord`                         | Detecta transferência ilegal de fichas |
| **Bot Detection** | `antifraud/bot_detection.rs`  | `BotMetrics`                             | Detecta comportamento robótico      |
| **Multi-Account** | `antifraud/multi_account.rs`  | `PlayerFingerprint`, `MultiAccountAlert` | Detecta contas múltiplas            |

## 🔐 2.4 STACK DE SEGURANÇA ATUAL — Camadas de Defesa da Mesa de Poker

| Componente                | Tecnologia                    | Status                          |
|---------------------------|-------------------------------|---------------------------------|
| **TLS**                   | rustls (TLS 1.3)              | ✅ Planejado                    |
| **Criptografia simétrica**| aes-gcm (AES-256)             | ✅ Planejado                    |
| **JWT**                   | hmac + sha2 (manual)          | ✅ Implementado em auth.rs      |
| **Hash de senhas**        | bcrypt 0.16                   | ✅ Implementado em auth.rs      |
| **MFA/TOTP**              | RFC 6238                      | ✅ Implementado em auth.rs      |
| **RBAC**                  | Role-Based Access Control     | ✅ Implementado em auth.rs      |
| **RNG criptográfico**     | rand 0.8 + getrandom          | ✅ Implementado em rng_crypto.rs|

## 🐳 2.5 INFRAESTRUTURA — Containers Docker do Motor de Poker

| Componente        | Tecnologia                        | Função                                            |
|-------------------|-----------------------------------|---------------------------------------------------|
| **Banco de dados**| PostgreSQL 15                     | Persistência de usuários, mãos, transações        |
| **Cache**         | Redis 7                           | Sessões, estado de jogo em tempo real             |
| **Message queue** | Kafka + Zookeeper                 | Eventos de jogo, auditoria, analytics             |
| **Container**     | Docker (docker-compose)           | Orquestração local                                |
| **Deploy**        | render.yaml (legado — atualizar)  | Deploy cloud (precisa revisão)                    |

## ⚙️ 2.5.1 REGRA DO TOOLCHAIN — Frontend-Dioxus (REMOVIDO)

> **🗑️ Removido do monorepo (2026-08-28).** O frontend canônico é `Frontend-Web/` (`npm run lint` / `npm run build`).
> Histórico WASM/Dioxus e o antigo `scripts/cargo-dioxus.ps1` ficam só no git.
>
> Conteúdo abaixo é **arquivo histórico** (não executar).

### Script Wrapper (legado — arquivo removido)

```powershell
# REMOVIDO — não use
.\scripts\cargo-dioxus.ps1 check
```

### Comando Manual (se o script não estiver disponível)

```powershell
$gccLibPath = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\lib\gcc\x86_64-w64-mingw32\16.1.0"
$env:LIBRARY_PATH = $gccLibPath
$env:C_INCLUDE_PATH = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\include"
cd "c:\Users\leofr\Projetos\Poker_Project\Frontend-Dioxus"
cargo +stable-x86_64-pc-windows-gnu check
```

### Notas
- Se a versão do GCC mudar (ex: `16.1.0` → `16.2.0`), verificar o diretório real:
  ```powershell
  Get-ChildItem "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\lib\gcc\x86_64-w64-mingw32\"
  ```
- **NUNCA** rode `cargo check`/`cargo test`/`cargo clippy` diretamente no
  `Frontend-Dioxus/` sem configurar as variáveis — vai falhar.
- `Motor-Rust/` **não** precisa dessa configuração (não usa proc-macros DLLs).

## 📦 2.6 DEPENDÊNCIAS RUST (Cargo.toml) — Crates do Motor de Poker

### Motor (Motor-Rust)
```toml
[dependencies]
serde = "1.0"           # Serialização JSON
rand = "0.8"            # Aleatoriedade
bcrypt = "0.16"         # Hash de senhas
hmac = "0.12"           # JWT (HMAC)
sha2 = "0.10"           # JWT (SHA-256)
base64 = "0.22"         # Codificação base64

[dev-dependencies]
proptest = "1.0"        # Testes baseados em propriedades
```

### Frontend (Frontend-Dioxus)
```toml
[dependencies]
dioxus = "0.6"          # Framework frontend (estilo React, em Rust/WebAssembly)
wasm-bindgen = "0.2"    # Bridge Rust↔JavaScript
gloo-net = "0.6"        # HTTPS/WSS no WASM
```

---

# 🔺 3. PIRÂMIDE DE TESTES — Estratégia de Cobertura do Motor de Poker

> **Princípio:** A pirâmide de testes garante cobertura ampla na base (testes
> unitários rápidos e baratos) e vai estreitando até o topo (testes E2E lentos e
> caros). O objetivo é **maximizar confiança com mínimo tempo de execução**.

```
                    🔺 E2E (Playwright)
                   /   \
                  /     \  ← Poucos, lentos, alto valor
                 /───────\
                / Fuzz    \  ← Aleatoriedade extrema
               /  Mutation  \
              /──────────────\
             /  Contract Tests \  ← APIs e schemas
            /    Load & Stress    \  ← Performance e limite
           /────────────────────────\
          /  Integration Tests (Rust)  \  ← Entre módulos
         /────────────────────────────────\
        /    Property Tests (proptest)      \  ← Invariantes
       /────────────────────────────────────────\
      /          Unit Tests (484 testes)           \  ← Base sólida
     /──────────────────────────────────────────────────\
```

## 🧪 3.1 NÍVEL 1 — TESTES UNITÁRIOS (Base da Pirâmide do Motor de Poker)

**Status:** ✅ 484 testes implementados | **Meta:** 600+ testes até F2

### O que testar (específico para poker)

| Módulo                  | Cenários de Teste                              | Exemplos                                                                                              |
|-------------------------|------------------------------------------------|-------------------------------------------------------------------------------------------------------|
| **deck.rs**             | Embaralhamento, distribuição, integridade      | Baralho tem 52 cartas únicas; embaralhamento muda ordem; distribuição não repete carta               |
| **side_pots.rs**        | Potes laterais com all-in parcial              | 3 jogadores all-in com valores diferentes → potes calculados corretamente                            |
| **loss_deflator.rs**    | Cashback pós-rake por equity (7/15/25/35%)       | Base líquida 100: equity 70% → 15; equity 62% → 7; equity 55,9% → 0; equity 86% → 35 |
| **rake.rs**             | Taxa da casa com cap R$6                       | Pote R$50 → rake R$2.50; pote R$500 → rake R$6 (cap); pote < mínimo → rake zero                      |
| **rng_crypto.rs**      | Aleatoriedade criptográfica                    | Distribuição uniforme; não repete sequência; passa em NIST SP 800-22                                 |
| **hand_history.rs**     | Registro e replay de mãos                      | Replay reproduz mão idêntica; ações ordenadas corretamente                                           |
| **tournament_engine.rs**| Blinds crescentes, eliminação, premiação       | Blinds sobem no tempo certo; jogador eliminado quando fichas = 0; premiação por posição             |
| **auth.rs**             | JWT, MFA, RBAC, bcrypt                         | Token expira em 15min; MFA rejeita código expirado; RBAC bloqueia acesso não autorizado             |

### Padrões de teste a seguir

```rust
// PADRÃO: Arrange-Act-Assert (AAA)
#[test]
fn rake_respeita_cap_de_6_reais() {
    // Arrange
    let pot = 500.0_f64;
    let rake_calculator = RakeCalculator::new(0.05, 6.0);

    // Act
    let rake = rake_calculator.calculate(pot);

    // Assert
    assert_eq!(rake, 6.0, "Rake deve ser limitado ao cap de R$6");
}

// PADRÃO: Partições de equivalência
#[test]
fn rake_zero_para_potes_abaixo_do_minimo() {
    let pot = 5.0_f64; // Abaixo do mínimo de R$10
    let rake = RakeCalculator::new(0.05, 6.0).calculate(pot);
    assert_eq!(rake, 0.0, "Potes abaixo do mínimo não pagam rake");
}

// PADRÃO: Valores-limite
#[test]
fn rake_no_limite_exato_do_cap() {
    let pot = 120.0_f64; // 5% de 120 = 6.0 = cap exato
    let rake = RakeCalculator::new(0.05, 6.0).calculate(pot);
    assert_eq!(rake, 6.0);
}
```

### Cobertura de código

| Ferramenta            | Comando                                            | Quando     |
|-----------------------|----------------------------------------------------|------------|
| `cargo-tarpaulin`     | `cargo tarpaulin --workspace --out Html`           | Imediato (F2) |
| `cargo-llvm-cov`      | `cargo llvm-cov --html`                             | Alternativa|
| `grcov` (Docker)      | Ver `scripts/coverage.ps1`                          | Já configurado |
| **Meta de cobertura** | **Cobertura diferenciada por criticidade** (ver tabela abaixo) | Contínuo   |

**Metas de cobertura por criticidade:**

| Módulo                                                                  | Risco                                   | Cobertura mínima | Justificativa                                                                  |
|-------------------------------------------------------------------------|-----------------------------------------|------------------|--------------------------------------------------------------------------------|
| 💰 Financeiro (rake, loss_deflator, saldos)                              | Crítico — irreversível                  | **≥ 98%**        | Bug = perda de dinheiro real, irrecuperável                                    |
| 🔐 Segurança (auth, JWT, MFA, antifraude, rng_crypto)                    | Crítico — vazamento/fraude              | **≥ 98%**        | Bug = credenciais vazadas ou fraude não detectada                              |
| 🃏 Motor de jogo (deck, side_pots, tournament_engine, hand_history)     | Crítico — irreversível + dano à credibilidade | **≥ 98%**    | Bug = cartas duplicadas, pote errado, vencedor errado — dano permanente à reputação |
| 🌐 API (handlers, middleware, tournament_store)                         | Médio — recuperável                     | **≥ 95%**        | Bug = erro 500, corrigível com redeploy                                        |
| 🖥️ Frontend (Dioxus)                                                    | Baixo — recuperável                     | **≥ 90%**        | Bug = tela quebrada, corrigível sem impacto ao jogo                            |

## 🔗 3.2 NÍVEL 2 — TESTES DE INTEGRAÇÃO (Mesas de Poker Conectadas)

**Status:** ✅ Implementado (10 testes: 5 determinísticos `integration_tests.rs` + 5 stress `stress_integration_tests.rs` de 200k iters) | **Quando:** F2

### Cenários de integração entre módulos

| # | Cenário                  | Módulos Envolvidos                       | Validação                                                                                      |
|---|--------------------------|-----------------------------------------|------------------------------------------------------------------------------------------------|
| 1 | **Mão completa**         | deck → side_pots → rake → hand_history  | Distribuir cartas, jogar rodadas, calcular potes laterais, cobrar rake, registrar histórico    |
| 2 | **Torneio completo**     | tournament_engine → deck → side_pots   | Blinds sobem, jogadores eliminados, vencedor recebe prêmio                                     |
| 3 | **Rake + Loss Deflator** | side_pots → rake → loss_deflator → payouts | Tier vem da equity no all-in e cashback usa somente potes líquidos elegíveis                   |
| 4 | **Auth + lobby**         | auth → lobby                            | Jogador autentica, entra no lobby, vê mesas disponíveis                                        |
| 5 | **Antifraude + hand_history** | collusion → hand_history           | Detectar colusão analisando histórico de mãos                                                  |
| 6 | **Multi-account + auth** | multi_account → auth                   | Mesma fingerprint tentando criar 2ª conta → bloquear                                          |
| 7 | **Bot detection + lobby**| bot_detection → lobby                   | Comportamento robótico detectado → flag e/ou ban                                               |
| 8 | **RNG + deck**           | rng_crypto → deck                       | Embaralhamento usa RNG criptográfico, não vaza estado                                          |

```rust
// Exemplo: teste de integração de mão completa
#[test]
fn test_mao_completa_distribuicao_ate_rake() {
    let mut deck = Deck::new();
    deck.shuffle_with_crypto_rng();

    let mut players = vec![
        Player::new("alice", 1000.0),
        Player::new("bob", 1000.0),
        Player::new("carol", 1000.0),
    ];

    // Distribuir cartas
    for player in &mut players {
        player.receive_cards(deck.deal(2));
    }

    // Simular apostas
    let pot = 300.0;
    let rake = RakeCalculator::new(0.05, 6.0).calculate(pot);

    // Validar
    assert_eq!(rake, 6.0, "Rake deve respeitar cap");
    assert!(deck.cards_remaining() < 52, "Cartas foram distribuídas");
}
```

## 🎲 3.3 NÍVEL 3 — TESTES BASEADOS EM PROPRIEDADES (proptest)

**Status:** ✅ proptest 1.0 no Cargo.toml | **Arquivo:** `property_tests.rs`

### O que são testes baseados em propriedades?

Em vez de testar casos específicos, o `proptest` gera **centenas ou milhares de
casos aleatórios** automaticamente e verifica se uma **propriedade (invariante)**
sempre se mantém verdadeira. Se um caso falhar, o proptest faz "shrinking" —
reduz o caso de falha ao menor exemplo possível que ainda quebra.

### Propriedades a testar (específicas para poker)

| #  | Propriedade                    | Invariante                                                       | Módulo               |
|----|--------------------------------|------------------------------------------------------------------|----------------------|
| 1  | **Integridade do baralho**     | Após qualquer embaralhamento, o baralho sempre tem 52 cartas únicas | deck.rs              |
| 2  | **Rake nunca negativo**        | Para qualquer pote ≥ 0, o rake é sempre ≥ 0 e ≤ cap              | rake.rs              |
| 3  | **Rake nunca excede cap**      | Para qualquer pote, rake ≤ R$6.00                                | rake.rs              |
| 4  | **Side pots somam corretamente**| A soma de todos os potes laterais = pote total                  | side_pots.rs         |
| 5  | **Loss deflator dentro de tiers**| Cashback sempre entre 0% e 35%, nunca negativo                 | loss_deflator.rs     |
| 6  | **RNG distribuição uniforme**  | Após N gerações, cada valor aparece ~N/total vezes               | rng_crypto.rs        |
| 7  | **JWT sempre decifrável**      | Qualquer token gerado pode ser verificado com a chave correta    | auth.rs              |
| 8  | **MFA código de 6 dígitos**    | Qualquer código TOTP tem exatamente 6 dígitos                    | auth.rs              |
| 9  | **Tournament blinds só sobem** | Blinds nunca diminuem ao longo do torneio                        | tournament_engine.rs |
| 10 | **Hand history replay fiel**   | Replay de qualquer mão reproduz ações idênticas                 | hand_history.rs      |

```rust
// Exemplo: propriedade do rake nunca excede cap
proptest! {
    #[test]
    fn rake_nunca_excede_cap(pot in 0.0f64..100000.0) {
        let rake = RakeCalculator::new(0.05, 6.0).calculate(pot);
        prop_assert!(rake <= 6.0, "Rake {} excede cap para pote {}", rake, pot);
        prop_assert!(rake >= 0.0, "Rake negativo para pote {}", pot);
    }

    #[test]
    fn baralho_sempre_tem_52_cartas_unicas(seed in any::<u64>()) {
        let mut deck = Deck::new();
        deck.shuffle_with_seed(seed);
        let cards = deck.deal_all();
        let unique: HashSet<_> = cards.iter().collect();
        prop_assert_eq!(cards.len(), 52);
        prop_assert_eq!(unique.len(), 52, "Cartas duplicadas com seed {}", seed);
    }
}
```

### Regressões salvas

O proptest salva casos de falha em `proptest-regressions/motor_tests.txt`.
Estes casos são **re-executados automaticamente** em futuras execuções de teste,
garantindo que regressões não voltem a ocorrer.

## 📜 3.4 NÍVEL 4 — TESTES DE CONTRATO (API do Motor de Poker)

**Status:** ⏳ Pendente | **Quando:** F2 (quando API Axum existir)

### O que validar

| # | Contrato                          | Validação                                                                                              |
|---|-----------------------------------|--------------------------------------------------------------------------------------------------------|
| 1 | **POST /api/auth/register**       | Request: `{email, password, username}` → Response: `{token, expires_in}`                              |
| 2 | **POST /api/auth/login**          | Request: `{email, password}` → Response: `{token, mfa_required?}`                                     |
| 3 | **POST /api/auth/mfa/verify**     | Request: `{code}` → Response: `{token}` ou `401 Unauthorized`                                          |
| 4 | **GET /api/lobby/tables**         | Response: `[{id, name, players, max_players, blinds, type}]`                                          |
| 5 | **POST /api/lobby/join**          | Request: `{table_id, buy_in}` → Response: `{seat, chips}`; buy-in é movido para escrow transacional    |
| 6 | **WS /ws/game/{table_id}**        | Eventos: `deal`, `bet`, `fold`, `showdown` com schema definido                                         |
| 7 | **POST /api/tournament/register** | Request: `{tournament_id}` → Response: `{position, chips}`                                            |
| 8 | **GET /api/hand-history/{hand_id}**| Response: replay completo da mão                                                                       |

### Ferramentas

| Ferramenta        | Uso                                                  |
|-------------------|------------------------------------------------------|
| **Postman**       | Coleção exportável de endpoints REST + WebSocket     |
| **schemars** (Rust)| Gerar JSON Schema automaticamente dos tipos Serde   |
| **pact-rust**     | Consumer-Driven Contract Testing                      |
| **dioxus-cli**    | Testar componentes frontend contra contratos         |

## 🌐 3.5 NÍVEL 5 — TESTES E2E (End-to-End da Mesa de Poker)

**Status:** ⏳ Pendente | **Quando:** F3→F4 (após frontend Dioxus funcional)

### Fluxos completos a testar

| # | Fluxo E2E                    | Passos                                                                                       | Ferramenta |
|---|------------------------------|----------------------------------------------------------------------------------------------|------------|
| 1 | **Registro + Login + MFA**   | Registrar → verificar email → ativar MFA → login → inserir código TOTP → entrar              | Playwright |
| 2 | **Entrar em mesa cash game** | Login → lobby → selecionar mesa → sentar → receber cartas → ver blinds                       | Playwright |
| 3 | **Jogar mão completa**       | Sentar → blinds → pre-flop → flop → turn → river → showdown → rake cobrado                   | Playwright |
| 4 | **All-in e side pot**        | 3 jogadores all-in com valores diferentes → potes laterais → premiação correta              | Playwright |
| 5 | **Torneio MTT**              | Registrar → começar → blinds sobem → jogadores eliminados → premiação final                  | Playwright |
| 6 | **Depósito e saque**         | Depositar via PIX → saldo atualizado → jogar → sacar → saldo diminui                         | Playwright |
| 7 | **Chat da mesa**             | Enviar mensagem → outros jogadores veem → filtro de palavras proibidas                      | Playwright |
| 8 | **Desconexão e reconexão**   | Jogador desconecta no meio de mão → reconecta → volta à mesma mesa e posição                 | Playwright |

### Page Object Pattern para Dioxus

```
Frontend-Dioxus/tests/
├── pages/
│   ├── login_page.rs       # LoginPage: preencher_email, preencher_senha, clicar_entrar
│   ├── lobby_page.rs       # LobbyPage: listar_mesas, selecionar_mesa, entrar_mesa
│   ├── table_page.rs       # TablePage: ver_cartas, apostar, fold, check, all_in
│   └── tournament_page.rs  # TournamentPage: registrar, ver_blinds, ver_posicao
├── flows/
│   ├── login_flow.rs       # Fluxo completo de login + MFA
│   ├── cash_game_flow.rs   # Fluxo completo de cash game
│   └── tournament_flow.rs  # Fluxo completo de torneio
└── e2e_test.rs             # Suite E2E que orquestra todos os fluxos
```

## ⚡ 3.6 NÍVEL 6 — TESTES DE CARGA E STRESS (Mesas de Poker Sob Pressão)

**Status:** ✅ Implementado (20 testes: 15 `stress_tests.rs` + 5 `stress_integration_tests.rs`, até 200k iters/cenário) | **Quando:** F2

### Cenários de carga específicos para poker

| # | Cenário                              | Métrica Alvo                                  | Ferramenta      |
|---|--------------------------------------|-----------------------------------------------|-----------------|
| 1 | **1.000 jogadores simultâneos**      | Latência WebSocket < 100ms                    | k6 / locust     |
| 2 | **10.000 jogadores simultâneos**      | Latência WebSocket < 500ms                    | k6 / locust     |
| 3 | **Pico de torneio (start)**          | 5.000 jogadores entrando em 60s               | k6              |
| 4 | **1.000 mãos simultâneas**            | CPU < 80%, memória < 4GB                       | k6 + Prometheus |
| 5 | **Depósitos em massa**               | 100 transações/s sem erro                      | k6              |
| 6 | **Saques em massa**                  | 50 transações/s sem erro                       | k6              |
| 7 | **Chat em massa**                    | 10.000 mensagens/min sem lag                  | k6              |
| 8 | **All-in simultâneo**                | 100 all-ins ao mesmo tempo → side pots corretos | k6              |

### Métricas de performance alvo

| Métrica                              | Alvo                    | Crítico |
|--------------------------------------|-------------------------|---------|
| **Latência WebSocket (p50)**          | < 50ms                  | < 200ms |
| **Latência WebSocket (p99)**          | < 200ms                 | < 500ms |
| **Tempo de distribuição de cartas**   | < 10ms                  | < 50ms  |
| **Tempo de cálculo de side pot**      | < 5ms                   | < 20ms  |
| **Tempo de validação de mão**         | < 1ms                   | < 5ms   |
| **Throughput de mãos/hora**           | > 100 mãos/hora/mesa    | > 60    |
| **Uso de CPU (1k jogadores)**         | < 60%                   | < 80%   |
| **Uso de memória (1k jogadores)**     | < 2GB                   | < 4GB   |

## 💥 3.7 NÍVEL 7 — TESTES FUZZ (Fuzzing do Motor de Poker)

**Status:** ⏳ Pendente | **Quando:** F4→F5

### O que é fuzzing?

Fuzzing envia **dados aleatórios, malformados e inesperados** para as entradas do
sistema para descobrir crashes, panics e vulnerabilidades de segurança. O Rust
tem suporte nativo via `cargo-fuzz` (libFuzzer do LLVM).

### Alvos de fuzzing para poker

| # | Alvo                              | Entrada Fuzz                              | O que procurar                  |
|---|-----------------------------------|-------------------------------------------|---------------------------------|
| 1 | **Parser de ações do WebSocket**  | Mensagens JSON malformadas                | Panic, crash, deserialização incorreta |
| 2 | **Cálculo de side pots**          | Valores extremos (negativos, NaN, infinito) | Panic, resultado incorreto      |
| 3 | **Cálculo de rake**               | Potes negativos, zero, muito grandes      | Panic, rake negativo, rake > cap |
| 4 | **Validação de JWT**              | Tokens malformados, vazios, muito longos  | Panic, bypass de auth           |
| 5 | **Parser de hand history**         | Históricos truncados, corrompidos         | Panic, loop infinito            |
| 6 | **Loss deflator**                 | Perdas negativas, NaN, muito grandes      | Panic, cashback incorreto       |
| 7 | **Tournament engine**             | Blinds negativos, zero jogadores          | Panic, estado inválido          |
| 8 | **Antifraude (collusion)**        | Dados de jogadores malformados             | Panic, falso positivo/negativo  |

```rust
// Exemplo: fuzz target para cálculo de rake
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(pot) = f64::from_le_bytes(data.try_into().unwrap_or([0u8; 8])) {
        let rake = RakeCalculator::new(0.05, 6.0).calculate(pot);
        // Não deve panicar com qualquer entrada
        assert!(rake.is_finite());
        assert!(rake >= 0.0);
        assert!(rake <= 6.0);
    }
});
```

## 🧬 3.8 NÍVEL 8 — TESTES DE MUTAÇÃO (Robustez do Motor de Poker)

**Status:** ⏳ Pendente | **Quando:** F4

### O que são testes de mutação?

Testes de mutação modificam automaticamente o código-fonte (ex: trocar `>` por
`>=`, `+` por `-`, `&&` por `||`) e verificam se os testes existentes **detectam**
a mudança. Se um mutante sobrevive, significa que os testes não cobrem aquele
caminho — é um buraco na cobertura de testes.

### Ferramenta: `cargo-mutants`

```bash
cargo install cargo-mutants
cargo mutants --workspace
```

### Mutantes esperados e que devem ser detectados

| # | Mutante                                  | Teste que deve falhar                       |
|---|------------------------------------------|---------------------------------------------|
| 1 | `rake <= 6.0` → `rake < 6.0`             | `rake_no_limite_exato_do_cap`               |
| 2 | `rake >= 0.0` → `rake > 0.0`             | `rake_zero_para_potes_abaixo_do_minimo`     |
| 3 | `cards.len() == 52` → `cards.len() == 51` | `baralho_sempre_tem_52_cartas_unicas`       |
| 4 | `token.expires_in > now` → `token.expires_in >= now` | `token_expira_no_tempo_correto`   |
| 5 | `cashback <= 0.35` → `cashback <= 0.36`   | `loss_deflator_respeita_tier_maximo`        |

## 🔁 3.9 NÍVEL 9 — TESTES DE REGRESSÃO (Não Quebrar o Poker)

**Status:** ⏳ Pendente | **Quando:** F4 (após CI/CD configurado)

### Estratégia

| Componente                   | Estratégia                                                                                       |
|------------------------------|--------------------------------------------------------------------------------------------------|
| **Regressões do proptest**   | Casos de falha salvos em `proptest-regressions/` são re-executados automaticamente              |
| **Snapshots de hand history** | Mãos de referência são comparadas com saídas atuais                                             |
| **Snapshots de API**         | Respostas de endpoints comparadas com schemas salvos                                            |
| **Baseline de performance**   | Latência e throughput comparados com baseline anterior                                          |
| **CI/CD em cada PR**         | Toda suíte de testes roda em cada Pull Request — zero regressões permitidas                     |

## 🕵️ 3.10 NÍVEL 10 — TESTES DE SEGURANÇA (detalhado na Seção 4)

**Status:** ⏳ Pendente | **Quando:** F5

Ver Seção 4 (Hacker Ético) para a estratégia completa de testes de segurança
específicos para plataforma de poker online.

## 💥 3.11 NÍVEL 11 — TESTES DE CHAOS ENGINEERING (detalhado na Seção 11)

**Status:** ⏳ Pendente | **Quando:** F5→F6

Ver Seção 11 (Chaos Engineering) para a estratégia completa de injeção de falhas
em produção.

## 🐛 3.12 TEMPLATE DE BUG REPORT — Reportar Falhas no Motor de Poker

**Status:** ⏳ Pendente | **Arquivo a criar:** `Documentacao/BUG_REPORT_TEMPLATE.md`

```markdown
## BUG REPORT

**Título:** [Descrição curta do problema]
**Severidade (GUT):** Gravidade [1-5] × Urgência [1-5] × Tendência [1-5] = [Total]

### Ambiente
- Versão do Rust: [ex: 1.88]
- Módulo: [ex: rake.rs]
- OS: [ex: Windows 11 / Docker]
- Browser (se frontend): [ex: Chrome 126]

### Passos para reproduzir
1. [Passo 1]
2. [Passo 2]
3. [Passo 3]

### Resultado esperado
[O que deveria acontecer]

### Resultado atual
[O que aconteceu de errado]

### Evidências
- [Log/stack trace]
- [Screenshot se aplicável]
- [Caso de teste que falhou]

### Impacto no negócio
[Ex: Jogadores podem receber rake negativo → perda de receita]
```

## 📊 3.13 MATRIZ GUT PARA BACKLOG — Priorização do Poker

Priorizar tarefas do `DASHBOARD.md` por **Gravidade × Urgência × Tendência**:

| G | Gravidade (impacto se não fizer) | 1-5 |
|---|----------------------------------|-----|
| U | Urgência (prazo para fazer)      | 1-5 |
| T | Tendência (piora com o tempo?)   | 1-5 |

**Score GUT = G × U × T** (quanto maior, maior prioridade)

| Tarefa                | G | U | T | Score | Prioridade  |
|-----------------------|---|---|---|-------|-------------|
| CI/CD GitHub Actions  | 5 | 5 | 4 | 100   | ✅ Crítica (Implementado) |
| Cobertura de testes   | 4 | 5 | 3 | 60    | 🔴 Alta     |
| cargo audit (segurança)| 5 | 4 | 4 | 80    | ✅ Alta (Implementado no CI) |
| Template de bug report| 3 | 3 | 2 | 18    | 🟡 Média    |
| Fluxograma BPMN       | 2 | 3 | 1 | 6     | 🟢 Baixa    |

---

# 🕵️ 4. HACKER ÉTICO — Segurança Ofensiva Aplicada à Plataforma de Poker

> **Disclaimer:** Esta seção documenta testes de segurança **autorizados** a serem
> realizados na própria plataforma, como parte do ciclo de desenvolvimento
> (DevSecOps). Todo teste deve ser conduzido em ambiente de staging, nunca em
> produção sem autorização explícita. O conhecimento aqui documentado serve para
> **defender** a plataforma — não para atacar terceiros.

> **Princípio:** "Pensa como um atacante para defender como um guardião."
> A segurança de uma plataforma de poker online é **existencial** — um único
> incidente de fraude pode destruir a confiança dos jogadores para sempre.

## 🎯 4.0 FILOSOFIA PENTEST — Metodologia de Testes de Penetração para Poker

> **Fundamento:** Pentest não é "hackear para ver se consegue" — é uma disciplina
> estruturada com metodologia, escopo, regras de engajamento e relatório formal.
> Para uma plataforma de poker online, o pentest é a **última linha de defesa**
> antes de colocar dinheiro real e confiança dos jogadores em risco.

### 📜 4.0.1 Por Que Pentest é Existencial para Poker Online

Uma plataforma de poker não é um e-commerce comum. Ela combina:

- **Dinheiro real em movimento** (depósitos, saques, rake, side pots)
- **RNG criptográfico** que deve ser impredictível (se quebrado, jogo é injusto)
- **Tempo real** (WebSocket, decisões em segundos, race conditions)
- **Antifraude** (colusão, bots, chip dumping, multi-accounting)
- **Dados sensíveis** (CPF, contas bancárias, PIX, cartão de crédito — LGPD/PCI)
- **Confiança** — se um jogador desconfia que o jogo é injusto, a plataforma morre

> **Verdade dura:** "Em poker online, a segurança não é um recurso — é o produto.
> Se a segurança falha, não há segundo lance." — Mark

### 📐 4.0.2 Metodologias Formais de Pentest

O pentest da plataforma seguirá três metodologias reconhecidas internacionalmente,
combinadas e adaptadas para o contexto de poker online:

| # | Metodologia | Origem | Foco Principal | Aplicação no Poker |
|---|-------------|--------|----------------|---------------------|
| 1 | **PTES** (Penetration Testing Execution Standard) | Comunidade de segurança | Processo completo do início ao fim | Estrutura todas as fases do pentest da plataforma |
| 2 | **OWASP WSTG** (Web Security Testing Guide) | OWASP | Testes de aplicação web | Testes específicos de API Axum, WebSocket, frontend Dioxus (Seção 4.1) |
| 3 | **NIST SP 800-115** | NIST (EUA) | Guia técnico de testes de segurança | Validação governamental, necessário para compliance |
| 4 | **OSSTMM** (Open Source Security Testing Methodology Manual) | ISECOM | Auditoria de segurança operacional | Testes de infraestrutura, rede, Docker, Kubernetes |
| 5 | **MITRE ATT&CK** | MITRE Corporation | Táticas e técnicas de adversários reais | Simulação de adversários específicos de poker (fraudadores, bots) |

> **Decisão:** PTES será o **esqueleto** (fases), OWASP WSTG o **músculo**
> (testes web), NIST SP 800-115 a **validação** (compliance), OSSTMM a **base**
> (infraestrutura) e MITRE ATT&CK a **simulação** (adversários reais).

### 🔄 4.0.3 As 7 Fases do Pentest (PTES Adaptado para Poker)

O pentest da plataforma segue 7 fases sequenciais. Cada fase tem entregáveis
documentados e critérios de saída claros:

| Fase | Nome | O Que Faz | Entregável | Ferramentas |
|------|------|-----------|------------|-------------|
| 1 | **Pré-engajamento** | Definir escopo, regras, autorização, janela de tempo | Contrato + RoE (Rules of Engagement) | Documentação, reuniões |
| 2 | **Inteligência** | Coletar info sobre a plataforma (infra, API, frontend) | Mapa de superfície de ataque | `amass`, `nmap`, `subfinder`, `wappalyzer` |
| 3 | **Modelagem de Ameaças** | Mapear vetores de ataque específicos de poker | Matriz de ameaças (STRIDE) | `Threat Dragon`, diagramas DFD |
| 4 | **Análise de Vulnerabilidades** | Scan automatizado + auditoria manual de código Rust | Lista de vulnerabilidades | `cargo audit`, `trivy`, `gitleaks`, `sqlmap` |
| 5 | **Exploração** | Tentar explorar vulnerabilidades encontradas | Provas de conceito (PoC) | `metasploit`, `burp suite`, scripts custom |
| 6 | **Pós-exploração** | Avaliar impacto real (acesso a saldo, cartas, RNG) | Relatório de impacto por vulnerabilidade | Acesso manual, pivoting |
| 7 | **Relatório** | Documentar tudo: achados, risco, recomendações | Relatório formal + executivo | Markdown, PDF, apresentação |

> **Regra de Ouro do Pentest:** "Nunca pare na exploração. O valor está no
> relatório — se não está documentado, não existe." Um pentest sem relatório
> formal é tempo perdido.

### 🎭 4.0.4 Tipos de Pentest Aplicados ao Poker

| # | Tipo | Conhecimento do Testador | Aplicação no Poker | Quando Usar |
|---|------|--------------------------|---------------------|-------------|
| 1 | **Black-box** (Zero conhecimento) | Nada — só o URL | Simular atacante externo real | Pré-lançamento, auditoria externa anual |
| 2 | **Gray-box** (Conhecimento parcial) | Credenciais de jogador comum | Simular jogador malicioso | Rotina, a cada release maior |
| 3 | **White-box** (Conhecimento total) | Código-fonte + arquitetura + credenciais admin | Auditoria interna profunda | A cada mudança em módulo crítico (RNG, rake, auth) |
| 4 | **Double-blind** | Black-box + equipe de defesa não sabe | Testar capacidade de detecção/resposta | GameDay trimestral |

> **Estratégia para poker:**
> - **White-box** a cada mudança em `rng_crypto.rs`, `auth.rs`, `rake.rs`, `side_pots.rs` (módulos críticos)
> - **Gray-box** a cada release do frontend Dioxus (simular jogador)
> - **Black-box** antes de cada go-live e auditoria anual externa
> - **Double-blind** GameDay trimestral (testar SOC/monitoramento)

### 📋 4.0.5 Regras de Engajamento (RoE) — Contrato do Pentest

Antes de qualquer pentest, um documento de **Rules of Engagement** deve ser
assinado. Para a plataforma de poker, o RoE inclui:

| # | Cláusula | Descrição |
|---|----------|-----------|
| 1 | **Escopo autorizado** | Lista exata de IPs, domínios, endpoints, horários permitidos |
| 2 | **Janela de tempo** | Datas e horários (ex: staging only, 02:00-06:00 BRT) |
| 3 | **Proibições** | Não acessar dados reais de jogadores, não manipular saldos, não DoS em produção |
| 4 | **Notificação de incidente** | Se encontrar vulnerabilidade crítica, notificar em < 1h |
| 5 | **Confidencialidade** | Achados são confidenciais — NDA obrigatório |
| 6 | **Acesso a staging** | Pentest em staging com dados sintéticos, nunca em produção com dados reais |
| 7 | **Rollback** | Qualquer alteração deve ser revertida ao final |
| 8 | **Relatório** | Entrega em 5 dias úteis com severidade CVSS + recomendações |

> **Para poker online especificamente:** O RoE deve proibir explicitamente
> qualquer tentativa de manipular o RNG em produção, acessar hand history de
> jogadores reais, ou interferir em mesas ativas. O pentest ocorre em ambiente
> isolado com dados sintéticos.

### 🎰 4.0.6 Matriz de Ameaças Específicas de Poker (STRIDE + Poker)

A modelagem de ameaças usa o framework **STRIDE** da Microsoft, adaptado para os
vetores únicos de uma plataforma de poker online:

| Categoria STRIDE | Ameaça no Poker | Exemplo Concreto | Mitigação |
|------------------|-----------------|-------------------|-----------|
| **S**poofing (Falsificação) | Jogador finge ser outro | Roubo de JWT, session hijacking | MFA, JWT curto, refresh rotativo |
| **T**ampering (Adulteração) | Manipular aposta, pote, ou RNG | Enviar `{"bet": -500}`, manipular side pot | Validação server-side, RNG server-side |
| **R**epudiation (Repúdio) | Jogador nega ter apostado | "Não fui eu que fiz all-in" | Hand history imutável, logs assinados |
| **I**nformation Disclosure (Vazamento) | Ver cartas dos oponentes | Memory dump WASM, WebSocket sniff | Cartas nunca enviadas ao cliente até showdown |
| **D**enial of Service (DoS) | Derrubar a plataforma | Flood de conexões WebSocket, JSON bomb | Rate limiting, WAF, auto-scaling |
| **E**levation of Privilege (Escalonamento) | Jogador vira admin | Modificar role no JWT, IDOR em endpoint admin | RBAC estrito, validação server-side de role |

> **Priorização:** Em poker online, **Information Disclosure** (ver cartas
> alheias) e **Tampering** (manipular RNG/apostas) são as ameaças **existenciais**.
> Um jogador que vê cartas dos oponentes destrói a integridade do jogo. Um
> atacante que manipula o RNG torna todo o jogo injusto. Estas duas categorias
> recebem atenção máxima no pentest.

### 📅 4.0.7 Ciclo de Vida do Pentest Contínuo (DevSecOps Integration)

O pentest não é um evento único — é um **ciclo contínuo** integrado ao DevSecOps:

| Frequência | Tipo | Escopo | Responsável | Custo Estimado |
|------------|------|--------|-------------|----------------|
| **A cada commit** | SAST automatizado | Código Rust (clippy, audit) | CI/CD GitHub Actions | $0 (automatizado) |
| **A cada build** | SCA + container scan | Dependências + Docker image | CI/CD (cargo audit, trivy) | $0 (automatizado) |
| **Semanal** | DAST automatizado | API Axum + endpoints | CI/CD (OWASP ZAP baseline) | $0 (automatizado) |
| **A cada release maior** | Gray-box pentest | API + frontend + WebSocket | Equipe interna de segurança | 2-3 dias de trabalho |
| **A cada mudança crítica** | White-box pentest | `rng_crypto.rs`, `auth.rs`, `rake.rs` | Arquiteto de segurança | 1-2 dias por módulo |
| **Trimestral** | Double-blind GameDay | Plataforma inteira + SOC | Equipe vermelha vs azul | 1 dia inteiro |
| **Semestral** | Black-box pentest externo | Plataforma inteira (staging) | Firma externa de segurança | R$ 15-50 mil |
| **Anual** | Auditoria completa | Tudo + compliance (LGPD, PCI) | Firma externa + auditor | R$ 50-150 mil |

> **Princípio da Defesa em Profundidade:** "Uma camada de segurança pode falhar.
> Múltiplas camadas, testadas continuamente por métodos diferentes, reduzem o
> risco a níveis aceitáveis." O pentest contínuo é a camada que valida todas as
> outras.

### 🏆 4.0.8 Bug Bounty Program — Crowdsourced Security

Após o go-live, a plataforma manterá um **Bug Bounty Program** para receber
relatórios de pesquisadores de segurança externos:

| # | Severidade (CVSS) | Exemplo no Poker | Recompensa (R$) | Prazo de Resolução |
|---|-------------------|-------------------|-----------------|---------------------|
| 1 | **Crítica** (9.0-10.0) | RNG previsível, ver cartas alheias, RCE | R$ 5.000 - 20.000 | 24h (hotfix) |
| 2 | **Alta** (7.0-8.9) | SQL Injection, bypass de auth, IDOR de saldo | R$ 1.000 - 5.000 | 7 dias |
| 3 | **Média** (4.0-6.9) | XSS, CSRF, information disclosure limitado | R$ 200 - 1.000 | 30 dias |
| 4 | **Baixa** (0.1-3.9) | Headers faltando, configuração subótima | R$ 50 - 200 | 90 dias |
| 5 | **Informativa** | Best practice, hardening | Reconhecimento público | — |

> **Plataformas recomendadas:** HackerOne, Bugcrowd, ou Intigriti.
> **Escopo do bug bounty:** API Axum, frontend Dioxus/WASM, WebSocket, infraestrutura.
> **Fora do escopo:** DoS, social engineering, physical attacks, spam.

### 📊 4.0.9 Métricas de Sucesso do Programa de Pentest

| # | Métrica | Meta | Frequência de Medição |
|---|---------|------|----------------------|
| 1 | **Vulnerabilidades críticas em produção** | 0 | Contínuo |
| 2 | **Vulnerabilidades altas em produção** | 0 | Contínuo |
| 3 | **Tempo de correção (crítica)** | < 24h | Por vulnerabilidade |
| 4 | **Tempo de correção (alta)** | < 7 dias | Por vulnerabilidade |
| 5 | **Cobertura de testes de segurança** | 100% dos endpoints da API | A cada release |
| 6 | **MTTR (Mean Time To Remediate)** | < 30 dias (média) | Mensal |
| 7 | **Bug bounty: relatórios válidos/mês** | 5-20 (plataforma madura) | Mensal |
| 8 | **GameDay: ameaças detectadas** | > 80% | Trimestral |
| 9 | **Pentest externo: 0 críticas/altas** | 0 | Anual |
| 10 | **Recompensa total bug bounty/ano** | R$ 10-50 mil | Anual |

> **KPI mestre:** "Zero vulnerabilidades críticas ou altas em produção, em
> qualquer momento." Esta é a métrica que define se a plataforma é segura
> o suficiente para operar com dinheiro real.

### 🧭 4.0.10 Código de Ética do Pentester de Poker

O pentester que trabalha na plataforma deve seguir um código de ética rigoroso:

1. **Autorização prévia** — Nunca testar sem contrato e RoE assinados
2. **Escopo estrito** — Não acessar nada fora do escopo autorizado
3. **Dados reais intocáveis** — Nunca copiar, modificar ou exfiltrar dados de jogadores
4. **RNG intocável em produção** — Nunca testar manipulação de RNG em produção
5. **Confidencialidade total** — Achados são confidenciais, NDA perpétuo
6. **Relatório honesto** — Não inflar severidade, não omitir achados
7. **Não causar dano** — Não destruir dados, não causar downtime, não criar backdoors
8. **Responsabilidade disclosure** — Reportar ao fabricante antes de divulgar publicamente
9. **Respeito à lei** — Cumprir LGPD, Lei Carolina Dieckmann, CFAA, Computer Misuse Act
10. **Melhoria contínua** — Compartilhar conhecimento com a equipe de defesa

> **Juramento do Pentester de Poker:**
> "Juro usar meu conhecimento para proteger, nunca para explorar.
> Encontrarei vulnerabilidades antes dos atacantes, reportarei com honestidade,
> e nunca usarei meu acesso para prejudicar jogadores ou a integridade do jogo.
> A confiança dos jogadores é sagrada — minha missão é merecê-la."

---

## 🕵️ 4.1 FRAMEWORK OWASP WSTG (Web Security Testing Guide)

O OWASP WSTG define 11 categorias de testes de segurança. Abaixo, cada categoria
adaptada para a plataforma de poker:

### 🕵️ 4.1.1 WSTG-INFO — Information Gathering (Coleta de Inteligência da Mesa de Poker)

| # | Teste                    | Aplicação no Poker                                              | Como Testar                     |
|---|--------------------------|-----------------------------------------------------------------|---------------------------------|
| 1 | Footprint da infraestrutura | Mapear todos os subdomínios, IPs, serviços expostos          | `amass`, `subfinder`, `nmap`    |
| 2 | Metadados expostos       | Verificar se repositório Git expõe chaves, configs, .env        | `trufflehog`, `git-secrets`     |
| 3 | Tecnologias detectáveis  | Identificar versões de Rust, PostgreSQL, Redis, Kafka          | `wappalyzer`, análise de headers|
| 4 | Portas abertas           | Escanear portas não essenciais expostas                         | `nmap -sV`                      |
| 5 | DNS e certificados       | Verificar certificados TLS válidos, DNS não vaza info           | `sslscan`, `testssl.sh`         |

### 🕵️ 4.1.2 WSTG-CONF — Configuration and Deployment Management Testing (Configuração do Motor de Poker)

| # | Teste                      | Aplicação no Poker                                              | Como Testar                          |
|---|----------------------------|-----------------------------------------------------------------|--------------------------------------|
| 1 | Docker hardening           | Container do poker roda como non-root, read-only filesystem     | `docker scout`, `trivy`              |
| 2 | Secrets no código          | Nenhuma chave JWT, senha de DB, API key no código              | `gitleaks`, `trufflehog`             |
| 3 | Configuração padrão segura | PostgreSQL, Redis e Kafka sem credenciais padrão                | Auditoria manual do docker-compose   |
| 4 | Headers de segurança       | HSTS, X-Frame-Options, CSP, X-Content-Type-Options              | `curl -I`, scanner de headers         |
| 5 | CORS configurado           | Apenas origens permitidas (não `*`)                             | Testar preflight com origens arbitrárias |
| 6 | TLS 1.3 obrigatório        | Nenhum protocolo antigo (SSL, TLS 1.0/1.1/1.2)                  | `testssl.sh --severity HIGH`         |

### 🕵️ 4.1.3 WSTG-IDNT — Identity Management Testing (Identidade do Jogador de Poker)

| # | Teste                          | Aplicação no Poker                                  | Como Testar                              |
|---|--------------------------------|-----------------------------------------------------|------------------------------------------|
| 1 | Registro de conta múltipla     | Mesma pessoa não pode criar 2+ contas               | `multi_account.rs` — testar fingerprint |
| 2 | Enumeração de usuários         | Não revelar se email já está cadastrado             | Tentar registro com email existente      |
| 3 | Verificação de email           | Não é possível jogar sem verificar email           | Tentar entrar em mesa sem verificar      |
| 4 | Validação de idade             | Jogadores menores de 18 não podem se registrar      | Tentar registro com data < 18 anos       |
| 5 | KYC (Know Your Customer)       | Verificação de identidade antes de saque            | Tentar sacar sem KYC aprovado            |

### 🕵️ 4.1.4 WSTG-ATHN — Authentication Testing (Autenticação de Jogadores)

| #  | Teste                          | Aplicação no Poker                                  | Como Testar                              |
|----|--------------------------------|-----------------------------------------------------|------------------------------------------|
| 1  | **JWT expirado**               | Token expirado deve ser rejeitado                   | Enviar request com token expirado há 1h  |
| 2  | **JWT assinatura inválida**    | Token com assinatura alterada deve ser rejeitado    | Modificar payload, recalcular sem chave  |
| 3  | **JWT none algorithm**        | `alg: none` deve ser rejeitado                      | Craft token com `{"alg":"none"}`        |
| 4  | **JWT brute force de chave**   | Chave HMAC deve ser longa e aleatória                | `hashcat -m 16500 token.txt`             |
| 5  | **MFA bypass**                | Código TOTP expirado deve ser rejeitado             | Enviar código de 30s atrás               |
| 6  | **MFA brute force**           | 6 dígitos = 1M combinações → rate limit obrigatório | Tentar 1000 códigos em 1s                |
| 7  | **Login throttling**          | 5 tentativas falhas → bloqueio temporário           | `hydra` ou script de brute force         |
| 8  | **Password policy**           | Senhas fracas rejeitadas (mín 12 chars, complexidade) | Tentar `123456`, `password`           |
| 9  | **bcrypt cost**               | Cost ≥ 12 (tempo de hash > 250ms)                   | Verificar `bcrypt::hash(cost=12)`        |
| 10 | **Session fixation**          | Session ID regenerado após login                     | Comparar session ID antes/depois         |
| 11 | **Remember me seguro**        | Token de "lembrar-me" é rotativo e com expiração    | Analisar cookie de longa duração         |
| 12 | **Reautenticação para saque** | Saque requer senha + MFA novamente                  | Tentar sacar sem reautenticar            |

### 🕵️ 4.1.5 WSTG-ATHZ — Authorization Testing (Autorização de Acesso às Mesas)

| # | Teste                                              | Aplicação no Poker                          | Como Testar                                |
|---|----------------------------------------------------|---------------------------------------------|--------------------------------------------|
| 1 | **RBAC — escalonamento de privilégio**              | Jogador não pode virar admin                | Modificar role no JWT, tentar endpoint admin |
| 2 | **IDOR — acesso a mesa alheia**                     | Jogador não pode ver cartas de outra mesa   | Trocar `table_id` no request               |
| 3 | **IDOR — hand history alheia**                      | Jogador não pode ver histórico de outros    | Trocar `hand_id` no request                |
| 4 | **IDOR — saldo alheio**                             | Jogador não pode ver saldo de outro         | Trocar `user_id` no request                |
| 5 | **IDOR — transação alheia**                         | Jogador não pode ver saque de outro         | Trocar `transaction_id`                    |
| 6 | **Forced browsing**                                 | Acessar endpoints admin como jogador        | `ffuf`, `gobuster` em `/api/admin/*`       |
| 7 | **Insecure direct object reference em torneios**    | Entrar em torneio sem pagar buy-in          | Manipular request de registro              |

### 🕵️ 4.1.6 WSTG-SESS — Session Management Testing (Sessões de Mesa de Poker)

| # | Teste                       | Aplicação no Poker                                    | Como Testar                           |
|---|-----------------------------|-------------------------------------------------------|---------------------------------------|
| 1 | **Session hijacking**        | Roubo de session → acesso à conta                     | Capturar session, usar em outro IP    |
| 2 | **Session fixation**         | Session ID não muda após login                        | Comparar antes/depois do login        |
| 3 | **Session timeout**          | Sessão expira após inatividade (30min)                | Deixar sessão ociosa 31min            |
| 4 | **Concurrent sessions**      | Mesma conta em 2 dispositivos → bloquear ou permitir? | Logar em 2 browsers simultaneamente   |
| 5 | **WebSocket session**        | Conexão WS autenticada e não sequestrável             | Capturar WS, tentar replay em outra sessão |
| 6 | **Logout invalida sessão**   | Após logout, token JWT na blocklist                   | Usar token após logout                |

### 🕵️ 4.1.7 WSTG-INPV — Input Validation Testing (Validação de Ações do Jogador)

| #  | Teste                              | Aplicação no Poker                                  | Como Testar                            |
|----|------------------------------------|-----------------------------------------------------|----------------------------------------|
| 1  | **SQL Injection**                  | Login com `' OR 1=1 --`                             | `sqlmap`, payloads manuais             |
| 2  | **SQL Injection em hand history**  | Buscar mão com `hand_id = "1; DROP TABLE hands"`    | `sqlmap`                               |
| 3  | **XSS no chat da mesa**            | Enviar `<script>alert('xss')</script>` no chat       | Tentar executar JS no chat             |
| 4  | **XSS armazenado em username**     | Nome com `<img onerror=alert(1)>`                   | Registrar com username malicioso       |
| 5  | **Command Injection**              | Inputs não devem executar comandos OS               | `; rm -rf /`, `$(whoami)`              |
| 6  | **Integer Overflow em apostas**    | Aposta de `u64::MAX` → overflow                     | Enviar aposta de 18446744073709551615  |
| 7  | **NaN/Infinity em potes**          | Pote com `f64::NAN` → cálculo quebra                | Enviar `{"pot": "NaN"}`               |
| 8  | **Negative values em depósito**    | Depositar R$ -100 → saldo aumenta                   | Enviar `{"amount": -100}`             |
| 9  | **XML/JSON bombing**               | Payload gigante → DoS                               | Enviar 10MB de JSON                    |
| 10 | **Template injection (SSTI)**      | Se usar templates, injetar `{{7*7}}`                | Tentar eval de código                  |

### 🕵️ 4.1.8 WSTG-ERRH — Error Handling Testing (Tratamento de Erros do Motor de Poker)

| # | Teste                            | Aplicação no Poker                       | Como Testar                           |
|---|----------------------------------|------------------------------------------|---------------------------------------|
| 1 | **Stack trace exposto**          | Erros não revelam código Rust            | Provocar erro 500, verificar resposta |
| 2 | **Mensagens de erro genéricas**  | "Erro interno" em vez de detalhes        | Provocar panic, verificar resposta    |
| 3 | **Códigos de erro informativos** | Não revelar se usuário existe            | Login com email inexistente           |
| 4 | **Panic do Rust tratado**        | `catch_unwind` captura panics            | Provocar panic em cálculo de rake     |

### 🕵️ 4.1.9 WSTG-CRYP — Cryptography Testing (Criptografia do Baralho e RNG)

| # | Teste                       | Aplicação no Poker                               | Como Testar                           |
|---|-----------------------------|--------------------------------------------------|---------------------------------------|
| 1 | **RNG criptográfico**       | `rng_crypto.rs` usa `OsRng`, não `thread_rng`    | Auditar código, testar entropia       |
| 2 | **RNG previsível**          | Seed não deve ser previsível                     | Tentar prever próxima carta           |
| 3 | **RNG state leakage**       | Estado interno não vaza via logs                 | Verificar logs de debug               |
| 4 | **AES-256-GCM correto**     | Nonce único por mensagem, tag verificada         | Enviar nonce repetido → deve rejeitar |
| 5 | **HMAC-SHA256 para JWT**    | Chave ≥ 256 bits, não hardcoded                  | `gitleaks` + auditoria manual         |
| 6 | **bcrypt cost ≥ 12**        | Tempo de hash > 250ms                            | Medir tempo de `bcrypt::hash`         |
| 7 | **TOTP com janela correta** | Aceita ±1 janela de 30s                          | Testar código de 60s atrás            |
| 8 | **TLS 1.3 obrigatório**     | Nenhum downgrade para TLS 1.2                    | `testssl.sh --severity HIGH`          |
| 9 | **Certificate pinning**     | App mobile faz pinning do certificado            | `frida` para tentar bypass            |

### 🕵️ 4.1.10 WSTG-BUSL — Business Logic Testing (Lógica de Negócio do Poker)

| #  | Teste                               | Aplicação no Poker                              | Como Testar                             |
|----|-------------------------------------|-------------------------------------------------|-----------------------------------------|
| 1  | **Manipulação de RNG**              | Atacante não pode prever/influenciar embaralhamento | Análise estatística de 1M mãos         |
| 2  | **Visibilidade de cartas**          | Jogador não pode ver cartas dos oponentes       | Inspecionar WebSocket, memory hacking   |
| 3  | **Bypass de colusão**               | Colusão não detectada pelo antifraude           | 2 contas jogando em coordenação         |
| 4  | **Chip dumping bypass**             | Transferência de fichas disfarçada              | Perder propositalmente para amigo       |
| 5  | **Bot injection**                   | Bot automatizado jogando 24/7                   | Script que joga automaticamente         |
| 6  | **Multi-accounting**                | Mesma pessoa com 2+ contas na mesma mesa        | 2 contas, mesmo IP/fingerprint          |
| 7  | **Aposta negativa**                 | Apostar valor negativo → ganhar fichas          | Enviar `{"bet": -500}`                 |
| 8  | **All-in duplicado**                | All-in quando já sem fichas                     | Enviar all-in com 0 fichas              |
| 9  | **Fold fora de turno**              | Fold quando não é sua vez                       | Enviar fold fora de ordem               |
| 10 | **Manipulação de blinds**           | Não pagar big blind e ainda jogar               | Tentar pular blinds                     |
| 11 | **Saque durante mão**               | Sacar fichas no meio de uma mão                 | Tentar saque durante all-in             |
| 12 | **Rake negativo**                   | Manipular para receber rake em vez de pagar     | Explorar edge case de cálculo           |
| 13 | **Loss deflator abuse**             | Perder propositalmente para receber cashback    | Perder mínimo para ativar tier          |
| 14 | **Torneio sem buy-in**              | Entrar em torneio sem pagar                     | Manipular request de registro           |
| 15 | **Re-entrar em torneio eliminado**  | Voltar após eliminação                          | Tentar re-entrar após bust              |
| 16 | **Race condition em depósito**      | Depositar 2x ao mesmo tempo → saldo duplicado   | 2 requests simultâneos de depósito      |
| 17 | **Race condition em saque**         | Sacar 2x ao mesmo tempo → sacar 2x              | 2 requests simultâneos de saque         |
| 18 | **Race condition em all-in**        | Múltiplos all-in simultâneos → side pot incorreto | 3+ all-in no mesmo tick               |

### 🕵️ 4.1.11 WSTG-CLNT — Client-Side Testing (Frontend Dioxus do Jogador)

| # | Teste                          | Aplicação no Poker                          | Como Testar                                |
|---|--------------------------------|---------------------------------------------|--------------------------------------------|
| 1 | **WebSocket hijacking (CSWSH)** | Cross-site WebSocket hijacking               | Página maliciosa conecta no WS da mesa     |
| 2 | **WebSocket origin check**     | WS aceita apenas origens permitidas          | Conectar de origem arbitrária              |
| 3 | **WASM reverse engineering**   | Código WASM não expõe lógica de cartas       | `wasm-decompile`, `wasm2wat`               |
| 4 | **Memory inspection**          | Cartas dos oponentes não estão na memória do cliente | Chrome DevTools, memory dump         |
| 5 | **DOM manipulation**           | Manipular UI para mostrar cartas escondidas  | DevTools, modificar DOM                    |
| 6 | **Local storage seguro**       | Tokens não armazenados em localStorage       | Inspecionar Application tab                |
| 7 | **PostMessage seguro**         | Mensagens entre janelas validam origem       | `window.postMessage` de origem arbitrária  |

## 🎯 4.2 ATAQUES ESPECÍFICOS DE POKER ONLINE — Vetores do Motor

### 🎲 4.2.1 Manipulação de RNG (Random Number Generator) — Embaralhamento do Baralho

**Risco:** Se um atacante conseguir prever o embaralhamento, ele sabe todas as
cartas antes dos oponentes. Isso é o **pior ataque possível** em poker online.

| # | Vetor de Ataque                    | Defesa                                              | Teste                              |
|---|------------------------------------|-----------------------------------------------------|------------------------------------|
| 1 | **Seed previsível**                | Usar `OsRng` (entropia do SO), nunca `thread_rng` ou seed baseada em timestamp | Auditar `rng_crypto.rs`           |
| 2 | **State leakage via logs**         | Nunca logar estado interno do RNG                   | `grep -r "rng" *.log`             |
| 3 | **State leakage via timing**       | Tempo de resposta não revela entropia               | Medir timing de distribuição       |
| 4 | **State leakage via side-channel** | Memória não vaza estado                             | Memory dump do processo            |
| 5 | **Ataque estatístico**             | Distribuição deve ser uniforme (NIST SP 800-22)     | Rodar 1M mãos, chi-quadrado        |
| 6 | **Ataque de reseed**               | Reseed não deve ser previsível                      | Tentar forçar reseed               |
| 7 | **Múltiplas instâncias**           | Cada mesa tem seu próprio RNG state                 | Verificar isolamento               |

### 🤝 4.2.2 Colusão (Collusion) — Jogadores Combinados na Mesa de Poker

**Risco:** 2+ jogadores cooperando para ganhar vantagem sobre outros jogadores na
mesa. É a fraude **mais comum** em poker online.

| # | Tipo de Colusão                                      | Como Detectar                           | Módulo            |
|---|------------------------------------------------------|-----------------------------------------|-------------------|
| 1 | **Soft play** (nunca apostar contra parceiro)        | Análise de padrões de aposta entre pares | `collusion.rs`    |
| 2 | **Squeeze play** (parceiros aumentam pote para vítima) | Análise de aumento de pote coordenado  | `collusion.rs`    |
| 3 | **Whipsaw** (parceiros re-raise para isolar vítima)  | Detectar re-raise coordenado            | `collusion.rs`    |
| 4 | **Stack mining** (parceiro perde de propósito)       | Detectar perdas intencionais            | `chip_dumping.rs` |
| 5 | **Compartilhamento de cartas**                       | Mesmo IP/fingerprint na mesma mesa      | `multi_account.rs`|
| 6 | **Sinais externos** (Discord/WhatsApp)               | Impossível detectar tecnicamente — focar em padrões | `collusion.rs` |

**Métricas de detecção:**
- Win rate anômalo entre pares de jogadores (> 65% heads-up)
- Frequência de fold entre pares (> 80% quando ambos na mesma mão)
- Correlação de apostas entre pares (Pearson > 0.7)
- Mesma mesa com frequência anômala (> 50% das sessões)

### 💸 4.2.3 Chip Dumping — Transferência Ilegal de Fichas entre Jogadores

**Risco:** Transferência ilegal de fichas de uma conta para outra, geralmente
para lavar dinheiro ou transferir saldo de contas hackeadas.

| # | Padrão de Chip Dumping                  | Como Detectar                        | Módulo               |
|---|----------------------------------------|--------------------------------------|----------------------|
| 1 | **All-in com mão fraca vs mão forte**  | Avaliar hand strength no all-in      | `chip_dumping.rs`    |
| 2 | **Perda consistente para mesmo jogador** | Win rate > 90% para um lado        | `chip_dumping.rs`    |
| 3 | **Valores redondos**                   | Transferências de R$100, R$500, R$1000 | `chip_dumping.rs`  |
| 4 | **Timing rápido**                      | All-in em < 2s (sem pensar)          | `bot_detection.rs`   |
| 5 | **Novo jogador perde rápido**          | Conta nova perde tudo em < 10 mãos   | `chip_dumping.rs`    |
| 6 | **Mesa heads-up isolada**              | 2 jogadores em mesa privada          | `lobby.rs` + `collusion.rs` |

### 🤖 4.2.4 Bot Detection — Detecção de Bots na Mesa de Poker

**Risco:** Bots automatizados jogando 24/7 com vantagem computacional injusta.

| # | Métrica de Bot                   | Como Detectar                      | Módulo               |
|---|----------------------------------|------------------------------------|----------------------|
| 1 | **Tempo de decisão constante**   | Variância de timing < 50ms         | `bot_detection.rs`   |
| 2 | **Sem pausas humanas**           | Nunca fica idle > 5min em 24h      | `bot_detection.rs`   |
| 3 | **Decisões perfeitamente ótimas**| Win rate anômalo, GTO perfeito     | `bot_detection.rs`   |
| 4 | **Múltiplas mesas simultâneas**  | > 4 mesas ao mesmo tempo           | `lobby.rs`           |
| 5 | **Padrão de mouse/teclado**      | Sem variabilidade de input         | `bot_detection.rs`   |
| 6 | **Sessões de 24h**               | Sessões > 20h sem pausa            | `bot_detection.rs`   |
| 7 | **Mesmo IP/datacenter**          | IP de VPN ou datacenter conhecido  | `multi_account.rs`   |

### 👥 4.2.5 Multi-Accounting — Múltiplas Contas do Mesmo Jogador

**Risco:** Mesma pessoa com múltiplas contas para bônus, colusão, ou evasão de ban.

| # | Fingerprint                     | Como Detectar                                  | Módulo                    |
|---|---------------------------------|------------------------------------------------|---------------------------|
| 1 | **Mesmo IP**                    | Múltiplas contas do mesmo IP                   | `multi_account.rs`        |
| 2 | **Mesmo device fingerprint**    | Canvas fingerprint, WebGL, fontes              | `multi_account.rs`        |
| 3 | **Mesmo browser fingerprint**   | User-Agent, plugins, timezone                  | `multi_account.rs`        |
| 4 | **Mesmo método de pagamento**   | Mesmo cartão de crédito, mesma conta PIX       | `multi_account.rs`        |
| 5 | **Mesmo email pattern**         | `joao1@gmail`, `joao2@gmail`                   | `multi_account.rs`        |
| 6 | **Mesma mesa simultânea**       | 2 contas na mesma mesa ao mesmo tempo          | `multi_account.rs` + `lobby.rs` |

## 💳 4.3 TESTES DE PAGAMENTO E FRAUDE FINANCEIRA — Rake e Depósitos

| #  | Teste                                | Risco                                    | Como Testar                                |
|----|--------------------------------------|------------------------------------------|--------------------------------------------|
| 1  | **Cartão de crédito roubado**        | Chargeback após depósito                 | Usar cartão de teste de fraude             |
| 2  | **PIX fraudulento**                  | Depósito não confirmado mas saldo creditado | Manipular webhook de confirmação         |
| 3  | **Saque para conta de terceiro**     | Lavagem de dinheiro                      | Sacar para conta com nome diferente        |
| 4  | **Depósito e saque imediato**        | Bonus abuse / money laundering           | Depositar → sacar sem jogar                |
| 5  | **Múltiplos depósitos pequenos**     | Structuring (evitar report)              | 10 depósitos de R$999 em 1h                |
| 6  | **Chargeback após perda**            | Jogador perde, faz chargeback            | Disputar transação após perder             |
| 7  | **Webhook de pagamento falsificado** | Atacante simula confirmação de pagamento | Enviar webhook falso com assinatura inválida |
| 8  | **Replay de webhook**                | Reenviar webhook legítimo                | Capturar webhook, reenviar                 |
| 9  | **Manipulação de saldo**             | Alterar saldo diretamente no DB          | Tentar UPDATE direto no PostgreSQL         |
| 10 | **Race condition em saque**          | Sacar 2x o mesmo saldo                   | 2 requests de saque simultâneos            |

## ✅ 4.4 CHECKLIST DE SEGURANÇA PARA LANÇAMENTO — Go-Live do Poker

### Pré-lançamento (F5)

- [ ] **TLS 1.3** obrigatório em todos os endpoints
- [ ] **HSTS** habilitado (min 1 ano, includeSubDomains, preload)
- [ ] **CSP** configurado (sem `unsafe-inline`)
- [ ] **CORS** restrito a origens permitidas
- [ ] **Rate limiting** em login, MFA, registro, saque
- [ ] **JWT** com expiração de 15min, refresh token rotativo
- [ ] **MFA/TOTP** obrigatório para saque e mudança de senha
- [ ] **bcrypt** cost ≥ 12
- [ ] **RBAC** implementado e testado
- [ ] **SQL Injection** testado (sqlmap)
- [ ] **XSS** testado em chat, username, bio
- [ ] **CSRF** tokens em todas as mutations
- [ ] **WebSocket origin** check
- [ ] **RNG criptográfico** auditado
- [ ] **Antifraude** (collusion, chip dumping, bot, multi-account) ativo
- [ ] **PCI DSS** compliance para pagamentos
- [ ] **LGPD/RGPD** compliance para dados pessoais
- [ ] **Logs de auditoria** para todas as transações
- [ ] **Backup criptografado** testado e restaurável
- [ ] **Plano de resposta a incidentes** documentado
- [ ] **Bug bounty** program lançada
- [ ] **Pen test** externo concluído
- [ ] **Chaos engineering** GameDay executado

### Pós-lançamento (contínuo)

- [ ] **Monitoramento 24/7** de anomalias
- [ ] **Alertas** de fraude em tempo real
- [ ] **Revisão semanal** de alertas de antifraude
- [ ] **Pen test** trimestral
- [ ] **Audit** anual de segurança
- [ ] **Dependency scan** semanal (`cargo audit`)
- [ ] **Container scan** a cada build (`trivy`)

## 📚 4.5 OWASP CHEAT SHEETS DE REFERÊNCIA — Biblioteca de Poker

As seguintes cheat sheets do OWASP devem ser consultadas e aplicadas:

| #  | Cheat Sheet                         | Aplicação no Poker                                  |
|----|-------------------------------------|-----------------------------------------------------|
| 1  | **Authentication Cheat Sheet**      | Login, MFA, JWT, bcrypt, rate limiting              |
| 2  | **AI Agent Security**               | Segurança de agentes de IA (coach, suporte)         |
| 3  | **Bot Management**                  | Detecção de bots no poker                           |
| 4  | **WebSocket Security**              | Comunicação de jogo em tempo real                   |
| 5  | **Zero Trust Architecture**         | Nenhuma parte da rede é confiável por padrão        |
| 6  | **LLM Prompt Injection**            | Proteção do AI coach contra injection               |
| 7  | **Docker Security**                 | Hardening de containers                             |
| 8  | **Kubernetes Security**             | Orquestração segura                                 |
| 9  | **Microservices Security**          | Comunicação entre serviços                          |
| 10 | **Secrets Management**              | Chaves JWT, senhas de DB, API keys                  |
| 11 | **Secure Coding with AI**           | Validação de código gerado por IA                   |
| 12 | **Threat Modeling**                 | Modelagem de ameaças do poker                       |
| 13 | **SQL Injection Prevention**        | Queries parametrizadas                              |
| 14 | **XSS Prevention**                  | Sanitização de chat e username                      |
| 15 | **CSRF Prevention**                 | Tokens anti-CSRF                                    |
| 16 | **Session Management**              | Sessões seguras                                     |
| 17 | **Password Storage**                | bcrypt com cost ≥ 12                                |
| 18 | **Transport Layer Protection**      | TLS 1.3                                             |
| 19 | **Input Validation**                | Validação de todas as entradas                      |
| 20 | **Error Handling**                  | Mensagens genéricas, sem stack traces               |

---

# 💼 5. PLANO DE NEGÓCIO — Visão Empreendedora da Plataforma de Poker

> **Princípio:** "Construa algo que as pessoas amem. Se as pessoas amam, você
> não precisa gastar tanto em marketing." — Adaptado de Elon Musk
>
> Uma plataforma de poker online não é apenas software — é um **ecossistema**
> onde jogadores, a casa, reguladores e parceiros coexistem em equilíbrio.
> O sucesso depende de **confiança, liquidez e experiência**.

## 🏆 5.1 VISÃO E MISSÃO — Propósito da Plataforma de Poker

### Visão
> Ser a plataforma de poker online **mais confiável e transparente** do mercado
> brasileiro e latino-americano, onde a integridade do jogo é garantida por
> criptografia de nível militar e IA antifraude de ponta.

### Missão
> Proporcionar a experiência de poker online **mais justa, segura e divertida**,
> onde cada jogador sabe que suas cartas são distribuídas com RNG criptográfico
> auditável, que nenhum bot ou colusão rouba seu dinheiro, e que cada centavo de
> rake é transparente.

### Valores
1. **Integridade acima de tudo** — RNG auditável, antifraude implacável
2. **Transparência radical** — rake visível, hand history público, provably fair
3. **Jogador em primeiro lugar** — UX excepcional, suporte humano, sem dark patterns
4. **Inovação constante** — Loss Deflator, AI Coach, torneios criativos
5. **Segurança como cultura** — DevSecOps desde o primeiro commit

## 📊 5.2 ANÁLISE DE MERCADO — Onde o Poker Online se Encaixa

### Tamanho do mercado (dados PokerIndustryPro)

| Métrica                                | Valor                    | Fonte              |
|----------------------------------------|--------------------------|--------------------|
| **Mercado global de poker online**     | ~US$ 100 bilhões/ano     | PokerIndustryPro   |
| **CAGR (taxa de crescimento)**         | ~10-12% ao ano           | PokerIndustryPro   |
| **Jogadores ativos globais**           | ~100 milhões             | PokerIndustryPro   |
| **Mercado brasileiro (estimado)**      | ~US$ 2-3 bilhões/ano     | Estimativa         |
| **Jogadores brasileiros ativos**       | ~3-5 milhões             | Estimativa         |
| **Mercado LATAM**                      | ~US$ 8-10 bilhões/ano    | Estimativa         |

### Concorrentes principais

| Concorrente                           | Forças                                  | Fraquezas                                          | Nossa Vantagem                                   |
|---------------------------------------|-----------------------------------------|----------------------------------------------------|--------------------------------------------------|
| **PokerStars**                        | Marca global, liquidez massiva          | rake alto, suporte lento, sem Loss Deflator        | Rake menor, Loss Deflator, suporte humano BR     |
| **GGPoker**                           | Inovação, rakeback agressivo            | UX complexa, foco em high-stakes                   | UX simples, foco em recreacional                |
| **PartyPoker**                        | Marca histórica                         | Liquidez baixa, tecnologia antiga                  | Stack Rust moderno, WASM rápido                 |
| **888poker**                          | Bônus atrativos                         | Software instável                                  | Estabilidade Rust, zero panics                  |
| **Plataformas BR (não reguladas)**    | Liquidez local                          | Sem KYC, sem segurança, sem antifraude             | Conformidade, segurança, antifraude IA          |

### Diferenciais competitivos (Moat)

1. **Loss Deflator** — Único no mercado: cashback automático por perdas em tiers
2. **Rust + WASM** — Performance e segurança superiores a concorrentes em JS/C++
3. **Antifraude IA** — Detecção de colusão, chip dumping, bots e multi-account em tempo real
4. **Provably Fair** — Jogadores podem verificar a justiça de cada mão
5. **AI Coach** — Treinador de poker integrado (diferencial de retenção)
6. **Suporte humano BR** — Atendimento em português, sem bots de suporte
7. **Rake transparente** — Sem taxas escondidas, rake visível em cada mão

## 🎨 5.3 MODELO DE NEGÓCIO (Business Model Canvas do Poker)

```
┌─────────────────┬─────────────────┬─────────────────┬─────────────────┬─────────────────┐
│   PARCEIROS     │   ATIVIDADES    │   RECURSOS      │   PROPOSTA      │   RELACIONAMENTO│
│                 │   PRINCIPAIS    │   PRINCIPAIS    │   DE VALOR      │   COM CLIENTES  │
├─────────────────┼─────────────────┼─────────────────┼─────────────────┼─────────────────┤
│ • Provedores de │ • Desenvolver e │ • Stack Rust    │ • Poker justo e │ • Suporte       │
│   pagamento     │   manter motor │   (motor +      │   seguro        │   humano 24/7   │
│   (PIX, cartão) │   de poker     │   frontend)    │ • Loss Deflator │ • Comunidade    │
│ • Provedores de │ • Operar        │ • Infra Docker  │   (cashback)    │   ativa         │
│   KYC           │   antifraude IA │ • Equipe de     │ • Antifraude    │ • Torneios      │
│ • Auditores     │   em tempo real│   segurança     │   implacável    │   exclusivos    │
│   independentes │ • Processar     │ • Equipe de     │ • AI Coach      │ • Programa VIP  │
│ • Streamers e   │   pagamentos   │   suporte BR    │ • Rake          │ • Rakeback      │
│   influenciadores│ • Suporte ao   │ • Certificações │   transparente  │   escalonado    │
│                 │   cliente      │   de segurança  │                 │                 │
├─────────────────┴─────────────────┴─────────────────┴─────────────────┴─────────────────┤
│                           CANAIS DE DISTRIBUIÇÃO                                          │
│ • Web app (Dioxus/WASM) • App mobile (futuro) • SEO • Streamers • Comunidade • Afiliados │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                           SEGMENTOS DE CLIENTES                                           │
│ • Recreacionais (80% receita) • Regulares (15%) • High-rollers (5%) • Iniciantes (foco)  │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                           ESTRUTURA DE CUSTOS                                            │
│ • Infra (Docker/K8s) • Pagamentos (taxas) • Equipe • Marketing • Compliance • Segurança │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                           FONTES DE RECEITA                                              │
│ • Rake (principal) • Torneios (buy-in fee) • VIP/Rakeback premium • Anúncios (futuro)    │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## 🚀 5.4 PRÁTICAS Y COMBINATOR PARA STARTUPS — Poker Edition

> "Make something people want." — Paul Graham, Y Combinator

| # | Prática YC | Aplicação no Poker | Status |
|---|-----------|---------------------|--------|
| 1 | **Make something people want** | Poker online justo, seguro, com Loss Deflator | ✅ Em desenvolvimento |
| 2 | **Launch fast, iterate faster** | MVP com cash game + torneio básico | 🔄 F2 |
| 3 | **Talk to users** | Entrevistar 50+ jogadores recreacionais | ⏳ F2 |
| 4 | **Do things that don't scale** | Suporte manual, torneios manuais no início | ⏳ F2 |
| 5 | **Pivot on feedback** | Ajustar rake, Loss Deflator, UX com base em feedback | Contínuo |
| 6 | **Focus on retention before growth** | Retenção semanal > 40% antes de escalar marketing | ⏳ F4 |
| 7 | **Default alive, not default dead** | Receita de rake > custos operacionais | ⏳ F5 |
| 8 | **Hire slow, fire fast** | Contratar apenas quando necessário | Contínuo |
| 9 | **Write clearly** | Documentação clara (este arquivo!) | ✅ |
| 10 | **Measure what matters** | DAU, MAU, rake/jogador, churn, LTV/CAC | ⏳ F3 |

## 🗺️ 5.5 ROADMAP DE LANÇAMENTO (Lean Startup do Poker)

### Fase 1 — Validação (atual)
- ✅ Motor de poker em Rust (deck, side pots, rake, RNG, auth, antifraude)
- ✅ 484 testes unitários
- 🔄 Frontend Dioxus básico
- 🔄 CI/CD GitHub Actions

### Fase 2 — MVP (Mínimo Produto Viável)
- [ ] Cash game funcional (Texas Hold'em, 9 jogadores)
- [ ] Torneio MTT básico
- [ ] Auth + MFA + KYC
- [ ] Depósito/saque via PIX
- [ ] Antifraude ativo (collusion, chip dumping, bot, multi-account)
- [ ] Hand history público (provably fair)
- [ ] Suporte humano (chat/email)
- **Meta:** 100 jogadores ativos, rake de R$ 5.000/mês

### Fase 3 — Tração
- [ ] Loss Deflator ativo
- [ ] AI Coach básico
- [ ] Programa de rakeback
- [ ] Torneios especiais (freerolls, satélites)
- [ ] App mobile (PWA)
- **Meta:** 1.000 jogadores ativos, rake de R$ 50.000/mês

### Fase 4 — Crescimento
- [ ] Programa de afiliados
- [ ] Streamers patrocinados
- [ ] Torneios com premiação garantida (GTD)
- [ ] VIP program com tiers
- [ ] AI Coach avançado (GTO analysis)
- **Meta:** 10.000 jogadores ativos, rake de R$ 500.000/mês

### Fase 5 — Escala
- [ ] Expansão LATAM (espanhol)
- [ ] App mobile nativo (iOS/Android)
- [ ] Licença regulamentada (Curação, Malta, ou UKGC)
- [ ] Bug bounty program
- [ ] Pen test externo
- **Meta:** 100.000+ jogadores ativos, rake de R$ 5M/mês

### Fase 6 — Domínio
- [ ] Expansão global
- [ ] Marketplace de skins
- [x] Poker variants cash: Short Deck + Short Deck Omaha (Hold’em legado); Stud / Omaha full-deck ainda backlog
- [ ] Esports poker
- [ ] IPO ou aquisição
- **Meta:** 1M+ jogadores ativos

## 📈 5.6 MÉTRICAS-CHAVE (KPIs) DE NEGÓCIO — Saúde da Plataforma de Poker

| KPI                        | Definição                       | Meta F2    | Meta F4     | Meta F6      |
|----------------------------|---------------------------------|------------|-------------|--------------|
| **DAU** (Daily Active Users)   | Jogadores únicos/dia            | 50         | 2.000       | 50.000       |
| **MAU** (Monthly Active Users) | Jogadores únicos/mês            | 200        | 10.000      | 200.000      |
| **DAU/MAU** (Stickiness)       | Engajamento                     | > 20%      | > 25%       | > 30%        |
| **Rake/jogador/mês**           | Receita média por usuário       | R$ 50      | R$ 80       | R$ 100       |
| **ARPU** (Average Revenue Per User) | Receita/jogador/mês        | R$ 50      | R$ 80       | R$ 100       |
| **CAC** (Customer Acquisition Cost) | Custo de aquisição          | R$ 30      | R$ 50       | R$ 40        |
| **LTV** (Lifetime Value)       | Receita total por jogador       | R$ 300     | R$ 600      | R$ 1.000     |
| **LTV/CAC**                    | Eficiência de aquisição         | > 10       | > 12        | > 25         |
| **Churn rate**                 | % que param de jogar/mês        | < 15%      | < 10%       | < 5%         |
| **Retention W4**               | % que voltam após 4 semanas     | > 25%      | > 35%       | > 45%        |
| **NPS** (Net Promoter Score)   | Satisfação                      | > 40       | > 50        | > 60         |
| **Mãos/dia**                   | Volume de jogo                  | 5.000      | 500.000     | 10M          |
| **Pote médio**                 | Tamanho médio dos potes         | R$ 50      | R$ 80       | R$ 120       |
| **Mesas ativas simultâneas**   | Liquidez                        | 10         | 500         | 10.000       |

---

# 💰 6. GESTÃO FINANCEIRA — Unit Economics do Rake e da Mesa de Poker

> **Princípio:** "Receita não é lucro. Lucro não é caixa. Caixa não é
> sobrevivência." — Adaptado de startup wisdom
>
> A gestão financeira de uma plataforma de poker é única porque a **receita
> principal (rake)** é gerada em **milissegundos**, mas os **custos de fraude e
> chargeback** podem aparecer **meses depois**.

## 🔥 6.1 BURN RATE E RUNWAY — Sobrevivência da Plataforma de Poker

### Definições

| Termo              | Definição                                          | Fórmula               |
|--------------------|----------------------------------------------------|-----------------------|
| **Burn Rate**      | Quanto dinheiro a empresa queima por mês           | Despesas totais/mês   |
| **Gross Burn**     | Despesas totais/mês (sem receita)                  | Σ despesas            |
| **Net Burn**       | Despesas - receita (queima líquida)                | Despesas - Receita    |
| **Runway**         | Meses até o dinheiro acabar                        | Caixa ÷ Net Burn      |
| **Default Alive**  | Receita > despesas (net burn negativo)             | Receita > Despesas    |
| **Default Dead**   | Receita < despesas (precisa de investimento)       | Receita < Despesas    |

### Tabela de burn rate (estimativa)

| Item                       | F2 (MVP)          | F4 (Crescimento)  | F6 (Escala)       |
|----------------------------|-------------------|-------------------|-------------------|
| **Infra (Docker/K8s)**     | R$ 2.000          | R$ 15.000         | R$ 80.000         |
| **Equipe (5→20→50 pessoas)** | R$ 50.000       | R$ 200.000        | R$ 500.000        |
| **Marketing**              | R$ 5.000          | R$ 50.000         | R$ 200.000        |
| **Compliance/Segurança**   | R$ 3.000          | R$ 20.000         | R$ 80.000         |
| **Pagamentos (taxas)**     | R$ 2.000          | R$ 20.000         | R$ 100.000        |
| **Gross Burn**             | **R$ 62.000**     | **R$ 305.000**    | **R$ 960.000**    |
| **Receita (rake)**         | R$ 5.000          | R$ 500.000        | R$ 5.000.000      |
| **Net Burn**               | **R$ 57.000**     | **-R$ 195.000**   | **-R$ 4.040.000** |
| **Status**                 | Default Dead      | Default Alive     | Default Alive     |

### Runway calculation

```
Runway = Caixa atual ÷ Net Burn mensal

Exemplo (F2):
  Caixa = R$ 500.000 (investimento inicial)
  Net Burn = R$ 57.000/mês
  Runway = 500.000 ÷ 57.000 = 8.8 meses

  → Precisa atingir Default Alive antes de 9 meses
  → Ou levantar nova rodada de investimento
```

### Alertas de runway

| Runway      | Status          | Ação                                  |
|-------------|-----------------|---------------------------------------|
| > 12 meses  | 🟢 Saudável     | Continuar executando                  |
| 6-12 meses  | 🟡 Atenção      | Planejar próxima rodada               |
| 3-6 meses   | 🔴 Crítico      | Levantar capital URGENTE              |
| < 3 meses   | ⛔ Emergência   | Cortar custos, pivotar ou vender      |

## 💵 6.2 UNIT ECONOMICS — Economia por Jogador de Poker

### CAC (Customer Acquisition Cost)

```
CAC = (Custo de Marketing + Custo de Vendas) ÷ Novos Jogadores Adquiridos

Exemplo (F4):
  Marketing = R$ 50.000/mês
  Vendas (afiliados) = R$ 10.000/mês
  Novos jogadores = 1.200/mês
  CAC = (50.000 + 10.000) ÷ 1.200 = R$ 50/jogador
```

### LTV (Lifetime Value)

```
LTV = ARPU × (1 ÷ Churn Rate) × Margem

Exemplo (F4):
  ARPU = R$ 80/mês
  Churn = 10% = 0.10
  Margem = 70% (após taxas de pagamento, infra, etc.)
  LTV = 80 × (1 ÷ 0.10) × 0.70 = 80 × 10 × 0.70 = R$ 560

LTV/CAC = 560 ÷ 50 = 11.2x  → Saudável (> 3x é bom, > 10x é excelente)
```

### Payback Period

```
Payback = CAC ÷ ARPU

Exemplo (F4):
  Payback = 50 ÷ 80 = 0.625 meses ≈ 19 dias

  → Jogador "paga" seu custo de aquisição em ~3 semanas
  → Qualquer receita após isso é lucro
```

### Tabela de unit economics por fase

| Métrica       | F2        | F4        | F6         |
|---------------|-----------|-----------|------------|
| **CAC**       | R$ 30     | R$ 50     | R$ 40      |
| **ARPU**      | R$ 50     | R$ 80     | R$ 100     |
| **Churn**     | 15%       | 10%       | 5%         |
| **LTV**       | R$ 233    | R$ 560    | R$ 1.400   |
| **LTV/CAC**   | 7.8x      | 11.2x     | 35x        |
| **Payback**   | 18 dias   | 19 dias   | 12 dias    |
| **Margem**    | 50%       | 70%       | 80%        |

## 💰 6.3 RECEITA DE RAKE — Modelo de Negócio do Poker

### Como o rake gera receita

```
Receita de Rake = Σ (Rake por mão) = Σ (Pote × Taxa de Rake, limitado ao cap)

Parâmetros atuais:
  Taxa de rake = 5% do pote
  Cap máximo = R$ 6.00 por mão
  Pote mínimo para rake = R$ 10.00
```

### Projeção de receita

| Cenário               | Jogadores/dia | Mãos/jogador/dia | Pote médio | Rake/mão      | Receita/dia   | Receita/mês    |
|-----------------------|---------------|-------------------|------------|---------------|---------------|----------------|
| **F2 (MVP)**          | 50            | 50                | R$ 50      | R$ 2.50       | R$ 6.250      | R$ 187.500     |
| **F4 (Crescimento)**  | 2.000         | 80                | R$ 80      | R$ 4.00       | R$ 640.000    | R$ 19.2M       |
| **F6 (Escala)**       | 50.000        | 100               | R$ 120     | R$ 6.00 (cap) | R$ 30M        | R$ 900M        |

> **Nota:** Os valores acima são **potencial máximo**. A receita real depende
> de retenção, liquidez e taxa de ocupação das mesas.

### Fatores que afetam receita de rake

| Fator                    | Impacto                                          | Como otimizar                           |
|--------------------------|--------------------------------------------------|-----------------------------------------|
| **Liquidez (mesas ativas)** | Mais mesas = mais mãos                           | Marketing, retenção, torneios           |
| **Pote médio**           | Potes maiores = mais rake (até cap)              | Atrair high-rollers, stakes maiores     |
| **Mãos/hora**            | Mais mãos = mais rake                            | Software rápido, sem lag, auto-fold     |
| **Taxa de rake**         | % maior = mais rake (mas menos jogadores)        | Equilibrar competitividade              |
| **Cap de rake**          | Cap menor = menos rake em potes grandes          | Ajustar por stake level                 |
| **Rakeback/VIP**         | Reduz receita líquida mas aumenta retenção       | Otimizar tiers                          |

## ⚠️ 6.4 GESTÃO DE RISCO FINANCEIRO — Proteção da Banca do Poker

### Riscos financeiros específicos de poker

| #  | Risco                        | Probabilidade | Impacto | Mitigação                                      |
|----|------------------------------|---------------|---------|------------------------------------------------|
| 1  | **Chargeback em massa**      | Média         | Alto    | KYC obrigatório, regras de chargeback          |
| 2  | **Fraude de pagamento**      | Alta          | Alto    | Antifraude, 3D Secure, análise de risco        |
| 3  | **Lavagem de dinheiro**      | Média         | Crítico | AML/KYC, reportar transações suspeitas         |
| 4  | **Colusão reduz rake**       | Alta          | Médio   | Antifraude IA, banimento                       |
| 5  | **Bots ganham dos recreacionais** | Alta     | Alto    | Bot detection, banimento, CAPTCHA              |
| 6  | **Regulação adversa**        | Média         | Crítico | Compliance proativo, licenças                  |
| 7  | **Queda de liquidez**        | Média         | Alto    | Marketing, retenção, torneios GTD              |
| 8  | **Bug de saldo**             | Baixa         | Crítico | Testes, auditoria, reconciliação diária        |
| 9  | **Ataque DDoS**              | Alta          | Médio   | Cloudflare, rate limiting, WAF                 |
| 10 | **Vazamento de dados**       | Baixa         | Crítico | Criptografia, pentest, bug bounty              |

### Reserva financeira recomendada

| Reserva           | Valor                      | Propósito                                          |
|-------------------|----------------------------|----------------------------------------------------|
| **Operacional**   | 3-6 meses de burn          | Cobrir despesas em caso de queda de receita        |
| **Chargeback**    | 5% da receita de pagamento | Cobrir chargebacks e disputas                      |
| **Fraude**        | 2% da receita de rake      | Cobrir perdas por fraude não detectada             |
| **Regulatória**   | 10% da receita             | Cobrir multas e custos de compliance               |
| **Emergência**    | 1 mês de receita           | Cobrir incidentes imprevistos                      |

## 📊 6.5 FLUXO DE CAIXA PROJETADO (Simplificado) — Poker Online

```
Mês 1-3 (F2 MVP):
  Receita:     R$   15.000 (rake inicial)
  Despesas:    R$  186.000 (3 meses × R$ 62.000)
  Net Burn:    -R$ 171.000
  Caixa final: R$  329.000 (de R$ 500.000 inicial)

Mês 4-6 (F2→F3):
  Receita:     R$  150.000 (crescimento)
  Despesas:    R$  186.000
  Net Burn:    -R$   36.000
  Caixa final: R$  293.000

Mês 7-9 (F3 Tração):
  Receita:     R$  500.000 (1.000 jogadores)
  Despesas:    R$  300.000
  Net Burn:    +R$ 200.000  ← DEFAULT ALIVE! 🎉
  Caixa final: R$  493.000

Mês 10-12 (F3→F4):
  Receita:     R$ 1.500.000
  Despesas:    R$   900.000
  Lucro:       R$   600.000
  Caixa final: R$ 1.093.000
```

---

# ⚖️ 7. ESTRUTURA JURÍDICA, FISCAL E HOSTING — Modelo Brasileiro para Poker Online

> **Princípio:** "No Brasil, a pergunta não é 'quanto imposto pago', é
> 'qual estrutura pago menos imposto **legalmente**'." — Adaptado de
> planejamento tributário
>
> O Brasil tem uma das cargas tributárias mais complexas do mundo. Para
> uma plataforma de poker online — atividade em zona cinzenta jurídica e
> com classificação tributária ambígua — a **estrutura societária** é tão
> importante quanto o código. Esta seção documenta o **modelo híbrido
> offshore + nacional** usado pela indústria (PokerStars, GGPoker,
> Partypoker) e adaptado à realidade brasileira.

## ⚖️ 7.1 REALIDADE JURÍDICA DO POKER ONLINE NO BRASIL

### ⚖️ 7.1.1 Marco Legal Atual do Poker Online no Brasil

| Lei                        | Ano  | Conteúdo                                                              | Impacto no Poker                                         |
|----------------------------|------|-----------------------------------------------------------------------|----------------------------------------------------------|
| **Decreto-Lei 3.688/1941** | 1941 | Lei de Contravenções Penais, Art. 50 proíbe "casa de jogos de azar"   | Poker = jogo de habilidade (não de azar), então tecnicamente não se enquadra |
| **Lei nº 13.756/2018**     | 2018 | Autoriza loterias (Lotex, apostas de quota)                           | **Não menciona poker online**                            |
| **Lei nº 14.790/2023**     | 2023 | Regulamenta apostas de quota (bets esportivas)                        | Poker = jogo de habilidade, **não se enquadra** como aposta de quota |
| **STF RE 966.437**         | 2020 | Julgamento sobre legalidade do poker                                  | Reconheceu que **poker é jogo de habilidade**, não de azar |
| **SPA (Secretaria de Prêmios)** | 2024+ | Nova regulamentação de jogos                                     | Em construção, **ainda não regulamenta poker online**    |

### ⚖️ 7.1.2 Status Atual: Zona Cinza do Poker Online Brasileiro

```
┌───────────────────────────────────────────────────────────────┐
│                    STATUS JURÍDICO DO POKER ONLINE             │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│   ❌ NÃO é expressamente legal (sem licença federal)          │
│   ✅ NÃO é expressamente ilegal (STF: jogo de habilidade)    │
│   ⚠️  ZONA CINZENTA — sem regulamentação específica            │
│                                                               │
│   RESULTADO: Plataformas operam offshore, atendendo BR       │
│   (PokerStars, GGPoker, 888poker, Partypoker — todas offshore)│
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### ⚖️ 7.1.3 Riscos Jurídicos do Poker Online no Brasil

| Risco                                  | Probabilidade | Impacto | Mitigação                                          |
|----------------------------------------|---------------|---------|----------------------------------------------------|
| **Ação civil pública** contra operação | Baixa         | Alto    | Operar via offshore, empresa BR é só TI            |
| **Bloqueio de PIX** para gambling      | Média         | Alto    | Processador offshore (crypto, Skrill)              |
| **Questionamento fiscal** da Receita   | Média         | Médio   | Estrutura híbrida documentada, contabilidade regular |
| **COAF** (lavagem de dinheiro)         | Baixa         | Alto    | KYC/AML completo, reportar operações suspeitas     |
| **Reclamação de jogador**              | Média         | Baixo   | Termos de uso claros, jurisdição offshore          |

## 💸 7.2 TRIBUTAÇÃO BRASILEIRA — Por Que Não Operar com Empresa BR

### 💸 7.2.1 Carga Tributária Comparada do Rake de Poker

| Jurisdição                     | Tributo sobre GGR        | Carga Total     | Observação                                              |
|--------------------------------|--------------------------|-----------------|---------------------------------------------------------|
| **Brasil (Lucro Real)**        | IRPJ 15% + CSLL 9% + PIS 1,65% + COFINS 7,6% | **~33%**        | Mais impostos sobre lucro, mais complexo                |
| **Brasil (Lucro Presumido)**   | IRPJ 24% + CSLL 9% + PIS 0,65% + COFINS 3% | **~37%**        | Presumido = tributação sobre faturamento, não lucro real |
| **Brasil (Simples Nacional)**  | 6-15%                    | **6-15%**       | ⚠️ Atividade financeira pode ser EXCLUÍDA do Simples (LC 154/2016) |
| **Curação**                    | 2% licença sobre GGR     | **2%**          | Sem IRPJ, sem CSLL, sem PIS/COFINS                      |
| **Malta**                      | 5% sobre GGR             | **5%**          | Sem tributação adicional sobre lucro distribuído        |
| **Ilha de Man**                | 1,5% sobre GGR (até £20M) | **1,5-1,5%**   | Escala regressiva após £20M                             |

### 💸 7.2.2 Por que o Simples Nacional Provavelmente NÃO Aplica ao Poker Online

```
┌───────────────────────────────────────────────────────────────┐
│  PROBLEMA: Classificação da atividade                          │
│                                                               │
│  Poker online pode ser classificado como:                     │
│  • "Atividade de jogos de azar" → EXCLUÍDO do Simples         │
│  • "Atividade financeira" → EXCLUÍDO do Simples (LC 154/2016) │
│  • "Serviço de entretenimento" → POSSÍVEL no Simples          │
│                                                               │
│  RISCO: Receita Federal pode reclassificar retroativamente    │
│  → Multa + juros + crime fiscal                               │
│                                                               │
│  CONCLUSÃO: NÃO arriscar. Operar offshore.                    │
└───────────────────────────────────────────────────────────────┘
```

### 💸 7.2.3 Tributos Adicionais sobre Jogadores de Poker (Ganho de Capital)

| Tributo                           | Alíquota       | Aplicação                                                          |
|-----------------------------------|----------------|--------------------------------------------------------------------|
| **IRRF sobre prêmios**            | 30%            | Se Receita classificar saques como "prêmio de jogo" (como loterias) |
| **IOF**                           | 0,38%          | Transferências internacionais (offshore → BR)                      |
| **Tributação sobre dividendos**   | 0% (atual) / 15-20% (reforma) | Reforma tributária pode tributar dividendos           |

## 🌐 7.3 MODELO HÍBRIDO — Offshore + Nacional (Recomendado para Poker)

### 🏛️ 7.3.1 Arquitetura da Estrutura Jurídica Híbrida (BR + Exterior)

```
┌───────────────────────────────────────────────────────────────┐
│  EMPRESA OFFSHORE (Curação ou Malta)                          │
│  • Licença de gaming (Curação: ~USD 17k/ano, Malta: ~EUR 25k) │
│  • Opera a plataforma (servidores, rake, depósitos, saques)    │
│  • Contrata processadores de pagamento (Skrill, crypto, etc)  │
│  • Recebe 100% do rake                                         │
│  • Paga 2-5% de licença sobre GGR                             │
│  • Conta bancária offshore (segregada, não mistura com BR)    │
└───────────────────────┬───────────────────────────────────────┘
                        │ paga por "serviços de desenvolvimento de software"
                        │ (contrato de prestação de serviços internacionais)
                        ▼
┌───────────────────────────────────────────────────────────────┐
│  EMPRESA BRASILEIRA (CNPJ — TI)                               │
│  • CNAE: 6201-5/01 (Desenvolvimento de Software)              │
│  • Regime: Simples Nacional (Anexo III ou V)                  │
│  • Recebe da offshore como "exportação de serviços"          │
│  • Tributação: 6-15% (muito menor que 30-40%)                 │
│  • Você como sócio: pró-labore + dividendos (0% hoje)        │
│  • Contrata desenvolvedores, paga folha, aluga escritório     │
└───────────────────────────────────────────────────────────────┘
```

### ⚖️ 7.3.2 Por Que Esse Modelo Híbrido é Legal para Poker Online

| Aspecto                                  | Justificativa                                                                                      |
|------------------------------------------|----------------------------------------------------------------------------------------------------|
| **Empresa BR é só TI**                   | Não opera poker, não recebe rake, não tem contato com jogadores. Desenvolve software e exporta serviço |
| **Exportação de serviços**               | Atividade incentivada pelo BNDES/Receita. Empresa BR pode faturar em USD/EUR                       |
| **Empresa offshore opera poker**         | Sob licença válida (Curação/Malta), em jurisdição que autoriza gaming                              |
| **Contrato entre as duas**               | Contrato de prestação de serviços internacionais, com transferência bancária documentada           |
| **Você não esconde nada**                | Tudo é declarado: empresa BR paga imposto sobre receita, você declara dividendos                   |

### 💳 7.3.3 Fluxo de Pagamentos do Rake e Depósitos de Jogadores

```
Jogador BR → Deposita via PIX/Crypto → Processador Offshore
                                              │
                                              ▼
                                    Empresa Offshore (recebe)
                                              │
                                    ┌─────────┴─────────┐
                                    │                   │
                                    ▼                   ▼
                            Rake (2-5% licença)    Repasse à Empresa BR
                            para jurisdição        (serviços de TI)
                                                    │
                                                    ▼
                                            Empresa BR (Simples)
                                            paga 6-15% imposto
                                                    │
                                                    ▼
                                            Sócio (você)
                                            pró-labore + dividendos
```

### 💰 7.3.4 Custos da Estrutura Híbrida para Operação de Poker Online

| Item                      | Custo Anual          | Observação                           |
|---------------------------|----------------------|--------------------------------------|
| **Licença Curação**       | USD 17.000           | Mais barato, reputação menor         |
| **Licença Malta (MGA)**   | EUR 25.000-50.000    | Mais caro, reputação maior           |
| **Constituição offshore** | USD 2.000-5.000 | One-time, via agência especializada |
| **Contabilidade offshore** | USD 3.000-8.000/ano | Auditores locais |
| **CNPJ BR (TI)** | R$ 1.500-3.000/ano | Contador brasileiro |
| **Processador de pagamentos** | 2-5% por transação | Skrill, Neteller, crypto gateways |
| **Advogado especializado** | R$ 5.000-15.000/ano | Consultoria jurídica gaming |
| **TOTAL estimado (F2)** | ~R$ 150.000/ano | Curação + CNPJ BR + processador |

## 🖥️ 7.4 HOSTING E INFRAESTRUTURA OFFSHORE — Servidores do Poker

### 🖥️ 7.4.1 Onde Hospedar o Motor de Poker (por jurisdição da licença)

| Jurisdição        | Provedor                | Região        | Custo Estimado     | Latência BR   |
|-------------------|-------------------------|---------------|--------------------|---------------|
| **Curação**       | DigitalOcean / Vultr    | Miami, FL     | USD 80-500/mês     | ~120ms        |
| **Malta**         | AWS (eu-south-1)        | Malta         | USD 200-2.000/mês  | ~180ms        |
| **Ilha de Man**   | AWS (eu-west-2)         | Londres       | USD 200-2.000/mês  | ~160ms        |
| **Costa Rica**    | Vultr / Linode          | Miami, FL     | USD 80-500/mês     | ~120ms        |

### 🖥️ 7.4.2 Arquitetura de Hosting Recomendada para Mesas de Poker Concorrentes

```
┌───────────────────────────────────────────────────────────────┐
│  ARQUITETURA DE HOSTING (MODELO HÍBRIDO)                       │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  [Jogador BR] → [CDN Cloudflare] → [VPS Offshore]            │
│                    │                  │                       │
│                    │                  ├─ API Axum (Rust)      │
│                    │                  ├─ Motor de jogo (Rust)  │
│                    │                  ├─ PostgreSQL            │
│                    │                  └─ Redis (cache/sessão)  │
│                    │                                          │
│                    └─ WAF + DDoS protection                   │
│                                                               │
│  [Backup] → S3-compatible (offshore, mesma jurisdição)       │
│                                                               │
│  [DNS] → .com (sem .com.br, evita jurisdição BR sobre domínio)│
│                                                               │
│  [Monitoring] → Grafana Cloud / Datadog (qualquer região)    │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 🌐 7.4.3 VPN — Quando Usar e Quando NÃO Usar no Poker Online

| Cenário                                      | VPN necessária?  | Justificativa                                                      |
|----------------------------------------------|------------------|--------------------------------------------------------------------|
| **Acessar servidor offshore (SSH)**          | ✅ Sim           | Segurança, não evasão                                              |
| **Esconder operação da Receita**             | ❌ NÃO           | Estrutura híbrida é legal, não precisa esconder                    |
| **Acessar serviços bloqueados por IP BR**    | ✅ Sim           | Alguns processadores bloqueiam IP brasileiro                       |
| **Jogadores acessando a plataforma**         | ❌ NÃO           | Jogadores acessam via CDN, sem VPN                                 |

> **⚠️ IMPORTANTE:** VPN **não é uma estratégia fiscal**. É uma
> ferramenta de acesso. A estratégia fiscal é a **estrutura societária
> híbrida** (offshore + CNPJ BR). VPN apenas facilita acesso técnico a
> servidores e serviços offshore.

## 🪪 7.5 KYC/AML/COAF — Conformidade Brasileira do Poker

### 🪪 7.5.1 Requisitos COAF (Conselho de Controle de Atividades Financeiras) para Poker

| Requisito                                      | Implementação                                        | Status          |
|------------------------------------------------|------------------------------------------------------|-----------------|
| **KYC (identificação do cliente)**             | Coletar CPF, RG, comprovante de residência           | ⏳ F2           |
| **Due diligence ampliada**                     | Para jogadores high-roller (>R$ 50k/mês)             | ⏳ F3           |
| **Reportar operação suspeita (COAF)**          | Transações > R$ 100k ou padrão anômalo               | ⏳ F3           |
| **Manter registros por 5 anos**                | Hand history + logs financeiros                      | ✅ (hand_history.rs) |
| **Plano de AML**                               | Documento formal, revisado anualmente                | ⏳ F2           |

### 🪪 7.5.2 Fluxo KYC/AML de Jogadores de Poker (Know Your Customer)

```
Jogador se cadastra
    │
    ▼
[Coleta de dados] → CPF, RG, selfie, comprovante residência
    │
    ▼
[Verificação automática] → API de validação (ex: CAF, Sumsub)
    │
    ├── ✅ Aprovado → Conta ativa, pode depositar
    ├── ⚠️ Manual → Análise humana (high-roller)
    └── ❌ Rejeitado → Conta bloqueada, fundos devolvidos
    │
    ▼
[Monitoramento contínuo] → Antifraude (collusion, chip dumping, multi-account)
    │
    ▼
[Operação suspeita?] → Reportar ao COAF em até 30 dias
```

## ✅ 7.6 CHECKLIST DE IMPLEMENTAÇÃO — Estrutura Jurídica do Poker

| #  | Item                                          | Fase   | Status   |
|----|-----------------------------------------------|--------|----------|
| 1  | **Constituir CNPJ BR (TI, CNAE 6201-5/01)**  | F1     | ⏳       |
| 2  | **Optar pelo Simples Nacional (Anexo III/V)** | F1     | ⏳       |
| 3  | **Contratar advogado especializado em gaming** | F1    | ⏳       |
| 4  | **Constituir empresa offshore (Curação)**     | F2     | ⏳       |
| 5  | **Obter licença de gaming (Curação)**         | F2     | ⏳       |
| 6  | **Abrir conta bancária offshore**             | F2     | ⏳       |
| 7  | **Contratar processador de pagamento**        | F2     | ⏳       |
| 8  | **Contrato de prestação de serviços (offshore → BR)** | F2 | ⏳       |
| 9  | **Implementar KYC/AML completo**              | F2     | ⏳       |
| 10 | **Plano AML formal (COAF)**                   | F2     | ⏳       |
| 11 | **Migrar para Malta (MGA) se escalar**        | F3     | ⏳       |
| 12 | **Avaliar licença BR (SPA) quando regulamentar** | F5+ | ⏳       |

## ⚠️ 7.7 RISCOS E MITIGAÇÕES — Resumo Executivo do Poker

| Risco                                            | Probabilidade | Impacto | Mitigação                                            |
|--------------------------------------------------|---------------|---------|------------------------------------------------------|
| **Receita Federal reclassifica atividade**       | Média         | Alto    | Empresa BR é só TI, não opera poker                  |
| **COAF multa por não reportar**                  | Baixa         | Alto    | KYC/AML completo, reportar suspeitas                 |
| **Bloqueio de PIX para gambling**                | Média         | Alto    | Processador offshore (crypto, Skrill)                |
| **Ação civil pública**                           | Baixa         | Alto    | Offshore + termos de uso (jurisdição offshore)       |
| **Reforma tributária tributa dividendos**        | Alta          | Médio   | Planejar fluxo de dividendos com contador            |
| **Reputação (offshore = ilegal?)**               | Média         | Médio   | Comunicação: "licenciado em Curação/Malta"           |

---

# 📣 8. MARKETING — Funil AARRR e Crescimento da Comunidade de Poker

> **Princípio:** "Marketing não é sobre vender, é sobre **compartilhar
> paixão**." — Adaptado de marketing moderno
>
> Poker é uma **comunidade**, não apenas um produto. O marketing de poker
> deve construir **confiança, comunidade e paixão pelo jogo**.

## 🏴‍☠️ 8.1 FUNIL AARRR (Pirate Metrics do Poker)

```
    ┌─────────────────────────────────────────────────────────────┐
    │                    FUNIL AARRR                              │
    │                                                             │
    │  A ──→ Acquisition (Aquisição)                              │
    │  │      "Quantos visitantes chegam ao site?"                │
    │  │                                                          │
    │  ▼                                                          │
    │  A ──→ Activation (Ativação)                                │
    │  │      "Quantos se registram e jogam a 1ª mão?"             │
    │  │                                                          │
    │  ▼                                                          │
    │  R ──→ Retention (Retenção)                                 │
    │  │      "Quantos voltam a jogar na semana seguinte?"        │
    │  │                                                          │
    │  ▼                                                          │
    │  R ──→ Revenue (Receita)                                    │
    │  │      "Quantos geram rake?"                                │
    │  │                                                          │
    │  ▼                                                          │
    │  R ──→ Referral (Indicação)                                 │
    │         "Quantos indicam amigos?"                           │
    └─────────────────────────────────────────────────────────────┘
```

### 🎯 7.1.1 Acquisition (Aquisição) — Atraindo Novos Jogadores de Poker

| #  | Canal                                | Estratégia                                              | CAC Alvo  | Quando   |
|----|--------------------------------------|---------------------------------------------------------|-----------|----------|
| 1  | **SEO**                              | Conteúdo: "como jogar poker", "regras Texas Hold'em", "melhores mãos" | R$ 5      | F2       |
| 2  | **Google Ads**                       | Palavras-chave: "poker online", "jogar poker Brasil"    | R$ 40     | F3       |
| 3  | **Meta Ads (Facebook/Instagram)**    | Targeting: interesses em poker, cassino, jogos          | R$ 35     | F3       |
| 4  | **YouTube Ads**                      | Vídeos antes de conteúdo de poker                       | R$ 30     | F3       |
| 5  | **TikTok Ads**                       | Vídeos curtos de jogadas épicas, bad beats              | R$ 20     | F4       |
| 6  | **Streamers (Twitch/YouTube)**       | Patrocinar streamers de poker BR                        | R$ 50     | F3       |
| 7  | **Afiliados**                        | Comissão de 30-50% do rake gerado por jogador indicado  | R$ 25     | F4       |
| 8  | **Comunidade (Discord/Reddit)**      | Presença orgânica em comunidades de poker               | R$ 5      | F2       |
| 9  | **Freerolls**                        | Torneios gratuitos para atrair novos jogadores          | R$ 15     | F2       |
| 10 | **Indicação (Referral)**             | Bônus para quem indica amigos                           | R$ 10     | F3       |

### ✅ 7.1.2 Activation (Ativação) — Primeira Mão de Poker do Jogador

**Definição de "Ativado":** Jogador se registrou, verificou email, fez 1º depósito e jogou a 1ª mão.

| #  | Etapa de Ativação               | Taxa Alvo   | Otimização                                            |
|----|---------------------------------|-------------|-------------------------------------------------------|
| 1  | Visitante → Registro            | > 10%       | Landing page clara, CTA forte                         |
| 2  | Registro → Verificação de email | > 90%       | Email imediato, reenvio automático                    |
| 3  | Verificação → 1º depósito       | > 30%       | Bônus de 1º depósito (100% match até R$500)           |
| 4  | Depósito → 1ª mão               | > 80%       | Tutorial interativo, mesa de iniciantes               |
| 5  | 1ª mão → 10 mãos                | > 60%       | Freeroll de boas-vindas, missões                      |

**Aha Moment:** O momento em que o jogador sente o "uau!" — para poker, é ganhar
a primeira mão ou ver o flop pela primeira vez. Otimizar para que isso aconteça
nos primeiros 5 minutos.

### 🔄 7.1.3 Retention (Retenção) — Jogadores Voltando às Mesas de Poker

| #  | Cohort              | Retenção Alvo   | Estratégia                              |
|----|---------------------|-----------------|-----------------------------------------|
| 1  | **D1** (dia 1)      | > 40%           | Notificação push, torneio diário        |
| 2  | **D7** (semana 1)   | > 25%           | Missões semanais, rakeback              |
| 3  | **D30** (mês 1)     | > 20%           | Torneio mensal, VIP tier                |
| 4  | **W4** (semana 4)   | > 25%           | Loss Deflator (cashback por perdas)     |
| 5  | **M3** (mês 3)      | > 15%           | Programa VIP, torneios exclusivos       |

**Estratégias de retenção específicas para poker:**

| Estratégia                    | Como Funciona                                      | Impacto na Retenção          |
|-------------------------------|----------------------------------------------------|------------------------------|
| **Loss Deflator**             | Cashback por equity no all-in (7/15/25/35%)        | Alta — reduz churn de perdedores |
| **Rakeback**                  | Devolução de rake semanal (5-30% por tier)         | Alta — recompensa volume     |
| **Missões diárias**           | "Jogue 20 mãos", "Ganhe 3 all-in"                  | Média — gamificação          |
| **Freerolls semanais**        | Torneio gratuito para depositantes                 | Média — razão para voltar    |
| **Torneios GTD**              | Premiação garantida (ex: R$10.000 GTD)             | Alta — atrai volume          |
| **Bad Beat Jackpot**          | Jackpot progressivo para quem perde com quadra+    | Alta — sonho de jackpot      |
| **Leaderboards**              | Ranking semanal/mensal com prêmios                 | Média — competição           |
| **AI Coach**                  | Treinador que melhora o jogo do jogador            | Alta — valor agregado        |
| **Comunidade**                | Chat ativo, Discord, eventos                       | Média — pertencimento        |
| **Notificações inteligentes** | "Sua mesa favorita tem vaga!"                      | Média — reativação           |

### 💰 7.1.4 Revenue (Receita) — Monetização via Rake das Mesas de Poker

| #  | Fonte de Receita                | % da Receita  | Margem   | Quando   |
|----|--------------------------------|---------------|----------|----------|
| 1  | **Rake de cash game**          | 60%           | 70%      | F2       |
| 2  | **Fee de torneios**            | 25%           | 75%      | F2       |
| 3  | **Rakeback premium (VIP)**     | 5%            | 50%      | F4       |
| 4  | **Loss Deflator (comissão)**   | 5%            | 60%      | F3       |
| 5  | **Anúncios (futuro)**          | 3%            | 90%      | F6       |
| 6  | **Skins/Customização**         | 2%            | 95%      | F5       |

### 👥 7.1.5 Referral (Indicação) — Boca a Boca entre Jogadores de Poker

| #  | Programa                | Mecânica                                | Recompensa                       |
|----|-------------------------|-----------------------------------------|----------------------------------|
| 1  | **Indique um amigo**    | Amigo se registra + deposita            | R$ 50 para ambos                 |
| 2  | **Afiliado**            | Afiliado indica jogadores               | 30-50% do rake gerado            |
| 3  | **Stream partner**      | Streamer joga na plataforma             | Rakeback + patrocínio            |
| 4  | **Community leader**    | Líder de comunidade traz grupo          | Torneio privado + bônus          |

## ✍️ 8.2 ESTRATÉGIA DE CONTEÚDO E SEO — Marketing de Poker

### Palavras-chave alvo

| #  | Keyword                       | Volume/mês (BR)  | Dificuldade  | Intenção                    |
|----|-------------------------------|------------------|--------------|-----------------------------|
| 1  | "poker online"                | 50.000           | Alta         | Transacional                |
| 2  | "jogar poker"                 | 30.000           | Média        | Transacional                |
| 3  | "regras texas hold'em"        | 15.000           | Baixa        | Informacional               |
| 4  | "como jogar poker"            | 20.000           | Média        | Informacional               |
| 5  | "melhores mãos de poker"      | 10.000           | Baixa        | Informacional               |
| 6  | "rakeback poker"              | 5.000            | Baixa        | Transacional                |
| 7  | "freeroll poker"              | 8.000            | Baixa        | Transacional                |
| 8  | "torneio de poker online"     | 12.000           | Média        | Transacional                |
| 9  | "poker com dinheiro real"     | 8.000            | Alta         | Transacional                |
| 10 | "loss deflator poker"         | 100              | Baixa        | Informacional (própria)     |

### Calendário de conteúdo

| Frequência        | Conteúdo                          | Canal                    |
|-------------------|-----------------------------------|--------------------------|
| **Diário**        | Jogada do dia, dica rápida        | Instagram, TikTok        |
| **3x/semana**     | Artigo de blog (SEO)              | Blog                     |
| **Semanal**       | Video de estratégia               | YouTube                  |
| **Semanal**       | Stream ao vivo                    | Twitch                   |
| **Mensal**        | Torneio da comunidade             | Plataforma               |
| **Mensal**        | Newsletter                        | Email                    |

## 🎙️ 8.3 PARCERIA COM STREAMERS E INFLUENCIADORES — Poker Edition

| #  | Tipo                                    | Estratégia                              | Custo                    | ROI Esperado   |
|----|-----------------------------------------|-----------------------------------------|--------------------------|----------------|
| 1  | **Micro-influencer** (10k-50k seguidores) | Rakeback + R$ 2.000/mês                 | Baixo                    | 5-10x          |
| 2  | **Mid-tier** (50k-200k)                 | Rakeback + R$ 10.000/mês                | Médio                    | 3-7x           |
| 3  | **Macro** (200k-1M)                     | Rakeback + R$ 50.000/mês                | Alto                     | 2-5x           |
| 4  | **Mega** (1M+)                          | Patrocínio + R$ 200.000/evento          | Muito alto               | 1-3x           |

**Critérios para parceria:**
- Audiência majoritariamente brasileira
- Conteúdo de poker (não apenas cassino genérico)
- Engajamento real (comentários, não apenas views)
- Alinhamento com valores da marca (jogo responsável)

## 👑 8.4 PROGRAMA VIP E RAKEBACK — Retenção de Jogadores de Poker

### Tiers do programa VIP

| Tier           | Rake/mês            | Rakeback   | Benefícios                                          |
|----------------|---------------------|------------|-----------------------------------------------------|
| **Bronze**     | R$ 0-500            | 5%         | Suporte padrão                                      |
| **Prata**      | R$ 500-2.000        | 10%        | Suporte prioritário, freerolls                      |
| **Ouro**       | R$ 2.000-5.000      | 15%        | Torneios exclusivos, manager dedicado               |
| **Platina**    | R$ 5.000-15.000     | 20%        | Eventos presenciais, bônus mensal                   |
| **Diamante**   | R$ 15.000-50.000    | 25%        | Tudo acima + viagens, patrocínio                    |
| **Black**      | R$ 50.000+          | 30%        | Tudo acima + condições customizadas                 |

### Loss Deflator como diferencial de retenção

O cashback é determinado pela **equity** (probabilidade de vencer) do perdedor no momento do all-in, não pelo volume de perdas.

| Tier  | Equity do Perdedor | Cashback   | Descrição                              |
|-------|--------------------|------------|----------------------------------------|
| **0** | 56,0% – 65,9%      | 7%         | Favorito leve, bad beat leve           |
| **1** | 66,0% – 75,9%      | 15%        | Favorito moderado                      |
| **2** | 76,0% – 85,9%      | 25%        | Favorito forte                         |
| **3** | ≥ 86,0%            | 35%        | Favorito esmagador, bad beat extremo   |
| —     | < 56,0%            | 0%         | Não elegível                           |

> **Regra normativa:** a equity é congelada no instante em que o all-in é pago; a fase não define o tier. Primeiro o rake é retirado do main pot e dos side pots, depois o cashback é calculado somente sobre os potes líquidos elegíveis. O cálculo heads-up é determinístico, com enumeração quando viável e Monte Carlo determinístico nos espaços maiores.

## 📊 8.5 MÉTRICAS DE MARKETING — Performance da Plataforma de Poker

| Métrica                          | Definição                              | Meta F2       | Meta F4       | Meta F6        |
|----------------------------------|----------------------------------------|---------------|---------------|----------------|
| **Visitantes únicos/mês**        | Tráfego do site                        | 10.000        | 500.000       | 5.000.000      |
| **Taxa de conversão**            | Visitante → Registro                   | > 10%         | > 12%         | > 15%          |
| **CAC blended**                  | Custo médio de aquisição               | R$ 30         | R$ 50         | R$ 40          |
| **ROAS** (Return on Ad Spend)    | Receita ÷ gasto em ads                 | > 3x          | > 5x          | > 8x           |
| **MRR** (Monthly Recurring Revenue) | Receita recorrente/mês              | R$ 5.000      | R$ 500.000    | R$ 5M          |
| **NPS**                          | Net Promoter Score                     | > 40          | > 50          | > 60           |
| **Brand awareness**              | Pesquisa espontânea                    | 5%            | 20%           | 50%            |

---

# 🎯 9. METAS E OKRs — Objectives and Key Results da Plataforma de Poker

> **Princípio:** "O que não é medido não é gerenciado." — Peter Drucker
>
> OKRs (Objectives and Key Results) são uma metodologia de definição de metas
> que alinha toda a equipe em torno de objetivos ambiciosos e resultados
> mensuráveis. Cada Objective é qualitativo (o que queremos alcançar) e cada
> Key Result é quantitativo (como sabemos que alcançamos).

## 🏗️ 9.1 ESTRUTURA DE OKRs — Framework da Plataforma de Poker

```
Objective (O):  Qualitativo, inspirador, ambicioso
    └── Key Result 1 (KR1): Quantitativo, mensurável, com prazo
    └── Key Result 2 (KR2): Quantitativo, mensurável, com prazo
    └── Key Result 3 (KR3): Quantitativo, mensurável, com prazo
```

### Regras dos OKRs

1. **3-5 Objectives por ciclo** (trimestral)
2. **3-5 Key Results por Objective**
3. **Key Results devem ser mensuráveis** (números, não adjetivos)
4. **70% de alcance = sucesso** (OKRs são ambiciosos por design)
5. **Transparência total** — todos veem os OKRs de todos
6. **Revisão semanal** de progresso
7. **Não usar OKRs para avaliação de performance** (separar metas de bônus)

## 🚀 9.2 OKRs TRIMESTRAIS — FASE 2 (MVP do Poker)

### O1: Lançar MVP de poker online funcional e seguro

| KR     | Descrição                                      | Meta        | Status          |
|--------|-----------------------------------------------|-------------|-----------------|
| KR1.1  | Cash game Texas Hold'em operacional (9 jogadores) | 100%     | 🔄              |
| KR1.2  | Torneio MTT básico operacional                | 100%        | ⏳              |
| KR1.3  | Auth + MFA + KYC implementados                | 100%        | ⏳              |
| KR1.4  | Depósito/saque via PIX funcionando            | 100%        | ⏳              |
| KR1.5  | Antifraude ativo (4 módulos)                  | 100%        | 🔄              |

### O2: Garantir qualidade e segurança do software

| KR     | Descrição                                              | Meta                                      | Status                    |
|--------|-------------------------------------------------------|-------------------------------------------|---------------------------|
| KR2.1  | Cobertura de testes (diferenciada por criticidade)    | ≥ 98% críticos / ≥ 95% API / ≥ 90% frontend | 🔄 (atual: ~60%)          |
| KR2.2  | Zero vulnerabilidades críticas (cargo audit)          | 0                                         | ⏳                        |
| KR2.3  | CI/CD em todos os PRs                                 | 100%                                      | ⏳                        |
| KR2.4  | Pen test interno concluído (Seção 4)                  | 100%                                      | ⏳                        |
| KR2.5  | Latência WebSocket < 100ms (1k jogadores)             | < 100ms                                   | ⏳                        |

### O3: Construir comunidade inicial de jogadores

| KR     | Descrição                          | Meta        | Status   |
|--------|-----------------------------------|-------------|----------|
| KR3.1  | Jogadores registrados             | 500         | ⏳       |
| KR3.2  | Jogadores ativos/dia (DAU)        | 50          | ⏳       |
| KR3.3  | Mãos jogadas/dia                  | 5.000       | ⏳       |
| KR3.4  | NPS inicial                       | > 40        | ⏳       |
| KR3.5  | Freerolls realizados              | 4 (1/semana)| ⏳       |

## 📈 9.3 OKRs TRIMESTRAIS — FASE 3 (Tração do Poker)

### O4: Atingir product-market fit

| KR     | Descrição                          | Meta        |
|--------|-----------------------------------|-------------|
| KR4.1  | Retenção W4 (semana 4)            | > 25%       |
| KR4.2  | NPS                               | > 50        |
| KR4.3  | DAU/MAU (stickiness)              | > 20%       |
| KR4.4  | Churn mensal                      | < 15%       |
| KR4.5  | Entrevistas com usuários          | 50          |

### O5: Escalar aquisição de jogadores

| KR     | Descrição                          | Meta        |
|--------|-----------------------------------|-------------|
| KR5.1  | Novos jogadores/mês               | 1.000       |
| KR5.2  | CAC blended                       | < R$ 50     |
| KR5.3  | LTV/CAC                           | > 10x       |
| KR5.4  | ROAS (Return on Ad Spend)         | > 3x        |
| KR5.5  | Afiliados ativos                  | 20          |

### O6: Aumentar receita de rake

| KR     | Descrição                                  | Meta            |
|--------|-------------------------------------------|-----------------|
| KR6.1  | MRR (Monthly Recurring Revenue)           | R$ 50.000       |
| KR6.2  | Rake/jogador/mês (ARPU)                   | R$ 80           |
| KR6.3  | Mesas ativas simultâneas                  | 100             |
| KR6.4  | Torneios GTD realizados                   | 12 (3/mês)      |
| KR6.5  | Payback period                            | < 20 dias       |

## 🌱 9.4 OKRs TRIMESTRAIS — FASE 4 (Crescimento do Poker)

### O7: Escalar para 10.000 jogadores ativos

| KR     | Descrição                          | Meta                    |
|--------|-----------------------------------|-------------------------|
| KR7.1  | MAU (Monthly Active Users)        | 10.000                  |
| KR7.2  | DAU                               | 2.000                   |
| KR7.3  | Mesas ativas simultâneas          | 500                     |
| KR7.4  | Novos jogadores/mês               | 5.000                   |
| KR7.5  | Países ativos                     | 3 (BR, AR, MX)          |

### O8: Otimizar unit economics

| KR     | Descrição                          | Meta        |
|--------|-----------------------------------|-------------|
| KR8.1  | LTV                               | > R$ 600    |
| KR8.2  | CAC                               | < R$ 50     |
| KR8.3  | LTV/CAC                           | > 12x       |
| KR8.4  | Margem                            | > 70%       |
| KR8.5  | Churn                             | < 10%       |

### O9: Fortalecer segurança e conformidade

| KR     | Descrição                          | Meta                    |
|--------|-----------------------------------|-------------------------|
| KR9.1  | Pen test externo concluído        | 1                       |
| KR9.2  | Vulnerabilidades críticas         | 0                       |
| KR9.3  | Bug bounty program lançado        | 100%                    |
| KR9.4  | Licença regulamentada obtida      | 1 (Curação ou Malta)    |
| KR9.5  | Incidentes de segurança           | 0                       |

## 🛠️ 9.5 OKRs DE ENGENHARIA (Contínuos) — Motor de Poker

### O10: Manter excelência técnica

| KR      | Descrição                                              | Meta                                      | Frequência   |
|---------|-------------------------------------------------------|-------------------------------------------|--------------|
| KR10.1  | Cobertura de testes (diferenciada por criticidade)    | ≥ 98% críticos / ≥ 95% API / ≥ 90% frontend | Contínuo     |
| KR10.2  | Tempo de build (CI)                                   | < 5 min                                   | Contínuo     |
| KR10.3  | Tempo de deploy                                       | < 10 min                                  | Contínuo     |
| KR10.4  | MTTR (Mean Time To Recovery)                          | < 30 min                                  | Contínuo     |
| KR10.5  | Uptime                                                | > 99.9%                                   | Mensal       |
| KR10.6  | Débito técnico (issues abertas)                       | < 50                                      | Mensal       |
| KR10.7  | Dependências desatualizadas                           | 0 críticas                                | Semanal      |

### O11: Excelência em antifraude

| KR      | Descrição                          | Meta                    | Frequência   |
|---------|-----------------------------------|-------------------------|--------------|
| KR11.1  | Falsos positivos de antifraude    | < 5%                    | Mensal       |
| KR11.2  | Fraude não detectada              | < 0.1% da receita       | Mensal       |
| KR11.3  | Tempo de detecção de colusão      | < 24h                   | Contínuo     |
| KR11.4  | Bots banidos/mês                  | > 100 (quando escala)   | Mensal       |
| KR11.5  | Contas multi-account bloqueadas   | > 50/mês                | Mensal       |

## 🔄 9.6 RITMO DE REVISÃO DE OKRs — Cadência do Poker

| Frequência        | Atividade                               | Participantes                    |
|-------------------|-----------------------------------------|----------------------------------|
| **Semanal**       | Check-in de progresso (15 min)          | Toda a equipe                    |
| **Mensal**        | Revisão de OKRs mensais (1h)            | Liderança                        |
| **Trimestral**    | Definição de novos OKRs (2h)            | Liderança + representantes       |
| **Anual**         | Revisão estratégica (4h)                | Liderança + investidores         |

---

# 🤖 10. PRÁTICAS DE IA — Inteligência Artificial no Antifraude e Motor de Poker

> **Princípio:** "IA não substitui engenheiros, **amplifica** engenheiros."
>
> No contexto de uma plataforma de poker, a IA tem **4 aplicações principais**:
> (1) Antifraude, (2) AI Coach para jogadores, (3) Geração de código e testes,
> (4) Análise de dados de jogo. Cada uma com responsabilidades e limites claros.

## 🛡️ 10.1 IA PARA ANTIFRAUDE (Em Tempo Real) — Motor de Poker

### Arquitetura de IA antifraude

```
┌─────────────────────────────────────────────────────────────────┐
│                    PIPELINE DE IA ANTIFRAUDE                     │
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  Event   │───→│ Feature  │───→│   ML     │───→│  Action  │  │
│  │  Stream  │    │ Enginner │    │  Models  │    │  Engine  │  │
│  │ (Kafka)  │    │ (Rust)   │    │ (Rust)   │    │ (Rust)   │  │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘  │
│       │               │               │               │          │
│       ▼               ▼               ▼               ▼          │
│  Hand History   Features:        Modelos:        Ações:         │
│  - ações        - VPIP, PFR      - Colusão       - Alertar       │
│  - timing       - WTSD, AF       - Chip Dump     - Congelar      │
│  - chat         - Timing         - Bot           - Banir         │
│  - depósitos    - Geolocation   - Multi-account  - Revisar       │
└─────────────────────────────────────────────────────────────────┘
```

### Modelos de IA por tipo de fraude

| #   | Tipo de Fraude          | Algoritmo                              | Features                                                              | Ação                                    |
|-----|-------------------------|----------------------------------------|-----------------------------------------------------------------------|-----------------------------------------|
| 1   | **Colusão**             | Anomaly detection (Isolation Forest)   | Win rate conjunto, timing correlacionado, mesmo IP/VPN                | Alertar + revisar hand history          |
| 2   | **Chip Dumping**        | Graph analysis + clustering            | Transferências unidirecionais, novos contas, padrão de perda intencional | Congelar contas + investigar            |
| 3   | **Bot detection**       | Behavioral biometrics                  | Timing entre ações (muito consistente = bot), desvio padrão de timing, padrões de clique | CAPTCHA + verificação humana            |
| 4   | **Multi-accounting**    | Device fingerprinting + graph          | Mesmo dispositivo, mesmo IP, correlação de jogo entre contas          | Banir contas secundárias                |
| 5   | **Lavagem de dinheiro** | Transaction pattern analysis           | Depósito → jogo mínimo → saque, volume alto com jogo baixo            | Reportar (AML) + congelar               |

### Métricas de qualidade dos modelos de IA

| Métrica            | Definição                                                       | Meta                    |
|--------------------|-----------------------------------------------------------------|-------------------------|
| **Precision**      | TP ÷ (TP + FP) — % de alertas que são fraude real               | > 90%                   |
| **Recall**         | TP ÷ (TP + FN) — % de fraudes detectadas                        | > 95%                   |
| **F1-Score**       | Média harmônica de Precision e Recall                           | > 0.92                  |
| **Falso positivo** | % de jogadores legítimos marcados como fraude                   | < 5%                    |
| **Latência**       | Tempo de inferência por evento                                  | < 50ms                  |
| **Drift**          | Mudança de distribuição dos dados ao longo do tempo             | Monitorar mensalmente   |

## 🎓 10.2 AI COACH PARA JOGADORES — Poker Mentor

### Funcionalidades do AI Coach

| #   | Funcionalidade                  | Descrição                                              | Tecnologia                  |
|-----|---------------------------------|-------------------------------------------------------|-----------------------------|
| 1   | **Análise de mão**              | Avalia cada mão jogada e sugere melhor jogada         | GTO solver simplificado     |
| 2   | **Identificação de leaks**      | Detecta padrões de erro do jogador                    | Análise estatística         |
| 3   | **Recomendação de stakes**      | Sugere nível de aposta adequado ao bankroll           | Bankroll management         |
| 4   | **Treino de pré-flop**          | Exercícios de decisão pré-flop                        | Flashcards adaptativos      |
| 5   | **Análise de oponentes**        | Stats dos oponentes (VPIP, PFR, AF)                   | Tracking + ML               |
| 6   | **Replay com GTO**              | Compara jogada real com jogada GTO ótima              | GTO baseline                |

### Limites éticos do AI Coach

1. **Não joga pelo usuário** — apenas sugere, não executa
2. **Não usa dados de oponentes em tempo real** — apenas stats históricas
3. **Transparência** — jogador sabe que está usando IA
4. **Não cria vantagem injusta** — disponível para todos os jogadores
5. **Foco em educação** — não em automação de jogo

## 🤖 10.3 IA PARA DESENVOLVIMENTO DE SOFTWARE — Motor de Poker

### Uso responsável de IA em engenharia

| #   | Uso                          | Ferramenta                    | Boas Práticas                                                   |
|-----|------------------------------|-------------------------------|-----------------------------------------------------------------|
| 1   | **Geração de código**        | GitHub Copilot, Claude        | Revisar TODO código gerado, nunca aceitar cegamente             |
| 2   | **Geração de testes**        | Copilot, ChatGPT              | Usar como ponto de partida, expandir manualmente                |
| 3   | **Code review**              | Copilot, CodeRabbit           | IA sugere, humano decide                                        |
| 4   | **Documentação**             | Copilot, ChatGPT              | Revisar factualidade, manter voz consistente                    |
| 5   | **Debugging**                | Copilot, Claude               | IA sugere hipóteses, humano investiga                           |
| 6   | **Refactoring**              | Copilot                       | Validar com testes após refatoração                             |

### Regras de ouro para IA em engenharia

1. **IA é assistente, não substituto** — humano é responsável final
2. **Todo código gerado por IA deve ser revisado por humano**
3. **Testes gerados por IA devem ser validados** — executar e verificar
4. **Nunca commitar código sem entender** — se não entende, não merge
5. **Documentar uso de IA** — qual ferramenta, qual prompt, qual output
6. **Segurança primeiro** — nunca enviar dados sensíveis para IA externa
7. **Vieses** — IA pode gerar código com vieses; revisar criticamente

## 📊 10.4 IA PARA ANÁLISE DE DADOS DE JOGO — Poker Analytics

### Dashboards de IA

| Dashboard                      | Métricas                                                       | Frequência     | Ação                        |
|--------------------------------|----------------------------------------------------------------|----------------|-----------------------------|
| **Saúde da plataforma**        | DAU, MAU, rake, churn, mesas ativas                            | Tempo real     | Alertas automáticos         |
| **Detecção de fraude**         | Alertas ativos, falsos positivos, contas banidas               | Tempo real     | Revisão humana              |
| **Comportamento de jogo**      | Hands/hora, pote médio, all-in rate, fold rate                 | Diário         | Otimizar UX                 |
| **Economia do jogador**        | Depósitos, saques, rake gerado, LTV                            | Diário         | Segmentação                 |
| **Risco de churn**             | Probabilidade de churn por jogador                             | Semanal        | Campanhas de retenção       |
| **Anomalias**                  | Picos de tráfego, quedas de rake, comportamentos anômalos      | Tempo real     | Investigação                |

---

# 💥 11. CHAOS ENGINEERING — Engenharia do Caos na Plataforma de Poker

> **Princípio:** "Caos não é destruição, é **descoberta**. Quebramos em
> produção controlada para não quebrar em produção inesperada."
>
> Chaos Engineering é a disciplina de experimentação em sistemas distribuídos
> para construir confiança na capacidade do sistema de resistir a condições
> turbulentas e extremas. Para uma plataforma de poker, onde **dinheiro real**
> está em jogo, a resiliência não é opcional — é **obrigatória**.

## 💥 11.1 PRINCÍPIOS DO CHAOS ENGINEERING — Resiliência do Poker

1. **Definir estado estável** — métricas que indicam saúde do sistema
2. **Variar hipóteses** — "o sistema sobrevive a X?"
3. **Experimentar em produção** — simular falhas reais
4. **Automatizar experimentos** — executar continuamente
5. **Minimizar raio de explosão** — isolar impacto
6. **Começar pequeno** — expandir gradualmente

## 🌪️ 11.2 CENÁRIOS DE CAOS PARA PLATAFORMA DE POKER

### 💥 10.2.1 Falhas de Infraestrutura do Motor de Poker

| #   | Experimento                               | Hipótese                                                       | Métricas                                      | Raio de explosão   |
|-----|-------------------------------------------|----------------------------------------------------------------|-----------------------------------------------|--------------------|
| 1   | **Matar container do motor de poker**     | Sistema reinicia em < 30s, sem perda de mãos em andamento      | Uptime, MTTR, mãos perdidas                   | 1 mesa             |
| 2   | **Matar container do PostgreSQL**         | Failover para réplica em < 60s, sem perda de dados             | RPO, RTO, transações perdidas                 | 0 (read replica)   |
| 3   | **Matar container do Redis**              | Cache reconstrói em < 10s, latência aumenta mas sistema funciona | Latência, cache hit rate                       | 1 sala             |
| 4   | **Matar container do Kafka**              | Mensagens enfileiram localmente, processam quando Kafka volta  | Mensagens perdidas, latência                  | 1 sala             |
| 5   | **Latência de rede artificial (500ms)**   | Jogadores com lag tolerável, timeout ajusta                    | Timeout rate, dropout rate                    | 1 sala             |
| 6   | **Perda de pacotes (10%)**                | Sistema detecta e reconecta jogadores                          | Reconexões, mãos pausadas                     | 1 sala             |
| 7   | **Disco cheio**                           | Sistema alerta e limpa logs antigos                            | Alertas, espaço em disco                      | 1 container        |
| 8   | **CPU 100%**                              | Sistema degrada graciosamente, prioriza jogo em andamento      | Latência, timeouts                            | 1 container        |

### 🐛 10.2.2 Falhas de Aplicação nas Mesas de Poker

| #   | Experimento                               | Hipótese                                                       | Métricas                          | Raio de explosão   |
|-----|-------------------------------------------|----------------------------------------------------------------|-----------------------------------|--------------------|
| 1   | **Panic em thread do motor**              | Thread reinicia, mesa afetada pausa, outras continuam          | Mesas afetadas, mesas ativas      | 1 mesa             |
| 2   | **Memory leak simulado**                  | Sistema detecta uso anormal de memória e reinicia container    | Memória, uptime                   | 1 container        |
| 3   | **Timeout em API de pagamento**           | Sistema enfileira saque, processa quando API volta             | Saques pendentes, timeout         | 1 sala             |
| 4   | **Erro em RNG**                           | Sistema detecta RNG inválido, pausa mesa, usa RNG backup       | Mesas pausadas, RNG falhas        | 1 mesa             |
| 5   | **Inconsistência de saldo**               | Sistema detecta e reconcilia com PostgreSQL                    | Discrepâncias, reconciliação      | 1 jogador          |

### 🔐 10.2.3 Falhas de Segurança no Antifraude do Poker

| #   | Experimento                               | Hipótese                                                       | Métricas                          | Raio de explosão   |
|-----|-------------------------------------------|----------------------------------------------------------------|-----------------------------------|--------------------|
| 1   | **Ataque DDoS simulado**                  | WAF + rate limiting absorvem tráfego                           | Latência, requests bloqueados     | 0 (mitigado)       |
| 2   | **Tentativa de SQL injection**            | Input validation bloqueia                                      | Tentativas bloqueadas             | 0                  |
| 3   | **Token JWT expirado em massa**           | Sistema renova tokens, jogadores reconectam                    | Reconexões, timeouts              | 1 sala             |
| 4   | **Vazamento de chave simulado**           | Sistema detecta uso anormal, revoga chave                      | Alertas, chaves revogadas         | 0                  |

## 📅 11.3 GAME DAYS — Dias de Caos no Poker

### Definição
Game Day é um exercício programado onde a equipe **intencionalmente** causa
falhas no sistema para testar resiliência, procedimentos de resposta a
incidentes e comunicação da equipe.

### Estrutura de um Game Day

| Fase                | Duração        | Atividade                                                       |
|---------------------|----------------|-----------------------------------------------------------------|
| **Planejamento**    | 1 semana       | Definir experimentos, métricas, raio de explosão                |
| **Execução**        | 4-8 horas      | Executar experimentos, monitorar, documentar                    |
| **Análise**         | 2 horas        | Revisar o que quebrou, o que funcionou                          |
| **Melhoria**        | 1-2 semanas    | Corrigir falhas encontradas, adicionar testes                   |
| **Documentação**    | 1 hora         | Escrever postmortem, atualizar runbook                          |

### Cronograma de Game Days

| Frequência        | Foco                                   | Participantes                         |
|-------------------|----------------------------------------|---------------------------------------|
| **Mensal**        | Falha de infraestrutura                | Engenharia + DevOps                   |
| **Trimestral**    | Falha de aplicação + segurança         | Engenharia + Segurança                |
| **Semestral**     | Cenário completo (multi-falha)         | Toda a empresa                        |
| **Anual**         | Disaster recovery completo             | Toda a empresa + stakeholders         |

## 🛠️ 11.4 FERRAMENTAS DE CHAOS ENGINEERING — Poker Stack

| #   | Ferramenta                          | Função                                    | Quando usar       |
|-----|-------------------------------------|-------------------------------------------|-------------------|
| 1   | **Chaos Mesh** (K8s)                | Injeção de falhas em containers           | F4 (K8s)          |
| 2   | **Litmus** (K8s)                    | Experimentos de chaos em K8s              | F4                |
| 3   | **Pumba** (Docker)                  | Caos em containers Docker                 | F2-F3             |
| 4   | **Toxiproxy**                       | Simular falhas de rede                    | F2                |
| 5   | **Chaos Monkey** (Netflix)          | Matar instâncias aleatórias               | F4                |
| 6   | **Scripts customizados (Rust)**     | Caos específico de poker                  | F2                |

## 📋 11.5 POSTMORTEMS — Cultura Blameless do Poker

### Template de postmortem

```markdown
# Postmortem: [Título do Incidente]

**Data:** YYYY-MM-DD
**Duração:** X horas
**Impacto:** X jogadores afetados, R$ Y de rake perdido
**Severidade:** SEV-1/2/3
**Status:** Resolvido

## Resumo
[1-2 parágrafos descrevendo o incidente]

## Linha do tempo
- HH:MM — Alerta disparado
- HH:MM — Engenheiro on-call notificado
- HH:MM — Investigação começou
- HH:MM — Causa raiz identificada
- HH:MM — Mitigação aplicada
- HH:MM — Sistema restaurado

## Causa raiz
[O que causou o incidente — focar no SISTEMA, não na PESSOA]

## O que funcionou
[O que ajudou a detectar/mitigar rapidamente]

## O que não funcionou
[O que falhou ou atrasou a resposta]

## Lições aprendidas
1. [Lição 1]
2. [Lição 2]

## Ações corretivas
| #   | Ação        | Owner     | Prazo     | Status     |
|-----|-------------|-----------|-----------|------------|
| 1   | [Ação]      | [Nome]    | [Data]    | [Status]   |

## Apêndice
[Gráficos, logs, evidências]
```

### Princípios blameless

1. **Focar no sistema, não na pessoa** — "O que no sistema permitiu esse erro?"
2. **Assumir boa intenção** — ninguém quer causar incidente
3. **Documentar tudo** — para aprender, não para punir
4. **Ações corretivas** — sempre gerar ações, não apenas discussão
5. **Cultura de transparência** — postmortems são públicos internamente

---

# 📅 8-BIS. GESTÃO DE TEMPO, TAREFAS E DASHBOARD EXECUTIVO — Sistema Nervoso da Plataforma de Poker

> **Princípio:** "OKRs sem execução são sonhos. Execução sem visibilidade é
> caos. Visibilidade sem dashboard é cegueira."
>
> OKRs definem **o quê** alcançar (estratégia). A gestão de tempo e tarefas
> define **como** alcançar (execução). O dashboard é o **sistema nervoso
> central** que conecta os dois, dando visibilidade em tempo real do progresso,
> gargalos e saúde do projeto. Para uma plataforma de poker — onde dinheiro
> real está em jogo e a concorrência é agressiva — **executar mais rápido e
> melhor que os concorrentes é vantagem competitiva**.

## ⏱️ 8.B.1 GESTÃO DE TEMPO — Técnicas Modernas e Comprovadas no Poker

### ⏱️ 8.B.1.1 Time Blocking (Blocos de Tempo) — Sessões de Desenvolvimento do Motor de Poker

> "Dê a cada minuto da sua vida um trabalho para fazer." — Adaptado de
> Cal Newport, autor de *Deep Work*

| Conceito            | Descrição                                                                               | Aplicação no Projeto                                                                                       |
|---------------------|-----------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
| **Time Blocking**   | Dividir o dia em blocos de tempo dedicados a tarefas específicas                        | Bloco 9h-11h: Deep work (motor Rust); 11h-12h: Code review; 14h-16h: Feature; 16h-17h: Reuniões            |
| **Deep Work**       | Trabalho focado, sem distrações, em tarefas cognitivamente exigentes                     | Implementar lógica de side pots, antifraude, algoritmos de poker — 2-4h/dia                                 |
| **Shallow Work**    | Tarefas administrativas, emails, mensagens, reuniões curtas                              | Responder issues, atualizar DASHBOARD.md, revisar PRs simples — 1-2h/dia                                      |
| **Batching**        | Agrupar tarefas similares para reduzir troca de contexto                                 | Responder todos os emails/Slack em 2 blocos (manhã e tarde), não ao longo do dia                           |

### 🍅 8.B.1.2 Pomodoro Technique (Técnica do Pomodoro) — Sprints de Codificação do Motor

| Ciclo            | Duração        | Atividade                                          |
|------------------|----------------|----------------------------------------------------|
| **Pomodoro**     | 25 min         | Trabalho focado em UMA tarefa                      |
| **Pausa curta**  | 5 min          | Descanso, alongamento, água                        |
| **Pausa longa**  | 15-30 min      | Após 4 pomodoros, descanso maior                   |

**Aplicação no projeto:**
- 1 Pomodoro = 1 tarefa do `DASHBOARD.md` ou 1 bug do `DEVELOPMENT_LOG.md`
- 4 Pomodoros = 1 feature completa (ex: implementar regra de rake)
- Pausas = revisar hand history, estudar poker (aprendizado contínuo)

### 🎯 8.B.1.3 Eisenhower Matrix (Matriz de Urgência vs Importância) — Priorização de Features de Poker

```
                    URGENTE              NÃO URGENTE
              ┌──────────────────┬──────────────────┐
              │                  │                  │
    IMPORTANTE│   FAZER AGORA    │   PLANEJAR       │
              │   (Quadrante I)  │   (Quadrante II) │
              │                  │                  │
              │  • Bug de saldo  │  • Implementar   │
              │  • Vulnerabilidade│     Loss Deflator│
              │    crítica        │  • Escrever      │
              │  • Servidor fora  │     testes       │
              │                  │  • Documentação  │
              ├──────────────────┼──────────────────┤
              │                  │                  │
  NÃO IMPORT.│   DELEGAR        │   ELIMINAR       │
              │   (Quadrante III)│   (Quadrante IV) │
              │                  │                  │
              │  • Reuniões      │  • Scroll social │
              │    desnecessárias│  • Notificações  │
              │  • Emails        │     irrelevantes │
              │    de baixa prio │  • Tarefas que   │
              │                  │     não agregam  │
              └──────────────────┴──────────────────┘
```

**Regra de ouro:** Gastar **80% do tempo no Quadrante II** (importante mas
não urgente) — é onde está o trabalho que previne crises e gera valor real.

### ✅ 8.B.1.4 Getting Things Done (GTD) — David Allen — Gestão de Tarefas do Projeto de Poker

| Fase                | Descrição                                  | Aplicação                                                              |
|---------------------|--------------------------------------------|------------------------------------------------------------------------|
| **1. Capturar**     | Registrar tudo que chama atenção           | Toda ideia, bug, feature → GitHub Issues ou arquivo `inbox.md`         |
| **2. Clarificar**   | Decidir o que é cada item                  | É ação? Qual a próxima ação? É projeto? É referência?                  |
| **3. Organizar**    | Colocar em lugares certos                  | Projetos → DASHBOARD.md; Ações → Kanban; Referência → docs/               |
| **4. Refletir**     | Revisar semanalmente                       | Revisão semanal: DASHBOARD.md, Kanban, OKRs, inbox                        |
| **5. Engajar**      | Executar com confiança                     | Escolher tarefa por contexto, energia, prioridade                      |

### 📊 8.B.1.5 Lei de Parkinson e Princípio de Pareto (80/20) no Desenvolvimento de Poker

| Princípio                          | Enunciado                                                                                       | Aplicação                                                                                    |
|------------------------------------|-------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| **Lei de Parkinson**               | "O trabalho se expande para preencher o tempo disponível"                                       | Definir **timeboxes** rígidos: "implementar rake em 2h, não 2 dias"                          |
| **Princípio de Pareto (80/20)**    | "80% dos resultados vêm de 20% dos esforços"                                                    | Focar nas 20% de features que geram 80% do valor: motor de poker, antifraude, pagamentos     |
| **Lei de Brooks**                  | "Adicionar pessoas a um projeto atrasado o atrasa mais"                                         | Não escalar equipe cegamente; automatizar e otimizar primeiro                                |
| **Lei de Hofstadter**              | "Tudo leva mais tempo do que o esperado, mesmo considerando a lei de Hofstadter"                | Adicionar **buffer de 30%** a todas as estimativas                                           |

## 📋 8.B.2 GESTÃO DE TAREFAS — Metodologias Modernas do Poker

### 📋 8.B.2.1 Kanban — Fluxo Visual Contínuo de Tarefas do Motor de Poker

```
┌─────────┬──────────┬──────────┬──────────┬──────────┐
│ BACKLOG │ TO DO    │ DOING    │ REVIEW   │ DONE     │
│         │          │ (WIP ≤3) │          │          │
├─────────┼──────────┼──────────┼──────────┼──────────┤
│ • Idea  │ • Task 1 │ • Task A │ • Task X │ • Task Z │
│ • Bug   │ • Task 2 │ • Task B │ • Task Y │ • Task W │
│ • Feat  │ • Task 3 │          │          │ • Task V │
│  47     │  12      │  2       │  3       │  89      │
└─────────┴──────────┴──────────┴──────────┴──────────┘
```

**Regras do Kanban:**

| Regra                            | Descrição                                      | Por quê                                      |
|----------------------------------|-----------------------------------------------|----------------------------------------------|
| **Visualizar o trabalho**        | Toda tarefa é um card visível                 | Reduz trabalho invisível                     |
| **Limitar WIP** (Work in Progress) | Máximo 3 tarefas em DOING                     | Reduz troca de contexto, aumenta foco        |
| **Gerenciar fluxo**              | Medir lead time e cycle time                  | Identificar gargalos                         |
| **Políticas explícitas**         | Definir critérios de "Done"                   | Qualidade consistente                        |
| **Feedback loops**               | Revisão semanal do fluxo                      | Melhoria contínua                            |
| **Melhorar colaborativamente**   | Equipe sugere melhorias                       | Evolução do processo                         |

### 🏃 8.B.2.2 Scrum — Sprints Estruturadas para Features de Poker

| Cerimônia                    | Frequência          | Duração     | Propósito                                   |
|------------------------------|---------------------|-------------|---------------------------------------------|
| **Sprint Planning**          | A cada 2 semanas    | 2h          | Definir o que fazer na sprint               |
| **Daily Standup**            | Diário              | 15 min      | O que fiz, o que farei, impedimentos        |
| **Sprint Review**            | A cada 2 semanas    | 1h          | Demonstrar o que foi feito                  |
| **Sprint Retrospective**     | A cada 2 semanas    | 1h          | O que funcionou, o que melhorar             |

**Formato do Daily Standup (assíncrono quando possível):**

```markdown
## Daily — YYYY-MM-DD

### ✅ Fez ontem
- Implementei validação de side pots (3 testes passando)
- Corrigi bug #47 (timing de all-in)

### 🔄 Vai fazer hoje
- Implementar rake para torneios (motor_tests.rs)
- Code review do PR #12

### 🚧 Impedimentos
- Aguardando definição de regra de rake para Omaha
```

### ⚖️ 8.B.2.3 Escolha: Kanban vs Scrum para o Projeto de Poker Online

| Critério                      | Kanban                    | Scrum                         |
|-------------------------------|---------------------------|-------------------------------|
| **Time solo (atual)**         | ✅ Melhor                 | ❌ Overhead de cerimônias     |
| **Time 2-5 pessoas**          | ✅ Bom                    | ✅ Bom                        |
| **Time 6+ pessoas**           | ⚠️ Pode caos              | ✅ Melhor (estrutura)         |
| **Mudança frequente**         | ✅ Flexível               | ❌ Sprint rígida              |
| **Previsibilidade**           | ⚠️ Menor                  | ✅ Maior                      |

> **Recomendação para o projeto atual (solo):** **Kanban** com elementos de
> Scrum (sprints de 2 semanas para planejamento, daily assíncrono, retrospectiva
> mensal). Quando o time crescer para 3+ pessoas, transitar para Scrum.

## 📊 8.B.3 DASHBOARD EXECUTIVO — Sistema Nervoso Central do Poker

> "O que é medido é gerenciado. O que é exibido é priorizado."
>
> Um dashboard moderno é **não negociável** para uma plataforma de poker.
> Ele deve responder 3 perguntas em **menos de 5 segundos**:
> 1. **Estamos saudáveis?** (saúde do sistema, uptime, latência)
> 2. **Estamos crescendo?** (DAU, MAU, rake, receita)
> 3. **Estamos seguros?** (fraude, alertas, vulnerabilidades)

### 🏗️ 8.B.3.1 Arquitetura do Dashboard Executivo da Plataforma de Poker

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DASHBOARD EXECUTIVO                            │
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│  │   SAÚDE DO       │  │   NEGÓCIO       │  │   SEGURANÇA      │    │
│  │   SISTEMA        │  │   & CRESCIMENTO │  │   & FRAUDE       │    │
│  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤    │
│  │ • Uptime         │  │ • DAU/MAU       │  │ • Alertas ativos │    │
│  │ • Latência       │  │ • Rake/mês      │  │ • Contas banidas │    │
│  │ • Erros/min      │  │ • MRR           │  │ • Falsos posit.  │    │
│  │ • CPU/Memória    │  │ • Novos jogadores│  │ • Pen test status│    │
│  │ • Mesas ativas   │  │ • Churn rate    │  │ • Vulnerabilidades│   │
│  │ • Mãos/seg       │  │ • LTV/CAC       │  │ • Bug bounty     │    │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘    │
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│  │   OKRs          │  │   SPRINT/       │  │   FINANCEIRO    │    │
│  │   PROGRESSO     │  │   KANBAN        │  │   & BURN RATE   │    │
│  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤    │
│  │ • O1: 70%       │  │ • Backlog: 47   │  │ • Caixa: R$ X   │    │
│  │ • O2: 45%       │  │ • To Do: 12     │  │ • Burn rate      │    │
│  │ • O3: 80%       │  │ • Doing: 2      │  │ • Runway: X meses│    │
│  │ • KR1.1: ✅     │  │ • Review: 3     │  │ • MRR            │    │
│  │ • KR1.2: 🔄     │  │ • Done: 89      │  │ • Net Burn       │    │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### 📡 8.B.3.2 Métricas em Tempo Real (Live Metrics) das Mesas de Poker

| Categoria       | Métrica                              | Fonte              | Atualização     | Alerta                        |
|-----------------|--------------------------------------|--------------------|-----------------|-------------------------------|
| **Sistema**     | Uptime                               | Prometheus         | 10s             | < 99.9%                       |
| **Sistema**     | Latência WebSocket (p99)             | tracing            | 10s             | > 200ms                       |
| **Sistema**     | Erros/min                            | logs + tracing     | 10s             | > 10/min                      |
| **Sistema**     | CPU/Memória por container            | cAdvisor           | 30s             | > 85%                         |
| **Sistema**     | Mesas ativas simultâneas             | motor              | 30s             | < 10 (F2)                     |
| **Sistema**     | Mãos/segundo                         | hand_history       | 10s             | Queda > 50%                   |
| **Negócio**     | DAU                                  | auth + sessões     | 1h              | < meta                        |
| **Negócio**     | Rake acumulado hoje                  | rake               | 1min            | < meta diária                 |
| **Negócio**     | Novos registros hoje                 | auth               | 5min            | < meta                        |
| **Negócio**     | Depósitos hoje                       | pagamentos         | 5min            | Anomalia                      |
| **Segurança**   | Alertas de fraude ativos             | antifraude         | tempo real      | > 5 críticos                  |
| **Segurança**   | Tentativas de login falhas           | auth               | 1min            | > 100/min (brute force)       |
| **Segurança**   | Contas banidas hoje                  | antifraude         | 1h              | —                             |
| **Financeiro**  | Caixa atual                          | financeiro         | 1h              | < 3 meses runway              |

### 🎯 8.B.3.3 Dashboard de OKRs (Visual) da Plataforma de Poker Online

```
┌─────────────────────────────────────────────────────────────────┐
│  OKRs — Q3 2026                              Atualizado: agora │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  O1: Lançar MVP de poker online funcional e seguro    72% ████ │
│  ├── KR1.1: Cash game operacional              100% ██████████ │
│  ├── KR1.2: Torneio MTT básico                  80% ████████░░ │
│  ├── KR1.3: Auth + MFA + KYC                    60% ██████░░░░░ │
│  ├── KR1.4: Depósito/saque PIX                  50% █████░░░░░ │
│  └── KR1.5: Antifraude ativo                    70% ███████░░░░ │
│                                                                 │
│  O2: Garantir qualidade e segurança do software      45% ██░░░ │
│  ├── KR2.1: Cobertura de testes ≥ 80%           60% ██████░░░░ │
│  ├── KR2.2: Zero vulnerabilidades críticas      80% ████████░░ │
│  ├── KR2.3: CI/CD em todos os PRs               20% ██░░░░░░░░░ │
│  ├── KR2.4: Pen test interno                    0%  ░░░░░░░░░░░ │
│  └── KR2.5: Latência WebSocket < 100ms          60% ██████░░░░ │
│                                                                 │
│  O3: Construir comunidade inicial de jogadores       30% █░░░░ │
│  ├── KR3.1: 500 jogadores registrados           40% ████░░░░░░ │
│  ├── KR3.2: 50 DAU                               20% ██░░░░░░░░ │
│  ├── KR3.3: 5.000 mãos/dia                      30% ███░░░░░░░ │
│  ├── KR3.4: NPS > 40                             0%  ░░░░░░░░░░ │
│  └── KR3.5: 4 freerolls                          25% ██░░░░░░░░ │
│                                                                 │
│  Legenda: ░ Pendente  █ Em progresso  █████ Concluído           │
└─────────────────────────────────────────────────────────────────┘
```

### 📋 8.B.3.4 Dashboard de Kanban/Sprint do Motor de Poker

```
┌─────────────────────────────────────────────────────────────────┐
│  SPRINT 12 — 2026-07-06 a 2026-07-19    Dia 3 de 14  │ 57% │   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  BACKLOG (47)    TO DO (12)    DOING (2/3)   REVIEW (3)  DONE   │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐ ┌──────────┐ ┌────┐│
│  │#48 Rake  │   │#45 Omaha │   │#42 Side  │ │#39 Auth  │ │#37 ││
│  │  Omaha   │   │  rules   │   │  pots    │ │  MFA     │ │ RNG││
│  │          │   │          │   │  edge    │ │          │ │ ✓  ││
│  ├──────────┤   ├──────────┤   └──────────┘ └──────────┘ ├────┤│
│  │#49 VIP   │   │#46 Tour  │   ┌──────────┐ ┌──────────┐ │#36 ││
│  │  tiers   │   │  engine │   │#43 Loss  │ │#40 KYC   │ │Side││
│  │          │   │  rebuy  │   │ Deflator │ │  flow    │ │ ✓  ││
│  ├──────────┤   ├──────────┤   └──────────┘ └──────────┘ ├────┤│
│  │#50 AI    │   │#47 Chat │                            │#35 ││
│  │ Coach    │   │  modera │                            │Deck││
│  └──────────┘   └──────────┘                            │ ✓  ││
│                                                          └────┘│
│  Lead time: 4.2 dias  |  Cycle time: 2.1 dias  |  WIP: 2/3     │
└─────────────────────────────────────────────────────────────────┘
```

### 💰 8.B.3.5 Dashboard Financeiro (Burn Rate & Runway) do Rake de Poker

```
┌─────────────────────────────────────────────────────────────────┐
│  FINANCEIRO — Julho 2026                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Caixa atual:     R$ 487.000     🟢                             │
│  Burn rate:       R$ 57.000/mês  (net)                          │
│  Runway:          8.5 meses     🟡                              │
│  Status:          Default Dead  (precisa de receita ou aporte)  │
│                                                                 │
│  Receita (rake):  R$  5.000/mês  ▓▓▓░░░░░░░░░░░░░░  8% da meta │
│  Meta F2:         R$ 60.000/mês                                │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  PROJEÇÃO DE CAIXA                                       │   │
│  │                                                          │   │
│  │  R$500K ┤█████                                           │   │
│  │         │     ███                                        │   │
│  │  R$400K ┤        ██                                      │   │
│  │         │          ███                                   │   │
│  │  R$300K ┤             ██                                  │   │
│  │         │               ███                              │   │
│  │  R$200K ┤                  ██  ← PONTO CRÍTICO           │   │
│  │         │                    ███                          │   │
│  │  R$100K ┤                       ██  ← DEFAULT ALIVE! 🎉  │   │
│  │         │                         ████████████████       │   │
│  │    R$0  └──────────────────────────────────────────→     │   │
│  │          M1  M2  M3  M4  M5  M6  M7  M8  M9  M10 M11 M12│   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Alertas:                                                       │
│  ⚠️ Runway < 9 meses — Planejar aporte ou acelerar receita     │
│  ⚠️ Receita de rake 92% abaixo da meta F2                      │
└─────────────────────────────────────────────────────────────────┘
```

### 🛡️ 8.B.3.6 Dashboard de Segurança e Antifraude do Poker Online

```
┌─────────────────────────────────────────────────────────────────┐
│  SEGURANÇA & ANTIFRAUDE — Tempo Real                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────────┬────────────────┬────────────────┐          │
│  │ ALERTAS HOJE   │ CONTAS BANIDAS │ FALSOS POSIT.  │          │
│  │      12        │       3        │     5.2%       │          │
│  │  ⚠️ 2 críticos │  hoje          │  meta: < 5%    │          │
│  └────────────────┴────────────────┴────────────────┘          │
│                                                                 │
│  Alertas ativos:                                                │
│  🔴 #001 — Possível colusão (mesas 7, 12) — 2 jogadores        │
│  🔴 #002 — Chip dumping detectado — conta "playerXYZ"         │
│  🟡 #003 — Timing anômalo (possível bot) — conta "proGamer"   │
│  🟡 #004 — Multi-account suspeito — 3 contas, mesmo device    │
│  🟢 #005 — Login de novo IP — conta "oldPlayer" (baixo risco)  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  TENTATIVAS DE LOGIN (últimas 24h)                       │   │
│  │                                                          │   │
│  │  1000 ┤                                                  │   │
│  │      │                  █                                │   │
│  │   500 ┤            █     █                              │   │
│  │      │      █     █     █     █                          │   │
│  │     0 └──────────────────────────────────────────→       │   │
│  │       00h  04h  08h  12h  16h  20h  24h                  │   │
│  │                                                          │   │
│  │  Total: 3.247  |  Falhas: 312 (9.6%)  |  Bloqueadas: 89  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Vulnerabilidades conhecidas: 0 críticas, 2 médias, 5 baixas   │
│  Último pen test: Em andamento (Seção 4)                       │
│  Bug bounty: Não iniciado                                       │
└─────────────────────────────────────────────────────────────────┘
```

## 🛠️ 8.B.4 FERRAMENTAS RECOMENDADAS PARA DASHBOARD — Stack do Poker

### 📊 8.B.4.1 Stack de Observabilidade (Rust-native) do Motor de Poker

| Camada                      | Ferramenta                                      | Função                              | Quando     |
|-----------------------------|-------------------------------------------------|-------------------------------------|------------|
| **Coleta de métricas**      | Prometheus                                      | Time-series metrics                 | F2         |
| **Visualização**            | Grafana                                         | Dashboards                          | F2         |
| **Logs**                    | Loki (Grafana stack)                            | Log aggregation                     | F2         |
| **Tracing**                 | Jaeger / Tempo                                  | Distributed tracing                 | F3         |
| **Rust integration**        | `tracing` crate + `tracing-subscriber`          | Instrumentação Rust                 | F2 (já no stack) |
| **Rust metrics**            | `metrics` crate + `metrics-exporter-prometheus` | Exportar métricas Rust → Prometheus | F2         |
| **Alertas**                 | Alertmanager (Prometheus)                       | Alertas baseados em regras          | F2         |
| **Uptime**                  | Uptime Kuma / Blackbox Exporter                 | Monitoramento externo               | F2         |

### 📋 8.B.4.2 Gestão de Tarefas e Projetos da Plataforma de Poker

| Ferramenta              | Tipo                        | Quando        | Por quê                                      |
|-------------------------|-----------------------------|---------------|----------------------------------------------|
| **GitHub Projects**     | Kanban + Issues             | F2 (atual)    | Já usamos GitHub, integração nativa          |
| **Linear**              | Issue tracking moderno      | F3            | Mais rápido que Jira, UX excelente           |
| **Notion**              | Documentação + dashboard    | F2            | Flexível, dashboard com tabelas              |
| **Excalidraw**          | Diagramas                   | F2            | Diagramas de arquitetura rápidos             |

### 🖥️ 8.B.4.3 Dashboard Customizado (Rust + Dioxus) para Poker Online

> **Diferencial:** Como o projeto já usa Rust + Dioxus para o frontend, podemos
> construir um **dashboard customizado** com a mesma stack, consumindo dados
> do Prometheus via API e renderizando em WASM.

| Vantagem                  | Descrição                                                       |
|---------------------------|-----------------------------------------------------------------|
| **Stack unificada**       | Mesma linguagem (Rust) do motor e do frontend                   |
| **Performance**           | WASM é mais rápido que JS para renderização                     |
| **Customização total**    | Layout e métricas sob medida                                    |
| **Sem custo adicional**   | Sem licença de ferramenta externa                               |
| **Aprendizado**           | Pratica Dioxus para o produto real                              |

### 🔗 8.B.4.4 Integração com Arquivos Existentes do Projeto de Poker

| Arquivo do Projeto        | Função no Dashboard                 | Atualização     |
|---------------------------|-------------------------------------|-----------------|
| `DASHBOARD.md`            | Métricas de desenvolvimento         | Semanal         |
| `CRONOGRAMA.md`           | Timeline e marcos                   | Mensal          |
| `CRONOGRAMA.md`           | Timeline e marcos                   | Mensal          |
| `DEVELOPMENT_LOG.md`      | Log de mudanças                     | Contínuo        |
| `BUSINESS_RULES.md`       | Regras de negócio implementadas     | Contínuo        |
| GitHub Issues             | Backlog e tarefas                   | Contínuo        |
| GitHub Actions CI         | Status de builds e testes           | Contínuo        |

## 🔄 8.B.5 ROTINA DE GESTÃO — RITMO DE EXECUÇÃO DA PLATAFORMA DE POKER

### Rotina diária (15-30 min)

| Horário               | Atividade                                               | Duração     |
|-----------------------|---------------------------------------------------------|-------------|
| **Início do dia**     | Revisar dashboard (saúde, alertas, Kanban)              | 5 min       |
| **Início do dia**     | Definir 3 tarefas do dia (MIT — Most Important Tasks)   | 5 min       |
| **Durante o dia**     | Time blocks + Pomodoros                                 | 6-8h        |
| **Fim do dia**        | Atualizar DASHBOARD.md e Kanban                         | 10 min      |
| **Fim do dia**        | Daily assíncrono (o que fiz, o que farei)               | 5 min       |

### Rotina semanal (1-2h)

| Dia            | Atividade                                                     | Duração     |
|----------------|---------------------------------------------------------------|-------------|
| **Segunda**    | Planejamento da semana (revisar OKRs, priorizar backlog)      | 30 min      |
| **Sexta**      | Retrospectiva semanal (o que funcionou, o que melhorar)       | 30 min      |
| **Domingo**    | Revisão GTD (inbox, projetos, próximas ações)                 | 30 min      |

### Rotina mensal (2-3h)

| Atividade                               | Duração         |
|-----------------------------------------|-----------------|
| Revisão de OKRs mensais                 | 1h              |
| Atualização do DASHBOARD.md             | 30 min          |
| Revisão de burn rate e runway           | 30 min          |
| Game Day (chaos engineering)            | 4h (mensal)     |
| Retrospectiva do mês                    | 30 min          |

### Rotina trimestral (4h)

| Atividade                                  | Duração     |
|--------------------------------------------|-------------|
| Definição de novos OKRs                    | 2h          |
| Revisão de OKRs do trimestre anterior      | 1h          |
| Planejamento de roadmap                    | 1h          |

## 📌 8.B.6 PRINCÍPIOS DE GESTÃO — RESUMO DO ARQUITETO DE POKER

| #   | Princípio                                                       | Aplicação                                                              |
|-----|-----------------------------------------------------------------|------------------------------------------------------------------------|
| 1   | **OKRs definem a direção, tarefas definem o caminho**           | OKRs → Sprint → Tarefas → Pomodoros                                   |
| 2   | **Medir tudo, mas focar no que importa**                        | Dashboard mostra 50 métricas, foco em 5-7 KPIs                         |
| 3   | **Deep work é a superpotência do engenheiro**                   | 2-4h/dia de trabalho focado, sem distrações                            |
| 4   | **Limitar WIP aumenta produtividade**                           | Máximo 3 tarefas em progresso                                          |
| 5   | **Automatizar o repetitivo**                                    | CI/CD, testes, deploy, dashboard                                       |
| 6   | **Transparência radical**                                       | Dashboard visível para toda a equipe                                   |
| 7   | **Melhoria contínua**                                           | Retrospectivas semanais e mensais                                      |
| 8   | **Buffer para o inesperado**                                    | 30% de buffer em estimativas (Lei de Hofstadter)                       |
| 9   | **Foco no Quadrante II**                                        | 80% do tempo no importante-não-urgente                                 |
| 10  | **Dashboard é o sistema nervoso**                               | 5 segundos para saber saúde, crescimento e segurança                   |

---

# 🏗️ 12. ARQUITETURA DE SOFTWARE — Padrões e Decisões Estruturais da Plataforma de Poker

> **Princípio:** "A arquitetura é a decisão mais cedo que você toma e mais
> tarde que você pode mudar." — Adaptado de Ralph Johnson
>
> Para uma plataforma de poker com dinheiro real, a arquitetura deve ser:
> **segura por design**, **escalável horizontalmente**, **tolerante a falhas**
> e **auditável**. Cada decisão arquitetural deve ser justificada por um
> requisito de negócio, não por hype tecnológico.

## 🏗️ 12.1 ESTILO ARQUITETURAL — MICROSERVIÇOS MODULAR DA PLATAFORMA DE POKER

### 🏛️ 12.1.1 Decisão: Monolito Modular → Microserviços (Evolução do Motor de Poker)

| Fase              | Arquitetura                                                          | Justificativa                                    |
|-------------------|----------------------------------------------------------------------|--------------------------------------------------|
| **F1 (atual)**    | Monolito modular em Rust                                             | Time solo, simplicidade, velocidade de desenvolvimento |
| **F2**            | Monolito modular + serviços externos (pagamentos, antifraude)        | Isolar domínios sensíveis                        |
| **F3**            | Microserviços por domínio (motor, auth, pagamentos, antifraude)      | Escalar componentes independentemente            |
| **F4**            | Event-driven + microserviços                                         | Desacoplar via Kafka, escalar por evento         |

> **Regra de ouro:** Começar com monolito modular e extrair microserviços
> **apenas quando houver necessidade real** (escala, equipe, deploy independente).
> Não fazer microserviços prematuros — é a armadilha mais comum.

### 🧩 12.1.2 Domínios Identificados (Bounded Contexts) da Plataforma de Poker

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PLATAFORMA DE POKER ONLINE                       │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │   MOTOR DE   │  │   AUTH &     │  │   LOBBY &    │             │
│  │   POKER      │  │   SESSÃO     │  │   MATCHMAKING│             │
│  │              │  │              │  │              │             │
│  │ • Deck       │  │ • Login      │  │ • Mesas      │             │
│  │ • Side pots  │  │ • JWT        │  │ • Torneios   │             │
│  │ • Rake       │  │ • MFA/TOTP   │  │ • Matchmaking│             │
│  │ • RNG        │  │ • RBAC       │  │ • Fila       │             │
│  │ • Hand hist. │  │ • KYC        │  │              │             │
│  │ • Loss Defl. │  │              │  │              │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │   PAGAMENTOS │  │   ANTIFRAUDE │  │   ANALYTICS  │             │
│  │              │  │              │  │   & IA       │             │
│  │ • Depósito   │  │ • Colusão    │  │ • Métricas   │             │
│  │ • Saque      │  │ • Chip dump  │  │ • Dashboards │             │
│  │ • PIX        │  │ • Bot detect │  │ • IA Coach   │             │
│  │ • Cartão     │  │ • Multi-acc  │  │ • Hand anal. │             │
│  │ • Saldo      │  │              │  │              │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │   NOTIFICA-  │  │   AUDITORIA  │  │   COMPLIANCE │             │
│  │   ÇÕES       │  │   & LOGS     │  │   REGULATÓRIO│             │
│  │              │  │              │  │              │             │
│  │ • Email      │  │ • Hand hist. │  │ • KYC/AML    │             │
│  │ • Push       │  │ • Transações │  │ • RGPS       │             │
│  │ • In-game    │  │ • Eventos    │  │ • LGPD       │             │
│  │              │  │ • Imutável   │  │ • UK GC      │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
└─────────────────────────────────────────────────────────────────────┘
```

### 📐 12.1.3 Princípios Arquiteturais (12-Factor App) Aplicados ao Poker Online

| #   | Princípio                | Aplicação no Projeto                                              |
|-----|--------------------------|------------------------------------------------------------------|
| 1   | **Codebase**             | 1 repositório, múltiplos deploys (dev, staging, prod)            |
| 2   | **Dependencies**         | `Cargo.toml` explícito, sem dependências implícitas              |
| 3   | **Config**               | Variáveis de ambiente (`.env`), não hardcoded                    |
| 4   | **Backing services**     | PostgreSQL, Redis, Kafka como recursos externos                  |
| 5   | **Build, release, run**  | CI/CD separa build de release (Docker tags)                      |
| 6   | **Processes**            | Stateless (sessão no Redis, não na memória)                      |
| 7   | **Port binding**         | Cada serviço em sua porta (Docker)                               |
| 8   | **Concurrency**          | Tokio async, workers por CPU                                     |
| 9   | **Disposability**        | Graceful shutdown (SIGTERM), startup rápido                      |
| 10  | **Dev/prod parity**      | Docker idêntico em dev e prod                                    |
| 11  | **Logs**                 | stdout como event stream (Loki + Grafana)                        |
| 12  | **Admin processes**      | Migrations, seeds como tasks one-off                             |

## 🧩 12.2 PADRÕES DE PROJETO (DESIGN PATTERNS) DO MOTOR DE POKER

### 🏗️ 12.2.1 Padrões Estruturais Aplicados ao Motor de Poker

| Padrão          | Aplicação                                                    | Arquivo                              |
|-----------------|--------------------------------------------------------------|--------------------------------------|
| **Repository**  | Abstrair acesso a dados (PostgreSQL, Redis)                  | `auth.rs`, `hand_history.rs`         |
| **Factory**     | Criar diferentes tipos de jogo (cash, MTT, Sit&Go)           | `tournament_engine.rs`               |
| **Strategy**    | Diferentes estratégias de rake, blind structure              | `rake.rs`, `tournament_engine.rs`    |
| **Observer**    | Eventos de mão, notificações de jogo                         | `hand_history.rs`                    |
| **Builder**     | Construir estado de jogo passo a passo                       | `types.rs`                           |
| **State**       | Estado da mesa (waiting, dealing, betting, showdown)         | `types.rs`                           |
| **Command**     | Ações do jogador (fold, call, raise, all-in)                 | `types.rs`                           |
| **Decorator**   | Middleware de auth, rate limiting, logging                   | `auth.rs`                            |

### ⚡ 12.2.2 Padrões de Concorrência (Rust-specific) para Mesas de Poker Concorrentes

| Padrão                    | Aplicação                                      | Crate                        |
|---------------------------|-----------------------------------------------|------------------------------|
| **Actor Model**           | Cada mesa de poker como um actor isolado      | `actix` ou `tokio::spawn`    |
| **Channel (mpsc)**        | Comunicação entre motor e clientes            | `tokio::sync::mpsc`          |
| **Mutex/RwLock**          | Estado compartilhado (lobby, salas)           | `tokio::sync::RwLock`        |
| **Arc**                   | Compartilhamento imutável entre tasks         | `std::sync::Arc`             |
| **Semaphore**             | Limitar conexões concorrentes                 | `tokio::sync::Semaphore`     |
| **Barrier**               | Sincronizar início de torneio                 | `tokio::sync::Barrier`       |

### 🃏 12.2.3 Padrões de Domínio (DDD) do Jogo de Poker

| Conceito              | Aplicação                                                             |
|-----------------------|-----------------------------------------------------------------------|
| **Entity**            | `Player`, `Table`, `Tournament`, `Hand`                               |
| **Value Object**      | `Card`, `ChipCount`, `PlayerId`, `HandRank`                           |
| **Aggregate Root**    | `Table` (contém jogadores, pot, deck, estado)                         |
| **Domain Event**      | `HandStarted`, `PlayerActed`, `PotAwarded`, `TournamentFinished`      |
| **Repository**        | `PlayerRepository`, `HandHistoryRepository`                           |
| **Service**           | `RakeCalculator`, `SidePotCalculator`, `LossDeflator`                 |
| **Factory**           | `GameFactory` (cria cash game, MTT, Sit&Go)                           |

## 📡 12.3 COMUNICAÇÃO ENTRE SERVIÇOS DA PLATAFORMA DE POKER

### 🔌 12.3.1 Protocolos de Comunicação do Motor de Poker (gRPC, WebSocket, REST)

| Tipo                          | Protocolo               | Quando        | Por quê                              |
|-------------------------------|-------------------------|---------------|--------------------------------------|
| **Cliente ↔ Servidor**        | WebSocket (TLS 1.3)     | Tempo real de jogo | Baixa latência, bidirecional     |
| **Serviço ↔ Serviço**         | gRPC (TLS + Protobuf)   | F3+           | Tipado, performático, streaming     |
| **Serviço ↔ Serviço (atual)** | API REST via HTTPS/JSON | F2            | Simples, interoperável              |
| **Eventos assíncronos**       | Kafka                   | F2+           | Desacoplar, replay, audit           |
| **Cache**                     | Redis (RESP)            | F2            | Sub-ms, pub/sub                     |
| **Métricas**                  | Prometheus (HTTPS /metrics)| F2          | Padrão de mercado                   |

### 📜 12.3.2 Event Sourcing para Hand History (Histórico de Mãos de Poker)

> **Decisão crítica:** Hand history deve ser **imutável e auditável**. Cada
> mão é uma sequência de eventos que pode ser reproduzida.

```rust
// Eventos de domínio (imutáveis)
enum HandEvent {
    HandStarted { hand_id: Uuid, table_id: Uuid, players: Vec<PlayerId>, blinds: Blinds },
    CardsDealt { hand_id: Uuid, player_id: PlayerId, cards: [Card; 2] },
    ActionTaken { hand_id: Uuid, player_id: PlayerId, action: Action, amount: Option<u64> },
    FlopDealt { hand_id: Uuid, cards: [Card; 3] },
    TurnDealt { hand_id: Uuid, card: Card },
    RiverDealt { hand_id: Uuid, card: Card },
    Showdown { hand_id: Uuid, results: Vec<PotResult> },
    HandFinished { hand_id: Uuid, rake_collected: u64 },
}
```

| Vantagem              | Descrição                                                       |
|-----------------------|-----------------------------------------------------------------|
| **Auditabilidade**    | Replay completo de qualquer mão para disputas                   |
| **Debug**             | Reproduzir bug exato que um jogador reportou                    |
| **Antifraude**        | Análise post-mortem de padrões suspeitos                        |
| **Compliance**        | Requisito regulatório (UK GC, Malta GA)                         |
| **IA**                | Dataset para treinar IA coach e detecção de fraude              |

## 📈 12.4 ESCALABILIDADE DAS MESAS DE POKER CONCORRENTES

### 📈 12.4.1 Estratégias de Escala da Plataforma de Poker Online

| Componente              | Estratégia                                  | Métrica de escala              |
|-------------------------|---------------------------------------------|--------------------------------|
| **Motor de poker**      | Stateless, escalar por mesas                | Mesas ativas por instância     |
| **WebSocket gateway**   | Sticky sessions ou Redis pub/sub            | Conexões concorrentes          |
| **Auth**                | Stateless (JWT), escalar horizontalmente    | Logins/seg                     |
| **PostgreSQL**          | Read replicas + particionamento             | Queries/seg                    |
| **Redis**               | Cluster mode, sharding por mesa             | Operações/seg                  |
| **Kafka**               | Particionamento por topic                   | Mensagens/seg                  |
| **Antifraude**          | Workers assíncronos consumindo Kafka        | Eventos/seg                    |

### 🎯 12.4.2 Targets de Escala por Fase do Poker Online (MVP → Growth → Scale)

| Fase   | Mesas simultâneas   | Jogadores online   | Mãos/seg   | Infra                              |
|--------|---------------------|--------------------|------------|------------------------------------|
| **F1** | 10                  | 50                 | 5          | 1 container                        |
| **F2** | 100                 | 500                | 50         | 3-5 containers                     |
| **F3** | 1.000               | 5.000              | 500        | 10-20 containers + replicas        |
| **F4** | 10.000              | 50.000             | 5.000      | Kubernetes, auto-scaling           |

## 🛡️ 12.5 TOLERÂNCIA A FALHAS E RESILIÊNCIA DO MOTOR DE POKER

### 🛡️ 12.5.1 Padrões de Resiliência do Motor de Poker (Circuit Breaker, Retry, Bulkhead)

| Padrão                      | Aplicação                                            | Biblioteca                    |
|-----------------------------|------------------------------------------------------|-------------------------------|
| **Circuit Breaker**         | Proteger contra dependência falha (pagamentos)       | `tower` middleware            |
| **Retry com backoff**       | Re-tentar operações transitórias                     | `backoff` crate               |
| **Timeout**                 | Evitar travar indefinidamente                        | `tokio::time::timeout`        |
| **Bulkhead**                | Isolar recursos por domínio                          | Pools de conexões separados   |
| **Rate limiting**           | Proteger APIs de abuso                               | `tower::limit::rate`          |
| **Graceful degradation**    | Funcionar parcialmente se algo cair                  | Modo "read-only" se DB cair   |
| **Idempotency**             | Operações seguras de repetir                         | Idempotency keys em pagamentos|

### 💥 12.5.2 Cenários de Falha e Resposta nas Mesas de Poker

| Cenário                  | Impacto                    | Resposta                                          | RTO     | RPO     |
|--------------------------|----------------------------|---------------------------------------------------|---------|---------|
| **WebSocket cai**        | Jogadores desconectados    | Reconnect automático, sync de estado              | 5s      | 0s      |
| **PostgreSQL cai**       | Sem novas mãos             | Read replica assume, graceful degrade             | 30s     | 0s (sync)|
| **Redis cai**            | Sem cache de sessão        | Fallback para DB, degradação lenta                | 10s     | 0s      |
| **Kafka cai**            | Eventos não processados    | Buffer local, replay ao recuperar                 | 1min    | 0s      |
| **Motor crash**          | Mesa congelada             | Reiniciar mesa, restaurar estado de hand history  | 10s     | 0s      |
| **DDoS**                 | Indisponibilidade          | Rate limit + WAF + Cloudflare                     | 1min    | —       |

> **RTO** = Recovery Time Objective (tempo máximo de recuperação)
> **RPO** = Recovery Point Objective (perda máxima de dados aceitável)

## 🔐 12.6 SEGURANÇA ARQUITETURAL DA PLATAFORMA DE POKER ONLINE

### 🛡️ 12.6.1 Defesa em Profundidade (Defense in Depth) da Plataforma de Poker

```
┌─────────────────────────────────────────────────────────────────┐
│  Camada 1: Rede (Cloudflare WAF, DDoS protection, TLS 1.3)      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Camada 2: API Gateway (rate limiting, auth, validation)  │  │
│  │  ┌─────────────────────────────────────────────────────┐ │  │
│  │  │  Camada 3: Aplicação (RBAC, input validation, CSRF) │ │  │
│  │  │  ┌───────────────────────────────────────────────┐ │ │  │
│  │  │  │  Camada 4: Dados (AES-256 at-rest, TLS in-transit)│ │ │  │
│  │  │  │  ┌─────────────────────────────────────────┐  │ │ │  │
│  │  │  │  │  Camada 5: Auditoria (logs imutáveis, │  │ │ │  │
│  │  │  │  │  hand history, alertas antifraude)     │  │ │ │  │
│  │  │  │  └─────────────────────────────────────────┘  │ │ │  │
│  │  │  └───────────────────────────────────────────────┘ │ │  │
│  │  └─────────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 🔐 12.6.2 Princípio do Menor Privilégio no Poker Online (Jogadores e Operadores)

| Componente          | Privilégios                                    | Justificativa                    |
|---------------------|------------------------------------------------|----------------------------------|
| **Motor de poker**  | Apenas ler/escrever hand history               | Não acessa pagamentos            |
| **Auth service**    | Apenas ler/escrever users                      | Não acessa hand history          |
| **Pagamentos**      | Apenas ler saldo, escrever transações          | Isolado do motor                 |
| **Antifraude**      | Ler eventos, escrever alertas                  | Não modifica estado de jogo      |
| **Analytics**       | Ler (read-only)                                | Não escreve em produção          |

### 🔒 12.6.3 Zero Trust Architecture Aplicada ao Motor de Poker

| Princípio                                  | Aplicação                                                          |
|--------------------------------------------|--------------------------------------------------------------------|
| **Nunca confiar, sempre verificar**        | Cada request valida JWT, mesmo interno                             |
| **Menor privilégio**                       | Tokens com escopo mínimo, expiração curta                          |
| **Assumir breach**                         | Criptografia even dentro da rede (mTLS)                            |
| **Verificação contínua**                   | Re-autenticação para ações sensíveis (saque)                       |
| **Segmentação**                            | Redes isoladas por domínio (VPC, security groups)                  |

---

# 🛡️ 13. DEVSECOPS — Maturidade DSOMM (OWASP) Aplicada à Plataforma de Poker

> **Princípio:** "Segurança não é um produto, mas um processo." — Bruce Schneier
>
> DevSecOps integra segurança em **todo o ciclo de vida** do software —
> não como uma auditoria no final, mas como prática contínua desde o commit
> até a produção. O OWASP DSOMM (DevSecOps Maturity Model) é o framework
> que guia essa jornada.

## 📊 13.1 MODELO DE MATURIDADE DSOMM — Poker Edition

### 📊 13.1.1 Os 5 Níveis de Maturidade DSOMM da Plataforma de Poker

| Nível   | Nome           | Descrição                                    | Estado do Projeto   |
|---------|----------------|----------------------------------------------|---------------------|
| **1**   | Basic          | Segurança manual, ad hoc                     | ❌ Não aceitável    |
| **2**   | Intermediate   | Algumas ferramentas automatizadas            | 🔄 Meta F2          |
| **3**   | Advanced       | Segurança integrada no pipeline              | ✅ Meta F3          |
| **4**   | Advanced+      | Segurança como código, métricas              | 🎯 Meta F4          |
| **5**   | Expert         | Segurança auto-adaptativa, IA                | 🚀 Visão            |

### 🧩 13.1.2 As 4 Dimensões do DSOMM Aplicadas ao Motor de Poker

```
┌─────────────────────────────────────────────────────────────────┐
│                    DSOMM — 4 DIMENSÕES                          │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  CULTURE &   │  │  GOVERNANCE  │  │  PIPELINE    │         │
│  │  ORGANIZATION│  │  & MANAGEMENT│  │  & BUILD     │         │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤         │
│  │ • Treinamento│  │ • Políticas  │  │ • SAST       │         │
│  │ • Awareness  │  │ • Compliance │  │ • DAST       │         │
│  │ • Responsab. │  │ • Métricas   │  │ • SCA        │         │
│  │ • Colaboração│  │ • Auditoria  │  │ • Secrets    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  ENVIRONMENT & CONFIGURATION                             │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │ • IaC scanning • Container hardening • Runtime protection│   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 👥 13.2 CULTURE & ORGANIZATION — Cultura DevSecOps do Poker

### 📈 13.2.1 Maturidade por Nível (DSOMM) da Plataforma de Poker Online

| Nível   | Prática                                            | Status              |
|---------|----------------------------------------------------|---------------------|
| **1**   | Conscientização básica de segurança                | ✅ Já temos (Seção 4)|
| **2**   | Treinamento regular de segurança                   | 🔄 Meta F2          |
| **3**   | Security Champions no time                         | 🎯 Meta F3          |
| **4**   | Cultura "security first" em toda equipe            | 🚀 Meta F4          |
| **5**   | Segurança auto-organizada, gamificada              | 🚀 Visão            |

### 🛠️ 13.2.2 Práticas Recomendadas de DevSecOps para Poker Online

| Prática                          | Descrição                                              | Frequência              |
|----------------------------------|--------------------------------------------------------|-------------------------|
| **Treinamento OWASP Top 10**     | Todo dev conhece as 10 vulnerabilidades                | Onboarding + anual      |
| **Security Friday**              | 1h/semana dedicada a segurança                         | Semanal                 |
| **Threat modeling**              | Modelar ameaças antes de cada feature                  | Por feature             |
| **Security Champion**            | 1 pessoa responsável por segurança no time             | Contínuo                |
| **Bug bounty interno**           | Recompensa por encontrar bugs de segurança             | Contínuo                |
| **Postmortem de incidentes**     | Aprender com cada incidente                            | Por incidente           |

## 🏛️ 13.3 GOVERNANCE & MANAGEMENT — Governança do Poker

### 📜 13.3.1 Políticas de Segurança (como código) da Plataforma de Poker

| Política                       | Ferramenta              | Arquivo                  |
|--------------------------------|-------------------------|--------------------------|
| **Análise de dependências**    | `cargo-audit`           | CI pipeline              |
| **Licença compliance**         | `cargo-deny`            | `deny.toml`              |
| **Secrets scanning**           | `gitleaks`              | Pre-commit hook          |
| **Política de senhas**         | bcrypt 0.16 + MFA       | `auth.rs`                |
| **Política de JWT**            | RS256, expiração 15min access / 7d refresh | `auth.rs`   |
| **Política de CORS**           | Whitelist de origens    | API gateway              |
| **Política de rate limit**     | 100 req/min por IP, 1000 por token | `tower` middleware  |

### ⚖️ 13.3.2 Compliance e Auditoria do Poker Online (LGPD, UKGC, MGA)

| Framework                      | Aplicação                                  | Quando                          |
|--------------------------------|--------------------------------------------|---------------------------------|
| **OWASP WSTG**                 | Testes de segurança (Seção 4)              | F2                              |
| **OWASP ASVS**                 | Verification standard para app             | F2                              |
| **OWASP DSOMM**                | Este documento                             | F2                              |
| **PCI-DSS**                    | Se processar cartões diretamente           | F3 (ou usar gateway)            |
| **LGPD**                       | Proteção de dados (Brasil)                 | F2                              |
| **UK Gambling Commission**     | Licença de poker (Seção 16)                | F4                              |
| **RGPS**                       | Regulamento de jogos (Brasil)              | F4                              |

## 🔧 13.4 PIPELINE & BUILD — Ferramentas de Segurança do Poker

### 🔄 13.4.1 Pipeline CI/CD com Segurança Integrada do Motor de Poker

```yaml
# .github/workflows/security.yml (conceitual)
name: Security Pipeline
on: [push, pull_request]

jobs:
  # 1. SAST — Static Application Security Testing
  sast:
    steps:
      - run: cargo clippy -- -W clippy::all -D warnings
      - run: cargo audit                          # CVE check
      - run: cargo deny check                      # licenças + advisories

  # 2. SCA — Software Composition Analysis
  sca:
    steps:
      - run: cargo audit --db ./advisory-db
      - run: cargo outdated                        # dependências desatualizadas

  # 3. Secrets Scanning
  secrets:
    steps:
      - uses: gitleaks/gitleaks-action@v2

  # 4. DAST — Dynamic Application Security Testing
  dast:
    steps:
      - run: docker compose up -d
      - run: cargo test --test integration_tests
      - run: owasp-zap-baseline https://localhost

  # 5. Container Scanning
  container:
    steps:
      - uses: aquasecurity/trivy-action@master
        with: { image-ref: poker-platform:latest }

  # 6. IaC Scanning
  iac:
    steps:
      - run: trivy config docker-compose.yml
      - run: checkov -d Infraestrutura-Docker/

  # 7. Fuzzing (contínuo)
  fuzz:
    steps:
      - run: cargo fuzz run motor_fuzz -- -max_total_time=300
```

### 🧰 13.4.2 Ferramentas por Categoria (SAST, DAST, SCA) para Poker Online

| Categoria        | Ferramenta              | O que detecta                          | Quando               |
|------------------|-------------------------|----------------------------------------|----------------------|
| **SAST**         | `cargo clippy`          | Code smells, bugs, más práticas        | Cada commit          |
| **SAST**         | `cargo audit`           | CVEs em dependências                   | Cada commit + diário |
| **SAST**         | `cargo deny`            | Licenças incompatíveis, advisories     | Cada commit          |
| **SCA**          | `cargo outdated`        | Dependências desatualizadas            | Semanal              |
| **Secrets**      | `gitleaks`              | Senhas/tokens no código                | Pre-commit + CI      |
| **DAST**         | OWASP ZAP               | Vulnerabilidades em runtime            | Por release          |
| **Container**    | Trivy                   | Vulnerabilidades em imagem Docker      | Cada build           |
| **IaC**          | Checkov / Trivy config  | Misconfig em Docker/Terraform          | Cada commit          |
| **Fuzzing**      | `cargo-fuzz`            | Crashes, panics, UB                    | Contínuo             |
| **Mutation**     | `cargo-mutants`         | Testes insuficientes                   | Semanal              |

### 🪝 13.4.3 Pre-commit Hooks para Segurança do Código do Motor de Poker

```yaml
# .pre-commit-config.yaml (conceitual)
repos:
  - repo: https://github.com/cargo-bins/cargo-binstall
    hooks:
      - id: cargo-fmt
      - id: cargo-clippy
        args: [--, -D, warnings]
      - id: cargo-audit
      - id: cargo-deny
        args: [check]
  - repo: https://github.com/gitleaks/gitleaks
    hooks:
      - id: gitleaks
```

## ⚙️ 13.5 ENVIRONMENT & CONFIGURATION — Configuração Segura do Poker

### 🐳 13.5.1 Hardening de Container Docker do Motor de Poker

| Prática                    | Implementação                                          |
|----------------------------|--------------------------------------------------------|
| **Imagem mínima**          | `FROM debian:bookworm-slim` ou `distroless`            |
| **Usuário não-root**       | `USER 1000:1000` no Dockerfile                         |
| **Read-only filesystem**   | `read_only: true` no docker-compose                    |
| **No new privileges**      | `security_opt: [no-new-privileges:true]`               |
| **Capacities drop**        | `cap_drop: [ALL]`                                      |
| **Resource limits** | `mem_limit`, `cpus` no compose |
| **Health checks** | `HEALTHCHECK` no Dockerfile |
| **Multi-stage build** | Builder → runtime, imagem final mínima |

### 📜 13.5.2 Dockerfile Hardened (Exemplo) para o Motor de Poker em Rust

```dockerfile
# Build stage
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage (mínimo possível)
FROM debian:bookworm-slim AS runtime
RUN groupadd -r poker && useradd -r -g poker -u 1000 poker
COPY --from=builder /app/target/release/poker /usr/local/bin/poker
USER 1000:1000
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s CMD curl -fk https://localhost/health || exit 1
ENTRYPOINT ["poker"]
```

### 🛡️ 13.5.3 Runtime Protection do Motor de Poker (Falco, Tracee)

| Camada     | Ferramenta                  | Função                              |
|------------|-----------------------------|-------------------------------------|
| **WAF**    | Cloudflare / AWS WAF        | Filtrar SQLi, XSS, DDoS L7          |
| **DDoS**   | Cloudflare / AWS Shield     | Mitigar ataques volumétricos        |
| **RASP**   | (Runtime App Self-Protection) | Detectar ataques em runtime       |
| **IDS/IPS**| Suricata / Snort            | Intrusion detection                 |
| **SIEM**   | ELK / Loki + Grafana        | Correlacionar eventos de segurança  |

## 🗺️ 13.6 MATURIDADE DSOMM — Roadmap do Poker

| Dimensão         | F2 (Nível 2)                  | F3 (Nível 3)                   | F4 (Nível 4)                       |
|------------------|-------------------------------|--------------------------------|------------------------------------|
| **Culture**      | Treinamento OWASP Top 10      | Security Champion              | Security-first culture             |
| **Governance**   | Políticas como código         | Métricas de segurança          | Compliance automatizado            |
| **Pipeline**     | SAST + SCA + secrets scan     | + DAST + container scan        | + Fuzzing contínuo + mutation      |
| **Environment**  | Container hardening           | IaC scanning                   | Runtime protection + RASP          |

---

# 📈 14. OBSERVABILIDADE — Tracing, Metrics e Logs em Rust no Motor de Poker

> **Princípio:** "Observabilidade não é sobre monitorar o que você sabe que
> pode quebrar. É sobre entender o que você não sabia que podia quebrar."
>
> — Adaptado de Charity Majors, co-autora de *Observability Engineering*
>
> Para uma plataforma de poker, observabilidade é **não negociável**:
> - Cada mão de poker é uma transação financeira
> - Cada disconnect é perda de receita
> - Cada bug de saldo é um desastre regulatório
> - Cada fraude não detectada é perda de confiança

## 🏛️ 14.1 OS 3 PILARES DA OBSERVABILIDADE — Motor de Poker

```
┌─────────────────────────────────────────────────────────────────┐
│              OBSERVABILIDADE (3 PILARES)                        │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   MÉTRICAS   │  │    LOGS      │  │   TRACING    │         │
│  │              │  │              │  │              │         │
│  │ • Numéricos  │  │ • Eventos    │  │ • Causalidade│         │
│  │ • Agregados  │  │ • Contexto   │  │ • Latência   │         │
│  │ • Alertas    │  │ • Debug      │  │ • Spans      │         │
│  │              │  │              │  │              │         │
│  │ Prometheus   │  │ Loki/ELK     │  │ Jaeger/Tempo │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  CORTE: OpenTelemetry (padrão unificado)                 │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

| Pilar          | Responde                    | Granularidade                    | Custo   |
|----------------|-----------------------------|----------------------------------|---------|
| **Métricas**   | "Quantos?"                  | Agregado, baixa cardinalidade    | Baixo   |
| **Logs**       | "O que aconteceu?"          | Evento discreto, alto contexto   | Médio   |
| **Tracing**    | "Por que demorou? Onde falhou?" | Por request, distribuído      | Alto    |

## 🔍 14.2 TRACING EM RUST — O Crate `tracing` no Poker

### 🔍 14.2.1 Por que `tracing` e não `log`? — Tracing Distribuído do Motor de Poker

| `log`                              | `tracing`                                          |
|------------------------------------|----------------------------------------------------|
| Eventos pontuais sem contexto      | Spans estruturados com contexto                    |
| Sem correlação entre eventos       | Correlação via span hierarchy                      |
| Sem async awareness                | Integra com Tokio (task context)                   |
| Apenas texto                       | Structured fields (key-value)                      |
| Sem sampling                       | Sampling configurável                              |

### 🃏 14.2.2 Instrumentação do Motor de Poker (Spans por Mão e Ação)

```rust
use tracing::{info_span, instrument, info, warn, error, Instrument};
use tracing_subscriber::{fmt, EnvFilter};

// Setup do subscriber
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("poker_motor=debug".parse().unwrap()))
        .with_target(false)
        .json()  // Structured JSON para Loki
        .init();
}

// Span para uma mão de poker inteira
async fn play_hand(table: &Table) -> Result<HandResult, Error> {
    let span = info_span!("hand", hand_id = %hand_id, table_id = %table.id);
    async move {
        info!(players = table.players.len(), "Hand started");

        // Sub-span para dealing
        let _deal_span = info_span!("dealing").entered();
        deck.shuffle_and_deal()?;
        drop(_deal_span);

        // Sub-span para betting rounds
        for round in [Preflop, Flop, Turn, River] {
            let _round_span = info_span!("betting_round", ?round).entered();
            betting::process_round(table, round).await?;
        }

        // Sub-span para showdown
        let _showdown_span = info_span!("showdown").entered();
        let result = showdown::resolve(table)?;
        info!(pot = result.pot, rake = result.rake, "Hand finished");
        Ok(result)
    }.instrument(span).await
}

// Instrument automático em funções
#[instrument(skip(self, player), fields(player_id = %player.id))]
async fn process_action(&self, player: &Player, action: Action) -> Result<(), Error> {
    match action {
        Action::Fold => info!("Player folded"),
        Action::Call(amount) => info!(amount, "Player called"),
        Action::Raise(amount) => info!(amount, "Player raised"),
        Action::AllIn => warn!("Player all-in"),
    }
    Ok(())
}
```

### 📊 14.2.3 Spans Estruturados por Domínio (Mesa, Jogador, Rake, Antifraude)

| Domínio          | Span raiz       | Sub-spans                                      | Campos chave                                          |
|------------------|-----------------|------------------------------------------------|-------------------------------------------------------|
| **Mão de poker** | `hand`          | `dealing`, `betting_round`, `showdown`         | `hand_id`, `table_id`, `players`, `pot`, `rake`       |
| **Torneio**      | `tournament`    | `register`, `blind_level`, `hand`, `payout`    | `tournament_id`, `level`, `players_remaining`         |
| **Auth**         | `auth`          | `login`, `verify_mfa`, `issue_token`           | `user_id`, `method`, `success`                        |
| **Pagamento**    | `payment`       | `validate`, `process`, `confirm`               | `user_id`, `amount`, `method`, `status`               |
| **Antifraude**   | `fraud_check`   | `collusion`, `chip_dump`, `bot_detect`         | `player_id`, `score`, `alert_level`                   |

## 📊 14.3 MÉTRICAS — Prometheus em Rust no Poker

### 🎯 14.3.1 Métricas Chave do Motor de Poker (Hands/s, Latência, Rake)

```rust
use metrics::{counter, gauge, histogram, describe_counter};

// Contadores (só aumentam)
counter!("poker_hands_total").increment(1);
counter!("poker_rake_collected_total", "currency" => "BRL").increment(rake_amount);
counter!("poker_errors_total", "type" => "side_pot").increment(1);

// Gauges (sobem e descem)
gauge!("poker_active_tables").set(active_tables);
gauge!("poker_active_players").set(active_players);
gauge!("poker_websocket_connections").set(ws_conns);

// Histogramas (distribuição)
histogram!("poker_hand_duration_seconds").record(duration.as_secs_f64());
histogram!("poker_websocket_latency_seconds").record(latency.as_secs_f64());
histogram!("poker_action_processing_seconds").record(proc_time.as_secs_f64());
```

### 📋 14.3.2 Catálogo de Métricas da Plataforma de Poker Online

| Nome                                | Tipo       | Labels                        | Descrição                          |
|-------------------------------------|------------|-------------------------------|------------------------------------|
| `poker_hands_total`                 | Counter    | `table_type`                  | Total de mãos jogadas              |
| `poker_rake_collected_total`        | Counter    | `currency`, `game_type`       | Rake acumulado                     |
| `poker_active_tables`               | Gauge      | —                             | Mesas ativas agora                 |
| `poker_active_players`              | Gauge      | —                             | Jogadores online agora             |
| `poker_websocket_connections`       | Gauge      | —                             | Conexões WS ativas                 |
| `poker_hand_duration_seconds`       | Histogram  | `game_type`                   | Duração de cada mão                |
| `poker_websocket_latency_seconds`   | Histogram  | —                             | Latência WS (p50, p95, p99)        |
| `poker_action_processing_seconds`   | Histogram  | `action_type`                 | Tempo para processar ação          |
| `poker_errors_total`                | Counter    | `type`, `module`              | Erros por tipo                     |
| `auth_login_attempts_total`         | Counter    | `method`, `success`           | Tentativas de login                |
| `auth_login_failures_total`         | Counter    | `reason`                      | Falhas de login por motivo         |
| `fraud_alerts_total`                | Counter    | `type`, `severity`            | Alertas de fraude                  |
| `payment_transactions_total`        | Counter    | `type`, `status`              | Transações de pagamento            |
| `payment_amount_total`              | Counter    | `type`, `currency`            | Volume financeiro                  |

### 🚨 14.3.3 Alertas (Prometheus Alertmanager) para Mesas de Poker

```yaml
# alerting_rules.yml (conceitual)
groups:
  - name: poker_critical
    rules:
      - alert: HighErrorRate
        expr: rate(poker_errors_total[5m]) > 0.1
        for: 2m
        labels: { severity: critical }
        annotations:
          summary: "Taxa de erro alta no motor de poker"

      - alert: WebSocketLatencyHigh
        expr: histogram_quantile(0.99, poker_websocket_latency_seconds) > 0.2
        for: 5m
        labels: { severity: warning }
        annotations:
          summary: "Latência WebSocket p99 > 200ms"

      - alert: FraudAlertsSpike
        expr: rate(fraud_alerts_total[10m]) > 5
        for: 5m
        labels: { severity: critical }
        annotations:
          summary: "Pico de alertas de fraude"

      - alert: NoActiveTables
        expr: poker_active_tables == 0
        for: 10m
        labels: { severity: warning }
        annotations:
          summary: "Nenhuma mesa ativa — possível outage"
```

## 📝 14.4 LOGS ESTRUTURADOS — Motor de Poker

### 📝 14.4.1 Padrão de Log Estruturado (JSON) do Motor de Poker

```json
{
  "timestamp": "2026-07-08T14:23:45.123Z",
  "level": "INFO",
  "target": "poker_motor::hand",
  "span": {
    "hand_id": "550e8400-e29b-41d4-a716-446655440000",
    "table_id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "hand"
  },
  "fields": {
    "message": "Hand finished",
    "players": 6,
    "pot": 1500,
    "rake": 45,
    "duration_ms": 12340
  }
}
```

### 📊 14.4.2 Níveis de Log por Ambiente (Dev, Staging, Prod) do Poker Online

| Ambiente      | Nível                  | Justificativa                                  |
|---------------|------------------------|------------------------------------------------|
| **Dev**       | `DEBUG`                | Máximo contexto para debug                     |
| **Staging**   | `INFO`                 | Equilíbrio entre contexto e volume             |
| **Produção**  | `WARN`                 | Apenas avisos e erros + tracing amostrado      |
| **Audit**     | `INFO` (separado)      | Logs de auditoria nunca filtrados              |

### 🔒 14.4.3 Logs de Auditoria (Imutáveis) das Mãos de Poker e Ações de Jogadores

| Evento               | Campos                                                      | Retenção   |
|----------------------|--------------------------------------------------------------|------------|
| **Login**            | user_id, ip, user_agent, success, method                     | 7 anos     |
| **Saque**            | user_id, amount, method, status, approved_by                 | 7 anos     |
| **Mão de poker**     | hand_id, players, pot, rake, winners                         | 7 anos     |
| **Banimento**        | user_id, reason, evidence, approved_by                       | 7 anos     |
| **Mudança de saldo** | user_id, delta, reason, balance_before, balance_after        | 7 anos     |

> **Requisito regulatório:** Logs de auditoria devem ser **imutáveis** (WORM —
> Write Once Read Many) e retidos por **7 anos** (padrão UK GC / Malta GA).

## 📈 14.5 DASHBOARDS DE OBSERVABILIDADE (Grafana) — Poker

### 📊 14.5.1 Dashboards Recomendados (Grafana) para a Plataforma de Poker

| Dashboard               | Painéis                                                    | Fonte                    | Atualização   |
|-------------------------|------------------------------------------------------------|--------------------------|---------------|
| **Saúde do Sistema**    | Uptime, error rate, CPU, memória, latência                 | Prometheus               | 10s           |
| **Negócio de Poker**    | Mesas ativas, mãos/seg, rake, DAU                          | Prometheus               | 30s           |
| **Performance**         | Latência p50/p95/p99, throughput, slow queries             | Prometheus + Jaeger      | 10s           |
| **Segurança**           | Login failures, fraud alerts, banned accounts              | Prometheus + logs        | 1min          |
| **Infraestrutura**      | Container health, DB connections, Kafka lag                | Prometheus + cAdvisor    | 30s           |
| **Tracing**             | Distributed traces, slowest requests, error traces         | Jaeger                   | On-demand     |

## 🛡️ 14.6 SRE — Site Reliability Engineering do Poker

### 🎯 14.6.1 SLIs, SLOs e SLAs da Plataforma de Poker Online

| Conceito             | Definição                                    | Exemplo                                      |
|----------------------|----------------------------------------------|----------------------------------------------|
| **SLI** (Indicator)  | Métrica que mede confiabilidade              | Disponibilidade = uptime / total             |
| **SLO** (Objective)  | Meta interna de confiabilidade               | 99.9% de disponibilidade/mês                 |
| **SLA** (Agreement)  | Contrato com cliente (com penalidade)        | 99.5% ou reembolso                           |
| **Error Budget**     | 100% - SLO = tolerância de falha             | 99.9% → 43min/mês de downtime permitido      |

### 📈 14.6.2 SLOs do Projeto de Poker Online (Disponibilidade, Latência, Erro)

| Serviço           | SLI                     | SLO (F2)        | SLO (F3)        | Error Budget/mês      |
|-------------------|-------------------------|-----------------|-----------------|-----------------------|
| **API**           | Disponibilidade         | 99.5%           | 99.9%           | 43min → 4.3min        |
| **WebSocket**     | Disponibilidade         | 99.5%           | 99.9%           | 43min → 4.3min        |
| **Latência WS**   | p99 < 200ms             | 95% < 200ms     | 99% < 200ms     | —                     |
| **Auth**          | Disponibilidade         | 99.9%           | 99.95%          | 4.3min → 2.2min       |
| **Pagamentos**    | Disponibilidade         | 99.9%           | 99.99%          | 4.3min → 0.4min       |
| **Hand history**  | Integridade             | 100%            | 100%            | 0 (não tolerável)     |

### 💰 14.6.3 Error Budget Policy do Motor de Poker Online

| Situação                              | Ação                                                    |
|---------------------------------------|---------------------------------------------------------|
| **Error budget saudável**             | Liberar novas features, experimentar                    |
| **Error budget consumindo rápido**    | Pausar features, focar em estabilidade                  |
| **Error budget esgotado**             | Freeze de features, apenas fixes de estabilidade        |
| **Error budget negativo**             | Incidente, postmortem obrigatório                       |

---

# ⚡ 15. STACK ASSÍNCRONO — Tokio e Runtime Rust para Mesas de Poker Concorrentes

> **Princípio:** "Async não é sobre velocidade. É sobre **escala** —
> lidar com milhares de conexões simultâneas com recursos limitados."
>
> O motor de poker precisa manter milhares de mesas ativas simultaneamente,
> cada uma com WebSocket aberto, processando ações em tempo real. Tokio é
> o runtime que torna isso possível em Rust.

## ⚡ 15.1 TOKIO — O Runtime Assíncrono do Poker

### ⚡ 15.1.1 Por que Tokio? — Runtime Assíncrono para Mesas de Poker Concorrentes

| Característica                    | Benefício para Poker                                |
|-----------------------------------|-----------------------------------------------------|
| **Multi-threaded scheduler**      | Usa todos os cores da CPU                           |
| **Work-stealing**                 | Balanceia carga entre threads automaticamente       |
| **I/O não-bloqueante**            | Milhares de conexões sem bloquear threads           |
| **Zero-cost abstractions**        | Overhead mínimo, performance de C                   |
| **Ecosystem maduro**              | Hyper, Tonic, Axum, Tower — todos sobre Tokio       |

### ⚙️ 15.1.2 Configuração do Runtime Tokio para o Motor de Poker

```rust
use tokio::runtime::Runtime;

// Runtime customizado para o motor de poker
fn build_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())        // 1 thread por core
        .max_blocking_threads(512)              // Pool para tarefas bloqueantes
        .enable_all()                           // IO + time
        .thread_stack_size(2 * 1024 * 1024)     // 2MB por thread
        .thread_name("poker-worker")
        .build()
        .expect("Failed to build Tokio runtime")
}
```

### 🎰 15.1.3 Modelo de Concorrência por Mesa de Poker (Tokio Tasks)

```rust
// Cada mesa de poker roda como uma task Tokio isolada
async fn run_table(table: Arc<Table>, mut events: Receiver<PlayerAction>) {
    let table_id = table.id;

    loop {
        tokio::select! {
            // Receber ação de jogador
            Some(action) = events.recv() => {
                let _span = info_span!("table_event", table_id = %table_id);
                table.process_action(action).await;
            }

            // Timer de blind (torneios)
            _ = table.blind_timer.tick() => {
                table.increase_blinds().await;
            }

            // Graceful shutdown
            _ = shutdown_signal() => {
                info!(table_id = %table_id, "Graceful shutdown");
                table.save_state().await;
                break;
            }
        }
    }
}
```

## 🧰 15.2 STACK ASSÍNCRONA COMPLETA — Motor de Poker

| Camada              | Crate                                  | Função                                            |
|---------------------|----------------------------------------|---------------------------------------------------|
| **Runtime**         | `tokio`                                | Executor assíncrono                               |
| **HTTPS server**    | `axum` + Caddy                         | API REST protegida (sobre Hyper + Tokio)          |
| **WebSocket**       | `tokio-tungstenite`                    | Conexões WS em tempo real                         |
| **gRPC**            | `tonic`                                | RPC entre serviços (F3+)                          |
| **Middleware**      | `tower`                                | Composição de middleware (auth, rate limit, tracing)|
| **PostgreSQL**      | `sqlx`                                 | Driver async com compile-time check               |
| **Redis**           | `redis` (async)                        | Cache + pub/sub                                   |
| **Kafka**           | `rdkafka`                              | Event streaming                                   |
| **Serialization**   | `serde` + `serde_json`                 | JSON / Protobuf                                   |
| **Tracing**         | `tracing` + `tracing-subscriber`       | Observabilidade                                   |
| **Metrics**         | `metrics` + `metrics-exporter-prometheus`| Prometheus                                      |
| **Config**          | `config` + `dotenvy`                   | Configuração por ambiente                         |
| **Error handling**  | `thiserror` + `anyhow`                 | Erros tipados + ergonomia                         |

## 🎰 15.3 PADRÕES DE CONCORRÊNCIA PARA POKER — Mesas Simultâneas

### 🎭 15.3.1 Actor Model — Uma Mesa de Poker = Um Actor

```
┌─────────────────────────────────────────────────────────────────┐
│                    MODELO DE ACTORS                              │
│                                                                 │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐                │
│  │  Mesa 1  │     │  Mesa 2  │     │  Mesa 3  │  ... N mesas  │
│  │ (Actor)  │     │ (Actor) │     │ (Actor)  │                │
│  └────┬─────┘     └────┬─────┘     └────┬─────┘                │
│       │                │                │                       │
│       └────────────────┴────────────────┘                       │
│                        │                                        │
│                  ┌─────▼─────┐                                  │
│                  │  Lobby    │                                  │
│                  │  (Router) │                                  │
│                  └─────┬─────┘                                  │
│                        │                                        │
│              ┌─────────┼─────────┐                              │
│              │         │         │                              │
│         ┌────▼───┐ ┌───▼────┐ ┌──▼───┐                          │
│         │ Player │ │ Player │ │Player│  ... M jogadores         │
│         │   1    │ │   2    │ │  3   │                          │
│         └────────┘ └────────┘ └──────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

| Vantagem               | Descrição                                              |
|------------------------|--------------------------------------------------------|
| **Isolamento**         | Cada mesa tem estado próprio, sem locks globais        |
| **Escalabilidade**     | Mesas distribuídas em múltiplos workers                |
| **Tolerância a falhas**| Crash de uma mesa não afeta outras                     |
| **Localidade**         | Estado da mesa na memória do actor, sem DB por ação    |

### 🔄 15.3.2 Backpressure com Channels (mpsc, broadcast) nas Mesas de Poker

```rust
use tokio::sync::mpsc;

// Channel com backpressure (buffer limitado)
let (tx, rx) = mpsc::channel::<PlayerAction>(100);

// Se o buffer encher, o sender espera (backpressure natural)
// Evita OOM e prioriza processamento sobre enfileiramento
```

| Padrão                                      | Quando                                            | Por quê                                |
|----------------------------------------------|---------------------------------------------------|----------------------------------------|
| `mpsc` (multi-producer, single-consumer)     | Múltiplos jogadores → 1 mesa                      | Mesa processa sequencialmente          |
| `broadcast`                                  | 1 mesa → múltiplos observadores (spectators)      | Fan-out para espectadores              |
| `oneshot`                                    | Request-response (ex: validar token)               | Uma resposta única                     |
| `Semaphore`                                  | Limitar conexões concorrentes                      | Proteger recursos                      |
| `Notify`                                     | Sinalizar evento (ex: torneio começa)              | Sem dados, apenas sinal                |

## 🛑 15.4 GRACEFUL SHUTDOWN — Encerramento Seguro do Poker

```rust
async fn graceful_shutdown() {
    let shutdown = tokio::signal::ctrl_c();

    tokio::select! {
        _ = shutdown => {
            info!("Shutdown signal received");

            // 1. Parar de aceitar novas conexões
            listener.stop_accepting();

            // 2. Notificar todas as mesas para salvar estado
            for table in &tables {
                table.save_state().await;
            }

            // 3. Fechar conexões WebSocket gracefully
            for conn in &connections {
                conn.close(CloseCode::AWAY).await;
            }

            // 4. Flush de logs e métricas
            tracing::flush();

            info!("Shutdown complete");
        }
    }
}
```

| Passo                          | Timeout     | Ação se timeout                                    |
|--------------------------------|-------------|----------------------------------------------------|
| Parar aceitar conexões         | Imediato    | —                                                  |
| Salvar estado das mesas        | 10s         | Forçar shutdown (perda de estado em memória)       |
| Fechar WebSockets              | 5s          | Abortar conexões                                   |
| Flush de logs                  | 5s          | Logs em buffer são perdidos                        |

## ⚠️ 15.5 ANTI-PADRÕES ASSÍNCRONOS EM RUST — Motor de Poker

| Anti-padrão                        | Problema                    | Solução                                            |
|------------------------------------|-----------------------------|----------------------------------------------------|
| `.block_on()` em contexto async    | Deadlock                    | `spawn_blocking` para tarefas CPU-bound            |
| `std::sync::Mutex` em async        | Bloqueia executor           | `tokio::sync::Mutex`                               |
| `unwrap()` em futures              | Panic mata task             | `?` + error handling                               |
| Spawn sem `JoinHandle`             | Task órfã                   | Guardar handle, aguardar no shutdown               |
| Loop infinito sem `select!`        | Não responde a shutdown     | `select! { _ = shutdown => break }`                |
| Channel sem backpressure           | OOM                         | Buffer limitado (`mpsc::channel(N)`)               |
| `await` com lock segurado          | Deadlock                    | Soltar lock antes de `.await`                      |

---

# 🔐 16. OWASP CHEAT SHEETS — Aplicadas à Plataforma de Poker Online

> **Princípio:** "A melhor defesa é conhecer os ataques." — Provérbio de
> segurança ofensiva
>
> O OWASP mantém **50+ Cheat Sheets** com soluções práticas para
> vulnerabilidades específicas. Aqui, cada cheat sheet é **adaptado ao
> contexto de uma plataforma de poker online com dinheiro real**.

## 🛡️ 16.1 CHEAT SHEETS MAIS CRÍTICOS PARA POKER — Segurança do Motor

### 🛡️ 16.1.1 Input Validation Cheat Sheet — Validação de Ações do Jogador de Poker

| Regra                                | Aplicação no Poker                                                          |
|--------------------------------------|-----------------------------------------------------------------------------|
| **Whitelist, não blacklist**         | Validar `action_type` contra enum fixo (Fold, Call, Raise, AllIn)           |
| **Validar no servidor**              | Nunca confiar no cliente — re-validar ação, valor, jogador                  |
| **Limites de tamanho**               | `raise_amount` entre min_raise e max_raise (stack do jogador)               |
| **Sanitizar strings**                | Nickname do jogador: `[a-zA-Z0-9_-]{3,20}`                                  |
| **Rejeitar caracteres perigosos**    | Chat de mesa: filtrar XSS, SQLi, comandos                                   |
| **Type checking**                    | `amount: u64` (não `f64` — dinheiro é inteiro em centavos)                  |
| **Range checking**                   | `table_size: 2..=9`, `blinds: 1..=100000`                                   |

```rust
// Exemplo: validação de ação de jogador
fn validate_action(action: &PlayerAction, table: &Table) -> Result<(), ActionError> {
    // 1. Jogador está na mesa?
    if !table.has_player(action.player_id) {
        return Err(ActionError::PlayerNotAtTable);
    }

    // 2. É a vez do jogador?
    if table.current_player() != action.player_id {
        return Err(ActionError::NotYourTurn);
    }

    // 3. Tipo de ação válido?
    match action.action_type {
        ActionType::Fold => Ok(()),
        ActionType::Call => {
            // Validar que call é igual ao que está em jogo
            let to_call = table.to_call(action.player_id);
            if action.amount != Some(to_call) {
                return Err(ActionError::InvalidCallAmount);
            }
            Ok(())
        }
        ActionType::Raise => {
            let amount = action.amount.ok_or(ActionError::MissingAmount)?;
            let min_raise = table.min_raise();
            let max_raise = table.player_stack(action.player_id);
            if amount < min_raise || amount > max_raise {
                return Err(ActionError::RaiseOutOfRange);
            }
            Ok(())
        }
        ActionType::AllIn => Ok(()),
    }
}
```

### 🔑 16.1.2 Authentication Cheat Sheet — Autenticação de Jogadores de Poker

| Regra                          | Implementação no Projeto                                  |
|--------------------------------|----------------------------------------------------------|
| **Senhas com bcrypt**          | `bcrypt 0.16`, cost = 12 (ver `auth.rs`)                 |
| **MFA/TOTP**                   | RFC 6238, janela de ±1 (ver `auth.rs`)                   |
| **JWT com RS256**              | Chave privada no HSM/KMS, não no código                  |
| **Access token curto**         | 15 minutos                                               |
| **Refresh token longo**        | 7 dias, rotacionável                                     |
| **Revogação**                  | Blacklist no Redis (jti)                                 |
| **Rate limiting**              | 5 tentativas de login por IP por minuto                  |
| **Lockout**                    | Após 5 falhas, bloquear 15min + notificar                |
| **Session fixation**           | Regenerar session ID após login                          |
| **No password in logs**        | Nunca logar senha, mesmo hash                            |

### 🎫 16.1.3 Session Management Cheat Sheet — Sessões de Mesa de Poker

| Regra                          | Aplicação                                                    |
|--------------------------------|--------------------------------------------------------------|
| **Session ID aleatório**       | `rand::thread_rng()` com 256 bits (ver `rng_crypto.rs`)      |
| **Session no Redis**           | Não na memória do servidor (stateless)                       |
| **Timeout de sessão**          | 30min de inatividade → logout                                |
| **Concurrent sessions**        | Limitar a 2 dispositivos simultâneos                         |
| **Invalidação no logout**      | Deletar do Redis + revogar JWT                               |
| **Bind ao IP/UA**              | Se IP mudar drasticamente, re-autenticar                     |
| **Secure cookie**              | `Secure`, `HttpOnly`, `SameSite=Strict`                      |

### 💉 16.1.4 SQL Injection Cheat Sheet — Proteção do Banco de Dados do Poker

| Regra                          | Implementação                                                          |
|--------------------------------|------------------------------------------------------------------------|
| **Prepared statements**        | `sqlx` com query parametrizada: `SELECT ... WHERE id = $1`             |
| **Nunca concatenar SQL**       | Proibido `format!("SELECT ... WHERE name = '{}'", name)`              |
| **ORM ou query builder**       | `sqlx::query!` valida em compile-time                                  |
| **Least privilege**            | Usuário do DB só pode SELECT/INSERT/UPDATE nas tabelas necessárias     |
| **No dynamic table names**     | Nunca usar input para nome de tabela                                   |

```rust
// ❌ VULNERÁVEL
let query = format!("SELECT * FROM players WHERE nickname = '{}'", nickname);
sqlx::query(&query).fetch_all(&pool).await?;

// ✅ SEGURO (prepared statement)
let players = sqlx::query_as::<_, Player>(
    "SELECT * FROM players WHERE nickname = $1"
).bind(&nickname).fetch_all(&pool).await?;
```

### 🕷️ 16.1.5 XSS (Cross-Site Scripting) Cheat Sheet — Proteção do Frontend Dioxus do Poker

| Regra                          | Aplicação no Poker                                              |
|--------------------------------|----------------------------------------------------------------|
| **Output encoding**            | Dioxus escapa HTML automaticamente (framework seguro)          |
| **Content Security Policy**    | Header `Content-Security-Policy: default-src 'self'`           |
| **No inline scripts**          | Scripts externos apenas                                        |
| **Sanitizar chat**             | Filtrar `<script>`, `onerror=`, `javascript:`                  |
| **HttpOnly cookies**           | JS não acessa cookies de sessão                                |
| **Trusted Types**              | Para Dioxus, usar API de DOM segura                            |

### 🎭 16.1.6 CSRF (Cross-Site Request Forgery) Cheat Sheet — Proteção das Ações do Jogador

| Regra                          | Implementação                                                |
|--------------------------------|--------------------------------------------------------------|
| **Token CSRF**                 | Por sessão, em header `X-CSRF-Token`                         |
| **SameSite cookies**           | `SameSite=Strict` para cookies de sessão                     |
| **Origin/Referer check**       | Validar header `Origin` no servidor                          |
| **Double submit cookie**       | Token em cookie + header, comparar no servidor               |
| **No GET para mutações**       | Ações de estado (saque, ban) só via POST/PUT/DELETE          |

### 🔌 16.1.7 API Security Cheat Sheet — Segurança da API Axum do Poker Online

| Regra                              | Aplicação                                                    |
|------------------------------------|--------------------------------------------------------------|
| **Auth em todos endpoints**        | Exceto `/health` e `/login`                                  |
| **Rate limiting**                  | 100 req/min por token, 1000 por IP                           |
| **Input validation**               | Schema validation com `serde` + `validator`                  |
| **Output encoding**                | JSON com `serde_json`, nunca concatenar                      |
| **No sensitive data in response**  | Não retornar hash de senha, saldo de outros                  |
| **CORS whitelist**                 | Apenas origens permitidas                                    |
| **HTTPS only**                     | Redirecionar tráfego inseguro → HTTPS, HSTS header            |
| **API versioning**                 | `/api/v1/...` para backward compat                           |
| **Pagination**                     | Limitar resultados (max 100 por página)                      |
| **Idempotency keys**               | Para POST de saque/deposito (evitar duplicação)              |

### 🔐 16.1.8 Cryptographic Storage Cheat Sheet — Criptografia de Dados de Jogadores

| Dado                       | Algoritmo              | Justificativa                                |
|----------------------------|------------------------|----------------------------------------------|
| **Senha**                  | bcrypt cost 12         | Slow hash, resistente a GPU/ASIC             |
| **Token secreto TOTP**     | AES-256-GCM            | Simétrico, rápido, autenticado               |
| **Hand history**           | AES-256-GCM at-rest    | Dados sensíveis de jogo                      |
| **Dados de pagamento**     | AES-256-GCM + field-level | Criptografar mesmo no DB                  |
| **Chave privada JWT**      | RSA 2048+ ou Ed25519   | Assimétrica para assinar tokens              |
| **Saldo de jogadores**     | AES-256-GCM            | Dado financeiro crítico                      |
| **Logs de auditoria**      | Hash chain (SHA-256)   | Imutabilidade verificável                    |

### 🌐 16.1.9 Transport Layer Protection Cheat Sheet — TLS nas Mesas de Poker

| Regra                          | Implementação                                                    |
|--------------------------------|------------------------------------------------------------------|
| **TLS 1.3 only**               | `rustls` com `ProtocolVersion::TLSv1_3`                          |
| **No TLS 1.0/1.1**             | Desativado (deprecado, vulnerável)                               |
| **HSTS**                       | `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`|
| **Certificate pinning**        | Para app mobile (se aplicável)                                   |
| **No mixed content**           | Tudo HTTPS, sem recursos inseguros                                  |
| **Forward secrecy**            | TLS 1.3 tem por padrão (ECDHE)                                   |

### ⚠️ 16.1.10 Error Handling Cheat Sheet — Tratamento de Erros do Motor de Poker

| Regra                          | Aplicação                                                              |
|--------------------------------|------------------------------------------------------------------------|
| **No stack traces em prod**    | Logar internamente, retornar mensagem genérica                         |
| **Error codes**                | `POKER_ERR_001`, `AUTH_ERR_042` para suporte                           |
| **No sensitive info**          | Não revelar se email existe (use "credenciais inválidas")              |
| **Structured errors**          | `{"error": {"code": "...", "message": "...", "details": {...}}}`     |
| **Fail secure**                | Em caso de erro, negar acesso (não permitir)                           |

## 📋 16.2 CHEAT SHEETS ADICIONAIS (Resumo) — Poker Online

| Cheat Sheet                           | Aplicação Resumida                                                      |
|---------------------------------------|-------------------------------------------------------------------------|
| **Denial of Service**                 | Rate limiting, WAF, auto-scaling, circuit breaker                       |
| **Insecure Direct Object Reference**  | Validar ownership (jogador só vê suas mãos)                             |
| **Security Headers**                  | CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy           |
| **File Upload**                       | Validar tipo, tamanho, scan de malware, storage separado                |
| **Deserialization**                   | `serde` com tipos estritos, nunca `serde_json::Value` sem validação     |
| **Logging**                           | Structured, sem dados sensíveis, retenção definida                      |
| **Threat Modeling**                   | STRIDE por feature (Seção 4)                                            |
| **Docker Security**                   | Imagem mínima, non-root, read-only, scan (Seção 12)                     |
| **Secrets Management**                | Variáveis de ambiente, Vault, nunca no código                           |
| **Dependency Management**             | `cargo audit`, `cargo deny`, `cargo outdated`                           |
| **Mobile Security**                   | Certificate pinning, root/jailbreak detection (se mobile)               |
| **WebSocket Security**                | Origin check, auth no handshake, rate limit por conexão                 |
| **Race Conditions**                   | `tokio::sync::Mutex`, transações atômicas no DB                         |
| **Business Logic**                    | Validar regras de poker no servidor (não confiar no cliente)            |

## 🗺️ 16.3 MAPEAMENTO OWASP TOP 10 → CHEAT SHEETS — Poker

| OWASP Top 10 (2021)                | Cheat Sheet Principal                  | Seção no Projeto       |
|------------------------------------|----------------------------------------|------------------------|
| **A01: Broken Access Control**     | Authorization, IDOR                    | Seção 4 (WSTG-ACLZ)    |
| **A02: Cryptographic Failures**    | Cryptographic Storage, TLS             | Seção 2 + 15.1.8       |
| **A03: Injection**                 | SQL Injection, Input Validation        | 15.1.1 + 15.1.4        |
| **A04: Insecure Design**           | Threat Modeling, Secure Design         | Seção 4                |
| **A05: Security Misconfiguration** | Docker, IaC, Headers                   | Seção 12 + 15.2        |
| **A06: Vulnerable Components**     | Dependency Management                  | 15.2 + Seção 12        |
| **A07: Auth Failures**             | Authentication, Session                | 15.1.2 + 15.1.3        |
| **A08: Data Integrity Failures**   | Deserialization, CI/CD                 | 15.2 + Seção 12        |
| **A09: Logging Failures**          | Logging, Audit                         | Seção 13 + 15.1.10     |
| **A10: SSRF**                      | API Security, Network                  | 15.1.7                 |

---

# 📜 17. COMPLIANCE REGULATÓRIO — UK Gambling Commission e Jurisdições de Poker

> **Princípio:** "Em jogos com dinheiro real, a regulamentação não é um
> obstáculo — é a **licença para operar**. Sem compliance, não há negócio."
>
> Uma plataforma de poker online opera em um dos setores mais regulados
> do mundo. Esta seção mapeia os requisitos regulatórios para a
> **UK Gambling Commission** (referência internacional), **Malta Gaming
> Authority** (jurisdição popular para poker), e o **RGPS brasileiro**
> (regulamento nacional em evolução).

## 🌐 17.1 JURISDIÇÕES E LICENÇAS — Poker Online Internacional

### 🌍 17.1.1 Comparativo de Jurisdições para Licença de Poker Online

| Jurisdição      | Autoridade                        | Licença              | Custo Anual            | Reputação              | Tempo          |
|-----------------|-----------------------------------|----------------------|------------------------|------------------------|----------------|
| **UK**          | UK Gambling Commission            | Operating Licence    | £5.000-£50.000+        | ⭐⭐⭐⭐⭐              | 6-12 meses     |
| **Malta**       | Malta Gaming Authority (MGA)      | MGA/B2B ou B2C       | €25.000-€50.000        | ⭐⭐⭐⭐               | 4-6 meses      |
| **Curaçao**     | Curaçao Gaming Control Board      | Master License       | $2.000-$25.000         | ⭐⭐                  | 2-3 meses      |
| **Gibraltar**   | Gibraltar Gambling Commissioner   | Remote License       | £10.000+               | ⭐⭐⭐⭐⭐              | 6-12 meses     |
| **Ilha de Man** | GSC Isle of Man                   | OGRA License         | £5.000+                | ⭐⭐⭐⭐               | 4-6 meses      |
| **Brasil**      | SPA (Secretaria de Prêmios)       | Licença (novo)       | R$30 milhões           | ⭐⭐⭐ (em construção)| TBD            |

### 🗺️ 17.1.2 Estratégia de Licenciamento por Fase do Poker Online

| Fase   | Jurisdição               | Justificativa                                    |
|--------|--------------------------|--------------------------------------------------|
| **F1-F2** | Curaçao (provisória)   | Rápido, barato, para validar produto             |
| **F3**    | Malta (MGA B2C)        | Reconhecida na UE, custo razoável                |
| **F4**    | UK Gambling Commission | Mercado mais rigoroso e lucrativo                |
| **F4+**   | Brasil (RGPS)          | Mercado nacional em regulamentação               |

## 🇬🇧 17.2 UK GAMBLING COMMISSION — Requisitos Detalhados do Poker

### 🇬🇧 17.2.1 Licença UKGC Necessária para Poker Online no Reino Unido

| Tipo                                      | Quando                                    |
|--------------------------------------------|-------------------------------------------|
| **Operating Licence**                      | Para operar a plataforma                  |
| **Personal Management Licence (PML)**      | Para diretores e gerentes seniores        |
| **Premises Licence**                       | Não aplicável (online)                    |

### 🔧 17.2.2 Requisitos Técnicos (Remote Technical Standards) do Motor de Poker

| Requisito                       | Implementação no Projeto                                                       |
|---------------------------------|-------------------------------------------------------------------------------|
| **RNG certificado**             | `rng_crypto.rs` com `OsRng` (CSPRNG), auditoria por laboratório independente  |
| **RTP reportable**              | Hand history completa, replay de qualquer mão                                 |
| **Audit trail**                 | Logs imutáveis de todas as ações (Seção 14.4.3)                               |
| **Player funds protection**     | Conta segregada, não misturar com operacional                                 |
| **Self-exclusion**              | GAMSTOP integration, auto-exclusão configurável                               |
| **Reality check**               | Pop-up a cada 30min mostrando tempo e perdas                                  |
| **Time-out**                    | Pausa de 24h-6 semanas                                                        |
| **Deposit limits**              | Configurável pelo jogador, obrigatório                                        |
| **Age verification**            | KYC obrigatório antes de depositar                                            |
| **Location verification**       | Geo-block para jurisdicões não licenciadas                                    |

### 🎯 17.2.3 Requisitos de Jogo Responsável (Responsible Gambling) do Poker Online

| Recurso                  | Descrição                                      | Implementação                                      |
|--------------------------|------------------------------------------------|----------------------------------------------------|
| **Self-exclusion**       | Jogador auto-exclui por 6 meses a permanente   | `self_exclusion` table, bloqueio no login          |
| **Time-out**             | Pausa temporária (24h-6 semanas)               | `time_out` table, auto-reativação                  |
| **Deposit limits**       | Limite diário/semanal/mensal                   | `deposit_limit` table, enforcement no pagamento    |
| **Loss limits**          | Limite de perda por sessão                     | Tracking de P&L, bloqueio ao atingir               |
| **Reality check**        | Pop-up periódico                               | WebSocket push a cada 30min                        |
| **Session timer**        | Mostrar tempo de sessão                        | Display no cliente, tracking no servidor           |
| **Cooling-off**          | 24h após registro antes de depositar           | `created_at + 24h` check                           |
| **Self-assessment**      | Questionário de risco                          | Modal no cliente                                   |
| **Links de ajuda**       | BeGambleAware, GamCare, GAMSTOP                | Footer + página dedicada                           |
| **Activity statements**  | Histórico de jogo exportável                   | API `/me/activity`                                 |

### 🪪 17.2.4 AML (Anti-Money Laundering) e KYC de Jogadores de Poker

| Requisito                       | Implementação                                                    |
|---------------------------------|------------------------------------------------------------------|
| **KYC obrigatório**             | Verificar identidade antes de depositar                          |
| **Verificação de documento**    | Passaporte/RG + selfie + comprovante de endereço                 |
| **PEP check**                   | Verificar se é Pessoa Exposta Politicamente                      |
| **Sanctions check**             | Verificar contra listas OFAC, UE, ONU                            |
| **Source of funds**             | Para depósitos > €2.000, comprovar origem                        |
| **Transaction monitoring**      | Detectar padrões suspeitos (antifraude)                          |
| **SAR (Suspicious Activity Report)**| Reportar à UK FIU se necessário                              |
| **Record keeping**              | 7 anos de registros de AML                                       |

### 📋 17.2.5 Requisitos de Auditoria do Motor de Poker (RNG, RTP, Hand History)

| Auditoria                   | Frequência          | Quem                                      |
|-----------------------------|---------------------|-------------------------------------------|
| **RNG certification**       | Anual               | Laboratório independente (eCOGRA, GLI)    |
| **Security audit**          | Anual               | Firma externa (pentest)                   |
| **Financial audit**         | Anual               | Auditor contábil                          |
| **Compliance audit**        | Anual               | UK GC ou representante                    |
| **Technical standards**     | Por release maior   | Laboratório independente                  |
| **AML audit**               | Bienal              | Compliance officer                        |

## 🇧🇷 17.3 RGPS — Regulamento Brasileiro (Em Construção) do Poker

### 🇧🇷 17.3.1 Contexto do Poker Online no Brasil (Zona Cinza)

| Aspecto              | Status (2025)                              |
|----------------------|--------------------------------------------|
| **Lei**              | Lei 14.790/2023 (jogos online)             |
| **Regulador**        | SPA (Secretaria de Prêmios e Apostas)      |
| **Taxa**             | 12% sobre receita bruta                    |
| **Licença**          | R$30 milhões por 5 anos                    |
| **KYC**              | Obrigatório                                |
| **Publicidade**      | Restrita (Lei 10.671/2023)                 |
| **Patrocinadores**   | Proibido patrocinar times esportivos       |

### 🔧 17.3.2 Requisitos Técnicos (Previstos) para Poker Online no Brasil

| Requisito                          | Status        | Implementação                            |
|------------------------------------|---------------|------------------------------------------|
| **Servidores no Brasil**           | Obrigatório   | Data center BR (AWS São Paulo)           |
| **RNG certificado**                | Obrigatório   | `rng_crypto.rs` + auditoria              |
| **Dados de jogadores no Brasil**   | Obrigatório   | LGPD compliance                          |
| **Relatórios regulatórios**        | Mensal        | Dashboard automatizado                   |
| **Jogo responsável**               | Obrigatório   | Seção 17.2.3 adaptada                    |
| **AML**                            | Obrigatório   | COAF reporting                           |

## 🔒 17.4 LGPD (Brasil) e UK GDPR — Proteção de Dados do Poker

### 📜 17.4.1 Princípios Comuns de Compliance (LGPD/GDPR) para Poker Online

| Princípio                  | Aplicação                                                    |
|----------------------------|--------------------------------------------------------------|
| **Finalidade**             | Dados só para propósito declarado (jogo de poker)            |
| **Adequação**              | Compatível com o propósito                                   |
| **Necessidade**            | Coletar apenas o necessário                                  |
| **Livre acesso**           | Jogador pode ver seus dados                                  |
| **Qualidade**              | Dados precisos e atualizados                                 |
| **Transparência**          | Política de privacidade clara                                |
| **Segurança**              | Criptografia, access control (Seção 2)                       |
| **Prevenção**              | Evitar vazamento                                             |
| **Não discriminação**      | Não usar dados para discriminar                              |
| **Responsabilização**      | Demonstrar compliance                                        |

### 👤 17.4.2 Direitos do Titular (DSAR) — Dados de Jogadores de Poker

| Direito            | Implementação                                                    |
|--------------------|------------------------------------------------------------------|
| **Acesso**         | API `/me/data` exporta todos os dados                           |
| **Retificação**    | API para corrigir dados                                         |
| **Eliminação**     | Soft delete + retenção mínima legal (7 anos para AML)           |
| **Portabilidade**  | Export em JSON/CSV                                              |
| **Oposição**       | Opt-out de marketing                                            |
| **Limitação**      | Congelar processamento                                          |

## 🗺️ 17.5 ROADMAP DE COMPLIANCE POR FASE — Poker Online

| Fase     | Compliance                                                    | Custo Estimado            |
|----------|---------------------------------------------------------------|---------------------------|
| **F1**   | LGPD básico, termos de uso, política de privacidade           | R$5.000 (advogado)        |
| **F2**   | KYC/AML básico, self-exclusion, RGPS preparação               | R$50.000                  |
| **F3**   | Licença Curaçao/Malta, RNG cert, audit externo                | €50.000-€100.000          |
| **F4**   | Licença UK GC, AML completo, auditoria anual                  | £100.000+/ano             |
| **F4+**  | Licença BR (RGPS), data center BR                             | R$30 milhões + custos     |

---

# 🗺️ 18. CONTEXTO DO PROJETO — Como Tudo se Conecta na Plataforma de Poker

> **Princípio:** "Um documento de qualidade não vive isolado. Ele é o
> **mapa** que conecta todos os artefatos do projeto — código, testes,
> documentação, infraestrutura e processos."

## 🗺️ 18.1 MAPA DE ARTEFATOS DO PROJETO — Plataforma de Poker

```
┌─────────────────────────────────────────────────────────────────┐
│                  ECOSSISTEMA DE DOCUMENTOS                       │
│                                                                 │
│                    ┌──────────────┐                              │
│                    │  QUALITY.md  │ ← Você está aqui (mestre)    │
│                    └──────┬───────┘                              │
│                           │                                      │
│          ┌────────────────┼────────────────┐                     │
│          │                │                │                     │
│  ┌───────▼──────┐  ┌──────▼───────┐  ┌────▼────────┐            │
│  │ README.md    │  │ DASHBOARD.md │  │ CRONOGRAMA  │            │
│  │ (visão geral)│  │ (métricas)   │  │ .md (prazos)│            │
│  └──────────────┘  └──────────────┘  └─────────────┘            │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐            │
│  │ BUSINESS_    │  │ DEVELOPMENT  │  │ ARQUITETURA │            │
│  │ RULES.md     │  │ _LOG.md      │  │ _MOTOR.md   │            │
│  └──────────────┘  └──────────────┘  └─────────────┘            │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │ guia_aprendi │  │ TESTING_     │                            │
│  │ zagem.md     │  │ GOALS.md     │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## 📂 18.2 PAPEL DE CADA DOCUMENTO — Poker Project

| Documento                        | Propósito                                                    | Quando Consultar                    |
|----------------------------------|--------------------------------------------------------------|-------------------------------------|
| **QUALITY.md** (este)            | Mestre — qualidade, segurança, negócio, arquitetura          | Sempre (referência contínua)        |
| **README.md**                    | Visão geral do projeto, setup, como rodar                    | Onboarding                          |
| **CRONOGRAMA.md**                | Prazos, milestones, roadmap temporal                         | Planejamento                        |
| **BUSINESS_RULES.md**            | Regras de negócio do poker (rake, blinds, side pots)         | Ao implementar lógica de jogo       |
| **DASHBOARD.md**                 | Métricas de negócio e técnicas, KPIs                         | Revisões semanais                   |
| **ARQUITETURA_MOTOR.md**         | Arquitetura detalhada do motor Rust                          | Ao mexer no motor                   |
| **DEVELOPMENT_LOG.md**           | Log de desenvolvimento, decisões técnicas                    | Histórico de mudanças               |
| **guia_aprendizado.md**          | Guia de aprendizado consolidado (Protocolo Mark, módulos)    | Estudo                              |
| **TESTING_GOALS.md**             | Metas de testes por módulo e fase                             | Planejamento de testes              |

## 🗂️ 18.3 ESTRUTURA DE PASTAS E PROPÓSITO — Poker Online

| Pasta                          | Propósito                          | Conteúdo Principal                                      |
|--------------------------------|------------------------------------|---------------------------------------------------------|
| `Motor-Rust/`                  | Motor de poker em Rust             | `src/` (10 módulos + 4 antifraude), `tests/`            |
| `Frontend-Web/`                | Frontend canônico (React/Vite)     | `src/`, `Dockerfile`, Tailwind                          |
| `API-Axum/`                    | API HTTPS/WSS                      | `src/` (handlers, middleware, state), `migrations/`     |
| `Infraestrutura-Docker/`       | Infraestrutura como código         | `docker-compose.yml`, `Caddyfile`                       |
| `Documentacao/`                | Documentação do projeto            | Todos os `.md` de documentação                          |
| `Arquitetura-Motor/`           | Arquitetura do motor               | `ARQUITETURA_MOTOR.md`                                  |
| `scripts/`                     | Scripts de automação               | `deploy.*`, `full-validation.*`, `coverage.*`, live e2e |

## 📖 18.4 COMO USAR ESTE DOCUMENTO — Guia do Poker Project

### 👥 18.4.1 Por Persona (Jogador, Operador, Desenvolvedor, Investidor)

| Persona                    | Seções Prioritárias                                                                          |
|----------------------------|----------------------------------------------------------------------------------------------|
| **Desenvolvedor**          | 2 (Stack), 3 (Testes), 11 (Arquitetura), 14 (Tokio), 15 (Cheat Sheets)                      |
| **QA/Tester**              | 3 (Pirâmide de Testes), 4 (OWASP WSTG), 12 (DevSecOps)                                      |
| **Segurança/Pentester**    | 4 (Hacker Ético), 12 (DevSecOps), 15 (Cheat Sheets), 16 (Compliance)                        |
| **Empreendedor/CEO**       | 5 (Plano de Negócio), 6 (Financeiro), 7 (Marketing), 8 (OKRs)                               |
| **Gestor de Projeto**      | 8 (OKRs), 8-BIS (Gestão de Tempo), 17 (Contexto)                                            |
| **Compliance Officer**     | 16 (Regulatório), 13 (Observabilidade), 12 (DevSecOps)                                      |
| **SRE/DevOps**             | 13 (Observabilidade), 14 (Tokio), 12 (DevSecOps)                                            |

### 🎯 18.4.2 Por Situação (Onboarding, Mesa Ativa, Disputa, Auditoria)

| Situação                            | Seção                                                              |
|-------------------------------------|--------------------------------------------------------------------|
| **Começando uma nova feature**      | 11 (Arquitetura) + 4 (Threat Modeling) + 3 (Testes)                |
| **Investigando um bug**             | 13 (Observabilidade) + 14 (Tokio)                                  |
| **Preparando release**              | 12 (DevSecOps pipeline) + 3 (Testes E2E)                           |
| **Respondendo a incidente**         | 13.6 (SRE) + 12.5 (Runtime Protection)                             |
| **Planejando roadmap**              | 8 (OKRs) + 8-BIS (Gestão) + 16 (Compliance)                        |
| **Auditando segurança**             | 4 (OWASP WSTG) + 15 (Cheat Sheets) + 12 (DSOMM)                    |
| **Onboarding de novo dev**          | 17 (Contexto) + 2 (Stack) + 9 (IA Practices)                       |

## ✅ 18.5 CHECKLIST DE QUALIDADE — Antes de Cada Release do Poker

### ⚙️ 18.5.1 Checklist Técnico do Motor de Poker (Rust, Tokio, Dioxus, Axum)

- [ ] `cargo test --lib` — 100% passing (1.813 testes determinísticos no Motor-Rust)
- [ ] Perfil autorizado — 1.892 testes do motor, mais cargas de API HTTPS, WSS e frontend registradas em `FULL_VALIDATION.md`
- [ ] `cargo clippy` — 0 warnings (validado: `cargo clippy --all-targets -- -D warnings`)
- [ ] `cargo fmt --check` — formatado
- [ ] `cargo audit` — 0 CVEs conhecidos
- [ ] `cargo deny check` — licenças OK
- [ ] Cobertura ≥ 80% (`cargo-tarpaulin` ou `cargo-llvm-cov`)
- [ ] `cargo fuzz` — 0 crashes em 5min
- [ ] `cargo mutants` — mutation score ≥ 80%
- [ ] Load test (k6/locust) — suporta target de fase
- [ ] OWASP ZAP scan — 0 alertas high/critical
- [ ] Trivy container scan — 0 vulnerabilidades high
- [ ] Secrets scan (gitleaks) — 0 secrets no código

### 🔐 18.5.2 Checklist de Segurança da Plataforma de Poker Online

- [ ] Threat model atualizado para novas features
- [ ] Pen test executado (interno ou externo)
- [ ] JWT expiração configurada corretamente
- [ ] Rate limiting ativo em todos endpoints
- [ ] CORS whitelist revisada
- [ ] Logs de auditoria funcionando e imutáveis
- [ ] MFA funcionando
- [ ] RBAC testado (cada role só acessa o permitido)
- [ ] Criptografia at-rest e in-transit verificada
- [ ] Secrets em variáveis de ambiente (não no código)

### 💼 18.5.3 Checklist de Negócio do Poker Online (Rake, KPIs, Marketing)

- [ ] Rake calculado corretamente em todos os cenários
- [ ] Side pots calculados corretamente
- [ ] Loss Deflator funcionando
- [ ] Hand history completa e replayable
- [ ] Antifraude rodando em todas as mesas
- [ ] Self-exclusion e limites de depósito funcionando
- [ ] Reality check pop-up aparecendo a cada 30min
- [ ] KYC obrigatório antes de depositar
- [ ] Dashboard executivo atualizado
- [ ] OKRs revisados (trimestralmente)

### ⚖️ 18.5.4 Checklist de Compliance do Poker Online (UKGC, LGPD, MGA)

- [ ] Logs de AML retidos por 7 anos
- [ ] Auditoria de RNG atualizada (anual)
- [ ] Política de privacidade atualizada
- [ ] Termos de uso atualizados
- [ ] Links de jogo responsável visíveis
- [ ] GAMSTOP integrado (se UK)
- [ ] Geo-block funcionando para jurisdicões não licenciadas
- [ ] DSAR (Data Subject Access Request) funcional

## 🏆 18.6 PRINCÍPIOS FINAIS — O Decálogo da Qualidade do Poker

| #  | Princípio                                      | Aplicação                                            |
|----|------------------------------------------------|------------------------------------------------------|
| 1  | **Segurança é processo, não produto**          | Praticar DevSecOps contínuo (Seção 12)               |
| 2  | **Testes são seguro de vida**                  | Pirâmide de testes, 484+ testes (Seção 3)            |
| 3  | **Observabilidade é transparência**            | Métricas, logs, tracing sempre on (Seção 13)         |
| 4  | **Compliance é licença para operar**           | UK GC, RGPS, LGPD (Seção 16)                         |
| 5  | **O cliente confia no RNG**                    | CSPRNG auditável (Seção 2)                           |
| 6  | **Fraude é inevitável, detecção é obrigatória**| 4 módulos antifraude (Seção 4)                       |
| 7  | **Performance é experiência**                  | Tokio, latência < 200ms (Seção 14)                   |
| 8  | **Arquitetura evolui, não nasce pronta**       | Monolito modular → microserviços (Seção 11)          |
| 9  | **Dados são o ativo mais valioso**             | Hand history imutável, LGPD, 7 anos (Seção 13)       |
| 10 | **Qualidade é cultura, não checklist**         | Este documento é vivo, atualizado sempre             |

---

> **FIM DO DOCUMENTO**
>
> *QUALITY.md — O documento mestre da Plataforma de Poker Online.*
> *Última atualização: 2026-07-08*
> *Mantido por: Leo Frigon (GitHub Copilot)*
> *Status: Vivo — atualizar a cada release e revisão trimestral*

---


<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-01):** S20d — Polimento total: barra felt/gold, bloco Zero Tilt sem Full Tilt, H2 fontes e sem A♠; catálogo cash canônico NLHE 0,25/0,25, Hold'em Short Deck 0,25/0,50 e Omaha Short Deck 0,50/0,50 (Play Money e Jogo Real). **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy; migrations 001–032 aplicadas. Gate S20d: cargo fmt, Clippy estrito, tsc -b + Vite 60 módulos — todos sem falhas; VPS 4/4 healthy, 26 níveis ante=big_blind (invalid_BBA=0), backup verificável, rebuilds e health público OK. Frontend: PT-BR + Dica do Pró + história 8+7 com fontes H2 2006 + vazios com história + sem painel duplicado + sem A♠ (case-sensitive) + scrollbar felt/gold + bloco Zero Tilt sem Full Tilt + notícias com foto oficial (sem placeholder). A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
