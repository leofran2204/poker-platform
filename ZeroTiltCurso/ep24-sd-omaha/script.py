"""EP24 v1 (versão aprofundada ~74s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, glow, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("OMAHA SHORT DECK", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("video 21: 4 cartas, 2+3", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("nuts ou nada", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        backs = VGroup(*[CardBack(height=0.6) for _ in range(4)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(backs), run_time=1.0)
        self.wait(13.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Regra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.55)
        table.move_to(DOWN * 0.4)
        hole = [Card(r, s, height=0.6) for r, s in
                [("A", "s"), ("K", "s"), ("Q", "d"), ("J", "d")]]
        for i, c in enumerate(hole):
            c.move_to(table.get_center() + LEFT * 1.05 + RIGHT * 0.7 * i + DOWN * 1.15)
        board = [Card(r, s, height=0.6) for r, s in
                 [("T", "s"), ("9", "s"), ("8", "d"), ("7", "c"), ("2", "h")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 1.4 + RIGHT * 0.7 * i + UP * 0.5)
        use = VGroup(hole[0], hole[1], board[0], board[1], board[2])
        cap = Text("sempre 2 + 3 exato", font=MONO, font_size=22, color=CREAM)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        deal_in(self, hole + board, run_time=1.4)
        self.play(Create(glow(use)), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(9.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Flush(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("flush: raro e mortal", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        nuts = [Card(r, s, height=0.75) for r, s in
                [("A", "s"), ("K", "s"), ("Q", "s"), ("J", "s"), ("9", "s")]]
        row = VGroup(*nuts).arrange(RIGHT, buff=0.1).move_to(DOWN * 0.2)
        cap = Text("9 por naipe: baixo e armadilha", font=MONO, font_size=22, color=CREAM)
        cap.next_to(row, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, nuts, run_time=1.0)
        self.play(Create(glow(row)), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(9.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Wrap(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("voltas abracam tudo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        hero = [Card(r, s, height=0.7) for r, s in
                [("J", "d"), ("T", "d"), ("9", "c"), ("8", "h")]]
        row = VGroup(*hero).arrange(RIGHT, buff=0.12).move_to(DOWN * 0.2)
        cap = Text("13+ outs cercam", font=MONO, font_size=22, color=CREAM)
        cap.next_to(row, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, hero, run_time=1.0)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.1)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("nuts ou nada, 2+3", font=MONO, font_size=26, color=CREAM)
        nxt = Text("proximo: Pineapple", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.9)
