"""EP23 v1 (versão aprofundada ~75s). Sem LaTeX. Render: manim -ql script.py S1..S6. Áudio por cena (segN.mp3), waits casados."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, ChipStack, EquityBar, glow, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("SHORT DECK", font=MONO, font_size=46, color=GOLD, weight=BOLD),
            Text("video 20: 36 cartas", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("o jogo dos high rollers", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        deck = VGroup(*[CardBack(height=0.55) for _ in range(9)]).arrange(RIGHT, buff=0.1)
        deck.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(deck), run_time=1.0)
        self.wait(8.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Origem(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("Macau 2014, Triton 2018", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        trophy = ChipStack("2018", n=4)
        trophy.move_to(DOWN * 0.3)
        cap = Text("Ivey vence o primeiro na TV", font=MONO, font_size=22, color=CREAM)
        cap.next_to(trophy, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(trophy), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.1)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_36(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("6 a A, 630 maos", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        ranks = Text("6  7  8  9  T  J  Q  K  A", font=MONO, font_size=30, color=CREAM)
        ranks.move_to(DOWN * 0.3)
        cap = Text("AA 1 em 105", font=MONO, font_size=22, color=CREAM)
        cap.next_to(ranks, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ranks), run_time=0.7)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(9.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Ranking(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("trinca > straight", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        trips = [Card(r, s, height=0.7) for r, s in [("7", "c"), ("7", "d"), ("7", "h")]]
        seq = [Card(r, s, height=0.7) for r, s in [("6", "s"), ("8", "d"), ("9", "c")]]
        trow = VGroup(*trips).arrange(RIGHT, buff=0.1).move_to(LEFT * 2.6 + DOWN * 0.2)
        srow = VGroup(*seq).arrange(RIGHT, buff=0.1).move_to(RIGHT * 2.6 + DOWN * 0.2)
        cap = Text("14% x 7%: comum perde", font=MONO, font_size=22, color=CREAM)
        cap.move_to(DOWN * 2.5)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, trips + seq, run_time=1.2)
        self.play(Create(glow(trow)), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(10.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Draws(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("draws valem ouro", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        bar = EquityBar(50)
        bar.move_to(DOWN * 0.2)
        cap = Text("OESD ~50%, flush 5 outs", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(10.8)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S6_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("pares caem, draws sobem", font=MONO, font_size=26, color=CREAM)
        nxt = Text("proximo: Omaha Short Deck", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(8.2)
