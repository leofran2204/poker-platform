# Guia de Exemplos Práticos — Loss Deflator (Bad Beat Cashback)

**Atualizado:** 2026-07-30 | **Status:** Regra normativa 56/66/76/86; exemplos em fichas playmoney e potes pós-rake.

Este documento serve como material de apoio técnico e educacional para o funcionamento do módulo **Loss Deflator** (`loss_deflator.rs`). Aqui você encontrará definições visuais, conceitos probabilísticos e simulações matemáticas reais de mãos de poker para cada um dos Tiers de cashback.


---

## 1. Conceitos Fundamentais

### 📐 O que é Equity?
A **Equity** representa a sua **chance percentual matemática de vencer o pote** em um determinado momento da mão, caso nenhuma outra ação de aposta ocorra e todas as cartas restantes sejam distribuídas. 

*   Ela é calculada simulando todas as combinações possíveis de cartas comunitárias (*board*) restantes e dividindo o número de vitórias pelo total de cenários.
*   A Equity é dinâmica: ela muda drasticamente a cada rodada (Pré-flop ➔ Flop ➔ Turn ➔ River) à medida que novas cartas são reveladas.
*   **O Loss Deflator só é ativado se o perdedor tinha equity ≥ 56% no instante em que o all-in foi pago.**
*   **A fase não determina o tier:** ela apenas define quantas cartas do board já eram conhecidas no snapshot.
*   **A base financeira é pós-rake:** primeiro o rake sai do main pot e de todos os side pots; depois aplica-se o percentual somente aos potes líquidos em que o perdedor era elegível.

---

## 2. Tipos de Draws (Projetos de Sequência e Flush)

Draws são projetos de mãos que ainda não estão prontas, mas que podem se tornar mãos muito fortes (como Sequências ou Flushes) se as cartas certas baterem no *board*. A força e a Equity desses draws dependem do número de **outs** (cartas restantes no baralho que completam o jogo).

### 🕳️ Gutshot (Sequência Interna / Broca)
É quando o jogador tem 4 cartas da sequência, mas falta exatamente **uma carta específica no meio** para completá-la.
*   **Outs:** **4 outs** (ex: existem apenas quatro cartas daquele valor específico no baralho de 52 cartas).
*   **Chance de bater no River:** ~9% (com 1 carta por vir).
*   **Exemplo:**
    *   Sua mão: `8♦ 7♥`
    *   Board: `J♠ T♣ 2♦` (faltando apenas o `9` no meio para formar a sequência 7-8-**9**-T-J).

### 👐 OESD (Open-Ended Straight Draw / Sequência Aberta)
É quando o jogador possui **4 cartas consecutivas** e pode completar a sequência por **qualquer uma das duas pontas**.
*   **Outs:** **8 outs** (4 cartas que completam a ponta de cima e 4 que completam a de baixo).
*   **Chance de bater no River:** ~18% (com 1 carta por vir).
*   **Exemplo:**
    *   Sua mão: `9♠ 8♦`
    *   Board: `T♥ 7♣ 2♠` (qualquer `6` ou `J` completa a sequência: **6**-7-8-9-T ou 7-8-9-T-**J**).

### ⛈️ Combo Draw (Monster Draw / OESD + Flush Draw)
Ocorre quando o jogador tem, simultaneamente, um projeto de sequência aberta (OESD) e um projeto de Flush (4 cartas do mesmo naipe).
*   **Outs:** **15 outs** (9 outs de flush + 8 outs de sequência, subtraindo as 2 cartas que dão ambos e já foram contadas).
*   **Chance de bater no River:** ~34% (com 1 carta por vir).
*   **Exemplo:**
    *   Sua mão: `8♠ 7♠`
    *   Board: `T♠ 9♣ 4♠ 2♦` (Turn)
    *   Qualquer espada (♠) dá um Flush; qualquer `6` ou `J` dá uma sequência.

---

## 3. Tabela Comparativa de Draws (Flop ➔ River)

A tabela abaixo resume a probabilidade de um draw bater a partir do Flop (2 cartas por vir) ou a partir do Turn (1 carta por vir):

