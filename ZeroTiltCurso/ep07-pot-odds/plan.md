# EP07 — Pot odds v2 (versão aprofundada ~85s)

Público: iniciante. Uma ideia só: a conta pagar-vs-pote decide o call; impláid decide os duvidosos.

## Arco narrativo
1. Gancho: quanto custa ver a próxima carta?
2. Conta 25 em 75 = 4:1, precisa de 20%.
3. Áuts: flush draw 9 áuts, 36% flóp-ríver, 18% turn-ríver; regra x4/x2.
4. Compara: 36 vs 25 paga, vermelho folda.
5. Fecho: impláid nos duvidosos; próximo: EV e fold equity.

## Cenas (script.py v3 visual) — waits casados com áudio por cena (1.0x)
- S1: título (cartaz video 16) + pilhas 25/100. S2: EquityBar 20%. S3: 9 versos áuts.
- S4: EquityBar 36%. S5: fecho. Biblioteca `shared.py`. Total ~84s.

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6. Monospace, sem LaTeX.
- Narração PT-BR por cena (segN.txt → segN.mp3) muxada via ffmpeg. Termos reescritos (pote ódz, áuts, impláid).

## Narração (~210 palavras, ~84s em 1.0x, áudio por cena)
Voz Edge-TTS pt-BR-AntonioNeural. Texto integral (5 atos = seg1..seg5):

"Quanto custa ver a próxima carta? Essa pergunta vale dinheiro — e a resposta se chama pote ódz. É a conta mais importante do poker depois da posição.

A conta é simples: divida o que você tem que pagar pelo pote total depois do seu call. Aposta de vinte e cinco num pote de setenta e cinco? Você paga vinte e cinco para concorrer a cem: quatro para um. Precisa de vinte por cento de chance para o call empatar.

E de onde vem a chance? Dos áuts: cartas que te colocam na frente. Flâsh draw tem nove áuts — cerca de trinta e seis por cento do flóp ao ríver, dezoito do târn ao ríver. Regra de bolso: áuts vezes quatro no flóp, vezes dois no târn.

Compare sempre: nove áuts no flóp contra aposta de meio pote? Trinta e seis contra vinte e cinco: call lucrativo. ou-í-és-dí sem posição contra ouver-bét? A conta fecha no vermelho: folde sem apego.

E tem a segunda conta: impláid ódz, o que você ganha a mais quando acertar. Contra pagador apaixonado, pague um pouco pior. Contra rocha que larga tudo, exija a conta exata. Pote ódz decide o call, impláid decide os casos duvidosos. No próximo episódio: valor esperado e fold equity."
