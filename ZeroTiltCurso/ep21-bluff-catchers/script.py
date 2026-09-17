"""EP21 v1 (versão aprofundada ~91s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, EquityBar, glow, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("CACE O BLEFE", font=MONO, font_size=44, color=GOLD, weight=BOLD),
            Text("video 18: hero paga", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("pagar ganhando so de blefe", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        pair = VGroup(*[Card("7", "c", height=0.8), Card("7", "d", height=0.8)]
                      ).arrange(RIGHT, buff=0.12)
        pair.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(pair), run_time=1.0)
        self.wait(14.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Historia(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("historia logica?", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["flop: aposta pequena", "turn: check", "river: bomba?"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        trig = Text("linha quebrada cheira a blefe", font=MONO, font_size=22, color=CREAM)
        trig.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        self.wait(13.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Blockers(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("As tira o nut flush", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        board = [Card(r, s, height=0.7) for r, s in [("K", "s"), ("Q", "s"), ("J", "s")]]
        hero = [Card(r, s, height=0.7) for r, s in [("A", "s"), ("5", "h")]]
        cards = VGroup(*(board + hero)).arrange(RIGHT, buff=0.12).move_to(DOWN * 0.2)
        cap = Text("6 vira 3; nut flush zera", font=MONO, font_size=22, color=CREAM)
        cap.next_to(cards, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, board + hero, run_time=1.2)
        self.play(Create(glow(hero[0])), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(18.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Preco(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("conta do call", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        conta = Text("9 blefes x 6 valores", font=MONO, font_size=26, color=CREAM)
        conta.move_to(UP * 0.3)
        bar = EquityBar(60)
        bar.move_to(DOWN * 1.2)
        cap = Text("60% de blefe: pague", font=MONO, font_size=22, color=CREAM)
        cap.next_to(bar, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(conta), run_time=0.6)
        self.play(FadeIn(bar), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(13.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("bloqueie valor, nunca blefes", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: overbet x blocking", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(15.8)
