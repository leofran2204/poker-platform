# 🃏 Regras de Negócio — Plataforma de Poker Online Texas Hold'em

**Versão:** 2.0  
**Data:** 2026-07-25  
**Status:** Documento normativo — implementação deve ser confirmada por testes e revisão de cada release


---

## 0. Escopo deste arquivo

**Dono das regras de pôquer e dinheiro.** Stack: [`Arquitetura-Motor/ARQUITETURA_MOTOR.md`](../Arquitetura-Motor/ARQUITETURA_MOTOR.md). Stakes, carteiras, PIX, ciclo: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md). Contrato de agentes: [`../AGENTS.md`](../AGENTS.md).

Cash exige **≥ 2 assentos** na mesma mesa. Registro com verificação de e-mail quando `REQUIRE_EMAIL_VERIFICATION=true`.

### 0.1. 💵 Arquitetura Financeira e Tipagem Estrita (`u64` Centavos)
- **Princípio da Precisão Bancária:** Todos os valores financeiros (saldo, apostas, stacks, potes, rake, buy-in e blinds) utilizam estritamente `u64` centavos inteiros no **backend** (`R$ 10,50` = `1050` centavos). Erros de arredondamento IEEE-754 flutuantes são eliminados na raiz.
- **Probabilidades e Estatísticas:** Mantidos em escala flutuante (`f64` entre `0.0` e `1.0` ou `0.0%` a `100.0%`) para cálculo de equidade e exibição de porcentagens.
- **Formatação de Exibição:** O frontend TypeScript converte centavos apenas na camada visual (`formatBrlFromCents` em `Frontend-Web/src/lib/money.ts`).
- **Depósito automatizado em Sandbox:** uma cobrança DePix exige usuário allowlisted, `Idempotency-Key`, valor entre 500 e 600.000 centavos e CPF/CNPJ enviado ao provedor sem armazenamento local.
- **Regra de crédito:** `checkout.processing` autenticado, com identificador e valor coincidentes e face ≤ R$ 50 (`5000` centavos), credita `balance_real` de forma provisória (face) e trava saques da conta até a liquidação. `checkout.completed` confirma (ajusta para o líquido recebido, sem recrédito). Acima do teto, só o `completed` credita. Cancelamento/expiração de provisório reverte até o saldo disponível (sem negativo; falta vira auditoria). Eventos repetidos e `approved` não alteram o saldo.
- **Regra de saque (AML):** saque de Jogo Real só com KYC aprovado e para chave PIX do **mesmo CPF do KYC** do titular; terceiros e CNPJ são recusados sem reservar saldo. Contas de bot nunca são debitadas nem jogam no Real.
- **Separação de ambientes:** a demo/staging usa DePix reconciliado (HMAC, teto de crédito provisório R$ 50, payout worker com teto/`HELD`). CPF/CNPJ não persiste em claro. Sem certificação de produção.

---

## 1. 🎯 Visão Geral

Plataforma de poker online inspirada no Full Tilt Poker (skin moderna, lobby denso, mesa de feltro). Suporta **Cash Game** e **Tournament** com variantes:

| Variante (`poker_variant`) | Baralho | Hole cards | Cap típico cash |
|----------------------------|---------|------------|-----------------|
| `holdem` (Texas Hold’em) | 52 | 2 | até 9 |
| `short_deck` (Texas Hold’em Short Deck) | 36 (6–A) | 2 | até 8 |
| `omaha` (Omaha 4) | 52 | **4** (usa exatamente 2 hole + 3 board) | até 6 |
| `pineapple_draw` (Brazilian Pineapple) | 52 | **3**; troca opcional de uma depois das apostas do flop e do turn; no river, descarte de uma antes da última rodada (showdown com 2, como no Hold’em) | até 6 (só Play Money por enquanto) |

**Carteiras:** Play Money (cash + MTT, reset diário) e Jogo Real (isolado). `money_mode` da mesa/torneio deve coincidir com o modo do cliente (`play` \| `real`).

