"""EP12 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, ChipStack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("DO MISSISSIPPI AO BOOM", font=MONO, font_size=36, color=GOLD, weight=BOLD),
            Text("video 2: 20 cartas ao mundo", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("o blefe nasceu junto com o jogo", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        fan = VGroup(*[CardBack(height=0.8) for _ in range(5)]).arrange(RIGHT, buff=0.12)
        fan.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(fan), run_time=0.8)
        self.wait(12.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Robstown(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.6)
        table.move_to(DOWN * 0.5)
        hero = [Card("A", "s", height=0.75), Card("A", "d", height=0.75)]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.45 + RIGHT * 0.85 * i + DOWN * 1.35)
        board = [CardBack(height=0.75) for _ in range(5)]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 1.5 + RIGHT * 0.75 * i + UP * 0.55)
        cap = Text("Robstown: 2 fechadas + 5 comunitarias", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero + board, run_time=1.2)
        self.wait(16.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Vegas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("1963: Vegas descobre", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("California Club", n=2)
        chips.move_to(LEFT * 3.4 + DOWN * 0.3)
        chips2 = ChipStack("Nugget poeirento", n=4)
        chips2.move_to(RIGHT * 3.2 + DOWN * 0.3)
        cap = Text("1967: o as passa a valer alto", font=MONO, font_size=22, color=CREAM)
        cap.move_to(DOWN * 2.9)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), FadeIn(chips2), run_time=0.9)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(15.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Dunes(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.6)
        table.move_to(DOWN * 0.5)
        hero = [Card("A", "s", height=0.75), Card("K", "s", height=0.75)]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 2.2 + DOWN * 1.35 + RIGHT * 0.85 * i)
        vil = [CardBack(height=0.75), CardBack(height=0.75)]
        for i, c in enumerate(vil):
            c.move_to(table.get_center() + RIGHT * 1.4 + DOWN * 1.35 + RIGHT * 0.85 * i)
        cap = Text("1969 Dunes: quem sabia levou", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero + vil, run_time=1.0)
        self.wait(18.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("1970 WSOP / 2003 Moneymaker", font=MONO, font_size=25, color=CREAM)
        pot = ChipStack("2,5 mi", n=5)
        pot.move_to(DOWN * 1.2)
        nxt = Text("proximo: as lendas", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, pot, nxt).arrange(DOWN, buff=0.5).move_to(DOWN * 0.2)
        self.play(Write(line), run_time=1.0)
        self.play(FadeIn(pot), run_time=0.8)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.7)
