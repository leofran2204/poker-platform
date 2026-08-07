# 🃏 Regras de Negócio — Plataforma de Poker Online Texas Hold'em

**Versão:** 2.0  
**Data:** 2026-07-25  
**Status:** Documento normativo — implementação deve ser confirmada por testes e revisão de cada release


---

## 0. 🏗️ Stack Alvo — Backend Rust + Frontend TypeScript (atualizado em 2026-08-04)

| Camada            | Linguagem                                      | Responsabilidade                                  |
|-------------------|------------------------------------------------|---------------------------------------------------|
| **Backend**       | **Rust**                                       | Motor de jogo, APIs, autenticação, lobby          |
| **IA e Dados**    | **Rust**                                       | Antifraude, estatísticas, relatórios              |
| **Front-end**     | **TypeScript (React + Vite + Tailwind)**       | UI jogadores e admin B2B (`Frontend-Web/`)        |
| **Comunicação**   | **JSON**                                       | Formato universal entre módulos                   |

> 📐 Detalhes completos em `Arquitetura-Motor/ARQUITETURA_MOTOR.md` (v4.0).
> ✅ **Motor e API em Rust.** Frontend canônico em TypeScript desde 2026-08-04. `Frontend-Dioxus/` é legado.
> 📌 Estado operacional (PIX, ownership, ciclo S11): ver `STATUS_OPERACIONAL.json` — **sem** certificação de produção.
> ⚖️ **Regulação / compliance de jogo e dinheiro real:** planejada para **janeiro de 2027**.
> ✉️ **Registro (S11):** senha + confirmação; com `REQUIRE_EMAIL_VERIFICATION=true`, conta fica `pending_email_verification` até código de 6 dígitos no e-mail. Deploy demo: **`EMAIL_PROVIDER=resend`** (domínio verified); lab/testes: **log**. SMTP da caixa webmail ainda não implementado.

### 0.1. 💵 Arquitetura Financeira e Tipagem Estrita (`u64` Centavos)
- **Princípio da Precisão Bancária:** Todos os valores financeiros (saldo, apostas, stacks, potes, rake, buy-in e blinds) utilizam estritamente `u64` centavos inteiros no **backend** (`R$ 10,50` = `1050` centavos). Erros de arredondamento IEEE-754 flutuantes são eliminados na raiz.
- **Probabilidades e Estatísticas:** Mantidos em escala flutuante (`f64` entre `0.0` e `1.0` ou `0.0%` a `100.0%`) para cálculo de equidade e exibição de porcentagens.
- **Formatação de Exibição:** O frontend TypeScript converte centavos apenas na camada visual (`formatBrlFromCents` em `Frontend-Web/src/lib/money.ts`).

---

## 1. 🎯 Visão Geral — Texas Hold'em Tradicional

Plataforma de poker online **Texas Hold'em Tradicional** inspirada no Full Tilt Poker (skin moderna, lobby denso, mesa de feltro). Suporta mesas de **Cash Game** e **Tournament** com até 9 jogadores.

---

## 2. 🂡 Regras do Baralho — Texas Hold'em Tradicional

### 2.1 📦 Composição do Baralho — 52 Cartas
- **52 cartas** no total
- **Naipes:** hearts (♥), diamonds (♦), clubs (♣), spades (♠)
- **Ranks:** A, K, Q, J, T, 9, 8, 7, 6, 5, 4, 3, 2
- **Valores numéricos:** 2=2, 3=3, 4=4, 5=5, 6=6, 7=7, 8=8, 9=9, T=10, J=11, Q=12, K=13, A=14

### 2.2 🏆 Ranking de Mãos — Ordem de Força
| Rank | Nome              | Valor |
|------|-------------------|-------|
| 10   | Royal Flush       | 10    |
| 9    | Straight Flush    | 9     |
| 8    | Four of a Kind    | 8     |
| 7    | Full House        | 7     |
| 6    | Flush             | 6     |
| 5    | Three of a Kind   | 5     |
| 4    | Straight          | 4     |
| 3    | Two Pair          | 3     |
| 2    | One Pair          | 2     |
| 1    | High Card         | 1     |