**Catálogo publicado (blinds, frentes, MTT, PIX):** [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md). A migração 059 publica as quatro modalidades e separa Texas tradicional de Texas Short Deck. A migration 062 tira do catálogo o formato antigo do Brazilian Pineapple (2 privadas + 1 por street) e abre o Brazilian Pineapple atual (`pineapple_draw`), primeiro só no Play Money. Frentes cash **fixas** (`min_buy_in = max_buy_in`). Small blind pode **igualar** big blind.

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
| 5    | Straight          | 5     |
| 4    | Three of a Kind   | 4     |
| 3    | Two Pair          | 3     |
| 2    | One Pair          | 2     |
| 1    | High Card         | 1     |

### 2.3 🔄 Straight Especial — Ás como Carta Baixa
- **A-2-3-4-5** é um straight válido (Ás jogando como carta baixa)
- Straight normal: 5 cartas consecutivas (ex: 8-9-T-J-Q)

### 2.4 🂡 Omaha 4 — `omaha` (6-max cash + torneio)
- Baralho tradicional de **52** cartas
- Cada jogador recebe **4** hole cards
- No showdown: exatamente **2** hole + **3** community (melhor combo)
- Ranking **clássico** (igual ao Hold’em: sequência > trinca, full house > flush; wheel A-2-3-4-5)
- Implementação: `evaluate_hand_omaha`

### 2.5 🍍 Brazilian Pineapple — `pineapple_draw` (6-max cash + torneio; Play Money primeiro)
- **Nome e identificador:** o jogador vê **Brazilian Pineapple** (decisão do proprietário em 11/11/2026: o nome marca uma modalidade criada no Brasil). No código, no banco e nos históricos o identificador é `pineapple_draw`. O identificador `brazilian_pineapple` pertence ao formato antigo (2 privadas + 1 a cada street, sem descarte), que saiu do catálogo e permanece só no histórico.
- Substitui o formato antigo (decisão do proprietário em 09/11/2026). A migration 062 tira mesas e torneios do formato antigo do catálogo e abre o Brazilian Pineapple atual só no Play Money; o Jogo Real exige nova decisão e nova migration. Mãos, settlements e torneios antigos ficam preservados no histórico, identificados por `betting_rule_version`.
- Baralho tradicional de **52** cartas (ranks 2–A); até **6 jogadores**.
- **Ordem da mão (decisão do proprietário em 11/11/2026):**
  1. Cada jogador recebe **3** cartas privadas. Rodada de apostas. Não há troca no pré-flop.
  2. **Flop** (3 comunitárias). Rodada de apostas. Depois dela, **rodada de troca**.
  3. **Turn**. Rodada de apostas. Depois dela, **rodada de troca**.
  4. **River**. **Descarte obrigatório** de uma carta: cada jogador fica com **2**. Só então a última rodada de apostas.
  5. Showdown.
