"""EP19 v1 (versão aprofundada ~83s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
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
            Text("CONTROLE X POLAR", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("video 15: medio ou extremo", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("mao media nao joga pote gigante", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        bar = EquityBar(50)
        bar.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.wait(13.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Controle(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("pot control: barato", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("1/3", n=1)
        chips.move_to(DOWN * 0.3)
        cap = Text("check ou miuda: showdown barato", font=MONO, font_size=22, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(13.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Polar(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("polar: nuts ou nada", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("75%+", n=5)
        chips.move_to(DOWN * 0.3)
        cap = Text("dois extremos, sem meio", font=MONO, font_size=22, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(13.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Leitura(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("leia o tamanho", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        small = ChipStack("miuda", n=1)
        small.move_to(LEFT * 2.8 + DOWN * 0.3)
        big = ChipStack("enorme", n=5)
        big.move_to(RIGHT * 2.8 + DOWN * 0.3)
        cap = Text("pequena = medo, grande = polar", font=MONO, font_size=22, color=CREAM)
        cap.move_to(DOWN * 2.6)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(small), FadeIn(big), run_time=0.9)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(16.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("meio controla, extremo pressiona", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: matematica do turn", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.4)
