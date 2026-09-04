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
| NL 0,25 | Texas Hold’em | 0,25 / 0,25 | 9 | R$25 |
| SD 0,25/0,50 | Texas Short Deck | 0,25 / 0,50 | 8 | R$75 |
| SD Omaha 0,50 | Short Deck Omaha | 0,50 / 0,50 | 5 | R$100 |
| Pineapple 0,50 | Ultimate Pineapple | 0,50 / 0,50 | 6 | R$75 |

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

Catálogo MTT (PM e Real): Texas Hold’em R$15 GTD R$150 (9-max) · Texas Freeroll (FT Short Deck 8-max, 9-max) · Omaha 4 cartas 5-max R$10 GTD R$100 · **Ultimate Pineapple** 6-max R$10 GTD R$100. Início agendado **21:30 America/Sao_Paulo**, auto-start com **5+** jogadores; FT Short Deck troca só no próximo blind + popup. Inscrição no lobby — mãos MTT ao vivo ainda em evolução.

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
4) Lobby → escolha mesa (NL / Short Deck / Omaha) → Entrar
5) Me diga o que travou ou gostou

Play Money = fichas virtuais. Jogo Real = saldo separado.
Precisa de 2+ pessoas na mesma mesa para começar a mão.
```

## Do seu lado (anfitrião)

1. Stack na VPS: `zerotiltpoker.net` (API + Frontend-Web + Caddy). Migrations até **040** (Texas rename + FT 8 + Omaha 5 + scheduled 21:30 + SD 8-max).
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
> **Estado operacional sincronizado (2026-09-04):** S21 — Texas Hold’em rename + FT Short Deck 8-max + Omaha 5-max + Pineapple 6-max + Short Deck ranking trips>straight + torneio agendado 21:30 SP auto-start 5 + Pix Leofran + saque 24h + lobby max sempre + sim 100k/mesa + PM 150+150 sem rebuy (ilimitado com saldo, play money) **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–044 na VPS (cash Texas SD 8-max + Omaha 5-max + Pineapple 6-max + Texas rename + FT 8 + scheduled 21:30 + PM 150+150 + restore catálogo + Dockerfile cache). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor short_deck_massive + tournament_to_champion PASS (Texas/Omaha 5/Pineapple 6 até 1 campeão; flush>FH e trips>straight). VPS 2h real 100 contas: 980 mãos R$135,11 rake, 4 campeões MTT. Simulado Motor-Rust/src/bin/simulated_100.rs 100k/mesa (400k total). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT site: inscrição + horário agendado + popup FT; gameplay_ready=false (sem WS de torneio). Health público OK. Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
