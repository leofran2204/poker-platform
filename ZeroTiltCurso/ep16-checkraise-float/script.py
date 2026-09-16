"""EP16 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S4. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("CHECK-RAISE E FLOAT", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 12: forca ou plano", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("tomou raise em cima: e agora?", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        table = MiniTable().scale(0.45)
        table.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Significado(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.58)
        table.move_to(DOWN * 0.5)
        hero = [Card(r, s, height=0.75) for r, s in [("J", "h"), ("J", "d")]]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 2.0 + DOWN * 1.35 + RIGHT * 0.85 * i)
        board = [Card(r, s, height=0.75) for r, s in [("J", "c"), ("7", "h"), ("2", "h")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.75 + RIGHT * 0.75 * i + UP * 0.4)
        draw = [Card(r, s, height=0.75) for r, s in [("A", "h"), ("T", "h")]]
        for i, c in enumerate(draw):
            c.move_to(table.get_center() + RIGHT * 1.6 + DOWN * 1.35 + RIGHT * 0.85 * i)
        cap = Text("trinca x nut flush draw", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero + board + draw, run_time=1.2)
        self.wait(21.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Float(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.58)
        table.move_to(DOWN * 0.5)
        hero = [CardBack(height=0.75), CardBack(height=0.75)]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.45 + RIGHT * 0.85 * i + DOWN * 1.35)
        board = [Card(r, s, height=0.75) for r, s in [("K", "s"), ("5", "c"), ("2", "h")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.75 + RIGHT * 0.75 * i + UP * 0.4)
        cap = Text("floute: posicao + plano no turn", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero + board, run_time=1.2)
        self.wait(23.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("topo, draw ou fold: sem meio-termo", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: multiway", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(17.0)
