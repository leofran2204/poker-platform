"""Gera os roteiros do Brazilian Pineapple (src/data/*.json) a partir de uma mão-modelo.

A mão-modelo completa está no vídeo "Regras na prática" (e em trechos no teaser): quatro
jogadores, blinds R$ 0,50/R$ 1, sem ante. No flop, no turn e no river, cada jogador troca
ou mantém na sua vez; no river, depois da troca, descarta (a carta descartada fica aberta por
2 s para todos, regra pineapple_draw_v4); só então age. Relógio: 15 s + 5 s de tempo extra por jogada. Cada cena "hand" recebe
o estado da mesa naquele ponto (stacks, apostas, pote, cartas) calculado aqui, para
que os valores na tela nunca saiam da conta. O motor confere tudo em
Motor-Rust/tests/pineapple_draw_film_examples.rs.

Uso: python tools/build_scripts.py
"""
from __future__ import annotations

import copy
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import render  # noqa: E402  (voz e padrão de narração da plataforma)

DATA = Path(__file__).resolve().parents[1] / "src" / "data"
VOICE, RATE, FPS = render.NARRATION_VOICE, "-3%", 30
RULE = "pineapple_draw_v4"

NAMES = ["Ana · você", "Beto", "Caio", "Duda"]
START_CARDS = [["Ah", "Kd", "7c"], ["9s", "9d", "4h"], ["Qh", "Jh", "5s"], ["Tc", "8c", "2d"]]
MONEY = {"post", "bet", "call", "raise", "allin"}