### 2.3 🔄 Straight Especial — Ás como Carta Baixa
- **A-2-3-4-5** é um straight válido (Ás jogando como carta baixa)
- Straight normal: 5 cartas consecutivas (ex: 8-9-T-J-Q)

---

## 3. 🪑 Estrutura da Mesa — Configuração e Parâmetros

### 3.1 ⚙️ Configuração da Mesa — Parâmetros
| Parâmetro     | Tipo     | Validação                          |
|---------------|----------|------------------------------------|
| `name`        | string   | 3-100 caracteres                   |
| `gameType`    | enum     | `cash` \| `tournament`             |
| `smallBlind`  | number   | > 0                                |
| `bigBlind`    | number   | > 0                                |
| `minBuyIn`    | number   | > 0                                |
| `maxBuyIn`    | number   | > 0                                |
| `maxPlayers`  | int      | 2-9                                |
| `speed`       | enum     | `normal` \| `turbo` \| `hyper`     |
| `ante`        | number?  | ≥ 0 (opcional)                     |

### 3.2 ⏱️ Velocidades e Timeouts — Normal, Turbo e Hyper
| Speed    | Timeout por ação |
|----------|------------------|
| `normal` | 30 segundos      |
| `turbo`  | 15 segundos      |
| `hyper`  | 8 segundos       |

**Timeout = auto-fold** automático.

### 3.3 💵 Buy-in — Entrada e Cash-out
- Jogador deve ter saldo suficiente para o buy-in
- Buy-in deduzido do saldo ao entrar na mesa
- Cash-out devolve as fichas ao saldo ao sair

---

## 4. 🔄 Fluxo do Jogo — Fases e Deal

### 4.1 📊 Fases do Jogo — GamePhase
```
waiting → preflop → flop → turn → river → showdown → (volta para preflop)
```

| Fase       | Cartas Comunitárias | Ação Inicia                          |
|------------|---------------------|--------------------------------------|
| `waiting`  | 0                   | Aguardando jogadores                 |
| `preflop`  | 0                   | Após Big Blind (sentido horário)     |
| `flop`     | 3                   | Após Dealer (sentido horário)        |
| `turn`     | 4                   | Após Dealer (sentido horário)        |
| `river`    | 5                   | Após Dealer (sentido horário)        |
| `showdown` | 5                   | Revelação das cartas                 |

### 4.4 🪙 Resolução de Centavos Ímpares em Split Pots (Regra do Centavo Ímpar — Odd Cent Rule)

Em conformidade estrita com as regras oficiais do Poker Internacional Live (WSOP / TDA Regra 68):

1. **Divisão Truncada (2 Casas Decimais)**:
   - Em potes divididos (*split pots*), o valor atribuído a cada jogador empatado é truncado em 2 casas decimais (centavos exatos).
2. **Atribuição do Centavo Remanescente**:
   - Caso haja um resto indivisível de centavos ($R = \text{Pote} - N \times \text{Valor Base}$), o(s) centavo(s) de R$ 0,01 são entregues de 1 em 1 aos jogadores empatados **mais próximos à esquerda do Botão (Dealer)** na ordem dos assentos.
3. **Conservação Financeira**:
   - Garante a conservação exata de fichas na mesa ($\sum \text{Prêmios} = \text{Pote Líquido}$), sem acumular resíduos de ponto flutuante nem retenções indevidas pela casa.
4. **Decisão Arquitetural Monetária Estrita (`u64` Centavos Inteiros em Toda a Stack)**:
   - A plataforma opera 100% em **centavos inteiros (`u64`)** em todas as camadas (Banco de Dados, Motor de Jogo, APIs REST, WebSockets e Dioxus WASM).
   - O frontend Dioxus converte os centavos `u64` para o formato legível `R$ {:.2}` exclusivamente na camada visual de renderização.
   - Elimina 100% de ruídos ou discrepâncias de ponto flutuante IEEE 754 e preserva precisão bancária B3 de centavos exatos em todas as operações de potes, rake e saldos.