| Tipo de Draw | Outs no Baralho | Chance no River (Turn) | Chance até o River (Flop) |
| :--- | :---: | :---: | :---: |
| **Gutshot (Broca)** | 4 | ~9% | ~17% |
| **OESD (Sequência Aberta)** | 8 | ~18% | ~31% |
| **Flush Draw (4 do mesmo naipe)** | 9 | ~20% | ~35% |
| **Combo Draw (OESD + Flush)** | 15 | ~34% | ~54% |

---

## 4. Exemplos Reais do Loss Deflator por Tier

Nas tabelas a seguir, todas as perdas simuladas foram padronizadas em **500** fichas/moeda (sem moedas específicas).

### Tier 3 — 35% (Equity do Perdedor ≥ 86%)
O perdedor tinha chance quase nula de perder a mão (Bad Beats extremos).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♠ A♦` vs `7♥ 2♣` | **12%** *(88% Equity)* | Sem board <br> *(72o acerta dois pares milagrosos)* | 500 | **Tier 3 (35%)** | **175** |
| **Flop** | `A♦ A♣` *(set)* vs `K♦ Q♠` | **3%** *(97% Equity)* | Board: `A♠ 7♣ 2♥` <br> *(KQ acerta J+T runner-runner para Broadway)* | 500 | **Tier 3 (35%)** | **175** |
| **Turn** | `Q♣ Q♦` *(set)* vs `J♦ 9♥` | **9%** *(91% Equity)* | Board: `Q♥ 8♠ 3♣ 2♦` <br> *(J9 acerta T no river para sequência)* | 500 | **Tier 3 (35%)** | **175** |

---

### Tier 2 — 25% (Equity do Perdedor 76%–85,9%)
O perdedor era claro favorito, mas o oponente possuía um projeto com alguns outs (ex: gutshot simples).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♥ A♣` vs `K♠ K♦` | **18%** *(82% Equity)* | Sem board <br> *(KK acerta K no flop para trinca)* | 500 | **Tier 2 (25%)** | **125** |
| **Flop** | `A♠ A♣` *(overpair)* vs `6♣ 5♦` | **17%** *(83% Equity)* | Board: `K♦ 7♠ 3♥` <br> *(65 acerta 4 no river para sequência)* | 500 | **Tier 2 (25%)** | **125** |
| **Turn** | `A♦ A♣` *(overpair)* vs `J♥ T♥` | **18%** *(82% Equity)* | Board: `Q♠ 9♣ 4♦ 2♠` <br> *(JT acerta K ou 8 no river para sequência)* | 500 | **Tier 2 (25%)** | **125** |

---

### Tier 1 — 15% (Equity do Perdedor 66%–75,9%)
O perdedor era favorito moderado, mas o oponente tinha bons draws (ex: OESD simples).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `J♠ J♦` vs `A♣ T♥` | **29%** *(71% Equity)* | Sem board <br> *(AT acerta Ás no flop)* | 500 | **Tier 1 (15%)** | **75** |
| **Flop** | `A♠ A♣` *(overpair)* vs `9♣ 8♦` | **31%** *(69% Equity)* | Board: `T♥ 7♠ 2♣` <br> *(98 completa sequência aberta no river)* | 500 | **Tier 1 (15%)** | **75** |
| **Turn** | `A♠ A♣` *(overpair)* vs `8♠ 7♠` | **34%** *(66% Equity)* | Board: `T♠ 9♣ 4♠ 2♦` <br> *(87♠ completa flush ou sequência — 15 outs)* | 500 | **Tier 1 (15%)** | **75** |

---

### Tier 0 — 7% (Equity do Perdedor 56,0%–65,9%)
Cenários em que o perdedor era favorito leve e acabou superado.

