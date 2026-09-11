"""EP08 — EV e fold equity. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
GREEN = "#6BCB77"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("EV + FOLD EQUITY", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episódio 8: as duas portas", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 1.8)
        line = Text("poker é repetição, não a mão de hoje", font=MONO, font_size=25, color=CREAM)
        line.move_to(DOWN * 0.8)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(line), run_time=0.8)
        self.wait(8.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Moeda(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a moeda viciada", font=MONO, font_size=26, color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        cara = Text("cara: +R$ 2", font=MONO, font_size=28, color=GREEN)
        coroa = Text("coroa: -R$ 1", font=MONO, font_size=28, color=RED)
        grp = VGroup(cara, coroa).arrange(DOWN, buff=0.4).move_to(UP * 0.2)
        bad = Text("dia ruim: 7 coroas em 10 → prejuízo", font=MONO, font_size=22, color=CREAM)
        good = Text("10 mil jogadas → lucro certo (+EV)", font=MONO, font_size=24, color=GREEN, weight=BOLD)
        tail = VGroup(bad, good).arrange(DOWN, buff=0.4).move_to(DOWN * 1.6)
        self.play(FadeIn(tag), run_time=0.6)
        self.play(FadeIn(cara), FadeIn(coroa), run_time=0.8)
        self.play(FadeIn(bad), run_time=0.7)
        self.play(FadeIn(good), run_time=0.7)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Portas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("duas portas para vencer", font=MONO, font_size=26, color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        d1 = Text("porta 1: melhor mão no final", font=MONO, font_size=25, color=CREAM)
        d2 = Text("porta 2: todos desistem antes", font=MONO, font_size=25, color=CREAM)
        call = Text("call/limp: abre mão da porta 2", font=MONO, font_size=24, color=RED)
        grp = VGroup(d1, d2, call).arrange(DOWN, buff=0.4).move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.6)
        self.play(FadeIn(d1), FadeIn(d2), run_time=0.8)
        self.play(FadeIn(call), run_time=0.8)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Agressao(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("raise = cartas + fold equity", font=MONO, font_size=30,
                    color=GREEN, weight=BOLD)
        sub = Text("a chance real do rival largar por medo", font=MONO, font_size=24, color=CREAM)
        grp = VGroup(line, sub).arrange(DOWN, buff=0.5)
        self.play(FadeIn(line, scale=1.1), run_time=0.9)
        self.play(FadeIn(sub), run_time=0.7)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("jogue o +EV e bata na porta do fold", font=MONO, font_size=27, color=CREAM)
        nxt = Text("próximo episódio: gestão de banca", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(6.5)
