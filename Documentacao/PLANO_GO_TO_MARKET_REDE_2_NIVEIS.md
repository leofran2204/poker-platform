# Plano de mercado — Rede Zero Tilt em 2 níveis (Play Money)

**Público:** fundador e cada afiliado com painel da própria rede
**Data:** 2026-09-05
**Premissa:** o jogo público **hoje é Play Money**. Nenhum real circula na rede. Este documento é o **molde** para ligar o mesmo grafo em dinheiro real **somente** quando SPA, PSP, KYC e a documentação de compliance estiverem corretos.

> Fonte operacional: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md). Demo: [https://zerotiltpoker.net](https://zerotiltpoker.net). Este plano não duplica o catálogo.

---

## 1. Em uma frase

A sala cresce por **convite de quem já joga**. O cadastro amarra **só o ID do patrocinador** (`sponsored_by`). Clube é feltro (onde a pessoa escolhe jogar), **sem fatia de rake**. A árvore tem **exatamente 2 níveis**. A bonificação — já no desenho de dinheiro real — é **18% do rake individual** de cada um do 1º nível e **12% do rake individual** de cada um do 2º nível. O resto fica com a **casa**. Sempre rake de **mão jogada daquele jogador**, nunca cadastro.

Não é pirâmide. Não há taxa de adesão. Quem não senta não pontua.

---

## 2. Quem é quem

O **primeiro cadastro** da plataforma é a raiz (fundador). Cada pessoa que entra pelo convite dele (`/register?ref=CODIGO`) é **1º nível** dele. Cada pessoa que entra pelo convite de alguém do 1º nível é **2º nível** dele. A partir daí a raiz **não vê** e **não recebe**.

A regra é a mesma para todo mundo: no admin da própria conta, o afiliado vê só **os seus** 1º e 2º níveis. Não vê neto do neto. Não vê a árvore inteira da casa.

**Contas que já existiam** antes do convite fechado entram como **1º nível da raiz** (patrocinador = primeiro usuário). Elas passam a ter código de convite e montam o próprio 2º nível da raiz (e o 1º nível delas).

```
Raiz (1º cadastro)
 ├── 1º nível — entrou pelo convite da raiz  (ou já estava na sala)
 │      └── 2º nível da raiz — entrou pelo convite desse 1º nível
 │             └── (nível 3: existe no banco como filho do 2º, mas a raiz não vê e não ganha)
 └── 1º nível
        └── 2º nível
```

Cada afiliado, olhando o **próprio** painel:

```
Eu
 ├── meu 1º nível  → eu levo 18% do rake individual deles
 └── meu 2º nível  → eu levo 12% do rake individual deles
```

---

## 3. Por que 2 níveis (e o que isso não é)

As redes que quebram reputação pagam por **recrutar**, vendem **kit** e deixam a árvore **sem fundo**. Aqui:

| Mecânica | Agora (Play Money) | Depois da licença |
|----------|--------------------|-------------------|
| Entrada | Convite `?ref=` de quem já tem conta | O mesmo link, com KYC |
| 1º nível | Entrou pelo **meu** código | Igual |
| 2º nível | Entrou pelo código do **meu** 1º nível | Igual |
| Além do 2º | **Invisível** no meu admin; **zero** comissão minha | Igual |
| Taxa / kit | **Proibido** | **Proibido** |
| Bônus por cadastrar | **Proibido** | **Proibido** |
| Base da comissão | Rake da mão no motor (`u64` centavos) | Rake real da mesma árvore |
| Liquidação | Pontos, tickets de freeroll, ranking | Centavos no ledger + saque legal |
| Discurso | Mesa cheia, status, tickets | Rakeback — nunca “renda indicando” |

**Regra de ouro:** quem não **joga** não pontua na linha. Rede parada não gera “renda” virtual.

---

## 4. Plano de compensação

### 4.1 Unidade

**1 ZT Point = 1 centavo de rake Play Money** que o motor tirou do pote. Não mistura com o stack da mesa. Não converte para carteira Real. Não sai em PIX.

O reset diário de fichas continua: cash **R$ 150** e torneio **R$ 150** (fuso `America/Sao_Paulo`). Pontos de rede vivem num **ledger separado**.

### 4.2 Dois volumes (o que cada um vê)

| Sigla | O que conta | Quem vê |
|-------|-------------|---------|
| **VP** (volume pessoal) | Rake das mãos em que **você** sentou (qualquer clube) | Você, para se qualificar |
| **VL1** | Soma do rake **individual** de cada pessoa do seu 1º nível | Você, em **Minha Estrutura** |
| **VL2** | Soma do rake **individual** de cada pessoa do seu 2º nível | Você, em **Minha Estrutura** |

Ninguém vê VL3. O banco pode ter `sponsored_by` encadeado; a API do painel **corta em 2**.

### 4.3 Percentuais (canônicos — iguais em ponto e em real)

Sobre o **rake da mão** daquele jogador (não buy-in, não stack, não “rake do clube”):

| Destino | % do rake gerado por aquele jogador |
|---------|-------------------------------------|
| Patrocinador direto (1º nível) | **18%** |
| Avô da indicação (2º nível), se existir | **12%** |
| Soma máxima de rede nessa mão | **30%** |
| Casa | **resto** (70% com os dois andares; 82% só com 1º nível; 100% sem patrocinador) |

Clube **não recebe**. Não há overlay 15/85 nesta rede. Conta de uma mão com 1.000 centavos de rake e os dois andares:

| Destino | Centavos |
|---------|----------:|
| Patrocinador direto (18%) | 180 |
| Avô (12%) | 120 |
| Casa | 700 |

Sem avô: 180 ao pai, 820 à casa. Sem patrocinador (a raiz jogando): 1.000 à casa. A raiz nas mãos do **próprio** 1º nível leva 18%; nas do 2º nível, 12%. Não leva 18%+12% da mesma mão.

Play Money hoje só muda a **unidade** (ZT Point = 1 centavo de rake PM). Os % não mudam quando (e se) virar real.

### 4.4 Qualificação e Ativação (anti-pirâmide)

Para ter direito e receber bonificação de sua rede de afiliados:

1. **Ter jogado pelo menos 100 mãos na semana**, considerando o ciclo semanal canônico de **segunda-feira (00:00:00 BRT) até domingo (23:59:59 BRT)**.

Quem não atinge o critério **zera a linha naquela semana** (a pontuação é retida pela casa e não pontua). Quem só indica e não joga ativamente não recebe comissão. (Assinatura da Academy como critério: só quando o produto de assinatura existir.)

### 4.5 Pagamento e Apuração Mensal (todo dia 25)

As comissões e bonificações apuradas nas semanas em que o afiliado esteve ativo e qualificado são consolidadas e pagas **todo dia 25 de cada mês**.

No estágio Play Money:
1. **Seats de freeroll da rede**
2. **Tickets de MTT Play Money**
3. **Ranking / badge** visível no lobby

Teto semanal sugerido: o equivalente a **2 seats** de freeroll por afiliado qualificado. O restante vira posição no ranking do mês.

**Nunca:** PIX informal, “vender pontos”, transferência direta entre contas não autorizadas.

### 4.6 Cadastro

- Grátis. Sem kit. Sem taxa de ativação.
- Um patrocinador só (`users.sponsored_by` = ID de quem convidou). Não se troca de pai. Clube é outra coisa: o jogador joga onde quiser; a rede não muda porque ele trocou de mesa ou de clube.
- Com `REQUIRE_INVITE=true`, cadastro exige código (exceto o **primeiro** usuário da base).
- Contas antigas: `sponsored_by` = id do primeiro usuário; cada uma recebe `referral_code`.
- E-mail verificado (Resend + código de 6 dígitos).

---

## 5. Painel Minha Estrutura (o que o admin mostra)

Cada conta autenticada tem **a sua** estrutura, não a da casa:

- Lista do 1º nível: apelido, mãos da semana, rake gerado, % 18
- Lista do 2º nível: apelido, quem é o pai (1º nível), rake gerado, % 12
- Totais VL1 / VL2 / pontos da semana / se está qualificado
- Botão de copiar `https://zerotiltpoker.net/register?ref=MEUCODIGO`

A raiz vê o mesmo recorte: só os **seus** dois andares (incluindo quem já estava na sala, agora 1º nível). Não é um organograma mundial.

**Implementado:** convite `?ref=`, `referral_code` / `sponsored_by`, painel **Minha Estrutura**, ledger 18/12 mão a mão **e** sobre o fee 15% de torneio (`source_type='fee'`). VP 100 mãos por semana (segunda a domingo); sem VP a linha não pontua (retido visível). Pagamento mensal dia 25.

---

## 6. Fases

### Fase A — mesa viva

Objetivo: **≥ 2 pessoas na mesma mesa, no mesmo modo, no horário nobre**. Convite só Play Money. Janela âncora **20h–24h BRT**. Proibido no grupo: PIX, “renda”, fichas combinadas.

### Fase B — painel e pontos

- Backfill: contas existentes → 1º nível da raiz
- Painel 2 níveis por usuário
- Planilha ou ledger de VP / VL1 / VL2
- Freeroll de domingo só para quem bateu VP

KPI: profundidade observada **≤ 2 no painel**; 70%+ dos afiliados que recebem VL também jogaram.

### Fase C — parceiro vê o molde

O relatório de parceiros mostra 18/12, clube sem fatia, o corte em 2 níveis e os gaps. Pedido: **liquidez + compliance**, não cheque para ligar PIX.

### Fase D — interruptor (papelada completa)

SPA ou white-label, PSP com aceite de iGaming, KYC/AML, autoexclusão, saque auditável. Aí `1 ZT Point` → `1 centavo real`. **Os % 18/12, o teto de 2 níveis e “clube sem fatia” não mudam.**

---

## 7. Como convidar (sem parecer MMN de renda)

### 7.1 Mensagem para jogador

```text
Zero Tilt — pôquer online de verdade, fichas virtuais, HTTPS:

https://zerotiltpoker.net/register?ref=SEUCODIGO

1) Cria a conta e confirma o e-mail (código de 6 dígitos; olha o spam)
2) No topo: Play Money
3) Lobby → Hold’em 0,25/0,25 → me avisa que sentou
4) Precisa de 2 pessoas na mesma mesa pra começar

Não é dinheiro real. É pra jogar, aprender e encher a sala.
Hoje 20h, mesa combinada.
```

### 7.2 Mensagem para quem vai indicar

```text
Você indica com o seu código. Quem entra por você é o seu 1º nível.
Quem entra pelo código deles é o seu 2º nível. Acabou.

Agora: pontos e tickets, se você também jogar.
Não é dinheiro. Sem taxa. Sem nível 3 na sua tela.
```

### 7.3 Frases proibidas

- “Renda extra”, “primeiro a entrar ganha mais”, “taxa pra ativar”
- “Joga valendo no PIX do grupo”
- “Indica 10 e fica rico”
- Analogia pública com Amway/Herbalife **para o jogador final**

---

## 8. Papéis

| Papel | Faz | Não faz |
|-------|-----|---------|
| **Raiz (1º cadastro)** | Sobe a demo, regras, freeroll, corta discurso ruim; vê só os **seus** 2 níveis | Prometer BRL; ver a árvore infinita |
| **Afiliado** | Joga, copia o link, acompanha o **próprio** 1º e 2º nível no admin | Cobrar cadastro; inventar nível 3 |
| **Jogador na ponta** | Senta, manda bug | Depositar real “por fora” |

Rotina semanal da casa: segunda presença/mãos; quarta quem não bateu VP; sexta 18h ranking e seats; domingo freeroll no mesmo feltro.

---

## 9. KPIs

| KPI | Meta 30d | Meta 90d | Por que importa |
|-----|----------|----------|-----------------|
| Jogadores que sentaram ≥1 mão | 20 | 80 | Ativação |
| Mãos/semana (PM) | 200 | 1.500 | Liquidez |
| Pico `online` 20h–24h | 4 | 12 | Sala viva |
| Afiliados com VP na semana | — | ≥70% | Anti-pirâmide |
| Profundidade no **painel** | 1 | 2 | Nunca 3 visível |
| Tickets de freeroll jogados | — | ≥80% dos emitidos | Ponto vira gente na mesa |

Se VL cresce e mãos não crescem: **cortar**.

---

## 10. O que já existe vs o que ainda é modelo

Já no produto ou no disco desta entrega: demo HTTPS, e-mail, MFA, Play Money ≠ Real, convite `?ref=` / `REQUIRE_INVITE`, `users.referral_code` e `sponsored_by` (migration `045`). O motor ainda calcula split 15/85 em mesa com `club_id` — **esta rede não usa isso como pagamento de clube**.

Ainda modelo: ledger 18/12 por mão, resto para a casa, painel **Minha Estrutura**, backfill das contas antigas como 1º nível da raiz.

---

## 11. Interruptor para dinheiro real

Não se “liga o MMN em real”. Substitui-se a **unidade** (ponto → centavo) com a **mesma** árvore, os **mesmos** 18/12, clube sem fatia, o **mesmo** teto de 2 níveis. Saque só com PSP e licença. Play Money não saca.

---

## 12. Não faça

- Operar pôquer real “enquanto o registro não vem”
- Terceiro nível “só no WhatsApp” ou no admin da raiz
- Taxa de adesão, kit, prêmio em dinheiro por cadastro
- Misturar pontos de rede com carteira Real
- Desligar Loss Deflator, antifraude ou verificação de e-mail para crescer mais rápido
- Prometer que 18/12 Play Money **já é** receita em BRL
- Pagar clube **e** afiliado na mesma mão; clube nesta rede não leva rake
- Somar 18/12 em cima do split 15/85 do motor

---

## 13. Checklist do dono

- [ ] Convites falam só **Play Money**
- [ ] Horário âncora combinado
- [ ] Backfill: contas antigas = 1º nível da raiz
- [ ] Cada afiliado vê só 2 níveis no admin
- [ ] 18% VL1 e 12% VL2 sobre rake individual; resto à casa; clube sem fatia
- [ ] Qualificação 50 mãos ou R$ 20 rake PM / semana
- [ ] Liquidação = tickets + ranking, nunca PIX
- [ ] Interruptor SPA + PSP + KYC **desligado** até a Fase D

---

*Este plano descreve a árvore comercial: 18/12 sobre rake individual, clube só como lugar, casa com o resto. Os % são os de dinheiro real; Play Money só troca a unidade.*

<!-- DOCUMENTATION_SYNC:START -->
> **S24** (2026-09-10) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático desligado.
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
