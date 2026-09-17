"""EP20 v1 (versão aprofundada ~78s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
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
            Text("RIVER: VALOR OU BLEFE", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 17: 3 botoes", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("cada botao, uma pergunta", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        backs = VGroup(*[CardBack(height=0.6) for _ in range(3)]).arrange(RIGHT, buff=0.15)
        backs.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(backs), run_time=1.0)
        self.wait(11.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Value(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.6)
        table.move_to(DOWN * 0.4)
        hero = [Card(r, s, height=0.7) for r, s in [("K", "s"), ("Q", "s")]]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.4 + RIGHT * 0.8 * i + DOWN * 1.1)
        board = [Card(r, s, height=0.7) for r, s in [("J", "h"), ("T", "s"), ("2", "c")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.7 + RIGHT * 0.7 * i + UP * 0.5)
        cap = Text("value: quem paga pior", font=MONO, font_size=24, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        deal_in(self, hero + board, run_time=1.2)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(14.1)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Blefe(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.6)
        table.move_to(DOWN * 0.4)
        hero = [Card(r, s, height=0.7) for r, s in [("9", "h"), ("8", "h")]]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.4 + RIGHT * 0.8 * i + DOWN * 1.1)
        board = [Card(r, s, height=0.7) for r, s in [("K", "s"), ("Q", "s"), ("2", "d")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.7 + RIGHT * 0.7 * i + UP * 0.5)
        cap = Text("blefe: venda o fold", font=MONO, font_size=24, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        deal_in(self, hero + board, run_time=1.2)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(15.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Check(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("resto: check gratis", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        backs = VGroup(*[CardBack(height=0.75) for _ in range(2)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 0.3)
        cap = Text("sem alvo: mostre gratis", font=MONO, font_size=22, color=CREAM)
        cap.next_to(backs, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(backs), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("3 botoes, 3 perguntas", font=MONO, font_size=26, color=CREAM)
        nxt = Text("proximo: cace o blefe", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(12.9)
