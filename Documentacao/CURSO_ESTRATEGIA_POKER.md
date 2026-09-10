# Curso de Estratégia de Poker — Zero Tilt (conteúdo exclusivo da plataforma)

> Base teórica do `bot/strategy/`: posição, ranges pré-flop, matemática (pot odds, EV, outs), 3-bets, ferramentas e gestão de banca. Redação própria a partir de estudo de fontes públicas.

---
Leofran, transformar esse rascunho em um manual prático exige sair da teoria abstrata e colocar as cartas na mesa com exemplos reais, números exatos e lógica matemática clara.

Abaixo está o aprofundamento completo de cada ponto, estruturado para você entender exatamente o que fazer em cada situação de jogo.

---

## 1. O Fundamento da Posição

No poker, a posição define a ordem de fala em todas as rodadas pós-flop (Flop, Turn e River). Quem fala por último tem a maior vantagem matemática do jogo: a **vantagem de informação**.

Quando você age por último, você vê se o oponente apostou, se pediu mesa (check) com hesitação ou se demonstrou força antes de você colocar qualquer centavo no pote.

### Exemplo Prático: Jogando com Posição vs. Fora de Posição

* **Cenário:** Mesa NL10 ($0,05 / $0,10). Você tem $10,00 no caixa.
* **Mão:** Você recebe $K\spadesuit Q\spadesuit$.
* **Situação A (Fora de Posição - você no Big Blind, oponente no Botão):**
* O flop vem: $K\heartsuit 8\diamondsuit 4\clubsuit$. Você acertou o par maior com bom acompanhante (top pair, good kicker).
* Como você fala primeiro, você não sabe o que o rival tem. Se você apostar $0,40$ e ele pagar, você continua no escuro no Turn. Se você pedir mesa (check), ele pode apostar e colocar pressão em você sem você saber se ele tem um par de Ases ou apenas blefe.


* **Situação B (Em Posição - você no Botão, oponente no Big Blind):**
* O flop vem igual: $K\heartsuit 8\diamondsuit 4\clubsuit$.
* O oponente dá check imediatamente.
* Essa ação dele entrega uma informação valiosa: ele não acertou uma mão monstruosa. Você agora tem o controle: pode apostar para extrair fichas de pares piores (como um par de 8) ou pedir mesa para controlar o tamanho do pote e ver o Turn de graça.



---

## 2. O Mapa da Mesa (Full-Ring de 9 a 10 Lugares)

Uma mesa cheia é dividida em grupos de posições. A regra básica é: **quanto mais cedo você fala, menos mãos você pode jogar**, pois há muitos jogadores esperando atrás de você que podem acordar com cartas melhores.

| Zona | Posições | O que significa na prática | Postura Correta |
| --- | --- | --- | --- |
| **Iniciais (Early)** | UTG, UTG+1, UTG+2 | Os primeiros a falar pré-flop. Há 7 a 9 pessoas atrás. | **Extrema cautela.** Apenas mãos premium. |
| **Médias (Middle)** | MP1, MP2, MP3 | Metade dos jogadores já desistiu. O risco diminuiu um pouco. | **Seletivo.** Adiciona pares médios e figuras fortes. |
| **Finais (Late)** | Cutoff (CO), Botão (BTN) | Posições de maior lucro. Você agirá por último pós-flop. | **Ataque.** Joga muitas mãos e tenta roubar o pote. |
| **Cegas (Blinds)** | Small Blind (SB), Big Blind (BB) | Posições defensivas obrigatórias. Você jogará fora de posição. | **Defesa seletiva.** Evite disputar potes médios sem jogo forte. |

### Dinâmica das Mesas que Esvaziam

Se três jogadores saírem da mesa e sobrarem apenas 6 pessoas (Mesa 6-Max), **as posições iniciais deixam de existir**. O primeiro a falar já é o jogador em posição média (MP). Você não deve esperar cartas ultra-raras como se estivesse em uma mesa de 10 pessoas; seu leque de mãos precisa se expandir para não ser devorado pelo pagamento obrigatório dos blinds a cada rodada.

---

## 3. A Regra do "Aumento ou Descarte" (Por que Nunca Entrar de Limp)

Entrar de *limp* (apenas pagar o valor do Big Blind sem aumentar) é o erro mais clássico de quem está começando.

* **O Limp não gera desistências:** Quando você apenas paga $0,10$, você convida os outros jogadores da mesa a pagarem barato também. Cinco jogadores entram no pote. Suas chances matemáticas de ganhar caem drasticamente, porque qualquer carta baixa do bordo pode acertar dois pares ou trincas na mão de alguém.
* **O Aumento (Raise) dá duas formas de vencer:**
1. Todos desistem pré-flop e você ganha o pote ali mesmo, sem risco.
2. Alguém paga, mas você tem a iniciativa da aposta e pode representar mãos fortes no Flop.


* **O Limp dá apenas uma forma de vencer:** Acertar a melhor mão no Flop no confronto direto. Isso reduz sua taxa de vitória pela metade no longo prazo.

