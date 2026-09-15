"""EP06 — Roubos e 3-bet v2 (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S4. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("ROUBOS E 3-BET", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 6: salario sem showdown", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("fim da fila: crime perfeito", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(11.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Roubo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("botao: abra largo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["qualquer As, reis, conectores", "blinds largam a fracao", "pagou? posicao o resto"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(17.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Camera(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("restile: aperte e puna", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["frequente? range menor", "QQ+ AK adoram", "short? so com mao"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        trig = Text("3-bet valor neles", font=MONO, font_size=24, color=CREAM)
        trig.next_to(rows, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        self.wait(19.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("defesa + squeeze", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=23, color=CREAM)
            for s in ["3x com / 4x sem posicao", "squeeze: raiser + calls = pote morto"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        trig = Text("cobre pedagio ou tome de volta", font=MONO, font_size=24, color=CREAM)
        trig.next_to(rows, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        self.wait(27.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)
