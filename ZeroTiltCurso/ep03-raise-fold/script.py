"""EP03 v3 visual (versão aprofundada ~80s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("AUMENTO OU DESCARTE", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 6: a primeira decisao", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("acertar aqui vence mesa iniciante", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        lixo = VGroup(*[Card("7", "c", height=0.8), Card("2", "d", height=0.8)]
                      ).arrange(RIGHT, buff=0.12)
        lixo.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(lixo), run_time=0.8)
        self.wait(7.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Lixo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("lixo fora de posicao: fold", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        pair = VGroup(*[Card("7", "c", height=0.9), Card("2", "d", height=0.9)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("7-2 no UTG: nem pense", font=MONO, font_size=22, color=CREAM)
        both = VGroup(pair, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(pair), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(14.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Premium(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("AA no botao: aumente", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.4)
        pair = VGroup(*[Card("A", "s", height=0.9), Card("A", "d", height=0.9)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("pote grande, poucos oponentes", font=MONO, font_size=22, color=CREAM)
        both = VGroup(pair, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.2)
        frame = SurroundingRectangle(both, color=GOLD, buff=0.2, stroke_width=4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(pair), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.play(Create(frame), run_time=0.6)
        self.wait(10.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Meio(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o meio: barato ou fold", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["par baixo sem stack? fold", "duvida fora de posicao? fold"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        backs = VGroup(*[CardBack(height=0.75) for _ in range(2)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        deal_in(self, list(backs), run_time=0.7)
        self.wait(17.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("forte aumenta, lixo descarta", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: ranges por posicao", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.0)