- **Rodada de troca:** quando as apostas do flop ou do turn fecham, cada jogador que ainda pode apostar decide **manter** as três cartas ou **trocar uma**. A carta descartada sai do jogo e outra vem do topo do baralho. Só depois a próxima comunitária é aberta.
- A troca e o descarte seguem a ordem das apostas pós-flop (primeiro jogador à esquerda do botão). A quantidade trocada (0 ou 1) é pública; as cartas descartadas e recebidas, não. A rodada de troca só admite manter ou trocar: tempo esgotado, saída da mesa ou desconexão mantêm as cartas, e a desistência acontece na vez seguinte, nas apostas.
- **Descarte do river:** obrigatório para quem ainda está na mão; a carta sai do jogo sem ser mostrada. Tempo esgotado, saída da mesa ou desconexão: o motor guarda as duas cartas que formam a melhor mão com a mesa (`pineapple_draw_best_discard`; em empate, descarta a carta mais baixa). Não há desistência nem aposta durante o descarte.
- **All-in não troca.** Quando no máximo um jogador ainda tem fichas para apostar, não há rodada de troca nem escolha de descarte: o board é completado direto. Quem está all-in, ou chega ao showdown ainda com três cartas, fica automaticamente com as duas melhores.
- **No showdown:** melhor mão de 5 entre as **2** privadas que restaram e as **5** comunitárias, como no Hold’em (duas, uma ou nenhuma privada). Quatro cartas do naipe na mesa e uma na mão formam flush. Ranking **clássico**: sequência > trinca, full house > flush; wheel A-2-3-4-5.
- O baralho cobre o pior caso: 6 × 3 privadas + 3 queimadas + 5 comunitárias + 12 trocas = 38 cartas.
- Histórico: ações públicas `keep`, `swap` e `discard`, sem a carta; `HandHistory.draws` guarda a carta descartada e a recebida (ausente no descarte do river) apenas para auditoria (campo ausente nas demais modalidades, preservando o formato assinado anterior).
- Implementação: `PokerVariant::PineappleDraw`, `evaluate_hand_pineapple_draw`, `pineapple_draw_best_discard`; rodada de troca e descarte em `game_loop.rs` (`begin_draw_round`, `player_draw`, `begin_discard_round`, `player_discard`, `settle_pending_discards`). API: ações `keep`, `swap`, `discard` (posição 0 a 2 em `card` ou `amount`) e `auto_discard`; o estado da mesa publica `drawing` e `discarding`. A heurística pública de troca de bots, laboratório e simulações (`pineapple_draw::suggested_swap`) não participa da liquidação.
- Referência de simulação (11/11/2026; 20 mil mãos por modalidade, 6 jogadores até o showdown, política passiva): a mão vencedora é sequência ou melhor em 73,0% das mãos no Brazilian Pineapple, 78,7% no Omaha 4 e 40,1% no Hold’em; trocas e descarte acrescentam até 18 decisões por mão. Teste `pineapple_draw_simulation` (feature `massive-tests`).

#### Aumento limitado ao pote antes do call — `pineapple_draw_v1`

Regra da modalidade em cash e torneios, desde a primeira mão e inclusive heads-up. Sem opção administrativa. Não altera outras modalidades.

- **Todas as rodadas:** sendo `P` o pote efetivamente formado antes do call, `C` o pagamento necessário e `B` a contribuição do jogador na rodada, o máximo total é `min(B + saldo, B + C + P)`. Sem patamares pré-flop ou teto de 4 BB. O ante está em `P` uma única vez e não faz parte de `B` nem abate `C`. Não é o Pot Limit tradicional.
- **Ante de 1 BB apenas do big blind:** o motor cobra o blind primeiro e depois o ante com o saldo restante. Blind incompleto não paga ante; ante parcial é permitido. O ante é dinheiro morto no pote principal e não aumenta a contribuição viva usada para elegibilidade aos potes paralelos. Nos torneios acompanha o BB do nível, sem cobrança adicional pelo ator.
- O incremento mínimo começa em 1 BB e acompanha o último aumento completo. Reabertura exige um incremento completo desde a última ação daquele jogador, inclusive cumulativamente por all-ins curtos. O curto não muda o mínimo; all-in acima do máximo é rejeitado. Check em rodada sem aposta preserva o direito de aumentar após abertura.
- Ações ilegais são rejeitadas antes de alterar fichas, turno ou histórico. Centavos inteiros, aritmética protegida, limites recalculados por ação. A rodada de troca acontece antes das apostas de cada street pós-flop e não mexe em fichas nem em limites.

Exemplo aprovado, três jogadores: blinds **R$ 0,50/R$ 1**, ante **R$ 1**, pote inicial **R$ 2,50**. Todos acompanhando formam **R$ 4**. A primeira abertura máxima é **para R$ 3,50**; dois calls formam **R$ 11,50**. O incremento passa a R$ 2,50: o próximo aumento completo mínimo é para R$ 6, podendo ultrapassar os antigos 4 BB quando o pote permitir.

Exemplo independente pós-flop: pote R$ 10, aposta R$ 5, `P=15`, `C=5`, `B=0`: máximo **para R$ 20**. Dois, três ou quatro participantes contribuindo R$ 20 na rodada formam **R$ 50, R$ 70 e R$ 90**. Após o aumento para R$ 20, `P=35`, `C=20`, `B=0`: próximo máximo **para R$ 55**.

