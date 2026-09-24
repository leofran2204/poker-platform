# Demo com amigos — zerotiltpoker.net

Guia curto para convidar pessoas a testar e mandar feedback.

Casa: **ZT Poker** (header **Zero Tilt**). Slogan: **Estude. Jogue. Sem tilt.** Três jogos: Hold’em, Omaha 4 (baralho 52) e Brazilian Pineapple (Short Deck).

## O que cada amigo precisa fazer

1. Abrir **https://zerotiltpoker.net** (HTTPS público)
2. Ver o contador **“X online”** no topo (visitante via GET público; logado via heartbeat ~90s) e a mesa de vitrine na home
3. **Registrar** (username 3–30 `[A-Za-z0-9_]`, e-mail válido, data de nascimento, confirmação de 18+, senha forte + confirmação; ex. `PokerDemo1`). Convite é o **último** campo e opcional
4. **Verificar e-mail** — código de 6 dígitos (inbox + spam); tela `/verify-email`
5. No header, escolher o modo de carteira:
   - **Play Money** — fichas de diversão (renovam todo dia)
   - **Jogo Real** — saldo real; depósito e entrada exigem verificação manual da conta em **Verificação**
6. Ir ao **Lobby**, filtrar o stake desejado e combinar a **mesma mesa** com pelo menos **2 pessoas**
7. Clicar **Entrar** (frente fixa da mesa)
8. Jogar e anotar bugs / sensações

> **Importante:** 1 pessoa sozinha **não** inicia mão. Precisa de **≥ 2 assentos** na mesma mesa. Chame os amigos para o mesmo horário.

## Catálogo cash (Play Money e Jogo Real)

<!-- DOCUMENTATION_SYNC:CASH_CATALOG:START -->
| Mesa | Jogo | Blinds | Cap | Frente |
|------|------|--------|-----|--------|
| NL 0,25 | Texas Hold’em | 0,25 / 0,25 | 9 | R$ 25 |
| NL 0,75/1,50 | Texas Hold’em | 0,75 / 1,50 | 9 | R$ 150 |
| Omaha 0,50 | Omaha 4 Cartas | 0,50 / 0,50 | 6 | R$ 100 |
| Pineapple 0,50 | Brazilian Pineapple | 0,50 / 0,50 | 5 | R$ 75 |
<!-- DOCUMENTATION_SYNC:CASH_CATALOG:END -->

- **Omaha 4:** baralho 52; 4 cartas na mão; showdown exatamente 2 hole + 3 board; ranking clássico
- **Brazilian Pineapple:** baralho 36; 2 cartas no pré-flop e +1 após flop/turn/river; showdown 2+3; **trinca > sequência** e **flush > full house**

## Carteiras

| Item | Play Money | Jogo Real |
|------|------------|-----------|
| Cash | R$ 150 / dia (reset SP, sem rebuy) | Depósito manual PIX + aprovação |
| Torneio | R$ 150 / dia (reset SP, sem rebuy) | Buy-in com saldo real |
| Mistura | **Não** — PM não entra em mesa Real e vice-versa | idem |

## Torneios

<!-- DOCUMENTATION_SYNC:MTT_CATALOG:START -->
| Evento | Variante | Buy-in | GTD | Cap | Máx. | Reentradas |
|--------|----------|--------|-----|-----|------|------------|
| Texas Hold’em | Texas Hold’em | R$ 15 | R$ 150 | 9 | 27 | 1 |
| Texas R$25 | Texas Hold’em | R$ 25 | — | 9 | 27 | 1 |
| Texas Hold’em Freeroll | Texas Hold’em | Grátis | R$ 75 | 9 | 27 | 1 |
| Omaha 4 Cartas | Omaha 4 Cartas | R$ 10 | R$ 100 | 6 | 18 | 1 |
| Brazilian Pineapple | Brazilian Pineapple | R$ 10 | R$ 100 | 5 | 15 | 1 |
<!-- DOCUMENTATION_SYNC:MTT_CATALOG:END -->

Data e horário definidos pelo **admin** em `America/Sao_Paulo`, com auto-start para **5+** jogadores. Inscrição no lobby com **taxa 15% por cima** (freeroll grátis) — **mãos MTT ao vivo em 3 mesas**; a página lista `live_table_ids` (não só a mesa 0). Cancelar inscrição devolve buy-in + taxa antes do start.

## Pix / Saque (Jogo Real)

- **Antes do depósito:** concluir a verificação de identidade. Enquanto a análise estiver pendente, Play Money continua disponível.
- **Depósito automático (DePix):** até R$ 50 as fichas entram ao confirmar o PIX (`processing`); o saque fica travado até a liquidação (`completed`). Acima de R$ 50 o saldo espera o `completed`. Valor creditado é o líquido (taxa DePix no extrato).
- **Depósito manual (fallback):** recebedor **Leofran**, chave `6eefcd53-686e-42d4-a062-03751336251c`. Pague no app do banco, cole o comprovante e aguarde aprovação.
- **Saque:** informe sua chave Pix (titular); **recebimento em até 24h**. Com depósito provisório em aberto o saque é recusado.

## Proteção e suporte

- Em **Jogo Responsável**, o jogador configura limites de depósito, perda e tempo; reduções são imediatas e relaxamentos aguardam 24 horas.
- A autoexclusão bloqueia imediatamente operações e ações de Jogo Real pelo período escolhido; não há cancelamento antecipado pela interface.
- **Suporte** permite abrir e acompanhar chamados. Nunca pedir nem enviar senha, código de e-mail ou MFA.
- **Esqueci minha senha** envia código de uso único sem revelar se o e-mail está cadastrado.

## Mensagem pronta para WhatsApp / Discord

```text
Teste do Zero Tilt Poker (demo HTTPS):

https://zerotiltpoker.net

1) Crie conta (senha tipo PokerDemo1 — maiúscula + minúscula + número)
2) Confirme o e-mail (código 6 dígitos; olhe o spam)
3) No header: Play Money (imediato) ou Jogo Real (após Verificação)
4) Lobby → escolha mesa (NL 0,25 · NL 0,75/1,50 · Short Deck / Omaha / Pineapple) → Entrar
5) Me diga o que travou ou gostou

Play Money = fichas virtuais. Jogo Real = saldo separado.
Precisa de 2+ pessoas na mesma mesa para começar a mão.
```

## Do seu lado (anfitrião)

1. Stack na VPS: `zerotiltpoker.net` (API + Frontend-Web + Caddy). Release atual usa migrations até **058**, incluindo proteção do jogador, KYC básico/manual, recuperação de senha e suporte.
2. Health: `https://zerotiltpoker.net/api/health` e `/api/presence/online`
3. Peça feedback: registro, e-mail, verificação, limites, suporte, modo carteira, lobby, join, lag, mobile e crashes

## Limites honestos

- Demo/staging: se a VPS cair, o site some
- Rate limit de auth ~30 req/min por IP
- MTT: inscrição + mãos ao vivo nas 3 mesas; rebalance/FT no coordenador
- Não alegar certificação de produção

## Checklist rápido

```text
[ ] /api/health → OK
[ ] /api/presence/online → JSON online_count
[ ] Badge “N online” no header
[ ] Registrar + verificar e-mail
[ ] Toggle Play Money / Jogo Real no header
[ ] Lobby lista NL 0,25 · NL 0,75/1,50 · SD 0,25/0,50 · Omaha · Pineapple (+ Torneios)
[ ] Dois perfis no MESMO modo entram na MESMA mesa
[ ] Mão inicia com ≥ 2 assentos
```

<!-- DOCUMENTATION_SYNC:START -->
> **S24** (2026-09-24) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