### 4.2 🃏 Deal — Distribuição de Cartas
1. Embaralhar baralho (Fisher-Yates)
2. Distribuir 2 cartas por jogador
3. **Burn 1 carta** antes de cada street (flop/turn/river)
4. Flop: 3 cartas | Turn: 1 carta | River: 1 carta

### 4.3 📍 Posições — Dealer, SB, BB e UTG
- **Dealer (Botão):** posição rotativa a cada mão
- **Small Blind (SB):** jogador à esquerda do Dealer
- **Big Blind (BB):** jogador à esquerda do SB
- **Under the Gun (UTG):** primeiro a agir no preflop

### 4.5 🔁 Ordem de Ação — Sentido Horário
- **Preflop:** UTG → ... → BB (último a agir)
- **Pos-flop:** SB → ... → Dealer (último a agir)
- Sentido **horário**

---

## 5. 🎮 Ações do Jogador — Fold, Check, Call, Bet, Raise, All-in

### 5.1 🕹️ Tipos de Ação — Comandos Disponíveis
| Ação     | Descrição                  | Condição                              |
|----------|----------------------------|---------------------------------------|
| `fold`   | Desistir da mão            | Sempre                                |
| `check`  | Passar a vez               | Só se `currentBet == player.bet`      |
| `call`   | Igualar aposta atual       | `amount = currentBet - player.bet`    |
| `bet`    | Abrir aposta               | `amount ≥ bigBlind`                   |
| `raise`  | Aumentar aposta            | `amount ≥ currentBet + minRaise`      |
| `all_in` | Apostar todas as fichas    | Sempre (se `chips > 0`)               |

### 5.2 ✅ Validações — Regras por Ação
- Jogador deve ser o `currentPlayer`
- Valor não pode exceder `player.chips`
- `bet` mínimo = `bigBlind`
- `raise` mínimo = `currentBet + minRaise`

### 5.3 ⏭️ Auto-Avanço — Progressão Automática
- Após cada ação, verifica se todos os jogadores ativos igualaram a aposta
- Se sim, avança para próxima fase
- Se não, passa para o próximo jogador

---

## 6. 💰 Pot e Apostas — Estrutura e Distribuição

### 6.1 🏦 Estrutura do Pot — Pot Principal e Side Pots
- **Pot principal:** soma de todas as apostas
- **Side pots:** ⚠️ **NÃO IMPLEMENTADO** (ver auditoria)

### 6.2 🤝 Distribuição — Vencedor e Split Pot
- Vencedor recebe o pot inteiro
- Em caso de empate (tie), pot dividido (split)
- ⚠️ **Split pot NÃO implementado** (ver auditoria)

---

## 7. 🔍 Avaliação de Mãos — Showdown

### 7.1 📏 Critérios de Desempate
1. Maior `value` (rank) vence
2. Se mesmo rank, comparar cartas principais em ordem
3. Se mesmo cartas principais, comparar kickers
4. Se tudo igual → empate (split pot)

### 7.2 🗺️ Mapeamento de Cartas — Cards e Kickers
- `cards`: cartas que formam a mão (ex: par, trinca)
- `kickers`: cartas de desempate (ex: par + 3 kickers)

---

## 8. 💬 Sistema de Chat — Comunicação na Mesa

### 8.1 📝 Regras do Chat
- Mensagens limitadas a **500 caracteres**
- Tipos: `chat`, `system`, `action`
- Sem moderação automática implementada
- ⚠️ **Filtro de palavrões NÃO implementado** (ver auditoria)

---

## 9. 🔐 Autenticação e Usuários — JWT e Saldo

### 9.1 📝 Registro e Login — JWT + bcrypt
- JWT com `userId` e `username`
- Senha hasheada com bcrypt
- Token obrigatório para WebSocket