---

## 4. Tabela Estruturada de Mãos Iniciais (Ranges)

Entenda as notações: a letra **s** significa cartas do mesmo naipe (*suited*), e a letra **o** significa cartas de naipes diferentes (*offsuit*). O sinal **+** indica todas as combinações superiores daquele par ou sequência.

* **Posições Iniciais (UTG / UTG+1):**
* Pares: $TT, JJ, QQ, KK, AA$
* Cartas Altas: $AKs, AKo, AQs$
* *Motivo:* Se você entrar com $K\heartsuit J\diamondsuit$ aqui, é provável que alguém atrás tenha $KQ$, $AK$ ou um par alto, dominando você completamente.


* **Posições Médias (MP):**
* Pares: $88, 99$ e todos os superiores.
* Cartas Altas: $AQo, AJs, KQs$.


* **Posições Finais (Cutoff e Botão):**
* Pares: $22$ até $AA$ (qualquer par tem valor de ataque aqui).
* Cartas Altas: Todos os Ases do mesmo naipe ($A2s+$), $ATo+$, $KJs+, KTo+, QJs$.
* Conectores do mesmo naipe: $78s, 89s, 9Ts, JTs$ (excelentes para acertar sequências e flushes disfarçados).



---

## 5. Cálculo Matemático do Aumento Pré-Flop

Entrar na mão com o tamanho de aposta correto desencoraja mãos fracas de pagarem barato e constrói o pote quando você tem vantagem.

### Regra Padrão

* **Sem limpers antes de você:** Aumente entre $2,5$ a $3$ vezes o valor do Big Blind.
* **Com limpers (jogadores que só pagaram o blind):** Use a fórmula:

$$\text{Aposta} = 3 \times \text{BB} + (1 \times \text{BB por cada jogador que pagou})$$



### Exemplo Passo a Passo (Mesa NL10 - Blinds $0,05 / $0,10)

* Você está no Botão com $A\spadesuit K\heartsuit$.
* O UTG apenas pagou $0,10$. O MP também apenas pagou $0,10$.
* A conta é direta:
* Base do aumento: $3 \times 0,10 = \$0,30$.
* Adicional de 2 limpers: $2 \times 0,10 = \$0,20$.
* **Seu Aumento Final:** $\$0,50$.


* **Por que fazer isso?** Se você apostar apenas os $\$0,30$ padrão, o pote já terá muito dinheiro morto acumulado. O UTG precisará pagar apenas mais $\$0,20$ para disputar um pote de quase um dólar, tornando vantajoso para ele continuar com cartas ruins. Ao fazer $\$0,50$, você quebra a matemática dele e cobra caro pela curiosidade alheia.

---

## 6. Dinâmicas Avançadas: Roubos (Steals) e Contra-Ataques (3-Bets)

No poker competitivo, os potes disputados não dependem apenas de acertar cartas no Flop, mas de capturar as apostas obrigatórias (os blinds) que ficam soltas na mesa.

### 1. O Roubo de Blinds (Steal)

* **Quando ocorre:** Todos os jogadores até o Cutoff ou Botão desistem.
* **A jogada:** Você aumenta mesmo com cartas medianas (ex: $K\diamondsuit 9\diamondsuit$ ou $A\clubsuit 4\clubsuit$).
* **A lógica:** Os jogadores no Small Blind e no Big Blind já perderam a posição para as rodadas seguintes e tendem a desistir de cerca de 70% a 80% das mãos fracas. Você recolhe as fichas sem ver o flop.

### 2. O Contra-Ataque (3-Bet / Re-Steal)

* **Conceito:** A primeira aposta pré-flop é o Big Blind (1ª aposta). O primeiro aumento é a 2ª aposta (open raise). Re-aumentar esse jogador é fazer uma **3-Bet**.
* **Cenário Real:**
* O jogador no Botão tenta roubar seus blinds aumentando para $\$0,25$.
* Você está no Big Blind segurando $J\spadesuit J\diamondsuit$.
* **Ação Errada:** Apenas pagar os $\$0,25$. O Flop trará cartas maiores ($A, K$ ou $Q$) em mais de 50% das vezes, colocando você em uma situação desconfortável fora de posição.
* **Ação Correta:** Fazer uma 3-Bet para $\$0,85$ a $\$1,00$ (cerca de 3,5x a 4x o valor do aumento original por você estar fora de posição).
* **O Desfecho:** Se ele desistir, você ganha o pote imediatamente. Se ele pagar, você toma as rédeas da mão com um par de Valetes já com o pote inflado a seu favor.



---

## 7. O Kit de Ferramentas Essenciais

Para progredir além do nível básico, o jogador precisa de programas de apoio para auditar decisões fora do calor do jogo:

