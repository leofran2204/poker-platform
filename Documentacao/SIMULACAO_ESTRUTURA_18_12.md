# Ata — Teste de Sustentabilidade 18/12 (Fase D)

**Data:** 2026-09-07/08 · **Ambiente:** VPS `zerotiltpoker.net` (demo/staging, play money)
**Escopo:** rede Minha Estrutura 18%/12% em cash half/half + MTT + fee 15% + cenários funcionais e adversariais.
**Contas:** 15 `t18_*` em 2 árvores + ciclo + L3 + fantasmas + sem-patrocínio; 6 `smoke_mtt_*` (suporte); frota `bot_*` lag_v2.
**Ferramentas:** `scripts/estrutura-ws-hold.mjs` (modos nit/loose/fest, heartbeat, anti-queda),
`scripts/estrutura-auditoria.sql` (gates G1–G4).

## Volume executado

- Cash half/half nas 4 mesas (NL 4+5 · SD 4+4 · Omaha 3+2 · Pineapple 2+4 → depois 4+2): **~700 mãos/hora** no pico; VP 200–700 mãos/conta (meta: 50).
- MTT: Pineapple 5 contas + 5 bots (running); Omaha/Texas com bots; 1 freeroll anterior até o campeão.
- Rodadas: nit (VP) → loose-passive 35% (rake real) → allin-fest NL (potes máximos).

## Gates — todos verdes

| Gate | Resultado |
|---|---|
| G1 matemática 18/12 exata (incl. truncamento) | **0 linhas erradas** em 8.400+ linhas |
| G1b níveis/tipos válidos; G1c fee↔hand nulos | 0 / 0 |
| G2 casa ≥ 70% | **98,61%** (11.527 total: 900 fee + 10.627 rake; rede 160) |
| G2b bots fora da rede | 0 linhas com bot |
| G3 retido sem crédito + sem retroatividade | n1b1 54 e raizB 90 retidos; pontos intactos; flip VP gera só linhas novas elegíveis |
| G4 zero 5xx/halts | nenhum `MTT_TABLE_HALTED` no período |

## Cenários

- **F1–F4** (árvore, sem-patrocínio, só-L1, cheia): exatos. Linha de prova: `n2a1→n1a1 L1 37→6` + `n2a1→raizA L2 37→4` (18%/12% com truncamento), pontos 6+10 creditados.
- **F5 VP mãos** ✓ (50+ em todas as mesas ativas). **VP por rake (R$ 20, <50 mãos): não observado** — gap de cobertura.
- **F6 fantasma** ✓ (n1a1/raizA: 355 linhas retidas, 0 pontos pré-VP). **F7 virada** ✓ (raizB 353 elegíveis + 111 antigas inelegíveis).
- **A1 auto-convite: não testado** — gap.
- **A2 ciclo** ✓ (cyc1↔cyc2 geram L1 mútuo; trava `l2==source` impede auto-crédito L2; sem loop).
- **A3 L3** ✓ (deep1 paga n2a1 L1 + n1a1 L2; **raizA recebe zero**).
- **A4 bots** ✓ (0 linhas; fee de bot 100% casa).
- **A5 allin-fest** ✓ (conservação por mão; potes máximos geraram as 3 linhas pagas).
- **A6 MTT** ✓ (só fee pontua; mãos MTT rake 0 sem linhas).
- **A7 disconnect** ✓ por desenho (sweep 45s observado esvaziando Omaha/SD após fim do holder).

## Achados de produto (não-bugs)

1. **Truncamento inteiro em micro-stakes:** maior share em 8.438 linhas foi 2c → comissão 0. Rede 18/12 é simbólica no NL25; paga de verdade em potes ≥ ~R$ 6 de share ou fees. Considerar piso/arredondamento futuro (fora deste escopo).
2. **Boot pausa mesas com guard órfão** (6x após deploys) — fluxo auditado `recovery/abort` + reopen validado; cogitar auto-resume supervisionado.
3. **Sem MTT play em `registering`** após os atuais fecharem — ciclo de vida precisa de reset para operação contínua.
4. Cash actor tinha o mesmo stall all-in do MTT (Pineapple parou 18:13); corrigido com run-out (`d25e752`).

## Veredito

**Modelo sustentável e correto por construção:** margem da casa ≥ 98% no volume testado (pior caso teórico 70%), anti-pirâmide (VP) funcional, sem vazamento em ciclos/L3/bots. Aprovado para seguir operando a demo; revisitar com stakes maiores antes de qualquer alegação de receita real.
