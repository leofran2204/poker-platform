"""EP12 — Mississippi ao boom (versão aprofundada ~95s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("DO MISSISSIPPI AO BOOM", font=MONO, font_size=36, color=GOLD, weight=BOLD),
            Text("episodio 12: 20 cartas ao mundo", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("o blefe nasceu junto com o jogo", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(13.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Robstown(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("Robstown: 2 + 5", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["Blondie Forbes escreve as regras", "2 fechadas + 5 comunitarias", "Texas espalha por decadas"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(17.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Vegas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("1963: Vegas descobre", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["Corky no California Club", "salao poeirento do Nugget", "1967: as passa a valer alto"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Dunes(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("1969: a entrada do Dunes", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["Addington / Brunson / Amarillo", "novatos contra quem sabia", "dinheiro facil"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(19.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("1970 WSOP / 2003 Moneymaker", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: as lendas", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.5)
