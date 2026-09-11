"""EP09 — Gestão de banca. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
GREEN = "#6BCB77"
MONO = "DejaVu Sans Mono"
RED_SUITS = ("♥", "♦")


def card(rank="", suit="", w=0.85, h=1.2):
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
            Text("GESTÃO DE BANCA", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episódio 9: nunca zerar", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 1.8)
        line = Text("técnica sem banca não sobrevive", font=MONO, font_size=25, color=CREAM)
        line.move_to(DOWN * 0.8)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(line), run_time=0.8)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Regra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("regra dos 30 a 50 buy-ins", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.9)
        ex = Text("NL10 (entrada 10) → 300 a 500", font=MONO, font_size=30,
                  color=CREAM, weight=BOLD)
        ex.move_to(DOWN * 0.2)
        sub = Text("reservados SÓ para o poker", font=MONO, font_size=24, color=CREAM)
        sub.next_to(ex, DOWN, buff=0.5)
        frame = SurroundingRectangle(ex, color=GOLD, buff=0.25, stroke_width=4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.play(Create(frame), FadeIn(sub), run_time=0.8)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Variancia(Scene):
    def construct(self):
        self.camera.background_color = FELT
        aa = VGroup(card("A", "♠"), card("A", "♥")).arrange(RIGHT, buff=0.15)
        vs = Text("vs", font=MONO, font_size=26, color=CREAM)
        rag = VGroup(card("7", "♦"), card("2", "♣")).arrange(RIGHT, buff=0.15)
        row = VGroup(aa, vs, rag).arrange(RIGHT, buff=0.5).move_to(UP * 0.6)
        stat = Text("perde 1 em cada 8", font=MONO, font_size=32, color=GOLD, weight=BOLD)
        stat.next_to(row, DOWN, buff=0.6)
        norm = Text("normal. não é azar. não é tilt.", font=MONO, font_size=24, color=CREAM)
        norm.to_edge(DOWN, buff=0.9)
        self.play(FadeIn(row, shift=DOWN * 0.3), run_time=0.9)
        self.play(FadeIn(stat), run_time=0.8)
        self.play(FadeIn(norm), run_time=0.7)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Mente(Scene):
    def construct(self):
        self.camera.background_color = FELT
        bad = Text("amador: tenta recuperar no desespero", font=MONO, font_size=25, color=RED)
        good = Text("experiente: plano intacto na próxima", font=MONO, font_size=25,
                    color=GREEN, weight=BOLD)
        grp = VGroup(bad, good).arrange(DOWN, buff=0.5)
        self.play(FadeIn(bad, shift=RIGHT * 0.3), run_time=0.8)
        self.play(FadeIn(good, shift=RIGHT * 0.3), run_time=0.8)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("banca é cinto: não evita a batida", font=MONO, font_size=28, color=CREAM)
        nxt = Text("episódio final: revisão do método", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(7.0)