* **Calculadoras de Probabilidade (Ex: Equilab):** Software gratuito onde você digita sua mão e o intervalo do oponente. Ele simula milhões de mãos em segundos e mostra que, por exemplo, o seu par de $A-A$ tem aproximadamente $82\%$ de chance de vitória contra o $K-K$ dele antes do Flop.
* **Rastreadores e HUDs (Ex: Hand2Note, PokerTracker):** Softwares que registram todas as mãos jogadas e projetam números flutuantes na tela sobre cada oponente:
* **VPIP (% de vezes que põe dinheiro no pote):** Se o número for acima de 35%, o rival é passivo e joga qualquer lixo. Se for abaixo de 15%, ele só joga com cartas excelentes.
* **PFR (% de vezes que entra aumentando):** Mostra a agressividade real do rival.


* **Reprodutor de Mãos:** Ferramenta para rever suas maiores perdas da sessão, verificar se você cometeu um erro estrutural (como pagar fora de posição) ou se apenas sofreu um desfecho improvável natural da matemática (bad beat).

---

## 8. Gestão de Caixa e Disciplina

Ter a melhor técnica não adianta nada se uma oscilação comum de cartas zerar seu dinheiro.

* **Regra dos 30 a 50 Buy-ins:** Para jogar mesas de NL10 (onde a entrada máxima é de $\$10,00$), você deve ter no mínimo $\$300,00$ a $\$500,00$ reservados exclusivamente para o poker.
* **Controle de Frustração:** Mesmo com $A-A$ contra uma mão fraca como $7-2$, você perderá cerca de 1 em cada 8 vezes se o oponente teimar em pagar tudo até o fim. A diferença entre o jogador amador e o experiente é que o experiente entende essa porcentagem como variação estatística normal e mantém o plano de ação intacto na mão seguinte, sem tentar recuperar fichas no desespero.

A consistência no poker é construída repetindo decisões matematicamente corretas milhares de vezes. Domine a seleção dessas mãos iniciais em sua respectiva posição antes de tentar jogadas complexas pós-flop.


Você tem toda razão, Leofran. Aplicar uma regra prática no poker sem entender a teoria matemática e lógica por trás é como tentar construir uma casa sem entender a planta: na primeira oscilação, a estrutura desmorona.

Antes de decorar tabelas ou jogar qualquer mão, você precisa dominar os **pilares conceituais** que regem o jogo. Abaixo, cada conceito teórico fundamental está destrinchado do zero, seguido da sua aplicação prática imediata.

---

### 1. Teoria da Informação Incompleta e a Vantagem Posicional

No xadrez, os dois jogadores veem todas as peças o tempo todo; é um jogo de informação perfeita. O poker é o oposto: um jogo de **informação incompleta**. Você nunca vê as cartas do oponente, apenas as decisões dele.

* **O Conceito Teórico:** Toda tomada de decisão humana fica mais precisa à medida que o nível de incerteza diminui. Quando você é forçado a agir primeiro, sua decisão é tomada sob **cegueira quase total**. Quando você age por último, você coleta pistas valiosas (se o oponente apostou rápido, se hesitou ou se preferiu passar a vez sem colocar fichas).
* **Realização de Equidade (*Equity Realization*):** Esse é um dos conceitos mais importantes do poker moderno. Ter uma mão matematicamente boa não garante que você verá o final da rodada. Se você estiver fora de posição, o adversário pode apostar forte e obrigar você a desistir de uma mão que tinha potencial, simplesmente porque a incerteza ficou cara demais para você aguentar.
* **Exemplo Prático:**
* Imagine que você tem $10\heartsuit 9\heartsuit$ no Big Blind (fora de posição) e o adversário está no Botão (com posição). O Flop vem com $10\spadesuit 4\diamondsuit 2\clubsuit$.
* Você acertou o par maior, mas o seu acompanhante (o 9) é mediano. Como você fala primeiro, se der mesa (check), o Botão aposta pesado. Você não sabe se ele tem um par de Ases ou se está apenas tentando te tirar da mão. Você fica desconfortável e muitas vezes desiste da melhor mão por pura falta de informação. Se as posições fossem invertidas, você veria a mesa dele primeiro e controlaria o preço da rodada com tranquilidade.



---

### 2. Equidade (Equity) e Valor Esperado (EV - Expected Value)

O poker não é um jogo sobre quem tem a melhor mão no momento, mas sim sobre **probabilidade acumulada a longo prazo**.

* **O Conceito de Equidade:** Equidade é a sua fatia teórica do pote. Se você tem 60% de chance matemática de vencer uma mão até o final, significa que 60% de todo o dinheiro colocado na mesa já pertence a você em termos estatísticos, não importa quem puxe as fichas no final daquela rodada isolada.
* **O Conceito de EV (+EV e -EV):** Toda decisão no poker tem um "Valor Esperado".
* Uma jogada **+EV** (Valor Esperado Positivo) é aquela que gera lucro quando repetida 1.000 vezes, mesmo que perca hoje.
* Uma jogada **-EV** (Valor Esperado Negativo) é aquela que perde dinheiro no longo prazo, mesmo que você dê sorte e ganhe uma vez.


