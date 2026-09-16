"""EP08 v3 visual (versão aprofundada ~85s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, MiniTable, EquityBar, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("EV: A MEDIA MANDA", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("video 13: jogue a media", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("perder hoje, lucrar em mil", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        bar = EquityBar(50)
        bar.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.wait(15.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Pernas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("2 pernas: equidade + fold", font=MONO, font_size=27,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        bar = EquityBar(36)
        bar.move_to(DOWN * 0.2)
        cap = Text("36% das cartas + largam agora", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Semi(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.58)
        table.move_to(DOWN * 0.5)
        hero = [Card(r, s, height=0.75) for r, s in [("A", "h"), ("5", "h")]]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.45 + RIGHT * 0.85 * i + DOWN * 1.35)
        board = [Card(r, s, height=0.75) for r, s in [("K", "h"), ("9", "h"), ("2", "d")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 0.75 + RIGHT * 0.75 * i + UP * 0.4)
        cap = Text("semi-blefe: 2 jeitos de ganhar", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero + board, run_time=1.2)
        self.wait(17.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Erro(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("leia antes de blefar", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["rocha folda: blefe", "apaixonado: so valor"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(12.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("soma a seu favor? entre", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo: banca, o cinto", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.9)
