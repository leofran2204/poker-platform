"""Gera os roteiros das aulas de estratégia da Academy (Texas, Short Deck e Omaha) em Remotion.

Cada aula segue o texto da aula no site (Frontend-Web/src/data/courseContent.json): a fala
cobre as mesmas ideias, com exemplos animados na mesa (`variant: "holdem"`, valores em big
blinds), grade de mãos, posições e comparações. O exercício do vídeo é diferente do quiz da
aula e da prova final (o vídeo é público). As contas da mesa (stacks, apostas e pote) saem
da classe `Mesa`, e quem ganha no showdown é conferido pelo avaliador de mãos abaixo, para
que nada na tela saia da conta.

Depois de gerar, `src/generated/academy-index.ts` lista as aulas que já têm narração
(`narrate.py`), e o Root registra uma composição para cada uma (ex.: AcademyM1L1).

Uso: python tools/build_academy.py            (todas)
     python tools/build_academy.py m1l1 m1l2  (só essas)
"""
from __future__ import annotations

import itertools
import json
import math
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from build_scripts import DATA, FPS, RATE, VOICE, ev, render_words  # noqa: E402
import render  # noqa: E402  (importado por build_scripts; padrão de narração)

GENERATED = DATA.parent / "generated"
BB = 100  # 1 big blind = 100 unidades (a mesa mostra "2,5 BB")
STACK = 100 * BB
BADGE = "Fichas sem valor · este curso não promete lucro"
OUT_VOICE = ("Registre sua justificativa no caderno: acertar a leitura e respeitar as hipóteses importa mais do que ganhar a mão. "
             "O treino usa fichas sem valor, e este curso não promete lucro.")
OUT_LINES = ["Registre a justificativa no caderno", "Fichas sem valor no treino", "Este curso não promete lucro"]

# ─────────────────────────────── Avaliador de mãos (conferência) ───────────────────────────────
RANKS = "AKQJT98765432"
VALUE = {r: 14 - i for i, r in enumerate(RANKS)}


def _rank5(cards, short):
    vals = sorted((VALUE[c[0]] for c in cards), reverse=True)
    counts = sorted(Counter(vals).items(), key=lambda kv: (kv[1], kv[0]), reverse=True)
    flush = len({c[1] for c in cards}) == 1
    uniq = sorted(set(vals), reverse=True)
    low = [14, 9, 8, 7, 6] if short else [14, 5, 4, 3, 2]  # Short Deck: A-6-7-8-9 é a sequência baixa
    straight = uniq[0] if len(uniq) == 5 and uniq[0] - uniq[4] == 4 else low[1] if uniq == low else 0
    kinds = [k for k, _ in counts]
    if straight and flush:
        return (8, straight)
    if counts[0][1] == 4:
        return (7, *kinds)
    if counts[0][1] == 3 and counts[1][1] == 2:
        return (6, *kinds)
    if flush:
        return (5, *vals)
    if straight:
        return (4, straight)
    if counts[0][1] == 3:
        return (3, *kinds)
    if counts[0][1] == 2 and counts[1][1] == 2:
        return (2, *kinds)
    if counts[0][1] == 2:
        return (1, *kinds)
    return (0, *vals)


# Short Deck da ZT: trinca vence sequência e flush vence full house.
SHORT_ORDER = {3: 4, 4: 3, 5: 6, 6: 5}


def rank5(cards, short=False):
    r = _rank5(cards, short)
    return (SHORT_ORDER.get(r[0], r[0]), *r[1:]) if short else r


def best(hole, board, omaha=False, short=False):
    """Melhor mão de cinco: Texas e Short Deck usam qualquer combinação; Omaha usa duas da mão e três da mesa."""
    if omaha:
        return max(rank5(list(h) + list(b)) for h in itertools.combinations(hole, 2) for b in itertools.combinations(board, 3))
    return max(rank5(list(c), short) for c in itertools.combinations(list(hole) + list(board), 5))


def deck(exclude=(), short=False):
    return [r + s for r in (RANKS[:9] if short else RANKS) for s in "shdc" if r + s not in exclude]


def equity(hands, board, omaha=False):
    """Equity exata de cada mão até o river (empates divididos), enumerando as cartas que faltam."""
    rest = deck([c for h in hands for c in h] + list(board))
    shares = [0.0] * len(hands)
    total = 0
    for runout in itertools.combinations(rest, 5 - len(board)):
        full = list(board) + list(runout)
        scores = [best(h, full, omaha) for h in hands]
        top = max(scores)
        winners = [i for i, s in enumerate(scores) if s == top]
        for i in winners:
            shares[i] += 1 / len(winners)
        total += 1
    return [s / total for s in shares]


# ─────────────────────────────── Mesa animada (estado entre cenas) ───────────────────────────────
MONEY = {"post", "bet", "call", "raise", "allin"}


