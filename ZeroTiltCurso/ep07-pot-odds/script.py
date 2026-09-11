"""EP07 — Pot odds. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
GREEN = "#6BCB77"
MONO = "DejaVu Sans Mono"
RED_SUITS = ("♥", "♦")


def card(rank="", suit="", w=0.8, h=1.15):
    rect = RoundedRectangle(
        corner_radius=0.08, width=w, height=h,
        fill_opacity=1, fill_color=CREAM, stroke_color=GOLD, stroke_width=3,
    )
    color = RED if suit in RED_SUITS else INK
    r = Text(rank, font=MONO, font_size=26, color=color, weight=BOLD)
    s = Text(suit, font=MONO, font_size=28, color=color)
    return VGroup(rect, VGroup(r, s).arrange(DOWN, buff=0.05))


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("POT ODDS", font=MONO, font_size=44, color=GOLD, weight=BOLD),
            Text("episódio 7: pagar é negócio", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 1.8)
        line = Text("curiosidade não paga aposta", font=MONO, font_size=26, color=CREAM)
        line.move_to(DOWN * 0.8)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(line), run_time=0.8)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Conta(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("pote 8 + aposta 2: custa 2 para levar 12", font=MONO, font_size=24,
                   color=CREAM)
        tag.to_edge(UP, buff=0.8)
        pot = Rectangle(width=4.0, height=0.9, fill_opacity=1, fill_color=GOLD,
                        stroke_width=0).move_to(UP * 0.5)
        pot_label = Text("pote 10", font=MONO, font_size=24, color=INK, weight=BOLD)
        pot_label.move_to(pot.get_center())
        call = Rectangle(width=0.8, height=0.9, fill_opacity=1, fill_color=RED,
                         stroke_width=0).next_to(pot, RIGHT, buff=0.3)
        call_label = Text("2", font=MONO, font_size=24, color=CREAM, weight=BOLD)
        call_label.move_to(call.get_center())
        math = Text("2 / 12 = 16,6%", font=MONO, font_size=34, color=GOLD, weight=BOLD)
        math.to_edge(DOWN, buff=1.0)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(pot), FadeIn(pot_label), run_time=0.8)
        self.play(FadeIn(call), FadeIn(call_label), run_time=0.8)
        self.play(FadeIn(math), run_time=0.8)
        self.wait(9.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Outs(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("flush draw: 4 na mão+mesa, 9 no baralho", font=MONO, font_size=24, color=CREAM)
        tag.to_edge(UP, buff=0.8)
        flush = VGroup(card("A", "♠"), card("K", "♠"),
                       card("7", "♠"), card("2", "♠")).arrange(RIGHT, buff=0.15)
        flush.move_to(UP * 0.4)
        rule = Text("regra do 2: 9 x 2 = 18%", font=MONO, font_size=32,
                    color=GOLD, weight=BOLD)
        rule.to_edge(DOWN, buff=1.0)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(flush, shift=DOWN * 0.3), run_time=0.9)
        self.play(FadeIn(rule), run_time=0.8)
        self.wait(9.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Decisao(Scene):
    def construct(self):
        self.camera.background_color = FELT
        cmp = Text("18%  >  16,6%", font=MONO, font_size=40, color=GREEN, weight=BOLD)
        veredict = Text("CALL: lucrativo no longo prazo", font=MONO, font_size=26, color=CREAM)
        grp = VGroup(cmp, veredict).arrange(DOWN, buff=0.5)
        frame = SurroundingRectangle(cmp, color=GREEN, buff=0.25, stroke_width=5)
        self.play(FadeIn(cmp), run_time=0.9)
        self.play(Create(frame), FadeIn(veredict), run_time=0.9)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("outs x 2 contra o preço do pote", font=MONO, font_size=28, color=CREAM)
        nxt = Text("próximo episódio: EV e fold equity", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(6.5)