Novas mãos registram `betting_rule_version=pineapple_draw_v1`. Mãos do formato antigo do Brazilian Pineapple mantêm `brazilian_pineapple_pot_before_call_v2`, `brazilian_pineapple_hybrid_v1` ou ausência de versão para a regra original; históricos, assinaturas e settlements são preservados e nunca reliquidados. Publicação conjunta de motor, API, frontend, Academy e migration 062 exige CI aprovado, backup validado e nenhum torneio do formato antigo em andamento (a migration aborta nesse caso). Mesas ocupadas são fechadas e as fichas dos assentos voltam à carteira no início da API; inscrições do Jogo Real em torneio do formato antigo não iniciado são canceladas sem reembolso, por decisão do proprietário (09/11/2026), e ficam registradas em `audit_logs`; o torneio Play Money passa ao Brazilian Pineapple atual com os inscritos. Evidência de publicação pertence a `QUALITY.md` e `DEVELOPMENT_LOG.md`.

### 2.6 🂡 Texas Hold’em Short Deck — `short_deck` (8-max cash + torneio)
- Baralho de **36** cartas (ranks 6–A; sem 2–5)
- Duas cartas fechadas, cinco comunitárias; melhor combinação de cinco como no Hold’em
- Ranking Six Plus: **trinca > sequência** e **flush > full house**; wheel **A-6-7-8-9**
- Implementação: `create_short_deck` / `evaluate_hand_short_deck`

---

## 3. 🪑 Estrutura da Mesa — Configuração e Parâmetros

### 3.1 ⚙️ Configuração da Mesa — Parâmetros
| Parâmetro     | Tipo     | Validação                          |
|---------------|----------|------------------------------------|
| `name`        | string   | 3-100 caracteres                   |
| `gameType`    | enum     | `cash` \| `tournament`             |
| `smallBlind`  | number   | > 0; pode ser **igual** a `bigBlind` |
| `bigBlind`    | number   | > 0; `smallBlind ≤ bigBlind`       |
| `minBuyIn`    | number   | > 0 (cash oficial: = `maxBuyIn`)   |
| `maxBuyIn`    | number   | > 0                                |
| `maxPlayers`  | int      | 2–9 (Texas tradicional: 9; Texas Short Deck: 8; Omaha 4 e Brazilian Pineapple: 6) |
| `poker_variant` | string | `holdem` \| `short_deck` \| `omaha` \| `pineapple_draw` |
| `money_mode`  | string   | `play` \| `real`                   |
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
   - A plataforma opera 100% em **centavos inteiros (`u64`)** em todas as camadas (Banco de Dados, Motor de Jogo, APIs REST e WebSockets).
   - O frontend (`Frontend-Web`) converte os centavos `u64` para o formato legível `R$ x,xx` exclusivamente na camada visual de renderização.
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
- **Side pots:** implementado (`Motor-Rust/src/side_pots.rs` `calculate_side_pots()`; all-ins multiway; conservação `payouts + rake == before` validada em `cash_catalog_10k_hands` e `simulated_100` 100k/mesa)

### 6.2 🤝 Distribuição — Vencedor e Split Pot
- Vencedor recebe o pot inteiro
- Em caso de empate (tie), pot dividido (split) pela Regra do Centavo Ímpar (§4.4)
- Split pot implementado (`utils::dividir_pote_empatado`, WSOP/TDA Regra 68)

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
- ✅ **Fee Torneios: 15% por cima do Buy-in (S22, antes 7%):** ex. R$ 10 + R$ 1,50; freeroll sem fee; taxa 0% em Re-buys e Add-ons. Na migration `060`, o fee passa a compor o NGR mensal do Agente ZT direto (`source_type='fee'`, `program_version=2`); a comissão só é creditada no fechamento.
- ✅ **Cancelamento de inscrição:** só pré-start; reembolsa buy-in + fee integralmente. No programa Agente ZT, gera dedução idempotente apenas das receitas daquela inscrição, preservando o ledger e os fechamentos. Se o mês original estiver fechado, o estorno entra no mês atual. Linhas legadas sem referência de torneio exigem conciliação manual; não são apagadas nem estornadas por aproximação. Auditoria `MTT_UNREGISTER`. Pós-start não cancela.

