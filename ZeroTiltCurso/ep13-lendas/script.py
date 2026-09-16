"""EP13 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, ChipStack, glow, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("LENDAS DO POKER", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episodio 13: Moss, Brunson, Ungar", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("eleito em 70, venceu em 71", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        trio = VGroup(*[CardBack(height=0.8) for _ in range(3)]).arrange(RIGHT, buff=0.12)
        trio.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(trio), run_time=0.8)
        self.wait(13.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_USA(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("Brunson e Ungar", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        pair = VGroup(*[Card("T", "d", height=0.9), Card("2", "h", height=0.9)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("10 braceletes + o 10-2 leva o nome", font=MONO, font_size=22, color=CREAM)
        both = VGroup(pair, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(pair), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(14.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Modernos(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a era moderna", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["Ivey / Hellmuth 17 / Negreanu", "Holz: aposentou antes dos 30"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        brace = ChipStack("17", n=4)
        brace.move_to(DOWN * 2.4)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(brace), run_time=0.7)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Brasil(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o Brasil no mapa", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=23, color=CREAM)
            for s in ["2004 CPH / BSOP maior fora de Vegas", "Gomes 2008: 2317, 770 mil", "Akkari 2011: 675 mil, 2o do pais"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        pot = ChipStack("770 mil", n=5)
        pot.move_to(DOWN * 2.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(pot), run_time=0.7)
        self.wait(18.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("Yuri 6 / Botteon vice 2020", font=MONO, font_size=25, color=CREAM)
        mid = Text("uma ideia por sessao", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo modulo: ranges", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, mid, nxt).arrange(DOWN, buff=0.5)
        prize = ChipStack("6", n=4)
        prize.move_to(RIGHT * 4.6 + DOWN * 0.4)
        self.play(Write(line), run_time=1.0)
        self.play(FadeIn(mid, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.play(FadeIn(prize), run_time=0.7)
        self.wait(14.5)
