"""EP25 v1 (versão aprofundada ~74s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, ChipStack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("PINEAPPLE", font=MONO, font_size=46, color=GOLD, weight=BOLD),
            Text("video 22: 3 cartas, zero descarte", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("o dobro da confusao", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        trio = VGroup(*[Card(r, s, height=0.8) for r, s in
                        [("A", "h"), ("K", "h"), ("Q", "h")]]).arrange(RIGHT, buff=0.12)
        trio.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(trio), run_time=1.0)
        self.wait(8.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Familia(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("descarta quando?", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=22, color=CREAM)
            for s in ["normal: antes do flop", "crazy: depois do flop",
                      "lazy: segura ate o fim", "nossa: nunca descarta"]
        ]).arrange(DOWN, buff=0.35).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.5)
        self.wait(13.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Origem(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("home games anos 80", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["California e Colombia", "nome da fruta"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        cap = Text("2 cartas pouco, 4 demais", font=MONO, font_size=22, color=CREAM)
        cap.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Forca(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("3 cartas = acao", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        trio = VGroup(*[Card(r, s, height=0.85) for r, s in
                        [("A", "s"), ("A", "d"), ("K", "s")]]).arrange(RIGHT, buff=0.12)
        trio.move_to(DOWN * 0.2)
        cap = Text("valor p/ baixo, volume p/ cima", font=MONO, font_size=22, color=CREAM)
        cap.next_to(trio, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(trio), run_time=1.0)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(12.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("2+3 no showdown", font=MONO, font_size=26, color=CREAM)
        nxt = Text("proximo: EV e fold equity", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.4)
