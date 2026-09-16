"""EP14 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, ChipStack, EquityBar, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("PUNA O LIMP", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("video 7: convite ou armadilha", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("SB so pagou: leia antes de punir", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        backs = VGroup(*[CardBack(height=0.8) for _ in range(2)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(backs), run_time=0.8)
        self.wait(7.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Leia(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("primeiro: leia o vilao", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        bar = EquityBar(55)
        bar.move_to(DOWN * 0.2)
        cap = Text("fraco: isole 50%+ / equilibrado: 40-45%", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(15.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Armadilha(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("limp-reraise: alarme", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        trap = VGroup(*[Card("A", "s", height=0.9), Card("A", "d", height=0.9)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("voltou aumentando? quase sempre monstro", font=MONO, font_size=22, color=CREAM)
        both = VGroup(trap, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(trap), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(14.1)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Tamanho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o tamanho do castigo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        chips = ChipStack("3,5x - 4,5x", n=4)
        chips.move_to(DOWN * 0.5)
        cap = Text("regular 3,5-4x / fraco 4,5x / +1 por limper", font=MONO, font_size=21, color=CREAM)
        cap.next_to(chips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(17.1)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("complete barato, puna sempre", font=MONO, font_size=25, color=CREAM)
        pot = ChipStack("pote", n=5)
        pot.move_to(DOWN * 1.1)
        nxt = Text("proximo: continuation bet", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, pot, nxt).arrange(DOWN, buff=0.5).move_to(DOWN * 0.2)
        self.play(Write(line), run_time=1.2)
        self.play(FadeIn(pot), run_time=0.8)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(18.1)
