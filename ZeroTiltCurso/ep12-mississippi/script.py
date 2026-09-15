"""EP12 — No Mississippi nasce o blefe. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("NO MISSISSIPPI NASCE O BLEFE", font=MONO, font_size=34, color=GOLD, weight=BOLD),
            Text("episodio 12: 20 cartas e o blefe", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("vencia a mao — ou quem fazia desistir", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(4.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_20(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("1829: 20 cartas, 4 pessoas", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["do As ao 10", "blefe nasceu junto"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_52(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("Guerra Civil: 52 cartas", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["Draw: trocar cartas", "Stud: abertas e fechadas"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Holdem(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("1925 Robstown: 2 + 5", font=MONO, font_size=32,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.2)
        ex = Text("o jogo desta plataforma", font=MONO, font_size=26, color=CREAM)
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
        line = Text("1970: WSOP, 7 lendas", font=MONO, font_size=27, color=CREAM)
        nxt = Text("2003: US$ 86 viram 2,5 mi", font=MONO, font_size=27, color=CREAM)
        fin = Text("proximo: as lendas", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt, fin).arrange(DOWN, buff=0.5)
        self.play(Write(line), run_time=1.0)
        self.play(FadeIn(nxt, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(0.5)
        self.play(FadeIn(fin), run_time=0.8)
        self.wait(8.5)
