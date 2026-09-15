"""EP15 — C-bet: seco x molhado. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("C-BET: SECO X MOLHADO", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("episodio 15: a textura manda", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("olhe o board antes de apostar", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(4.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Seco(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("seco: K 7 2 / A 8 3", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["poucos draws", "frequente e barata: 25-33%"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Molhado(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("molhado: 9 8 7 / J T 5 flush", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["acerta pares e draws", "menos vezes, 60-75% por valor"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Regra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a regra de ouro", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.4)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["seco barato sempre", "molhado caro as vezes"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(6.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("tamanho conta historia", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo: check-raise e float", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(11.5)
