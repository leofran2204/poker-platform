# Guia de Exemplos Práticos — Loss Deflator (Bad Beat Cashback)

**Atualizado:** 2026-07-27 | **Status:** Exemplos técnicos; cálculos monetários usam inteiros e pontos-base.

Este documento serve como material de apoio técnico e educacional para o funcionamento do módulo **Loss Deflator** (`loss_deflator.rs`). Aqui você encontrará definições visuais, conceitos probabilísticos e simulações matemáticas reais de mãos de poker para cada um dos Tiers de cashback.


---

## 1. Conceitos Fundamentais

### 📐 O que é Equity?
A **Equity** representa a sua **chance percentual matemática de vencer o pote** em um determinado momento da mão, caso nenhuma outra ação de aposta ocorra e todas as cartas restantes sejam distribuídas. 

*   Ela é calculada simulando todas as combinações possíveis de cartas comunitárias (*board*) restantes e dividindo o número de vitórias pelo total de cenários.
*   A Equity é dinâmica: ela muda drasticamente a cada rodada (Pré-flop ➔ Flop ➔ Turn ➔ River) à medida que novas cartas são reveladas.
*   **O Loss Deflator só é ativado se o jogador com Equity ≥ 60% perder a mão.**

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

### Tier 3 — 35% (Equity do Perdedor ≥ 85%)
O perdedor tinha chance quase nula de perder a mão (Bad Beats extremos).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♠ A♦` vs `7♥ 2♣` | **12%** *(88% Equity)* | Sem board <br> *(72o acerta dois pares milagrosos)* | 500 | **Tier 3 (35%)** | **175** |
| **Flop** | `A♦ A♣` *(set)* vs `K♦ Q♠` | **3%** *(97% Equity)* | Board: `A♠ 7♣ 2♥` <br> *(KQ acerta J+T runner-runner para Broadway)* | 500 | **Tier 3 (35%)** | **175** |
| **Turn** | `Q♣ Q♦` *(set)* vs `J♦ 9♥` | **9%** *(91% Equity)* | Board: `Q♥ 8♠ 3♣ 2♦` <br> *(J9 acerta T no river para sequência)* | 500 | **Tier 3 (35%)** | **175** |

---

### Tier 2 — 25% (Equity do Perdedor 75%–84,9%)
O perdedor era claro favorito, mas o oponente possuía um projeto com alguns outs (ex: gutshot simples).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♥ A♣` vs `K♠ K♦` | **18%** *(82% Equity)* | Sem board <br> *(KK acerta K no flop para trinca)* | 500 | **Tier 2 (25%)** | **125** |
| **Flop** | `A♠ A♣` *(overpair)* vs `6♣ 5♦` | **17%** *(83% Equity)* | Board: `K♦ 7♠ 3♥` <br> *(65 acerta 4 no river para sequência)* | 500 | **Tier 2 (25%)** | **125** |
| **Turn** | `A♦ A♣` *(overpair)* vs `J♥ T♥` | **18%** *(82% Equity)* | Board: `Q♠ 9♣ 4♦ 2♠` <br> *(JT acerta K ou 8 no river para sequência)* | 500 | **Tier 2 (25%)** | **125** |

---

### Tier 1 — 15% (Equity do Perdedor 65%–74,9%)
O perdedor era favorito moderado, mas o oponente tinha bons draws (ex: OESD simples).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `J♠ J♦` vs `A♣ T♥` | **29%** *(71% Equity)* | Sem board <br> *(AT acerta Ás no flop)* | 500 | **Tier 1 (15%)** | **75** |
| **Flop** | `A♠ A♣` *(overpair)* vs `9♣ 8♦` | **31%** *(69% Equity)* | Board: `T♥ 7♠ 2♣` <br> *(98 completa sequência aberta no river)* | 500 | **Tier 1 (15%)** | **75** |
| **Turn** | `A♠ A♣` *(overpair)* vs `8♠ 7♠` | **34%** *(66% Equity)* | Board: `T♠ 9♣ 4♠ 2♦` <br> *(87♠ completa flush ou sequência — 15 outs)* | 500 | **Tier 1 (15%)** | **75** |

---