### 9.2 💳 Saldo do Jogador — Buy-in e Cash-out
- Campo `balance` em `users`
- Deduzido no buy-in
- Devolvido no cash-out
- **Rake Cash Games:** configuração por mesa em pontos-base inteiros (padrão: 500 = 5,00%; cap legado padrão: R$ 100,00). O cálculo e o rateio são feitos exclusivamente com inteiros em centavos.
- **Cap de rake por nº de jogadores (opcional):** a mesa pode definir agenda completa `rake_cap_heads_up` / `rake_cap_three_to_four` / `rake_cap_five_plus` (todos NULL = só cap legado; ou os três preenchidos). O motor escolhe o cap conforme quantos jogadores receberam cartas na mão (`RakeCapSchedule`).
- ✅ **Fee Torneios: 7% no Buy-in, com taxa 0% em Re-buys e Add-ons**

### 9.3 🏢 Divisão Financeira B2B SaaS (Rake Split 15% / 85%)
- **Ordem de Execução Inviolável**: Potes brutos → Cálculo de Rake → **Split B2B (15% Plataforma Zerotilt / 85% Clube Locatário)** → Aplicação do Loss Deflator sobre o pote líquido pós-rake → Distribuição dos prêmios.
- **Roteamento de Ledger**: O valor do Rake do Clube (`club_rake`) é injetado diretamente no saldo administrativo da tabela `clubs` (`balance`) ao final de cada mão (apenas mesas com `club_id`). O fee da plataforma (`platform_fee`) é contabilizado para a Zerotilt.
- **Agentes / rakeback**: percentuais 0–50% cadastrados em `club_agents` (admin HTTPS); comissões acumuladas em centavos. Não altera o split 15/85 plataforma/clube — o rakeback do agente é subconjunto da fatia do clube.
- **Play-money**: operação atual permanece sem dinheiro real; saques de clube via admin são intenções mock/sandbox.

---

## 10. ⚡ Estados Especiais — All-In, Timeout e Desconexão

### 10.1 🔥 All-In — Jogador sem Fichas
- Jogador com `chips == 0` é marcado `isAllIn`
- Continua elegível para ganhar o pot
- Não precisa agir mais

### 10.2 ⏰ Timeout — Auto-Fold
- Após `timeLeft == 0`, jogador é auto-folded
- Timer reinicia a cada ação

### 10.3 🔌 Desconexão — Remoção da Mesa
- Jogador é removido da mesa
- Fichas devolvidas ao saldo
- Mesa é limpa se ficar vazia

---

## 11. 🛡️ Loss Deflator — Cashback Progressivo por Equity

O **Loss Deflator** é um sistema de cashback automático que devolve parte das perdas em all-in calls quando o jogador tinha alta probabilidade de vencer (equity) mas perdeu por azar (bad beat).

### 11.1 🎯 Modelo Baseado em Equity

O cashback é determinado pela **equity do perdedor no instante em que o all-in é pago**. O cálculo heads-up é determinístico: enumeração quando viável e Monte Carlo determinístico nos espaços maiores (`get_heads_up_win_probability()`). A fase da mão serve apenas para reconstruir quais cartas já estavam abertas; **preflop, flop, turn ou river nunca determinam o percentual**.

| Tier  | Equity do Perdedor | Cashback | Perfil do Rango |
|-------|---------------------|----------|-----------------|
| **0** | **56,0% – 65,9%**   | **7%**   | Favorito leve |
| **1** | **66,0% – 75,9%**   | **15%**  | Favorito moderado |
| **2** | **76,0% – 85,9%**   | **25%**  | Grande favorito |
| **3** | **≥ 86,0%**         | **35%**  | Favorito esmagador / bad beat extrema |
| —     | **< 56,0%**         | **0%**   | Não elegível |

### 11.2 ⚙️ Regras de Aplicação e Origem Financeira

