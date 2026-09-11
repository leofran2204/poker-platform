# EP01 — Poker do zero: seu objetivo em 1 minuto

Público: quem nunca jogou. Uma ideia só: com 7 cartas, forme o melhor jogo de 5.

## Arco narrativo
1. Gancho: "52 cartas, 2 suas, 5 da mesa. Quem faz o melhor jogo de 5 leva as fichas."
2. Baralho + 2 cartas fechadas voando para VOCÊ.
3. Mesa abre 5 comunitárias (flop 3, turn 1, river 1).
4. Aha: das 7, destacam-se as 5 do melhor jogo; resto apaga. Fichas vão ao vencedor.
5. Fecho: "2 suas + 5 da mesa = melhor jogo de 5. Próximo episódio: posição."

## Cenas (script.py) — total ~60s
- S1_Titulo (8s): "POKER DO ZERO" + "episódio 1: seu objetivo" + monte do baralho.
- S2_SuasCartas (12s): 2 cartas fechadas voam para selo VOCÊ; viram A♠ K♠.
- S3_Mesa (16s): 5 comunitárias abrem flop/turn/river: Q♠ J♠ T♥ 3♦ 2♣.
- S4_MelhorJogo (15s): A♠ K♠ Q♠ J♠ T♥ acendem (royal!), resto apaga; pote vai ao vencedor.
- S5_Fecho (9s): "2 + 5 = melhor jogo de 5 leva tudo."

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6, vermelho #B33A3A, azul-preto #1A2A5A.
- Fonte monospace (DejaVu Sans Mono), sem LaTeX (sem MathTex).
- Narração PT-BR separada (narracao.txt) muxada via ffmpeg.

## Narração (~125 palavras, ~55s)
"O poker Texas Hold'em usa um baralho de 52 cartas. Cada jogador recebe duas
cartas fechadas, só suas. Depois, a mesa abre cinco cartas comunitárias, que
valem para todo mundo. Com as sete cartas, cada jogador forma o melhor jogo de
cinco cartas. Par, dois pares, trinca, sequência, flush... quem tem o jogo mais
forte leva as fichas do pote. Olha este exemplo: com Ás e Rei de espadas na mão
e Dama, Valete e Dez na mesa, forma-se a sequência real, o jogo mais forte que
existe. Entendeu a base? Duas suas, cinco da mesa, melhor jogo de cinco leva
tudo. No próximo episódio: a posição, a maior vantagem do poker."
