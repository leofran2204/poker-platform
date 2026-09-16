"""EP05 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import ChipStack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("O TAMANHO CERTO", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("video 7: caro ou barato, nunca errado", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("pequeno convida, grande isola", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        chips = VGroup(*[ChipStack("", n=1), ChipStack("", n=5)]).arrange(RIGHT, buff=1.0)
        chips.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.wait(12.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Padrao(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("padrao por posicao", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=25, color=CREAM)
            for s in ["UTG/HJ 2x, CO 2,3x", "BTN 2,5x, SB 3x"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        chips = ChipStack("2,5x", n=3)
        chips.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(chips), run_time=0.7)
        self.wait(15.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Reabre(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("reabertura: posicao manda", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        ip = ChipStack("3x IP", n=3)
        ip.move_to(LEFT * 3.0 + DOWN * 0.3)
        oop = ChipStack("4x OOP", n=4)
        oop.move_to(RIGHT * 3.0 + DOWN * 0.3)
        cap = Text("4-bet: 2,2-2,5x", font=MONO, font_size=22, color=CREAM)
        cap.move_to(DOWN * 2.8)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ip), FadeIn(oop), run_time=0.9)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(20.3)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Limpers(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("+1 por curioso", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("4,5x", n=4)
        chips.move_to(DOWN * 0.3)
        cap = Text("2 limpers + BTN 2,5x", font=MONO, font_size=22, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(14.1)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("padronize o seu, leia o dos outros", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: roubos e 3-bets", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(15.4)
