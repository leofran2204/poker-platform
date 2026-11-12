"""Gera os roteiros do Brazilian Pineapple (src/data/*.json) a partir de uma mão-modelo.

A mão-modelo é uma só, usada no trailer, no filme de regras e na aula 1: quatro
jogadores, blinds R$ 0,50/R$ 1, ante de R$ 1 do big blind. Cada cena "hand" recebe
o estado da mesa naquele ponto (stacks, apostas, pote, cartas) calculado aqui, para
que os valores na tela nunca saiam da conta. O motor confere tudo em
Motor-Rust/tests/pineapple_draw_film_examples.rs.

Uso: python tools/build_scripts.py
"""
from __future__ import annotations

import copy
import json
from pathlib import Path

DATA = Path(__file__).resolve().parents[1] / "src" / "data"
VOICE, RATE, FPS = "pt-BR-AntonioNeural", "-3%", 30
RULE = "pineapple_draw_v1"

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

    def scene(self, scene_id, chapter, title, voice, events, banner=None, note=None, pad=None):
        players = [
            {"name": NAMES[i], "stack": self.stacks[i], **({"bet": self.bets[i]} if self.bets[i] else {}),
             "cards": list(self.cards[i]) if (self.dealt or any(e["type"] == "deal" for e in events)) else [],
             **({"folded": True} if self.folded[i] else {})}
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
        for event in events:
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
        elif kind == "swap":
            self.cards[who][event["position"]] = event["receive"]
        elif kind == "discard":
            del self.cards[who][event["position"]]
        elif kind == "reveal":
            self.open += [p for p in event["players"] if p not in self.open]
        elif kind == "award":
            winners = event.get("players", [who])
            shares = event.get("amounts", [self.pot // len(winners)] * len(winners))
            for player, share in zip(winners, shares):
                self.stacks[player] += share
            self.pot -= sum(shares)


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
                "Quatro jogadores na mesa. Cada um recebe três cartas fechadas. Ana paga o small blind. "
                "Beto paga o big blind e o ante, que vai direto para o pote.",
                [ev("deal", "recebe três cartas"),
                 ev("post", "Ana paga", player=0, to=50, label="Small blind"),
                 ev("post", "Beto paga", player=1, to=100, label="Big blind"),
                 ev("ante", "o ante", player=1, to=100, label="Big blind + ante")],
                banner="Pré-flop"),
        h.scene(f"{prefix}pre", c["pre"], "Primeira rodada de apostas",
                "Primeira rodada de apostas. Caio paga. Duda paga. Ana completa a aposta. Beto dá check. "
                "As fichas de todos vão para o pote. Não há troca antes do flop.",
                [ev("call", "Caio paga", player=2, to=100),
                 ev("call", "Duda paga", player=3, to=100),
                 ev("call", "Ana completa", player=0, to=100, label="Completa"),
                 ev("check", "Beto dá check", player=1),
                 ev("collect", "vão para o pote")],
                banner="Pré-flop · apostas"),
        h.scene(f"{prefix}flop", c["flop"], "Flop: primeiro as apostas",
                "O flop abre três cartas na mesa. Primeiro vêm as apostas. Ana tem par de ases e aposta. "
                "Beto paga. Caio paga. Duda desiste. As apostas vão para o pote.",
                [ev("board", "abre três cartas", cards=["As", "Qd", "6h"]),
                 ev("bet", "e aposta", player=0, to=300),
                 ev("call", "Beto paga", player=1, to=300),
                 ev("call", "Caio paga", player=2, to=300),
                 ev("fold", "Duda desiste", player=3),
                 ev("collect", "vão para o pote")],
                banner="Flop · apostas"),
        h.scene(f"{prefix}swap1", c["swap1"], "Depois das apostas, a troca",
                "Só depois das apostas vem a troca. Cada jogador mantém as três cartas ou troca uma. "
                "Ana troca o sete e recebe outro ás: trinca. Beto troca uma carta. Caio também troca uma. "
                "Todos veem quantas cartas cada um trocou, mas não quais.",
                [ev("swap", "Ana troca", player=0, position=2, receive="Ac"),
                 ev("swap", "Beto troca", player=1, position=2, receive="3c"),
                 ev("swap", "Caio também", player=2, position=2, receive="Th")],
                banner="Flop · troca"),
        h.scene(f"{prefix}turn", c["turn"], "Turn: apostas e nova troca",
                "O turn abre a quarta carta. Nova rodada de apostas: Ana aposta com a trinca. Beto desiste. Caio paga. "
                "Depois das apostas, a segunda troca: Ana mantém as cartas e Caio também.",
                [ev("board", "quarta carta", cards=["Kh"]),
                 ev("bet", "Ana aposta", player=0, to=700),
                 ev("fold", "Beto desiste", player=1),
                 ev("call", "Caio paga", player=2, to=700),
                 ev("collect", "Depois das apostas"),
                 ev("banner", "a segunda troca", text="Turn · troca"),
                 ev("keep", "Ana mantém", player=0),
                 ev("keep", "Caio também", player=2)],
                banner="Turn · apostas"),
        h.scene(f"{prefix}river", c["river"], "River: cada um descarta uma carta",
                "O river abre a quinta carta. Agora a regra muda: cada jogador descarta uma carta e fica com duas. "
                "Ana descarta o rei. Caio descarta a dama.",
                [ev("board", "quinta carta", cards=["6s"]),
                 ev("discard", "Ana descarta", player=0, position=1),
                 ev("discard", "Caio descarta", player=2, position=0)],
                banner="River · descarte"),
        h.scene(f"{prefix}last", c["last"], "Última rodada, com duas cartas",
                "Última rodada de apostas, com duas cartas na mão. Ana aposta. Caio paga. As fichas vão para o pote.",
                [ev("bet", "Ana aposta", player=0, to=1500),
                 ev("call", "Caio paga", player=2, to=1500),
                 ev("collect", "vão para o pote")],
                banner="River · apostas"),
        h.scene(f"{prefix}show", c["show"], "Showdown",
                "Showdown. Caio mostra uma sequência até o ás. Ana mostra full house: três ases e dois seis. "
                "O pote de cinquenta e oito reais vai para Ana.",
                [ev("reveal", "Caio mostra", players=[2]),
                 ev("highlight", "Ana mostra", player=0, hole=[0, 1], boardPick=[0, 2, 4], text="Full house de ases com seis"),
                 ev("award", "vai para Ana", player=0)],
                banner="Showdown", pad=1.6),
    ]
    assert h.stacks == [13200, 9500, 7400, 9900] and h.pot == 0, (h.stacks, h.pot)
    return scenes


# ───────────────────────────────────── Trailer ─────────────────────────────────────
def trailer():
    chapter = "Brazilian Pineapple"
    chapters = {k: chapter for k in ["deal", "pre", "flop", "swap1", "turn", "river", "last", "show"]}
    scenes = [intro("t01", chapter, "Brazilian Pineapple",
                    "Brazilian Pineapple. Uma modalidade de pôquer criada no Brasil, na Zero Tilt. "
                    "Três cartas na mão, duas trocas e um descarte.",
                    "Zero Tilt Poker · modalidade criada no Brasil", ["Três cartas. Duas trocas. Um descarte."],
                    "Exclusivo ZT · primeiro no Play Money")]
    scenes += model_hand("t-", chapters)
    scenes.append(outro("t99", chapter,
                        "Aprenda as regras na Zero Tilt Academy e pratique com Play Money. Poker envolve risco. "
                        "Só para maiores de dezoito anos.",
                        ["Regras completas na Academy", "Pratique com Play Money", "18+ • Jogue com responsabilidade"]))
    write("pineapple-draw-trailer", "Brazilian Pineapple — trailer", scenes)


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
                   "uma só ou nenhuma. Aqui há quatro copas na mesa. Ana descarta um sete e guarda o ás de copas. "
                   "Uma carta da mão já basta para o flush.",
                   [ev("discard", "Ana descarta", player=0, position=1),
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
ORDER_LINES = ["Três cartas · apostas", "Flop · apostas · troca", "Turn · apostas · troca", "River · descarte · apostas", "Showdown"]
ORDER_VOICE = ("A ordem da mão é sempre a mesma. Três cartas e a primeira rodada de apostas. Flop, apostas e troca. "
               "Turn, apostas e troca. River, descarte e a última rodada de apostas. Por fim, o showdown.")
ORDER_CUES = ["Três cartas", "Flop apostas", "Turn apostas", "River descarte", "o showdown"]


# ──────────────────────────────── Filme de regras ────────────────────────────────
def rules():
    scenes = [intro("r01", "O jogo", "Brazilian Pineapple",
                    "Zero Tilt Academy. Brazilian Pineapple: as regras completas. Uma modalidade de pôquer criada no Brasil, "
                    "com três cartas na mão, duas trocas e um descarte.",
                    "Zero Tilt Academy · regras completas", ["Três cartas. Duas trocas. Um descarte."],
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
    scenes += model_hand("r-", {"deal": "Pré-flop", "pre": "Pré-flop", "flop": "Flop: apostas e troca", "swap1": "Flop: apostas e troca",
                                "turn": "Turn: apostas e troca", "river": "River: descarte e apostas", "last": "River: descarte e apostas",
                                "show": "Showdown"})
    scenes += [
        lines("r20", "A troca em detalhe", "Manter ou trocar uma carta",
              "A troca é opcional e de uma carta por vez. A carta trocada sai do jogo e outra vem do baralho. "
              "Todos veem quantas cartas cada jogador trocou; ninguém vê quais. Se o tempo acabar, você mantém as cartas. "
              "E quem está all-in não troca mais.",
              ["Opcional: uma carta por vez", "Quantas, todos veem. Quais, ninguém vê", "Tempo esgotado: você mantém as cartas", "All-in não troca"],
              ["opcional", "Todos veem", "Se o tempo acabar", "all-in não troca"]),
        lines("r21", "O descarte do river", "Três cartas viram duas",
              "No river, o descarte é obrigatório. A carta sai do jogo sem ser mostrada, e a última rodada de apostas é disputada "
              "com duas cartas. Se o tempo acabar, o sistema guarda as duas cartas que formam a sua melhor mão. "
              "Quem está all-in também fica com as duas melhores.",
              ["Descarte obrigatório, antes das apostas", "Tempo esgotado: ficam as duas melhores", "All-in: ficam as duas melhores"],
              ["obrigatório", "Se o tempo acabar", "Quem está all-in"]),
        flush_scene("r22", "Formação da mão"),
        lines("r23", "Formação da mão", "Ranking clássico",
              "O ranking é o clássico. Sequência vence trinca. Full house vence flush. E ás, dois, três, quatro e cinco formam a sequência mais baixa.",
              ["Sequência vence trinca", "Full house vence flush", "A-2-3-4-5 é a sequência baixa"],
              ["Sequência vence", "Full house vence", "sequência mais baixa"]),
        lines("r24", "Apostas", "Blinds, ante e limite do pote",
              "As apostas seguem três regras. Há small blind e big blind. O big blind também paga um ante do mesmo valor, que vai direto para o pote. "
              "E, em todas as rodadas, o aumento máximo é o pote formado antes do call.",
              ["Small blind e big blind", "Ante de 1 big blind, pago só pelo big blind", "Aumento máximo: o pote antes do call"],
              ["small blind e big blind", "um ante", "aumento máximo"]),
    ]
    h = custom(START_CARDS, ["As", "Qd", "6h"], 500, stacks=(9900, 9800, 9900, 9900))
    scenes.append(h.scene("r25", "Apostas", "O limite do pote, na prática",
                          "Veja o limite na prática. O pote tem cinco reais, então Ana pode apostar até cinco reais. Ana aposta cinco. "
                          "Agora o pote tem dez, e pagar custa cinco. Beto pode aumentar para até quinze reais. Beto aumenta. "
                          "Caio desiste. Duda desiste. Ana paga.",
                          [ev("bet", "Ana aposta cinco", player=0, to=500, label="Aposta o máximo"),
                           ev("raise", "Beto aumenta", player=1, to=1500, label="Aumenta para o máximo"),
                           ev("fold", "Caio desiste", player=2),
                           ev("fold", "Duda desiste", player=3),
                           ev("call", "Ana paga", player=0, to=1500),
                           ev("collect", delay=0.9)],
                          banner="Flop · apostas"))
    scenes += [
        lines("r26", "All-in", "Quando alguém aposta tudo",
              "Quem está all-in não troca mais cartas. No river, fica automaticamente com as duas melhores. "
              "E, quando ninguém mais pode apostar, não há troca nem escolha de descarte: a mesa é completada direto.",
              ["All-in não troca", "No river, ficam as duas melhores", "Sem apostas possíveis: a mesa é completada"],
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
              "Pratique com Play Money e estude as três aulas do Brazilian Pineapple na Zero Tilt Academy. Poker envolve risco. "
              "Só para maiores de dezoito anos.",
              ["Três aulas na Academy", "Pratique com Play Money", "18+ • Jogue com responsabilidade"]),
    ]
    write("pineapple-draw-rules", "Brazilian Pineapple — regras completas", scenes)


# ──────────────────────────────────── Aulas ────────────────────────────────────
def lesson_m5l3():
    kicker = "Zero Tilt Academy · Brazilian Pineapple · Aula 1"
    scenes = [intro("m5l3-01", "Abertura", "Três cartas, duas trocas e um descarte",
                    "Zero Tilt Academy. Brazilian Pineapple, aula um: três cartas, duas trocas e um descarte. "
                    "O objetivo é acompanhar a mão do começo ao fim e decidir quando manter, trocar e descartar.",
                    kicker, ["Objetivo: acompanhar a mão e decidir a troca"], "Fichas sem valor · este curso não promete lucro"),
              lines("m5l3-02", "A ordem da mão", "A ordem da mão", ORDER_VOICE, ORDER_LINES, ORDER_CUES)]
    one = "A mão completa"
    scenes += model_hand("m5l3-", {k: one for k in ["deal", "pre", "flop", "swap1", "turn", "river", "last", "show"]})
    scenes.append(lines("m5l3-20", "All-in e tempo", "Casos especiais",
                        "Três casos especiais. Quem está all-in não troca e, no river, fica com as duas melhores cartas. "
                        "Se o tempo acabar na troca, você mantém as cartas. Se acabar no descarte, o sistema guarda as duas melhores.",
                        ["All-in: não troca e fica com as duas melhores", "Tempo esgotado na troca: mantém",
                         "Tempo esgotado no descarte: ficam as duas melhores"],
                        ["all-in não troca", "na troca", "no descarte"]))
    scenes += quiz_pair("m5l3", "Exercício", "Qual privada fica fora da sua melhor mão?",
                        [("A", "Ás de espadas", "ás de espadas"), ("B", "Rei de espadas", "rei de espadas"), ("C", "Ás de copas", "ás de copas")], "B",
                        "Exercício. Você tem ás e rei de espadas e ás de copas. Na mesa, ás de ouros, dama de paus e dois de espadas. "
                        "Qual privada fica fora da sua melhor mão? A: ás de espadas. B: rei de espadas. C: ás de copas. Pause o vídeo e responda.",
                        "Resposta: B, o rei de espadas. Os dois ases da mão, com o ás da mesa, formam trinca de ases. "
                        "Só duas cartas chegam ao showdown, então o rei é o candidato natural à troca. "
                        "Manter também é legítimo, porque a quantidade trocada é pública.",
                        "Trinca de ases com dama e dois · o rei fica fora e é o candidato à troca", ["As", "Ks", "Ah"], ["Ad", "Qc", "2s"])
    scenes += [lines("m5l3-30", "Laboratório", "Leve a aula para a mesa", LAB_VOICE, LAB_LINES),
               outro("m5l3-99", "Fechamento", OUT_VOICE, OUT_LINES)]
    write("academy-m5l3", "Aula m5l3 — Três cartas, duas trocas e um descarte", scenes, lesson="m5l3")


def lesson_m5l4():
    kicker = "Zero Tilt Academy · Brazilian Pineapple · Aula 2"
    scenes = [intro("m5l4-01", "Abertura", "Combinações, trocas e probabilidade",
                    "Zero Tilt Academy. Brazilian Pineapple, aula dois: combinações, trocas e probabilidade. "
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
              "Conte as mãos de cinco cartas que usam no máximo duas das suas três. No flop são três. No turn, quinze. No river, quarenta e seis. "
              "Elas compartilham cartas: não são quarenta e seis chances independentes.",
              ["Flop: 3 mãos possíveis", "Turn: 15", "River: 46", "Compartilham cartas: não são chances independentes"],
              ["No flop", "No turn", "No river", "compartilham cartas"]),
        lines("m5l4-04", "Uma chance exata", "A carta que você recebe na troca",
              "Agora uma chance exata. Você tem ás, rei e sete; na mesa, ás, dama e dois. Você conhece seis cartas, "
              "então restam quarenta e seis desconhecidas. Dois ases estão entre elas. Ao trocar o sete, a chance de receber um ás "
              "é de dois em quarenta e seis: cerca de quatro vírgula três por cento.",
              ["6 cartas conhecidas → 46 desconhecidas", "2 ases restantes", "2 ÷ 46 ≈ 4,35%"],
              ["seis cartas", "Dois ases", "dois em quarenta e seis"],
              note="Hipótese: baralho embaralhado, sem ler as mãos dos oponentes"),
        lines("m5l4-05", "Uma chance exata", "O que a conta não diz",
              "Essa conta não diz se você vai ganhar. Os oponentes também trocam, a mesa muda e, no fim, só duas das suas cartas contam. "
              "Cartas descartadas que você não viu continuam desconhecidas: não invente o valor delas.",
              ["Receber o ás não garante a vitória", "Os oponentes também trocam", "Carta não vista continua desconhecida"]),
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
                    "Zero Tilt Academy. Brazilian Pineapple, aula três: showdown, empates e leitura de equity. "
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
    trailer()
