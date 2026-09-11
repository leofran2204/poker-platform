"""EP10 — Revisão do método. Sem LaTeX. Render: manim -ql script.py S1..S4"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("O MÉTODO COMPLETO", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episódio 10: revisão final", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 1.8)
        line = Text("10 episódios viram 5 regras", font=MONO, font_size=26, color=CREAM)
        line.move_to(DOWN * 0.8)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(line), run_time=0.8)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Regras(Scene):
    def construct(self):
        self.camera.background_color = FELT
        rules = VGroup(*[
            Text(f"{i}. {t}", font=MONO, font_size=24, color=CREAM)
            for i, t in [
                (1, "objetivo: melhor jogo de 5"),
                (2, "posição: falar por último"),
                (3, "agressão seletiva: raise, não limp"),
                (4, "a conta: odds e EV sempre"),
                (5, "banca: 30 a 50 buy-ins"),
            ]
        ]).arrange(DOWN, aligned_edge=LEFT, buff=0.35).move_to(DOWN * 0.2)
        for r in rules:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(12.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Checklist(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("antes de cada mão, pergunte:", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.9)
        qs = VGroup(*[
            Text(t, font=MONO, font_size=24, color=CREAM)
            for t in ["posição?  range?  tamanho?  odds?  banca?"]
        ]).move_to(DOWN * 0.3)
        ok = Text("tudo sim? jogue sem medo do hoje", font=MONO, font_size=25,
                  color="#6BCB77", weight=BOLD)
        ok.to_edge(DOWN, buff=1.0)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(qs), run_time=0.8)
        self.play(FadeIn(ok), run_time=0.8)
        self.wait(10.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("consistência vence talento", font=MONO, font_size=32,
                    color=CREAM, weight=BOLD)
        nxt = Text("te vejo no Zero Tilt Poker", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.4)
        self.wait(0.6)
        self.play(FadeIn(nxt, scale=1.1), run_time=1.0)
        self.wait(9.0)
