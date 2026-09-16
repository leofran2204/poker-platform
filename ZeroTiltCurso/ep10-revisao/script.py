"""EP10 v3 visual (versão aprofundada ~85s). Sem LaTeX. Render: manim -ql script.py S1..S5. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import CardBack, ChipStack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("O METODO COMPLETO", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("video 15: 5 regras", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("17 episodios em 5 regras", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        rules = VGroup(*[CardBack(height=0.6) for _ in range(5)]).arrange(RIGHT, buff=0.15)
        rules.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(rules), run_time=1.0)
        self.wait(13.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Regras(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("regras 1, 2 e 3", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["objetivo: melhor 5 ou blefe", "posicao: ultimo decide", "agressao seletiva"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        chips = ChipStack("3x", n=3)
        chips.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(chips), run_time=0.7)
        self.wait(15.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Conta(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("regras 4 e 5", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["a conta: odds + EV", "banca: 30-50 buy-ins"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        chips = ChipStack("30-50", n=3)
        chips.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(chips), run_time=0.7)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Check(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("checklist de mesa", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=22, color=CREAM)
            for s in ["posicao? range? tamanho?", "preco? banca?"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.3)
        trig = Text("nao em tudo? volte uma casa", font=MONO, font_size=22, color=CREAM)
        trig.next_to(rows, DOWN, buff=0.4)
        checks = VGroup(*[CardBack(height=0.5) for _ in range(5)]).arrange(RIGHT, buff=0.12)
        checks.move_to(DOWN * 2.5)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        deal_in(self, list(checks), run_time=1.0)
        self.wait(21.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("repeticao lucra, ponto final", font=MONO, font_size=26, color=CREAM)
        nxt = Text("agora e mesa: te vejo la", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(10.0)
