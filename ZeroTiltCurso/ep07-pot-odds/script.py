"""EP07 v3 visual (versão aprofundada ~85s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import CardBack, ChipStack, EquityBar, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("POT ODDS", font=MONO, font_size=44, color=GOLD, weight=BOLD),
            Text("episodio 7: quanto custa ver", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("a conta mais importante", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        chips = VGroup(*[ChipStack("25", n=1), ChipStack("100", n=4)]).arrange(RIGHT, buff=1.2)
        chips.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(chips), run_time=0.8)
        self.wait(8.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Conta(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("25 em 75: 4 para 1", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        bar = EquityBar(20)
        bar.move_to(DOWN * 0.2)
        cap = Text("precisa de 20% para empatar", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(14.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Outs(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("flush draw: 9 outs", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        outs = VGroup(*[CardBack(height=0.62) for _ in range(9)]).arrange(RIGHT, buff=0.08)
        cap = Text("36% flop-river, 18% turn-river", font=MONO, font_size=22, color=CREAM)
        both = VGroup(outs, cap).arrange(DOWN, buff=0.4).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(outs), run_time=1.0)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(15.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Compara(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("36 contra 25: call", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        bar = EquityBar(36)
        bar.move_to(DOWN * 0.2)
        cap = Text("verde paga, vermelho folda", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(12.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("implied decide os duvidosos", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: EV e fold equity", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(18.7)