class Mesa:
    """Aplica os eventos de cada cena e guarda o estado (valores em centésimos de BB)."""

    def __init__(self, names, cards, stacks=None, dealer=None, hero=0, layout=4, seats=None, board=(), pot=0, omaha=False, short=False):
        n = len(names)
        self.names = list(names)
        self.cards = [list(c) for c in cards]
        self.stacks = list(stacks or [STACK] * n)
        self.bets = [0] * n
        self.folded = [False] * n
        self.board = list(board)
        self.pot = pot
        self.dealer, self.hero, self.layout, self.seats, self.omaha, self.short = dealer, hero, layout, seats, omaha, short
        self.open: list[int] = []
        used = [c for h in self.cards for c in h] + self.board
        assert len(used) == len(set(used)), f"carta repetida: {used}"
        assert set(used) <= set(deck(short=short)), f"carta fora do baralho: {used}"

    def scene(self, scene_id, chapter, title, voice, events, banner=None, note=None, pad=None):
        players = [
            {"name": self.names[i], "stack": self.stacks[i], **({"bet": self.bets[i]} if self.bets[i] else {}),
             "cards": list(self.cards[i]), **({"folded": True} if self.folded[i] else {})}
            for i in range(len(self.names))
        ]
        visual = {"mode": "hand", "variant": "holdem", "unit": "bb", "layout": self.layout, "hero": self.hero,
                  "players": players, "board": list(self.board), "pot": self.pot, "events": events}
        if self.dealer is not None:
            visual["dealer"] = self.dealer
        if self.seats:
            visual["seats"] = list(self.seats)
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
        if kind in MONEY:
            paid = event["to"] - self.bets[who]
            assert 0 <= paid <= self.stacks[who], event
            if kind == "call":
                assert event["to"] == max(self.bets) or paid == self.stacks[who], f"call errado: {event}"
            if kind == "raise":
                assert event["to"] > max(self.bets), f"raise abaixo da aposta: {event}"
            self.stacks[who] -= paid
            self.bets[who] = event["to"]
        elif kind == "check":
            assert self.bets[who] == max(self.bets), f"check com aposta a pagar: {event}"
        elif kind == "fold":
            self.folded[who] = True
        elif kind == "collect":
            self.pot += sum(self.bets)
            self.bets = [0] * len(self.bets)
        elif kind == "board":
            self.board += event["cards"]
            used = [c for h in self.cards for c in h] + self.board
            assert len(used) == len(set(used)), f"carta repetida: {used}"
        elif kind == "reveal":
            self.open += [p for p in event["players"] if p not in self.open]
        elif kind == "award":
            winners = event.get("players", [who])
            live = [i for i in range(len(self.names)) if not self.folded[i]]
            if len(live) > 1:
                assert len(self.board) == 5, "showdown sem river"
                scores = {i: best(self.cards[i], self.board, self.omaha, self.short) for i in live}
                top = max(scores.values())
                assert sorted(winners) == sorted(i for i in live if scores[i] == top), f"vencedor errado: {event}"
            shares = event.get("amounts", [self.pot // len(winners)] * len(winners))
            for player, share in zip(winners, shares):
                self.stacks[player] += share
            self.pot -= sum(shares)

    def total(self):
        return self.pot + sum(self.bets)


# ─────────────────────────────── Cenas genéricas ───────────────────────────────
def _cues(scene_id, voice, cues, ordered=True):
    spoken = render_words(voice)
    for cue in cues:
        assert not cue or render_words(cue) in spoken, f"{scene_id}: deixa '{cue}' fora da narração"
    positions = [spoken.index(render_words(c)) for c in cues if c]
    assert not ordered or positions == sorted(positions), f"{scene_id}: deixas fora de ordem {cues}"


def scene(scene_id, chapter, title, voice, visual, pad=None):
    out = {"id": scene_id, "chapter": chapter, "title": title, "voice": voice, "visual": visual}
    if pad is not None:
        out["padAfter"] = pad
    return out


def intro(scene_id, title, voice, kicker, objective):
    return scene(scene_id, "Abertura", title, voice, {"mode": "intro", "kicker": kicker, "lines": [objective], "badge": BADGE})


def steps(scene_id, chapter, title, voice, items, cues, note=None):
    _cues(scene_id, voice, cues)
    visual = {"mode": "steps", "lines": items, "cues": cues}
    if note:
        visual["note"] = note
    return scene(scene_id, chapter, title, voice, visual)


def metrics(scene_id, chapter, title, voice, pairs, cues, note=None):
    _cues(scene_id, voice, cues)
    visual = {"mode": "money", "metrics": [list(p) for p in pairs], "cues": cues}
    if note:
        visual["note"] = note
    return scene(scene_id, chapter, title, voice, visual)


def compare(scene_id, chapter, title, voice, columns, note=None, note_cue=None):
    """`columns`: [(título, [(item, deixa), ...], tom), ...]."""
    cols = []
    for head, items, tone in columns:
        cols.append({"title": head, "items": [i for i, _ in items], "cues": [c for _, c in items], "tone": tone})
    _cues(scene_id, voice, [c for col in cols for c in col["cues"]] + ([note_cue] if note_cue else []))
    visual = {"mode": "compare", "columns": cols}
    if note:
        visual["note"] = note
    if note_cue:
        visual["noteCue"] = note_cue
    return scene(scene_id, chapter, title, voice, visual)


def positions(scene_id, chapter, title, voice, seats, button=None, note=None, note_cue=None):
    """`seats`: [(rótulo, valor, deixa, você?), ...] a partir de quem fica embaixo, em sentido horário."""
    out = [{"label": label, **({"value": value} if value else {}), **({"cue": cue} if cue else {}), **({"hero": True} if hero else {})}
           for label, value, cue, hero in seats]
    _cues(scene_id, voice, [s.get("cue") for s in out if s.get("cue")] + ([note_cue] if note_cue else []), ordered=False)
    visual = {"mode": "positions", "seats": out}
    if button is not None:
        visual["button"] = button
    if note:
        visual["note"] = note
    if note_cue:
        visual["noteCue"] = note_cue
    return scene(scene_id, chapter, title, voice, visual)


def cell(row, col):
    return RANKS[row] + RANKS[col] if row == col else f"{RANKS[row]}{RANKS[col]}s" if row < col else f"{RANKS[col]}{RANKS[row]}o"


def combos(hand):
    return 6 if len(hand) == 2 else 4 if hand.endswith("s") else 12


def grid(scene_id, chapter, title, voice, groups, note=None, note_cue=None):
    """`groups`: [(rótulo, mãos, deixa, detalhe), ...]; mãos como "AKs", "QQ", "T9o"."""
    every = {cell(r, c) for r in range(13) for c in range(13)}
    out = []
    for label, hands, cue, detail in groups:
        assert set(hands) <= every, f"{scene_id}: mão fora da grade {set(hands) - every}"
        out.append({"label": label, "hands": list(hands), "cue": cue, **({"detail": detail} if detail else {})})
    _cues(scene_id, voice, [g["cue"] for g in out] + ([note_cue] if note_cue else []))
    visual = {"mode": "grid", "groups": out}
    if note:
        visual["note"] = note
    if note_cue:
        visual["noteCue"] = note_cue
    return scene(scene_id, chapter, title, voice, visual)


def quiz(prefix, question, options, correct, voice_q, voice_a, explanation, hole=None, board=None):
    """Exercício com pausa e resposta comentada. `options`: [(letra, texto, deixa), ...]."""
    opts = [{"key": k, "text": t, "cue": f"{k} {c}"} for k, t, c in options]
    base = {"question": question, "options": opts, "correct": correct,
            **({"hole": hole} if hole else {}), **({"board": board} if board else {})}
    _cues(f"{prefix}-quiz", voice_q, [o["cue"] for o in opts] + ["Pause"])
    _cues(f"{prefix}-answer", voice_a, ["Resposta"])
    assert correct in {o["key"] for o in opts}
    return [
        scene(f"{prefix}-quiz", "Exercício", "Sua vez", voice_q, {"mode": "quiz", **base}, pad=3.0),
        scene(f"{prefix}-answer", "Exercício", "Resposta", voice_a, {"mode": "answer", **base, "cue": "Resposta", "explanation": explanation}),
    ]


def lab(scene_id, voice, items, cues):
    return steps(scene_id, "Laboratório", "Leve a aula para a mesa", voice, items, cues)


def outro(scene_id):
    return scene(scene_id, "Fechamento", "Comece pelo treino", OUT_VOICE, {"mode": "outro", "lines": OUT_LINES}, pad=2.0)


def write(lesson, title, scenes):
    name = f"academy-{lesson}"
    script = {"id": name, "title": title, "voice": VOICE, "rate": RATE, "fps": FPS, "lesson": lesson, "scenes": scenes}
    ids = [s["id"] for s in scenes]
    assert len(ids) == len(set(ids)), f"{name}: ids repetidos"
    heavy = [(s["id"], h) for s in scenes for h in render.heavy_english_sentences(s["voice"])]
    assert not heavy, f"{name}: frases com muito inglês, reescreva com mais português: {heavy}"
    chapters = []
    for s in scenes:
        if not chapters or chapters[-1] != s["chapter"]:
            assert s["chapter"] not in chapters, f"{name}: capítulo '{s['chapter']}' volta depois de outro"
            chapters.append(s["chapter"])
    (DATA / f"{name}.json").write_text(json.dumps(script, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")
    words = sum(len(s["voice"].split()) for s in scenes)
    chars = sum(len(s["voice"]) for s in scenes)
    print(f"{name}: {len(scenes)} cenas, {len(chapters)} capítulos, {words} palavras (~{words / 2.45 / 60:.1f} min), {chars} caracteres")


def write_index():
    """Lista para o Root as aulas com roteiro e narração (tempos) prontos."""
    rows = []
    for path in sorted(DATA.glob("academy-m*.json")):
        lesson = path.stem.removeprefix("academy-")
        if lesson in {"m5l3", "m5l4", "m5l5"} or not (GENERATED / f"{path.stem}.timings.json").exists():
            continue  # Pineapple: registrado à parte no Root, com as cenas próprias
        module, number = lesson[1:].split("l")
        rows.append((lesson, f"Academy{lesson.upper()}", f"ZT ACADEMY · M{module} · AULA {number}"))
    lines = ["// Gerado por tools/build_academy.py: aulas de estratégia com roteiro e narração prontos.", ""]
    for lesson, _, _ in rows:
        lines.append(f'import {lesson}Script from "../data/academy-{lesson}.json";')
        lines.append(f'import {lesson}Timings from "./academy-{lesson}.timings.json";')
    lines += ["", "export const ACADEMY_LESSONS = ["]
    for lesson, comp, label in rows:
        lines.append(f'  {{ id: "{comp}", script: {lesson}Script, timings: {lesson}Timings, label: "{label}" }},')
    lines += ["];", ""]
    (GENERATED / "academy-index.ts").write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"academy-index.ts: {len(rows)} aulas com narração")


# ─────────────────────────────── Módulo 1 ───────────────────────────────
def m1l1():
    p = "m1l1"
    scenes = [
        intro(f"{p}-01", "Ranges de abertura por posição",
              "Esta é a Zero Tilt Academy. Na primeira aula do módulo um, vamos estudar ranges de abertura por posição, na mesa de nove jogadores. "
              "O objetivo é relacionar quantos jogadores ainda vão agir, a sua posição e o plano de abertura.",
              "Zero Tilt Academy · Módulo 1 · Aula 1", "Objetivo: relacionar posição, jogadores por agir e abertura"),
        steps(f"{p}-02", "Range e cenário", "O que é um range",
              "Um range é um conjunto de mãos possíveis, e cada mão pode aparecer com uma frequência diferente. "
              "Por isso, a pergunta não é só qual mão abrir, mas em que cenário: posição, stacks, rake e as respostas dos outros jogadores.",
              ["Range: um conjunto de mãos possíveis", "Cada mão pode ter uma frequência", "Depende de posição, stacks, rake e respostas"],
              ["Um range é", "frequência diferente", "em que cenário"]),
        positions(f"{p}-03", "Posições", "Quantos ainda vão agir",
                  "Na mesa de nove, quem abre primeiro é a posição inicial, logo à esquerda do big blind. Ela ainda enfrenta oito jogadores por agir. "
                  "A cada lugar para a esquerda, falta um a menos. Quando a vez chega ao cutoff, ainda faltam três jogadores. No botão, faltam só os dois blinds.",
                  [("BTN · você", "2 por agir", "No botão", True), ("SB", "1 por agir", None, False), ("BB", "último", None, False),
                   ("UTG", "8 por agir", "posição inicial", False), ("UTG+1", "7 por agir", None, False), ("UTG+2", "6 por agir", None, False),
                   ("LJ", "5 por agir", None, False), ("HJ", "4 por agir", None, False), ("CO", "3 por agir", "chega ao cutoff", False)],
                  button=0, note="Pré-flop: a ação começa à esquerda do big blind"),
        compare(f"{p}-04", "Posições", "Os dois extremos",
                "Compare os dois extremos. Na posição inicial, oito jogadores ainda podem ter uma mão forte, e quem pagar à sua esquerda terá posição depois do flop. "
                "No botão, só os blinds ficam por agir, e você fala por último depois do flop. "
                "Por isso o botão costuma abrir mais mãos, desde que stacks, rake e respostas sejam comparáveis.",
                [("Posição inicial (UTG)", [("Oito jogadores por agir", "oito jogadores"), ("Quem pagar terá posição", "quem pagar")], "danger"),
                 ("Botão", [("Só os blinds por agir", "só os blinds"), ("Fala por último no pós-flop", "você fala por último")], "ok")],
                note="Mais aberturas no botão, com hipóteses comparáveis", note_cue="Por isso o botão"),
        steps(f"{p}-05", "Pós-flop", "A ordem muda depois do flop",
              "Atenção à ordem depois do flop. A posição inicial fica fora de posição contra quem pagar à sua esquerda, mas fica em posição contra os blinds, "
              "porque do flop em diante os blinds falam primeiro e o botão fala por último. Não confunda a ordem do pré-flop com a do pós-flop.",
              ["UTG: fora de posição contra quem está à esquerda", "UTG: em posição contra os blinds", "Do flop em diante, o botão fala por último"],
              ["fora de posição", "em posição contra os blinds", "o botão fala por último"]),
    ]
    t = Mesa(["Você · BTN", "SB", "BB"], [["Ks", "Qs"], ["9d", "4c"], ["Jh", "8h"]], dealer=0, seats=[0, 1, 2])
    scenes.append(t.scene(f"{p}-06", "Exercício guiado", "Rei e dama do mesmo naipe no botão",
                          "Agora o exercício guiado. A mesa é de Texas cash, com nove jogadores e cem big blinds de stack. "
                          "Os blinds entram, e todos desistem até você, no botão, com rei e dama de espadas. "
                          "No plano básico, essa mão entra na abertura. Você abre para dois big blinds e meio, um tamanho comum de referência. "
                          "O jogador do small blind desiste, e quem está no big blind paga a diferença: mais um big blind e meio. O pote do flop fica com cinco big blinds e meio.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "Você abre", player=0, to=250), ev("fold", "small blind desiste", player=1),
                           ev("call", "paga a diferença", player=2, to=250), ev("collect", "O pote do flop")],
                          banner="Pré-flop · todos desistiram até você"))
    assert t.total() == 550
    scenes += [
        steps(f"{p}-07", "Exercício guiado", "Registre o plano",
              "Antes de agir, registre duas coisas: o tamanho da abertura e o que você vai fazer diante de uma 3-bet. "
              "Um quadro de ranges sem essas hipóteses fica incompleto.",
              ["O tamanho da abertura", "O plano diante de uma 3-bet", "Sem hipóteses, o quadro é incompleto"],
              ["o tamanho da abertura", "diante de uma 3-bet", "Um quadro"]),
        grid(f"{p}-08", "Combinações", "Cento e sessenta e nove classes",
             "Agora, conte combinações. A grade mostra as cento e sessenta e nove classes de mãos iniciais. "
             "Na diagonal ficam os pares: cada um tem seis combinações. Acima dela, as mãos do mesmo naipe, com quatro combinações cada. "
             "Abaixo, as de naipes diferentes, com doze. Somando tudo, são mil trezentas e vinte e seis mãos possíveis.",
             [("Pares", [cell(i, i) for i in range(13)], "Na diagonal", "13 classes × 6 = 78 combinações"),
              ("Mesmo naipe (suited)", [cell(r, c) for r in range(13) for c in range(r + 1, 13)], "Acima dela", "78 classes × 4 = 312 combinações"),
              ("Naipes diferentes (offsuit)", [cell(c, r) for r in range(13) for c in range(r + 1, 13)], "Abaixo", "78 classes × 12 = 936 combinações")],
             note="Total: 1.326 combinações", note_cue="Somando tudo"),
    ]
    assert 13 * 6 + 78 * 4 + 78 * 12 == 1326 == len(list(itertools.combinations(deck(), 2)))
    scenes += [
        metrics(f"{p}-09", "Combinações", "Frequência é por combinação",
                "Um exemplo: ás e rei têm quatro combinações do mesmo naipe e doze de naipes diferentes, dezesseis no total. "
                "Abrir uma classe em metade das vezes não quer dizer que cada combinação apareceu metade das vezes. "
                "Primeiro defina quais classes entram, e em que cenário.",
                [("AK SUITED", "4"), ("AK OFFSUIT", "12"), ("AK NO TOTAL", "16")],
                ["quatro combinações", "doze", "dezesseis no total"], note="Primeiro as classes, depois as frequências"),
        steps(f"{p}-10", "Outros fatores", "Stack, rake e jogadores",
              "Os outros fatores também mudam a conta. Uma abertura pode receber call, raise ou fold. "
              "Stacks menores deixam as decisões seguintes mais rasas; stacks maiores aumentam o peso da posição e das mãos que formam combinações fortes. "
              "Rake e cap mudam o valor de continuar. Por isso, os treinos usam rake zero e stacks declarados, e o resultado não é uma tabela pronta para qualquer mesa do lobby.",
              ["Stacks menores: decisões mais rasas", "Stacks maiores: a posição pesa mais", "Treino: rake zero e stacks declarados"],
              ["Stacks menores", "stacks maiores", "os treinos usam"], note="Não é uma tabela pronta para o lobby"),
        steps(f"{p}-11", "Pergunta estudável", "Monte uma pergunta estudável",
              "Então, em vez de perguntar se rei e dama do mesmo naipe é sempre abertura, monte uma pergunta estudável. "
              "Defina a posição, o número de jogadores, o stack efetivo e o tamanho. "
              "Depois pergunte quais respostas dos blinds tornariam outra linha melhor. Não usamos tabelas de fornecedores como gabarito deste treino.",
              ["Posição e número de jogadores", "Stack efetivo e tamanho", "Que resposta mudaria a linha?"],
              ["Defina a posição", "o stack efetivo", "quais respostas"]),
    ]
    scenes += quiz(p, "Todos desistiram até você, no cutoff. Quantos jogadores ainda vão agir no pré-flop?",
                   [("A", "Dois", "dois"), ("B", "Três", "três"), ("C", "Quatro", "quatro")], "B",
                   "Exercício. Na mesa de nove, todos desistiram até você, no cutoff. Quantos jogadores ainda vão agir no pré-flop? "
                   "A: dois. B: três. C: quatro. Pause o vídeo e responda.",
                   "Resposta: B, três. Depois do cutoff ainda falam o botão, o small blind e o big blind. "
                   "Com mais jogadores por agir, o cutoff costuma abrir menos mãos que o botão, com as mesmas hipóteses.",
                   "Botão, small blind e big blind ainda falam")
    scenes += [
        lab(f"{p}-30",
            "Agora leve a aula para a mesa de treino da Academy. A mesa didática começa com três jogadores: ajuste o range ao número real de oponentes, "
            "sem copiar um quadro de nove. Comece pelo treino guiado, depois repita no modo desafio e use o replay para comparar uma alternativa na mesma distribuição.",
            ["A mesa didática começa com três", "Ajuste o range aos oponentes reais", "Guiado, desafio e replay"],
            ["começa com três jogadores", "ajuste o range", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m1l1 — Ranges de abertura por posição", scenes)


def m1l2():
    p = "m1l2"
    scenes = [
        intro(f"{p}-01", "Como punir o limp do small blind",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo um, vamos estudar como punir o limp do small blind. "
              "O objetivo é comparar o check e o isolamento depois que o small blind completa.",
              "Zero Tilt Academy · Módulo 1 · Aula 2", "Objetivo: comparar check e isolamento após o limp"),
    ]
    t = Mesa(["Você · BB", "SB"], [["As", "5s"], ["Qd", "7c"]], seats=[0, 2])
    scenes.append(t.scene(f"{p}-02", "Limp do small blind", "O small blind completa",
                          "Todos desistiram até o small blind. Ele já tinha meio big blind e completa com mais meio. "
                          "Agora os dois têm um big blind comprometido. Você, no big blind, não precisa pagar nada para ver o flop: pode dar check. "
                          "Ou pode aumentar, dentro dos limites legais.",
                          [ev("post", None, player=1, to=50, label="SB"), ev("post", None, player=0, to=100, label="BB"),
                           ev("call", "completa com mais meio", player=1, to=100, label="Limp")],
                          banner="Pré-flop · todos desistiram até o small blind", note="Check é legal: nada falta pagar"))
    scenes += [
        steps(f"{p}-03", "Limp do small blind", "O limp não define o jogador",
              "Cuidado com o rótulo. O limp pode conter mãos fracas e mãos fortes. "
              "Sozinho, ele não identifica um jogador fraco: é preciso observar o que o rival faz depois.",
              ["O limp pode ter mãos fracas", "E também mãos fortes", "Sozinho, não identifica um jogador fraco"],
              ["mãos fracas", "mãos fortes", "Sozinho"]),
        steps(f"{p}-04", "Plano do exercício", "O perfil do exercício",
              "No exercício, a mesa é de Texas cash, com blinds de meio e um, e cem big blinds de stack. "
              "O small blind completa e tende a pagar isolamentos com mãos piores, sem muitas armadilhas. "
              "Contra esse perfil, treinamos o raise com ás e cinco de espadas, usando de três e meio a quatro big blinds como referência de tamanho.",
              ["Texas cash · 100 BB · blinds 0,5/1", "Rival paga com mãos piores, poucas armadilhas", "Plano: raise com A5s, de 3,5 a 4 BB"],
              ["a mesa é de Texas", "tende a pagar", "treinamos o raise"]),
    ]
    scenes.append(t.scene(f"{p}-05", "Plano do exercício", "Total da aposta e custo adicional",
                          "Você aumenta para quatro big blinds no total. Como já tinha um na frente, coloca mais três. "
                          "O small blind então decide se acrescenta outros três big blinds. "
                          "Não confunda o custo adicional com o total da aposta.",
                          [ev("raise", "Você aumenta", player=0, to=400)],
                          note="Total: 4 BB · a mais: 3 BB"))
    scenes += [
        compare(f"{p}-06", "O tamanho depende da resposta", "O tamanho depende da resposta",
                "O tamanho depende da resposta do rival. Se ele paga aumentos com muitas mãos fracas, aumentar uma mão que joga bem contra essa faixa pode construir valor. "
                "Ás e cinco do mesmo naipe combina força, bloqueador de ás e a chance de formar flush. "
                "Mas, se ele tem armadilhas e aumenta de volta, a avaliação muda, e o check também pode fazer parte da estratégia.",
                [("Paga com mãos piores", [("O raise pode construir valor", "pode construir valor"), ("A5s: bloqueador e flush possível", "bloqueador de ás")], "ok"),
                 ("Tem armadilhas e reaumenta", [("A avaliação muda", "a avaliação muda"), ("O check também entra", "o check também")], "danger")],
                note="O perfil é uma hipótese do exercício"),
        steps(f"{p}-07", "Observe sem concluir cedo", "Observe antes de explorar",
              "Antes de explorar uma tendência, observe. Registre quantas vezes o rival completou, quantas desistiu para um aumento e quantas fez limp-reraise. "
              "Amostras pequenas oscilam bastante. No simulador, o perfil é declarado para o exercício ser reproduzível; numa mesa real, você teria de inferir essa informação.",
              ["Quantas vezes completou", "Quantas desistiu para um aumento", "Quantas fez limp-reraise"],
              ["quantas vezes o rival completou", "quantas desistiu", "quantas fez"], note="Amostras pequenas oscilam bastante"),
    ]
    scenes += quiz(p, "Você aumenta para 4 BB e o small blind paga. Quanto haverá no pote do flop?",
                   [("A", "5 BB", "cinco"), ("B", "7 BB", "sete"), ("C", "8 BB", "oito")], "C",
                   "Exercício. Você aumenta para quatro big blinds, e o small blind paga. Quantos big blinds haverá no pote do flop? "
                   "A: cinco. B: sete. C: oito. Pause o vídeo e responda.",
                   "Resposta: C, oito. Cada jogador termina a rodada com quatro na frente, e as duas apostas vão para o pote.",
                   "4 BB de cada jogador = 8 BB no pote")
    scenes += [
        lab(f"{p}-30",
            "Agora leve a aula para a mesa de treino. Você está no big blind: se nada falta pagar, o check é legal, e o isolamento aumenta o total da aposta. "
            "Comece pelo treino guiado e repita no modo desafio. No replay, compare check e raise na mesma distribuição, sem usar as cartas do final como justificativa.",
            ["Você está no big blind", "Compare check e raise no replay", "As cartas do final não justificam a decisão"],
            ["Você está no big blind", "compare check e raise", "sem usar as cartas"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m1l2 — Como punir o limp do small blind", scenes)


def m1l3():
    p = "m1l3"
    scenes = [
        intro(f"{p}-01", "3-bet por valor e 3-bet light",
              "Esta é a Zero Tilt Academy. Na aula três do módulo um, vamos estudar a 3-bet por valor e a 3-bet light com bloqueadores. "
              "O objetivo é construir uma 3-bet e distinguir o total do incremento.",
              "Zero Tilt Academy · Módulo 1 · Aula 3", "Objetivo: construir uma 3-bet e separar total e incremento"),
        steps(f"{p}-02", "Valor, blefe e bloqueadores", "O que é uma 3-bet",
              "Uma 3-bet é um aumento sobre o raise inicial. O range e o tamanho dependem das posições, do stack efetivo e das respostas dos adversários. "
              "Ás e rei do mesmo naipe pode fazer parte do valor. Ás e cinco do mesmo naipe pode aparecer como blefe em certos cenários, mas não é obrigação.",
              ["3-bet: um aumento sobre o raise inicial", "Valor: AKs pode fazer parte", "Blefe: A5s pode aparecer, sem obrigação"],
              ["Uma 3-bet é", "Ás e rei", "Ás e cinco"]),
    ]
    t = Mesa(["Você · BTN", "SB", "BB", "CO"], [["Ah", "Kh"], ["Tc", "6d"], ["8s", "3s"], ["Qd", "Jd"]], dealer=0)
    scenes.append(t.scene(f"{p}-03", "A 3-bet na prática", "A 3-bet por valor",
                          "Veja o exercício. A mesa é de Texas cash, com cem big blinds de stack. "
                          "O cutoff abre para dois big blinds e meio, e você está no botão com ás e rei de copas. "
                          "No plano de treino, isso é uma 3-bet por valor, com referência de sete e meio a oito big blinds. Você aumenta para oito no total. "
                          "Os blinds desistem, e o cutoff paga.",
                          [ev("post", None, player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "O cutoff abre", player=3, to=250), ev("raise", "Você aumenta", player=0, to=800),
                           ev("fold", "Os blinds desistem", player=1), ev("fold", None, player=2),
                           ev("call", "o cutoff paga", player=3, to=800), ev("collect", None)],
                          banner="Pré-flop · Texas cash · 100 BB"))
    assert t.total() == 1750
    scenes += [
        metrics(f"{p}-04", "Três tamanhos", "Total, incremento e custo",
                "Repare nos três números. A sua 3-bet é de oito big blinds no total. O incremento sobre a abertura foi de cinco big blinds e meio. "
                "E, como você não tinha fichas na rodada, o seu custo também foi de oito. No big blind, parte já estaria comprometida. "
                "Essa diferença importa para as pot odds, o stack efetivo e a resposta a uma 4-bet.",
                [("TOTAL DA 3-BET", "8 BB"), ("INCREMENTO", "5,5 BB"), ("SEU CUSTO", "8 BB")],
                ["oito big blinds no total", "O incremento", "o seu custo"], note="No big blind, parte já estaria comprometida"),
        compare(f"{p}-05", "Contagem de combinações", "O ás na sua mão",
                "Agora, os bloqueadores. Sem cartas conhecidas, existem seis combinações de par de ases e dezesseis de ás e rei. "
                "Se você segura um ás, sem conhecer outras cartas, restam três combinações de par de ases e doze de ás e rei.",
                [("Sem cartas conhecidas", [("AA: 6 combinações", "seis combinações"), ("AK: 16 combinações", "dezesseis")], "mint"),
                 ("Com um ás na sua mão", [("AA: 3 combinações", "restam três"), ("AK: 12 combinações", "doze de ás e rei")], "gold")]),
        steps(f"{p}-06", "Bloqueadores têm dois lados", "Bloqueadores têm dois lados",
              "Mas o bloqueador tem dois lados. Segurar um ás reduz certas mãos fortes do rival, e também algumas mãos com que ele desistiria. "
              "Um bloqueador, sozinho, não dá a frequência de fold. Liste primeiro o range de abertura, depois as mãos que pagam e as que aumentam de novo.",
              ["Reduz mãos fortes do rival", "Também reduz mãos que desistiriam", "Liste: abre, paga, aumenta de novo"],
              ["reduz certas mãos", "e também algumas", "Liste primeiro"], note="Bloquear ases não prova que o blefe lucra"),
        compare(f"{p}-07", "Planeje a próxima decisão", "Antecipe o stack que resta",
                "Ao escolher a 3-bet, antecipe o stack que vai restar se houver call. "
                "Com cem big blinds, ainda cabem várias apostas depois do flop. "
                "Com vinte, a relação entre o pote e o stack muda muito: não transplante o tamanho automaticamente.",
                [("Com 100 BB", [("Ainda cabem várias apostas", "ainda cabem")], "mint"),
                 ("Com 20 BB", [("Pote e stack mudam muito", "Com vinte"), ("Não transplante o tamanho", "não transplante")], "gold")]),
    ]
    scenes += quiz(p, "O CO abre 2,5 BB; você faz 3-bet para 7,5 BB, os blinds desistem e o CO paga. Qual é o pote do flop?",
                   [("A", "15 BB", "quinze"), ("B", "16,5 BB", "dezesseis e meio"), ("C", "18 BB", "dezoito")], "B",
                   "Exercício. O cutoff abre para dois big blinds e meio, e você faz 3-bet para sete e meio. Os blinds desistem, e o cutoff paga. "
                   "Quantos big blinds haverá no pote do flop? A: quinze. B: dezesseis e meio. C: dezoito. Pause o vídeo e responda.",
                   "Resposta: B, dezesseis e meio. São sete e meio de cada um de vocês, mais o meio do small blind e o um do big blind, que desistiram.",
                   "7,5 + 7,5 + 0,5 + 1 = 16,5 BB")
    assert 750 + 750 + 50 + 100 == 1650
    scenes += [
        lab(f"{p}-30",
            "Agora leve a aula para a mesa de treino. A abertura adversária é de dois big blinds e meio: compare uma 3-bet por valor com apenas pagar. "
            "Pratique o total legal do raise e diga quais respostas fariam você continuar. Os bots não são um solver, e uma vitória contra eles não valida o range.",
            ["Compare a 3-bet com apenas pagar", "Pratique o total legal do raise", "Vencer os bots não valida o range"],
            ["compare uma 3-bet", "Pratique o total", "Os bots não são"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m1l3 — 3-bet por valor e 3-bet light com bloqueadores", scenes)


def guided_lab(scene_id, voice, items, cues):
    """Laboratório com o fechamento padrão (treino guiado, desafio e replay)."""
    return lab(scene_id, voice + " Comece pelo treino guiado, repita no modo desafio e use o replay para comparar a outra linha na mesma distribuição.",
               items, cues)


# ─────────────────────────────── Módulo 2 ───────────────────────────────
def m2l1():
    p = "m2l1"
    scenes = [
        intro(f"{p}-01", "C-bet no flop: boards secos e molhados",
              "Esta é a Zero Tilt Academy. Na primeira aula do módulo dois, vamos estudar a c-bet no flop em boards secos e molhados. "
              "O objetivo é ler a textura e os ranges antes de decidir entre apostar e dar check.",
              "Zero Tilt Academy · Módulo 2 · Aula 1", "Objetivo: ler textura e ranges antes de apostar ou dar check"),
        compare(f"{p}-02", "Textura", "Seco ou molhado",
                "Compare dois flops. Ás, sete e dois de naipes diferentes é um board seco: há poucos projetos diretos de sequência ou de flush. "
                "Nove, oito e sete com duas copas é um board molhado: conectado e com duas cartas do mesmo naipe, ele oferece muitos projetos aos dois jogadores. "
                "Mas a textura sozinha não decide a frequência nem o tamanho da c-bet.",
                [("A♠ 7♦ 2♣ · seco", [("Cartas distantes, naipes diferentes", "naipes diferentes"), ("Poucos projetos diretos", "poucos projetos")], "mint"),
                 ("9♥ 8♥ 7♣ · molhado", [("Conectado, com duas copas", "conectado"), ("Projetos para os dois jogadores", "muitos projetos")], "gold")],
                note="A textura sozinha não decide frequência nem tamanho", note_cue="a textura sozinha"),
        steps(f"{p}-03", "Textura", "O que entra na decisão",
              "Antes de apostar, compare os ranges dos dois jogadores, a vantagem de nuts, a posição e o número de jogadores no pote. "
              "A textura é só uma parte desse cenário.",
              ["Os ranges dos dois jogadores", "A vantagem de nuts", "Posição e número de jogadores"],
              ["compare os ranges", "a vantagem de nuts", "a posição"], note="A textura é só uma parte do cenário"),
    ]
    t = Mesa(["Você · BTN", "SB", "BB"], [["Qs", "Js"], ["Kd", "3s"], ["Ac", "4d"]], dealer=0, seats=[0, 1, 2])
    scenes.append(t.scene(f"{p}-04", "Exemplo guiado", "Dama e valete de espadas no botão",
                          "Agora o exemplo guiado, numa mesa de Texas cash com cem big blinds. Os blinds entram, todos desistem até você, no botão, com dama e valete de espadas, "
                          "e você abre para dois big blinds e meio. O jogador do small blind desiste, e quem está no big blind paga. O pote do flop fica com cinco big blinds e meio.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "você abre", player=0, to=250), ev("fold", "small blind desiste", player=1),
                           ev("call", "big blind paga", player=2, to=250), ev("collect", "O pote do flop")],
                          banner="Pré-flop · todos desistiram até você"))
    assert t.total() == 550
    scenes.append(t.scene(f"{p}-05", "Exemplo guiado", "O big blind dá check",
                          "O flop vem nove de copas, oito de copas e sete de paus, e o big blind dá check. Você tem uma broca: só um dez completa a sequência, da dama até o oito. "
                          "Este board conecta com várias mãos de quem pagou, e não existe aposta para pagar, então call é ilegal neste momento. "
                          "No exercício, treinamos o check-back: você vê o turn sem aumentar o pote.",
                          [ev("board", "O flop vem", cards=["9h", "8h", "7c"]), ev("check", "o big blind dá check", player=2),
                           ev("check", "treinamos o check-back", player=0)],
                          banner="Flop · nada a pagar", note="Sem aposta a pagar: check ou aposta"))
    assert t.total() == 550
    scenes += [
        steps(f"{p}-06", "Exemplo guiado", "Broca, não projeto aberto",
              "Atenção à contagem. Com dama e valete em nove, oito e sete, só os quatro dez completam a sequência: é uma broca, e não um projeto aberto com oito outs. "
              "E mesmo quando o dez chega, analise o resto: quem também tiver dama e valete empata, e outras mãos mudam de força conforme os pares e os naipes.",
              ["Só os quatro dez completam", "Broca: não são oito outs", "Mesmo o dez exige análise"],
              ["só os quatro dez", "não um projeto aberto", "mesmo quando o dez chega"]),
        compare(f"{p}-07", "Vantagens", "Vantagem de range e vantagem de nuts",
                "Separe duas ideias. Vantagem de range compara a distribuição de forças dos dois conjuntos de mãos. Vantagem de nuts olha a presença das combinações mais fortes. "
                "As duas podem apontar para lados diferentes, e quem abriu no pré-flop não tem sempre as duas. "
                "Pergunte quais pares, conectores e mãos do mesmo naipe chegam a esse board por cada linha.",
                [("Vantagem de range", [("Distribuição de forças dos dois ranges", "compara a distribuição")], "mint"),
                 ("Vantagem de nuts", [("Presença das combinações mais fortes", "presença das combinações")], "gold")],
                note="Quem abriu não tem sempre as duas", note_cue="quem abriu"),
        metrics(f"{p}-08", "Tamanho", "Tamanho como hipótese",
                "E o tamanho? Em certos boards secos, apostar entre um quarto e um terço do pote é uma referência frequente de estudo, e não uma regra para todo flop. "
                "Num pote de cinco big blinds e meio, isso fica entre um vírgula quatro e um vírgula oito big blinds. Apostar mais também muda quais mãos podem continuar.",
                [("POTE", "5,5 BB"), ("UM QUARTO (25%)", "≈ 1,4 BB"), ("UM TERÇO (33%)", "≈ 1,8 BB")],
                ["cinco big blinds e meio", "um vírgula quatro", "um vírgula oito"], note="Referência de estudo, não regra"),
        compare(f"{p}-09", "Check ou aposta", "Check também é estratégia",
                "Dar check pode preservar equity, proteger a parte do range que também dá check e deixar a decisão para depois. "
                "Apostar cobra dos projetos e pode gerar folds, mas expõe a mão a um check-raise. "
                "No simulador, compare o tamanho do pote que chega ao turn em cada linha. Nenhum tamanho de c-bet é uma solução universal.",
                [("Check", [("Preserva equity", "preservar equity"), ("Protege o range de check", "proteger a parte"), ("Decide depois", "deixar a decisão")], "mint"),
                 ("Aposta", [("Cobra dos projetos", "cobra dos projetos"), ("Pode gerar folds", "pode gerar folds"), ("Expõe a check-raise", "expõe a mão")], "gold")],
                note="Compare o pote que chega ao turn", note_cue="compare o tamanho"),
    ]
    assert round(550 / 4) == 138 and round(550 / 3) == 183  # 1,4 e 1,8 BB
    scenes += quiz(p, "Qual destes flops é o mais seco?",
                   [("A", "J♥ 10♥ 8♠", "valete"), ("B", "K♣ 7♦ 2♠", "rei"), ("C", "7♠ 6♠ 5♦", "sete")], "B",
                   "Exercício. Qual destes três flops é o mais seco? A: valete de copas, dez de copas e oito de espadas. "
                   "B: rei de paus, sete de ouros e dois de espadas. C: sete de espadas, seis de espadas e cinco de ouros. Pause o vídeo e responda.",
                   "Resposta: B. Rei, sete e dois de naipes diferentes não têm duas cartas do mesmo naipe nem cartas próximas, então quase não há projetos diretos. "
                   "Os outros dois flops são conectados e têm duas cartas de um mesmo naipe.",
                   "Naipes diferentes e cartas distantes: quase nenhum projeto")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. No board conectado, reconheça a broca antes de apostar ou dar check: um dez completa a sequência, "
                   "e o flop oferece várias mãos fortes a quem pagou. Ninguém é obrigado a fazer a c-bet.",
                   ["Reconheça a broca antes de agir", "Nenhuma c-bet é obrigatória", "Guiado, desafio e replay"],
                   ["reconheça a broca", "Ninguém é obrigado", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m2l1 — C-bet no flop: boards secos e molhados", scenes)


def m2l2():
    p = "m2l2"
    scenes = [
        intro(f"{p}-01", "Check-raise e float",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo dois, vamos estudar como responder a um check-raise e quando um float tem plano. "
              "O objetivo é refazer o preço depois do aumento e decidir com um modelo explícito do rival.",
              "Zero Tilt Academy · Módulo 2 · Aula 2", "Objetivo: refazer o preço após o raise e decidir com um modelo do rival"),
        steps(f"{p}-02", "Check-raise", "O que cabe num check-raise",
              "Um check-raise pode conter mãos de valor e blefes. Antes de responder, compare o total a pagar, os stacks, a textura e o range plausível do rival. "
              "Por isso, ter top pair não obriga você a pagar sempre, nem a desistir sempre.",
              ["Valor e blefes", "Total a pagar, stacks, textura e range", "Top pair: nem sempre paga, nem sempre desiste"],
              ["mãos de valor e blefes", "compare o total", "ter top pair"]),
    ]
    t = Mesa(["Você · BTN", "BB"], [["Kd", "9d"], ["8h", "8d"]], stacks=[9500, 9500], dealer=0, seats=[0, 2], pot=1000)
    scenes.append(t.scene(f"{p}-03", "Check-raise", "O rival aumenta",
                          "Veja um exemplo. O pote tem dez big blinds, e o flop traz rei de paus, oito de espadas e três de copas. "
                          "Você está no botão com rei e nove de ouros, um top pair. Quem está no big blind dá check, e você aposta três, menos de um terço do pote. Ele aumenta para doze.",
                          [ev("board", "o flop traz", cards=["Kc", "8s", "3h"]), ev("check", "dá check", player=1),
                           ev("bet", "você aposta", player=0, to=300), ev("raise", "Ele aumenta", player=1, to=1200)],
                          banner="Flop · pote de 10 BB"))
    assert t.total() == 2500
    to_call = t.bets[1] - t.bets[0]
    assert to_call == 900 and abs(to_call / (t.total() + to_call) - 0.2647) < 0.0001
    scenes.append(metrics(f"{p}-04", "O preço depois do raise", "Refaça o preço",
                          "Agora refaça o preço. Antes da sua decisão, há vinte e cinco big blinds no centro: dez do pote, três seus e doze do rival. "
                          "Você já colocou três, então precisa acrescentar nove, e não doze. Depois do call, o pote terá trinta e quatro. "
                          "O limiar é nove sobre trinta e quatro, cerca de vinte e seis e meio por cento, sem taxas e supondo que o call encerre as apostas.",
                          [("A ACRESCENTAR", "9 BB"), ("POTE APÓS O CALL", "34 BB"), ("LIMIAR · 9/34", "≈ 26,47%")],
                          ["acrescentar nove", "o pote terá trinta e quatro", "nove sobre trinta e quatro"],
                          note="O custo é o que falta, não o total do raise"))
    scenes.append(t.scene(f"{p}-05", "Modelo do rival", "Decida pelo modelo",
                          "Agora o modelo do exercício: este rival só dá check-raise grande com dois pares ou melhor, e quase nunca blefa. "
                          "Contra esse range, rei e nove perde para quase todas as mãos que aumentam, e a decisão conservadora é desistir. "
                          "Se as frequências mudarem, refaça a análise.",
                          [ev("fold", "a decisão conservadora é desistir", player=0), ev("collect", None), ev("award", None, player=1)],
                          banner="Modelo: check-raise grande = dois pares ou melhor", note="Frequências mudaram? Refaça a análise"))
    scenes += [
        compare(f"{p}-06", "Modelo do rival", "Força absoluta e força relativa",
                "Top pair pode ser forte contra um range amplo, com muitas mãos piores que pagam. Mas fica fraca contra um range só de dois pares, trincas e mãos melhores. "
                "No treino, o bot de valor só aumenta quando a mão dele cumpre a política declarada, e é isso que torna a leitura estudável. "
                "Contra um rival que também blefa e aumenta com projetos, você teria de reconstruir esse range.",
                [("Contra um range amplo", [("Top pair é forte", "range amplo"), ("Muitas mãos piores pagam", "muitas mãos piores")], "mint"),
                 ("Contra dois pares ou melhor", [("Top pair é fraca", "fica fraca"), ("Quase nada pior continua", "trincas")], "danger")],
                note="Rival que blefa? Reconstrua o range", note_cue="reconstruir esse range"),
        steps(f"{p}-07", "Float", "O que é um float",
              "Agora vamos ao float, uma ideia diferente. Float é pagar com o plano de disputar o pote numa rodada seguinte, muitas vezes em posição. "
              "Ele exige equity, preço e leitura do que o rival faz depois. Não é pagar qualquer mão esperando um check.",
              ["Pagar com plano para uma rodada seguinte", "Exige equity, preço e leitura", "Não é pagar esperando um check"],
              ["disputar o pote", "exige equity", "Não é pagar"]),
        steps(f"{p}-08", "Float", "Um float precisa de plano",
              "Pagar uma aposta pequena em posição pode preservar opções, quando a sua mão tem equity e o rival abandona certas linhas depois. "
              "Mas você não sabe se ele vai dar check no turn. Então registre quais cartas permitem continuar e qual nova aposta tornaria o plano ruim. "
              "Um call só por curiosidade não é float.",
              ["Quais cartas permitem continuar", "Que aposta tornaria o plano ruim", "Call por curiosidade não é float"],
              ["quais cartas permitem", "qual nova aposta", "só por curiosidade"]),
    ]
    assert abs(10 / 36 - 0.2778) < 0.0001 and round(100 * 10 / 26) == 38 and round(100 * 14 / 36) == 39
    scenes += quiz(p, "Pote de 8 BB. Você aposta 4 BB e o rival aumenta para 14 BB. Qual é o limiar de equity para pagar?",
                   [("A", "10/26 ≈ 38%", "dez sobre vinte e seis"), ("B", "10/36 ≈ 28%", "dez sobre trinta e seis"), ("C", "14/36 ≈ 39%", "catorze")], "B",
                   "Exercício. O pote tem oito big blinds. Você aposta quatro, e o rival aumenta para catorze. "
                   "Sem taxas e com o call encerrando as apostas, qual é o limiar de equity para pagar? "
                   "A: dez sobre vinte e seis. B: dez sobre trinta e seis. C: catorze sobre trinta e seis. Pause o vídeo e responda.",
                   "Resposta: B. Antes da decisão, há vinte e seis no centro: oito do pote, quatro seus e catorze do rival. "
                   "Você acrescenta dez, e o pote depois do call fica com trinta e seis. Dez sobre trinta e seis dá cerca de vinte e oito por cento.",
                   "Acrescentar 10 · pote após o call 36 · 10/36 ≈ 27,8%")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. O bot de valor dá check-raise com dois pares ou melhor, e uma aposta sua pode provocar essa resposta. "
                   "Recalcule o preço depois da agressão.",
                   ["O bot de valor: dois pares ou melhor", "Recalcule o preço após o raise", "Guiado, desafio e replay"],
                   ["O bot de valor", "Recalcule o preço", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m2l2 — Como responder a check-raise e o float", scenes)


def m2l3():
    p = "m2l3"
    hit = 1 - math.comb(48, 3) / math.comb(50, 3)
    assert abs(hit - 0.1176) < 0.0001 and abs((1 - hit) / hit - 7.5) < 0.01 and abs(1 / hit - 8.5) < 0.01
    scenes = [
        intro(f"{p}-01", "Potes multiway e set mining",
              "Esta é a Zero Tilt Academy. Na aula três do módulo dois, vamos estudar os potes com vários jogadores, os potes multiway, e o set mining. "
              "O objetivo é buscar valor e escolher blefes sem esquecer os jogadores a mais na mão.",
              "Zero Tilt Academy · Módulo 2 · Aula 3", "Objetivo: buscar valor sem ignorar os jogadores a mais"),
        steps(f"{p}-02", "Potes multiway", "O blefe precisa passar por todos",
              "Num pote multiway, o blefe só ganha na hora se todos os adversários que ainda disputam desistirem. "
              "As frequências de defesa e o valor das mãos mudam com mais jogadores. Mesmo assim, não presuma fold equity zero, nem copie uma estratégia de dois jogadores.",
              ["O blefe exige que todos desistam", "Defesa e valor mudam", "Nem fold equity zero, nem cópia do heads-up"],
              ["todos os adversários", "frequências de defesa", "não presuma"]),
        metrics(f"{p}-03", "Potes multiway", "Um modelo artificial",
                "Um modelo artificial ajuda a ver o efeito. Se um adversário desiste metade das vezes, o blefe passa metade das vezes. "
                "Com dois adversários, cada um desistindo metade das vezes e de forma independente, a chance conjunta cai para um quarto. "
                "Mas, no pôquer, as decisões podem ser ligadas: o segundo jogador vê o que o primeiro fez. O produto ilustra uma hipótese, e não uma fórmula.",
                [("UM ADVERSÁRIO", "50%"), ("DOIS · ½ × ½", "25%"), ("DECISÕES LIGADAS", "Hipótese")],
                ["passa metade das vezes", "um quarto", "o segundo jogador"], note="Ilustra uma hipótese, não uma fórmula"),
        metrics(f"{p}-04", "Set mining", "A chance de acertar a trinca",
                "Agora vamos ao set mining, a busca pela trinca. Com um par na mão, e só as suas duas cartas conhecidas, restam duas cartas do seu valor entre cinquenta. "
                "A chance de o flop trazer pelo menos uma delas é de cerca de onze vírgula oito por cento. Mais ou menos um flop a cada oito e meio.",
                [("CARTAS DO SEU VALOR", "2 de 50"), ("FLOP COM PELO MENOS UMA", "≈ 11,76%"), ("UM FLOP A CADA", "≈ 8,5")],
                ["restam duas cartas", "pelo menos uma delas", "a cada oito e meio"], note="1 − C(48,3) / C(50,3)"),
        metrics(f"{p}-05", "Set mining", "Quanto precisa ganhar",
                "Quanto você precisa ganhar quando acerta? Num modelo simples, você paga um big blind só para buscar a trinca, nunca ganha quando erra e não tem custos depois. "
                "Errando em oitenta e oito por cento das vezes, o ganho líquido médio nos acertos precisa passar de uns sete big blinds e meio só para empatar. "
                "O modelo ignora as derrotas depois de acertar, o rake e as ações futuras. Um stack grande do rival, sozinho, não garante esse pagamento.",
                [("CUSTO", "1 BB"), ("ACERTO", "≈ 11,76%"), ("GANHO NOS ACERTOS", "> 7,5 BB")],
                ["paga um big blind", "oitenta e oito por cento", "sete big blinds e meio"], note="Um stack grande não garante pagamento"),
        steps(f"{p}-06", "Set mining", "Uma heurística, não uma prova",
              "A regra de pagar com vinte vezes o valor do call é uma heurística, e não uma prova de rentabilidade. Acertar a trinca não garante vitória nem pagamento. "
              "O stack efetivo, a posição, o rake e o risco de uma trinca maior entram na decisão.",
              ["Vinte vezes o call: heurística", "Acertar não garante pagamento", "Stack, posição, rake e trinca maior"],
              ["vinte vezes", "Acertar a trinca", "O stack efetivo"]),
    ]
    t = Mesa(["Você · BTN", "SB", "BB"], [["8c", "8d"], ["9s", "9h"], ["6h", "5h"]], dealer=0, seats=[0, 1, 2])
    scenes.append(t.scene(f"{p}-07", "Exemplo guiado", "Top set com três jogadores",
                          "Agora um exemplo com três jogadores. Os blinds entram, e você abre no botão com um par de oitos, para dois big blinds e meio. Os dois blinds pagam, "
                          "e o pote do flop tem sete big blinds e meio. O flop vem oito de copas, sete de espadas e dois de paus: você tem top set, a maior trinca possível neste flop. "
                          "Depois do flop, os dois jogadores dos blinds dão check.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "você abre", player=0, to=250), ev("call", "Os dois blinds pagam", player=1, to=250),
                           ev("call", None, player=2, to=250), ev("collect", "o pote do flop"),
                           ev("board", "O flop vem", cards=["8h", "7s", "2c"]), ev("check", "dão check", player=1),
                           ev("check", None, player=2)],
                          banner="Pote multiway · três jogadores"))
    assert t.total() == 750
    scenes.append(t.scene(f"{p}-08", "Exemplo guiado", "Aposta por valor",
                          "Com os dois checks, treinamos a aposta por valor contra adversários que pagam com pares e projetos. Aqui você aposta cinco big blinds, dois terços do pote. "
                          "Antes de escolher o tamanho, pergunte quais pares e projetos pagariam cada aposta, e quantas cartas do turn podem mudar a força relativa.",
                          [ev("bet", "você aposta cinco", player=0, to=500)],
                          banner="Flop · dois checks até você", note="Quais pares e projetos pagam este tamanho?"))
    scenes += quiz(p, "Modelo artificial: você blefa contra três adversários, e cada um desiste metade das vezes, de forma independente. Qual é a chance de todos desistirem?",
                   [("A", "Metade", "metade"), ("B", "Um terço", "um terço"), ("C", "Um oitavo", "um oitavo")], "C",
                   "Exercício. Num modelo artificial, você blefa contra três adversários, e cada um desiste metade das vezes, de forma independente. "
                   "Qual é a chance de todos desistirem? A: metade. B: um terço. C: um oitavo. Pause o vídeo e responda.",
                   "Resposta: C, um oitavo. Metade vezes metade vezes metade dá um oitavo, doze e meio por cento. "
                   "Na mesa de verdade, as decisões podem ser ligadas, então use esse número como hipótese, e não como regra.",
                   "½ × ½ × ½ = 1/8 = 12,5%")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino, com top set contra dois bots. A sua trinca é a maior possível neste flop: "
                   "pense em quais pares e projetos pagariam cada tamanho.",
                   ["Top set contra dois bots", "Quem paga cada tamanho?", "Guiado, desafio e replay"],
                   ["top set contra dois bots", "pense em quais pares", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m2l3 — Potes multiway e set mining", scenes)


# ─────────────────────────────── Módulo 3 ───────────────────────────────
def m3l1():
    p = "m3l1"
    scenes = [
        intro(f"{p}-01", "Double barrel no turn",
              "Esta é a Zero Tilt Academy. Na primeira aula do módulo três, vamos estudar o segundo barril: a aposta no turn depois de apostar no flop. "
              "O objetivo é atualizar a força relativa da sua mão quando chega uma carta nova e planejar o river.",
              "Zero Tilt Academy · Módulo 3 · Aula 1", "Objetivo: atualizar a força relativa e planejar o river"),
        steps(f"{p}-02", "O segundo barril", "Uma aposta precisa de razão",
              "Uma aposta no turn, depois de apostar no flop, precisa de uma razão. Ela pode receber de mãos piores, fazer mãos melhores desistirem, "
              "ou somar fold equity à chance de melhorar. A iniciativa, sozinha, não justifica a segunda aposta.",
              ["Receber de mãos piores", "Fazer mãos melhores desistirem", "Fold equity mais chance de melhorar"],
              ["receber de mãos piores", "fazer mãos melhores", "somar fold equity"], note="A iniciativa sozinha não justifica"),
    ]
    t = Mesa(["Você · BTN", "SB", "BB"], [["As", "Qd"], ["Jc", "4d"], ["8h", "7h"]], dealer=0, seats=[0, 1, 2])
    scenes.append(t.scene(f"{p}-03", "Exemplo guiado", "Top pair no flop",
                          "Veja o exemplo. Os blinds entram, e você abre no botão com ás de espadas e dama de ouros, para dois big blinds e meio. O jogador do small blind desiste, e quem está no big blind paga. "
                          "O flop vem dama de espadas, oito de paus e três de copas. Quem está no big blind dá check, você aposta dois com top pair, e ele paga.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "você abre", player=0, to=250), ev("fold", "small blind desiste", player=1),
                           ev("call", "big blind paga", player=2, to=250), ev("collect", None),
                           ev("board", "O flop vem", cards=["Qs", "8c", "3h"]), ev("check", "big blind dá check", player=2),
                           ev("bet", "você aposta dois", player=0, to=200), ev("call", "e ele paga", player=2, to=200), ev("collect", None)],
                          banner="Flop · c-bet paga"))
    assert t.total() == 950
    scenes.append(t.scene(f"{p}-04", "Exemplo guiado", "Chega um rei",
                          "No turn chega o rei de ouros, e o big blind dá check de novo. A sua mão mudou: com o rei na mesa, a dama deixou de ser o par mais alto, "
                          "e você passou de top pair a segundo par. Se o plano declarado é controlar o pote e preservar o valor de showdown, o check-back cumpre esse plano.",
                          [ev("board", "chega o rei", cards=["Kd"]), ev("check", "dá check de novo", player=2),
                           ev("check", "o check-back cumpre", player=0)],
                          banner="Turn · K♦", note="Top pair virou segundo par"))
    scenes += [
        compare(f"{p}-05", "Leitura do turn", "O rei favorece quem?",
                "Um rei pode ajudar parte do range de quem abriu, como ás e rei. Mas também melhora quem pagou: rei e dama vira dois pares, "
                "e outros reis que pagaram o flop passam na frente da sua dama. Por isso, não trate toda carta alta como uma carta que assusta automaticamente.",
                [("Quem abriu (botão)", [("Alguns reis fortes, como AK", "como ás e rei")], "mint"),
                 ("Quem pagou (big blind)", [("KQ vira dois pares", "rei e dama vira"), ("Outros reis passam na frente", "outros reis")], "gold")],
                note="Nenhuma carta assusta automaticamente", note_cue="não trate toda carta"),
        steps(f"{p}-06", "Leitura do turn", "Atualize os ranges, não só o board",
              "Depois de uma c-bet paga, o range de quem pagou já não é o do pré-flop: parte das mãos desistiu, e parte continuou. "
              "A carta do turn age sobre essa distribuição filtrada. A história da iniciativa não substitui a contagem das mãos plausíveis.",
              ["Parte desistiu, parte continuou", "O turn age sobre o range filtrado", "Conte as mãos plausíveis"],
              ["parte das mãos desistiu", "distribuição filtrada", "não substitui"]),
        steps(f"{p}-07", "Valor, blefe ou proteção", "Diga quem paga e quem desiste",
              "Antes de apostar, diga quais mãos piores pagam e quais melhores desistem. Com ás e dama neste turn, algumas damas piores e alguns projetos podem pagar, "
              "mas quase nenhuma mão melhor desiste. Negar equity aos projetos pode fazer parte da razão, mas compare isso com o risco de enfrentar um raise e com o valor do check. "
              "Apostar aqui não vira, automaticamente, um segundo barril polarizado.",
              ["Quais piores pagam?", "Quais melhores desistem?", "Proteção: compare com o risco de raise"],
              ["quais mãos piores pagam", "quais melhores desistem", "Negar equity"]),
        steps(f"{p}-08", "Plano para o river", "Planeje dois tipos de river",
              "Se escolher apostar no turn, descreva antes o que fará num river favorável e num river que complete um projeto do rival. "
              "Use os stacks e o pote para saber quais tamanhos ainda cabem. No replay, compare o que você sabia antes do river com o que veio depois, "
              "sem usar o resultado para reescrever a hipótese.",
              ["River favorável: qual plano?", "River que completa projeto: qual plano?", "Não reescreva a hipótese pelo resultado"],
              ["num river favorável", "complete um projeto", "No replay"]),
    ]
    board = ["Qs", "8c", "3h", "Kd"]
    for hand, improved in ((["8h", "7h"], False), (["Kc", "Qh"], True), (["Ac", "5c"], False)):
        assert (best(hand, board)[0] > best(hand, board[:3])[0]) == improved, hand  # categoria da mão subiu
    scenes += quiz(p, "Flop Q♠ 8♣ 3♥, turn K♦. Qual destas mãos de quem pagou melhorou com o rei?",
                   [("A", "8♥ 7♥", "oito"), ("B", "K♣ Q♥", "rei"), ("C", "A♣ 5♣", "ás")], "B",
                   "Exercício. O flop foi dama, oito e três, e o turn trouxe o rei de ouros. Qual destas mãos de quem pagou melhorou com o rei? "
                   "A: oito e sete de copas. B: rei de paus e dama de copas. C: ás e cinco de paus. Pause o vídeo e responda.",
                   "Resposta: B. Rei e dama tinha top pair no flop e virou dois pares no turn. Oito e sete continua com o mesmo par de oitos, e ás e cinco continua sem par. "
                   "O rei também melhora o range de quem pagou.",
                   "KQ: top pair no flop, dois pares no turn", board=board)
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Ás e dama passou a segundo par: o rei no turn não obriga você a apostar, nem acaba com o valor de showdown.",
                   ["AQ virou segundo par", "O rei não obriga a apostar", "Guiado, desafio e replay"],
                   ["passou a segundo par", "não obriga você", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m3l1 — Double barrel: cartas que assustam e tamanho no turn", scenes)


def m3l2():
    p = "m3l2"
    scenes = [
        intro(f"{p}-01", "Pot control e apostas polarizadas",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo três, vamos comparar o controle do pote com as apostas polarizadas. "
              "O objetivo é decidir entre check-back e aposta com uma mão intermediária, num board de três paus.",
              "Zero Tilt Academy · Módulo 3 · Aula 2", "Objetivo: comparar check-back e aposta com mão intermediária"),
        steps(f"{p}-02", "Pot control", "O que o check preserva",
              "Dar check pode preservar mãos de força intermediária e evita aumentar a exposição naquela rodada. "
              "Mas o check não garante um showdown barato: o rival ainda pode apostar depois.",
              ["Preserva mãos intermediárias", "Evita aumentar a exposição", "Não garante showdown barato"],
              ["força intermediária", "a exposição", "não garante"]),
        compare(f"{p}-03", "Polarização", "Polarizar é compor o range",
                "Um range polarizado reúne mãos fortes, que apostam por valor, e blefes. Ele não se define por apostar setenta e cinco por cento do pote ou mais: "
                "uma aposta grande só com mãos médias não fica polarizada por causa do tamanho. Tudo depende da composição do range e do contexto.",
                [("Polarizado", [("Mãos fortes por valor", "mãos fortes"), ("E blefes", "e blefes")], "mint"),
                 ("Não basta", [("Apostar 75% ou mais", "setenta e cinco por cento"), ("Aposta grande só com mãos médias", "só com mãos médias")], "danger")],
                note="Composição e contexto, não só tamanho", note_cue="Tudo depende"),
    ]
    t = Mesa(["Você · BTN", "SB", "BB"], [["Jh", "Td"], ["Kd", "2s"], ["As", "8s"]], dealer=0, seats=[0, 1, 2])
    scenes.append(t.scene(f"{p}-04", "Exercício guiado", "Top pair no flop",
                          "Veja o exercício. Os blinds entram, e você abre no botão com valete de copas e dez de ouros, para dois big blinds e meio. O jogador do small blind desiste, e quem está no big blind paga. "
                          "No flop, valete de paus, oito de ouros e quatro de paus, o big blind dá check, você aposta dois big blinds com top pair, e ele paga.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "você abre", player=0, to=250), ev("fold", "small blind desiste", player=1),
                           ev("call", "big blind paga", player=2, to=250), ev("collect", None),
                           ev("board", "No flop", cards=["Jc", "8d", "4c"]), ev("check", "o big blind dá check", player=2),
                           ev("bet", "você aposta dois", player=0, to=200), ev("call", "e ele paga", player=2, to=200), ev("collect", None)],
                          banner="Flop · c-bet paga"))
    assert t.total() == 950 and t.stacks[0] == t.stacks[2] == 9550
    scenes.append(t.scene(f"{p}-05", "Exercício guiado", "Chega o terceiro paus",
                          "No turn chega o nove de paus. Agora você tem top pair e um projeto aberto de sequência: um sete ou uma dama completam. "
                          "Mas a mesa tem três paus, e um rival com dois paus já tem flush. Depois disso, o big blind dá check de novo. "
                          "No exercício, o plano é controlar o pote com check-back. Call não existe aqui, porque não há aposta para pagar.",
                          [ev("board", "chega o nove", cards=["9c"]), ev("check", "dá check de novo", player=2),
                           ev("check", "com check-back", player=0)],
                          banner="Turn · 9♣ · três paus na mesa", note="Projeto aberto: 7 ou Q completam"))
    scenes += [
        steps(f"{p}-06", "Exercício guiado", "Classifique antes de decidir",
              "Antes de decidir, classifique a mão. Top pair e projeto aberto dão uma força intermediária. Os três paus põem flushes possíveis no range do rival. "
              "Apostar pode cobrar dos projetos, mas expõe a mão a um check-raise; o check-back preserva o showdown e mantém o pote menor. Compare a exposição criada por cada tamanho.",
              ["Top pair e projeto aberto: força média", "Três paus: flushes possíveis", "Check-back preserva o showdown"],
              ["classifique a mão", "Os três paus", "o check-back preserva"]),
        metrics(f"{p}-07", "Stack e pote", "SPR: stack sobre pote",
                "Uma medida ajuda a planejar: o SPR, que é o stack efetivo dividido pelo pote. Neste turn, restam noventa e cinco big blinds e meio, "
                "e o pote tem nove e meio: o SPR fica perto de dez. Esse número mostra quantas apostas grandes ainda cabem, mas não manda ir all-in nem proíbe nada. "
                "A mesma relação pede estratégias diferentes conforme o board, a modalidade, a posição e as forças.",
                [("STACK EFETIVO", "95,5 BB"), ("POTE NO TURN", "9,5 BB"), ("SPR", "≈ 10")],
                ["noventa e cinco", "nove e meio", "perto de dez"], note="O SPR orienta, não ordena"),
        steps(f"{p}-08", "Leitura do check", "O check também tem mãos fortes",
              "Cuidado com a leitura do check do rival. Um range de check pode ter mãos fracas, médias e fortes. "
              "Se você presumir que todo check permite blefar sem resistência, vai ficar exposto a calls e raises. "
              "O bot de valor do treino mostra uma dessas possibilidades: ele pode dar check com mão forte e reagir à sua aposta.",
              ["Fracas, médias e fortes", "Nem todo check é desistência", "O bot de valor pode reagir"],
              ["mãos fracas", "todo check", "O bot de valor"]),
    ]
    assert round(9550 / 950) == 10
    hole, board = ["Jh", "Td"], ["Jc", "8d", "4c", "9c"]
    outs = [c for c in deck(hole + board) if best(hole, board + [c])[0] == 4]
    assert sorted(outs) == sorted(r + s for r in "Q7" for s in "cdhs"), outs
    assert len([c for c in outs if c[1] != "c"]) == 6
    scenes += quiz(p, "Board J♣ 8♦ 4♣ 9♣, você tem J♥ 10♦. Quantas cartas do river completam a sua sequência sem pôr o quarto paus na mesa?",
                   [("A", "Oito", "oito"), ("B", "Seis", "seis"), ("C", "Quatro", "quatro")], "B",
                   "Exercício. Com valete de copas e dez de ouros, num board de valete de paus, oito de ouros, quatro de paus e nove de paus, "
                   "quantas cartas do river completam a sua sequência sem colocar o quarto paus na mesa? A: oito. B: seis. C: quatro. Pause o vídeo e responda.",
                   "Resposta: B, seis. Um sete ou uma dama completam a sequência: oito cartas. Tirando o sete e a dama de paus, que põem o quarto paus na mesa, sobram seis. "
                   "E lembre: um rival com dois paus já tem flush agora.",
                   "7 ou Q: 8 cartas · sem 7♣ e Q♣: 6", hole=hole, board=board)
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino e compare check-back e aposta num board de três paus. "
                   "Top pair e um projeto aberto de sequência convivem com flushes possíveis, e a composição do range importa mais que o nome da mão.",
                   ["Board de três paus", "A composição importa mais que o nome", "Guiado, desafio e replay"],
                   ["três paus", "a composição do range", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m3l2 — Pot control e apostas polarizadas", scenes)


def m3l3():
    p = "m3l3"
    scenes = [
        intro(f"{p}-01", "Draws, outs e pot odds no turn",
              "Esta é a Zero Tilt Academy. Na aula três do módulo três, vamos estudar projetos, outs e pot odds no turn. "
              "O objetivo é calcular o limiar do call e separar a chance de completar um flush da chance de vencer.",
              "Zero Tilt Academy · Módulo 3 · Aula 3", "Objetivo: calcular o limiar do call e separar completar de vencer"),
    ]
    t = Mesa(["Você · BTN", "BB"], [["7h", "6h"], ["Kc", "Td"]], stacks=[9400, 9400], dealer=0, seats=[0, 2], board=["Kh", "9h", "4c"], pot=1200)
    scenes.append(t.scene(f"{p}-02", "Exemplo guiado", "Projeto de flush no turn",
                          "Veja agora uma decisão tomada no turn. O pote tem doze big blinds, e a mesa mostra rei de copas, nove de copas e quatro de paus. "
                          "Você tem sete e seis de copas: um projeto de flush. O turn traz o dois de espadas, e o rival aposta três big blinds.",
                          [ev("board", "O turn traz", cards=["2s"]), ev("bet", "o rival aposta", player=1, to=300)],
                          banner="Turn · pote de 12 BB"))
    assert t.total() == 1500
    hearts = [c for c in deck(t.cards[0] + t.board) if c[1] == "h"]
    assert len(deck(t.cards[0] + t.board)) == 46 and len(hearts) == 9
    scenes += [
        metrics(f"{p}-03", "O preço do call", "Limiar: C sobre P mais C",
                "Agora o preço. Chame de P o pote que já inclui a aposta do rival: quinze big blinds. Pagar custa C, que aqui são três big blinds. "
                "Sem rake, o limiar é C sobre P mais C: três sobre dezoito, cerca de dezesseis vírgula sete por cento. O denominador inclui o seu próprio call.",
                [("P · POTE COM A APOSTA", "15 BB"), ("C · CUSTO DO CALL", "3 BB"), ("LIMIAR · 3/18", "≈ 16,67%")],
                ["quinze big blinds", "três big blinds", "três sobre dezoito"], note="O denominador inclui o seu call"),
        metrics(f"{p}-04", "Outs", "Nove copas em quarenta e seis",
                "Quantas copas ainda podem vir? Você conhece seis cartas: as suas duas e as quatro da mesa. Restam quarenta e seis desconhecidas. "
                "Com quatro copas à vista, sobram nove. Então a chance de uma copa no river é nove sobre quarenta e seis, cerca de dezenove vírgula seis por cento, numa distribuição uniforme.",
                [("DESCONHECIDAS · 52 − 6", "46"), ("COPAS RESTANTES · 13 − 4", "9"), ("COPA NO RIVER · 9/46", "≈ 19,57%")],
                ["quarenta e seis desconhecidas", "sobram nove", "nove sobre quarenta e seis"]),
        compare(f"{p}-05", "Outs", "Completar não é vencer",
                "Mas completar o flush e ter equity são medidas diferentes. Algumas copas podem não vencer: num board pareado, a carta que completa o seu flush pode dar full house ao rival. "
                "E, em outros cenários, cartas fora do projeto também podem dar a melhor mão. Por isso, a equity não é automaticamente a chance de acertar o projeto.",
                [("Outs contaminados", [("Seu flush, full house do rival", "full house")], "danger"),
                 ("Outras vitórias", [("Cartas fora do projeto também ganham", "cartas fora do projeto")], "mint")],
                note="Equity não é a chance de completar", note_cue="a equity não é"),
        steps(f"{p}-06", "Regra de bolso", "Outs vezes dois, outs vezes quatro",
              "Há uma regra de bolso: outs vezes dois para uma carta, e outs vezes quatro para duas cartas. "
              "Com nove outs no turn, dá dezoito por cento, perto dos dezenove vírgula seis da conta exata. "
              "Mas ver só o flop não garante chegar ao river pelo preço de uma aposta. Considere apostas futuras, rake, posição e outs contaminados.",
              ["Uma carta: outs × 2", "Duas cartas: outs × 4", "Ver o flop não garante o river"],
              ["outs vezes dois", "outs vezes quatro", "ver só o flop"]),
        metrics(f"{p}-07", "Valor esperado", "Valor esperado com premissas",
                "A conta do valor esperado mostra o papel da equity. Suponha, só como premissa, que nada mais seja apostado no river e que a sua equity seja de vinte por cento: "
                "zero vírgula dois vezes dezoito, menos três, dá mais zero vírgula seis big blind. Com quinze por cento, o resultado seria menos zero vírgula três. "
                "Repare que o limiar, de dezesseis vírgula sete por cento, fica entre os dois. Esses números são premissas para exercitar a fórmula, e não a equity real das cartas da mesa.",
                [("EQUITY 20% · 0,2 × 18 − 3", "+0,6 BB"), ("EQUITY 15% · 0,15 × 18 − 3", "−0,3 BB"), ("LIMIAR · 3/18", "≈ 16,67%")],
                ["vinte por cento", "quinze por cento", "Repare que o limiar"], note="Premissas, não a equity das cartas da mesa"),
        steps(f"{p}-08", "Cartas reveladas", "Revise o evento, não só o denominador",
              "E se duas cartas do rival forem reveladas, como num all-in? O denominador cai de quarenta e seis para quarenta e quatro. "
              "Mas também retire do numerador qualquer copa que apareceu entre elas. Não reduza o denominador e esqueça de revisar o evento.",
              ["Duas reveladas: 44 desconhecidas", "Copa revelada sai do numerador", "Revise o evento inteiro"],
              ["quarenta e quatro", "retire do numerador", "Não reduza"]),
    ]
    assert abs(0.2 * 18 - 3 - 0.6) < 1e-9 and abs(0.15 * 18 - 3 + 0.3) < 1e-9
    two = 1 - (38 / 47) * (37 / 46)
    assert abs(two - 0.3497) < 0.0001
    scenes += quiz(p, "Ainda no flop, você tem nove outs para flush. Pela regra de bolso, qual é a chance aproximada de completar até o river, vendo as duas cartas?",
                   [("A", "Cerca de 18%", "cerca de dezoito"), ("B", "Cerca de 36%", "cerca de trinta e seis"), ("C", "Cerca de 9%", "cerca de nove")], "B",
                   "Exercício. Ainda no flop, você tem nove outs para flush. Pela regra de bolso, qual é a chance aproximada de completar até o river, vendo as duas cartas? "
                   "A: cerca de dezoito por cento. B: cerca de trinta e seis por cento. C: cerca de nove por cento. Pause o vídeo e responda.",
                   "Resposta: B. Nove vezes quatro dá trinta e seis por cento. A conta exata, com quarenta e sete cartas desconhecidas no flop, dá cerca de trinta e cinco. "
                   "E ela só vale se você vir as duas cartas pelo preço de agora, o que nenhuma aposta garante.",
                   "9 × 4 = 36% · conta exata ≈ 35%")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Com quatro copas conhecidas, restam nove em quarenta e seis cartas desconhecidas: "
                   "isso descreve completar uma copa, e não uma equity garantida. Calcule o limiar do call antes de decidir.",
                   ["Nove copas em 46 cartas", "Completar não é vencer", "Calcule o limiar do call"],
                   ["restam nove", "não uma equity garantida", "Calcule o limiar"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m3l3 — Projetos, outs e pot odds no turn", scenes)


# ─────────────────────────────── Módulo 4 ───────────────────────────────
def m4l1():
    p = "m4l1"
    scenes = [
        intro(f"{p}-01", "Thin value bet e blefe puro",
              "Esta é a Zero Tilt Academy. Na aula um do módulo quatro, vamos estudar a aposta por valor no river, inclusive a de margem fina. "
              "O objetivo é medir a aposta contra as mãos que continuam, e não contra todas as mãos do rival.",
              "Zero Tilt Academy · Módulo 4 · Aula 1", "Objetivo: medir a aposta contra as mãos que continuam"),
        steps(f"{p}-02", "Valor no river", "Quem paga é o que importa",
              "Uma aposta por valor busca ser paga por mãos piores. Quando a margem contra as mãos que pagam é pequena, chamamos de thin value. "
              "Ser forte contra o range inteiro do rival não basta: compare a sua mão com a parte que vai continuar.",
              ["Valor: ser pago por mãos piores", "Thin value: margem pequena", "Compare com quem continua"],
              ["ser paga por mãos piores", "chamamos de thin value", "compare a sua mão"]),
    ]
    t = Mesa(["Você · BTN", "BB"], [["Ad", "Jd"], ["Ah", "5c"]], stacks=[9400, 9400], dealer=0, seats=[0, 2],
             board=["As", "9c", "4d", "2h", "7s"], pot=1200)
    scenes.append(t.scene(f"{p}-03", "Exemplo com hipótese", "Um pagador com ás menor",
                          "Veja um exemplo. No river, a mesa mostra ás, nove, quatro, dois e sete, sem três cartas do mesmo naipe, e o pote tem doze big blinds. "
                          "Você tem ás e valete: top pair com valete. O jogador do big blind dá check, e a ação chega a você. "
                          "Para este exercício, sabemos que ele paga uma aposta pequena com vários ases menores. Você aposta quatro big blinds, e ele paga.",
                          [ev("check", "dá check", player=1), ev("bet", "Você aposta", player=0, to=400),
                           ev("call", "e ele paga", player=1, to=400)],
                          banner="River · pote de 12 BB", note="Hipótese do exercício, não leitura garantida"))
    assert t.total() == 2000
    scenes.append(t.scene(f"{p}-04", "Exemplo com hipótese", "Recebeu de uma mão pior",
                          "No showdown, o rival mostra ás e cinco. Os dois têm um par de ases, mas o seu valete vence o nove que acompanha a mão dele. "
                          "A aposta recebeu de uma mão pior, exatamente o que a hipótese previa. Se ele só pagasse com mãos melhores, a conclusão mudaria.",
                          [ev("collect", None), ev("reveal", "o rival mostra", players=[1]),
                           ev("highlight", "o seu valete vence", player=0, hole=[0, 1], boardPick=[0, 1, 4], text="Par de ases, valete"),
                           ev("award", "recebeu de uma mão pior", player=0)],
                          banner="Showdown · pote de 20 BB"))
    assert best(["Ad", "Jd"], t.board) > best(["Ah", "5c"], t.board)
    scenes += [
        steps(f"{p}-05", "Ações legais", "O que a mesa permite",
              "Antes de escolher o tamanho, confira as ações legais. Depois de um check, você pode dar check ou apostar. "
              "Diante de uma aposta, as opções passam a ser fold, call ou raise, respeitando os stacks e as regras da mesa.",
              ["Depois de um check: check ou bet", "Diante de aposta: fold, call ou raise", "Respeite stacks e regras da mesa"],
              ["dar check ou apostar", "Diante de uma aposta", "respeitando os stacks"]),
        compare(f"{p}-06", "O range que paga", "Força contra o range que continua",
                "Agora imagine um rival que chega ao river com muitos pares fracos, mas desiste de todos diante de uma aposta grande. "
                "A força da sua mão contra o conjunto inicial não prova que a aposta grande seja por valor. "
                "Uma aposta pequena pode receber calls mais amplos, mas também oferece ao rival a opção de aumentar.",
                [("Aposta grande", [("Pares fracos desistem", "desiste de todos"), ("Força inicial não prova valor", "não prova")], "danger"),
                 ("Aposta pequena", [("Calls mais amplos", "calls mais amplos"), ("O rival ainda pode aumentar", "opção de aumentar")], "mint")],
                note="Identifique mãos piores que continuam"),
        steps(f"{p}-07", "O range que paga", "O ganho é a diferença",
              "O valor de apostar deve ser comparado ao que aconteceria se você desse check. "
              "Se você já venceria o showdown contra uma mão fraca que sempre desiste, o fold dela não acrescenta nada. "
              "Receber de mãos piores aumenta o ganho, e pagar raises de mãos melhores aumenta o custo. Um call sozinho não prova que a aposta foi boa.",
              ["Compare com o check", "Fold de mão pior não acrescenta", "Raise de mão melhor aumenta o custo"],
              ["comparado ao que aconteceria", "o fold dela", "pagar raises"]),
        steps(f"{p}-08", "Uma amostra só", "Uma amostra não estima um range",
              "No treino, o cenário fixa um pagador com um ás menor. É uma condição didática, e não uma leitura garantida sobre jogadores reais. "
              "Repita a mão com tamanhos diferentes e anote em que ponto a política do bot deixa de continuar. Uma única amostra não estima o range de uma população.",
              ["Pagador fixo: condição didática", "Varie o tamanho e anote", "Uma amostra não estima um range"],
              ["condição didática", "Repita a mão", "Uma única amostra"]),
    ]
    scenes += quiz(p, "Modelo do exercício: o rival paga 4 BB com pares médios e ases menores, e só paga 12 BB com mãos que vencem você. Que aposta cumpre o papel de valor?",
                   [("A", "Apostar 12 BB", "a aposta grande"), ("B", "Apostar 4 BB", "a aposta pequena"), ("C", "Nenhum tamanho", "nenhum tamanho")], "B",
                   "Exercício. No modelo, o rival paga uma aposta de quatro big blinds com pares médios e ases menores, "
                   "mas só paga doze big blinds com mãos que vencem você. Que aposta cumpre o papel de valor? "
                   "A: a aposta grande, de doze big blinds. B: a aposta pequena, de quatro big blinds. C: nenhum tamanho serve. Pause o vídeo e responda.",
                   "Resposta: B. Na aposta pequena, mãos piores continuam e pagam você. Na aposta grande, só as melhores pagam, "
                   "e a aposta deixa de ser por valor. O tamanho certo depende de quem continua, e o rival ainda pode aumentar.",
                   "4 BB: mãos piores pagam · 12 BB: só as melhores")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. O bot pagador continua com pares a preços moderados. "
                   "A hipótese de receber de mãos piores precisa sustentar a sua aposta.",
                   ["Bot pagador: pares a preços moderados", "Receber de pior sustenta a aposta", "Guiado, desafio e replay"],
                   ["O bot pagador", "receber de mãos piores", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m4l1 — Thin value bet e blefe puro", scenes)


def m4l2():
    p = "m4l2"
    scenes = [
        intro(f"{p}-01", "Bluff-catchers e blockers",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo quatro, vamos estudar o bluff-catcher e o efeito dos blockers. "
              "O objetivo é calcular quantos blefes um call exige e contar o que as suas cartas removem.",
              "Zero Tilt Academy · Módulo 4 · Aula 2", "Objetivo: calcular os blefes que o call exige e contar o que suas cartas removem"),
        steps(f"{p}-02", "Bluff-catcher", "Vence os blefes, perde para o valor",
              "Um bluff-catcher é uma mão que vence os blefes considerados e perde para as mãos de valor consideradas. "
              "A categoria depende do range: nem sempre um overpair é um bluff-catcher adequado.",
              ["Vence os blefes considerados", "Perde para o valor considerado", "Depende do range"],
              ["vence os blefes", "perde para as mãos de valor", "A categoria depende"]),
    ]
    t = Mesa(["Você · BB", "BTN"], [["Ad", "Ac"], ["Kc", "Qd"]], stacks=[9000, 9000], dealer=1, seats=[0, 2],
             board=["Js", "Ts", "5c", "2d", "9h"], pot=2000)
    scenes.append(t.scene(f"{p}-03", "Preço e frequência", "Aposta do tamanho do pote",
                          "Veja um river disputado entre dois jogadores. A mesa mostra valete, dez, cinco, dois e nove, e o pote tem vinte big blinds. "
                          "Você está no big blind com dois ases. Você dá check, e o rival aposta vinte big blinds, o tamanho do pote.",
                          [ev("check", "Você dá check", player=0), ev("bet", "o rival aposta", player=1, to=2000)],
                          banner="River · pote de 20 BB"))
    assert t.total() == 4000
    scenes += [
        metrics(f"{p}-04", "Preço e frequência", "Um terço de blefes",
                "Pagar custa vinte, para um pote final de sessenta. O limiar é vinte sobre sessenta, um terço. "
                "Se a sua mão vence todos os blefes e perde para todo valor, sem empates nem rake, você precisa de mais de um terço de blefes no range que apostou. "
                "No limite de um terço, a expectativa é zero.",
                [("CUSTO DO CALL", "20 BB"), ("POTE FINAL", "60 BB"), ("BLEFES · 20/60", "> 33,33%")],
                ["Pagar custa vinte", "pote final de sessenta", "mais de um terço"],
                note="Proporção dentro do range que apostou"),
        steps(f"{p}-05", "Preço e frequência", "A frequência que importa",
              "A frequência relevante é a proporção de blefes dentro do range que apostou, e não dentro de todas as mãos que o rival recebeu. "
              "Por isso, a pergunta certa é: quando este rival aposta o pote no river, quantas dessas apostas são blefes?",
              ["Blefes dentro do range que apostou", "Não entre todas as mãos recebidas", "Quantas apostas são blefes?"],
              ["dentro do range que apostou", "e não dentro", "quantas dessas apostas"]),
        compare(f"{p}-06", "MDF", "MDF responde a outra pergunta",
                "Num modelo simplificado de river, a frequência mínima de defesa, P sobre P mais B, torna indiferente um blefe sem equity. "
                "Para uma aposta do pote, ela vale cinquenta por cento. Mas isso não manda pagar metade das suas mãos contra qualquer pessoa: "
                "contra um range sem blefes, muitos calls são ruins. Em resumo, pot odds avaliam o preço de uma mão, e a MDF é uma referência de defesa do range num modelo.",
                [("MDF", [("P / (P + B): blefe indiferente", "frequência mínima de defesa"), ("Não vale contra quem não blefa", "range sem blefes")], "gold"),
                 ("Pot odds", [("Preço de uma mão", "pot odds avaliam")], "mint")],
                note="Aposta do pote: MDF de 50%"),
    ]

    def kq(hand):
        rest = deck(t.board + hand)
        return sum(c[0] == "K" for c in rest) * sum(c[0] == "Q" for c in rest)
    assert kq(["Ad", "Ac"]) == 16 and kq(["Qc", "8d"]) == 12
    scenes += [
        compare(f"{p}-07", "Blockers", "Conte o que foi removido",
                "Agora vamos aos blockers, as cartas suas que removem combinações do rival. Nesta mesa, rei e dama formam a sequência até o rei, a maior possível. Dois ases não removem nenhum rei nem nenhuma dama dessa combinação. "
                "Se você tivesse uma dama, removeria algumas combinações de rei e dama, mas também poderia retirar blefes que usam uma dama.",
                [("Dois ases", [("Não removem rei nem dama", "Dois ases não removem")], "danger"),
                 ("Uma dama na mão", [("Remove combinações de KQ", "removeria algumas combinações"), ("Também retira blefes com dama", "retirar blefes")], "gold")],
                note="Ter blocker não basta para pagar"),
        steps(f"{p}-08", "Blockers", "Liste valor e blefes primeiro",
              "Uma carta pode remover blefes e piorar o call. Então ter blocker não é razão suficiente. "
              "Liste as mãos de valor e os blefes antes de atribuir um efeito favorável ao blocker, e só depois compare com o preço.",
              ["Blocker pode remover blefes", "Liste valor e blefes", "Depois compare com o preço"],
              ["remover blefes", "Liste as mãos de valor", "compare com o preço"]),
    ]
    assert abs(10 / 35 - 0.2857) < 0.0001 and abs(15 / 25 - 0.6) < 1e-9
    scenes += quiz(p, "River: pote de 15 BB e o rival aposta 10 BB. Sua mão vence todos os blefes e perde para todo valor. Quantos blefes o range de aposta precisa ter?",
                   [("A", "Mais de 10/25 = 40%", "mais de dez sobre vinte e cinco"), ("B", "Mais de 10/35 ≈ 29%", "mais de dez sobre trinta e cinco"),
                    ("C", "Mais de 15/35 ≈ 43%", "mais de quinze sobre trinta e cinco")], "B",
                   "Exercício. No river, o pote tem quinze big blinds, e o rival aposta dez. A sua mão vence todos os blefes e perde para todo valor, sem empates nem rake. "
                   "Quantos blefes o range de aposta precisa ter? A: mais de dez sobre vinte e cinco. B: mais de dez sobre trinta e cinco. "
                   "C: mais de quinze sobre trinta e cinco. Pause o vídeo e responda.",
                   "Resposta: B. Pagar custa dez, e o pote final fica com trinta e cinco. O limiar é dez sobre trinta e cinco, cerca de vinte e oito vírgula seis por cento. "
                   "A conta de quinze sobre vinte e cinco, sessenta por cento, é outra coisa: a frequência mínima de defesa.",
                   "Custo 10 · pote final 35 · 10/35 ≈ 28,6%")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Dois ases não bloqueiam rei e dama. "
                   "Não use a força absoluta do overpair para inventar a frequência de blefes, e faça a conta do preço antes de pagar.",
                   ["AA não bloqueia KQ", "Não invente a frequência de blefes", "Faça a conta do preço"],
                   ["não bloqueiam", "Não use a força", "faça a conta"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m4l2 — Bluff-catchers e blockers", scenes)


def m4l3():
    p = "m4l3"
    scenes = [
        intro(f"{p}-01", "Overbet polarizada e blocking bet",
              "Esta é a Zero Tilt Academy. Na aula três do módulo quatro, vamos comparar a aposta acima do pote, a overbet, com a aposta pequena fora de posição. "
              "O objetivo é classificar a mão antes de escolher o tamanho e calcular o preço de cada aposta.",
              "Zero Tilt Academy · Módulo 4 · Aula 3", "Objetivo: classificar a mão antes do tamanho e calcular o preço"),
    ]
    t = Mesa(["Você · BB", "BTN"], [["Jh", "Th"], ["Kc", "Kd"]], stacks=[9000, 9000], dealer=1, seats=[0, 2],
             board=["9h", "8h", "7c", "2d"], pot=2000)
    scenes.append(t.scene(f"{p}-02", "Primeiro classifique a mão", "Qual é a sua mão?",
                          "Comece pela mão. A mesa mostra nove e oito de copas, sete de paus e dois de ouros, e o pote tem vinte big blinds. "
                          "Você tem valete e dez de copas. O river traz o ás de copas. "
                          "Agora as suas cinco melhores cartas são ás, valete, dez, nove e oito de copas: um flush, e não apenas uma sequência.",
                          [ev("board", "O river traz", cards=["Ah"]),
                           ev("highlight", "um flush", player=0, hole=[0, 1], boardPick=[0, 1, 4], text="Flush A-J-T-9-8")],
                          banner="River · pote de 20 BB"))
    hero = best(["Jh", "Th"], t.board)
    nut = best(["Kh", "Qh"], t.board)
    assert hero == (5, 14, 11, 10, 9, 8) and nut[0] == 5 and nut > hero and max(best(list(h), t.board) for h in itertools.combinations(deck(t.board + ["Jh", "Th"]), 2))[0] == 5
    scenes += [
        metrics(f"{p}-03", "Primeiro classifique a mão", "Flush, mas não o nut flush",
                "Mas esse flush ainda não é o nut flush, o maior possível. Com rei e dama de copas, o rival teria ás, rei, dama, nove e oito: um flush maior. "
                "E não há straight flush, porque falta uma carta de copas em sequência. Uma categoria errada contamina todo o plano de aposta.",
                [("SUA MÃO · FLUSH", "A-J-T-9-8"), ("K♥ Q♥ · FLUSH MAIOR", "A-K-Q-9-8"), ("STRAIGHT FLUSH", "Não há")],
                ["não é o nut flush", "um flush maior", "não há straight flush"],
                note="Classifique antes de escolher o tamanho"),
        compare(f"{p}-04", "Dois tamanhos", "Overbet e blocking bet",
                "Agora os dois tamanhos. Uma aposta maior que o pote pode fazer parte de uma estratégia polarizada, quando ranges e stacks sustentam valor forte e blefes adequados. "
                "Ela não maximiza o retorno em qualquer cenário. Já uma aposta pequena fora de posição pode buscar valor fino ou induzir certas respostas. "
                "Mas ela não fixa o preço do showdown: o rival continua podendo aumentar.",
                [("Overbet polarizada", [("Valor forte e blefes", "valor forte"), ("Não vale em qualquer cenário", "qualquer cenário")], "gold"),
                 ("Blocking bet", [("Valor fino ou induzir", "valor fino"), ("Não fixa o preço", "não fixa o preço")], "mint")],
                note="O rival ainda pode aumentar", note_cue="continua podendo aumentar"),
        metrics(f"{p}-05", "O preço de uma overbet", "Pote de 100, aposta de 150",
                "Agora o preço de uma overbet. Com um pote de cem e uma aposta de cento e cinquenta, pagar custa cento e cinquenta, para um total de quatrocentos: "
                "o limiar é trinta e sete e meio por cento, sem taxas e sem apostas futuras. "
                "E um blefe puro, que perde sempre quando é pago, precisa de folds acima de cento e cinquenta sobre duzentos e cinquenta: sessenta por cento.",
                [("POTE · APOSTA", "100 · 150"), ("CALL · 150/400", "37,5%"), ("BLEFE · 150/250", "> 60%")],
                ["pote de cem", "o limiar é trinta e sete", "sessenta por cento"],
                note="Aposta maior: mais equity para pagar, mais risco no blefe"),
        steps(f"{p}-06", "Aposta pequena", "Uma aposta pequena não fecha a ação",
              "A blocking bet pode buscar pagamento de mãos piores ou influenciar respostas, mas o rival pode aumentar. "
              "Se você aposta pequeno sem plano contra o raise, só transfere a dificuldade para a próxima decisão. "
              "Registre antes se a sua mão pretende pagar, desistir ou aumentar de volta.",
              ["O rival pode aumentar", "Sem plano: a dificuldade só muda de lugar", "Pagar, desistir ou aumentar de volta?"],
              ["o rival pode aumentar", "sem plano contra o raise", "Registre antes"]),
        steps(f"{p}-07", "Força não é invulnerabilidade", "A leitura certa é o ponto de partida",
              "Ler a mão certa é só o ponto de partida. Escolher uma overbet ainda exige justificar quais mãos piores vão pagar "
              "e quais mãos melhores podem continuar ou aumentar.",
              ["Leitura certa: ponto de partida", "Quais mãos piores pagam", "Quais melhores continuam ou aumentam"],
              ["ponto de partida", "quais mãos piores", "quais mãos melhores"]),
    ]
    assert 150 / 400 == 0.375 and 150 / 250 == 0.6 and 20 / 50 == 0.4
    scenes += quiz(p, "O pote tem 10 BB, e o rival faz uma overbet de 20 BB. Sem taxas e sem apostas futuras, qual é o limiar de equity para pagar?",
                   [("A", "20/30 ≈ 67%", "vinte sobre trinta"), ("B", "20/50 = 40%", "vinte sobre cinquenta"), ("C", "10/30 ≈ 33%", "dez sobre trinta")], "B",
                   "Exercício. O pote tem dez big blinds, e o rival faz uma overbet de vinte. Sem taxas e sem apostas futuras, qual é o limiar de equity para pagar? "
                   "A: vinte sobre trinta. B: vinte sobre cinquenta. C: dez sobre trinta. Pause o vídeo e responda.",
                   "Resposta: B. Pagar custa vinte, e o pote final fica com cinquenta: vinte sobre cinquenta, quarenta por cento. "
                   "A conta de vinte sobre trinta é a do rival: o blefe dele precisa de folds acima de dois terços.",
                   "Call 20 · pote final 50 · 20/50 = 40%")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Você tem um flush com ás, valete, dez, nove e oito. "
                   "Um adversário com rei e dama de copas tem um flush maior, então a sua mão não é o nuts. Classifique a mão antes de comparar os tamanhos.",
                   ["Seu flush: A-J-T-9-8", "Kh Qh: flush maior", "Classifique antes do tamanho"],
                   ["Você tem um flush", "flush maior", "Classifique a mão"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m4l3 — Overbet polarizada e blocking bet", scenes)


# ─────────────────────────────── Módulo 6 ───────────────────────────────
def m6l1():
    p = "m6l1"
    scenes = [
        intro(f"{p}-01", "EV e fold equity",
              "Esta é a Zero Tilt Academy. Na aula um do módulo seis, vamos estudar o valor esperado e a fold equity. "
              "O objetivo é montar a conta de uma aposta com projeto, ramo por ramo, e ver como as premissas mudam o resultado.",
              "Zero Tilt Academy · Módulo 6 · Aula 1", "Objetivo: montar a conta de uma aposta, ramo por ramo"),
        steps(f"{p}-02", "Valor esperado", "Uma média do modelo",
              "O valor esperado, ou EV, é a média ponderada dos resultados de um modelo. "
              "Para um call final, sem rake, o EV é a equity vezes o pote final, menos o custo do call. "
              "E um EV positivo não garante lucro em mil mãos, nem em nenhuma sequência finita.",
              ["EV: média ponderada do modelo", "Call final: equity × pote − custo", "EV positivo não garante lucro"],
              ["média ponderada", "a equity vezes o pote final", "não garante lucro"]),
        metrics(f"{p}-03", "Modelo simplificado", "As hipóteses fazem parte da fórmula",
                "Agora uma aposta com projeto. Chame de P o pote, de B a sua aposta, de F a frequência com que todos desistem e de e a sua equity quando há call. "
                "Sem raises, apostas futuras nem rake, o EV é F vezes P, mais um menos F vezes o ramo do call. "
                "E o ramo do call vale e vezes P mais duas vezes B, menos B.",
                [("RAMO DO FOLD", "F × P"), ("PESO DO CALL", "1 − F"), ("RAMO DO CALL", "e(P+2B) − B")],
                ["F vezes P", "um menos F", "o ramo do call vale"],
                note="Sem raises, apostas futuras nem rake"),
    ]
    hands, board = [["6h", "5h"], ["Kd", "Qc"]], ["Kh", "Jh", "2c", "9s"]
    eq = equity(hands, board)[0]
    assert round(eq, 2) == 0.20
    a = Mesa(["Você · BTN", "BB"], hands, stacks=[9500, 9500], dealer=0, seats=[0, 2], board=board, pot=1000)
    scenes.append(a.scene(f"{p}-04", "Dois ramos", "Se todos desistem",
                          "Veja um exemplo no turn, a quarta carta da mesa. O pote tem dez big blinds, e a mesa mostra rei e valete de copas, dois de paus e nove de espadas. "
                          "Você tem seis e cinco de copas: um projeto de flush. O rival dá check, e você aposta cinco. "
                          "No ramo do fold, ele desiste: você recebe a sua aposta de volta e ganha os dez big blinds que já estavam no pote.",
                          [ev("check", "O rival dá check", player=1), ev("bet", "você aposta cinco", player=0, to=500),
                           ev("fold", "ele desiste", player=1), ev("collect", None), ev("award", "ganha os dez", player=0)],
                          banner="Turn · pote de 10 BB · ramo do fold"))
    assert a.stacks[0] - 9500 == 1000
    b = Mesa(["Você · BTN", "BB"], hands, stacks=[9500, 9500], dealer=0, seats=[0, 2], board=board, pot=1000)
    scenes.append(b.scene(f"{p}-05", "Dois ramos", "Se o rival paga",
                          "Agora o outro ramo, com as mesmas cartas. Você aposta cinco, e o rival paga com rei e dama. "
                          "O pote final fica com vinte big blinds. Contra essa mão, só uma copa no river vence: nove cartas entre quarenta e quatro, cerca de vinte por cento.",
                          [ev("check", None, player=1), ev("bet", "Você aposta cinco", player=0, to=500),
                           ev("call", "o rival paga", player=1, to=500), ev("reveal", "rei e dama", players=[1]),
                           ev("collect", "O pote final")],
                          banner="Turn · ramo do call", note="Equity ≈ 20% contra esta mão"))
    assert b.total() == 2000 and abs(0.2 * 20 - 5 + 1) < 1e-9 and abs(0.3 * 10 + 0.7 * -1 - 2.3) < 1e-9
    scenes += [
        metrics(f"{p}-06", "Dois ramos", "A hipótese de resposta decide",
                "Agora a conta. No ramo do call, zero vírgula dois vezes vinte, menos cinco, dá menos um big blind. "
                "Se ninguém desiste, a aposta vale menos um. Se todos desistem trinta por cento das vezes, vale zero vírgula três vezes dez, "
                "mais zero vírgula sete vezes menos um: mais dois vírgula três big blinds.",
                [("RAMO DO CALL · 0,2×20−5", "−1 BB"), ("F = 0", "−1 BB"), ("F = 30%", "+2,3 BB")],
                ["menos um big blind", "Se ninguém desiste", "trinta por cento"],
                note="A mudança veio da hipótese, não das cartas"),
        steps(f"{p}-07", "Fold equity", "Duas formas de ganhar não bastam",
              "Uma aposta com projeto pode ganhar quando todos desistem ou no showdown. Mas ter duas formas de ganhar não prova que apostar é lucrativo. "
              "Considere a frequência de folds, o range que paga, o tamanho, os raises e as apostas futuras.",
              ["Ganha no fold ou no showdown", "Duas formas não provam lucro", "Folds, range, tamanho, raises"],
              ["quando todos desistem", "não prova que apostar", "Considere a frequência"]),
        steps(f"{p}-08", "Fold equity", "Registre os parâmetros",
              "Pequenas diferenças de EV podem desaparecer quando você muda o rake, os ranges ou as frequências. "
              "Não transporte esses trinta por cento para o perfil do bot sem medir ou definir antes. "
              "E lembre que, numa sessão curta, a soma observada pode se afastar bastante da média prevista.",
              ["Pequenas margens são frágeis", "Não transporte os 30% sem medir", "Sessão curta: longe da média"],
              ["Pequenas diferenças", "Não transporte", "numa sessão curta"]),
    ]
    assert abs(0.4 * 10 + 0.6 * -10 + 2) < 1e-9
    scenes += quiz(p, "Blefe puro, sem equity quando pago: pote de 10 BB, aposta de 10 BB, e todos desistem 40% das vezes. Sem raises nem rake, qual é o EV?",
                   [("A", "+4 BB", "o EV é de mais quatro"), ("B", "−2 BB", "menos dois"), ("C", "0", "zero")], "B",
                   "Exercício. Um blefe puro, sem equity quando é pago. O pote tem dez big blinds, a aposta é de dez, e todos desistem quarenta por cento das vezes. "
                   "Sem raises nem rake, qual é o EV? A: o EV é de mais quatro big blinds. B: menos dois. C: zero. Pause o vídeo e responda.",
                   "Resposta: B. No ramo do fold, você ganha dez em quarenta por cento das vezes: mais quatro. No ramo do call, perde dez em sessenta por cento: menos seis. "
                   "A soma é menos dois. Para uma aposta do tamanho do pote, o blefe puro precisa de mais de cinquenta por cento de folds.",
                   "0,4 × 10 + 0,6 × (−10) = −2 BB")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. A tela informa o preço. A equity depende do range, "
                   "e o resultado desta mão não prova a rentabilidade do call. Separe a conta exata do preço da hipótese de equity.",
                   ["A tela informa o preço", "A equity depende do range", "O resultado não prova o call"],
                   ["A tela informa o preço", "A equity depende", "o resultado desta mão"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m6l1 — EV e fold equity", scenes)


def m6l2():
    p = "m6l2"
    tail = sum(math.comb(100, k) * 0.8 ** k * 0.2 ** (100 - k) for k in range(76))
    assert round(100 * tail) == 13 and math.sqrt(100 * 0.8 * 0.2) == 4
    scenes = [
        intro(f"{p}-01", "Banca: o cinto de segurança",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo seis, vamos falar de variância, orçamento e limites pessoais. "
              "O objetivo é decidir o tempo e o limite de uma sessão antes de começar, e respeitar esse limite.",
              "Zero Tilt Academy · Módulo 6 · Aula 2", "Objetivo: decidir tempo e limite antes de começar"),
        metrics(f"{p}-02", "Variância", "Média não é garantia",
                "Pense em cem situações independentes, cada uma com oitenta por cento de chance de vitória. A média esperada é de oitenta vitórias, e não uma contagem garantida. "
                "Nesse modelo, terminar com setenta e cinco vitórias ou menos acontece em cerca de treze por cento das séries de cem. "
                "Sequências desfavoráveis fazem parte do jogo, mesmo quando a chance de cada situação é boa.",
                [("SITUAÇÕES · 80% CADA", "100"), ("MÉDIA ESPERADA", "80"), ("75 OU MENOS", "≈ 13%")],
                ["cem situações independentes", "A média esperada", "setenta e cinco vitórias"],
                note="Modelo binomial, situações independentes"),
        steps(f"{p}-03", "Variância", "Nenhum número fixo protege",
              "Nenhuma quantidade fixa de buy-ins garante segurança, principalmente se a estratégia tiver expectativa negativa. "
              "Recomendações como trinta a cinquenta buy-ins são heurísticas de certas situações de cash game, e não uma regra universal para variantes, torneios ou jogadores.",
              ["Nenhum número fixo garante", "Expectativa negativa: risco maior", "30–50 buy-ins: heurística, não regra"],
              ["Nenhuma quantidade fixa", "expectativa negativa", "trinta a cinquenta"]),
        steps(f"{p}-04", "Limites pessoais", "Defina o que pode ser perdido",
              "Separe um orçamento que você possa perder sem afetar as despesas essenciais, e não use crédito nem valores de terceiros. "
              "Defina o tempo e o limite de perda antes de jogar. Um limite não é uma meta de perda: é uma barreira para encerrar a exposição. "
              "Para estudar este curso, o Play Money permite praticar sem depósito.",
              ["Orçamento separado do essencial", "Tempo e limite antes de jogar", "Limite é barreira, não meta"],
              ["Separe um orçamento", "Defina o tempo", "Um limite não é uma meta"]),
        compare(f"{p}-05", "Decisão após perdas", "Depois de perder",
                "Depois de uma sequência de perdas, reduzir a exposição ou interromper a sessão são escolhas válidas. "
                "Já aumentar as apostas para recuperar não é uma justificativa técnica. "
                "E, quando precisar, use as ferramentas de limites e de autoexclusão da plataforma.",
                [("Escolhas válidas", [("Reduzir a exposição", "reduzir a exposição"), ("Interromper a sessão", "interromper a sessão")], "mint"),
                 ("Sem justificativa técnica", [("Aumentar para recuperar", "aumentar as apostas")], "danger")],
                note="Limites e autoexclusão estão na plataforma", note_cue="ferramentas de limites"),
        steps(f"{p}-06", "Pesquisa sobre tilt", "O que a pesquisa mostra",
              "Palomäki e colaboradores analisaram as narrativas de sessenta jogadores finlandeses e associaram a indignação à perseguição de perdas nos relatos. "
              "É uma evidência qualitativa, e não uma prova causal nem um tratamento. "
              "O nosso exercício é registrar o estado emocional e decidir os limites antes da sessão.",
              ["Narrativas de 60 jogadores", "Evidência qualitativa, não causal", "Registre o estado e os limites"],
              ["Palomäki", "evidência qualitativa", "registrar o estado emocional"]),
        steps(f"{p}-07", "Sessão de treino", "Uma sessão com começo e fim",
              "Antes de abrir a mesa, escreva o objetivo, um número de mãos ou um tempo, e o limite. Ao terminar, conte quais decisões você conseguiu explicar. "
              "Se estiver repetindo apostas para recuperar fichas perdidas, encerre e reveja o registro depois. "
              "As fichas não têm valor, mas respeitar o plano faz parte do exercício.",
              ["Objetivo, mãos ou tempo e limite", "Conte as decisões explicadas", "Recuperando fichas? Encerre"],
              ["escreva o objetivo", "conte quais decisões", "encerre e reveja"]),
    ]
    scenes += quiz(p, "Antes de abrir a mesa de treino, qual destes é um plano de sessão completo?",
                   [("A", "Jogar até recuperar o que perdeu ontem", "jogar até recuperar"),
                    ("B", "Objetivo, número de mãos ou tempo e limite de perda", "objetivo, número de mãos"),
                    ("C", "Aumentar o stack sempre que perder", "aumentar o stack")], "B",
                   "Exercício. Antes de abrir a mesa de treino, qual destes é um plano de sessão completo? "
                   "A: jogar até recuperar o que perdeu ontem. B: objetivo, número de mãos ou tempo, e limite de perda. "
                   "C: aumentar o stack sempre que perder. Pause o vídeo e responda.",
                   "Resposta: B. O plano diz o que treinar, quando parar e quanto você aceita perder, tudo decidido antes. "
                   "As outras duas opções amarram a sessão à recuperação das perdas, e não ao estudo.",
                   "Objetivo + mãos ou tempo + limite, antes de jogar")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Registre no caderno o limite da sessão. "
                   "Ao atingir esse limite, use o botão Encerrar treino, e não aumente o stack para recuperar.",
                   ["Registre o limite no caderno", "Atingiu? Encerrar treino", "Não aumente o stack"],
                   ["Registre no caderno", "Encerrar treino", "não aumente o stack"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m6l2 — Banca: o cinto de segurança", scenes)


def m6l3():
    p = "m6l3"
    scenes = [
        intro(f"{p}-01", "Revisão final: o método completo",
              "Esta é a Zero Tilt Academy. Na aula três do módulo seis, vamos juntar o curso num método de revisão. "
              "O objetivo é registrar a informação, as alternativas e a justificativa de cada decisão, separando a decisão do resultado.",
              "Zero Tilt Academy · Módulo 6 · Aula 3", "Objetivo: separar a decisão do resultado"),
        steps(f"{p}-02", "Método de revisão", "Cinco passos",
              "O método tem cinco passos. Primeiro, identifique a modalidade e as regras de formação da mão. "
              "Segundo, registre a fase, a posição, os stacks, as ações e as informações disponíveis. "
              "Terceiro, defina ranges ou hipóteses, sem atribuir equity sem um modelo. Quarto, compare as ações legais, o preço e as continuações. "
              "Quinto, respeite o orçamento e o tempo, e avalie a decisão separada do resultado.",
              ["1. Modalidade e formação da mão", "2. Fase, posição, stacks e ações", "3. Ranges ou hipóteses", "4. Ações legais, preço e continuações", "5. Orçamento, tempo e resultado à parte"],
              ["Primeiro", "Segundo", "Terceiro", "Quarto", "Quinto"]),
    ]
    hands, board = [["As", "Ad"], ["Kh", "Qh"]], ["Jh", "7h", "2c", "5s"]
    eq = equity(hands, board)
    assert round(eq[0], 3) == 0.795
    t = Mesa(["Você · BTN", "BB"], hands, stacks=[8000, 8000], dealer=0, seats=[0, 2], board=board, pot=4000)
    scenes.append(t.scene(f"{p}-03", "Decisão e resultado", "Uma boa decisão que perde",
                          "Veja uma mão para revisar. No turn, o pote tem quarenta big blinds, e a mesa mostra valete e sete de copas, dois de paus e cinco de espadas. "
                          "Você tem dois ases. O rival vai all-in com oitenta, e você paga. Ele mostra rei e dama de copas, um projeto de flush. "
                          "O river traz o três de copas, e o pote vai para ele.",
                          [ev("allin", "O rival vai all-in", player=1, to=8000), ev("call", "e você paga", player=0, to=8000),
                           ev("collect", None), ev("reveal", "Ele mostra", players=[1]),
                           ev("board", "O river traz", cards=["3h"]), ev("award", "o pote vai para ele", player=1)],
                          banner="Turn · pote de 40 BB"))
    assert t.stacks == [0, 20000] and best(["Kh", "Qh"], t.board)[0] == 5
    scenes += [
        metrics(f"{p}-04", "Decisão e resultado", "O que se sabia no call",
                "Agora revise com o que se sabia no momento do call. Contra rei e dama de copas, os seus ases venciam cerca de setenta e nove vírgula cinco por cento das vezes. "
                "O rival precisava de uma das nove copas entre quarenta e quatro cartas. A derrota veio da carta do river, e não de um erro na decisão.",
                [("SEUS ASES NO CALL", "≈ 79,5%"), ("COPAS DO RIVAL", "9 de 44"), ("RESULTADO", "Perdeu")],
                ["setenta e nove", "nove copas", "A derrota veio"],
                note="Decisão boa, resultado ruim"),
        compare(f"{p}-05", "Duas passagens", "Revise em duas passagens",
                "Revise em duas passagens. Na primeira, esconda o resultado e reconstrua a informação disponível: modalidade, cartas, board, posições, stacks e ações. "
                "Escreva pelo menos duas alternativas legais. Na segunda, veja a continuação e identifique o que foi resultado aleatório, mudança de informação ou falha no raciocínio.",
                [("Primeira passagem", [("Esconda o resultado", "esconda o resultado"), ("Duas alternativas legais", "duas alternativas")], "mint"),
                 ("Segunda passagem", [("Veja a continuação", "veja a continuação"), ("Sorte, informação ou raciocínio?", "falha no raciocínio")], "gold")],
                note="Nem toda vitória é boa decisão"),
        steps(f"{p}-06", "Quatro jogos", "Use os quatro jogos como teste de regras",
              "Use os quatro jogos como teste de regras. No Hold'em, pergunte se a mesa pode jogar sozinha. No Short Deck, refaça o universo de cartas e o ranking. "
              "No Omaha, confirme duas privadas entre quatro. "
              "E no Brazilian Pineapple, decida a troca na sua vez, antes de agir, no flop, no turn e no river; no river, depois da troca, escolha as duas privadas que ficam.",
              ["Hold'em: a mesa joga sozinha?", "Short Deck: universo e ranking", "Omaha: duas privadas entre quatro", "Pineapple: troca na sua vez"],
              ["pergunte se a mesa", "No Short Deck", "No Omaha", "no Brazilian Pineapple"]),
        steps(f"{p}-07", "Prática distribuída", "Teste antes de reler",
              "Responda aos exercícios antes de abrir as explicações, e volte em outra sessão para explicar o raciocínio com as suas palavras. "
              "Dunlosky e colaboradores classificam os testes de prática e o estudo distribuído como técnicas de alta utilidade, numa revisão educacional. "
              "Essa revisão não mediu ganhos financeiros neste curso.",
              ["Responda antes da explicação", "Volte em outra sessão", "Alta utilidade, sem medir ganhos"],
              ["Responda aos exercícios", "Dunlosky", "não mediu ganhos"]),
        steps(f"{p}-08", "Prática distribuída", "Um ciclo de aprendizagem",
              "Assista ao capítulo, resolva o exemplo sem ajuda e jogue o cenário correspondente. Salve a justificativa e volte em outra sessão no modo desafio. "
              "Se errar o preço ou a formação da mão, refaça a conta antes de aumentar a dificuldade. "
              "O registro de estudo não certifica habilidade profissional nem prevê lucro contra pessoas.",
              ["Capítulo, exemplo e cenário", "Errou? Refaça a conta", "Registro não é certificação"],
              ["Assista ao capítulo", "Se errar o preço", "O registro de estudo"]),
    ]
    kk = ["Ks", "Kd", "7c", "7h", "2s"]
    assert best(["Ac", "3d"], kk) > best(["Qc", "Jd"], kk) and best(["Ac", "3d"], kk)[0] == 2
    scenes += quiz(p, "Texas, river: a mesa tem K♠ K♦ 7♣ 7♥ 2♠. Você tem A♣ 3♦, e o rival tem Q♣ J♦. Quem leva o pote?",
                   [("A", "Divisão: a mesa joga para os dois", "divisão"), ("B", "Você, pelo ás", "você, pelo ás"), ("C", "O rival, pela dama e pelo valete", "o rival")], "B",
                   "Exercício. Texas, no river. A mesa tem rei de espadas, rei de ouros, sete de paus, sete de copas e dois de espadas. "
                   "Você tem ás de paus e três de ouros, e o rival tem dama de paus e valete de ouros. Quem leva o pote? "
                   "A: divisão, porque a mesa joga para os dois. B: você, pelo ás. C: o rival, pela dama e pelo valete. Pause o vídeo e responda.",
                   "Resposta: B. Os dois têm dois pares, reis e setes, que vêm da mesa. A quinta carta decide: o seu ás vence a dama do rival. "
                   "A mesa só jogaria sozinha se ninguém tivesse uma carta maior que o dois de espadas para a quinta posição.",
                   "Reis e setes da mesa · ás vence a dama",
                   hole=["Ac", "3d"], board=kk)
    scenes += [
        lab(f"{p}-30",
            "Agora leve a aula para a mesa de treino. Comece pelo treino guiado, depois use o modo desafio e escreva a sua justificativa antes de cada decisão. "
            "Em seguida, reveja a mesma distribuição com outra linha no replay, e separe a decisão do resultado.",
            ["Treino guiado, depois desafio", "Escreva a justificativa antes", "Replay com outra linha"],
            ["Comece pelo treino guiado", "escreva a sua justificativa", "reveja a mesma distribuição"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m6l3 — Revisão final: o método completo", scenes)


# ─────────────────────────────── Módulo 0 (história e regras) ───────────────────────────────
def m0l1():
    p = "m0l1"
    scenes = [
        intro(f"{p}-01", "De onde vêm as cartas",
              "Esta é a Zero Tilt Academy. Na primeira aula do módulo zero, vamos ver de onde vêm as cartas e por que o poker é um jogo de informação incompleta. "
              "O objetivo é diferenciar o que uma fonte comprova do que ela não comprova.",
              "Zero Tilt Academy · Módulo 0 · Aula 1", "Objetivo: separar o que a fonte comprova do que ela não comprova"),
        compare(f"{p}-02", "Um objeto, muitas perguntas", "O baralho do Met",
                "O Met, museu de arte de Nova York, preserva um baralho completo de cinquenta e duas cartas dos Países Baixos meridionais, "
                "datado de cerca de mil quatrocentos e setenta e cinco a mil quatrocentos e oitenta. Os naipes representam equipamentos de caça. "
                "A peça comprova a existência desse baralho, mas não o nascimento do poker. E o museu informa que as regras exatas dos jogos associados não são conhecidas.",
                [("A peça comprova", [("Um baralho de 52 cartas", "cinquenta e duas cartas"), ("Datado de c. 1475–1480", "cerca de mil")], "mint"),
                 ("A peça não comprova", [("O nascimento do poker", "não o nascimento"), ("As regras dos jogos da época", "regras exatas")], "danger")],
                note="Naipes com equipamentos de caça"),
        metrics(f"{p}-03", "O baralho de hoje", "A regra vem antes da estratégia",
                "No Hold'em tradicional, são treze valores vezes quatro naipes: cinquenta e duas cartas, sem curingas. O Short Deck usa trinta e seis. "
                "A regra da modalidade vem antes da estratégia.",
                [("VALORES × NAIPES", "13 × 4"), ("HOLD'EM TRADICIONAL", "52 cartas"), ("SHORT DECK", "36 cartas")],
                ["treze valores", "cinquenta e duas cartas", "trinta e seis"], note="Sem curingas"),
        steps(f"{p}-04", "Informação incompleta", "Três cartas bastam para estudar",
              "No modelo de Kuhn, três cartas e dois jogadores já bastam para estudar decisões com informação privada. "
              "O modelo não equivale ao Hold'em, mas ajuda a formular uma pergunta: o que você sabe, e o que ainda precisa estimar?",
              ["Kuhn: três cartas, dois jogadores", "Não equivale ao Hold'em", "O que você sabe? O que precisa estimar?"],
              ["No modelo de Kuhn", "não equivale", "o que você sabe"]),
        compare(f"{p}-05", "Para pensar", "Observação e inferência",
                "Para pensar depois da aula, diferencie observação de inferência. Ver uma carta é uma observação. "
                "Estimar quais mãos um adversário aposta é uma inferência, que pode estar errada e precisa ser revista.",
                [("Observação", [("Ver uma carta", "Ver uma carta")], "mint"),
                 ("Inferência", [("Estimar as mãos que o rival aposta", "Estimar quais mãos")], "gold")]),
    ]
    assert len(deck()) == 52 and len(deck(short=True)) == 36
    scenes += quiz(p, "Qual destas frases descreve uma inferência?",
                   [("A", "O flop mostra o sete de paus", "o flop mostra"), ("B", "O rival apostou 5 BB", "o rival apostou"),
                    ("C", "O rival costuma apostar com pares médios", "o rival costuma")], "C",
                   "Exercício. Qual destas frases descreve uma inferência? A: o flop mostra o sete de paus. B: o rival apostou cinco big blinds. "
                   "C: o rival costuma apostar com pares médios. Pause o vídeo e responda.",
                   "Resposta: C. A carta do flop e o valor da aposta são observações: estão na mesa. "
                   "Dizer com que mãos o rival costuma apostar é uma estimativa, que pode estar errada e deve ser revista.",
                   "Carta e aposta: observação · hábito do rival: inferência")
    scenes.append(outro(f"{p}-99"))
    write(p, "Aula m0l1 — De onde vêm as cartas", scenes)


def m0l2():
    p = "m0l2"
    scenes = [
        intro(f"{p}-01", "Do Mississippi ao mundo",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo zero, vamos percorrer a história do poker e do Omaha pelas fontes. "
              "O objetivo é distinguir relato, registro e modelo, sem transformar popularização em prova de invenção.",
              "Zero Tilt Academy · Módulo 0 · Aula 2", "Objetivo: distinguir relato, registro e modelo"),
        steps(f"{p}-02", "Um relato de 1844", "Uma partida num barco",
              "Em mil oitocentos e quarenta e quatro, Joe Cowell publicou uma memória em que descreve, na parte dois, página noventa e quatro, "
              "uma partida de poker num barco: cartas escondidas, apostas e desistências. Ele já coloca o ás no topo dos valores. "
              "É uma memória autobiográfica, publicada depois dos acontecimentos, e não uma certidão de nascimento do jogo.",
              ["Joe Cowell, 1844: partida num barco", "Cartas escondidas, apostas e desistências", "Memória, não certidão de nascimento"],
              ["Joe Cowell", "cartas escondidas", "não uma certidão"]),
        steps(f"{p}-03", "Cuidado com datas absolutas", "O que esta aula não afirma",
              "Esta aula não atribui a invenção do poker ou do blefe a mil oitocentos e vinte e nove. "
              "Também não afirma uma origem comprovada do Hold'em em mil novecentos e vinte e cinco, nem uma mudança do valor do ás em mil novecentos e sessenta e sete. "
              "Essas afirmações, de material anterior, não tinham sustentação nas fontes consultadas.",
              ["1829: sem prova de invenção", "1925: sem origem comprovada do Hold'em", "1967: sem mudança comprovada do ás"],
              ["mil oitocentos e vinte e nove", "mil novecentos e vinte e cinco", "mil novecentos e sessenta e sete"]),
        metrics(f"{p}-04", "1970 e 1971", "Formatos diferentes",
                "Segundo a WSOP, Johnny Moss foi escolhido por voto em mil novecentos e setenta e venceu o torneio em mil novecentos e setenta e um. "
                "Preservar essa diferença ajuda a entender a evolução da competição.",
                [("CAMPEÃO NOS DOIS ANOS", "Johnny Moss"), ("1970", "Por voto"), ("1971", "Torneio")],
                ["Johnny Moss", "escolhido por voto", "venceu o torneio"]),
        steps(f"{p}-05", "Omaha antes dos cassinos", "Relatos, não certidões",
              "Bob Ciaffone recorda ter jogado no Michigan, nos anos setenta, uma variante chamada two by three, com as comunitárias expostas uma a uma. "
              "Em mil novecentos e oitenta e dois, encontrou o Omaha no Golden Nugget de Las Vegas, promovido por Robert Turner e por Gwen, de Seattle. "
              "O relato dele, de dois mil e sete, também registra nomes regionais, como nine-card.",
              ["Ciaffone: two by three no Michigan, anos 1970", "1982: Omaha no Golden Nugget", "Relato de 2007, com nomes regionais"],
              ["Bob Ciaffone", "Em mil novecentos e oitenta e dois", "O relato dele"]),
        steps(f"{p}-06", "Turner, Gwen e Bill Boyd", "Difusão em cassinos",
              "Numa memória publicada em dois mil e catorze, Turner descreve a participação de Gwen e de Bill Boyd, responsável pela sala do Golden Nugget, "
              "na oferta do jogo de quatro cartas. Boyd o divulgou como Nugget Hold'em. "
              "Esse marco ajuda a documentar a difusão em cassinos, mas não determina a primeira partida da história.",
              ["Turner: memória de 2014", "Bill Boyd: Nugget Hold'em", "Difusão, não primeira partida"],
              ["Turner descreve", "Boyd o divulgou", "não determina"]),
        compare(f"{p}-07", "O nome Omaha", "O nome não comprova Nebraska",
                "Na entrevista de janeiro de dois mil e catorze, Turner disse não conhecer uma ligação do nome com Nebraska, e lembrou, com incerteza, um frequentador de Omaha. "
                "Isso não sustenta que a modalidade recebeu o nome porque nasceu naquela cidade. A origem exata do nome permanece incerta nas fontes consultadas.",
                [("O que Turner disse", [("Não conhece ligação com Nebraska", "não conhecer uma ligação"), ("Lembra, com incerteza, um frequentador", "com incerteza")], "gold"),
                 ("Conclusão", [("Origem do nome: incerta", "permanece incerta")], "mint")]),
        compare(f"{p}-08", "Omaha na WSOP", "Dois marcos, dois formatos",
                "A retrospectiva de Chad Holloway registra a estreia do Limit Omaha em mil novecentos e oitenta e três, com vitória de David Sklansky. "
                "Um relatório oficial da WSOP situa a primeira presença do Pot-Limit Omaha em mil novecentos e oitenta e quatro, com dois eventos. "
                "São formatos e marcos diferentes: não troque essas datas.",
                [("1983 · Limit Omaha", [("Vitória de David Sklansky", "David Sklansky")], "gold"),
                 ("1984 · Pot-Limit Omaha", [("Dois eventos", "dois eventos")], "mint")],
                note="Formatos diferentes, datas diferentes", note_cue="não troque"),
        steps(f"{p}-09", "O que muda entre os formatos", "Nomes que não são intercambiáveis",
              "No Omaha de quatro cartas, a identidade é receber quatro privadas e formar a mão com exatamente duas privadas e três comunitárias. "
              "Limit e pot-limit descrevem limites de aposta; high e hi-lo descrevem a disputa do pote. "
              "Esses nomes não são intercambiáveis, e a ZT segue as suas próprias regras de Omaha de quatro cartas.",
              ["Omaha 4: exatamente 2 + 3", "Limit e pot-limit: limites de aposta", "High e hi-lo: disputa do pote"],
              ["exatamente duas", "descrevem limites", "descrevem a disputa"]),
        steps(f"{p}-10", "Pesquisa e fontes", "Modelos e testemunhos",
              "Em dois mil e quinze, Kenneth Ho estudou o Omaha Hi-Lo numa dissertação da Harvard Extension School, com dois jogadores e as escolhas de all-in ou desistir. "
              "É pesquisa computacional com regras delimitadas, e não uma solução de todo o Omaha. "
              "Testemunhos retrospectivos ajudam a reconstruir a circulação do jogo, mas podem divergir: Turner e Ciaffone diferem em detalhes da passagem para o pot-limit.",
              ["Ho, 2015: modelo de Omaha Hi-Lo", "Pesquisa delimitada, não solução", "Testemunhos podem divergir"],
              ["Kenneth Ho", "pesquisa computacional", "Testemunhos retrospectivos"]),
        steps(f"{p}-11", "2003", "Uma vitória de grande repercussão",
              "Em dois mil e três, Chris Moneymaker venceu o Main Event depois de se classificar num satélite online. "
              "A organizadora associa essa vitória à expansão do público. "
              "Mas um caso excepcional não estima a chance de vitória nem o retorno financeiro de um iniciante.",
              ["2003: Moneymaker, via satélite online", "Marco de expansão do público", "Um caso não estima chances"],
              ["Chris Moneymaker", "associa essa vitória", "um caso excepcional"]),
    ]
    scenes += quiz(p, "O que um registro oficial de torneio, como o da WSOP de 1984, documenta?",
                   [("A", "A invenção da modalidade", "a invenção"), ("B", "Um evento: aquele torneio", "um evento"),
                    ("C", "A origem do nome da modalidade", "a origem do nome")], "B",
                   "Exercício. O que um registro oficial de torneio, como o da WSOP de mil novecentos e oitenta e quatro, documenta? "
                   "A: a invenção da modalidade. B: um evento, aquele torneio. C: a origem do nome da modalidade. Pause o vídeo e responda.",
                   "Resposta: B. Registros de torneios documentam um evento. Testemunhos ajudam a reconstruir a circulação do jogo, e modelos estudam decisões sob hipóteses. "
                   "Nenhuma dessas evidências transforma popularização em prova de invenção.",
                   "Registro: evento · testemunho: circulação · modelo: hipóteses")
    scenes.append(outro(f"{p}-99"))
    write(p, "Aula m0l2 — Do Mississippi ao mundo", scenes)


def m0l3():
    p = "m0l3"
    scenes = [
        intro(f"{p}-01", "Nomes que fizeram história",
              "Esta é a Zero Tilt Academy. Na aula três do módulo zero, vamos ler alguns feitos da história do poker junto com o contexto. "
              "O objetivo é separar o resultado de um campeão do processo de decisão que você pode estudar.",
              "Zero Tilt Academy · Módulo 0 · Aula 3", "Objetivo: separar o resultado do processo de decisão"),
        steps(f"{p}-02", "Feito e contexto", "Leia o feito junto com o contexto",
              "Johnny Moss foi eleito campeão da WSOP em mil novecentos e setenta e venceu o torneio de mil novecentos e setenta e um. "
              "Chris Moneymaker marcou a expansão do público em dois mil e três. "
              "E Alexandre Gomes conquistou o primeiro bracelete brasileiro em dois mil e oito, conforme o registro retrospectivo da WSOP.",
              ["Moss: eleito em 1970, campeão em 1971", "Moneymaker: expansão do público, 2003", "Alexandre Gomes: 1º bracelete brasileiro, 2008"],
              ["Johnny Moss", "Moneymaker marcou", "Alexandre Gomes"]),
        compare(f"{p}-03", "Ciência e contexto", "A ciência também precisa de contexto",
                "Em dois mil e quinze, Bowling e colaboradores publicaram um resultado para o heads-up limit Hold'em: uma solução essencialmente fraca daquele jogo. "
                "Isso não significa resolver o no-limit, mesas com mais jogadores ou ganhar toda mão.",
                [("O que foi resolvido", [("Limit Hold'em com dois jogadores", "publicaram um resultado"), ("Solução essencialmente fraca", "essencialmente fraca")], "mint"),
                 ("O que não foi", [("No-limit e mesas maiores", "Isso não significa"), ("Ganhar toda mão", "ganhar toda mão")], "danger")]),
        steps(f"{p}-04", "Resultado e processo", "Uma vitória não revela o processo",
              "Uma vitória isolada não revela todo o processo de decisão. No exercício da Academy, registre as informações disponíveis, "
              "as alternativas consideradas e a justificativa da escolha. Essa é uma proposta de estudo, e não uma afirmação sobre a rotina pessoal dos campeões.",
              ["Vitória isolada não revela o processo", "Informação, alternativas e justificativa", "Proposta de estudo, não rotina de campeões"],
              ["Uma vitória isolada", "registre as informações", "Essa é uma proposta"]),
    ]
    scenes += quiz(p, "O resultado publicado por Bowling e colaboradores em 2015 vale para qual jogo?",
                   [("A", "Todas as modalidades de poker", "todas as modalidades"), ("B", "Limit Hold'em com dois jogadores", "só o limit"),
                    ("C", "Mesas cheias de no-limit Hold'em", "mesas cheias")], "B",
                   "Exercício. O resultado publicado por Bowling e colaboradores em dois mil e quinze vale para qual jogo? "
                   "A: todas as modalidades de poker. B: só o limit Hold'em com dois jogadores. C: mesas cheias de no-limit Hold'em. Pause o vídeo e responda.",
                   "Resposta: B. O resultado vale para o limit Hold'em disputado por dois jogadores, e é uma solução essencialmente fraca. "
                   "Ele não resolve o no-limit, nem mesas com mais jogadores.",
                   "Limit Hold'em com dois jogadores · solução essencialmente fraca")
    scenes.append(outro(f"{p}-99"))
    write(p, "Aula m0l3 — Nomes que fizeram história", scenes)


def m0l4():
    p = "m0l4"
    scenes = [
        intro(f"{p}-01", "Regras e hierarquia de mãos",
              "Esta é a Zero Tilt Academy. Na aula quatro do módulo zero, vamos estudar as regras comuns e a hierarquia de mãos. "
              "O objetivo é montar as cinco cartas que contam antes de anunciar um vencedor.",
              "Zero Tilt Academy · Módulo 0 · Aula 4", "Objetivo: montar as cinco cartas que contam"),
        steps(f"{p}-02", "O objetivo de uma mão", "Você disputa potes",
              "Você disputa potes. Se todos os adversários desistirem, você vence sem mostrar as cartas, e isso pode acontecer com uma aposta de valor ou com um blefe. "
              "Quando há showdown, comparam-se as melhores mãos de cinco cartas entre os jogadores que disputam cada pote. "
              "Empates dividem o pote, e all-ins de valores diferentes podem criar potes paralelos.",
              ["Todos desistem: vence sem mostrar", "Showdown: melhor mão de cinco cartas", "Empate divide; all-ins criam potes paralelos"],
              ["Se todos os adversários desistirem", "Quando há showdown", "Empates dividem"]),
        steps(f"{p}-03", "Ranking clássico", "Do maior para o menor",
              "O ranking clássico, do maior para o menor, começa pelo royal flush, a sequência do mesmo naipe do dez ao ás. "
              "Depois vêm o straight flush, a quadra, o full house e, em seguida, o flush, com cinco cartas do mesmo naipe.",
              ["Royal flush: T-J-Q-K-A do mesmo naipe", "Straight flush", "Quadra", "Full house", "Flush"],
              ["royal flush", "straight flush", "a quadra", "o full house", "em seguida, o flush"]),
        steps(f"{p}-04", "Ranking clássico", "E as cinco seguintes",
              "Abaixo do flush vêm a sequência, a trinca, os dois pares, um par e, por último, a carta alta. "
              "Na sequência ás, dois, três, quatro e cinco, o ás vale baixo: ela perde para a sequência do dois ao seis. "
              "E os naipes nunca desempatam mãos equivalentes.",
              ["Sequência · trinca · dois pares", "Um par · carta alta", "A-2-3-4-5: o ás vale baixo", "Naipes não desempatam"],
              ["Abaixo do flush", "um par e", "o ás vale baixo", "os naipes nunca desempatam"]),
        steps(f"{p}-05", "Modalidades e ações", "As modalidades da ZT",
              "A ZT tem quatro modalidades. O Texas Hold'em usa cinquenta e duas cartas e duas privadas, e no showdown pode usar zero, uma ou duas delas. "
              "O Short Deck usa trinta e seis cartas, do seis ao ás, e na ZT a trinca vence a sequência e o flush vence o full house. "
              "O Omaha de quatro cartas usa o ranking clássico e exige exatamente duas privadas e três comunitárias. "
              "E o Brazilian Pineapple começa com três privadas, termina com duas e forma a mão como no Hold'em.",
              ["Texas: 52 cartas, usa 0, 1 ou 2 privadas", "Short Deck: 36 cartas, ranking próprio", "Omaha 4: exatamente 2 + 3", "Pineapple: começa com 3, termina com 2"],
              ["cinquenta e duas cartas", "O Short Deck", "O Omaha de quatro cartas", "o Brazilian Pineapple"]),
        steps(f"{p}-06", "Modalidades e ações", "Ações e ordem de ação",
              "Check só existe quando não há valor a pagar. Call cobre a aposta, bet inicia uma aposta na rodada e raise aumenta uma aposta existente. "
              "No pré-flop, com três ou mais jogadores, a ação começa à esquerda do big blind; depois do flop, no primeiro jogador ativo à esquerda do botão. "
              "Em heads-up, o botão é o small blind: age primeiro no pré-flop e por último depois. E, na ZT, os blinds podem ter o mesmo valor: confira a mesa.",
              ["Check: só sem valor a pagar", "Call, bet e raise", "Pré-flop: à esquerda do BB · depois: à esquerda do botão", "Dois jogadores: o botão é o SB"],
              ["Check só existe", "Call cobre", "No pré-flop", "o botão é o small blind"]),
    ]
    t = Mesa(["Você · BTN", "BB"], [["As", "Kd"], ["Ad", "Jc"]], stacks=[9000, 9000], dealer=0, seats=[0, 2],
             board=["Ah", "Qc", "8s", "4d", "2c"], pot=2000)
    scenes.append(t.scene(f"{p}-07", "Desempate", "Sem uma sexta carta",
                          "Agora o desempate. No river, a mesa mostra ás, dama, oito, quatro e dois, e o pote tem vinte big blinds. "
                          "Você tem ás e rei; o rival tem ás e valete, e os dois vão ao showdown. Compare primeiro a categoria: os dois têm um par de ases. "
                          "Depois os kickers: as suas cinco cartas são ás, ás, rei, dama e oito, e as dele são ás, ás, dama, valete e oito. "
                          "O rei decide o confronto, e o dois não entra em nenhuma das duas mãos.",
                          [ev("reveal", "os dois vão ao showdown", players=[1]),
                           ev("highlight", "as suas cinco cartas", player=0, hole=[0, 1], boardPick=[0, 1, 2], text="A-A-K-Q-8"),
                           ev("award", "O rei decide", player=0)],
                          banner="River · pote de 20 BB"))
    assert best(["As", "Kd"], t.board) == (1, 14, 13, 12, 8) and best(["Ad", "Jc"], t.board) == (1, 14, 12, 11, 8)
    scenes += [
        metrics(f"{p}-08", "Potes paralelos", "Vinte, cinquenta e cem",
                "Agora os potes paralelos. Imagine três jogadores comprometendo vinte, cinquenta e cem fichas, sem outros participantes nem taxas. "
                "O pote principal tem sessenta: vinte de cada um. Há um pote paralelo de sessenta: trinta dos dois que mais investiram. "
                "E as cinquenta fichas sem cobertura voltam ao maior apostador. Quem só investiu vinte não pode ganhar o paralelo.",
                [("POTE PRINCIPAL · 20 × 3", "60"), ("PARALELO · 30 × 2", "60"), ("DEVOLVIDAS", "50")],
                ["O pote principal tem sessenta", "Há um pote paralelo", "as cinquenta fichas"], note="60 + 60 + 50 = 170 fichas comprometidas"),
        steps(f"{p}-09", "Rotina de leitura", "Antes de agir e antes do vencedor",
              "Monte uma rotina de leitura. Antes de agir, diga a modalidade, a street e o valor que falta pagar. "
              "Antes de anunciar um vencedor, monte cinco cartas legais. "
              "Essa sequência evita confundir uma mão visualmente bonita com uma mão permitida pelas regras.",
              ["Modalidade, street e valor a pagar", "Cinco cartas legais antes do vencedor", "Bonita não é o mesmo que permitida"],
              ["Antes de agir", "Antes de anunciar", "Essa sequência evita"]),
    ]
    assert 20 * 3 + 30 * 2 + 50 == 170 and 10 * 3 + 30 * 2 == 90
    scenes += quiz(p, "Três jogadores vão all-in com 10, 40 e 40 fichas, sem outros participantes nem taxas. Quanto vale o pote paralelo?",
                   [("A", "60 fichas", "sessenta"), ("B", "80 fichas", "oitenta"), ("C", "90 fichas", "noventa")], "A",
                   "Exercício. Três jogadores vão all-in com dez, quarenta e quarenta fichas, sem outros participantes nem taxas. Quanto vale o pote paralelo? "
                   "A: sessenta fichas. B: oitenta fichas. C: noventa fichas. Pause o vídeo e responda.",
                   "Resposta: A. O pote principal junta dez de cada jogador: trinta fichas. O paralelo junta os trinta restantes dos dois maiores: sessenta fichas. "
                   "Nenhuma ficha volta, porque os dois maiores investiram o mesmo valor.",
                   "Principal 10 × 3 = 30 · paralelo 30 × 2 = 60")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Classifique a mão e compare os kickers antes de agir: um par de ases usa rei, dama e oito como kickers. "
                   "Observe também se a aposta do adversário muda o preço.",
                   ["Classifique a mão", "Par de ases: K, Q e 8 de kickers", "A aposta muda o preço?"],
                   ["Classifique a mão", "um par de ases usa", "Observe também"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m0l4 — Regras e hierarquia de mãos", scenes)


def m0l5():
    p = "m0l5"
    scenes = [
        intro(f"{p}-01", "Suas primeiras decisões e posições",
              "Esta é a Zero Tilt Academy. Na aula cinco do módulo zero, vamos estudar as primeiras decisões e a posição. "
              "O objetivo é saber quem age depois de você em cada street e separar o preço de um call da força da mão.",
              "Zero Tilt Academy · Módulo 0 · Aula 5", "Objetivo: ordem de ação por street e preço separado da força"),
        positions(f"{p}-02", "Uma volta ao botão", "A ordem muda com a street",
                  "Considere uma mesa com três lugares: o botão, o small blind e o big blind, nessa ordem. "
                  "No pré-flop, o botão age primeiro, porque está logo depois do big blind, e os blinds ainda agem depois dele. "
                  "No flop, o small blind começa e o botão termina, se os três continuarem.",
                  [("BTN · você", "1º no pré-flop · último no flop", "o botão termina", True), ("SB", "1º no flop", "o small blind começa", False),
                   ("BB", "último no pré-flop", "logo depois do big blind", False)],
                  button=0, note="A mesma cadeira muda de ordem conforme a street"),
        steps(f"{p}-03", "Posição", "Posição é informação adicional",
              "O botão age por último depois do flop, quando continua na mão. Já a posição inicial, o UTG, tem mais adversários por agir no pré-flop, "
              "mas fica em posição depois do flop contra os blinds. E posição não revela cartas escondidas, nem garante lucro. "
              "Um jogador em all-in continua disputando o pote, mas não recebe novos turnos de aposta.",
              ["Botão: último depois do flop", "UTG: em posição contra os blinds", "Posição não revela cartas"],
              ["age por último", "a posição inicial", "não revela cartas"]),
        steps(f"{p}-04", "Plano inicial", "Um plano inicial de abertura",
              "Para o exercício, a mesa é de Texas cash com nove jogadores, cem big blinds de stack e blinds de meio e de um big blind. "
              "Abrir com dois ases no botão e desistir de sete e dois de naipes diferentes no UTG são escolhas do plano didático. "
              "O tamanho de abertura é uma hipótese de treino: rake, stacks e respostas dos adversários mudam o range.",
              ["Texas cash, 9 jogadores, 100 BB", "AA no botão abre · 72o no UTG desiste", "Tamanho: hipótese de treino"],
              ["Para o exercício", "Abrir com dois ases", "O tamanho de abertura"]),
    ]
    t = Mesa(["Você · BTN", "BB"], [["Kh", "Jd"], ["Ac", "Kc"]], stacks=[9500, 9500], dealer=0, seats=[0, 2], board=["Ks", "7d", "3c"], pot=1000)
    scenes.append(t.scene(f"{p}-05", "Preço e força", "O rival aposta metade do pote",
                          "Agora separe o preço da força. No flop, o pote tem dez big blinds, e a mesa mostra rei, sete e três. "
                          "Você tem rei e valete, um top pair. O rival aposta cinco big blinds.",
                          [ev("bet", "O rival aposta", player=1, to=500)], banner="Flop · pote de 10 BB"))
    assert t.total() == 1500 and 500 / (1500 + 500) == 0.25
    scenes += [
        metrics(f"{p}-06", "Preço e força", "Preço não é força",
                "Pagar cinco forma um pote final de vinte big blinds: o preço é cinco sobre vinte, vinte e cinco por cento. "
                "Mas ter top pair não informa sozinho a sua equity contra o range do rival, e no flop as apostas futuras também influenciam a decisão.",
                [("CUSTO DO CALL", "5 BB"), ("POTE FINAL", "20 BB"), ("PREÇO · 5/20", "25%")],
                ["Pagar cinco", "pote final de vinte", "cinco sobre vinte"], note="Top pair não informa a equity sozinho"),
        metrics(f"{p}-07", "Stack efetivo", "O menor stack limita a disputa",
                "Agora vamos ao stack efetivo, o menor dos dois stacks. Em heads-up, se você tem cem big blinds e o rival tem quarenta, "
                "o máximo que vocês disputam entre si é quarenta de cada lado. O excesso não pode ser ganho desse rival nesta mão.",
                [("SEU STACK", "100 BB"), ("STACK DO RIVAL", "40 BB"), ("STACK EFETIVO", "40 BB")],
                ["cem big blinds", "o rival tem quarenta", "quarenta de cada lado"]),
        steps(f"{p}-08", "Plano e observação", "Planeje e observe",
              "Antes de abrir, planeje uma resposta para um call e para um raise. Identifique quais flops permitem valor, quais exigem cautela "
              "e quanto você aceita investir em cada hipótese. E lembre: agir depois revela uma ação, e não a mão escondida. "
              "Um check pode vir de fraqueza, de controle do pote ou de armadilha.",
              ["Plano para call e para raise", "Flops de valor e de cautela", "Uma ação não revela a mão"],
              ["planeje uma resposta", "Identifique quais flops", "agir depois revela"]),
    ]
    scenes += quiz(p, "Mesa de seis: o UTG abre, o botão paga, o small blind desiste e o big blind paga. Quem age primeiro no flop?",
                   [("A", "O UTG, que abriu", "o UTG"), ("B", "O big blind", "o jogador do big blind"), ("C", "O botão, que pagou", "o botão")], "B",
                   "Exercício. Numa mesa de seis, o UTG abre, o botão paga, o small blind desiste e o big blind paga. Quem age primeiro no flop? "
                   "A: o UTG, que abriu a ação. B: o jogador do big blind. C: o botão, que pagou. Pause o vídeo e responda.",
                   "Resposta: B. Depois do flop, a ação começa no primeiro jogador ainda ativo à esquerda do botão. "
                   "O small blind desistiu, então quem começa é o big blind, e o botão fala por último.",
                   "Depois do flop: primeiro ativo à esquerda do botão")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Reconheça quem age antes e depois do flop. "
                   "Com dois jogadores, o botão paga o small blind: age primeiro no pré-flop e por último depois do flop.",
                   ["Quem age antes e depois do flop", "Dois jogadores: o botão é o SB", "Primeiro no pré-flop, último depois"],
                   ["Reconheça quem age", "Com dois jogadores", "age primeiro no pré-flop"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m0l5 — Suas primeiras decisões e posições", scenes)


def m0l6():
    p = "m0l6"
    scenes = [
        intro(f"{p}-01", "Primeiros passos aqui",
              "Esta é a Zero Tilt Academy. Na aula seis do módulo zero, vamos conhecer o ambiente da plataforma e da Academy. "
              "O objetivo é saber onde praticar, com que fichas e como guardar o seu processo de estudo.",
              "Zero Tilt Academy · Módulo 0 · Aula 6", "Objetivo: onde praticar, com que fichas e como guardar o processo"),
        steps(f"{p}-02", "Conheça o ambiente", "Vídeo, leitura, exercícios e mesas",
              "A Academy combina vídeo, leitura, exercícios e mesas de estudo. Comece pelas regras comuns e depois escolha a modalidade. "
              "O lobby identifica separadamente as mesas de Play Money e as de Jogo Real, e nenhuma delas se mistura com as fichas sem valor dos simuladores.",
              ["Vídeo, leitura, exercícios e mesas", "Regras comuns, depois a modalidade", "Play Money, Jogo Real e simulador: separados"],
              ["A Academy combina", "Comece pelas regras", "O lobby identifica"]),
        steps(f"{p}-03", "Conta e acesso", "Cadastro e progresso",
              "No cadastro, informe o código de convite exigido pela plataforma e conclua a verificação de e-mail. "
              "Na Academy, a conta registra o progresso das aulas. E as mesas de estudo estão disponíveis para praticar sem depósito.",
              ["Código de convite e verificação de e-mail", "A conta registra o progresso", "Mesas de estudo sem depósito"],
              ["No cadastro", "a conta registra", "as mesas de estudo"]),
        compare(f"{p}-04", "Mesas", "Mesa do lobby e mesa do módulo",
                "Uma mesa do lobby precisa de pelo menos dois participantes para começar uma mão. Antes de ocupar o assento, confira a modalidade, os blinds, "
                "o tipo de saldo e a frente exigida. Já a mesa do módulo cria os bots do exercício, para você estudar sozinho, "
                "com fichas sem valor que não movimentam a sua carteira.",
                [("Mesa do lobby", [("Pelo menos dois participantes", "pelo menos dois"), ("Modalidade, blinds e saldo", "confira a modalidade")], "gold"),
                 ("Mesa do módulo", [("Bots do exercício", "cria os bots"), ("Fichas sem valor", "fichas sem valor")], "mint")]),
        steps(f"{p}-05", "Três modos de praticar", "Guiado, desafio e prática variada",
              "Há três modos de praticar. No treino guiado, leia o objetivo e a dica antes de agir. "
              "No desafio, as dicas ficam escondidas, e você responde à pergunta objetiva com as suas contas. "
              "Na prática variada, você circula pelas situações do módulo e se concentra no plano de jogo.",
              ["Treino guiado: objetivo e dica", "Desafio: sem dicas, com pergunta objetiva", "Prática variada: plano de jogo"],
              ["No treino guiado", "No desafio", "Na prática variada"]),
    ]
    t = Mesa(["Você · BTN", "Bot · BB"], [["Qs", "Js"], ["Th", "9h"]], stacks=[9750, 9750], dealer=0, seats=[0, 2], board=["Qh", "7c", "2d"], pot=500)
    scenes.append(t.scene(f"{p}-06", "Uma mão completa", "Os bots respondem a ações reais",
                          "Os bots respondem a ações reais. Veja uma mão até o fim. No flop, o pote tem cinco big blinds, e o bot aposta dois. "
                          "Pagar custa dois, porque o call cobre a aposta: você paga. No turn, o bot dá check, e você também dá check, que não custa fichas. "
                          "No river, os dois dão check, e o showdown mostra as cartas: o seu par de damas leva o pote.",
                          [ev("bet", "o bot aposta dois", player=1, to=200), ev("call", "você paga", player=0, to=200), ev("collect", None),
                           ev("board", "No turn", cards=["5s"]), ev("check", "o bot dá check", player=1), ev("check", "você também dá check", player=0),
                           ev("board", "No river", cards=["Kc"]), ev("check", "os dois dão check", player=1), ev("check", None, player=0),
                           ev("reveal", "o showdown mostra", players=[1]), ev("award", "leva o pote", player=0)],
                          banner="Flop · pote de 5 BB"))
    assert t.stacks == [9550 + 900, 9550] and best(["Qs", "Js"], t.board) > best(["Th", "9h"], t.board)
    scenes += [
        steps(f"{p}-07", "Guarde o processo", "Caderno e replay",
              "O caderno registra as suas observações neste navegador. Anote a hipótese sobre o rival, a ação escolhida e uma alternativa. "
              "Use o replay para rever o que estava visível em cada instante. E lembre que o resultado do treino não é promessa de retorno em mesas reais.",
              ["Caderno: neste navegador", "Hipótese, ação e alternativa", "Replay: o que estava visível"],
              ["O caderno registra", "Anote a hipótese", "Use o replay"]),
    ]
    scenes += quiz(p, "Onde ficam as anotações que você faz no caderno da Academy?",
                   [("A", "Na sua carteira", "na sua carteira"), ("B", "Neste navegador", "neste navegador"), ("C", "No replay da mesa", "no replay")], "B",
                   "Exercício. Onde ficam as anotações que você faz no caderno da Academy? A: na sua carteira. B: neste navegador. C: no replay que a mesa guarda. "
                   "Pause o vídeo e responda.",
                   "Resposta: B. O caderno registra as observações neste navegador. O replay mostra o que estava visível em cada instante, "
                   "e a carteira não tem relação com as fichas sem valor das mesas de estudo.",
                   "Caderno: neste navegador")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Percorra o pré-flop, o flop, o turn e o river com ações legais. "
                   "Leia quanto falta pagar: check não custa fichas, e call cobre uma aposta.",
                   ["Pré-flop, flop, turn e river", "Leia quanto falta pagar", "Check não custa; call cobre"],
                   ["Percorra o pré-flop", "Leia quanto falta", "check não custa"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m0l6 — Primeiros passos aqui", scenes)


def m0l7():
    p = "m0l7"
    scenes = [
        intro(f"{p}-01", "Aumento ou descarte",
              "Esta é a Zero Tilt Academy. Na aula sete do módulo zero, vamos estudar a primeira decisão: aumentar ou descartar a mão num pote fechado. "
              "O objetivo é abrir com um plano e calcular o risco de um roubo de blinds.",
              "Zero Tilt Academy · Módulo 0 · Aula 7", "Objetivo: abrir com um plano e calcular o risco do roubo"),
        steps(f"{p}-02", "Agressão seletiva", "Raise ou fold no pote fechado",
              "Em potes ainda fechados no Texas cash, um plano de raise ou fold pode simplificar as decisões iniciais. "
              "É uma estratégia didática, com hipóteses, e não uma lei do poker: completar no small blind e estratégias de limp podem existir em outros contextos.",
              ["Pote fechado: raise ou fold", "Estratégia didática, não lei", "Limp e completar: outros contextos"],
              ["Em potes ainda fechados", "estratégia didática", "completar no small blind"]),
        compare(f"{p}-03", "Os dois extremos", "Dois ases no botão, sete e dois no UTG",
                "No cenário da aula, a mesa tem nove jogadores e cem big blinds, sem ante. Com dois ases no botão, depois de todos desistirem, "
                "a intenção é construir valor contra mãos piores. Com sete e dois de naipes diferentes no UTG, a dificuldade de realizar equity "
                "contra muitos adversários sustenta o descarte. Entre esses extremos existe uma região que depende das condições do jogo, e não uma fronteira universal.",
                [("AA no botão", [("Todos desistiram até você", "depois de todos desistirem"), ("Aumentar por valor", "construir valor")], "mint"),
                 ("72o no UTG", [("Muitos adversários por agir", "contra muitos adversários"), ("Descartar", "sustenta o descarte")], "danger")],
                note="Entre os extremos: depende do jogo", note_cue="Entre esses extremos"),
    ]
    t = Mesa(["Você · BTN", "SB", "BB"], [["Ah", "As"], ["9c", "4d"], ["Jd", "6s"]], dealer=0, seats=[0, 1, 2])
    scenes.append(t.scene(f"{p}-04", "O plano na mesa", "Todos desistem até você",
                          "Veja o plano na mesa. Os blinds entram com meio e com um big blind, como sempre nesta mesa. Todos desistem até você, no botão, com dois ases. "
                          "Você aumenta para dois big blinds e meio. O small blind desiste, o big blind também, e você leva o pote.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("raise", "Você aumenta", player=0, to=250), ev("fold", "O small blind desiste", player=1),
                           ev("fold", "o big blind também", player=2), ev("collect", None), ev("award", "você leva o pote", player=0)],
                          banner="Pré-flop · pote fechado"))
    assert t.stacks[0] == STACK + 150
    assert 2.5 / 4 == 0.625 and abs(3 / 4.5 - 0.6667) < 0.0001
    scenes += [
        metrics(f"{p}-05", "Risco do roubo", "Quantos folds o roubo exige",
                "Agora o risco desse roubo, num modelo simplificado. Há um big blind e meio nos blinds, e você abre arriscando dois e meio, "
                "supondo que perde a aposta sempre que alguém continua. O blefe puro empata quando todos desistem em dois e meio sobre quatro: "
                "sessenta e dois e meio por cento das vezes.",
                [("NOS BLINDS", "1,5 BB"), ("RISCO DA ABERTURA", "2,5 BB"), ("FOLDS · 2,5/4", "62,5%")],
                ["Há um big blind e meio", "abre arriscando", "dois e meio sobre quatro"], note="Modelo: perde sempre que alguém continua"),
        steps(f"{p}-06", "O modelo isola uma parte", "A mão real tem mais ramos",
              "Na mão real, você ainda pode ganhar depois de receber um call, e também pode enfrentar uma 3-bet. "
              "Esse modelo isola uma parte da decisão e não descreve toda a rentabilidade de abrir. "
              "E um plano de abertura não responde automaticamente a um aumento: diante de uma 3-bet, fold, call e nova agressão dependem do range, do tamanho e dos stacks.",
              ["Depois do call, ainda pode ganhar", "O modelo isola uma parte", "Diante de 3-bet: outro plano"],
              ["ainda pode ganhar", "isola uma parte", "diante de uma 3-bet"]),
        steps(f"{p}-07", "Revisão", "O resultado não corrige o plano",
              "Anote o plano antes de ver o resultado. Ganhar uma mão não torna qualquer decisão correta, e perder não prova um erro. "
              "Se uma mão fraca ganhar depois de uma abertura ruim, faça a revisão no ponto da decisão.",
              ["Plano antes do resultado", "Ganhar não prova acerto", "Revise no ponto da decisão"],
              ["Anote o plano", "Ganhar uma mão", "faça a revisão"]),
    ]
    scenes += quiz(p, "No mesmo modelo, há 1,5 BB nos blinds e você abre para 3 BB. Com que frequência todos precisam desistir para o blefe puro empatar?",
                   [("A", "50%", "cinquenta"), ("B", "≈ 66,7%", "cerca de sessenta e seis"), ("C", "75%", "setenta e cinco")], "B",
                   "Exercício. No mesmo modelo, há um big blind e meio nos blinds, e você abre para três big blinds. "
                   "Com que frequência todos precisam desistir para o blefe puro empatar? A: cinquenta por cento. "
                   "B: cerca de sessenta e seis vírgula sete por cento. C: setenta e cinco por cento. Pause o vídeo e responda.",
                   "Resposta: B. Você arrisca três para ganhar um e meio. O total em jogo é quatro e meio, e três sobre quatro e meio dá dois terços, "
                   "cerca de sessenta e seis vírgula sete por cento. Nesse modelo, abrir maior exige mais folds.",
                   "3 / (3 + 1,5) ≈ 66,7%")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. O cenário tem três lugares, e a política do treino é seletiva. "
                   "Não transforme duas cartas ruins em obrigação de blefar.",
                   ["Três lugares, política seletiva", "Cartas ruins não obrigam blefe", "Guiado, desafio e replay"],
                   ["três lugares", "Não transforme", "Comece pelo treino guiado"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m0l7 — Aumento ou descarte: a primeira decisão", scenes)


def m0l8():
    p = "m0l8"
    scenes = [
        intro(f"{p}-01", "Sizing: o tamanho certo",
              "Esta é a Zero Tilt Academy. Na aula oito do módulo zero, vamos estudar o tamanho das apostas. "
              "O objetivo é declarar o total da aposta, saber o mínimo de um raise e ver o preço que cada tamanho oferece.",
              "Zero Tilt Academy · Módulo 0 · Aula 8", "Objetivo: total da aposta, mínimo do raise e preço de cada tamanho"),
        steps(f"{p}-02", "Tamanho e unidade", "Declare o total",
              "Uma abertura para dois big blinds e meio significa um total de dois e meio, e não acrescentar esse valor a uma aposta anterior. "
              "Quando já houver aposta, confira o mínimo de raise e o stack efetivo. "
              "E lembre que bet e raise são ações diferentes: bet inicia a aposta, e raise aumenta uma aposta existente.",
              ["2,5 BB é o total", "Confira o mínimo e o stack efetivo", "Bet inicia, raise aumenta"],
              ["significa um total", "confira o mínimo", "bet e raise"]),
    ]
    t = Mesa(["Você · BTN", "SB", "BB", "CO"], [["Ac", "Qd"], ["8c", "3d"], ["Ks", "4h"], ["7h", "7d"]], dealer=0)
    scenes.append(t.scene(f"{p}-03", "Exemplo de plano", "Isolar um limper",
                          "Veja o plano de isolamento. Os blinds entram, e o jogador do cutoff apenas completa um big blind: um limp. "
                          "No botão, o plano da aula soma um big blind por limper à abertura de dois e meio. Com um limper, você aumenta para três e meio. "
                          "Os blinds desistem, o cutoff paga, e o pote do flop fica com oito big blinds e meio.",
                          [ev("post", "Os blinds entram", player=1, to=50, label="SB"), ev("post", None, player=2, to=100, label="BB"),
                           ev("call", "apenas completa", player=3, to=100, label="Limp"), ev("raise", "você aumenta para três e meio", player=0, to=350),
                           ev("fold", "Os blinds desistem", player=1), ev("fold", None, player=2),
                           ev("call", "o cutoff paga", player=3, to=350), ev("collect", "o pote do flop")],
                          banner="Pré-flop · um limper"))
    assert t.total() == 850
    scenes += [
        steps(f"{p}-04", "Exemplo de plano", "Uma heurística de treino",
              "Com dois limpers, a mesma regra leva a quatro e meio. Mas essa regra de treino não prova que o tamanho é ótimo: "
              "posição, rake, stacks, adversários e estrutura de apostas mudam a análise.",
              ["Dois limpers: 4,5 BB", "Regra de treino, não tamanho ótimo", "Posição, rake, stacks e rivais mudam"],
              ["Com dois limpers", "não prova", "posição, rake"]),
        metrics(f"{p}-05", "Preço produzido", "Cada tamanho oferece um preço",
                "Agora o preço que cada tamanho oferece. Com pote de dez big blinds e aposta de cinco, o rival paga cinco para um pote final de vinte: "
                "precisa de vinte e cinco por cento de equity num call final sem taxas. Com aposta de dez, o pote final é de trinta, "
                "e o preço sobe para trinta e três vírgula três por cento. Apostar maior oferece preço pior a quem paga, mas arrisca mais e pode fazer mãos piores desistirem.",
                [("POTE ANTES DA APOSTA", "10 BB"), ("APOSTA DE 5 · 5/20", "25%"), ("APOSTA DE 10 · 10/30", "33,33%")],
                ["pote de dez big blinds", "vinte e cinco por cento", "trinta e três"], note="Maior: preço pior para quem paga e risco maior"),
        steps(f"{p}-06", "Incremento e mínimo", "O mínimo de um raise",
              "Se a aposta vigente é de cem fichas e o último aumento completo foi de cem, o próximo raise completo precisa chegar a pelo menos duzentas. "
              "Depois de um raise de cem para trezentas, o incremento é de duzentas, e o próximo total mínimo é quinhentas. "
              "A interface mostra o total da rodada: subtraia o que você já investiu para saber o adicional.",
              ["Aposta 100, aumento 100: mínimo 200", "De 100 para 300: mínimo 500", "Total menos o investido = adicional"],
              ["a aposta vigente", "Depois de um raise", "subtraia"]),
        steps(f"{p}-07", "Compare respostas", "Dois tamanhos, a mesma distribuição",
              "Repita uma distribuição e experimente dois tamanhos legais. Observe quais bots pagam, desistem ou aumentam. "
              "Uma única resposta não estima a frequência real de um perfil. Registre também o stack que sobra para a próxima street.",
              ["Dois tamanhos legais", "Quem paga, desiste ou aumenta", "Registre o stack que sobra"],
              ["experimente dois tamanhos", "Observe quais bots", "Registre também"]),
    ]
    assert 900 - 100 == 800
    scenes += quiz(p, "Você está no big blind com 1 BB investido. O botão abriu para 2,5 BB, e você quer aumentar para 9 BB no total. Quanto entra a mais?",
                   [("A", "9 BB", "os nove"), ("B", "8 BB", "oito"), ("C", "6,5 BB", "seis")], "B",
                   "Exercício. Você está no big blind, com um big blind já investido. O botão abriu para dois e meio, e você quer aumentar para nove big blinds no total. "
                   "Quanto entra a mais? A: os nove big blinds inteiros. B: oito big blinds a mais. C: seis big blinds e meio. Pause o vídeo e responda.",
                   "Resposta: B. O controle mostra o total da rodada, nove big blinds. Como você já tinha um big blind investido, entram mais oito. "
                   "A conta de seis e meio usa a diferença para a abertura, que não é o que você coloca.",
                   "Total 9 − 1 já investido = 8 BB a mais")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Para subir a três big blinds com meio big blind já investido, entram mais dois e meio. "
                   "O controle mostra o total.",
                   ["Subir a 3 BB com 0,5 investido", "Entram mais 2,5 BB", "O controle mostra o total"],
                   ["Para subir a três", "entram mais dois e meio", "O controle mostra"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m0l8 — Sizing: o tamanho certo", scenes)


# ─────────────────────────────── Módulo 5 (Short Deck e Omaha) ───────────────────────────────
def m5l1():
    p = "m5l1"
    scenes = [
        intro(f"{p}-01", "Short Deck: 36 cartas",
              "Esta é a Zero Tilt Academy. Na aula um do módulo cinco, vamos estudar o Texas Hold'em Short Deck, com trinta e seis cartas. "
              "O objetivo é refazer o ranking e o denominador com o baralho curto, sem importar probabilidades do Texas tradicional.",
              "Zero Tilt Academy · Módulo 5 · Aula 1", "Objetivo: refazer ranking e denominador com 36 cartas"),
        steps(f"{p}-02", "Regras", "Texas Hold'em Short Deck",
              "No Short Deck da ZT, para até oito jogadores, o baralho tem trinta e seis cartas: saem os dois, os três, os quatros e os cincos. "
              "Cada jogador recebe duas cartas fechadas, como no Texas tradicional, e no showdown vale a melhor combinação de cinco entre as duas da mão e as cinco da mesa. "
              "Mas o ranking muda: trinca vence sequência, e flush vence full house. As regras de Short Deck variam entre plataformas, e aqui estudamos as da ZT.",
              ["36 cartas: sem 2, 3, 4 e 5", "Duas privadas, melhor mão de cinco", "Trinca > sequência · flush > full house"],
              ["trinta e seis cartas", "duas cartas fechadas", "trinca vence sequência"]),
        steps(f"{p}-03", "Ranking local", "Do menor para o maior",
              "A ordem local, do menor para o maior, é carta alta, par, dois pares, sequência, trinca, full house, flush, quadra, straight flush e royal flush.",
              ["Carta alta · par · dois pares", "Sequência", "Trinca", "Full house · flush", "Quadra · straight flush · royal"],
              ["carta alta", "sequência", "trinca", "full house", "quadra"]),
    ]
    t = Mesa(["Você · BTN", "BB"], [["Ad", "6s"], ["Th", "6c"]], stacks=[9000, 9000], dealer=0, seats=[0, 2],
             board=["7c", "8d", "9h", "Ks", "Qd"], pot=2000, short=True)
    scenes.append(t.scene(f"{p}-04", "O ás abaixo do seis", "Ás, seis, sete, oito e nove",
                          "Veja o ás abaixo do seis. A mesa mostra sete, oito, nove, rei e dama. Você tem ás e seis: a sequência baixa do Short Deck, "
                          "ás, seis, sete, oito e nove, que termina no nove. Ela não vira uma sequência de ás alto. "
                          "O rival tem dez e seis: seis, sete, oito, nove e dez, que termina no dez e vence.",
                          [ev("highlight", "a sequência baixa", player=0, hole=[0, 1], boardPick=[0, 1, 2], text="A-6-7-8-9"),
                           ev("reveal", "O rival tem", players=[1]), ev("award", "termina no dez e vence", player=1)],
                          banner="River · Short Deck · pote de 20 BB"))
    assert best(["Ad", "6s"], t.board, short=True) == (3, 9) and best(["Th", "6c"], t.board, short=True) == (3, 10)
    known = ["Ah", "Kh", "Qh", "9h", "Tc", "7s"]
    assert len(deck(known, short=True)) == 30 and sum(c[1] == "h" for c in deck(known, short=True)) == 5
    scenes += [
        metrics(f"{p}-05", "Refaça o denominador", "Trinta cartas desconhecidas",
                "Agora refaça o denominador. No turn, você conhece duas privadas e quatro comunitárias. Do baralho de trinta e seis, restam trinta desconhecidas. "
                "Se quatro copas estão entre as seis conhecidas, restam cinco copas, porque cada naipe tem nove cartas. "
                "A chance de uma copa no river é cinco sobre trinta, cerca de dezesseis vírgula sete por cento.",
                [("DESCONHECIDAS · 36 − 6", "30"), ("COPAS RESTANTES · 9 − 4", "5"), ("COPA NO RIVER · 5/30", "≈ 16,67%")],
                ["restam trinta", "restam cinco copas", "cinco sobre trinta"], note="Mede a carta do river, não a vitória"),
        compare(f"{p}-06", "Não misture modalidades", "Baralho, ranking e seleção",
                "O nome Short Deck não altera a formação do Texas: você pode usar zero, uma ou duas privadas. "
                "Já o Omaha e o Brazilian Pineapple da plataforma usam cinquenta e duas cartas: o Omaha exige exatamente duas privadas, "
                "e o Pineapple termina com duas e forma a mão como no Texas. No treino, identifique baralho, ranking e seleção antes de calcular preços.",
                [("Short Deck", [("36 cartas, ranking próprio", "não altera a formação"), ("Usa 0, 1 ou 2 privadas", "zero, uma ou duas")], "gold"),
                 ("Omaha e Pineapple", [("52 cartas", "cinquenta e duas cartas"), ("Omaha: exatamente 2", "exige exatamente duas")], "mint")],
                note="Baralho, ranking e seleção antes do preço", note_cue="identifique baralho"),
    ]
    scenes += quiz(p, "Short Deck, flop: você conhece as suas duas cartas e as três comunitárias. Quantas cartas continuam desconhecidas?",
                   [("A", "47", "quarenta e sete"), ("B", "31", "trinta e uma"), ("C", "30", "trinta")], "B",
                   "Exercício. No Short Deck, ainda no flop, você conhece as suas duas cartas e as três comunitárias. Quantas cartas continuam desconhecidas? "
                   "A: quarenta e sete. B: trinta e uma. C: trinta. Pause o vídeo e responda.",
                   "Resposta: B. O baralho tem trinta e seis cartas, e você conhece cinco: restam trinta e uma. "
                   "Quarenta e sete seria a conta do baralho de cinquenta e duas, e trinta é o número do turn.",
                   "36 − 5 = 31 desconhecidas")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Leia ás, seis, sete, oito e nove como a sequência baixa do Short Deck, que termina no nove. "
                   "Uma sequência até o dez é maior, e a trinca supera a sequência na ZT.",
                   ["A-6-7-8-9 termina no nove", "Até o dez é maior", "Trinca supera sequência"],
                   ["Leia ás, seis", "Uma sequência até o dez", "a trinca supera"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m5l1 — Texas Hold'em Short Deck: 36 cartas", scenes)


def m5l2():
    p = "m5l2"
    scenes = [
        intro(f"{p}-01", "Omaha de quatro cartas",
              "Esta é a Zero Tilt Academy. Na aula dois do módulo cinco, vamos estudar o Omaha de quatro cartas, com baralho de cinquenta e duas. "
              "O objetivo é escolher duas privadas e três comunitárias, e contar as combinações sem confundir com chances de vencer.",
              "Zero Tilt Academy · Módulo 5 · Aula 2", "Objetivo: duas privadas, três comunitárias e a contagem certa"),
        steps(f"{p}-02", "Omaha 4", "Duas da mão, três da mesa",
              "No Omaha de quatro cartas da ZT, para até seis jogadores, cada um recebe quatro cartas fechadas no pré-flop. "
              "A mão é formada com exatamente duas cartas da mão e três da mesa. "
              "O baralho tem cinquenta e duas cartas, e o ranking é o clássico: sequência vence trinca, e full house vence flush.",
              ["Quatro cartas fechadas", "Exatamente 2 da mão + 3 da mesa", "52 cartas, ranking clássico"],
              ["quatro cartas fechadas", "exatamente duas", "o ranking é o clássico"]),
    ]
    hero = ["Ah", "Kh", "Qc", "Jc"]
    t = Mesa(["Você · BTN", "BB"], [hero, ["Ks", "Kd", "7s", "6d"]], stacks=[9500, 9500], dealer=0, seats=[0, 2],
             board=["Th", "9h", "8c"], pot=1000, omaha=True)
    scenes.append(t.scene(f"{p}-03", "Escolha pares privados", "Qual par forma a mão",
                          "Veja um exemplo. No flop, a mesa mostra dez e nove de copas e oito de paus. Você tem ás e rei de copas, dama e valete de paus. "
                          "Com o par dama e valete, você forma uma sequência até a dama. "
                          "Já o par ás e rei de copas ainda não forma flush: há duas copas na mão, mas só duas na mesa.",
                          [ev("highlight", "uma sequência até a dama", player=0, hole=[2, 3], boardPick=[0, 1, 2], text="Sequência até a dama")],
                          banner="Flop · Omaha de 4 cartas"))
    assert best(hero, t.board, omaha=True) == (4, 12) and best(["Ah", "Kh"], t.board)[0] < 5
    assert math.comb(4, 2) == 6 and math.comb(5, 3) == 10
    scenes += [
        metrics(f"{p}-04", "Contagem e leitura", "Sessenta candidatas no river",
                "Agora conte as combinações. As quatro privadas formam seis pares. No river, as cinco comunitárias formam dez trios. "
                "São sessenta candidatas, correlacionadas, para escolher a melhor mão legal. Isso não significa sessenta chances independentes de vencer.",
                [("PARES PRIVADOS · C(4,2)", "6"), ("TRIOS NO RIVER · C(5,3)", "10"), ("CANDIDATAS · 6 × 10", "60")],
                ["seis pares", "dez trios", "sessenta candidatas"], note="Não são 60 chances de vencer"),
        compare(f"{p}-05", "Contagem e leitura", "Um ás sozinho não faz flush",
                "Um único ás de copas na mão não basta para flush, nem com quatro copas na mesa, porque a mão usa exatamente duas privadas. "
                "Esse ás pode remover combinações dos rivais, mas não fabrica um flush seu. "
                "E ter uma mão feita e um projeto ao mesmo tempo não permite usar as quatro privadas na mesma combinação.",
                [("Um ás de copas sozinho", [("Não faz flush", "não basta para flush"), ("Remove combinações rivais", "remover combinações")], "gold"),
                 ("Mão feita e projeto", [("Não usam as quatro privadas juntas", "usar as quatro")], "danger")]),
        steps(f"{p}-06", "Nuts e melhoras", "Nuts agora não garante depois",
              "Ter a melhor mão possível numa street não garante continuar na frente nas cartas seguintes. "
              "Uma mesa que pareia pode permitir full house, e uma carta do terceiro naipe pode permitir flush. "
              "Por isso, nenhuma sequência é all-in automático: a análise depende das cartas e dos ranges.",
              ["Nuts agora não garante depois", "Mesa pareada: full house possível", "Sequência não é all-in automático"],
              ["Ter a melhor mão", "Uma mesa que pareia", "nenhuma sequência"]),
        steps(f"{p}-07", "Preço com quatro cartas", "O preço continua o mesmo conceito",
              "As pot odds continuam sendo uma relação entre o custo e o pote. O que muda é a distribuição de mãos e a forma de estimar a equity. "
              "E confira na mesa a estrutura de apostas, sem presumir um limite de pote.",
              ["Pot odds: custo e pote", "Muda a distribuição de mãos", "Confira a estrutura de apostas"],
              ["As pot odds", "O que muda", "confira na mesa"]),
    ]
    scenes += quiz(p, "Omaha de quatro cartas, ainda no flop: quantas combinações candidatas de duas privadas e três comunitárias existem?",
                   [("A", "6", "seis"), ("B", "10", "dez"), ("C", "60", "sessenta")], "A",
                   "Exercício. No Omaha de quatro cartas, ainda no flop, quantas combinações candidatas de duas privadas e três comunitárias existem? "
                   "A: seis. B: dez. C: sessenta. Pause o vídeo e responda.",
                   "Resposta: A. No flop há só três comunitárias, então existe um único trio da mesa. "
                   "Os seis pares privados vezes esse trio dão seis candidatas. As sessenta aparecem só no river.",
                   "6 pares × 1 trio = 6 candidatas")
    scenes += [
        guided_lab(f"{p}-30",
                   "Agora leve a aula para a mesa de treino. Conte os seis pares privados e reconheça a sequência feita: "
                   "dama e valete de paus, com dez, nove e oito, formam sequência até a dama. Ás e rei de copas oferecem um projeto diferente.",
                   ["Conte seis pares privados", "Q♣ J♣: sequência até a dama", "A♥ K♥: outro projeto"],
                   ["Conte os seis pares", "dama e valete de paus", "Ás e rei de copas"]),
        outro(f"{p}-99"),
    ]
    write(p, "Aula m5l2 — Omaha de quatro cartas", scenes)


LESSONS = {"m1l1": m1l1, "m1l2": m1l2, "m1l3": m1l3, "m2l1": m2l1, "m2l2": m2l2, "m2l3": m2l3,
           "m3l1": m3l1, "m3l2": m3l2, "m3l3": m3l3, "m4l1": m4l1, "m4l2": m4l2, "m4l3": m4l3,
           "m6l1": m6l1, "m6l2": m6l2, "m6l3": m6l3, "m0l1": m0l1, "m0l2": m0l2, "m0l3": m0l3, "m0l4": m0l4,
           "m0l5": m0l5, "m0l6": m0l6, "m0l7": m0l7, "m0l8": m0l8, "m5l1": m5l1, "m5l2": m5l2}

if __name__ == "__main__":
    chosen = sys.argv[1:] or list(LESSONS)
    for lesson in chosen:
        LESSONS[lesson]()
    write_index()