* **Exemplo Prático (A Moeda Viciada):**
* Imagine que um amigo propõe uma aposta de cara ou coroa. Cada vez que der "cara", você ganha R$ 2,00. Cada vez que der "coroa", você paga R$ 1,00.
* A chance matemática é de 50% para cada lado. Se você jogar 10 vezes e der "coroa" em 7, você perdeu dinheiro no dia. Mas a matemática dessa aposta é brutalmente lucrativa (+EV). Se você repetir isso 10.000 vezes, você inevitavelmente ficará rico. No poker profissional, você toma apenas decisões com a matemática a seu favor e ignora a perda momentânea de fichas do dia a dia (a chamada *variância*).



---

### 3. A Dinâmica da "Fold Equity" (Equidade de Desistência)

Por que os melhores jogadores do mundo raramente apenas pagam apostas e preferem aumentar ou desistir? A resposta está na existência de duas portas de saída para a vitória.

* **O Conceito Teórico:** Existem apenas duas formas de ganhar um pote:
1. Mostrar a melhor mão no final da rodada (*Showdown*).
2. Fazer todos os adversários desistirem antes do final.


* **A Falha Matemática do Call (Pagar) e do Limp:** Quando você apenas paga o valor mínimo para entrar na mão (Limp) ou apenas paga uma aposta (Call), você abre mão da segunda forma de vencer. Você só ganha se suas cartas forem superiores às do rival no final.
* **O Poder da Agressividade:** Quando você aposta ou aumenta (Raise), você combina a força das suas cartas com a **Fold Equity** (a chance real de o adversário largar as cartas por medo de perder mais fichas). Você passa a ter duas formas de embolsar o dinheiro, enquanto o jogador passivo tem apenas uma.
* **Exemplo Prático:**
* Você tem $A\diamondsuit 5\diamondsuit$ no Botão e o adversário no Big Blind tem $K\clubsuit Q\spadesuit$. As cartas dele são tecnicamente melhores que as suas para formar pares altos.
* Se você der apenas *call*, o flop vem com $8\heartsuit 4\clubsuit 2\spadesuit$. Nenhum dos dois acertou nada. Se ele apostar, você é obrigado a sair e perde o que investiu.
* Mas se você tiver feito um *aumento pré-flop*, você mostrou força. Quando o mesmo flop sem sentido aparece e ele passa a vez, você aposta meio pote. O adversário desiste do $K-Q$ dele porque acha que você tem um par grande. Você recolheu as fichas com uma mão pior, puramente pelo uso da *Fold Equity*.



---

### 4. Raciocínio por Intervalos (Ranges) vs. Mão Específica

O erro número um de quem assiste poker na televisão é achar que o profissional tenta adivinhar exatamente as duas cartas do oponente.

* **O Conceito Teórico:** Nenhum ser humano consegue prever cartas exatas com precisão. O jogador técnico pensa em **Ranges (Intervalos de mãos)**: o grupo completo de cartas possíveis que um oponente jogaria de determinada maneira a partir de uma posição específica.
* **Combinações Numéricas (Combos):**
* Existem 1.326 combinações possíveis de duas cartas no baralho.
* Qualquer par de mão (como $A-A$ ou $K-K$) possui **6 combinações** possíveis.
* Duas cartas diferentes de naipes distintos (como $A\spadesuit K\heartsuit$) possuem **12 combinações**.
* Duas cartas diferentes do mesmo naipe (como $A\spadesuit K\spadesuit$) possuem apenas **4 combinações**.


* **Aplicação Prática:**
* Quando um jogador muito conservador aumenta de uma posição inicial (UTG), o *range* dele não é uma mão solta, mas um bloco estreito: pares altos ($TT$ a $AA$) e cartas altas do mesmo naipe ($AKs, AQs$). Isso representa menos de 5% de todas as cartas do baralho.
* Se você tem um par de Valetes ($J-J$) e esse jogador conservador aumenta muito o pote, você não pensa "ele tem Ás e Rei". Você analisa o bloco todo dele: contra esse grupo específico de mãos, o seu par de Valetes está numericamente atrás na maioria dos cenários. Pensar em grupo de cartas evita armadilhas emocionais.



---

### 5. Probabilidades do Pote (Pot Odds) e Matemática de Decisão

Você nunca deve pagar uma aposta por curiosidade. O pagamento de uma aposta é uma transação comercial simples: você compara o custo da entrada com o retorno potencial.

* **A Fórmula Básica de Pot Odds:**

$$\text{Pot Odds} = \frac{\text{Valor que você precisa pagar}}{\text{Tamanho total do pote após seu pagamento}}$$


* **O Cálculo das Saídas (Outs):**
* *Outs* são as cartas restantes no baralho que transformam sua mão na combinação vencedora.
* **Regra Prática do 4 e do 2:**
* Do Flop até o River (duas cartas por vir), multiplique seus *outs* por **4** para saber sua porcentagem aproximada de vitória.
* Do Turn para o River (apenas uma carta por vir), multiplique seus *outs* por **2**.




