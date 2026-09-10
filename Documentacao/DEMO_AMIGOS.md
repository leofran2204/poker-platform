# Demo com amigos — zerotiltpoker.net

Guia curto para convidar pessoas a testar e mandar feedback.

## O que cada amigo precisa fazer

1. Abrir **https://zerotiltpoker.net** (HTTPS público)
2. Ver o contador **“X online”** no topo (visitante via GET público; logado via heartbeat ~90s) e a mesa de vitrine na home
3. **Registrar** (username 3–30 `[A-Za-z0-9_]`, e-mail válido, senha forte + confirmação; ex. `PokerDemo1`). Convite é o **último** campo e opcional
4. **Verificar e-mail** — código de 6 dígitos (inbox + spam); tela `/verify-email`
5. No header, escolher o modo de carteira:
   - **Play Money** — fichas de diversão (renovam todo dia)
   - **Jogo Real** — saldo real (precisa depósito aprovado / crédito admin)
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
| SD 0,25/0,50 | Texas Short Deck | 0,25 / 0,50 | 8 | R$ 75 |
| SD Omaha 0,50 | Short Deck Omaha | 0,50 / 0,50 | 5 | R$ 100 |
| Pineapple 0,50 | Ultimate Pineapple | 0,50 / 0,50 | 6 | R$ 75 |
<!-- DOCUMENTATION_SYNC:CASH_CATALOG:END -->

- **Texas Short Deck:** baralho 36 (sem 2–5); **trinca > sequência** e **flush > full house**; wheel A-6-7-8-9  
- **SD Omaha:** 4 cartas na mão; no showdown usa exatamente 2 hole + 3 board; mesmo ranking Short Deck  
- **Ultimate Pineapple:** 3 cartas na mão, **sem descarte**; showdown 2 hole + 3 board; mesmo ranking Short Deck  

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
| Texas Hold’em Freeroll (FT Texas Short Deck 8-max) | Texas Hold’em | Grátis | R$ 75 | 9 | 27 | 1 |
| Omaha 4 Cartas | Short Deck Omaha | R$ 10 | R$ 100 | 5 | 15 | 1 |
| Ultimate Pineapple | Ultimate Pineapple | R$ 10 | R$ 100 | 6 | 18 | 1 |
<!-- DOCUMENTATION_SYNC:MTT_CATALOG:END -->

Início agendado **21:30 America/Sao_Paulo**, auto-start com **5+** jogadores; FT Short Deck troca só no próximo blind + popup. Inscrição no lobby com **taxa 15% por cima** (freeroll grátis) — **mãos MTT ao vivo em 3 mesas**; a página lista `live_table_ids` (não só a mesa 0). Cancelar inscrição devolve buy-in + taxa antes do start.

## Pix / Saque (Jogo Real)

- **Depósito:** recebedor **Leofran**, chave `6eefcd53-686e-42d4-a062-03751336251c`. Pague no app do banco, cole o comprovante e aguarde aprovação.
- **Saque:** informe sua chave Pix; **recebimento em até 24h**.

## Mensagem pronta para WhatsApp / Discord

```text
Teste do Zero Tilt Poker (demo HTTPS):

https://zerotiltpoker.net

1) Crie conta (senha tipo PokerDemo1 — maiúscula + minúscula + número)
2) Confirme o e-mail (código 6 dígitos; olhe o spam)
3) No header: Play Money (fácil) ou Jogo Real
4) Lobby → escolha mesa (NL 0,25 · NL 0,75/1,50 · Short Deck / Omaha / Pineapple) → Entrar
5) Me diga o que travou ou gostou

Play Money = fichas virtuais. Jogo Real = saldo separado.
Precisa de 2+ pessoas na mesma mesa para começar a mão.
```

## Do seu lado (anfitrião)

1. Stack na VPS: `zerotiltpoker.net` (API + Frontend-Web + Caddy). Migrations até **053** (inclui cash NL 0,75/1,50 e Texas MTT R$25).
2. Health: `https://zerotiltpoker.net/api/health` e `/api/presence/online`
3. Peça feedback: registro, e-mail, modo carteira, lobby, join, lag, mobile, crashes

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
> **S24** (2026-09-10) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático desligado.
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
