"""EP15 v3 visual (versão aprofundada ~93s). Sem LaTeX. Render: manim -ql script.py S1..S5. S3 com parágrafo do bordo molhado (+flâsh)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, MiniTable, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("C-BET: SECO X MOLHADO", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 11: a textura manda", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("olhe o board antes de apostar", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        table = MiniTable().scale(0.45)
        table.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.wait(2.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Seco(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.58)
        table.move_to(DOWN * 0.5)
        board = [Card(r, s, height=0.75) for r, s in [("K", "h"), ("7", "d"), ("2", "c")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.75 + RIGHT * 0.75 * i + UP * 0.4)
        cap = Text("seco: quase ninguem acerta — 25-33%", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, board, run_time=1.0)
        self.wait(23.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Molhado(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.58)
        table.move_to(DOWN * 0.5)
        board = [Card(r, s, height=0.75) for r, s in [("9", "h"), ("8", "h"), ("7", "c")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.75 + RIGHT * 0.75 * i + UP * 0.4)
        cap = Text("molhado: acerta tudo — 60-75% por valor", font=MONO, font_size=21, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, board, run_time=1.0)
        self.wait(24.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_MeioTermo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.58)
        table.move_to(DOWN * 0.5)
        board = [Card(r, s, height=0.75) for r, s in [("Q", "h"), ("9", "s"), ("2", "d")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.75 + RIGHT * 0.75 * i + UP * 0.4)
        cap = Text("meio-termo: manda o range", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, board, run_time=1.0)
        self.wait(16.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("seco barato sempre, molhado caro as vezes", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: check-raise e float", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.5)
