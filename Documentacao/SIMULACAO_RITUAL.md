# Simulação ritual Play Money (2026-09-02)

Ritual “como se fosse real” contra `https://zerotiltpoker.net` (Play Money) + motor MTT até o campeão. **Não é certificação de produção.**

## O que o site faz e o que não faz

| Pedido | Resultado |
|--------|-----------|
| Registro + verificação de e-mail | Mail.tm; ~15 s/conta; DNS `api.mail.tm` falhou em alguns lotes |
| 1 pessoa / 1 e-mail por assento cash | **25/25** nas 4 mesas play (NL 9, SD 6, Omaha 4, Pineapple 6) |
| 2 reservas por mesa | Provisionadas; sentam no cash quando alguém zera |
| 2.000 mãos cash ao vivo no WS | **Não é o gate do motor.** Ritmo real: timeout de turno 30 s + pausa 6 s entre mãos. Hold'em 9-max ~79 mãos no run; Omaha ~305. 10k mãos já existem em `cash_catalog_10k_hands`. |
| MTT até o campeão no site | **Não.** `gameplay_ready: false` sempre; sem `/ws` de torneio. Inscrição/rebuy/addon na API: rebuy/addon **405**. |
| MTT até o campeão no motor | **PASS** `tournament_to_champion` (Docker Linux na VPS) |
| Pedido de fichas / saque | `deposit-request` 200 pending (Play). PIX withdraw 400 (saldo Real zerado). `pm-rebuy` 400 se o saldo não está zerado |
| Addon | Catálogo `allow_addon=false`; motor recusa |

## Motor MTT até o campeão

Campo = `table_max × 3` + 2 reservas **por mesa**. 1 rebuy até o nível 6; reservas entram depois. Addon tentado e recusado. 20 órbitas ≈ 5 min de relógio.

| Torneio | Campeão | Mãos | Relógio | Observação |
|---------|---------|------|---------|------------|
| Hold'em 9-max | p4 | 765 | ~90 min (nv. 18) | 6 rebuys; 6 reservas |
| Freeroll Long→SD | p13 | 563 | ~55 min (nv. 11) | 15 rebuys |
| Omaha 4 | w3 (reserva) | 842 | ~105 min (nv. 21) | 0 rebuys |
| Ultimate Pineapple 6 | p14 | 804 | ~95 min (nv. 19) | 2 rebuys |

`finish_tournament` paga só quem ainda está vivo: com 1 campeão, só o 1º (50% do poço). 2º/3º da ordem de eliminação não recebem no motor.

Late-reg do catálogo fecha no nível 4; o teste de simulação sobe para 26 para as reservas entrarem após o rebuy (nível 6), como pedido da simulação.

## Ritmo ao vivo vs plataformas

A VPS **não saturou** (API ~0,2% CPU com 25 WS). A lentidão das 2.000 mãos veio de **625 timeouts de 30 s** (bots sem agir) + **6 s entre mãos**. Com humanos que agem em 3–8 s, 9-max fica na média online (~55–75 mãos/h), um pouco mais calmo que Stars/GG.

## Assentos fantasma (corrigido em S20f)

Matar o script deixou 25 `ACTIVE`. O lobby escondia mesa cheia. Ops: `scripts/clear-zombie-play-seats.sql`. Código: cash-out 45 s após disconnect + reconciliação no boot + lobby lista mesas lotadas.

## Como repetir

```bash
# Motor (VPS Linux / Docker)
docker run --rm -v "$PWD":/app -w /app/Motor-Rust rust:bookworm \
  cargo test --test tournament_to_champion -- --nocapture

# Site (não usar 2000 mãos como gate)
ALLOW_TEMP_MAIL=true HANDS_PER_TABLE=2 node scripts/live-sim-full-ritual.mjs
```

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-04):** S21 — Texas Hold’em rename + FT Short Deck 8-max + Omaha 5-max + Pineapple 6-max + Short Deck ranking trips>straight + torneio agendado 21:30 SP auto-start 5 + Pix Leofran + saque 24h + lobby max sempre + sim 100k/mesa + PM 150+150 sem rebuy (ilimitado com saldo, play money) **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–044 na VPS (cash Texas SD 8-max + Omaha 5-max + Pineapple 6-max + Texas rename + FT 8 + scheduled 21:30 + PM 150+150 + restore catálogo + Dockerfile cache). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor short_deck_massive + tournament_to_champion PASS (Texas/Omaha 5/Pineapple 6 até 1 campeão; flush>FH e trips>straight). VPS 2h real 100 contas: 980 mãos R$135,11 rake, 4 campeões MTT. Simulado Motor-Rust/src/bin/simulated_100.rs 100k/mesa (400k total). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT site: inscrição + horário agendado + popup FT; gameplay_ready=false (sem WS de torneio). Health público OK. Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