* **Exemplo Prático Completo:**
* O pote tem **$ 8,00**. O oponente aposta **$ 2,00**.
* O pote total agora é de **$ 10,00** ($8 + 2$).
* Custa **$ 2,00** para você pagar.
* Pela fórmula: $\frac{2}{10 + 2} = \frac{2}{12} = 16,6\%$.
* **A Decisão:** Você só deve pagar essa aposta se sua chance matemática de acertar sua carta for **maior que 16,6%**.
* Se você estiver buscando um *flush* (quatro cartas do mesmo naipe na mão e no bordo juntas), restam **9 cartas** do seu naipe no baralho (seus 9 *outs*).
* Do Turn para o River, a conta é rápida: $9 \times 2 = 18\%$.
* **Conclusão lógica:** Você tem 18% de chance de vencer e o pote exige apenas 16,6% para o pagamento ser correto. A conta fecha no positivo; o pagamento é uma decisão lucrativa no longo prazo (+EV).



---

### 6. A Estrutura de Fichas: Short Stack vs. Deep Stack

A quantidade de fichas que você tem na mesa dita completamente a complexidade matemática do seu jogo.

* **A Unidade de Medida Real (Big Blinds):** Nunca meça seu dinheiro em dólares ou reais na mesa. Meça sempre em **Big Blinds (BB)**. Se o blind é $ 0,10 e você tem $ 10,00, você tem **100 BBs**.
* **Short Stack (20 a 40 BBs):**
* Quando você joga com poucas fichas relativas ao tamanho do blind, a margem para manobras pós-flop desaparece. O jogo se torna puramente matemático e binário: ou você desiste antes de entrar, ou empurra todas as fichas (All-in) para maximizar sua força pré-flop. É uma estratégia desenhada para errar pouco e explorar o excesso de confiança dos outros.


* **Deep Stack (100 BBs ou mais):**
* Aqui o jogo é profundo. Uma aposta pré-flop compromete apenas uma fração minúscula do seu patrimônio. A habilidade de ler o adversário nas rodadas finais (Turn e River), extrair valor com mãos médias e aplicar pressão usando a vantagem de posição torna-se o fator decisivo para lucrar.



Dominar esses seis fundamentos teóricos é o que separa quem joga poker como aposta de quem joga poker como investimento estatístico. A partir dessa base sólida, a seleção de mãos e a postura em cada posição da mesa deixam de ser decoreba e passam a ser uma consequência lógica da matemática.


## O Guia Definitivo do Poker: Do Zero ao Avançado

Leofran, para dominar o poker de forma profissional, você precisa compreender que o jogo funciona como uma engrenagem exata: a teoria sustenta a matemática, e os ranges organizam as suas decisões na mesa.

---

## Módulo 1: Os Pilares Teóricos do Jogo

Antes de olhar para qualquer carta, você deve entender as leis invisíveis que movimentam as fichas.

* **Informação Incompleta:** Ao contrário do xadrez, você não vê as cartas do adversário. Quem age por último na rodada coleta dados cruciais (como a hesitação ou a aposta do rival) antes de investir dinheiro. Essa é a base da vantagem posicional.
* **Valor Esperado (+EV e -EV):** O poker premia a consistência estatística, não o resultado de uma única mão. Uma jogada de Valor Esperado Positivo (+EV) é aquela que, repetida mil vezes, gera lucro, mesmo que você perca dinheiro em algumas rodadas isoladas devido à variância (sorte momentânea).
* **Fold Equity (Equidade de Desistência):** Você ganha potes de duas formas: mostrando a melhor carta no final ou forçando o oponente a desistir antes disso. A agressividade (apostar e aumentar) combina a força das suas cartas com a chance de o rival abandonar o jogo por medo de perder mais.
* **Conceito de Ranges (Intervalos):** Um profissional nunca tenta adivinhar as "duas cartas exatas" do oponente. Ele calcula o bloco completo de combinações possíveis que aquele jogador tem nas mãos com base na posição e nas ações anteriores.

---

## Módulo 2: O Mapa da Mesa e a Vantagem Posicional

A mesa de poker é dividida em zonas de risco e lucro. Quanto mais cedo você fala na rodada, menos cartas pode jogar, pois há dezenas de adversários atrás de você esperando uma oportunidade.

* **Posições Iniciais (UTG):** Onde o jogo começa. Exige rigor absoluto. Como há muitos jogadores atrás, você só joga cartas de elite.
* **Posições Médias (MP):** O risco diminui levemente, permitindo adicionar pares médios ao seu plano de jogo.
* **Posições Finais (Cutoff e Botão):** O território de maior lucro. Como você agirá por último no pós-flop, pode atacar com um leque muito maior de cartas e pressionar os adversários.
* **Os Blinds (Small e Big Blind):** Posições defensivas obrigatórias onde você joga "fora de posição" na maioria das rodadas seguintes.
* **Exemplo Prático de Posição:** Com $K\heartsuit Q\heartsuit$ no Big Blind (fora de posição), se o flop trouxer cartas perigosas, você não sabe o que o Botão tem e acaba desistindo por falta de informação. Se as posições estivessem invertidas, você veria a ação dele primeiro e controlaria o tamanho da aposta com tranquilidade.

