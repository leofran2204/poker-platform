# Simulação ritual Play Money (2026-09-02)

> **Snapshot de 2026-09-02.** MTT ao vivo no site veio depois (S22). Estado vigente: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md). As linhas abaixo que dizem `gameplay_ready: false` **não** são o produto atual.

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
> **S24** (2026-09-10) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático desligado.
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
