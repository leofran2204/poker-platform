# 🃏 Regras de Negócio — Plataforma de Poker Online Texas Hold'em

**Versão:** 2.0  
**Data:** 2026-07-24  
**Status:** Documento Oficial — 100% Sincronizado e Implementado


---

## 0. 🦀 Stack Alvo — Rust para Toda a Plataforma (atualizado em 2026-07-03)

| Camada            | Linguagem                      | Responsabilidade                                  |
|-------------------|--------------------------------|---------------------------------------------------|
| **Backend**       | **Rust**                       | Motor de jogo, APIs, autenticação, lobby          |
| **IA e Dados**    | **Rust**                       | Antifraude, estatísticas, relatórios              |
| **Front-end**     | **Rust (Dioxus/WebAssembly)**  | UI para jogadores e administradores               |
| **Comunicação**   | **JSON**                       | Formato universal entre módulos                   |

> 📐 Detalhes completos em `Arquitetura-Motor/ARQUITETURA_MOTOR.md`.
> ✅ Stack 100% Rust desde 2026-07-03 — Python, TypeScript, Go e Node.js foram removidos.

---

## 1. 🎯 Visão Geral — Texas Hold'em Tradicional

Plataforma de poker online **Texas Hold'em Tradicional** inspirada no Full Tilt Poker. Suporta mesas de **Cash Game** e **Tournament** com até 9 jogadores.

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
- ✅ **Rake de 2.5% (cap R$ 6.00) + Regra do Centavo Ímpar (WSOP 68)** 100% implementados em `rake.rs` e `utils.rs`

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

O cashback é determinado pela **equity** (probabilidade de vencer) do perdedor no momento do all-in, calculada via enumeração exata heads-up (`get_heads_up_win_probability()`).

| Tier  | Equity do Perdedor | Cashback | Perfil do Rango |
|-------|---------------------|----------|-----------------|
| **0** | **46,0% – 59,9%**   | **7%**   | Mão Parelha / Coin Flip |
| **1** | **60,0% – 74,9%**   | **15%**  | Favorito Moderado |
| **2** | **75,0% – 84,9%**   | **25%**  | Grande Favorito |
| **3** | **≥ 85,0%**         | **35%**  | Favorito Esmagador / Bad Beat |
| —     | **< 46,0%**         | **0%**   | Blefe sem valor ou Zebra extrema |

### 11.2 ⚙️ Regras de Aplicação e Origem Financeira

- **Origem das Fichas (Custo ZERO para a Plataforma):** O cashback é 100% autofinanciado pelas próprias fichas acumuladas no pote da mão. Ele é descontado da fatia do(s) vencedor(es) daquele pote específico e entregue ao perdedor All-in.
- **Aplicação Unificada:** Funciona identicamente em **Cash Games (Ring Games)** (devolvendo R$/fichas reais) e em **Torneios (MTT/SNG)** (devolvendo fichas de torneio).
- **Múltiplos All-Ins e Fases Distintas:** Suporta múltiplos jogadores All-in em fases diferentes (Preflop=15%, Flop=25%, Turn=35%). Cada perdedor tem sua fase rastreada individualmente (`PlayerState::all_in_phase`).
- **Isolamento de Side Pots:** O cashback de um perdedor é calculado e descontado APENAS dos potes em que ele participou. Potes secundários nos quais o jogador não participou ficam 100% intocados.
- **Limite Máximo:** Cashback nunca excede 35% do valor perdido.
- **Anti-abuso:** Perder propositalmente para receber cashback é detectado pelo módulo antifraude.

### 11.3 📐 Exemplos

| Cenário                          | Equity | Tier | Perda   | Cashback |
|----------------------------------|--------|------|---------|----------|
| All-in preflop, A♠A♦ vs K♠K♦    | 82%    | 2    | R$ 200  | R$ 50    |
| All-in flop, set vs flush draw   | 65%    | 1    | R$ 100  | R$ 15    |
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