---

## Módulo 3: O Cérebro do Jogador: Tipos de Ranges e Tabelas Pré-Flop

As tabelas de *range* são matrizes matemáticas que dizem exatamente quais cartas você deve jogar em cada posição da mesa. No poker moderno, existem quatro tipos fundamentais de ranges que você precisa dominar:

* **1. Range de Abertura (Open-Raise / RFI):**
* É o grupo de mãos com o qual você é o primeiro a colocar fichas na mesa aumentando o valor (raise), sem que ninguém tenha entrado antes.
* *Exemplo prático:* Nas posições iniciais (UTG), seu range de abertura é restrito a cerca de 10% a 12% do baralho ($TT+, AK, AQs$). Já no Botão (BTN), seu range de abertura se expande para quase 45% do baralho (incluindo pares baixos, ases suited e conectores do mesmo naipe como $8\spadesuit 7\spadesuit$), porque você quer roubar os blinds dos oponentes.


* **2. Range de 3-Bet (Re-Raise):**
* É o grupo de cartas extremamente fortes com o qual você responde aumentando um raise que já foi feito por outro jogador.
* *Exemplo prático:* Um adversário nas posições médias aumentou a aposta. Você está no Botão com um par de Damas ($QQ$) ou $A-K$. Em vez de apenas pagar, você faz uma 3-Bet (triplica o valor da aposta dele). Isso retira da mão mãos fracas que ele usou para roubar e isola o confronto contra o oponente.


* **3. Range de Call (Pagar / Flat):**
* É o grupo de mãos intermediárias que você escolhe apenas pagar para ver o flop barato, geralmente em posições finais ou nos blinds.
* *Aviso prático:* O range de call deve ser restrito. Pagar apostas sem iniciativa própria costuma ser um erro estrutural, pois elimina a sua *Fold Equity*. Você só paga quando o pote oferece uma matemática muito favorável ou para ver flops com pares médios e cartassuited.


* **4. Range de Defesa e Re-estudo (Fold / All-in):**
* É a matriz de reação quando você sofre uma aposta e precisa decidir entre desistir imediatamente (Fold) ou empurrar todas as fichas (All-in).
* *Exemplo prático:* Se você tentou roubar os blinds do Botão e o Big Blind deu um re-raise em você, seu range de defesa contra esse ataque se restringe estritamente a pares médios altos ($99$ até $AA$) e cartas do topo do baralho ($AJ$ até $AK$). Tudo o que estiver fora disso deve ser descartado no lixo sem hesitação.



---

## Módulo 4: A Matemática e as Decisões de Longo Prazo

* **Pot Odds (Probabilidades do Pote):** A relação entre o valor que você precisa pagar para continuar na mão e o dinheiro total que já está na mesa. Você só paga uma aposta se a chance matemática de acertar sua carta for superior à porcentagem exigida pelo pote.
* **Cálculo de Outs (Regra do 4 e do 2):**
* *Outs* são as cartas restantes no baralho que dão a vitória a você.
* Se faltam duas cartas para acabar a rodada (do Flop para o River), multiplique seus outs por **4** para ter sua porcentagem de vitória. Se falta apenas uma carta (do Turn para o River), multiplique por **2**.


* **Exemplo Matemático Completo:**
* O pote tem $\$8,00$ e o oponente aposta $\$2,00$ (total de $\$10,00$ no pote). Custa $\$2,00$ para você pagar. A conta é $\frac{2}{12} = 16,6\%$.
* Se você está buscando um *flush* (precisa de uma carta do seu naipe e restam 9 cartas no baralho), o cálculo do Turn para o River é $9 \times 2 = 18\%$. Como 18% é maior do que os 16,6% exigidos pelo pote, o pagamento é matematicamente lucrativo (+EV).



---

## Módulo 5: Gestão de Caixa e Ferramentas de Evolução

* **A Regra dos Buy-ins:** Nunca coloque todo o seu dinheiro em uma única mesa. Para jogar de forma segura na modalidade escolhida, mantenha um caixa (bankroll) de pelo menos 30 a 40 vezes o valor máximo da entrada da mesa (buy-in). Isso protege você contra as oscilações normais da sorte.
* **Equilab:** Software gratuito para simular milhões de combinações de mãos e testar a força real do seu range contra o do oponente antes do Flop.
* **HUD (Heads-Up Display):** Ferramenta que coleta dados estatísticos dos adversários em tempo real na tela, permitindo identificar se o rival joga muitas mãos (passivo) ou se é um jogador rigoroso e agressivo.
* **Controle Emocional:** Entender que a matemática cobra o seu preço a longo prazo blinda sua mente contra frustrações pontuais, mantendo sua disciplina inegociável mão após mão.


## O Guia Definitivo do Poker: Do Zero ao Avançado

