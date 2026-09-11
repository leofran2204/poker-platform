"""EP03 — Aumento ou descarte. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
BACK = "#1A2A5A"
GREEN = "#6BCB77"
MONO = "DejaVu Sans Mono"
RED_SUITS = ("♥", "♦")


def card(rank="", suit="", face_down=False, w=0.9, h=1.3):
    rect = RoundedRectangle(
        corner_radius=0.08, width=w, height=h,
        fill_opacity=1, fill_color=CREAM, stroke_color=GOLD, stroke_width=3,
    )
    if face_down:
        rect.set_fill(BACK, opacity=1)
        return VGroup(rect)
    color = RED if suit in RED_SUITS else INK
    r = Text(rank, font=MONO, font_size=28, color=color, weight=BOLD)
    s = Text(suit, font=MONO, font_size=30, color=color)
    return VGroup(rect, VGroup(r, s).arrange(DOWN, buff=0.05))


def chip_tag(text, pos):
    c = Circle(radius=0.34, fill_opacity=1, fill_color=GOLD,
               stroke_color=CREAM, stroke_width=2).move_to(pos)
    t = Text(text, font=MONO, font_size=18, color=INK, weight=BOLD).move_to(pos)
    return VGroup(c, t)


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("AUMENTO OU DESCARTE", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episódio 3: por que nunca entrar de limp", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.0)
        limp = chip_tag("0,10", LEFT * 2 + DOWN * 0.8)
        limp_tag = Text("limp: convida todo mundo", font=MONO, font_size=22, color=CREAM)
        limp_tag.next_to(limp, DOWN, buff=0.4)
        rai = chip_tag("0,50", RIGHT * 2 + DOWN * 0.8)
        rai_tag = Text("raise: cobra caro", font=MONO, font_size=22, color=CREAM)
        rai_tag.next_to(rai, DOWN, buff=0.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(limp), FadeIn(limp_tag), run_time=0.8)
        self.play(FadeIn(rai), FadeIn(rai_tag), run_time=0.8)
        self.wait(6.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Limp(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("limp: 5 pagam barato e o flop acerta alguém", font=MONO,
                   font_size=24, color=RED, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        seats = VGroup(*[
            Circle(radius=0.3, fill_opacity=1, fill_color=INK,
                   stroke_color=CREAM, stroke_width=2).move_to(
                [LEFT * 3 + RIGHT * i * 1.5 for i in range(5)][i])
            for i in range(5)
        ])
        seats.move_to(UP * 0.8)
        board = VGroup(card("9", "♣"), card("5", "♥"),
                       card("2", "♦")).arrange(RIGHT, buff=0.15)
        board.move_to(DOWN * 0.9)
        who = Text("quem acertou??", font=MONO, font_size=28, color=GOLD, weight=BOLD)
        who.to_edge(DOWN, buff=0.9)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(seats, shift=DOWN * 0.3), run_time=0.8)
        self.wait(0.5)
        self.play(FadeIn(board, shift=UP * 0.3), run_time=0.8)
        self.wait(0.6)
        self.play(FadeIn(who, scale=1.2), run_time=0.8)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Raise(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("raise: duas formas de vencer", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        way1 = Text("1) todos desistem → leva na hora", font=MONO, font_size=24, color=CREAM)
        folds = VGroup(*[Text("fold", font=MONO, font_size=20, color=CREAM) for _ in range(4)])
        folds.arrange(RIGHT, buff=0.4).move_to(UP * 0.4)
        way2 = Text("2) alguém paga → você manda no pote", font=MONO, font_size=24, color=CREAM)
        cbet = Text("+ c-bet no flop leva de novo", font=MONO, font_size=22, color=GREEN)
        grp = VGroup(way1, folds, way2, cbet).arrange(DOWN, buff=0.35, aligned_edge=LEFT)
        grp.move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(way1), run_time=0.7)
        self.play(FadeIn(folds, shift=DOWN * 0.2), run_time=0.8)
        self.wait(0.6)
        self.play(FadeIn(way2), run_time=0.7)
        self.play(FadeIn(cbet), run_time=0.7)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Conta(Scene):
    def construct(self):
        self.camera.background_color = FELT
        formula = Text("3x BB + 1x BB por limper", font=MONO, font_size=32,
                       color=GOLD, weight=BOLD)
        formula.move_to(UP * 1.6)
        ex = Text("A♠ K♥ no botão, 2 limpers, NL10:", font=MONO, font_size=24, color=CREAM)
        ex.next_to(formula, DOWN, buff=0.6)
        calc = Text("3 × 0,10 + 2 × 0,10 = R$ 0,50", font=MONO, font_size=32,
                    color=CREAM, weight=BOLD)
        calc.next_to(ex, DOWN, buff=0.5)
        frame = SurroundingRectangle(calc, color=GOLD, buff=0.2, stroke_width=4)
        self.play(Write(formula), run_time=1.2)
        self.play(FadeIn(ex), run_time=0.7)
        self.play(FadeIn(calc), run_time=0.8)
        self.play(Create(frame), run_time=0.7)
        self.wait(6.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        limp = Text("limp: 1 forma de vencer", font=MONO, font_size=28, color=RED)
        rai = Text("raise: 2 formas de vencer", font=MONO, font_size=28,
                   color=GREEN, weight=BOLD)
        nxt = Text("próximo episódio: ranges por posição", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(limp, rai, nxt).arrange(DOWN, buff=0.5)
        self.play(FadeIn(limp), run_time=0.8)
        self.play(FadeIn(rai), run_time=0.8)
        self.wait(1.0)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(6.0)
