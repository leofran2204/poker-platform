"""EP04 — Ranges. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
GREEN = "#6BCB77"
DIM = "#2A3A2A"
MONO = "DejaVu Sans Mono"
RANKS = ["A", "K", "Q", "J", "T", "9", "8", "7", "6", "5", "4", "3", "2"]


def utg_on(hi, lo, suited):
    if hi == lo:
        return RANKS.index(hi) <= RANKS.index("T")
    if suited:
        return (hi, lo) in [("A", "K"), ("A", "Q")]
    return (hi, lo) == ("A", "K")


def btn_on(hi, lo, suited):
    if hi == lo:
        return True
    if hi == "A":
        return suited or RANKS.index(lo) <= RANKS.index("T")
    if hi == "K":
        return suited and RANKS.index(lo) <= RANKS.index("9")
    if suited:
        return RANKS.index(hi) - RANKS.index(lo) <= 3 and RANKS.index(hi) <= RANKS.index("J")
    return False


def matrix(pred, cell=0.30):
    cells = VGroup()
    for i, hi in enumerate(RANKS):
        for j, lo in enumerate(RANKS):
            if i == j:
                on, tag = pred(hi, lo, True), hi + lo
            elif j > i:
                on, tag = pred(hi, lo, True), hi + lo + "s"
            else:
                on, tag = pred(lo, hi, False), lo + hi + "o"
            sq = Square(side_length=cell, fill_opacity=1,
                        fill_color=GREEN if on else DIM,
                        stroke_color=FELT, stroke_width=1)
            sq.move_to(RIGHT * j * cell + DOWN * i * cell)
            sq._tag = (tag, on)
            cells.add(sq)
    return cells


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("RANGES", font=MONO, font_size=44, color=GOLD, weight=BOLD),
            Text("episódio 4: o que jogar de cada posição", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.6)
        m = matrix(lambda *_: False, cell=0.22)
        m.move_to(DOWN * 0.6)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(m, shift=UP * 0.2), run_time=1.0)
        self.wait(5.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Notation(Scene):
    def construct(self):
        self.camera.background_color = FELT
        rows = VGroup(
            Text("AKs  = Ás-Rei do MESMO naipe (suited)", font=MONO, font_size=26, color=CREAM),
            Text("AKo = Ás-Rei de naipes DIFERENTES (offsuit)", font=MONO, font_size=26, color=CREAM),
            Text("22+  = qualquer par, do 2 ao Ás (para cima)", font=MONO, font_size=26, color=CREAM),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.5)
        self.play(FadeIn(rows[0], shift=RIGHT * 0.3), run_time=0.8)
        self.wait(0.4)
        self.play(FadeIn(rows[1], shift=RIGHT * 0.3), run_time=0.8)
        self.wait(0.4)
        self.play(FadeIn(rows[2], shift=RIGHT * 0.3), run_time=0.8)
        self.wait(7.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_UTG(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("UTG ~12%: só elite (TT+, AK, AQs)", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.7)
        m = matrix(utg_on, cell=0.34)
        m.next_to(tag, DOWN, buff=0.5)
        sub = Text("8 jogadores atrás: rigor absoluto", font=MONO, font_size=22, color=CREAM)
        sub.to_edge(DOWN, buff=0.8)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(m, shift=UP * 0.2), run_time=1.0)
        self.play(FadeIn(sub), run_time=0.7)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_BTN(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("BTN ~45%: ataque amplo", font=MONO, font_size=26,
                   color=GREEN, weight=BOLD)
        tag.to_edge(UP, buff=0.7)
        m = matrix(btn_on, cell=0.34)
        m.next_to(tag, DOWN, buff=0.5)
        sub = Text("último a falar: informação vira lucro", font=MONO, font_size=22, color=CREAM)
        sub.to_edge(DOWN, buff=0.8)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(m, shift=UP * 0.2), run_time=1.0)
        self.play(FadeIn(sub), run_time=0.7)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("cedo: elite · tarde: ataque", font=MONO, font_size=30, color=CREAM)
        nxt = Text("próximo episódio: o tamanho do aumento", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(6.0)