Leofran, para dominar o poker de forma profissional, você precisa compreender que o jogo funciona como uma engrenagem exata: a teoria sustenta a matemática, e os ranges organizam as suas decisões na mesa.

---

## Módulo 1: Os Pilares Teóricos do Jogo

Antes de olhar para qualquer carta, você deve entender as leis invisíveis que movimentam as fichas.

* **Informação Incompleta:** Ao contrário do xadrez, você não vê as cartas do adversário. Quem age por último na rodada coleta dados cruciais (como a hesitação ou a aposta do rival) antes de investir dinheiro. Essa é a base da vantagem posicional.
* **Valor Esperado (+EV e -EV):** O poker premia a consistência estatística, não o resultado de uma única mão. Uma jogada de Valor Esperado Positivo (+EV) é aquela que, repetida mil vezes, gera lucro, mesmo que você perca dinheiro em algumas rodadas isoladas devido à variância (sorte momentânea).
* **Fold Equity (Equidade de Desistência):** Você ganha potes de duas formas: mostrando a melhor carta no final ou forçando o oponente a desistir antes disso. A agressividade (apostar e aumentar) combina a força das suas cartas com a chance de o rival abandonar o jogo por medo de perder mais.
* **Conceito de Ranges (Intervalos):** Um profissional nunca tenta adivinhar as "duas cartas exatas" do oponente. Ele calcula o bloco completo de combinações possíveis que aquele jogador tem nas mãos com base na posição e nas ações anteriores.

---

## Módulo 2: O Mapa da Mesa e a Vantagem Posicional

A mesa de poker é dividida em zonas de risco e lucro. Quanto mais cedo você fala na rodada, menos cartas pode jogar, pois há dezenas de adversários atrás de você esperando uma oportunidade.

* **Posições Iniciais (UTG):** Onde o jogo começa. Exige rigor absoluto. Como há muitos jogadores atrás, você só joga cartas de elite.
* **Posições Médias (MP):** O risco diminui levemente, permitindo adicionar pares médios ao seu plano de jogo.
* **Posições Finais (Cutoff e Botão):** O território de maior lucro. Como você agirá por último no pós-flop, pode atacar com um leque muito maior de cartas e pressionar os adversários.
* **Os Blinds (Small e Big Blind):** Posições defensivas obrigatórias onde você joga "fora de posição" na maioria das rodadas seguintes.
* **Exemplo Prático de Posição:** Com $K\heartsuit Q\heartsuit$ no Big Blind (fora de posição), se o flop trouxer cartas perigosas, você não sabe o que o Botão tem e acaba desistindo por falta de informação. Se as posições estivessem invertidas, você veria a ação dele primeiro e controlaria o tamanho da aposta com tranquilidade.

---

## Módulo 3: O Cérebro do Jogador: Tipos de Ranges e Tabelas Pré-Flop

As tabelas de *range* são matrizes matemáticas que dizem exatamente quais cartas você deve jogar em cada posição da mesa. No poker moderno, existem quatro tipos fundamentais de ranges que você precisa dominar:

* **1. Range de Abertura (Open-Raise / RFI):**
* É o grupo de mãos com o qual você é o primeiro a colocar fichas na mesa aumentando o valor (raise), sem que ninguém tenha entrado antes.
* *Exemplo prático:* Nas posições iniciais (UTG), seu range de abertura é restrito a cerca de 10% a 12% do baralho ($TT+, AK, AQs$). Já no Botão (BTN), seu range de abertura se expande para quase 45% do baralho (incluindo pares baixos, ases suited e conectores do mesmo naipe como $8\spadesuit 7\spadesuit$), porque você quer roubar os blinds dos oponentes.


* **2. Range de 3-Bet (Re-Raise):**
* É o grupo de cartas extremamente fortes com o qual você responde aumentando um raise que já foi feito por outro jogador.
* *Exemplo prático:* Um adversário nas posições médias aumentou a aposta. Você está no Botão com um par de Damas ($QQ$) ou $A-K$. Em vez de apenas pagar, você faz uma 3-Bet (triplica o valor da aposta dele). Isso retira da mão mãos fracas que ele usou para roubar e isola o confronto contra o oponente.


* **3. Range de Call (Pagar / Flat):**
* É o grupo de mãos intermediárias que você escolhe apenas pagar para ver o flop barato, geralmente em posições finais ou nos blinds.
* *Aviso prático:* O range de call deve ser restrito. Pagar apostas sem iniciativa própria costuma ser um erro estrutural, pois elimina a sua *Fold Equity*. Você só paga quando o pote oferece uma matemática muito favorável ou para ver flops com pares médios e cartassuited.


* **4. Range de Defesa e Re-estudo (Fold / All-in):**
* É a matriz de reação quando você sofre uma aposta e precisa decidir entre desistir imediatamente (Fold) ou empurrar todas as fichas (All-in).
* *Exemplo prático:* Se você tentou roubar os blinds do Botão e o Big Blind deu um re-raise em você, seu range de defesa contra esse ataque se restringe estritamente a pares médios altos ($99$ até $AA$) e cartas do topo do baralho ($AJ$ até $AK$). Tudo o que estiver fora disso deve ser descartado no lixo sem hesitação.



