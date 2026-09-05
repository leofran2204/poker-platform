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
| Mãos de **torneio ao vivo** no site | Dá para se inscrever; o jogo MTT ainda não está ligado à mesa. Cash sim. |
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
3. Bonificação: ranking + no máximo 2 tickets de freeroll por semana para quem jogou e para quem trouxe gente que **sentou** (≥ 20 mãos). Cadastrou e sumiu = zero.
4. Dois andares só (clube → agente → jogador). Sem nível 3 no Zap. Sem Pix no grupo.

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
5. Abrir um torneio: inscrição existe; o aviso de “mãos ainda não ligadas” tem que aparecer. Se alguém disser que o MTT já joga no site, está mentindo.
6. Perguntar o que **não** está pronto (seção 3). Se o fundador pular, não feche.

---

## 8. Fecho

O trabalho pesado de programa já foi feito: motor em Rust, regras de verdade, demo no ar, honestidade de staging. O trabalho de **sala** ainda não: gente no mesmo feltro, no mesmo horário.

Zero caixa não é desculpa para mentir. É o motivo de o sócio certo ser quem **enche a cadeira** ou quem **liga o torneio** — não quem promete anúncio.

O nome Zero Tilt só vale se a mesa das 21h existir. O resto é conversa.

*Fonte operacional: `STATUS_OPERACIONAL.json` (S21). Este texto não alega certificação de produção, Pix de produção, nem autoexclusão pronta.*
