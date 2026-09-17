"""EP18 v1 (versão aprofundada ~77s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, EquityBar, ChipStack, glow, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("DOUBLE BARREL", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("video 14: o segundo barril", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("pressao com motivo", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        chips = ChipStack("2.o", n=2)
        chips.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.wait(12.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Scare(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("scare: As, Rei e completa", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        board = [Card(r, s, height=0.8) for r, s in
                 [("K", "d"), ("7", "c"), ("2", "h"), ("A", "s")]]
        row = VGroup(*board).arrange(RIGHT, buff=0.12).move_to(DOWN * 0.2)
        cap = Text("turn que ajuda o agressor", font=MONO, font_size=22, color=CREAM)
        cap.next_to(row, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, board, run_time=1.0)
        self.play(Create(glow(board[3])), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(13.8)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Equity(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("equidade nova no turn", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        hero = [Card(r, s, height=0.7) for r, s in [("A", "h"), ("5", "h")]]
        hrow = VGroup(*hero).arrange(RIGHT, buff=0.12).move_to(UP * 0.1)
        bar = EquityBar(35)
        bar.move_to(DOWN * 1.3)
        cap = Text("ganhou draw: dois jeitos", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, hero, run_time=1.0)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(12.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Sizing(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("60 a 75%, sempre igual", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("2/3", n=3)
        chips.move_to(DOWN * 0.3)
        cap = Text("valor e blefe: mesma aposta", font=MONO, font_size=22, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("scare + sizing: pressao", font=MONO, font_size=26, color=CREAM)
        nxt = Text("proximo: controle x polar", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.0)
