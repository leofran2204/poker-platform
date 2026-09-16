"""EP09 v3 visual (versão aprofundada ~80s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import ChipStack, EquityBar

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("BANCA: O CINTO", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("video 14: sobreviva a variancia", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("80% tambem perde 20 vezes", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        bar = EquityBar(80)
        bar.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.wait(11.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Regra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("30 a 50 buy-ins", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["NL10? banca 300-500", "dinheiro que pode perder"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        chips = ChipStack("30-50", n=4)
        chips.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(chips), run_time=0.7)
        self.wait(15.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Queda(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("caiu 1/4? desce", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["reconstrua sem ego", "tilt com aluguel e doacao"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        chips = ChipStack("3/4", n=3)
        chips.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(chips), run_time=0.7)
        self.wait(13.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Separa(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("cada formato, sua banca", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=23, color=CREAM)
            for s in ["cash nao paga torneio", "misturar quebra nos dois"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.2)
        cash = ChipStack("cash", n=3)
        cash.move_to(LEFT * 2.8 + DOWN * 1.9)
        mtt = ChipStack("MTT", n=3)
        mtt.move_to(RIGHT * 2.8 + DOWN * 1.9)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(cash), FadeIn(mtt), run_time=0.9)
        self.wait(12.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("viva ate o longo prazo", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo: revisao final", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.5)
