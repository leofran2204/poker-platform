# Demo com amigos — zerotiltpoker.net

Guia curto para convidar pessoas a testar e mandar feedback.

## O que cada amigo precisa fazer

1. Abrir **https://zerotiltpoker.net** (HTTPS público)
2. Ver o contador **“X online”** no topo — só conta quem **está logado** com heartbeat recente (~90s)
3. **Registrar** (username 3–30 chars, e-mail válido, senha forte + confirmação; ex. `PokerDemo1`)
4. **Verificar e-mail** — código de 6 dígitos (inbox + spam); tela `/verify-email`
5. No header, escolher o modo de carteira:
   - **Play Money** — fichas de diversão (renovam todo dia)
   - **Jogo Real** — saldo real (precisa depósito aprovado / crédito admin)
6. Ir ao **Lobby**, filtrar o stake desejado e combinar a **mesma mesa** com pelo menos **2 pessoas**
7. Clicar **Entrar** (frente fixa da mesa)
8. Jogar e anotar bugs / sensações

> **Importante:** 1 pessoa sozinha **não** inicia mão. Precisa de **≥ 2 assentos** na mesma mesa.

## Catálogo cash (Play Money e Jogo Real)

| Mesa | Jogo | Blinds | Cap | Frente |
|------|------|--------|-----|--------|
| NL 0,25 | Hold’em | 0,25 / 0,25 | 9 | R$25 |
| NL 0,50 | Hold’em | 0,25 / 0,50 | 9 | R$50 |
| SD 0,50 | Short Deck | 0,50 / 0,50 | 6 | R$75 |
| SD Omaha 0,50/1 | Short Deck Omaha | 0,50 / 1,00 | 4 | R$100 |

- **Short Deck:** baralho 36 (sem 2–5); flush > full house; wheel A-6-7-8-9  
- **SD Omaha:** 4 cartas na mão; no showdown usa exatamente 2 hole + 3 board  

## Carteiras

| Item | Play Money | Jogo Real |
|------|------------|-----------|
| Cash | R$ 1.000 / dia (reset SP) | Depósito manual PIX + aprovação |
| Torneio | R$ 15.000 / dia | Buy-in com saldo real |
| Mistura | **Não** — PM não entra em mesa Real e vice-versa | idem |

## Torneios

Freeroll R$100 GTD e MTT R$200 GTD em **NLHE e Short Deck**, modos PM e Real. Inscrição no lobby — mãos MTT ao vivo ainda em evolução.

## Mensagem pronta para WhatsApp / Discord

```text
Teste do Zero Tilt Poker (demo HTTPS):

https://zerotiltpoker.net

1) Crie conta (senha tipo PokerDemo1 — maiúscula + minúscula + número)
2) Confirme o e-mail (código 6 dígitos; olhe o spam)
3) No header: Play Money (fácil) ou Jogo Real
4) Lobby → escolha mesa (NL / Short Deck / Omaha) → Entrar
5) Me diga o que travou ou gostou

Play Money = fichas virtuais. Jogo Real = saldo separado.
Precisa de 2+ pessoas na mesma mesa para começar a mão.
```

## Do seu lado (anfitrião)

1. Stack na VPS: `zerotiltpoker.net` (API + Frontend-Web + Caddy). Migrations até **025**.
2. Health: `https://zerotiltpoker.net/api/health` e `/api/presence/online`
3. Peça feedback: registro, e-mail, modo carteira, lobby, join, lag, mobile, crashes

## Limites honestos

- Demo/staging: se a VPS cair, o site some
- Rate limit de auth ~30 req/min por IP
- MTT: inscrição ok; gameplay de mãos ainda limitado
- Não alegar certificação de produção

## Checklist rápido

```text
[ ] /api/health → OK
[ ] /api/presence/online → JSON online_count
[ ] Badge “N online” no header
[ ] Registrar + verificar e-mail
[ ] Toggle Play Money / Jogo Real no header
[ ] Lobby lista NL 0,25 · NL 0,50 · SD 0,50 · SD Omaha (+ Torneios)
[ ] Dois perfis no MESMO modo entram na MESMA mesa
[ ] Mão inicia com ≥ 2 assentos
```

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-29):** S18 — Catálogo cash NLHE+Short Deck+SD Omaha (PM×Real); frentes fixas; motor short_deck_omaha; notícias com capa temática; testes 10k/mesa + e2e seeded Real/PM. Demo VPS zerotiltpoker.net. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** VPS stack healthy (postgres, redis, api, frontend/Caddy). Migrations 001–025. Presence API no ar. Motor: cash_catalog_10k_hands PASS (4 configs × 10k); short_deck_massive PASS; tournament_engine 954 ok. Smoke live seeded Real+PM: join+≥1 mão em cada mesa OPEN; inscrição torneios OK. Mock/auto PIX bloqueado para gaming em vários PSPs. Fluxo vigente: Pedir fichas (depósito manual) + comprovante + aprovação admin. Nenhum gateway de saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