class Hand:
    """Estado da mesa entre cenas: aplica os eventos de cada cena e guarda o resultado."""

    def __init__(self, stacks=(10000, 10000, 10000, 10000), cards=START_CARDS, dealer=3, dealt=False):
        self.stacks = list(stacks)
        self.bets = [0, 0, 0, 0]
        self.cards = [list(c) for c in cards]
        self.folded = [False] * 4
        self.board: list[str] = []
        self.pot = 0
        self.dealer = dealer
        self.dealt = dealt
        self.open: list[int] = []
        self.drawn: list[str | None] = [None] * 4  # selo da troca na street atual

    def scene(self, scene_id, chapter, title, voice, events, banner=None, note=None, pad=None):
        players = [
            {"name": NAMES[i], "stack": self.stacks[i], **({"bet": self.bets[i]} if self.bets[i] else {}),
             "cards": list(self.cards[i]) if (self.dealt or any(e["type"] == "deal" for e in events)) else [],
             **({"folded": True} if self.folded[i] else {}),
             **({"drawn": self.drawn[i]} if self.drawn[i] else {})}
            for i in range(4)
        ]
        visual = {"mode": "hand", "dealer": self.dealer, "hero": 0, "players": players,
                  "board": list(self.board), "pot": self.pot, "events": events}
        if self.open:
            visual["open"] = list(self.open)
        if banner:
            visual["banner"] = banner
        if note:
            visual["note"] = note
        spoken = render_words(voice)
        for event in events:
            cue = event.get("cue")
            assert not cue or render_words(cue) in spoken, f"{scene_id}: deixa '{cue}' fora da narração"
            self.apply(event)
        out = {"id": scene_id, "chapter": chapter, "title": title, "voice": voice, "visual": visual}
        if pad is not None:
            out["padAfter"] = pad
        return out

    def apply(self, event):
        kind, who = event["type"], event.get("player")
        if kind == "deal":
            self.dealt = True
        elif kind in MONEY:
            paid = event["to"] - self.bets[who]
            assert 0 <= paid <= self.stacks[who], event
            self.stacks[who] -= paid
            self.bets[who] = event["to"]
        elif kind == "ante":
            self.stacks[who] -= event["to"]
            self.pot += event["to"]
        elif kind == "fold":
            self.folded[who] = True
        elif kind == "collect":
            self.pot += sum(self.bets)
            self.bets = [0, 0, 0, 0]
        elif kind == "board":
            self.board += event["cards"]
            self.drawn = [None] * 4
        elif kind == "keep":
            self.drawn[who] = "Manteve"
        elif kind == "swap":
            self.cards[who][event["position"]] = event["receive"]
            self.drawn[who] = "Trocou 1"
        elif kind == "discard":
            del self.cards[who][event["position"]]
            self.drawn[who] = "Descartou"
        elif kind == "reveal":
            self.open += [p for p in event["players"] if p not in self.open]
        elif kind == "award":
            winners = event.get("players", [who])
            shares = event.get("amounts", [self.pot // len(winners)] * len(winners))
            for player, share in zip(winners, shares):
                self.stacks[player] += share
            self.pot -= sum(shares)


def render_words(text):
    """Palavras normalizadas, separadas por espaço, para conferir as deixas."""
    import re
    import unicodedata
    plain = unicodedata.normalize("NFD", text.lower())
    plain = "".join(ch for ch in plain if unicodedata.category(ch) != "Mn")
    return " " + " ".join(re.findall(r"[a-z0-9]+", plain)) + " "


def ev(kind, cue=None, **fields):
    return {"type": kind, **({"cue": cue} if cue else {}), **fields}


def intro(scene_id, chapter, title, voice, kicker, lines, badge):
    return {"id": scene_id, "chapter": chapter, "title": title, "voice": voice,
            "visual": {"mode": "intro", "kicker": kicker, "lines": lines, "badge": badge}}


def outro(scene_id, chapter, voice, lines):
    return {"id": scene_id, "chapter": chapter, "title": "Comece pelo treino", "voice": voice,
            "visual": {"mode": "outro", "lines": lines}, "padAfter": 2.0}


def lines(scene_id, chapter, title, voice, items, cues=None, note=None):
    visual = {"mode": "steps", "lines": items}
    if cues:
        visual["cues"] = cues
    if note:
        visual["note"] = note
    return {"id": scene_id, "chapter": chapter, "title": title, "voice": voice, "visual": visual}


def write(name, title, scenes, lesson=None):
    script = {"id": name, "title": title, "voice": VOICE, "rate": RATE, "fps": FPS, "bettingRuleVersion": RULE,
              **({"lesson": lesson} if lesson else {}), "scenes": scenes}
    ids = [scene["id"] for scene in scenes]
    assert len(ids) == len(set(ids)), "ids repetidos"
    heavy = [(scene["id"], s) for scene in scenes for s in render.heavy_english_sentences(scene["voice"])]
    assert not heavy, f"{name}: frases com muito inglês, reescreva com mais português: {heavy}"
    (DATA / f"{name}.json").write_text(json.dumps(script, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")
    words = sum(len(scene["voice"].split()) for scene in scenes)
    print(f"{name}: {len(scenes)} cenas, {words} palavras (~{words / 2.45 / 60:.1f} min)")


# ─────────────── A mão-modelo, cena a cena (prefixo de id e capítulos variam) ───────────────
def model_hand(prefix, chapters, short=False):
    """Devolve as cenas da mão completa. `chapters` nomeia cada etapa."""
    h = Hand()
    c = chapters
    scenes = [
        h.scene(f"{prefix}deal", c["deal"], "Três cartas para cada jogador",
                "Agora, uma mão completa, passo a passo. Neste exemplo, a mesa tem quatro jogadores, "
                "mas o Brazilian Pineapple aceita até seis. Duda está no botão, que marca a posição do dealer. "
                "Cada jogador recebe três cartas fechadas, que só ele vê. "
                "Ana, a primeira à esquerda do botão, paga o small blind de cinquenta centavos. "
                "Beto, o seguinte, paga o big blind de um real. Não há ante: são só essas duas apostas obrigatórias.",
                [ev("deal", "recebe três cartas"),
                 ev("post", "paga o small blind", player=0, to=50, label="Small blind"),
                 ev("post", "paga o big blind", player=1, to=100, label="Big blind")],
                banner="Pré-flop"),
        h.scene(f"{prefix}pre", c["pre"], "Primeira rodada de apostas",
                "Começa a primeira rodada de apostas, ainda sem troca. Quem fala primeiro é Caio, o jogador à esquerda do big blind. "
                "Repare na barra verde sob a placa dele: é o relógio. Cada jogada tem quinze segundos e, se eles acabarem, "
                "ainda há cinco segundos de tempo extra. No canto da tela ficam os botões da vez: como há uma aposta a pagar, "
                "as opções de Caio são Fold, Call ou Raise. "
                "Para continuar na mão, ele precisa pelo menos pagar um real, o valor do big blind. "
                "Caio faz mais: dá raise, ou seja, aumenta a aposta. O máximo é o valor a pagar mais o pote antes do pagamento: "
                "um real mais um real e cinquenta, ou seja, dois reais e cinquenta. Caio vai ao máximo. "
                "Duda paga os dois reais e cinquenta. "
                "Agora Ana, que está no small blind, dá reraise, que é aumentar de novo depois de um aumento. "
                "Pela mesma conta, dois reais a pagar mais seis e cinquenta de pote, ela pode ir até nove reais, e vai. "
                "Como já tinha cinquenta centavos na mesa, ela completa com oito e cinquenta. "
                "Para seguir na mão, cada um precisa igualar os nove reais. "
                "Beto paga mais oito, porque o big blind de um real já conta. "
                "Caio paga mais seis e cinquenta. Duda também paga mais seis e cinquenta. "
                "Todos igualaram a aposta, então a rodada termina e as fichas vão para o pote: trinta e seis reais.",
                [ev("raise", "Caio vai ao máximo", player=2, to=250, max=250, label="Raise"),
                 ev("call", "Duda paga os dois", player=3, to=250),
                 ev("raise", "ir até nove reais", player=0, to=900, max=900, label="Reraise"),
                 ev("call", "Beto paga mais oito", player=1, to=900),
                 ev("call", "Caio paga mais seis", player=2, to=900),
                 ev("call", "Duda também paga", player=3, to=900),
                 ev("collect", "as fichas vão para o pote")],
                banner="Pré-flop · apostas"),
        h.scene(f"{prefix}flop", c["flop"], "Flop: na sua vez, troca e depois aposta",
                "O flop abre três cartas comunitárias: ás, dama e seis. "
                "Daqui em diante, quem fala primeiro é o jogador mais à esquerda do botão que ainda está na mão: aqui, Ana. "
                "No Brazilian Pineapple, a troca acontece na vez de cada jogador. "
                "Primeiro, ele decide se troca uma carta ou mantém as três; só depois aposta, paga, passa ou desiste. "
                "Por isso, os botões de Ana começam por Trocar e Manter; depois da troca, aparecem Check e Aposta. "
                "Ana troca o sete. A carta trocada sai do jogo, e outra vem do baralho: um ás. Agora Ana tem trinca de ases. "
                "Com a troca feita, ela aposta doze reais. Pelo limite do pote, poderia apostar até trinta e seis.",
                [ev("board", "abre três cartas", cards=["As", "Qd", "6h"]),
                 ev("swap", "Ana troca o sete", player=0, position=2, receive="Ac"),
                 ev("bet", "ela aposta doze", player=0, to=1200, max=3600)],
                banner="Flop · troca e aposta"),
        h.scene(f"{prefix}flopcall", c["flopcall"], "Cada um troca na sua vez",
                "Agora é a vez de Beto, com doze reais a pagar. Ele também troca uma carta e depois paga os doze. "
                "Caio troca uma e paga. "
                "Duda não acertou nada e desiste sem trocar: quem vai desistir não precisa decidir a troca. "
                "Todos veem quantas cartas cada um trocou, mas não quais. "
                "As apostas vão para o pote, que chega a setenta e dois reais.",
                [ev("swap", "Ele também troca", player=1, position=2, receive="3c"),
                 ev("call", "depois paga os doze", player=1, to=1200),
                 ev("swap", "Caio troca uma", player=2, position=2, receive="Th"),
                 ev("call", "uma e paga", player=2, to=1200, delay=0.4),
                 ev("fold", "desiste sem trocar", player=3),
                 ev("collect", "As apostas vão para o pote")],
                banner="Flop · troca e aposta"),
        h.scene(f"{prefix}turn", c["turn"], "Turn: manter também é uma escolha",
                "O turn abre a quarta carta, o rei de copas. Nova rodada, de novo a partir de Ana, "
                "e de novo cada um decide a troca na sua vez. Ana mantém as três cartas. "
                "Com trinca, ela prefere dar check, ou seja, passar a vez sem apostar, para deixar os outros apostarem. "
                "O check só é possível quando ninguém apostou ainda na rodada. Beto também mantém e dá check. "
                "Caio mantém e aposta trinta reais. Com uma aposta na mesa, a vez volta para Ana. "
                "Ela já decidiu a troca nesta rodada, então agora só escolhe a aposta: paga os trinta. "
                "Beto desiste. O pote chega a cento e trinta e dois reais.",
                [ev("board", "a quarta carta", cards=["Kh"]),
                 ev("keep", "Ana mantém", player=0),
                 ev("check", "prefere dar check", player=0),
                 ev("keep", "Beto também mantém", player=1),
                 ev("check", "mantém e dá check", player=1, delay=0.5),
                 ev("keep", "Caio mantém", player=2),
                 ev("bet", "aposta trinta", player=2, to=3000),
                 ev("call", "paga os trinta", player=0, to=3000),
                 ev("fold", "Beto desiste", player=1),
                 ev("collect", "O pote chega")],
                banner="Turn · troca e aposta"),
        h.scene(f"{prefix}river", c["river"], "River: troca, descarte e aposta",
                "O river abre a quinta carta, o seis de espadas. No river, a vez de cada jogador tem três passos: "
                "primeiro troca uma carta ou mantém as três, depois descarta uma e fica com duas, e só então age. "
                "Ana troca o rei. Ela ia descartar o rei de qualquer jeito, então trocar antes não custa nada: "
                "se a carta nova não ajudar, é ela que vai embora. Vem um dois de paus, que não ajuda. "
                "Ana descarta o dois e fica com os dois ases. "
                "O descarte é obrigatório para quem segue na mão, e a carta descartada fica aberta por dois segundos, "
                "para todos verem, antes de ir para a pilha de descarte. A carta trocada, não: essa ninguém vê. "
                "Depois, Ana aposta trinta e cinco reais. "
                "Ela poderia apostar até quarenta e nove, tudo o que ainda tem. O pote, de cento e trinta e dois, permitiria mais, "
                "mas ninguém aposta mais fichas do que tem na mesa.",
                [ev("board", "quinta carta", cards=["6s"]),
                 ev("swap", "Ana troca o rei", player=0, position=1, receive="2c"),
                 ev("discard", "Ana descarta o dois", player=0, position=1),
                 ev("bet", "Ana aposta trinta e cinco", player=0, to=3500, max=4900)],
                banner="River · troca, descarte e aposta"),
        h.scene(f"{prefix}last", c["last"], "Caio troca, descarta e paga",
                "Agora é a vez de Caio, com trinta e cinco reais a pagar. Ele segue a mesma lógica: troca a dama, "
                "que também ia descartar, e recebe um quatro de espadas. Descarta o quatro, que todos veem aberto, e fica com valete e dez, "
                "que fecham a sequência, e paga. "
                "As apostas se igualaram, e as fichas vão para o pote: duzentos e dois reais.",
                [ev("swap", "troca a dama", player=2, position=0, receive="4s"),
                 ev("discard", "Descarta o quatro", player=2, position=0),
                 ev("call", "fecham a sequência", player=2, to=3500, delay=0.3),
                 ev("collect", "as fichas vão para o pote")],
                banner="River · troca, descarte e aposta"),
        h.scene(f"{prefix}show", c["show"], "Showdown",
                "Agora é o showdown, a hora de mostrar as cartas. Caio mostra valete e dez: com o ás, o rei e a dama da mesa, "
                "forma uma sequência até o ás. Ana mostra os dois ases: com o ás e os dois seis da mesa, forma um full house, "
                "três ases e dois seis. Repare que cada um usou as duas cartas da mão. "
                "Full house vence sequência, e o pote de duzentos e dois reais vai para Ana.",
                [ev("reveal", "Caio mostra", players=[2]),
                 ev("highlight", "Ana mostra", player=0, hole=[0, 1], boardPick=[0, 2, 4], text="Full house de ases com seis"),
                 ev("award", "vai para Ana", player=0)],
                banner="Showdown", pad=1.6),
    ]
    assert h.stacks == [21600, 7900, 1400, 9100] and h.pot == 0, (h.stacks, h.pot)
    return scenes


# ───────────────────────────────────── Trailer ─────────────────────────────────────
SLOGAN = "Três cartas. Três trocas. Um descarte."


def teaser():
    """Teaser de ~60 s (home e redes): trechos da mão-modelo, com o estado da mesa de cada momento."""
    chapter = "Brazilian Pineapple"
    scenes = [intro("t01", chapter, "Brazilian Pineapple",
                    "Três cartas na mão, três chances de trocar e um descarte no final. "
                    "Este é o Brazilian Pineapple, o pôquer criado no Brasil, na Zero Tilt.",
                    "Zero Tilt Poker · modalidade criada no Brasil", [SLOGAN],
                    "Exclusivo ZT · primeiro no Play Money")]
    h = custom(START_CARDS, [], 3600, stacks=(9100, 9100, 9100, 9100))
    scenes.append(h.scene("t02", chapter, "Na sua vez, troca e aposta",
                          "No flop, na sua vez, você troca uma carta ou mantém as três, e só então aposta. "
                          "Ana troca o sete e recebe um ás: agora tem trinca de ases. E aposta doze reais.",
                          [ev("board", "No flop", cards=["As", "Qd", "6h"]),
                           ev("swap", "Ana troca o sete", player=0, position=2, receive="Ac"),
                           ev("bet", "aposta doze reais", player=0, to=1200)],
                          banner="Flop · troca e aposta"))
    river_cards = [["Ah", "Kd", "Ac"], ["9s", "9d", "3c"], ["Qh", "Jh", "Th"], ["Tc", "8c", "2d"]]
    h = custom(river_cards, ["As", "Qd", "6h", "Kh"], 13200, stacks=(4900, 7900, 4900, 9100), folded=(1, 3))
    scenes.append(h.scene("t03", chapter, "No river, troca, descarte e aposta",
                          "No river, a vez tem três passos: troca, descarte e aposta. "
                          "Ana troca o rei, descarta a carta nova, que todos veem aberta, e aposta trinta e cinco reais. "
                          "Caio troca uma carta, descarta outra e paga.",
                          [ev("board", "No river", cards=["6s"]),
                           ev("swap", "Ana troca o rei", player=0, position=1, receive="2c"),
                           ev("discard", "descarta a carta nova", player=0, position=1),
                           ev("bet", "aposta trinta e cinco", player=0, to=3500),
                           ev("swap", "Caio troca uma carta", player=2, position=0, receive="4s"),
                           ev("discard", "descarta outra", player=2, position=0),
                           ev("call", "e paga", player=2, to=3500, delay=0.2),
                           ev("collect", delay=0.8)],
                          banner="River · troca, descarte e aposta"))
    show_cards = [["Ah", "Ac"], ["9s", "9d", "3c"], ["Jh", "Th"], ["Tc", "8c", "2d"]]
    h = custom(show_cards, ["As", "Qd", "6h", "Kh", "6s"], 20200, stacks=(1400, 7900, 1400, 9100), folded=(1, 3))
    scenes.append(h.scene("t04", chapter, "Showdown",
                          "No showdown, valem as duas cartas que ficaram na mão. "
                          "Caio tem sequência, mas Ana tem full house de ases, e o pote de duzentos e dois reais é dela.",
                          [ev("reveal", "valem as duas cartas", players=[2]),
                           ev("highlight", "Ana tem full house", player=0, hole=[0, 1], boardPick=[0, 2, 4], text="Full house de ases com seis"),
                           ev("award", "o pote de duzentos", player=0)],
                          banner="Showdown", pad=1.0))
    scenes.append(outro("t99", chapter,
                        "Aprenda as regras na Zero Tilt Academy e pratique com Play Money. Poker envolve risco. "
                        "Só para maiores de dezoito anos.",
                        ["Regras na prática na Academy", "Pratique com Play Money", "18+ • Jogue com responsabilidade"]))
    write("pineapple-draw-trailer", "Brazilian Pineapple — teaser", scenes)


def custom(cards, board, pot, stacks=(9700, 9700, 9700, 9900), folded=(), dealer=3):
    h = Hand(stacks=stacks, cards=cards, dealer=dealer, dealt=True)
    h.board, h.pot = list(board), pot
    for player in folded:
        h.folded[player] = True
    return h


FLUSH_CARDS = [["Ah", "7s", "7d"], ["Kc", "Qs", "4d"], ["Jd", "Td", "2c"], ["8s", "5c", "3d"]]
FLUSH_BOARD = ["Kh", "9h", "6h", "3h", "Jc"]


def flush_scene(scene_id, chapter):
    h = custom(FLUSH_CARDS, FLUSH_BOARD, 1400, folded=(3,))
    return h.scene(scene_id, chapter, "Uma carta da mão já basta",
                   "Com duas cartas na mão, você forma a melhor mão de cinco como no Texas Hold’em: pode usar as duas, "
                   "uma só ou nenhuma. Aqui há quatro copas na mesa, e Ana tem o ás de copas: o flush mais alto possível. "
                   "Não há o que melhorar, então ela mantém as três e descarta um sete. "
                   "Uma carta da mão já basta para o flush.",
                   [ev("keep", "ela mantém as três", player=0),
                    ev("discard", "descarta um sete", player=0, position=1),
                    ev("highlight", "Uma carta da mão", player=0, hole=[0], boardPick=[0, 1, 2, 3], text="Flush de copas com o ás")],
                   banner="River · descarte")


def quiz_pair(prefix, chapter, question, options, correct, voice_q, voice_a, explanation, hole, board):
    opts = [{"key": k, "text": t, "cue": f"{k} {c}"} for k, t, c in options]
    base = {"question": question, "options": opts, "correct": correct, "hole": hole, "board": board}
    return [
        {"id": f"{prefix}-quiz", "chapter": chapter, "title": "Sua vez", "voice": voice_q, "visual": {"mode": "quiz", **base}, "padAfter": 3.0},
        {"id": f"{prefix}-answer", "chapter": chapter, "title": "Resposta", "voice": voice_a,
         "visual": {"mode": "answer", **base, "cue": "Resposta", "explanation": explanation}},
    ]


LAB_VOICE = ("Agora leve a aula para a mesa de treino da Academy. Comece pelo treino guiado e depois repita no modo desafio, "
             "respondendo à pergunta antes de cada ação. Use o replay para comparar uma alternativa na mesma distribuição.")
LAB_LINES = ["Treino guiado, depois modo desafio", "Responda antes de agir", "Use o replay para comparar"]
OUT_VOICE = ("Registre sua justificativa no caderno: acertar a leitura e respeitar as hipóteses importa mais do que ganhar a mão. "
             "O treino usa fichas sem valor, e este curso não promete lucro.")
OUT_LINES = ["Registre a justificativa no caderno", "Fichas sem valor no treino", "Este curso não promete lucro"]
ORDER_LINES = ["Três cartas · apostas, sem troca", "Flop · na sua vez, troca e aposta", "Turn · na sua vez, troca e aposta",
               "River · na sua vez, troca, descarte e aposta", "Showdown"]
ORDER_VOICE = ("A ordem da mão é sempre a mesma. Três cartas e a primeira rodada de apostas, ainda sem troca. "
               "No flop, cada jogador, na sua vez, troca ou mantém e depois aposta. No turn, de novo: troca na sua vez e depois aposta. "
               "No river, cada um, na sua vez, troca ou mantém, descarta uma carta e só então aposta. "
               "Por fim, as cartas são mostradas no showdown.")
ORDER_CUES = ["Três cartas", "No flop", "No turn", "No river", "Por fim"]


# ──────────────────────────────── Filme de regras ────────────────────────────────
def rules():
    """Regras na prática (~10 min): a mão-modelo comentada ensina cada regra; depois, blocos curtos
    para troca, tempo, formação da mão, apostas, all-in e Loss Deflator (decisão do dono em 21/11/2026)."""
    scenes = [intro("r01", "O jogo", "Brazilian Pineapple",
                    "Esta é a Zero Tilt Academy. Neste vídeo, você aprende as regras do Brazilian Pineapple "
                    "acompanhando uma mão inteira, comentada ação por ação. É uma modalidade de pôquer criada no Brasil, "
                    "com três cartas na mão, três trocas e um descarte.",
                    "Zero Tilt Academy · regras na prática", [SLOGAN],
                    "Valores ilustrativos · comece com Play Money")]
    h = Hand(dealt=True)
    scenes.append(h.scene("r02", "O jogo", "A mesa e as cinco cartas comunitárias",
                          "Esta é a mesa do Brazilian Pineapple, com até seis jogadores. No centro ficam cinco cartas comunitárias, "
                          "que valem para todos: as três do flop, a do turn e a do river. Cada jogador recebe três cartas só suas "
                          "e, no fim, forma a melhor mão de cinco cartas.",
                          [ev("board", "as três do flop", cards=["As", "Qd", "6h"]),
                           ev("board", "a do turn", cards=["Kh"]),
                           ev("board", "a do river", cards=["6s"])],
                          banner="As cinco comunitárias"))
    scenes.append(lines("r03", "O jogo", "A ordem da mão", ORDER_VOICE, ORDER_LINES, ORDER_CUES))
    scenes += model_hand("r-", {"deal": "Pré-flop", "pre": "Pré-flop", "flop": "Flop: troca e aposta na vez", "flopcall": "Flop: troca e aposta na vez",
                                "turn": "Turn: troca e aposta na vez", "river": "River: troca, descarte e aposta", "last": "River: troca, descarte e aposta",
                                "show": "Showdown"})
    scenes.append(
        lines("r20", "A troca em detalhe", "Trocar ou manter, na sua vez",
              "Resumindo a troca. Ela acontece na sua vez, no flop, no turn e no river: primeiro você troca uma carta "
              "ou mantém as três, e depois age. No river, entre a troca e a aposta, vem o descarte, e a carta descartada aparece aberta para todos. "
              "A troca vale só na primeira vez que você age na rodada; se a vez voltar por causa de um aumento, "
              "você só decide a aposta. Quem vai desistir pode desistir sem trocar. "
              "Todos veem quantas cartas cada jogador trocou; ninguém vê quais.",
              ["Flop, turn e river: troca ou mantém, depois age", "River: troca, descarte aberto e só então age",
               "Só na primeira vez de cada rodada", "Dá para desistir sem trocar", "Troca: todos veem quantas, ninguém vê quais"],
              ["acontece na sua vez", "No river, entre", "A troca vale só", "pode desistir sem trocar", "Todos veem"]))
    clock_cards = [["Kc", "Qd", "5h"], ["9h", "9c", "2s"], ["Ad", "7d", "4c"], ["Th", "6s", "6c"]]
    h = custom(clock_cards, ["Jc", "8d", "3s"], 400, stacks=(9900, 9900, 9900, 9900))
    scenes.append(h.scene("r21", "Tempo para agir", "O relógio da vez",
                          "Agora, o relógio. Cada jogada tem quinze segundos, marcados pela barra sob a placa. "
                          "Quando eles acabam, a barra fica vermelha, aparece o aviso de tempo extra e toca um som curto: "
                          "são mais cinco segundos. O relógio recomeça a cada jogada, e a troca e a aposta usam o mesmo tempo. "
                          "Quando chega a sua vez, também toca um som, e o título da aba do navegador pisca com o aviso. "
                          "Ana mantém e passa a vez. Agora é a vez de Beto, e ele demora. "
                          "O tempo normal acaba, depois o tempo extra, e o sistema age por ele: "
                          "sem aposta a pagar, mantém as cartas e dá check. Com aposta a pagar, seria fold, mesmo antes da troca. "
                          "No river, o sistema também faz o descarte, guardando as duas melhores cartas, e a descartada aparece aberta.",
                          [ev("keep", "Ana mantém", player=0),
                           ev("check", "e passa a vez", player=0),
                           ev("turn", "ele demora", player=1, elapsed=8, timeout=True),
                           ev("keep", "o sistema age por ele", player=1, label="Tempo esgotado"),
                           ev("check", "mantém as cartas e dá check", player=1, label="Check automático")],
                          banner="Flop · tempo esgotado", note="15 s por jogada + 5 s de tempo extra"))
    scenes += [
        flush_scene("r22", "Formação da mão"),
        lines("r23", "Formação da mão", "Ranking clássico",
              "O ranking é o clássico. Sequência vence trinca. Já o full house fica acima de qualquer flush, mesmo do mais alto. E ás, dois, três, quatro e cinco formam a sequência mais baixa.",
              ["Sequência vence trinca", "Full house vence flush", "A-2-3-4-5 é a sequência baixa"],
              ["Sequência vence", "o full house", "sequência mais baixa"]),
        lines("r24", "Apostas", "Blinds e limite do pote",
              "As apostas seguem poucas regras. Antes das cartas, há só duas apostas obrigatórias, o small blind e o big blind. "
              "Nesta modalidade, ninguém paga ante antes das cartas. "
              "E, em todas as rodadas, o aumento máximo é o pote formado antes do call.",
              ["Small blind e big blind", "Sem ante", "Aumento máximo: o pote antes do call"],
              ["o small blind", "ninguém paga ante", "aumento máximo"]),
    ]
    h = custom(START_CARDS, ["As", "Qd", "6h"], 400, stacks=(9900, 9900, 9900, 9900))
    scenes.append(h.scene("r25", "Apostas", "O limite do pote, na prática",
                          "Veja o limite na prática. O pote tem quatro reais, então Ana pode apostar até quatro reais. "
                          "Ana mantém as cartas e aposta quatro. Agora o pote tem oito, e pagar custa quatro. "
                          "Beto pode aumentar para até doze reais: quatro para pagar mais os oito do pote. "
                          "Beto mantém as cartas e aumenta. Caio desiste. Duda desiste. Ana paga.",
                          [ev("keep", "Ana mantém as cartas", player=0),
                           ev("bet", "e aposta quatro", player=0, to=400, label="Aposta máxima"),
                           ev("keep", "Beto mantém as cartas", player=1),
                           ev("raise", "cartas e aumenta", player=1, to=1200, label="Raise máximo"),
                           ev("fold", "Caio desiste", player=2),
                           ev("fold", "Duda desiste", player=3),
                           ev("call", "Ana paga", player=0, to=1500),
                           ev("collect", delay=0.9)],
                          banner="Flop · apostas"))
    scenes += [
        lines("r26", "All-in", "Quando alguém aposta tudo",
              "Quem está all-in não age mais, então não troca cartas. No river, fica automaticamente com as duas melhores, "
              "e a carta descartada aparece aberta, como a de todos. "
              "E, quando ninguém mais pode apostar, não há troca nem escolha de descarte: a mesa é completada direto.",
              ["All-in não troca", "No river, ficam as duas melhores; o descarte é aberto", "Sem apostas possíveis: a mesa é completada"],
              ["não troca", "No river", "ninguém mais pode apostar"]),
        lines("r27", "Loss Deflator", "Quando o favorito perde o all-in",
              "O Loss Deflator vale também aqui. Se você perde um all-in em que era favorito, com pelo menos cinquenta e seis por cento de chance, "
              "uma parte do pote volta para você: de sete a trinta e cinco por cento, conforme a sua vantagem. "
              "A conta usa as cartas que cada um tinha no instante do all-in.",
              ["Perdeu o all-in como favorito? Parte do pote volta", "De 7% a 35% do pote, conforme a vantagem", "Conta com as cartas do instante do all-in"],
              ["perde um all-in", "de sete a trinta e cinco", "A conta usa"]),
        lines("r28", "Recapitule e pratique", "O jogo em cinco passos",
              ORDER_VOICE.replace("A ordem da mão é sempre a mesma.", "Recapitulando."), ORDER_LINES, ORDER_CUES),
        outro("r99", "Recapitule e pratique",
              "Agora, pratique com Play Money e estude as três aulas do Brazilian Pineapple na Zero Tilt Academy. Poker envolve risco. "
              "Só para maiores de dezoito anos.",
              ["Três aulas na Academy", "Pratique com Play Money", "18+ • Jogue com responsabilidade"]),
    ]
    write("pineapple-draw-rules", "Brazilian Pineapple — Regras na prática", scenes)


# ──────────────────────────────────── Aulas ────────────────────────────────────
def lesson_m5l3():
    """Aula 1: as decisões da vez (o que trocar, quando manter, o que descartar e o que a troca
    dos outros revela), com mãos novas; a mão-modelo fica no vídeo Regras na prática (21/11/2026)."""
    kicker = "Zero Tilt Academy · Brazilian Pineapple · Aula 1"
    chapter_swap, chapter_river, chapter_read = "O que trocar", "O que descartar", "O que a troca revela"
    scenes = [intro("m5l3-01", "Abertura", "Três cartas, três trocas e um descarte",
                    "Esta é a Zero Tilt Academy. Na aula um do Brazilian Pineapple, vamos treinar as decisões da sua vez: "
                    "o que trocar, quando manter, o que descartar e o que a troca dos outros revela. "
                    "As regras completas estão no vídeo Regras na prática.",
                    kicker, ["Objetivo: decidir a troca e o descarte com critério"], "Fichas sem valor · este curso não promete lucro"),
              lines("m5l3-02", "A ordem da mão", "A ordem da mão", ORDER_VOICE, ORDER_LINES, ORDER_CUES),
              lines("m5l3-03", chapter_swap, "A pergunta de cada vez",
                    "Antes de trocar, faça sempre a mesma pergunta: quais duas cartas eu levaria ao showdown agora? "
                    "Só duas chegam ao fim, então a terceira é a candidata natural à troca. "
                    "Depois, pergunte se essa terceira carta ainda tem um papel, como um segundo plano de mão. "
                    "Se tiver, pense em manter.",
                    ["Quais duas eu levaria ao showdown agora?", "A terceira é a candidata à troca", "Ela ainda tem um papel? Pense em manter"],
                    ["faça sempre a mesma pergunta", "a terceira é a candidata", "Depois, pergunte"])]

    # Troca: a terceira carta não entra em nenhum plano.
    h = custom([["As", "Qs", "4d"], ["Kh", "Jc", "9d"], ["Tc", "Td", "5c"], ["7h", "6h", "2c"]], ["Qh", "8s", "3s"], 400,
               stacks=(9900, 9900, 9900, 9900))
    scenes.append(h.scene("m5l3-04", chapter_swap, "Troque a carta que fica fora",
                          "No flop, Ana tem ás e dama de espadas e o quatro de ouros. Na mesa, dama de copas, oito e três de espadas. "
                          "A dupla que ela levaria ao showdown é clara: ás e dama de espadas, com par de damas e quatro espadas para o flush. "
                          "O quatro de ouros não entra em nenhum plano. Ana troca o quatro e recebe um sete de paus, que não ajuda. "
                          "Ela não perdeu nada: a dupla boa continua na mão. Depois da troca, Ana aposta três reais.",
                          [ev("highlight", "A dupla que ela levaria", player=0, hole=[0, 1], boardPick=[0, 1, 2], text="Par de damas e quatro espadas"),
                           ev("swap", "Ana troca o quatro", player=0, position=2, receive="7c"),
                           ev("bet", "Ana aposta três reais", player=0, to=300)],
                          banner="Flop · troca e aposta"))

    # Manter: as três cartas trabalham, cada dupla é um plano.
    h = custom([["Ah", "Kh", "Kc"], ["Qd", "Js", "9c"], ["8d", "8c", "4s"], ["Ts", "5d", "3c"]], ["Kd", "7h", "2h"], 400,
               stacks=(9900, 9900, 9900, 9900))
    scenes.append(h.scene("m5l3-05", chapter_swap, "Quando manter: as três trabalham",
                          "Agora, Ana tem ás e rei de copas e o rei de paus. Na mesa, rei de ouros, sete e dois de copas. "
                          "Os dois reis da mão formam trinca. E o ás e o rei de copas, com as duas copas da mesa, "
                          "buscam o flush mais alto. São dois planos, e cada um usa uma dupla diferente: "
                          "trocar qualquer carta mataria um deles. Ana mantém as três e aposta quatro reais. "
                          "No river, o descarte escolhe o plano que vai ao showdown. E todos veem que ela não trocou nenhuma.",
                          [ev("keep", "Ana mantém as três", player=0),
                           ev("bet", "aposta quatro reais", player=0, to=400)],
                          banner="Flop · manter também é escolha"))

    # River: trocar a carta que você ia descartar não custa cartas.
    h = custom([["Ac", "Kd", "5h"], ["Qh", "Jh", "9c"], ["Td", "7s", "4h"], ["9h", "6c", "2c"]], ["As", "Kc", "8d", "3s"], 1600,
               stacks=(9300, 9300, 9900, 9900), folded=(2, 3))
    scenes.append(h.scene("m5l3-06", chapter_river, "Troque o que você ia descartar",
                          "No river, Ana tem ás de paus, rei de ouros e cinco de copas. Na mesa, ás, rei, oito, três e dois. "
                          "Ás e rei formam dois pares, e o cinco vai para o descarte. Mas, no river, a troca vem antes do descarte, "
                          "e trocar a carta que você ia descartar não custa nada: se a carta nova não ajudar, é ela que vai embora. "
                          "Ana troca o cinco. Um ás ou um rei daria trinca: são quatro cartas boas entre as quarenta e quatro que ela não vê, "
                          "cerca de nove por cento. Vem um seis de ouros, que não ajuda. "
                          "Ana descarta o seis e fica com os mesmos dois pares de antes. Depois, ela passa a vez. "
                          "O único custo é a informação: todos veem que ela trocou uma carta e veem o seis que ela descartou.",
                          [ev("board", "três e dois", cards=["2h"]),
                           ev("swap", "Ana troca o cinco", player=0, position=2, receive="6d"),
                           ev("discard", "Ana descarta o seis", player=0, position=2),
                           ev("check", "ela passa a vez", player=0)],
                          banner="River · troca, descarte e aposta"))

    # Leitura: o que manter ou trocar diz sobre a mão dos outros.
    h = custom([["Ad", "9d", "4c"], ["8h", "8c", "Kd"], ["Ts", "7c", "3h"], ["Kc", "5d", "2h"]], ["9s", "8s", "2d"], 400,
               stacks=(9900, 9900, 9900, 9900), dealer=0)
    scenes.append(h.scene("m5l3-07", chapter_read, "Quantas trocou, não quais",
                          "Agora, Ana está no botão e fala por último. Beto mantém as três e aposta quatro reais. "
                          "Caio troca uma e paga. Duda desiste sem trocar. Ana vê quantas cartas cada um trocou, mas não quais. "
                          "Quem mantém no flop costuma gostar das três cartas: uma mão feita, ou dois planos ao mesmo tempo, "
                          "como Ana no exemplo dos reis. Quem troca uma ainda tem uma carta sobrando e talvez procure melhorar. "
                          "É uma pista, não uma prova: manter também pode ser blefe. "
                          "Ana tem par de noves com ás. Contra quem manteve e apostou, ela não aumenta: troca o quatro e só paga.",
                          [ev("keep", "Beto mantém as três", player=1),
                           ev("bet", "aposta quatro reais", player=1, to=400),
                           ev("swap", "Caio troca uma", player=2, position=2, receive="Qd"),
                           ev("call", player=2, to=400, delay=0.5),
                           ev("fold", "Duda desiste", player=3),
                           ev("swap", "troca o quatro", player=0, position=2, receive="Jc"),
                           ev("call", "e só paga", player=0, to=400)],
                          banner="Flop · leitura da troca"))
    scenes.append(lines("m5l3-08", chapter_read, "O que a quantidade conta",
                        "Resumindo a leitura. Manter as três costuma indicar uma mão feita ou dois planos, mas pode ser blefe. "
                        "Trocar uma indica que sobrava uma carta, e a carta trocada nunca aparece. "
                        "No river, a carta descartada aparece aberta: ela mostra o que o jogador não quis "
                        "e deixa de ser uma carta que os outros podem ter. Junte essas pistas às apostas antes de decidir.",
                        ["Manteve: mão feita, dois planos ou blefe", "Trocou uma: sobrava uma carta, que nunca aparece",
                         "Descarte do river: aberto, mostra o que ele não quis", "Junte as pistas às apostas"],
                        ["Manter as três", "Trocar uma", "No river, a carta descartada", "Junte essas pistas"]))
    scenes += quiz_pair("m5l3", "Exercício", "Trocar o 5♣ antes do descarte pode piorar a sua mão final?",
                        [("A", "Sim, a carta nova pode ser pior", "sim, a carta nova"), ("B", "Não: se ela não ajudar, você a descarta", "não, se ela"),
                         ("C", "Só se o oponente vir a carta", "só se o oponente")], "B",
                        "Exercício. No river, você tem dama de espadas, dama de ouros e cinco de paus. "
                        "Na mesa, dama de copas, dez de ouros, oito de espadas, três de paus e dois de copas. "
                        "Você vai descartar o cinco. Trocar o cinco antes do descarte pode piorar a sua mão final? "
                        "A: sim, a carta nova pode ser pior. B: não, se ela não ajudar, você a descarta. "
                        "C: só se o oponente vir a carta. Pause o vídeo e responda.",
                        "Resposta: B. Depois da troca, você ainda descarta uma das três cartas. "
                        "Se a nova não ajudar, ela sai, e você fica com as duas damas, a mesma trinca de antes. "
                        "O que a troca entrega é informação: todos veem que você trocou uma carta, mas ninguém vê qual. "
                        "Só a carta descartada aparece aberta.",
                        "Se a carta nova não ajudar, ela vai para o descarte · o custo é só a informação",
                        ["Qs", "Qd", "5c"], ["Qh", "Td", "8s", "3c", "2h"])
    scenes += [lines("m5l3-30", "Laboratório", "Leve a aula para a mesa", LAB_VOICE, LAB_LINES),
               outro("m5l3-99", "Fechamento", OUT_VOICE, OUT_LINES)]
    write("academy-m5l3", "Aula m5l3 — Três cartas, três trocas e um descarte", scenes, lesson="m5l3")


def lesson_m5l4():
    kicker = "Zero Tilt Academy · Brazilian Pineapple · Aula 2"
    scenes = [intro("m5l4-01", "Abertura", "Combinações, trocas e probabilidade",
                    "Esta é a Zero Tilt Academy. Na aula dois do Brazilian Pineapple, vamos estudar combinações, trocas e probabilidade. "
                    "O objetivo é contar as suas opções sem duplicar e calcular uma chance exata, com as hipóteses declaradas.",
                    kicker, ["Objetivo: contar opções e calcular uma chance exata"], "Fichas sem valor · este curso não promete lucro")]
    h = custom([["Ad", "Kd", "Js"], ["9c", "8c", "4d"], ["7h", "7s", "3c"], ["6d", "5d", "2h"]], ["Ah", "Qc", "2s", "Tc"], 1400, folded=(3,))
    scenes.append(h.scene("m5l4-02", "Três duplas possíveis", "Três cartas, três duplas",
                          "Com três cartas na mão, existem três duplas possíveis para levar ao showdown. Aqui, no turn, Ana tem ás, rei e valete. "
                          "O rei e o valete fecham a sequência com o ás, a dama e o dez da mesa. O ás da mão é a carta que sobra.",
                          [ev("highlight", "O rei e o valete", player=0, hole=[1, 2], boardPick=[0, 1, 3],
                              text="Sequência até o ás: rei e valete da mão")],
                          banner="Turn"))
    scenes += [
        lines("m5l4-03", "Três duplas possíveis", "Quantas mãos de cinco cartas?",
              "Conte as mãos de cinco cartas que usam no máximo duas das suas três. Com o flop na mesa, são três. Com o turn, já são quinze. E com o river, chegam a quarenta e seis. "
              "Elas compartilham cartas: não são quarenta e seis chances independentes.",
              ["Flop: 3 mãos possíveis", "Turn: 15", "River: 46", "Compartilham cartas: não são chances independentes"],
              ["o flop", "o turn", "o river", "compartilham cartas"]),
        lines("m5l4-04", "Uma chance exata", "A carta que você recebe na troca",
              "Agora uma chance exata. Você tem ás, rei e sete; na mesa, ás, dama e dois. Você conhece seis cartas, "
              "então restam quarenta e seis desconhecidas. Dois ases estão entre elas. Ao trocar o sete, a chance de receber um ás "
              "é de dois em quarenta e seis: cerca de quatro vírgula três por cento.",
              ["6 cartas conhecidas → 46 desconhecidas", "2 ases restantes", "2 ÷ 46 ≈ 4,35%"],
              ["seis cartas", "Dois ases", "dois em quarenta e seis"],
              note="Hipótese: baralho embaralhado, sem ler as mãos dos oponentes"),
        lines("m5l4-05", "Uma chance exata", "O que a conta não diz",
              "Essa conta não diz se você vai ganhar. Os oponentes também trocam, a mesa muda e, no fim, só duas das suas cartas contam. "
              "As cartas que eles trocaram continuam desconhecidas: não invente o valor delas. Só o descarte do river aparece aberto.",
              ["Receber o ás não garante a vitória", "Os oponentes também trocam", "Carta trocada continua desconhecida"]),
    ]
    scenes += quiz_pair("m5l4", "Exercício", "Trocando o 7♣, qual a chance de receber um ás?",
                        [("A", "2 em 46", "dois em quarenta e seis"), ("B", "2 em 52", "dois em cinquenta e dois"),
                         ("C", "3 em 46", "três em quarenta e seis")], "A",
                        "Exercício. Você tem ás de ouros, rei de ouros e sete de paus. Na mesa, ás de copas, dama de paus e dois de espadas. "
                        "Você troca o sete. Qual a chance de receber um ás? A: dois em quarenta e seis. B: dois em cinquenta e dois. "
                        "C: três em quarenta e seis. Pause o vídeo e responda.",
                        "Resposta: A, dois em quarenta e seis. Você conhece seis cartas, e duas delas são ases. "
                        "Restam quarenta e seis cartas desconhecidas e dois ases.",
                        "6 cartas conhecidas, 2 ases já vistos · restam 46 cartas e 2 ases", ["Ad", "Kd", "7c"], ["Ah", "Qc", "2s"])
    scenes += [lines("m5l4-30", "Laboratório", "Leve a aula para a mesa", LAB_VOICE, LAB_LINES),
               outro("m5l4-99", "Fechamento", OUT_VOICE, OUT_LINES)]
    write("academy-m5l4", "Aula m5l4 — Combinações, trocas e probabilidade", scenes, lesson="m5l4")


def lesson_m5l5():
    kicker = "Zero Tilt Academy · Brazilian Pineapple · Aula 3"
    scenes = [intro("m5l5-01", "Abertura", "Showdown, empates e leitura de equity",
                    "Esta é a Zero Tilt Academy. Na aula três do Brazilian Pineapple, vamos estudar o showdown, os empates e a leitura de equity. "
                    "O objetivo é formar a melhor mão com duas cartas, dividir potes corretamente e não confundir ranking com equity.",
                    kicker, ["Objetivo: formar a mão, dividir potes e ler equity"], "Fichas sem valor · este curso não promete lucro"),
              flush_scene("m5l5-02", "Formação da mão")]
    h = custom([["9c", "9d"], ["Kc", "Qs", "4d"], ["8s", "8d"], ["7c", "5c", "3d"]], ["Ah", "Kh", "Qh", "Jh", "Th"], 2000,
               stacks=(9000, 9700, 9000, 9900), folded=(1, 3))
    scenes.append(h.scene("m5l5-03", "Formação da mão", "Quando a mesa é a melhor mão",
                          "A mesa também pode ser a melhor mão. Aqui o royal flush está inteiro nas cartas comunitárias. "
                          "Ana e Caio chegam ao showdown, e nenhum dos dois tem nada melhor. O pote é dividido: dez reais para cada um.",
                          [ev("reveal", "Ana e Caio", players=[2]),
                           ev("highlight", "nenhum dos dois", boardPick=[0, 1, 2, 3, 4], hole=[], text="Royal flush na mesa: pote dividido"),
                           ev("award", "dez reais para cada um", players=[0, 2], amounts=[1000, 1000], label="Divide o pote")],
                          banner="Showdown", pad=1.4))
    scenes += [
        lines("m5l5-04", "Ranking não é equity", "Duas medidas diferentes",
              "Ranking e equity são coisas diferentes. O ranking classifica a mão que você tem agora. A equity é a sua parcela esperada do pote, "
              "contando as frações de empate. Antes do river, ela também depende das trocas que ainda vão acontecer.",
              ["Ranking: a mão formada agora", "Equity: sua parcela esperada do pote", "Antes do river, depende das trocas futuras"],
              ["O ranking classifica", "A equity é", "Antes do river"]),
        lines("m5l5-05", "Loss Deflator", "A equity do all-in",
              "No all-in, o Loss Deflator usa uma regra fixa. Valem as cartas que cada jogador tinha naquele instante: "
              "três antes do descarte, duas depois. De três cartas, contam as duas melhores em cada desfecho. Trocas futuras não entram na conta.",
              ["Cartas do instante do all-in", "De três cartas, contam as duas melhores", "Trocas futuras não entram"],
              ["naquele instante", "De três cartas", "Trocas futuras"]),
    ]
    scenes += quiz_pair("m5l5", "Exercício", "Qual carta você descarta?",
                        [("A", "O ás de copas", "o ás de copas"), ("B", "Um dos setes", "um dos setes")], "B",
                        "Exercício. No river, você tem ás de copas e dois setes. Na mesa, rei, nove, seis e três de copas, e o valete de paus. "
                        "Qual carta você descarta? A: o ás de copas. B: um dos setes. Pause o vídeo e responda.",
                        "Resposta: B, um dos setes. O ás de copas, com as quatro copas da mesa, forma o flush. Uma carta da mão já basta. "
                        "O par de setes não melhora essa mão.",
                        "Quatro copas na mesa + ás de copas = flush · descarte um dos setes", ["Ah", "7s", "7d"], ["Kh", "9h", "6h", "3h", "Jc"])
    scenes += [lines("m5l5-30", "Laboratório", "Leve a aula para a mesa", LAB_VOICE, LAB_LINES),
               outro("m5l5-99", "Fechamento", OUT_VOICE, OUT_LINES)]
    write("academy-m5l5", "Aula m5l5 — Showdown, empates e leitura de equity", scenes, lesson="m5l5")


if __name__ == "__main__":
    rules()
    lesson_m5l3()
    lesson_m5l4()
    lesson_m5l5()
    teaser()
