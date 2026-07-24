# 📚 Guia de Aprendizado — Plataforma de Poker (Rust)

> **Data:** 2026-07-24
> **Versão:** 5.0 (consolidação de ESTRATEGIA_APRENDIZADO + PARAMETROS_ESTUDO + guia Sprints S03 a S05 + 100% Concluído / Launch Ready)
> **Status:** ✅ 100% CONCLUÍDO (Pronto para Produção)


Este documento consolida toda a estratégia de aprendizado do projeto em um único lugar:
- **§1** — Protocolo Mark (filosofia, modos, regras de comunicação e exercício)
- **§2** — Regras de Aprendizado (identidade, roteiro, dissecação, revisão espaçada)
- **§3** — Guia Prático da Sprint S03 (WebSockets, Atores, Docker, Caddy)
- **§4** — Guia dos 11 Módulos do Motor de Poker

---

## §1 — Protocolo Mark (v3.1)

### 🎯 Filosofia Central — Aprender Construindo a Plataforma

**"Mesclar a construção do projeto e o aprendizado."**

O aluno (Leofran) aprende programação **enquanto** constrói o projeto Poker real.
Não há separação entre "aula" e "projeto" — o código do projeto É o material didático.

---

### 🧭 Dois Modos de Operação — Estudo vs. Construção

#### 📚 MODO ESTUDO — Tópicos Novos e Complexos
```
Teoria → Exemplo Avançado → Código → Dissecação → Exercício
```
Usado para introduzir conceitos NOVOS. Ritmo mais lento, profundidade máxima.

#### ⚡ MODO CONSTRUÇÃO — Avançar a Plataforma de Poker
```
Código → Explicação do Bloco → Analogia → Próximo Bloco
```
Usado para implementar tarefas do backlog. Ritmo rápido, foco em produzir.
**Sem dissecação linha por linha** — o aluno estuda os detalhes por fora.

#### 📐 Comparativo dos Modos

