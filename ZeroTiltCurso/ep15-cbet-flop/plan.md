# EP15 — C-bet: seco x molhado

Público: M2 flop. Uma ideia só: textura manda no tamanho — seco barato sempre, molhado caro às vezes.

## Arco narrativo
1. Gancho: "Abriu pré-flop e o flop veio. E agora? Olhe a textura."
2. Seco (K72, A83): poucos draws — c-bet frequente e barata (25-33%).
3. Molhado (987, JTs): acerta tudo — menos vezes, e por valor cobre 60-75%.
4. Regra de ouro em uma linha.
5. Fecho: tamanho conta história; próximo: check-raise e float.

## Cenas (script.py) — total ~50s
- S1_Titulo (7s): "C-BET: SECO X MOLHADO" + "episódio 15: a textura manda".
- S2_Seco (11s): "K 7 2 / A 8 3" + "25-33%, frequente".
- S3_Molhado (12s): "9 8 7 / J T 5 flush" + "menos vezes, 60-75% por valor".
- S4_Regra (10s): "seco barato sempre / molhado caro às vezes".
- S5_Fecho (10s): "tamanho conta história" + "próximo: check-raise e float".

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6. Monospace, sem LaTeX.
- Narração PT-BR separada (narracao.txt) muxada via ffmpeg.

## Narração (~155 palavras, 62.4s → ~50s em 1.25x)
Voz Edge-TTS pt-BR-AntonioNeural.

"Você aumentou pré-flop e o flop veio. E agora? Olhe a textura do board. Board seco: Rei, sete e dois de naipes diferentes. Ás, oito e três. Poucos draws possíveis, quase ninguém acerta. Aqui a continuation bet é frequente e barata: vinte e cinco a trinta e três por cento do pote. Funciona como blefe e como valor fino. Board molhado: nove, oito e sete em sequência. Valete, dez e cinco com flush draw. Aqui o oponente acerta pares, sequências e draws. Então aposte menos vezes. E quando apostar por valor, cobre caro: sessenta a setenta e cinco por cento, para tirar o preço dos draws. A regra de ouro: no seco, barato e sempre; no molhado, caro e às vezes. O tamanho da aposta conta a história — e o board diz se ela é verdade. No próximo episódio: check-raise e float."
