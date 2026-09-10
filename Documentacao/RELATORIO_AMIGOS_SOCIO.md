# Zero Tilt — conversa com amigos (e um possível sócio)

**Para quem:** amigos que jogam, entendem de produto, ou poderiam entrar como sócio de suor.  
**Data:** 2026-09-04 · ciclo **S21**  
**Demo:** https://zerotiltpoker.net  
**Situação de caixa:** zero recurso além da nuvem. Não há rodada, não há folha, não há mídia paga.

Leia em voz alta em 4 minutos. Se alguém pedir “prova”, o site e o repositório existem. Se alguém pedir “já dá para faturar”, a resposta honesta é **não**.

---

## 1. O que é

Zero Tilt é uma **sala de pôquer recreativa**, feita do zero (não é skin em cima de cassino). Motor e API em **Rust**, tela em **TypeScript**. Inspiração de mesa: Full Tilt clássico, em português de verdade.

A tese tem três pernas:

1. A mesa tem que ser honesta (centavos inteiros, baralho auditável, liquidação assinada).
2. O jogador não pode sair destroçado (Play Money que renova, frentes fixas, Loss Deflator de bad beat).
3. O crescimento é de **clube**, não de anúncio: quem já está dentro chama quem joga.

Hoje o jogo público que faz sentido é **Play Money** (fichas de treino). Dinheiro real na lei brasileira exige papelada que este projeto **ainda não tem** — e não vamos fingir que tem.

---

## 2. Por que isso não é “mais um site de fichas”

Quem já tentou pôquer online no Brasil conhece duas dores: sala vazia e sala que trata o jogador como caixa eletrônico.

Aqui o catálogo é **curto de propósito** (uma mesa por variante, frente fixa). Short Deck e Pineapple não são nome no lobby: o motor troca baralho e ranking (trinca ganha de sequência; flush ganha de full house). Parceiro técnico testa isso em cinco minutos.

O que já está no ar e pode ser clicado:

- HTTPS no domínio, e-mail verificado, MFA
- Cash ao vivo: Hold’em 9-max, Short Deck 8-max, Omaha curto 5-max, Pineapple 6-max
- Duas carteiras de treino (cash e torneio), R$ 150 cada, reset diário
- Carteira Real **isolada** da de treino (não mistura salário com fichas de brincadeira)
- Admin de clubes e agentes (split 15% casa / 85% clube no motor)
- Stress do motor na casa das dezenas/centenas de milhares de mãos simuladas

Isso é produto. Não é pitch de PowerPoint.

---

## 3. O que ainda não somos (deixe visível)

Esconder gap queima sócio. Lista curta:

| Ainda não | O que isso significa na prática |
|-----------|----------------------------------|
| Mãos de **torneio ao vivo** no site | Ligado (S22): 3 mesas por torneio, eliminações, mesa final e prêmios. Cash sim. |
| Sala cheia | Sem 6 pessoas no Hold’em no mesmo horário, não existe rede nem “marca”. |
| Licença / KYC / Pix automático de produção | O código **recusa** Pix de produção. Não vamos operar informalmente “valendo”. |
| Multi-servidor de mesas | Uma mesa = um processo. Aguenta demo e clube pequeno. Não é PokerStars. |
| Link de indicação no cadastro | O plano de clube + agente está no papel e no banco; o `?ref=` ainda não. |
| Autoexclusão e limites de tempo | O nome Zero Tilt cobra isso no real. No treino, ainda é cultura, não botão. |

Quem entrar agora não compra um cassino. Compra **um motor pronto + uma sala para encher**.

---

## 4. A conta, sem romance

Não há folha. Não há mídia. A nuvem já está paga pelo fundador.

Rake de 980 mãos de teste com contas sintéticas deu cerca de **R$ 135** de comissão da casa — prova de motor, não de negócio. Com humanos, no micro, 15% de uma mesa vazia continua **zero**.

Por isso o sócio que este projeto precisa **agora** não é quem “põe dinheiro para ligar Pix”. É quem:

- **senta e traz gente** (liquidez), ou
- **fecha o produto** (torneio ao vivo, convite, mesa âncora), ou
- **abre porta legal** no futuro (advogado / operador autorizado) — sem isso, o real não liga.

85% do rake para o clube só vira conversa de dinheiro **depois** de licença. Até lá, a moeda é **ticket de freeroll e mesa cheia**.

---

## 5. Como crescemos sem caixa (clube fechado)

Operação informal **entre cadastrados**, Play Money:

1. Uma mesa âncora: Texas Hold’em 0,25/0,25, 20h–24h, quatro noites.
2. Quem já tem conta chama quem **joga**, não quem “quer renda”.
3. Bonificação: **18% do rake individual** de cada um do seu 1º nível e **12%** de cada um do 2º (mão jogada, qualquer clube). Resto à casa. Clube não recebe. Cadastrou e sumiu = zero. O vínculo é o **ID do patrocinador**.
4. Dois andares só na **sua** tela. Sem nível 3. Sem Pix no grupo.