- **Premiação MTT:** inclui eliminados que terminaram nas posições pagas. Se houver menos inscritos que posições premiadas, os percentuais das posições existentes são normalizados proporcionalmente (decisão do proprietário em 02/10/2026: 50%/30%/20% com dois inscritos vira 62,5%/37,5%). O cálculo monetário usa inteiros; centavos residuais vão às maiores frações descartadas, com desempate pela melhor posição. A soma dos prêmios é exatamente o prize pool. Créditos, posições pagas, finalização e auditoria são persistidos na mesma transação, sem novo crédito em repetição.

### 9.3 🏢 Divisão Financeira B2B SaaS (Rake Split 15% / 85%)
- **Ordem de Execução Inviolável**: Potes brutos → Cálculo de Rake → **Split B2B (15% Plataforma Zerotilt / 85% Clube Locatário)** → Aplicação do Loss Deflator sobre o pote líquido pós-rake → Distribuição dos prêmios.
- **Roteamento de Ledger**: O valor do Rake do Clube (`club_rake`) é injetado diretamente no saldo administrativo da tabela `clubs` (`balance`) ao final de cada mão (apenas mesas com `club_id`). O fee da plataforma (`platform_fee`) é contabilizado para a Zerotilt.
- **Split 15/85 no motor:** ainda existe para mesa com `club_id` (código legado). **Não é** a regra comercial desta rede.
- **Agente ZT Poker (migration `060`, publicada na demo em 01/10/2026):** um único nível. O agente recebe **30% do NGR dos jogadores ligados diretamente ao seu `users.sponsored_by`** e mais **5 pontos percentuais** no mês em que atingir a meta registrada antes do ciclo. Não há comissão de segundo nível, por cadastro, depósito, saldo, buy-in, prêmio ou perda do jogador.
- **Corte auditável:** linhas novas usam `program_version=2`, ficam separadas por `money_mode` e não creditam antecipadamente. O fechamento mensal idempotente desconta ajustes identificados, aplica 30%/35% e credita ZT Points ou saldo de comissão Real. O ledger 18%/12% permanece somente como histórico `program_version=1`. Detalhe: `PLANO_GO_TO_MARKET_AGENTE_ZT.md`.
- **Proteção da margem:** bots e autoindicações ficam fora da apuração. Em mesas privadas B2B, apenas a parcela retida pela plataforma (15% do rake) compõe a receita atribuível aos agentes; a parcela do clube não é remunerada novamente. Em mesas públicas, a base é o rake atribuível aos participantes, dividido igualmente e com resíduo de centavos retido pela plataforma.
- **Conciliação mensal:** metas só podem ser definidas para meses futuros. Sem meta positiva, vale 30%. Fechamento manual a partir do dia 25 do mês seguinte, em ordem cronológica por agente/carteira, exige confirmação da conciliação e dos totais apresentados. Comissão calculada em inteiros, com piso zero de NGR; deduções excedentes passam ao próximo ciclo. O saldo Real de comissões é separado da carteira do jogador; este fluxo não executa saques. Recompensas, tributos e custos atribuíveis devem ser conciliados pelo administrador antes de fechar.
- **Torneios multi-mesa:** 3 mesas físicas por torneio e `max_players = 3×table_max`; Texas tradicional até 27, Texas Short Deck até 24, Omaha 4 e Brazilian Pineapple até 18. O admin pode criar Texas Short Deck com mesa até 8, definindo data e horário. Rebalance com desnível >1; consolidação na FT; gameplay WS ao vivo; run-out automático em all-in geral; halt auditável (`MTT_TABLE_HALTED`).
- **Play Money × Jogo Real:** fichas Play Money não têm valor em dinheiro; o Jogo Real usa carteira separada com PIX DePix na demo (ver STATUS). O saque de saldo de clube (`POST /api/admin/clubs/:id/withdraw`, legado B2B) apenas debita `clubs.balance`: não envia PIX, não cria transação de carteira e não grava auditoria.

---

## 10. ⚡ Estados Especiais — All-In, Timeout e Desconexão

### 10.1 🔥 All-In — Jogador sem Fichas
- Jogador com `chips == 0` é marcado `isAllIn`
- Continua elegível para ganhar o pot
- Não precisa agir mais