| Fase do All-In | Mão do perdedor VS vencedor | Equity do perdedor no all-in | Board / Desfecho | Pote líquido elegível | Tier | Devolução |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♠ Q♦` vs `K♥ J♥` | **≈62%** | `2♣ 5♦ 7♠ J♣ 9♦` | 500 | **Tier 0 (7%)** | **35** |
| **Flop** | overpair vs combo draw | **60%** | O draw completa no river | 500 | **Tier 0 (7%)** | **35** |
| **Turn** | par maior vs duas overcards + draw | **58%** | O oponente acerta um out no river | 500 | **Tier 0 (7%)** | **35** |

---

### Sem Cashback — Equity do Perdedor < 56%
Perdas em que o jogador não alcançava a faixa mínima de 56% no snapshot do all-in.

| Fase do All-In | Mão do perdedor VS vencedor | Equity do perdedor no all-in | Board / Desfecho | Pote líquido elegível | Tier | Devolução |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `9♠ 9♦` vs `A♣ K♣` | **≈54%** | Um Ás aparece no board | 500 | **N/A** | **0** |
| **Flop** | projeto de sequência vs par feito | **48%** | O projeto não completa | 500 | **N/A** | **0** |
| **Turn** | duas overcards vs par | **≈14%** | O par segura | 500 | **N/A** | **0** |

---

## 5. 💡 Origem das Fichas e Exemplos Práticos de Múltiplos All-Ins

> ⚠️ **Princípio Fundamental:** os exemplos usam fichas playmoney e valores de pote **já líquidos de rake**. A ordem é main pot/side pots → rake → Loss Deflator nos potes líquidos elegíveis → pagamentos. O vencedor daquele pote financia o cashback; um side pot do qual o perdedor não participou fica intocado.

### 🎲 6 Casos Práticos Reais (Cash Games & Torneios)

#### Exemplo 1: Heads-up Simples (1 contra 1) — All-in no Flop
* **Situação:** Ana (100) vs Beto (100). Pote líquido elegível após o rake = **200 fichas playmoney**.
* **Ação:** Ana vai all-in no flop com `A♠ A♥`; sua equity registrada é **80%**, portanto o tier é 25%. Beto paga com `9♣ 8♣`.
* **Resultado:** Beto acerta um Flush no River e vence a mão.
* **Cálculo:** Cashback da Ana = 25% de R$ 200 = **R$ 50,00**.
* **Distribuição Final:**
  * Beto (Vencedor): R$ 200 - R$ 50 = **R$ 150,00**.
  * Ana (Perdedora All-in): Recebe **R$ 50,00**.
  * ⚖️ *Soma Total:* 150 + 50 = **R$ 200,00** (Conservação exata de 100%).

#### Exemplo 2: Múltiplos Stacks (Main Pot + Side Pot) — Todos All-in no Pré-flop
* **Situação:** Carlos (R$ 50), Diego (R$ 100) e Eduardo (R$ 100).
* **Potes Formados:**
  * **Main Pot:** R$ 150 (R$ 50 de cada). Elegíveis: Carlos, Diego, Eduardo.
  * **Side Pot:** R$ 100 (R$ 50 de Diego e Eduardo). Elegíveis: Diego e Eduardo.
* **Ação:** Todos vão all-in no pré-flop. Carlos e Diego têm **70% de equity** em seus confrontos elegíveis, portanto recebem a faixa de 15%. **Eduardo vence a mão inteira.**
* **Cálculo dos cashbacks sobre potes líquidos pós-rake:**
  * Carlos: 15% de 150 = **22,50**.
  * Diego: 15% de 250 = **37,50**.
* **Distribuição Final:**
  * Eduardo (Vencedor de tudo): R$ 250 - 22,50 - 37,50 = **R$ 190,00**.
  * Carlos: Recebe **R$ 22,50**.
  * Diego: Recebe **R$ 37,50**.
  * ⚖️ *Soma Total:* 190 + 22,50 + 37,50 = **R$ 250,00**.

#### Exemplo 3: Proteção de Side Pot (Respeito a quem não disputou)
* **Situação:** Fernando (R$ 20), Gabriela (R$ 100), Hélio (R$ 100).
* **Potes Formados:** Main Pot = R$ 60 | Side Pot = R$ 160 (Gabriela e Hélio).
* **Ação:** Fernando vai all-in no pré-flop com **70% de equity** (tier 15%). Gabriela e Hélio vão all-in no turn; suas fases não definem qualquer percentual.
* **Showdown:** **Gabriela** ganha o Main Pot (R$ 60). **Hélio** ganha o Side Pot (R$ 160). Fernando perdeu.
* **Cálculo:**
  * Fernando: 15% do main pot líquido de 60 = **9**.
  * Esse R$ 9,00 sai APENAS da Gabriela (ganhadora do Main Pot).
  * O pote de Hélio (Side Pot de R$ 160) fica **100% intocado**, pois Fernando não participou do Side Pot!
* **Distribuição Final:**
  * Gabriela: R$ 60 - R$ 9 = **R$ 51,00**.
  * Hélio: **R$ 160,00** (Intocado!).
  * Fernando: **R$ 9,00**.
  * ⚖️ *Soma Total:* 51 + 160 + 9 = **R$ 220,00**.

#### Exemplo 4: All-ins em fases diferentes, tiers definidos pela equity
* **Situação:** Igor (40), João (100), Lucas (100); os valores abaixo já são líquidos de rake.
* **Pré-flop:** Igor all-in; sua equity no snapshot é **70%**, então recebe 15%.
* **Turn:** João all-in; sua equity no snapshot é **90%**, então recebe 35%.
* **Showdown:** **Lucas** vence o Main Pot (R$ 120) e o Side Pot (R$ 120).
* **Cálculos:**
  * Igor: 15% de 120 = **18**.
  * João: 35% de 240 (sua participação elegível total) = **84**.
* **Distribuição Final:**
  * Lucas (Vencedor): R$ 240 - R$ 18 - R$ 84 = **R$ 138,00**.
  * Igor: **R$ 18,00**.
  * João: **R$ 84,00**.
  * ⚖️ *Soma Total:* 138 + 18 + 84 = **R$ 240,00**.

#### Exemplo 5: Pote Dividido (Split Pot) entre 2 Vencedores
* **Situação:** Marcelo (R$ 100), Natália (R$ 100), Otávio (R$ 100). Pote Total = R$ 300.
* **Ação:** Marcelo vai all-in no flop com **80% de equity** (tier 25%). Natália e Otávio empatam na melhor mão.
* **Cálculo:** Marcelo perdeu R$ 300 (25%) = **R$ 75,00**.
* **Distribuição:** Os R$ 75,00 são descontados meio a meio dos dois vencedores (R$ 37,50 cada).
  * Natália: R$ 150 - R$ 37,50 = **R$ 112,50**.
  * Otávio: R$ 150 - R$ 37,50 = **R$ 112,50**.
  * Marcelo: **R$ 75,00**.
  * ⚖️ *Soma Total:* 112,50 + 112,50 + 75 = **R$ 300,00**.

#### Exemplo 6: Equity abaixo do mínimo no river
* **Situação:** Pedro (100) vs Quênia (100). Com o board completo, Pedro paga all-in no river já drawing dead.
* **Resultado:** Quênia vence.
* **Cálculo:** A equity de Pedro no snapshot era 0%, abaixo de 56%; por isso o cashback é 0%. O motivo é a equity, não a fase.
* **Distribuição Final:** Quênia recebe **R$ 200,00 integralmente**. Pedro recebe R$ 0,00.

#### Exemplo 7: Tier 0 — Faixa mínima de 7% (equity de 56,0% a 65,9%)
* **Situação:** Rodrigo (R$ 200) vs Sandra (R$ 200). Pote Total = **R$ 400**.
* **Ação:** All-in no Pré-flop com `A♥ Q♥` (Rodrigo) vs `K♣ J♣` (Sandra). Rodrigo é favorito leve com **62% de Equity** (Tier 0 = **7%**).
* **Resultado:** Sandra acerta um Rei no Flop e vence a mão.
* **Cálculo:** Cashback do Rodrigo = 7% de R$ 400 = **R$ 28,00**.
* **Distribuição Final:**
  * Sandra (Vencedora): R$ 400 - R$ 28 = **R$ 372,00**.
  * Rodrigo (Perdedor All-in): Recebe **R$ 28,00** (7% de cashback).
  * ⚖️ *Soma Total:* 372 + 28 = **R$ 400,00**.



<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-02):** S20f — cash-out automático 45s após disconnect (WS) e no boot da API (assentos órfãos); lobby lista mesas cheias; admin mostra e-mail por assento/inscrito MTT; simulação ritual Play Money + motor MTT até o campeão. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–034 na VPS (cash+MTT Ultimate Pineapple). Disconnect cash-out 45s + reconciliação de assentos órfãos no boot (deploy S20f). Motor `tournament_to_champion` PASS (HE/Freeroll/Omaha/Pineapple até 1 campeão). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas. MTT site: gameplay_ready=false (sem WS de torneio). Health público OK. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
