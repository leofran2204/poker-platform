# Plano de mercado — Rede Zero Tilt em 2 níveis (Play Money)

**Público:** fundador, clubes âncora e agentes  
**Data:** 2026-09-01  
**Premissa:** o jogo público **hoje é Play Money**. Nenhum real circula na rede. Este documento é o **molde** para ligar o mesmo grafo em dinheiro real **somente** quando SPA, PSP, KYC e a documentação de compliance estiverem corretos.

> Fonte operacional: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json) (ciclo **S20c**). Demo: [https://zerotiltpoker.net](https://zerotiltpoker.net).

---

## 1. Em uma frase

Encher a sala **agora** com um marketing de rede de **exatamente 2 níveis**, pago só em **pontos Zero Tilt, tickets de freeroll e ranking**, medido por **rake de mãos jogadas** — e guardar essa árvore para o dia em que a unidade de liquidação passar de ponto para centavo real.

Não é “pôquer informal”. Não é pirâmide. É o modelo de **clube + agente** que o código já tem (`clubs` / `club_agents`, split 15/85), ensaiado em fichas virtuais.

---

## 2. Por que 2 níveis (e por que isso convence)

As redes clássicas (Amway, Hinode) crescem porque cada pessoa tem **um patrocinador**, um **volume pessoal** e um **volume de grupo**. O que destrói reputação — e o que a lei trata como pirâmide — é pagar por **recrutar**, vender **kit de adesão** e deixar a árvore **sem fundo**.

O Zero Tilt copia só o que é saudável:

| Mecânica de rede | No Play Money (agora) | No real (depois da licença) |
|------------------|----------------------|-----------------------------|
| Patrocinador indica alguém | Link do clube ou do agente | O mesmo link, conta com KYC |
| Volume pessoal (VP) | Rake PM + mãos do próprio jogador | Rake real do próprio jogador |
| Volume de linha (VL) | Rake PM dos indicados (1 linha) | Rake real da mesma árvore |
| Profundidade | **Teto = 2** (Clube → Agente → jogadores) | Igual — contrato e código iguais |
| Taxa de entrada / kit | **Proibido** | **Proibido** |
| Bônus por cadastrar gente | **Proibido** | **Proibido** |
| Comissão | Sobre **rake da mão jogada** | Sobre rake real |
| Pagamento | Pontos, seats de MTT, badge | BRL no ledger do clube/agente |
| Discurso | “Mesa cheia, status, tickets” | Rakeback — nunca “ficar rico indicando” |

**Regra de ouro:** quem não **joga** não pontua na linha. Rede parada não gera “renda” virtual. Isso é o equivalente ao volume pessoal das empresas sérias.

---

## 3. O grafo (igual hoje e no futuro)

```
Zero Tilt (casa) — 15% do rake contabilizado
   └── Nível 1 — Clube / líder de rede — até 85% do rake da sua liquidez
          └── Nível 2 — Agente do clube — 0 a 50% da fatia do clube, só da própria linha
                 └── Jogadores indicados pelo agente
                        └── Se um jogador passar a indicar, ele vira Agente
                            do MESMO clube (continua nível 2). Não existe nível 3.
```

Isso já cabe no banco:

- `clubs` — saldo, rake 85%, fee 15%
- `club_agents` — `rakeback_percentage` 0–50, `total_players_referred`, `total_commission_earned`
- `club_memberships` — quem joga em qual clube
- Mesas e torneios com `club_id` e `money_mode` (`play` | `real`)

**Cap de profundidade no discurso e no contrato:** agente não patrocina agente. Jogador que indica bem **sobe para agente do mesmo clube**, não abre um terceiro andar.

---

## 4. Plano de compensação Play Money

### 4.1 Unidade

**1 ZT Point = 1 centavo de rake Play Money** gerado no motor (o mesmo `u64` que já usa a casa). Não mistura com o stack da mesa. Não converte para carteira Real. Não sai em PIX.

O reset diário de fichas continua: cash **R$ 150** e torneio **zerado** (freerolls grátis, sem recarga MTT; fuso `America/Sao_Paulo`). Pontos de rede vivem num **ledger separado** — é o ensaio do `total_commission_earned` de verdade.

### 4.2 Dois volumes

| Sigla | O que conta | Quem vê |
|-------|-------------|---------|
| **VP** (volume pessoal) | Rake PM das mãos em que **você** sentou | Todo mundo que quer se qualificar |
| **VL** (volume de linha) | Rake PM dos jogadores que **você** patrocinou | Clube vê agentes; agente vê jogadores |

### 4.3 Split (espelha o 15/85 que o motor já calcula)

Exemplo: uma mão gera **1.000 centavos** de rake PM (R$ 10 virtuais).

| Destino | Conta | ZT Points |
|---------|------:|----------:|
| Casa | 15% | 150 (só dashboard; não “paga” pessoa) |
| Clube | 85% | 850 |
| Se o agente dessa linha tem 30% da fatia do clube | 30% × 850 | 255 para o agente, 595 ficam no clube |

O percentual do agente **nunca** come a fatia da casa. Sai só dos 85% do clube. É a regra já escrita em `BUSINESS_RULES.md` §9.3.

### 4.4 Qualificação (anti-pirâmide)

Para receber pontos de **linha** na semana:

1. Ter jogado **pelo menos 50 mãos** naquela semana (VP mínimo), **ou**
2. Ter gerado **R$ 20** de rake PM pessoal (2.000 centavos)

Quem só indica e não senta **zera a linha naquela semana**. Os pontos não acumulam “de graça”. Os jogadores da ponta continuam jogando; o patrocinador inativo simplesmente não leva VL.

### 4.5 Liquidação semanal (sexta 18h BRT)

ZT Points **não** viram fichas de cash que se misturam com os R$ 150 do reset. Isso bagunçaria a economia da sala. Viram:

1. **Seats de freeroll da rede** (prêmio principal — gente sentada = liquidez)
2. **Tickets de MTT Play Money** com overlay simbólico
3. **Ranking / badge** (Recreacional → Agente → Clube), visível no lobby

Teto semanal sugerido: o equivalente a **2 seats** de freeroll por agente qualificado, para não inflar prize pool. O restante dos pontos vira posição no ranking do mês.

**Nunca:** PIX, saque, conversão para `balance_real`, “vender pontos”, transferência entre contas.

### 4.6 Cadastro

- Grátis. Sem kit. Sem “taxa de ativação”.
- Um patrocinador só. Se chegar sem link, cai no **clube casa** (Zero Tilt) até alguém adotá-lo pelo admin.
- E-mail verificado (já é regra da demo: Resend + código de 6 dígitos).

---

## 5. Fases (o que fazer na prática)

### Fase A — 0 a 30 dias: mesa viva

Objetivo único: **≥ 2 pessoas na mesma mesa, no mesmo modo, no horário nobre**.

- Convite só Play Money. Texto pronto na seção 7.
- 10 a 20 embaixadores (amigos que **jogam**, não “vendedores”).
- Janela âncora: **20h–24h BRT**, quatro noites por semana.
- Combinar **mesa e variante** no grupo (Hold’em 0,25/0,25 é a porta de entrada).
- Proibido no grupo: pedir PIX, “fichas combinadas”, “depois a gente acerta”, discurso de renda.

KPI da fase: mãos/semana na demo e picos de `GET /api/presence/online`. Sem isso, rede nenhuma pega.

### Fase B — 30 a 90 dias: MMN no papel + admin que já existe

- Abrir **1 clube âncora** pelo admin HTTPS (`POST /api/admin/clubs`).
- Cadastrar agentes (`POST /api/admin/clubs/:id/agents`) com rakeback **20–35%** no começo (não 50% — reserva para quem realmente enche mesa).
- Planilha semanal de VP/VL (mesmo que o motor de pontos ainda não esteja no código: este documento é o modelo; a implementação vem depois).
- Freeroll de domingo só para quem bateu VP.

KPI da fase: árvore com profundidade real 2, **zero** nível 3 “no zap”, e 70%+ dos agentes qualificados jogando.

### Fase C — parceiro vê o molde

O relatório de parceiros mostra a árvore, os %, o ledger e os KPIs. O pedido é **capital de compliance + liquidez**, não “entrar no informal”. Até a porteira: **zero real na rede**.

### Fase D — interruptor (só com documentação correta)

Checklist mínimo, todos juntos:

- Autorização SPA **ou** white-label em operador já autorizado
- PSP com **aceite formal** de iGaming (o código sozinho não basta)
- KYC/AML, autoexclusão, limites de depósito
- Saque auditável; PIX production destravado **só então**
- Carteira Real continua **isolada** do Play Money

Aí: `1 ZT Point` → `1 centavo real` no mesmo split 15/85. Agentes **não mudam de lugar**. Contrato de profundidade 2 **não muda**.

---

## 6. Economia da rede (para o clube entender a conta)

Rake no pôquer recreacional costuma ficar na casa de **2,5% a 5% do pote**, com teto por mão. O número exato é o do motor; o que o clube precisa gravar é a **ordem**:

`potes → rake → split 15/85 → Loss Deflator no pote líquido → pagamentos`

O Loss Deflator **não** sai do bolso da casa nem do clube: sai do pote em que o perdedor estava. Não negociar “desligar o deflator para aumentar rake” — isso mata a marca Zero Tilt.

Régua de conversa com clube âncora (Play Money, portanto **pontos**, não BRL):

- 10 jogadores ativos × 30 mãos/noite × 4 noites × ~R$ 0,50 de rake médio/mão ≈ **R$ 600** de rake PM/semana
- 85% clube ≈ **510 ZT Points × 100** se o rake médio for outro — o importante é a **proporção**, não o valor de marketing
- Agente a 30% leva ~153 desses 510 **só da linha dele**

Quando virar real, a mesma conta vira centavos no `clubs.balance`. Por isso o ensaio precisa ser honesto agora: inflar VP com bots ou multi-conta **queima o molde** e o antifraude (IP /24, multi-account) existe para isso.

---

## 7. Como convidar (sem parecer MMN de renda)

### 7.1 Mensagem para jogador (WhatsApp / Discord)

```text
Zero Tilt — pôquer online de verdade, fichas virtuais, HTTPS:

https://zerotiltpoker.net

1) Cria a conta (senha forte, tipo PokerDemo1)
2) Confirma o e-mail (código de 6 dígitos; olha o spam)
3) No topo: Play Money
4) Lobby → Hold’em 0,25/0,25 → me avisa que sentou
5) Precisa de 2 pessoas na mesma mesa pra começar

Não é dinheiro real. É pra jogar, aprender e encher a sala.
Hoje 20h, mesa combinada.
```

### 7.2 Mensagem para futuro agente (amigo que traz gente)

```text
Quero que você seja agente do clube — 2 níveis só, sem taxa.

O que você faz: traz gente pra jogar Play Money e senta junto.
O que você ganha agora: pontos, seats de freeroll, ranking.
O que você NÃO ganha agora: dinheiro.

Se a plataforma regular, a mesma árvore vira rakeback de verdade.
Até lá, o jogo é o produto. Quem não joga não pontua.
```

### 7.3 Frases proibidas (queimam o parceiro depois)

- “Renda extra”, “primeiro a entrar ganha mais”, “taxa pra ativar”
- “Joga valendo no PIX do grupo”
- “Indica 10 e fica rico”
- Qualquer analogia pública com Amway/Herbalife **para o jogador final**

---

## 8. Papéis e rotina

| Papel | Faz | Não faz |
|-------|-----|---------|
| **Casa (você)** | Sobe a demo, health, admin de clubes, regras, freeroll semanal, corte de discurso ruim | Prometer BRL, ligar PIX de rede |
| **Clube (nível 1)** | Horário âncora, cultura da mesa, escolhe agentes, teto 50% | Vender vaga, abrir “subclube” |
| **Agente (nível 2)** | Indica, senta, tira dúvida de cadastro/e-mail | Recrutar “patrocinado do patrocinado” |
| **Jogador** | Joga, manda bug, indica no máximo virando agente | Depositar real “por fora” |

Rotina semanal da casa:

1. Segunda: olhar presença e mãos da semana
2. Quarta: ver quem não bateu VP (avisar: sem ticket no domingo)
3. Sexta 18h: fechar ranking e seats
4. Domingo: freeroll da rede, **mesmo feltro**, mesma regra Zero Tilt

---

## 9. KPIs (o que medir, senão é conversa)

| KPI | Meta Fase A (30d) | Meta Fase B (90d) | Por que importa |
|-----|-------------------|-------------------|-----------------|
| Jogadores que sentaram ≥1 mão | 20 | 80 | Ativação |
| Mãos/semana (todas as mesas PM) | 200 | 1.500 | Liquidez |
| Pico `online` 20h–24h | 4 | 12 | Sala percebida como viva |
| Agentes com VP na semana | — | ≥70% | Anti-pirâmide |
| Profundidade máxima observada | 1 | 2 | Nunca 3 |
| Tickets de freeroll realmente jogados | — | ≥80% dos emitidos | Ponto que vira gente na mesa |
| Bugs bloqueantes abertos >7 dias | 0 | 0 | Confiança |

Se VP cresce e mãos não crescem, alguém está “pontuando” sem jogar: **cortar**.

---

## 10. O que já existe no produto para este plano

Não precisa esperar um app de MMN para começar a Fase A.

- Demo pública HTTPS, e-mail verificado, MFA
- Carteiras **Play Money ≠ Jogo Real** (PM não senta em mesa Real)
- Admin de clubes, agentes, financials, tema white-label
- Split de rake 15/85 no motor, crédito de `club_rake` na liquidação
- Contador online, lobby com filtros, catálogo curto (Hold’em, Short Deck, Omaha Short Deck)
- Loss Deflator, Dica do Pró, história do pôquer — material de cultura, não de “venda”

O que **ainda é modelo** (não código, nesta entrega): ledger de ZT Points, link público de indicação no registro, dashboard do agente para o próprio agente, ranking de rede no lobby. Ver o relatório de parceiros, seção de melhorias.

---

## 11. Interruptor para dinheiro real (quando a papelada estiver correta)

Não se “liga o MMN em real”. Se **substitui a unidade** e se **abre a porteira**:

1. Mesma árvore `clube → agente → jogadores`
2. Mesmos % (15 / 85 / 0–50 da fatia do clube)
3. Mesmo teto de 2 níveis
4. Mesma qualificação por jogo (agora em rake real)
5. Liquidação deixa de ser ticket e passa a ser **centavos no ledger** + saque do clube já previsto em `POST /api/admin/clubs/:id/withdraw` — **somente** com PSP e licença

Até lá, o endpoint de saque do clube **não se usa** para esta rede. Play Money não saca.

---

## 12. Não faça

- Operar pôquer real “enquanto o registro não vem”
- Terceiro nível “só no WhatsApp”
- Taxa de adesão, kit, meta de cadastro com prêmio em dinheiro
- Misturar pontos de rede com carteira Real
- Desligar Loss Deflator, antifraude de IP ou verificação de e-mail para “crescer mais rápido”
- Prometer ao parceiro que o MMN Play Money **já é** receita em BRL

---

## 13. Checklist do dono (imprimir)

- [ ] Grupos e convites falam só **Play Money**
- [ ] Horário âncora combinado (20h–24h, mesa nomeada)
- [ ] Um clube âncora no admin; agentes com % ≤ 35% no início
- [ ] Qualificação 50 mãos ou R$ 20 de rake PM / semana
- [ ] Liquidação = tickets + ranking, nunca PIX
- [ ] Profundidade auditada: se aparecer nível 3, achatar para agente do mesmo clube
- [ ] Relatório de parceiros na pasta, com gaps honestos
- [ ] Interruptor (SPA + PSP + KYC + saque) **desligado** até a lista da Fase D estar completa

---

*Este plano não altera split, PIX, saque nem o motor. Ele descreve como crescer a sala com o que já existe e como a mesma rede vira negócio regulado depois.*

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-04):** S21 — Texas Hold’em rename + FT Short Deck 8-max + Omaha 5-max + Pineapple 6-max + Short Deck ranking trips>straight + torneio agendado 21:30 SP auto-start 5 + Pix Leofran + saque 24h + lobby max sempre + sim 100k/mesa **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–040 na VPS (cash Texas SD 8-max + Omaha 5-max + Pineapple 6-max + Texas rename + FT 8 + scheduled 21:30). Motor short_deck_massive + tournament_to_champion PASS (Texas/Omaha 5/Pineapple 6 até 1 campeão; flush>FH e trips>straight). VPS 2h real 100 contas: 980 mãos R$135,11 rake, 4 campeões MTT. Simulado Motor-Rust/src/bin/simulated_100.rs 100k/mesa (400k total). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT site: inscrição + horário agendado + popup FT; gameplay_ready=false (sem WS de torneio). Health público OK. Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