### 10.2 ⏰ Timeout — Auto-Fold
- Após `timeLeft == 0`, jogador é auto-folded
- Timer reinicia a cada ação

### 10.3 🔌 Desconexão — Cash-out após graça
- WebSocket cai: o jogador senta out (fold automático se for a vez); o assento fica reservado **45 s** para reconectar (F5).
- Passada a graça, e só **entre mãos**, o actor devolve o escrow à carteira (`SEAT_CASHED_OUT`) e libera a cadeira. Leave HTTP explícito cash-out imediato (ainda bloqueado no meio da mão).
- Boot da API: assentos `ACTIVE` sem `table_hand_recovery_guards` são reconciliados (órfãos após crash/kill de cliente).
- O lobby lista mesas `OPEN` mesmo lotadas (`Cheia`); não some o catálogo porque a mesa está 9/9.

---

## 11. 🛡️ Loss Deflator — Cashback Progressivo por Equity

O **Loss Deflator** é um sistema de cashback automático que devolve parte das perdas em all-in calls quando o jogador tinha alta probabilidade de vencer (equity) mas perdeu por azar (bad beat).

### 11.1 🎯 Modelo Baseado em Equity

O cashback é determinado pela **equity do perdedor no instante em que o all-in é pago**. O cálculo heads-up é determinístico: enumeração quando viável e Monte Carlo determinístico nos espaços maiores (`get_heads_up_win_probability()`). A fase da mão serve apenas para reconstruir quais cartas já estavam abertas; **preflop, flop, turn ou river nunca determinam o percentual**.

**Onde vale (decisão do proprietário em 08/11/2026):** em **todas as modalidades** (Texas Hold’em, Texas Hold’em Short Deck, Omaha 4 e Brazilian Pineapple), nas mesas **cash e nos torneios**, em **Play Money e Jogo Real**. Só fica desligado (`skip_loss_deflator`) no laboratório da Academy, que usa fichas didáticas, e nas campanhas de estresse. Nos torneios não há rake por mão, então a base é o pote inteiro e o cashback sai em fichas do torneio. Termos 6.4 descrevem a mesma regra.

**Cálculo por modalidade:** a entrada de liquidação é `get_multiway_win_probability_for_variant` (`Motor-Rust/src/loss_deflator.rs`), que valida baralho, cartas e modalidade.

| Modalidade | Cartas no snapshot | Método |
|---|---|---|
| Texas Hold’em | 2 privadas por jogador | Ranking clássico; enumeração quando cabe, senão Monte Carlo determinístico de até 500.000 boards sem repetição |
| Texas Hold’em Short Deck | 2 privadas, baralho de 36 | Ranking Short Deck; enumeração exata (até 201.376 boards heads-up pré-flop) |
| Omaha 4 | 4 privadas | Exatamente 2 privadas + 3 comunitárias; exata até 250.000 desfechos (ex.: flop e turn), senão Monte Carlo determinístico de 120.000 amostras (pré-flop) |
| Brazilian Pineapple | As privadas que cada jogador tinha no pagamento: 3 antes do descarte do river, 2 depois (quem está all-in não troca mais) | Melhor mão de 5 com no máximo 2 privadas; de 3 privadas contam as 2 melhores em cada desfecho (o descarte ótimo). Completa só o board. Trocas futuras de quem ainda pode apostar não entram no cálculo (regra pública e determinística). Exata até 250.000 desfechos (flop e turn heads-up), senão Monte Carlo de 120.000 amostras (pré-flop) |

No Monte Carlo, a semente vem das cartas conhecidas: a mesma mão sempre gera a mesma equity, o que permite auditar a liquidação. Com 120.000 amostras independentes, a margem de três desvios no pior caso é de cerca de 0,43 ponto percentual; perto dos limites de faixa (56%, 66%, 76% e 86%), o tier depende dessa estimativa. A pontuação rápida de 5 cartas usada no Omaha/Brazilian Pineapple foi conferida contra o avaliador oficial nas 2.598.960 mãos possíveis (`classic_five_card_score_matches_evaluator_exhaustively`). Equity inclui frações de empate; não é apenas frequência de vitória isolada.

