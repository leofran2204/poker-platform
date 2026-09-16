# EP07 — Pot odds v2 (versão aprofundada ~85s)

Público: iniciante. Uma ideia só: a conta pagar-vs-pote decide o call; implied decide os duvidosos.

## Arco narrativo
1. Gancho: quanto custa ver a próxima carta?
2. Conta 25 em 75 = 4:1, precisa de 20%.
3. Outs: flush draw 9 outs, 36% flop-river, 18% turn-river; regra x4/x2.
4. Compara: 36 vs 25 paga, vermelho folda.
5. Fecho: implied nos duvidosos; próximo: EV e fold equity.

## Cenas (script.py v3 visual) — waits casados com áudio por cena (1.0x)
- S1: título + pilhas 25/100. S2: EquityBar 20%. S3: 9 versos outs.
- S4: EquityBar 36%. S5: fecho. Biblioteca `shared.py`. Total ~84s.

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6. Monospace, sem LaTeX.
- Narração PT-BR por cena (segN.txt → segN.mp3) muxada via ffmpeg. Termos reescritos (pote odes, outs, implied).

## Narração (~210 palavras, ~84s em 1.0x, áudio por cena)
Voz Edge-TTS pt-BR-AntonioNeural. Texto integral (5 atos = seg1..seg5):

"Quanto custa ver a próxima carta? Essa pergunta vale dinheiro — e a resposta se chama pote odes. É a conta mais importante do poker depois da posição.

A conta é simples: divida o que você tem que pagar pelo pote total depois do seu call. Aposta de vinte e cinco num pote de setenta e cinco? Você paga vinte e cinco para concorrer a cem: quatro para um. Precisa de vinte por cento de chance para o call empatar.

E de onde vem a chance? Dos outs: cartas que te colocam na frente. Flush draw tem nove outs — cerca de trinta e seis por cento do flópi ao ríver, dezoito do térn ao ríver. Regra de bolso: outs vezes quatro no flópi, vezes dois no térn.

Compare sempre: nove outs no flópi contra aposta de meio pote? Trinta e seis contra vinte e cinco: call lucrativo. Oesdi sem posição contra overbete? A conta fecha no vermelho: folde sem apego.

E tem a segunda conta: implied odes, o que você ganha a mais quando acertar. Contra pagador apaixonado, pague um pouco pior. Contra rocha que larga tudo, exija a conta exata. Pote odes decide o call, implied decide os casos duvidosos. No próximo episódio: valor esperado e fold equity."