- **Ordem financeira obrigatória:** formar main pot e side pots → retirar o rake de cada pote → calcular o Loss Deflator somente sobre os potes elegíveis já líquidos → concluir os pagamentos.
- **Origem das Fichas:** O cashback é autofinanciado pelas fichas playmoney dos potes líquidos da mão. Ele é descontado da fatia do(s) vencedor(es) do pote elegível e entregue ao perdedor all-in; não cria fichas novas.
- **Aplicação atual:** Cash Games e torneios usam apenas fichas **playmoney**; não há dinheiro real habilitado.
- **Múltiplos All-Ins e Fases Distintas:** Cada perdedor possui um snapshot individual de fase e board para calcular sua equity. A fase não escolhe o tier.
- **Equity multiway:** quando o perdedor all-in compartilha potes com **dois ou mais** oponentes ainda na mão, a equity usa Monte Carlo multiway determinístico (`get_multiway_win_probability`). Com um único oponente, usa heads-up.
- **Isolamento de Side Pots:** O cashback de um perdedor é calculado e descontado APENAS dos potes líquidos pós-rake em que ele participou. Side pots nos quais não era elegível ficam intocados.
- **Limite Máximo:** Cashback nunca excede 35% do valor perdido.
- **Anti-abuso:** Perder propositalmente para receber cashback é detectado pelo módulo antifraude.

### 11.3 📐 Exemplos

| Cenário                          | Equity | Tier | Perda   | Cashback |
|----------------------------------|--------|------|---------|----------|
| All-in preflop, A♠A♦ vs K♠K♦    | 82%    | 2    | R$ 200  | R$ 50    |
| All-in flop, set vs flush draw   | 65%    | 0    | R$ 100  | R$ 7     |
| All-in turn, overpair vs set     | 7%     | —    | R$ 100  | R$ 0     |
| All-in preflop, AK vs QQ         | 62%    | 0    | R$ 500  | R$ 35    |
| All-in flop, flush vs straight   | 88%    | 3    | R$ 300  | R$ 105   |

## 12. �📖 Glossário — Termos do Poker

| Termo        | Significado                                      |
|--------------|--------------------------------------------------|
| **Pot**      | Total de fichas apostadas na mão                 |
| **Kicker**   | Carta de desempate                               |
| **Blinds**   | Apostas obrigatórias (SB + BB)                   |
| **Ante**     | Aposta forçada de todos (opcional)               |
| **Street**   | Cada fase de apostas (preflop, flop, turn, river) |
| **Showdown** | Revelação final das cartas                       |
| **Side Pot** | Pot separado para all-ins                        |
| **Rake**     | Taxa da casa                                     |

---

## 13. 🔗 Referências no Código — Onde Cada Regra Vive

| Regra               | Arquivo                          | Função/Local                    |
|---------------------|----------------------------------|---------------------------------|
| Baralho 52 cartas   | `Motor-Rust/src/deck.rs`         | `create_deck()`, `shuffle()`    |
| Ranking de mãos     | `Motor-Rust/src/deck.rs`         | `evaluate_hand()`               |
| Straight A-2-3-4-5  | `Motor-Rust/src/deck.rs`         | `is_straight()`                 |
| Side pots           | `Motor-Rust/src/side_pots.rs`    | `calculate_side_pots()`         |
| Loss Deflator       | `Motor-Rust/src/loss_deflator.rs`| `calculate_progressive_loss_deflator()` |
| Rake                | `Motor-Rust/src/rake.rs`         | `calculate_rake()`              |
| RNG criptográfico   | `Motor-Rust/src/rng_crypto.rs`   | `CryptoRng`                     |
| Hand history        | `Motor-Rust/src/hand_history.rs` | `HandHistory::record()`         |
| Torneios            | `Motor-Rust/src/tournament_engine.rs` | `TournamentEngine`          |
| Lobby               | `Motor-Rust/src/lobby.rs`        | `Lobby`                         |
| Antifraude          | `Motor-Rust/src/antifraud/`      | `collusion`, `bot_detection`    |
| Autenticação        | `Motor-Rust/src/auth.rs`         | `JWT`, `MFA`, `RBAC`            |
| Tipos               | `Motor-Rust/src/types.rs`        | Todos os tipos                  |

---

## 14. 🏛️ Sistema de Lobby — Gestão de Mesas e Matchmaking

### 14.1 🎛️ Gestão de Mesas — Criação e Listagem
- **Criação**: Mesas podem ser criadas com `gameType` (cash/tournament), `smallBlind`, `bigBlind`, `maxPlayers` e `visibility` (public/private).
- **Listagem**: O lobby deve fornecer uma lista de mesas filtrável por:
    - Tipo de jogo (Cash vs Tournament)
    - Limites de Blinds
    - Quantidade de jogadores disponíveis