### Tier 0 — 7% (Equity do Perdedor 46,0%–59,9%)
Cenários de jogada parelha ou coin-flip (onde o perdedor tinha de 46% a 59,9% de chance de vencer e acabou perdendo).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♣ K♣` vs `9♠ 9♦` | **46%** *(54% Equity)* | Sem board <br> *(99 segura no river)* | 500 | **Tier 0 (7%)** | **35** |
| **Flop** | `T♣ T♠` *(set)* vs `8♥ 7♥` | **52%** *(48% Equity)* | Board: `T♥ 9♣ 2♥` <br> *(87♥ acerta flush no river contra set)* | 500 | **Tier 0 (7%)** | **35** |
| **Turn** | `J♣ J♦` *(overpair)* vs `A♥ K♥` | **48%** *(52% Equity)* | Board: `Q♥ T♣ 5♥ 2♠` <br> *(AK♥ acerta um de seus outs no river)* | 500 | **Tier 0 (7%)** | **35** |

---

### Sem Cashback — Equity do Perdedor < 46%
Ações de blefe desesperado ou zebras absolutas onde o perdedor tinha menos de 46% de chance de vencer.

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `K♦ K♣` vs `A♠ T♠` | **32%** *(8% Equity)* | Sem board <br> *(KK segurou até o fim)* | 500 | **N/A** | **0** |
| **Flop** | `A♦ A♣` vs `7♥ 6♥` | **5%** *(95% Equity)* | Board: `K♠ Q♦ J♣` <br> *(AA segurou até o fim)* | 500 | **N/A** | **0** |
| **Turn** | `T♦ T♣` vs `5♠ 4♠` | **0%** *(100% Equity)* | Board: `A♥ K♦ Q♣ J♥` <br> *(54 drawing dead, TT segurou)* | 500 | **N/A** | **0** |

---

## 5. 💡 Origem das Fichas e Exemplos Práticos de Múltiplos All-Ins

> ⚠️ **Princípio Fundamental:** O dinheiro do cashback **nunca é retirado da plataforma/casa**. Ele vem 100% das próprias fichas acumuladas no pote da mesa. O vencedor daquele pote financia o cashback do perdedor que foi All-in. Se um jogador nem participou de um pote secundário (*side pot*), o dinheiro dele fica **100% intocado**.

### 🎲 6 Casos Práticos Reais (Cash Games & Torneios)

#### Exemplo 1: Heads-up Simples (1 contra 1) — All-in no Flop
* **Situação:** Ana (R$ 100) vs Beto (R$ 100). Pote Total = **R$ 200**.
* **Ação:** Ana vai All-in no **Flop** (25% cashback) com `A♠ A♥`. Beto paga com `9♣ 8♣`.
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
* **Ação:** Todos All-in no **Pré-flop** (15% cashback). **Eduardo vence a mão inteira.**
* **Cálculo dos Cashbacks:**
  * Carlos: Perdeu Main Pot (R$ 150) no Pré-flop (15%) = **R$ 22,50**.
  * Diego: Perdeu Main Pot + Side Pot (R$ 250) no Pré-flop (15%) = **R$ 37,50**.
* **Distribuição Final:**
  * Eduardo (Vencedor de tudo): R$ 250 - 22,50 - 37,50 = **R$ 190,00**.
  * Carlos: Recebe **R$ 22,50**.
  * Diego: Recebe **R$ 37,50**.
  * ⚖️ *Soma Total:* 190 + 22,50 + 37,50 = **R$ 250,00**.

#### Exemplo 3: Proteção de Side Pot (Respeito a quem não disputou)
* **Situação:** Fernando (R$ 20), Gabriela (R$ 100), Hélio (R$ 100).
* **Potes Formados:** Main Pot = R$ 60 | Side Pot = R$ 160 (Gabriela e Hélio).
* **Ação:** Fernando All-in no **Pré-flop** (15%). Gabriela e Hélio All-in no **Turn** (35%).
* **Showdown:** **Gabriela** ganha o Main Pot (R$ 60). **Hélio** ganha o Side Pot (R$ 160). Fernando perdeu.
* **Cálculo:**
  * Fernando perdeu o Main Pot de R$ 60 (15%) = **R$ 9,00**.
  * Esse R$ 9,00 sai APENAS da Gabriela (ganhadora do Main Pot).
  * O pote de Hélio (Side Pot de R$ 160) fica **100% intocado**, pois Fernando não participou do Side Pot!
* **Distribuição Final:**
  * Gabriela: R$ 60 - R$ 9 = **R$ 51,00**.
  * Hélio: **R$ 160,00** (Intocado!).
  * Fernando: **R$ 9,00**.
  * ⚖️ *Soma Total:* 51 + 160 + 9 = **R$ 220,00**.

#### Exemplo 4: All-ins em Fases Diferentes (Pré-Flop vs Turn)
* **Situação:** Igor (R$ 40), João (R$ 100), Lucas (R$ 100).
* **Pré-flop:** Igor All-in R$ 40. Main Pot = R$ 120 (Igor = **Pré-flop 15%**).
* **Turn:** João All-in mais R$ 60. Side Pot = R$ 120 (João = **Turn 35%**).
* **Showdown:** **Lucas** vence o Main Pot (R$ 120) e o Side Pot (R$ 120).
* **Cálculos:**
  * Igor (Pré-flop): 15% de R$ 120 = **R$ 18,00**.
  * João (Turn): 35% de R$ 240 (sua participação total) = **R$ 84,00**.
* **Distribuição Final:**
  * Lucas (Vencedor): R$ 240 - R$ 18 - R$ 84 = **R$ 138,00**.
  * Igor: **R$ 18,00**.
  * João: **R$ 84,00**.
  * ⚖️ *Soma Total:* 138 + 18 + 84 = **R$ 240,00**.

#### Exemplo 5: Pote Dividido (Split Pot) entre 2 Vencedores
* **Situação:** Marcelo (R$ 100), Natália (R$ 100), Otávio (R$ 100). Pote Total = R$ 300.
* **Ação:** Marcelo All-in no **Flop** (25%). Natália e Otávio empatam na melhor mão (Split Pot).
* **Cálculo:** Marcelo perdeu R$ 300 (25%) = **R$ 75,00**.
* **Distribuição:** Os R$ 75,00 são descontados meio a meio dos dois vencedores (R$ 37,50 cada).
  * Natália: R$ 150 - R$ 37,50 = **R$ 112,50**.
  * Otávio: R$ 150 - R$ 37,50 = **R$ 112,50**.
  * Marcelo: **R$ 75,00**.
  * ⚖️ *Soma Total:* 112,50 + 112,50 + 75 = **R$ 300,00**.

#### Exemplo 6: All-in no River (Zero Cashback)
* **Situação:** Pedro (R$ 100) vs Quênia (R$ 100). All-in no **River** (faltavam 0 cartas).
* **Resultado:** Quênia vence.
* **Cálculo:** All-in no River = **0% Cashback**.
* **Distribuição Final:** Quênia recebe **R$ 200,00 integralmente**. Pedro recebe R$ 0,00.

#### Exemplo 7: Tier 0 — Faixa Mínima de 7% (Equity de 60,0% a 64,9%)
* **Situação:** Rodrigo (R$ 200) vs Sandra (R$ 200). Pote Total = **R$ 400**.
* **Ação:** All-in no Pré-flop com `A♥ Q♥` (Rodrigo) vs `K♣ J♣` (Sandra). Rodrigo é favorito leve com **62% de Equity** (Tier 0 = **7%**).
* **Resultado:** Sandra acerta um Rei no Flop e vence a mão.
* **Cálculo:** Cashback do Rodrigo = 7% de R$ 400 = **R$ 28,00**.
* **Distribuição Final:**
  * Sandra (Vencedora): R$ 400 - R$ 28 = **R$ 372,00**.
  * Rodrigo (Perdedor All-in): Recebe **R$ 28,00** (7% de cashback).
  * ⚖️ *Soma Total:* 372 + 28 = **R$ 400,00**.



<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-07-29):** S09 — liquidação de mãos protegida por guarda transacional e PIX Asaas apenas em Sandbox autenticado. **Sem certificação de produção; o código rejeita PIX em modo production.** cargo fmt, cargo check --all-targets e cargo clippy --all-targets -- -D warnings passaram no WSL; cargo test --lib passou com 17 testes e a migration foi aceita em transação PostgreSQL revertida. A carga completa autorizada continua manual e não foi acionada neste ciclo. Mock é o padrão. O único adaptador externo é o Asaas Sandbox, restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção permanecem desabilitados. Nenhum depósito com dinheiro real foi habilitado. Mesas continuam com dono único por processo; uma guarda persistente pausa a mesa após falha entre início e liquidação da mão, exigindo revisão/abort administrativo antes da reabertura.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
