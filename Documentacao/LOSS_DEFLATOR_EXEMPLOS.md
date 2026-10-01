# Exemplos reproduzíveis — Loss Deflator

Atualizado em 27/09/2026. Regra normativa: [BUSINESS_RULES.md, seção 11](BUSINESS_RULES.md). Este documento explica contas; a disponibilidade operacional é definida no [STATUS](STATUS_OPERACIONAL.md).

## 1. Equity e base de cálculo

Equity é a parcela esperada do pote no showdown, incluindo a fração de empates. Ela depende da modalidade, cartas conhecidas, oponentes e hipóteses. Chance de completar um draw não é automaticamente equity. O deflator usa as cartas efetivas dos participantes considerados pelo motor; um jogador durante a mão não conhece necessariamente essas cartas.

A base financeira é o **pote líquido elegível**, após rake. Não é o aporte individual, o saldo da conta ou o valor perdido pelo jogador. Percentuais e teto compartilhado estão na regra normativa. A redistribuição sai das parcelas dos vencedores e conserva o total; não cria dinheiro nem usa caixa da plataforma.

O motor habilita o mecanismo para Texas Hold’em e Short Deck. Omaha e Brazilian Pineapple continuam excluídos até existir o modelo específico validado. Não transferir probabilidades de Hold’em para essas modalidades.

## 2. Um exemplo exato com cartas

Texas tradicional, heads-up, all-in no turn:

- Herói: Ad Ac.
- Rival: Jh Th.
- Board conhecido: Qs 9c 4d 2s.
- Oito cartas são conhecidas; restam 44 rivers possíveis.
- O rival vence nos quatro reis e quatro oitos: oito resultados. Nos outros 36, o herói vence. Não há empate.
- Equity do herói: 36/44 = 81,81818…%, faixa de 25%.
- Se o river for Kc, o rival forma sequência e o herói perde.

Adotando um pote elegível **já líquido** de 20.000 centavos e apenas esse perdedor elegível: cashback de 5.000, pagamento ao vencedor de 15.000. Soma: 20.000. Não inferimos o pote bruto ou rake a partir desses valores líquidos.

O teste `documented_turn_equity_is_exact_and_selects_twenty_five_percent` reproduz a equity e o cálculo em `Motor-Rust/tests/variant_audit_regressions.rs`.

## 3. Exercícios aritméticos por faixa

Nesta tabela a equity é uma entrada fornecida, não uma estimativa de mãos não especificadas. Um perdedor elegível, pote líquido de 50.000 centavos:

| Equity fornecida | Percentual | Ao perdedor | Ao vencedor |
|---|---:|---:|---:|
| 55% | 0% | 0 | 50.000 |
| 60% | 7% | 3.500 | 46.500 |
| 70% | 15% | 7.500 | 42.500 |
| 80% | 25% | 12.500 | 37.500 |
| 90% | 35% | 17.500 | 32.500 |

55% pode ser favorito heads-up e ainda ser inelegível. Com board completo e mãos fixas, um perdedor tem equity zero; uma entrada hipotética de 100% não representa um jogador que depois perde nesse mesmo modelo.

## 4. Potes paralelos e teto compartilhado

Os próximos casos são entradas contábeis ilustrativas. Tiers em snapshots distintos são fornecidos para demonstrar rateio, sem inventar cartas que os produzam.

### Um perdedor que participa apenas do main pot

Main pot líquido: 6.000; side pot: 16.000. Perdedor A é elegível apenas ao main e tem faixa de 15%. B vence ambos os potes. A recebe 900 do main; B recebe 5.100 + 16.000 = 21.100. Soma: 22.000. O side pot não financia A.

### Dois perdedores elegíveis em momentos diferentes

Main pot líquido: 12.000, elegíveis A/B/C. Side pot líquido: 12.000, elegíveis B/C. C vence ambos. A tem faixa de 15% e B de 35%, calculadas em snapshots distintos.

- Main: pedidos de 1.800 e 4.200; teto único de 35% × 12.000 = 4.200.
- Rateio proporcional 30%/70%: A recebe 1.260, B recebe 2.940.
- Side: só B é perdedor elegível; recebe 4.200.
- Totais: A 1.260; B 7.140; C 15.600. Soma: 24.000.

Somar os pedidos integrais violaria o teto do main. Também não se pode atribuir 70% de equity simultaneamente a dois jogadores no mesmo confronto e mesmo snapshot: suas parcelas e as dos demais somam 100%.

### Vencedores empatados

Pote líquido: 30.000. Um perdedor elegível tem faixa de 25%, recebendo 7.500. Dois vencedores empatados recebem 11.250 cada após financiar o benefício. Soma: 30.000. Havendo centavos indivisíveis, a ordem dos assentos a partir do botão resolve os resíduos, conforme o motor.

## 5. Limites da análise

Uma estimativa Monte Carlo não é uma garantia exata, sobretudo perto dos limites de faixa. Short Deck usa enumeração exata no código local corrigido. Em Hold’em, espaços maiores mantêm amostragem determinística. A auditoria de snapshots e de mãos históricas exige os registros reais de cada mão; esta documentação não recalcula nem movimenta saldos.

<!-- DOCUMENTATION_SYNC:START -->
> **S26** (2026-10-01) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
