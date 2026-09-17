"""EP22 v1 (versão aprofundada ~76s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import EquityBar, ChipStack

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("OVERBET X BLOCKING", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 19: gigante e ana", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("duas armas, dois problemas", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        chips = VGroup(*[ChipStack("125%", n=6), ChipStack("25%", n=1)]).arrange(RIGHT, buff=1.0)
        chips.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.9)
        self.wait(11.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Over(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("overbet: nuts ou blefe", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("125%", n=6)
        chips.move_to(DOWN * 0.3)
        cap = Text("cobra caro de pagador", font=MONO, font_size=22, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(15.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Mdf(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("pote 100, aposta 125", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        bar = EquityBar(44)
        bar.move_to(DOWN * 0.2)
        cap = Text("defenda 44%, resto e seu", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.8)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Block(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("blocking 20-30% OOP", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("25%", n=1)
        chips.move_to(DOWN * 0.3)
        cap = Text("seguro contra a bomba", font=MONO, font_size=22, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(9.3)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("nuts cobre, OOP taxa", font=MONO, font_size=26, color=CREAM)
        nxt = Text("proximo: Short Deck", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.8)
