"""EP17 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S4. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, ChipStack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("MULTIWAY: 3+ NO FLOP", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 13: disciplina", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("blefe aqui e dinheiro fora", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        backs = VGroup(*[CardBack(height=0.8) for _ in range(3)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(backs), run_time=0.8)
        self.wait(15.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Valor(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("acertou? cobre sem do", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        pot = ChipStack("50-70%", n=5)
        pot.move_to(DOWN * 0.4)
        cap = Text("sempre tem quem pague pior", font=MONO, font_size=22, color=CREAM)
        cap.next_to(pot, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(pot), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Mining(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("sete mining: a conta", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        pair = VGroup(*[Card("5", "h", height=0.9), Card("5", "d", height=0.9)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("55 pagando 1 com 100 atras = 20x", font=MONO, font_size=22, color=CREAM)
        both = VGroup(pair, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(pair), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(26.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("menos blefe, mais valor", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo modulo: o turn", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(16.5)
