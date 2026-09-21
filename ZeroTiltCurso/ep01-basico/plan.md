# EP01 — Poker do zero (versão aprofundada ~90s)

Público: quem nunca jogou. Uma ideia só: com 7 cartas, forme o melhor jogo de 5.

## Arco narrativo
1. Gancho: blinds, 2 fechadas, botão que anda.
2. 4 rodadas: pré-flop, flop 3, turn 1, river 1.
3. Melhor jogo de 5 leva (par até royal).
4. Royal de exemplo: A-K + Q-J-T de espadas.
5. Blefe: ninguém pagou, pote seu sem mostrar.
6. Fecho: showdown, empate divide; próximo: posição.

## Cenas (script.py v3 visual) — waits casados com áudio por cena (1.0x)
- S1: título (cartaz video 4) + 2 versos. S2: 5 versos flop/turn/river. S3: mesa + AA.
- S4: royal com glow. S5: pote do blefe. S6: fecho. Total ~85s. Piloto 720p30 (`manim -qm`), cartas e tipo maiores.

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6. Monospace, sem LaTeX.
- Narração PT-BR por cena (segN.txt → segN.mp3) muxada via ffmpeg. Termos reescritos (bláindes, flópi, térn, ríver, choudáun).

## Narração (~202 palavras, ~85s em 1.0x, áudio por cena)
Voz Edge-TTS pt-BR-AntonioNeural. Texto integral (6 atos = seg1..seg6):

"Toda mão começa igual: dois bláindes postos e duas cartas fechadas só suas. O small paga metade, o big paga cheio. E a cada mão o botão anda uma casa — todo mundo paga, todo mundo joga em posição.

São quatro rodadas de aposta. No pré-flópi, cada um decide: folde, pague ou aumente. Depois vêm as comunitárias, que valem para todo mundo: três no flópi, uma no térn, uma no ríver.

Com as suas duas mais as cinco da mesa, você forma o melhor jogo de cinco cartas. Par, dois pares, trinca, sequência, flush, full house, quadra: a hierarquia decide quem leva.

Olha o exemplo máximo: Ás e Rei de espadas na mão, com Dama, Valete e Dez de espadas na mesa. Royal flush: o jogo imbatível, cinco cartas do mesmo naipe em sequência até o Ás.

E se ninguém pagar a sua aposta no caminho? O pote é seu na hora, sem mostrar nada. É o blefe: vencer sem ter o melhor jogo.

No choudáun, abrem-se as cartas e compara-se o melhor jogo de cinco. Empatou? O pote divide no meio. Dois bláindes, duas suas, cinco da mesa: melhor jogo leva tudo. No próximo episódio: posição, a maior vantagem do poker."
