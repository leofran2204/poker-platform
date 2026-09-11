"""EP05 — Tamanho do aumento. Sem LaTeX. Render: manim -ql script.py S1..S5"""
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
            Text("O TAMANHO CERTO", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episódio 5: quanto aumentar", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        small = Text("pequeno: convida lixo", font=MONO, font_size=24, color=RED)
        big = Text("grande: só forte continua", font=MONO, font_size=24, color=RED)
        grp = VGroup(small, big).arrange(DOWN, buff=0.5).move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(small, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(big, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(6.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Opens(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("abertura padrão por posição", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(f"{pos:<4} {size}", font=MONO, font_size=26, color=CREAM)
            for pos, size in [("UTG", "2x"), ("HJ", "2x"), ("CO", "2,3x"),
                              ("BTN", "2,5x"), ("SB", "3x")]
        ]).arrange(DOWN, aligned_edge=LEFT, buff=0.3).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.5)
        self.wait(7.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_3bet(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("reabertura: posição manda no tamanho", font=MONO, font_size=25,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        ip = Text("com posição: 3x o aumento", font=MONO, font_size=26, color=GREEN)
        oop = Text("sem posição: 3,5x a 4x", font=MONO, font_size=26, color=CREAM)
        four = Text("4-bet: 2,2x a 2,5x", font=MONO, font_size=26, color=CREAM)
        grp = VGroup(ip, oop, four).arrange(DOWN, buff=0.4).move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ip, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(oop, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(four, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(7.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Limpers(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("contra limpers: +1 blind por curioso", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.2)
        ex = Text("2 limpers no NL10: 0,30 + 0,20 = R$ 0,50", font=MONO, font_size=28,
                  color=CREAM, weight=BOLD)
        ex.next_to(tag, DOWN, buff=0.7)
        frame = SurroundingRectangle(ex, color=GOLD, buff=0.2, stroke_width=4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.play(Create(frame), run_time=0.6)
        self.wait(6.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("tamanho conta história: padronize o seu", font=MONO, font_size=27, color=CREAM)
        nxt = Text("próximo episódio: roubos e 3-bets", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(6.5)