| Tier  | Equity do Perdedor | Cashback | Perfil do Rango |
|-------|---------------------|----------|-----------------|
| **0** | **56,0% – 65,9%**   | **7%**   | Favorito leve |
| **1** | **66,0% – 75,9%**   | **15%**  | Favorito moderado |
| **2** | **76,0% – 85,9%**   | **25%**  | Grande favorito |
| **3** | **86,0% – 99,9%**     | **35%**  | Favorito esmagador / bad beat extrema |
| —     | **< 56,0%**         | **0%**   | Não elegível |

> Forma exata no motor (`LossDeflatorTier::from_loser_equity`): topo é `[86%, 100%)`. Equity 100% não tem tier — quem tem 100% não perde o runout, logo não há perdedor elegível; "99,9%" é o arredondamento de exibição.

### 11.2 ⚙️ Regras de Aplicação e Origem Financeira

- **Ordem financeira obrigatória:** formar main pot e side pots → retirar o rake de cada pote → calcular o Loss Deflator somente sobre os potes elegíveis já líquidos → concluir os pagamentos.
- **Origem das Fichas:** O cashback sai dos potes líquidos da própria mão. É descontado da fatia do(s) vencedor(es) do pote elegível e entregue ao perdedor all-in; não cria fichas novas nem debita o caixa da plataforma.
- **Modos de saldo:** a regra vale igualmente em Play Money e Jogo Real, sem misturar as carteiras: o cashback sai do pote da própria mesa, na moeda dela. Disponibilidade de carteiras e mesas é definida no STATUS, não por este exemplo financeiro.
- **Múltiplos All-Ins e Fases Distintas:** Cada perdedor possui um snapshot individual de fase, board, das próprias privadas e das cartas dos oponentes, fixado no primeiro pagamento do seu all-in (inclusive pagamento all-in menor). No Brazilian Pineapple, o snapshot guarda as privadas do instante do pagamento (3 antes do descarte do river, 2 depois); quem está all-in não troca mais. A fase não escolhe o tier.
- **Equity multiway:** preserva todos os oponentes ainda na mão no instante do pagamento do all-in. Um fold posterior não retira esse oponente do snapshot nem altera retroativamente a equity. A elegibilidade financeira continua sendo apurada pelos potes finais; preservar cartas para equity não dá direito ao pote a quem desistiu. Os métodos por modalidade estão na tabela acima. A parcela dos empates é dividida pelo número de vencedores.
- **Isolamento de Side Pots:** O cashback de um perdedor é calculado e descontado APENAS dos potes líquidos pós-rake em que ele participou. Side pots nos quais não era elegível ficam intocados.
- **Teto compartilhado por pote:** Quando vários perdedores all-in se qualificam no mesmo pote, a maior faixa elegível define o teto percentual único daquele pote líquido. Os perdedores dividem esse teto proporcionalmente aos pedidos individuais; faixas iguais dividem em partes iguais. Centavos residuais seguem a ordem dos assentos a partir do botão. Um side pot só entra no rateio dos perdedores elegíveis a ele.
- **Limite Máximo:** o teto é de até 35% de cada pote líquido elegível, compartilhado conforme a regra acima. A base não é o aporte individual nem uma garantia de ressarcimento de 35% da perda pessoal.
- **Anti-abuso:** módulos antifraude produzem sinais para revisão. Não existe garantia de detectar toda perda proposital.

### 11.3 📐 Exemplos

Exemplos aritméticos com equity **fornecida**, um único perdedor elegível e base já líquida de rake. Não são estimativas de mãos sem board/ranges definidos.

| Equity fornecida | Pote líquido elegível | Percentual | Cashback |
|---|---:|---:|---:|
| 55% | 20.000 centavos | 0% | 0 |
| 60% | 20.000 centavos | 7% | 1.400 |
| 70% | 20.000 centavos | 15% | 3.000 |
| 80% | 20.000 centavos | 25% | 5.000 |
| 90% | 20.000 centavos | 35% | 7.000 |

