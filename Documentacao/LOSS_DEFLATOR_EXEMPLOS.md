# Guia de Exemplos Práticos — Loss Deflator (Bad Beat Cashback)

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

### Tier 0 — 7% (Equity do Perdedor 60%–64,9%)
Cenários de favoritismo marginal (quase coin-flips devido à quantidade massiva de outs do vilão).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `A♥ Q♥` vs `K♣ J♣` | **38%** *(62% Equity)* | Sem board <br> *(KJ acerta par de Reis no flop)* | 500 | **Tier 0 (7%)** | **35** |
| **Flop** | `T♣ T♠` *(set)* vs `8♥ 7♥` | **38%** *(62% Equity)* | Board: `T♥ 9♣ 2♥` <br> *(87♥ acerta flush no river contra set)* | 500 | **Tier 0 (7%)** | **35** |
| **Turn** | `J♣ J♦` *(overpair)* vs `A♥ K♥` | **36%** *(64% Equity)* | Board: `Q♥ T♣ 5♥ 2♠` <br> *(AK♥ acerta um de seus 16 outs no river)* | 500 | **Tier 0 (7%)** | **35** |

---

### Sem Cashback — Equity do Perdedor < 60%
Ações normais onde a mão favorita segurou e venceu, não gerando cashback para a mão perdedora (que já era a pior).

| Fase do All-In | Mão melhor VS Mão pior | Chances da mão melhor perder % | Board / Desfecho | Total da perda | Tier | Deflator de perda |
| :--- | :--- | :---: | :--- | :---: | :---: | :---: |
| **Pré-flop** | `K♦ K♣` vs `A♠ T♠` | **32%** *(68% Equity)* | Sem board <br> *(KK segurou até o fim)* | 500 | **N/A** | **0** |
| **Flop** | `A♦ A♣` vs `7♥ 6♥` | **5%** *(95% Equity)* | Board: `K♠ Q♦ J♣` <br> *(AA segurou até o fim)* | 500 | **N/A** | **0** |
| **Turn** | `T♦ T♣` vs `5♠ 4♠` | **0%** *(100% Equity)* | Board: `A♥ K♦ Q♣ J♥` <br> *(54 drawing dead, TT segurou)* | 500 | **N/A** | **0** |
