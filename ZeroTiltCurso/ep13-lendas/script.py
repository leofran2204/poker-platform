"""EP13 — Lendas do poker. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("LENDAS DO POKER", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episodio 13: nomes que fizeram historia", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("ninguem nasceu sabendo", font=MONO, font_size=26, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(4.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_USA(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("os primeiros gigantes", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["Moss: 70 e 71", "Brunson: 10 + o 10-2", "Ungar: 3 Main Events"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Modernos(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a era moderna", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["Ivey / Negreanu / Holz", "Hellmuth: 17 braceletes"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(6.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Brasil(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o Brasil no mapa", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.4)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["2004: CPH / BSOP", "Gomes 2008 / Akkari 2011"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("uma ideia por sessao", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo modulo: ranges", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(16.0)
