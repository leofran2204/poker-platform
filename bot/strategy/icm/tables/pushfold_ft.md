# Tabelas ICM — Mesa Final (baseline, nao solver)

Fonte: bot/strategy/icm/final_table.ts + bubble.ts. Gerado em 2026-09-10.
Para trocar por saidas ICMIZER/HRC/GTO Wizard: substitua os .ts de origem e rode generate.mjs de novo.

## Push/fold por stack (9-handed, ICM medio)

| Stack | Late (BTN/SB/CO) | Early | Combos late (1326) |
|---|---|---|---|
| 5bb | `22+, A2s+, A2o+, K2s+, K7o+, Q2s+, Q7o+, J5s+, J9o+, T7s+, 97s+, 87s, 76s` | `22+, A2s+, A5o+, K8s+, KJo+, QTs+, JTs` | 562 |
| 8bb | `22+, A2s+, A2o+, K2s+, K6o+, Q4s+, Q9o+, J7s+, JTo, T8s+, 98s, 87s` | `66+, A2s+, A7o+, K8s+, KTo+, QTs+, JTs` | 510 |
| 10bb | `22+, A2s+, A3o+, K4s+, K9o+, Q8s+, QJo, J9s+, T9s` | `77+, A5s+, A9o+, K9s+, KQo, QJs` | 382 |
| 12bb | `77+, A2s+, A5o+, K9s+, KJo+, QJs, JTs` | `88+, A7s+, ATo+, KQs` | 252 |
| 15bb | `77+, A7s+, ATo+, KJs+, QJs, TT+` | `99+, AJs+, AQo+` | 136 |
| 20bb | `TT+, AJs+, AQo+` | `JJ+, AK` | 66 |

## Call vs shove (ICM)

- Curto: `88+, ATs+, AQo+`
- Profundo (15bb+): `TT+, AQs+, AK`

## Bolha: encolhimento do open

| Situacao | Fator (x range ChipEV) |
|---|---|
| Mid stack | 0.55 |
| Chip leader | 1.15 |
| Short stack | 0.9 |
| Outros | 0.8 |
| Longe da bolha (10+ do dinheiro) | 1 |

Exemplo MP: MP ChipEV ~19% (22+, ATs+, KTs+, QTs+, J9s+, T9s, 98s, 87s, 76s, AJo+, KQo) -> bolha ~9.5% (77+, ATs+, KJs+, QJs, AJo+, KQo)