- **Privacidade**: Mesas privadas exigem uma `password` para entrada.

### 14.2 🚪 Matchmaking e Entrada — Fluxo do Jogador
- **Fluxo de Entrada**:
    1. Jogador seleciona mesa.
    2. Sistema verifica se `player.balance >= table.minBuyIn`.
    3. Se mesa for privada, valida `password`.
    4. Se `currentPlayers < maxPlayers`, jogador é movido para o estado `Playing`.
- **Estados do Jogador**:
    - `Lobby`: Navegando entre mesas.
    - `Playing`: Sentado em uma mesa ativa.
    - `Observing`: Assistindo a uma mesa sem apostar.
- **Criação Automática**: Se o jogador tentar entrar em um jogo com parâmetros específicos e não houver mesa disponível, o sistema pode sugerir a criação de uma nova.

### 14.3 📡 Sincronização em Tempo Real — WebSocket
- O estado do lobby (número de jogadores por mesa, mesas novas) deve ser transmitido via WebSocket para todos os usuários no estado `Lobby`.

---

## 15. 🕵️ Sistema Antifraude — Collusion, Bots e Chip Dumping

### 15.1 🤝 Detecção de Conluio — Collusion e Soft Play
- **Soft Play**: Detectar quando dois ou mais jogadores evitam apostar/aumentar contra cada other, mesmo com mãos fortes.
- **Coordenação**: Identificar padrões onde um jogador "limpa" o caminho para outro jogador ganhar o pot.

### 15.2 💸 Chip Dumping — Transferência Fraudulenta
- **Transferência Intencional**: Monitorar mãos onde um jogador faz all-in com uma mão absurdamente fraca contra um jogador específico, resultando em transferência massiva de fichas.
- **Análise de Histórico**: Cruzar dados de mãos repetitivas entre os mesmos usuários.

### 15.3 🤖 Detecção de Bots — Automação e GTO
- **Análise Temporal**: Detectar tempos de resposta constantes (ex: exatamente 2.0s para cada ação), sugerindo automação.
- **Padrões Matemáticos**: Identificar apostas que seguem rigorosamente a GTO (Game Theory Optimal) sem qualquer variação humana ou erro.

### 15.4 👥 Multi-accounting — Fingerprinting e Duplicidade
- **Fingerprinting**: Coleta de IP, User-Agent e Hardware ID.
- **Alerta de Duplicidade**: Bloquear ou alertar administradores quando múltiplas contas ativas compartilham o mesmo identificador de hardware ou IP em mesas diferentes simultaneamente.

---

**Próxima revisão:** Após implementação de side pots e split pot.

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-07):** S12 — Auth MFA + supply-chain CI; ações legais na mesa; settle pós-disconnect; liquidação de mão assinada (migração 017); smoke live 10 usuários/100 mãos com settlement verificado na VPS demo; branch codex/security-supply-chain fechada e documentada. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** VPS stack healthy (postgres, redis, api, frontend/Caddy). Migrations 001–017 aplicadas (017 hand settlement audit). Smoke live scripts/live-e2e-ten-users.mjs: run 202608070833 PASS (10 reg/100 mãos); run 202608070920 PASS com settlementsVerified=2 (assinatura + winner + payouts+rake=pote por mesa). Simulação motor 100k mãos release OK. Segundo lote sintético zte2e202608070920* removido; lote original zte2e202608070833* preservado (10 contas demo). Suíte histórica motor/API + gates supply-chain (Dependabot, audit, SBOM/Trivy workflows). Mock é o padrão. O único adaptador externo é o Asaas Sandbox, restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção permanecem desabilitados. Nenhum depósito com dinheiro real foi habilitado. Mesas continuam com dono único por processo; uma guarda persistente pausa a mesa após falha entre início e liquidação da mão, exigindo revisão/abort administrativo antes da reabertura. Liquidação de mão agora persiste settlement assinado (HMAC) e a API verifica assinatura no replay; históricos legados sem assinatura permanecem legíveis como não verificados.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
