"""EP11 — De onde vêm as cartas. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("DE ONDE VEM AS CARTAS", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("episodio 11: mil anos de baralho", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("as cartas tem mil anos", font=MONO, font_size=26, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(4.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_China(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("China, sec. IX — dinastia Tang", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["tiras de papel com simbolos",
                      "passatempo da corte",
                      "viajou pela Rota da Seda"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Mamelucos(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("do Oriente para a Franca", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        old = Text("Mamelucos: 4 naipes + rei", font=MONO, font_size=26, color=CREAM)
        new = Text("Franca: espadas copas ouros paus", font=MONO, font_size=26, color=CREAM)
        grp = VGroup(old, new).arrange(DOWN, buff=0.5).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(old, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(new, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_52(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("13 x 4 = 52 cartas", font=MONO, font_size=34,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.2)
        ex = Text("cruzou o oceano, virou poker", font=MONO, font_size=26, color=CREAM)
        ex.next_to(tag, DOWN, buff=0.7)
        frame = SurroundingRectangle(ex, color=GOLD, buff=0.2, stroke_width=4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.play(Create(frame), run_time=0.6)
        self.wait(7.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("informacao incompleta: pagar para ver", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: Mississippi e o blefe", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(11.7)