Isso é o máximo que dá para fazer com zero real e um servidor. Se a mesa das 21h não encher, nenhum organograma salva.

---

## 6. O pedido (escolha um chapéu)

Não pedimos cheque. Pedimos um dos três:

**A — Sócio de liquidez**  
Joga, chama 8–15 pessoas sérias, segura o horário âncora por 90 dias. Em troca: lugar de clube âncora na árvore (quando o real for legal, a mesma cadeira vira os 85% da liquidez **dessa** rede).

**B — Sócio de produto**  
Quem fecha o torneio ao vivo no site, o convite `?ref=`, o painel do agente. Em troca: participação no software / operação, a combinar por escrito (simples, duas páginas).

**C — Sócio de porteira (depois)**  
Capital de advogado, PSP e KYC. Só faz sentido **depois** da mesa âncora estar viva. Quem chega com dinheiro pedindo Pix no grupo não é o sócio certo.

Quem quiser os três chapéus, conversamos. Quem quiser atalho ilegal, não.

---

## 7. Como conferir em 20 minutos (roteiro do cético)

1. Abrir https://zerotiltpoker.net — cadeado HTTPS.
2. Criar conta, confirmar e-mail.
3. Header em **Play Money**. Lobby: Hold’em 0,25/0,25. Precisa de **2 pessoas** na mesma mesa.
4. Jogar 3 mãos. Ver se o turno é óbvio e se o timeout folda.
5. Abrir um torneio: inscrição com taxa 15% por cima do buy-in; mãos ao vivo ligadas (S22). Conferir campeão + prêmio de um freeroll já encerrado.
6. Perguntar o que **não** está pronto (seção 3). Se o fundador pular, não feche.

---

## 8. Fecho

O trabalho pesado de programa já foi feito: motor em Rust, regras de verdade, demo no ar, honestidade de staging. O trabalho de **sala** ainda não: gente no mesmo feltro, no mesmo horário.

Zero caixa não é desculpa para mentir. É o motivo de o sócio certo ser quem **enche a cadeira** ou quem **liga o torneio** — não quem promete anúncio.

O nome Zero Tilt só vale se a mesa das 21h existir. O resto é conversa.

*Fonte operacional: `STATUS_OPERACIONAL.json` (S21). Este texto não alega certificação de produção, Pix de produção, nem autoexclusão pronta.*

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-10):** S23 — cancela inscrição com reembolso total + admin agenda e cria torneios + textos da TournamentPage **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–049 na VPS (045 convite/fila, 046 ledger estrutura, 047 bots, 048 3 mesas + fee ledger, 049 total_fees). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor 1848 lib (fee 15%, seating 3 mesas, run-out all-in) + API 43 lib + ator MTT 2 testes integração PASS. VPS: 1º MTT fim a fim (freeroll 6 inscritos, 3 mesas, 5 mãos assinadas, campeão + payout GTD). Bots lag_v2 em MTT (12 inscritos, 43 mãos assinadas, zero erros). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT: inscrição + 3 mesas + gameplay WS ao vivo (mesmo protocolo do cash) + rebalance/consolidação FT + payouts; gameplay_ready=true. Health público OK. Diário de mãos + replay no frontend (2026-09-10): grava suas mãos no navegador (suas cartas, board, pote, resultado), replay passo a passo, download TXT/JSON, painel do vencedor sem botão (só as 5 cartas saltam, some sozinho em 7s). Ritmo de digestão no frontend (2026-09-10): board com stagger de 220ms por carta + painel de resultado fixo do showdown (vencedor, mão e cartas reveladas, sem auto-fechar, sobrevive à mão seguinte). Bots da casa desligados na VPS (stop oficial, reembolso) a pedido. Ritual do crupiê no frontend (2026-09-10): banner de embaralhamento + cartas distribuídas por assento a partir do dealer, versos para os oponentes, stagger no board. Migration 053 (2026-09-10): cash Texas 9-max NL 0,75/1,50 frente 15000 em PM e Real + torneios Texas R$25 em PM e Real (buy-in 2500, stack 15000, 1 reentrada 2500/25000, agenda 21:30 SP, auto-start 5). Bots externos de estratégia validados no local em 2026-09-10: bot/strategy (ranges cash 6-max/9-max, MTT ChipEV/ICM/PKO, avaliador próprio 5-7 cartas, push/fold FT, pot odds) com tsc limpo + 13/13 selftest offline; scripts/strategy-bots.mjs jogou mesa real PM NL 0,25 (3 bots, 5-6 mãos cada, 5 mãos no hand_history, decisões por ranges/odds). Tabelas ICM versionadas em bot/strategy/icm/tables (geradas de final_table.ts/bubble.ts via generate.mjs). Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo (cash TableActor + torneio TournamentActor, mesmo protocolo); settlement assinado (HMAC) na liquidação; halt de mesa MTT auditável (MTT_TABLE_HALTED).
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