Um exemplo de equity exata com cartas e regressão automatizada está em [`LOSS_DEFLATOR_EXEMPLOS.md`](LOSS_DEFLATOR_EXEMPLOS.md).

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
| Ranking de mãos     | `Motor-Rust/src/deck.rs`         | `evaluate_hand()` / `evaluate_hand_short_deck()` / `evaluate_hand_omaha()` / `evaluate_hand_pineapple_draw()` |
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

> **Estado no código (08/11/2026):** ligados ao jogo estão o escore de bot por tempo de reação no ator de cash (15.3, pode bloquear a ação) e a guarda de rede na entrada da mesa cash (15.5). Colusão, chip dumping e multi-conta (15.1, 15.2, 15.4) existem como módulos testados em `Motor-Rust/src/antifraud/`, mas a API ainda não os consulta; os itens dessas seções são o objetivo, não comportamento ativo. Detalhes em `ARQUITETURA_E_APIS.md` § 5.

### 15.1 🤝 Detecção de Conluio — Collusion e Soft Play
- **Soft Play**: Detectar quando dois ou mais jogadores evitam apostar/aumentar contra cada outro, mesmo com mãos fortes.
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

### 15.5 🌐 Guarda de rede na mesa cash (Termos 2.3) — ativa
- **Regra:** `POST /api/lobby/join` recusa com 403 a entrada quando outra conta tem assento `ACTIVE` na mesma mesa com o mesmo IP público ou a mesma sub-rede (/24 em IPv4, /64 em IPv6). Repetir a entrada da própria conta continua idempotente.
- **Privacidade:** só impressões HMAC (`KYC_DATA_PEPPER`, domínio `zero-tilt-seat-network-v1`) ficam em `cash_game_seats` (migration 061) e são apagadas por gatilho quando o assento deixa de estar ativo. O endereço em claro não é gravado; a recusa gera `CASH_SEAT_SAME_NETWORK_BLOCKED` em `audit_logs` só com o id da mesa.
- **Fora da comparação:** endereços não públicos (loopback, rede privada, link-local, CGNAT) e os listados em `SEAT_IP_GUARD_EXEMPT_IPS`. O IP vem da mesma regra do rate limit (cabeçalhos do proxy só com `TRUST_PROXY_HEADERS`).
- **Limites conhecidos:** redes móveis e empresas podem colocar desconhecidos atrás do mesmo IP público (falso positivo: o jogador escolhe outra mesa). Torneios ainda não aplicam a guarda, porque o coordenador distribui os assentos.

---

## 16. 🛡️ Proteção do Jogador e Acesso ao Jogo Real

- **Play Money:** permanece disponível sem KYC e não consome os limites de Jogo Real.
- **Maioridade:** novos cadastros do fluxo de produto informam data de nascimento e declaram 18+. A API rejeita data incompatível com maioridade.
- **KYC manual:** depósito e nova entrada em mesa/torneio de Jogo Real exigem `kyc_status = verified`. CPF não é persistido em claro: somente HMAC com pepper dedicado e os quatro últimos dígitos.
- **Limites voluntários:** depósito e perda têm tetos diário, semanal e mensal; tempo de Jogo Real tem teto diário em minutos. Reduções são imediatas. Aumento ou remoção só entra em vigor após 24 horas.
- **Perda computada:** cash considera assentos reais encerrados (`buy_in - cash_out`, nunca negativo); torneios reais consideram entradas, fees e reentradas, descontado prêmio registrado.
- **Tempo:** heartbeat de mesa real acumula no máximo 60 segundos por atualização. Ao atingir o teto, novas entradas e ações reais são bloqueadas e o jogador é colocado em `Sit-out`.
- **Autoexclusão:** opções de 24h, 7d, 30d, 180d ou permanente; exige confirmação textual e bloqueia imediatamente depósitos, novas entradas e ações de Jogo Real. Não há cancelamento antecipado pela API.
- **Recuperação de senha:** código de seis dígitos, validade de 15 minutos, cinco tentativas, uso único e revogação de tokens anteriores após a troca.
- **Suporte:** chamados autenticados ficam persistidos e podem ser respondidos pela administração; atendimento nunca solicita senha, código de e-mail ou MFA.

---

**Próxima revisão:** Após implementação de side pots e split pot.

<!-- DOCUMENTATION_SYNC:START -->
> **S27** (2026-11-09) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
