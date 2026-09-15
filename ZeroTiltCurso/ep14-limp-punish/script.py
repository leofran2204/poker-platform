"""EP14 — Puna o limp. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("PUNA O LIMP", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 14: 3,5x a 4x", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("SB so pagou: e um convite", font=MONO, font_size=26, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(4.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Vantagens(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("3 vantagens no BB", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["posicao absoluta", "range capped: sem AA KK QQ AK", "iniciativa"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Tamanho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o tamanho do castigo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=26, color=CREAM)
            for s in ["3,5x a 4x o blind", "pagou 1? suba 3,5-4"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(7.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Comque(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("isole com", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.4)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["pares medios / suited / Ax", "posicao x range fraco"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(6.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("puna o limp toda vez", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo: continuation bet", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(11.5)