---

## Módulo 4: A Matemática e as Decisões de Longo Prazo

* **Pot Odds (Probabilidades do Pote):** A relação entre o valor que você precisa pagar para continuar na mão e o dinheiro total que já está na mesa. Você só paga uma aposta se a chance matemática de acertar sua carta for superior à porcentagem exigida pelo pote.
* **Cálculo de Outs (Regra do 4 e do 2):**
* *Outs* são as cartas restantes no baralho que dão a vitória a você.
* Se faltam duas cartas para acabar a rodada (do Flop para o River), multiplique seus outs por **4** para ter sua porcentagem de vitória. Se falta apenas uma carta (do Turn para o River), multiplique por **2**.


* **Exemplo Matemático Completo:**
* O pote tem $\$8,00$ e o oponente aposta $\$2,00$ (total de $\$10,00$ no pote). Custa $\$2,00$ para você pagar. A conta é $\frac{2}{12} = 16,6\%$.
* Se você está buscando um *flush* (precisa de uma carta do seu naipe e restam 9 cartas no baralho), o cálculo do Turn para o River é $9 \times 2 = 18\%$. Como 18% é maior do que os 16,6% exigidos pelo pote, o pagamento é matematicamente lucrativo (+EV).



---

## Módulo 5: Gestão de Caixa e Ferramentas de Evolução

* **A Regra dos Buy-ins:** Nunca coloque todo o seu dinheiro em uma única mesa. Para jogar de forma segura na modalidade escolhida, mantenha um caixa (bankroll) de pelo menos 30 a 40 vezes o valor máximo da entrada da mesa (buy-in). Isso protege você contra as oscilações normais da sorte.
* **Equilab:** Software gratuito para simular milhões de combinações de mãos e testar a força real do seu range contra o do oponente antes do Flop.
* **HUD (Heads-Up Display):** Ferramenta que coleta dados estatísticos dos adversários em tempo real na tela, permitindo identificar se o rival joga muitas mãos (passivo) ou se é um jogador rigoroso e agressivo.
* **Controle Emocional:** Entender que a matemática cobra o seu preço a longo prazo blinda sua mente contra frustrações pontuais, mantendo sua disciplina inegociável mão após mão.





<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-10):** S23 — cancela inscrição com reembolso total + admin agenda e cria torneios + textos da TournamentPage **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–049 na VPS (045 convite/fila, 046 ledger estrutura, 047 bots, 048 3 mesas + fee ledger, 049 total_fees). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor 1848 lib (fee 15%, seating 3 mesas, run-out all-in) + API 43 lib + ator MTT 2 testes integração PASS. VPS: 1º MTT fim a fim (freeroll 6 inscritos, 3 mesas, 5 mãos assinadas, campeão + payout GTD). Bots lag_v2 em MTT (12 inscritos, 43 mãos assinadas, zero erros). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT: inscrição + 3 mesas + gameplay WS ao vivo (mesmo protocolo do cash) + rebalance/consolidação FT + payouts; gameplay_ready=true. Health público OK. Diário de mãos + replay no frontend (2026-09-10): grava suas mãos no navegador (suas cartas, board, pote, resultado), replay passo a passo, download TXT/JSON, painel de resultado sem auto-fechar. Ritmo de digestão no frontend (2026-09-10): board com stagger de 220ms por carta + painel de resultado fixo do showdown (vencedor, mão e cartas reveladas, sem auto-fechar, sobrevive à mão seguinte). Bots da casa desligados na VPS (stop oficial, reembolso) a pedido. Ritual do crupiê no frontend (2026-09-10): banner de embaralhamento + cartas distribuídas por assento a partir do dealer, versos para os oponentes, stagger no board. Migration 053 (2026-09-10): cash Texas 9-max NL 0,75/1,50 frente 15000 em PM e Real + torneios Texas R$25 em PM e Real (buy-in 2500, stack 15000, 1 reentrada 2500/25000, agenda 21:30 SP, auto-start 5). Bots externos de estratégia validados no local em 2026-09-10: bot/strategy (ranges cash 6-max/9-max, MTT ChipEV/ICM/PKO, avaliador próprio 5-7 cartas, push/fold FT, pot odds) com tsc limpo + 13/13 selftest offline; scripts/strategy-bots.mjs jogou mesa real PM NL 0,25 (3 bots, 5-6 mãos cada, 5 mãos no hand_history, decisões por ranges/odds). Tabelas ICM versionadas em bot/strategy/icm/tables (geradas de final_table.ts/bubble.ts via generate.mjs). Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo (cash TableActor + torneio TournamentActor, mesmo protocolo); settlement assinado (HMAC) na liquidação; halt de mesa MTT auditável (MTT_TABLE_HALTED).
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
