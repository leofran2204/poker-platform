# EP15 — C-bet: seco x molhado (versão aprofundada ~90s)

Público: M2 flop. Uma ideia só: textura manda no tamanho — seco barato sempre, molhado caro às vezes, meio-termo pelo range.

## Arco narrativo
1. Gancho: e agora? A resposta está na textura.
2. Seco (K72, A83): range com pares altos — frequente e barata 25-33%.
3. Molhado (987, JTs): acerta tudo — menos vezes, 60-75% por valor.
4. Meio-termo (Q92): manda a vantagem de range.
5. Fecho: regra de ouro; próximo: check-raise e float.

## Cenas (script.py v3 visual) — waits casados com áudio por cena (1.0x)
- S1: título (cartaz video 11) + mini-mesa. S2: mesa K72 + 25-33%. S3: mesa 987 flush + 60-75%.
- S4: mesa Q92 + range. S5: fecho. Biblioteca `shared.py`. Total ~86s.

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6. Monospace, sem LaTeX.
- Narração PT-BR por cena (segN.txt → segN.mp3) muxada via ffmpeg. Termos reescritos (flópi, bordo, bete).

## Narração (~222 palavras, ~86s em 1.0x, áudio por cena)
Voz Edge-TTS pt-BR-AntonioNeural. Texto integral (5 atos = seg1..seg5):

"Você aumentou pré-flópi e o flópi veio. E agora?

A resposta está na textura do bordo. Bordo seco: rei, sete e dois de naipes diferentes. Ás, oito e três. Quase ninguém acerta nada aqui. Por isso a continuation bete é frequente e barata: vinte e cinco a trinta e três por cento do pote. Funciona como blefe e como valor fino — e o seu range de agressor tem todos os pares altos que o pagador quase nunca tem.

Bordo molhado: nove, oito e sete em sequência. Valete, dez e cinco puxando pro flush. Aqui o oponente acerta pares, sequências e draws — e tem posição sobre o seu blefe. Então bete menos vezes. E quando apostar por valor, cobre caro: sessenta a setenta e cinco por cento, para tirar o preço de quem está comprando.

E tem o meio-termo traiçoeiro: bordo médio, tipo dama, nove e dois. Nem seco, nem molhado. Aqui manda a vantagem de range: se o bordo bate mais no seu range de agressor, bete pequeno e frequente. Se bate no range de quem pagou, controle o pote e cheque mais.

A regra de ouro: no seco, barato e sempre. No molhado, caro e às vezes. O tamanho da aposta conta a história — e a textura diz se ela é verdade. No próximo episódio: cheque-reise e floute."