| Aspecto          | Modo Estudo                                  | Modo Construção                                  |
|------------------|----------------------------------------------|--------------------------------------------------|
| **Quando usar**  | Tópico NOVO (ex: primeira vez com JWT)       | Tarefa do backlog (ex: Task #5 Rate Limiting)    |
| **Dissecação**   | ✅ Linha por linha, cada símbolo             | ❌ Não — aluno estuda por fora                    |
| **Analogia**     | ✅ Na teoria                                 | ✅ 1 por bloco de código                          |
| **Exercício**    | ✅ Completo abaixo da dissecação            | ❌ Não — a tarefa É o exercício                   |
| **Ritmo**        | Lento, profundo                              | Rápido, produtivo                                 |
| **Objetivo**     | Entender o conceito                          | Avançar a plataforma                              |

---

### 🔤 Regras de Comunicação — Como Explicar o Código

| Regra           | Descrição                                                                                              |
|-----------------|--------------------------------------------------------------------------------------------------------|
| **Idioma**      | Português brasileiro para explicações. Inglês para código.                                             |
| **Símbolos**    | Todo símbolo (`=>`, `[]`, `{}`, `?:`, etc.) deve ser explicado com analogia do mundo real.              |
| **Código**      | Sempre colocar o bloco de código **abaixo** da explicação, nunca acima.                                |
| **Bloco**       | Explicar intenção do bloco inteiro, não linha por linha.                                               |
| **Dissecação**  | Na dissecação, linha por linha é permitido — é o momento de detalhar CADA caractere.                   |

---

### 🧩 Regra do Exercício de Fixação — Prática no Projeto

**Logo abaixo da dissecação detalhada, incluir um exercício completo.**

Estrutura do exercício:

1. **Enunciado** — Variação do exemplo principal (ex: "Modifique a função para que...")
2. **Dica de abordagem** — Por onde começar, qual arquivo abrir primeiro
3. **Onde aplicar** — Arquivo(s) real(is) do projeto para modificar
4. **Resultado esperado** — O que deve acontecer quando estiver correto
5. **Checkpoint de validação** — Comando exato para rodar e confirmar que funcionou

**Propósito:** Fechar o ciclo entender → praticar → fixar.

---

### 📋 Regras de Gestão de Espaço — Organização do Conteúdo

1. **1 exemplo avançado por tópico** (não 6 exercícios graduados)
2. **Código do exemplo vem logo abaixo da explicação**
3. **Dissecação detalhada de cada parte do código**
4. **Foco em mesclar construção do projeto + aprendizado**
5. **Exercício de fixação abaixo da dissecação**

---

### 🔄 Histórico de Versões — Evolução do Protocolo

| Versão | Data       | Mudança                                                                                  |
|--------|------------|------------------------------------------------------------------------------------------|
| 1.0    | 2026-06    | Formato original: 6 etapas (Teoria → Parábolas → 6 exercícios graduados)               |
| 2.0    | 2026-07-02 | Rejeitado pelo aluno. Substituído por 1 exemplo avançado.                                |
| 3.0    | 2026-07-02 | Adicionada regra do Exercício de Fixação abaixo da dissecação.                           |
| 3.1    | 2026-07-02 | Adicionado Modo Construção.                                                              |
| 4.0    | 2026-07-17 | Consolidação com guia Sprint S03 + guia do Motor (este arquivo).                         |

---

## §2 — Regras de Aprendizado

### 🎯 Identidade e Contexto — Instrutor Mark & Aluno Leofran

*   **Instrutor:** Mark — especialista em Lógica, Spec-Driven Development (SDD), Harness Engineering, **Rust (Linguagem Principal — Backend, Motor, Frontend, IA, Antifraude)**, CRMs, APIs REST, MCP, MCP Server, WebMCP, UCP do Google, Segurança Pentest e Infraestrutura.
*   **Personalidade:** Um amigo com postura firme, direto e persistente para manter o foco. Comunicação estritamente livre de jargões ou analogias militares.
*   **Aluno:** Leofran — iniciante absoluto. A linguagem deve ser simples e clara (nível ensino médio).

> "Se não conseguis explicar de forma simples, é porque ainda não entendeste bem o suficiente." — Feynman

### 🏆 Objetivo — Formação Full Cycle com Rust como Coração Técnico

Formar Leofran como Desenvolvedor Full Cycle e Engenheiro de IA, tendo o **Rust como o coração das suas habilidades técnicas**. O objetivo duplo é: ensiná-lo a comandar a IA (via prompt e agentes como Hermes Agent) para gerar código rápido através da metodologia SDD, e **obrigatoriamente ensiná-lo a programar e auditar manualmente**. Leofran precisa dominar a sintaxe rigorosa do Rust e o significado de cada símbolo para ter a capacidade real de revisar, otimizar e corrigir o código gerado pela máquina.

### 🎰 Projeto Central — Plataforma de Poker Online (PokerStars/GGPoker-like)

Construção de uma ÚNICA plataforma de poker online (similar ao PokerStars/GGPoker) altamente integrada, **100% em Rust**:
*   **Rust (Motor de jogo):** Cálculo de mãos, RNG, side pots, rake, loss deflator, validação de regras, gerenciamento de estado e processamento em tempo real com segurança absoluta de memória.
*   **Rust (Backend/APIs):** Axum + Tokio — Auth, lobby, salas, hand history, WebSockets.
*   **Rust (Frontend):** Dioxus 0.6 (WebAssembly) — UI no navegador via WASM.
*   **Rust (Antifraude):** Colusão, chip dumping, bot detection, multi-account.

### 📋 Ordem dos Tópicos — Roteiro de Aprendizado da Plataforma

1.  **Interpretação e Especificação (Fase 1 do SDD)** — ler problemas e escrever as regras de negócio sem ambiguidades.
2.  **Lógica Proposicional e Matemática Aplicada** — condições, probabilidades e potes do jogo.
3.  **Planejamento e Tarefas (Fases 2 e 3 do SDD)** — quebrar o problema em pedaços lógicos e pseudocódigo.
4.  **Harness Engineering** — como criar "pistas de teste" para validar o código com segurança.
5.  **Rust & O Motor do Jogo (Imersão Principal)** — sintaxe, ownership, borrowing, alta concorrência e construção de APIs ultrarrápidas.
6.  **Rust & Frontend (Dioxus/WebAssembly)** — gerar e revisar componentes visuais e painéis no navegador via WASM.
7.  **Programação Agêntica & Rust** — orquestração com Hermes Agent, WebMCP, UCP e automações.
8.  **Banco de Dados e Persistência** — salvar históricos e saldos integrados ao Rust (PostgreSQL + Redis).
9.  **Git/GitHub e CI/CD** — versionamento e automação de entregas.
10. **Segurança Pentest & Pagamentos** — depósitos, saques e blindagem de transações.
11. **Deploy e Infraestrutura** — Docker, Caddy e Kubernetes.

### 🔤 Regra de Termos Técnicos — Definição Simples + Exemplo Prático

A cada termo técnico novo, abrir parêntese com definição simples + exemplo prático.
*   *Exemplo:* "O Rust usa o conceito de Ownership (Ownership = o sistema que garante que apenas uma parte do programa seja 'dona' de um dado por vez, evitando que o programa trave ou vaze memória)."

### ⌨️ Regra do Significado dos Símbolos — A Pontuação da Máquina (Rust)

É expressamente obrigatório explicar o que significa cada caractere especial no código, com atenção redobrada aos símbolos do Rust (`&`, `*`, `mut`, `!`, `<T>`, `| |`, `::`, etc.). O aluno precisa saber o que está "falando" ao digitar aquele símbolo.
*   *Exemplo:* "No Rust, quando usamos o e-comercial `&` antes de uma palavra, estamos dizendo: 'Estou apenas emprestando essa informação para você ler, não estou te dando ela em definitivo'. E o ponto de exclamação `!` em `println!` avisa que isso não é uma função comum, mas sim uma 'macro' (um atalho de código)."

### 📐 Regra de Metodologia SDD — Spec-Driven Development

1.  **Spec (Especificação):** O que o sistema tem que fazer e quais as regras.
2.  **Plan (Plano):** Como as partes vão se conectar.
3.  **Tasks (Tarefas):** O passo a passo exato para a IA programadora.

### 🔍 Regra de Dissecação de Código — Linha por Linha, Caractere por Caractere

A lógica de cada comando manual ou gerado pela IA deve ser dissecada linha por linha, caractere por caractere.
*   *Exemplo (Comando: `Get-ChildItem -Path C:\`):*
    **📖 Explicação detalhada:**
    *   `Get-ChildItem`: Cmdlet que lista arquivos.
    *   `-` (Traço): O traço avisa ao sistema que a próxima palavra é uma configuração (parâmetro).
    *   `Path`: A configuração que indica o "caminho" de onde vamos começar.
    *   `C:\`: A raiz do disco. Os dois pontos `:` separam o nome do disco da barra `\`, que indica "entrar na pasta".

### 🔄 Regra de Revisão Espaçada — 3 Conceitos Anteriores + 2 Exercícios

Toda aula começa com a revisão dos 3 conceitos anteriores, 2 mini exercícios integrados e explicação do aluno.

### 🧭 Sequência Obrigatória por Tópico — 4 Etapas

#### 📖 Etapa 1 — Teoria: O que é, Analogia, SDD e Símbolos Rust
O que é, analogia, como funciona no modelo SDD e explicação clara dos símbolos usados na sintaxe (com destaque para o Rust).

#### 🧩 Etapa 2 — 1 Exemplo Avançado com Código Real da Plataforma
*   **Enunciado:** Problema real do projeto (trecho de código em produção).
*   **Spec & Plan (SDD):** A especificação lógica e a arquitetura visual/fluxo.
*   **O Código Real:** O trecho de código **real** do projeto (não pseudocódigo).
*   **Dissecação Detalhada:** Cada linha, cada caractere especial, cada símbolo explicado.
*   **Erro de IA Comum:** Uma alucinação típica da IA e como revisar e corrigir manualmente.
*   **🧩 Exercício de Fixação:** Logo abaixo da dissecação, um exercício completo para o aluno praticar. Estrutura:
    1.  **Enunciado do exercício** (variação do exemplo principal)
    2.  **Dica de abordagem** (por onde começar)
    3.  **Onde aplicar no projeto** (arquivo real para modificar)
    4.  **Resultado esperado** (o que deve acontecer quando estiver correto)
    5.  **Checkpoint de validação** (como rodar o teste e confirmar que funcionou)

#### 🔗 Etapa 3 — Integração no Projeto: Tarefa Real do Backlog (DASHBOARD.md)
Após entender o exemplo, o aluno aplica o conceito em uma tarefa real do backlog (DASHBOARD.md).

#### 🔁 Etapa 4 — Revisão Espaçada: Conceitos do Tópico Anterior
Na próxima aula, revisão rápida dos conceitos do tópico anterior antes de avançar.

### 📡 Regras de Comunicação — Firmeza, Clareza e Foco na Plataforma

✅ **SEMPRE:**
*   Firmeza, clareza e persistência no foco.
*   Dissecar o código e os símbolos, especialmente as peculiaridades do Rust (Regra do Significado dos Símbolos).
*   Ensinar a auditar a IA manualmente.
*   Usar marcadores para parágrafos maiores.

❌ **NUNCA:**
*   Usar jargões ou expressões militares em nenhuma hipótese.
*   Usar jargão técnico sem definição imediata.
*   Pular a fase de Especificação (Spec).
*   Permitir que o aluno mude de assunto.

---

## §3 — Guia Prático da Sprint S03 (WebSockets, Atores & Docker)

Bem-vindo ao guia didático da Sprint S03! Aqui vamos desmistificar a teoria por trás das linhas de código que escrevemos, usando analogias do dia a dia e explicando o **porquê** de cada decisão de design.

---

### 🎭 1. O Modelo de Atores (Actor Model) no Poker

No poker real, cada mesa tem um **Dealer** (o crupiê) que distribui as cartas, recolhe as fichas e decide de quem é a vez. Os jogadores não mexem no baralho sozinhos; eles pedem para o Dealer fazer as ações (dar fold, bet, check).

No nosso código, o **`game_actor.rs`** (em `API-Axum/src/`) funciona exatamente como esse Dealer:
*   Ele é uma "entidade independente" (Actor) que roda dentro de uma tarefa separada do computador (`tokio::spawn`), sem travar o resto do servidor.
*   Ele gerencia sua própria mesa e ninguém pode alterar o estado do jogo diretamente.
*   **Como os jogadores falam com ele?** Usando um canal de comunicação chamado **`mpsc` (Multi-Producer Single-Consumer)**. Imagine que cada jogador tem um walkie-talkie para enviar comandos ("Quero dar Fold!") para a central do Dealer.
*   **Como o Dealer fala com todo mundo ao mesmo tempo?** Usando um canal **`broadcast`**. Quando o Dealer atualiza a mesa, ele grita no megafone e todos os jogadores escutam o novo estado do jogo.

---

### 🔌 2. WebSockets vs HTTP (Comunicação em Tempo Real)

Quando você navega em um site de notícias, seu navegador faz uma pergunta e o servidor responde (isso é HTTP REST — um ciclo de pergunta e resposta curta). Mas no Poker, as coisas acontecem muito rápido! Se um jogador apostar, você precisa ver a aposta na sua tela imediatamente, sem ter que ficar atualizando a página.

*   O **WebSocket** cria um "túnel aberto" bidirecional. O navegador e o servidor ficam de mãos dadas. Qualquer um dos dois pode enviar dados a qualquer momento.
*   No **`websocket.rs`** (em `API-Axum/src/handlers/`), assim que o túnel é aberto, nós conectamos o walkie-talkie do jogador ao Dealer da mesa correspondente.

#### 🛡️ O Sistema Anti-Cheat (Filtragem de Cartas)

Se enviássemos todas as informações da mesa de uma vez, um jogador mal-intencionado poderia abrir o console do navegador e ler as cartas de hole dos adversários.
*   Para evitar isso, criamos o filtro de segurança `filter_table_state` no WebSocket.
*   **A Regra:** O servidor lê o estado do jogo e apaga (torna `None` ou limpa o array) as cartas privadas de todos os outros jogadores. Você só recebe as suas próprias cartas no seu navegador. As cartas alheias só são transmitidas no Showdown (fim da mão), garantindo que ninguém consiga trapacear!

---

### 🐳 3. Docker e Builds Multi-Stage (Nossos Containers)

Imagine que você vai preparar um bolo para vender. Você usa batedeira, potes, formas, farinha e ovos. Mas o seu cliente só quer o bolo pronto em uma caixinha de papelão limpa, sem a sujeira da cozinha.

O **Docker** cria essa "cozinha isolada" (o Container). E o **build multi-stage** faz exatamente a separação entre a cozinha e a caixa de entrega:
1.  **Stage 1 (Builder - A Cozinha)**: Usamos uma imagem completa com o compilador do Rust (`rust:latest`). Baixamos todas as dependências, compilamos o código em modo release. Isso gera um binário executável e muita sujeira (pastas de build temporárias que ocupam gigabytes).
2.  **Stage 2 (Runner - A Caixa de Entrega)**: Criamos uma imagem vazia e mínima (`debian:bookworm-slim`). Copiamos apenas o bolo pronto (o binário compilado `poker-api`) do Stage 1.
*   **Resultado**: O container final fica incrivelmente pequeno (menos de 100MB em vez de 2GB), seguro (não tem compilador lá dentro para hackers usarem) e rápido.

---

### 🛠️ 4. Bypassando o Dioxus CLI (O Pulo do Gato)

Durante o desenvolvimento do Frontend no Docker, a ferramenta automática do Dioxus (`dx CLI`) travou por conflitos de versão do gerador de WebAssembly (`wasm-bindgen`).
*   **O aprendizado prático**: Em programação, quando uma ferramenta automática falha ou é muito engessada, nós podemos fazer o processo **manualmente**.
*   Substituímos o `dx build` por comandos puros do Rust:
    1. Compilamos o código Rust direto para WebAssembly: `cargo build --target wasm32-unknown-unknown`.
    2. Usamos a ferramenta oficial `wasm-bindgen` para empacotar o Wasm gerado em arquivos JavaScript compreensíveis pelo navegador.
    3. Usamos o comando de texto `sed` para injetar o script que inicia tudo no final do arquivo HTML.
*   Isso nos deu controle total, resolveu o bug e acelerou o build pela metade!

---

### 🔑 5. O Gateway com Caddy (HTTPS Local)

O **Caddy** funciona como a portaria de um condomínio de prédios:
*   Ele escuta nas portas de entrada da internet (`80` para HTTP e `443` para HTTPS).
*   Se alguém pede uma página do site, o Caddy lê o arquivo estático e entrega (o Frontend Dioxus).
*   Se alguém pede uma chamada de API (`/api/*`) ou uma conexão em tempo real (`/ws/*`), o Caddy redireciona essa chamada silenciosamente para o container do backend Axum (`poker_api:3000`).
*   Ele gera um certificado de segurança local automaticamente. Assim, rodamos o site em `https://localhost` com o cadeado verde ativo no navegador, que é um requisito de segurança real para proteger a senha e fichas dos jogadores.

---

### 🚀 Como aprender testando na prática?

1. Abra o arquivo `Caddyfile` (em `Frontend-Dioxus/`) e veja como as regras de proxy reverso são fáceis de ler.
2. Abra o `websocket.rs` (em `API-Axum/src/handlers/`) e encontre a função `filter_table_state`. Modifique-a temporariamente e tente entender como ela protege o fluxo de dados.
3. Suba a stack com `docker-compose up -d` e acompanhe os logs dos containers no Docker Desktop para ver as mensagens trafegando em tempo real.

---

## §4 — Guia dos 11 Módulos do Motor de Poker

Seja muito bem-vindo ao manual completo de aprendizado da nossa plataforma de poker escrita em Rust! Aqui, vamos passar por **todos os módulos** implementados, explicando os conceitos matemáticos, de segurança e de arquitetura usando analogias simples e exemplos de código práticos.

---

### 🗺️ Índice dos Módulos

1. [O Baralho e Avaliação de Mãos (`deck.rs`)](#1-o-baralho-e-classificação-de-mãos-deckrs)
2. [Divisão de Potes em Múltiplos All-ins (`side_pots.rs`)](#2-potes-paralelos-side_potsrs)
3. [Cashback Progressivo (`loss_deflator.rs`)](#3-cashback-de-perdas-loss_deflatorrs)
4. [A Comissão da Casa (`rake.rs`)](#4-comissão-da-casa-rakers)
5. [Gerador de Números Aleatórios Criptográfico (`rng_crypto.rs`)](#5-embaralhando-com-segurança-criptográfica-rng_cryptors)
6. [Auditoria e Logs de Partida (`hand_history.rs`)](#6-a-caixa-preta-do-jogo-hand_historyrs)
7. [Ciclo de Vida de Torneios (`tournament_engine.rs`)](#7-gerenciamento-de-torneios-tournament_enginers)
8. [Autenticação Segura e MFA (`auth.rs`)](#8-segurança-de-contas-authrs)
9. [Gerenciamento do Lobby (`lobby.rs`)](#9-matchmaking-e-lobby-lobbyrs)
10. [Segurança e Algoritmos Antifraude (`antifraud/`)](#10-detectando-trapaças-antifraud)
11. [A Máquina de Estados do Poker (`game_loop.rs`)](#11-o-fluxo-da-partida-game_looprs)

---

### 1. O Baralho e Classificação de Mãos (`deck.rs`)

O baralho é a fundação de qualquer jogo de cartas. Em computação, precisamos representar esses objetos do mundo real de forma estruturada.

#### Tipagem Forte com Enums e Structs

Em vez de salvar as cartas como texto puro (ex: `"As de Copas"`), usamos a tipagem estrita do Rust:
*   **`Suit` (Naipe)**: Um Enum com 4 variantes (`Spades` ♠, `Hearts` ♥, `Diamonds` ♦, `Clubs` ♣).
*   **`Rank` (Valor)**: Um Enum com 13 variantes (`Two` até `Ace`).
*   **`Card` (Carta)**: Uma Struct que junta um `Rank` e um `Suit`.

Isso impede que o programa crie cartas inválidas como um "As de Círculos" ou uma carta com valor "15". Se tentar fazer isso, o compilador do Rust impede o programa de rodar antes mesmo dele ser executado!

#### Classificação e Helpers

A classificação de uma mão de poker (descobrir se você tem um *Pair*, um *Flush* ou um *Royal Flush*) é um problema clássico de ordenação e agrupamento.
*   **Manutenibilidade**: Inicialmente, a função de avaliação tinha centenas de linhas de código aninhadas. Refatoramos o código dividindo-o em 9 funções auxiliares (*helpers*), como `get_high_card()` e `check_flush()`. Cada função faz apenas uma coisa muito bem, tornando o código legível e fácil de testar.

---

### 2. Potes Paralelos (`side_pots.rs`)

Imagine uma mesa de poker física onde 3 jogadores vão All-in (apostam tudo):
*   **Jogador A** tem R$ 10.
*   **Jogador B** tem R$ 50.
*   **Jogador C** tem R$ 50.

O Jogador A só pode ganhar no máximo R$ 10 de cada um dos oponentes. O que acontece com os outros R$ 40 que os jogadores B e C apostaram? Eles formam um **Pote Paralelo (Side Pot)** de R$ 80, no qual o Jogador A não pode tocar (mesmo que ele tenha a melhor mão de todas!).

#### O Algoritmo

O arquivo **`side_pots.rs`** resolve essa matemática:
1.  Ordena as apostas em ordem crescente.
2.  Cria potes em camadas (pote principal e potes paralelos).
3.  Calcula a contribuição de cada jogador para cada camada e define quem tem direito a disputar cada pote baseado no valor que investiu.

---

### 3. Cashback de Perdas (`loss_deflator.rs`)

Poker pode ser um jogo cruel para iniciantes que perdem suas fichas rapidamente em All-ins infelizes. Para incentivar o jogo responsável e reter jogadores casuais, criamos um sistema inédito de **Cashback Progressivo (Loss Deflator)**.

*   **Como funciona?** O algoritmo calcula a **equity** (probabilidade exata de vencer) do perdedor no momento do all-in, via enumeração heads-up (`get_heads_up_win_probability()`).
*   Se um jogador vai All-in com uma mão favorita mas sofre uma derrota de azar (um "bad beat"), o sistema devolve parte do valor perdido em saldo promocional, de acordo com a equity:

| Equity do Perdedor | Tier | Cashback |
|---------------------|------|----------|
| 60,0% – 64,9%       | 0    | **7%**   |
| 65,0% – 74,9%       | 1    | **15%**  |
| 75,0% – 84,9%       | 2    | **25%**  |
| ≥ 85,0%             | 3    | **35%**  |
| < 60,0%             | —    | 0%       |

*   **Exemplo:** All-in preflop com A♠A♦ vs K♠K♦ (equity ≈ 82%, Tier 2). Se o perdedor com AA perde R$ 200, recebe R$ 50 de cashback (25%).

> 💡 **Quer aprender mais?** Veja explicações detalhadas sobre Equity, OESD, Gutshots, Combo Draws e tabelas reais de simulação em [Exemplos Detalhados do Loss Deflator](LOSS_DEFLATOR_EXEMPLOS.md).

---

### 4. Comissão da Casa (`rake.rs`)

As plataformas de poker não jogam contra os usuários; elas cobram uma pequena taxa sobre as apostas de cada mesa para manter o serviço ativo. Essa taxa é chamada de **Rake**.

#### Protegendo o Jogador (O Cap)

Para não cobrar demais em potes muito grandes, usamos duas regras matemáticas:
1.  **Porcentagem fixa**: 2.5% de comissão sobre o pote total.
2.  **O Cap (Teto máximo)**: A comissão máxima cobrada nunca pode ultrapassar R$ 6.

O módulo **`rake.rs`** garante que essa conta seja feita com precisão decimal exata, prevenindo erros de arredondamento financeiros.

---

### 5. Embaralhando com Segurança Criptográfica (`rng_crypto.rs`)

Se você usar um gerador de números aleatórios padrão do computador (como `rand::thread_rng()`), um jogador inteligente que observa várias mãos pode descobrir a "semente" matemática do gerador e prever as próximas cartas do baralho!

*   **CSPRNG (Cryptographically Secure Pseudo-Random Number Generator)**: Usamos o gerador seguro do sistema operacional (`OsRng`). Ele pega entropia física do computador (como variações de temperatura do processador ou movimentos do mouse do servidor) para criar aleatoriedade verdadeiramente imprevisível.
*   **Rejection Sampling (Amostragem por Rejeição)**: Quando queremos escolher um número aleatório entre 0 e 51 (as 52 cartas), não podemos apenas pegar o número seguro e usar o operador de resto (`% 52`). Isso gera um viés matemático (*modulo bias*) onde algumas cartas saem mais que as outras. O Rejection Sampling descarta números fora do intervalo perfeito, garantindo que o embaralhamento seja 100% justo e auditável.

---

### 6. A Caixa Preta do Jogo (`hand_history.rs`)

Se houver uma disputa judicial, suspeita de fraude ou se um jogador quiser rever suas jogadas, precisamos de uma gravação imutável da partida.

O módulo **`hand_history.rs`** atua como essa caixa preta:
*   Ele registra cronologicamente todas as ações da mesa: quem sentou, quem deu fold, quais cartas saíram em cada etapa e o resultado.
*   Graças à biblioteca `serde`, ele é convertido diretamente para o formato **JSON**, que pode ser salvo facilmente em um banco de dados SQL ou enviado para o navegador do cliente para desenhar um replay visual na tela.

---

### 7. Gerenciamento de Torneios (`tournament_engine.rs`)

Ao contrário de mesas de Cash Game onde você entra e sai quando quer, os Torneios têm regras rígidas de ciclo de vida e economia.

O arquivo **`tournament_engine.rs`** orquestra isso:
*   **Estados do Torneio**: Registrando → Rodando → Pausado → Finalizado.
*   **Estrutura de Blinds**: Uma tabela configurável que define de quanto em quanto tempo as apostas obrigatórias aumentam, forçando a ação dos jogadores.
*   **Rebuy e Addon**: Permite que jogadores que perderam fichas comprem mais saldo em períodos pré-determinados.

---

### 8. Segurança de Contas (`auth.rs`)

Se a sua plataforma envolve dinheiro real ou saldos de fichas, a segurança de senhas é a sua prioridade número um.

#### Criptografia de Senhas (Bcrypt)

Nunca guarde senhas em texto puro ou com algoritmos simples (como MD5 ou SHA-256). Se o banco de dados vazar, um atacante pode descriptografá-las em segundos usando tabelas de hashes pré-computadas (Rainbow Tables).
*   Usamos o **`bcrypt`** com fator de custo 12. O `bcrypt` adiciona um segredo aleatório ("salt") à senha e roda o hash repetidas vezes. Isso torna cada tentativa de adivinhar a senha computacionalmente lenta, inviabilizando ataques de força bruta.

#### JWT (JSON Web Tokens)

Para manter o usuário logado sem que ele tenha que enviar sua senha a cada clique, o servidor emite uma chave digital temporária (o JWT) assinada com uma chave privada. O navegador guarda esse token e o anexa a cada nova requisição.

#### MFA/TOTP (Autenticação em Duas Etapas)

Seguindo o padrão do Google Authenticator (RFC 6238):
1.  O servidor gera uma chave secreta e exibe um QR Code.
2.  O app de autenticação do celular do usuário sincroniza com o relógio interno do aparelho.
3.  A cada 30 segundos, uma nova senha de 6 dígitos é calculada matematicamente baseada no tempo atual. Mesmo que alguém roube a senha do usuário, não conseguirá logar sem o celular em mãos.

---

### 9. Matchmaking e Lobby (`lobby.rs`)

Como conectar centenas de jogadores em dezenas de mesas diferentes sem que um entre na sala errada?

O **`lobby.rs`** gerencia essa recepção:
*   Permite a criação de mesas públicas e privadas (protegidas por senha com criptografia hash).
*   Filtra mesas por limites de blinds ou modalidade de jogo.
*   **Matching Inteligente (`find_or_suggest_table`)**: Se o jogador quer entrar rápido, o sistema busca uma mesa adequada vazia ou sugere uma mesa similar com assentos livres.

---

### 10. Detectando Trapaças (`antifraud/`)

Um dos maiores desafios de operar um site de poker é prevenir que bots ou jogadores em conluio roubem o dinheiro dos outros.

Criamos 4 submódulos inteligentes de segurança:
*   **`collusion.rs` (Conluio)**: Analisa se dois jogadores na mesma mesa jogam de forma coordenada (ex: um dando raise só para assustar os outros e depois dar fold para o amigo coletar as fichas).
*   **`bot_detection.rs` (Bots)**: Robôs tomam decisões matemáticas perfeitas de forma muito rápida. Nosso código analisa o tempo exato de ação. Humanos demoram segundos diferentes dependendo da complexidade da mão, enquanto bots de código simples têm atrasos constantes ou precisão milimétrica.
*   **`chip_dumping.rs` (Transferência ilícita)**: Detecta quando um jogador experiente perde de propósito fichas valiosas para uma conta nova para transferir fundos sem passar pela taxa de saque.
*   **`multi_account.rs` (Múltiplas Contas)**: Cruza dados de fingerprint do navegador, endereço de IP e padrões de conexão para verificar se um jogador abriu 3 abas no mesmo computador e está jogando contra si mesmo em posições diferentes da mesa.

---

### 11. O Fluxo da Partida (`game_loop.rs`)

Uma mão de Texas Hold'em segue um fluxo muito rígido (Preflop → Flop → Turn → River → Showdown). O computador precisa de uma **Máquina de Estados** para garantir que ninguém fure a fila.

*   O **`game_loop.rs`** gerencia o estado da partida ativa:
    *   **Preflop**: Duas cartas privadas para cada um. Blinds obrigatórios coletados.
    *   **Flop**: Três cartas comunitárias abertas na mesa. Rodada de apostas começando pelo Small Blind.
    *   **Turn**: Quarta carta aberta. Rodada de apostas.
    *   **River**: Quinta e última carta aberta. Última rodada de apostas.
    *   **Showdown**: Revelação das cartas. O motor ativa a avaliação de mãos e distribui o pote de forma justa.
*   Ele monitora de quem é a vez (`current_turn`) e impede que um jogador aposte fora de hora ou realize ações impossíveis (como dar Check quando há uma aposta pendente).
