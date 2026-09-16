# EP11 — De onde vêm as cartas: mil anos de baralho (versão aprofundada ~90s)

Público: iniciante (M0). Uma ideia só: o baralho tem mil anos e é uma máquina de informação incompleta.

## Arco narrativo
1. Gancho: mil anos antes do all-in.
2. China Tang séc. IX → Rota da Seda.
3. Pérsia: gandjifa (96/8) e aznás (25/5).
4. Mamelucos (Topkapi, 52 cartas) → Europa 1370.
5. França: impressão barata + reis com nome.
6. 13×4=52 cruzou o oceano.
7. Fecho: informação incompleta; próximo: Mississippi.

## Cenas (script.py v3 visual) — waits casados com áudio por cena (1.0x)
- S1_Titulo (~9s): MIL ANOS DE BARALHO, cartaz video 1 + monte de versos.
- S2_China (~13s): 3 tiras com símbolos (moeda furada, cordão, 3 moedas=miríades) + Rota da Seda.
- S3_Persia (~14.5s): 8 naipes gandjifa (coroa, prata, sabre, servo, ouro, harpa, documento, tecido) + 5 aznás nas cores das séries (sol, coroa, flor, espadas, nota).
- S4_Mamelucos (~15.5s): 4 naipes de Topkapi (moedas, tacos, taças, cimitarras) + 52→Europa.
- S5_Franca (~14s): 4 reis + nomes na ordem do leque (Davi/Carlos Magno/Cesar/Alexandre).
- S6_52 (~10s): MiniTable + board de 5 distribuído.
- S7_Fecho (~17s): showdown royal com glow nas 5 + pote. Total ~89s.
- Biblioteca: `ZeroTiltCurso/shared.py` (SuitTile, AsNasTile + pictogramas Coin/String/Crown/Sabre/Servant/Harp/Document/Bolster/Cup/Polo/Sun/Flower/Note/Swords).

## Identidade
- Feltro #0A2E1A, dourado #C9A227, creme #F4F0E6. Monospace, sem LaTeX.
- Narração PT-BR por cena (segN.txt → segN.mp3) muxada via ffmpeg.

## Narração (218 palavras, ~89s em 1.0x, áudio por cena)
Voz Edge-TTS pt-BR-AntonioNeural. Texto integral (7 atos = seg1..seg7):

"Mil anos antes do seu primeiro olin, a humanidade já jogava cartas. De onde veio o baralho que está na sua mão?

Na China do século nove, na época da dinastia Tang, a corte se divertia com tiras de papel marcadas com símbolos. Dali, pela Rota da Seda, a ideia viajou para o oeste.

Na Pérsia, nasceu o gandjifa: noventa e seis cartas em oito naipes, com rei e ministro. E o aznás, com vinte e cinco cartas em cinco naipes: xá, dama, soldado, bailarino e ás.

No Egito dos mamelucos, o baralho já tinha cinquenta e duas cartas em quatro naipes — o exemplar de Topcapi sobrevive até hoje. Em mil trezentos e setenta, as cartas chegam à Europa: Catalunha, Florença, Paris.

E foram os franceses que baratearam tudo: naipes simples de imprimir, em vermelho e preto. E cada rei ganhou nome: Davi de espadas, Alexandre de paus, César de ouros, Carlos Magno de copas.

Treze valores vezes quatro naipes. Cinquenta e duas cartas que cruzaram o oceano e viraram centenas de jogos — incluindo o poker.

E o detalhe que importa: o baralho é uma máquina de informação incompleta. Você vê as suas cartas, não as dos outros. Apostar é precificar o que você não vê. No próximo episódio: o Mississippi e o nascimento do blefe."
