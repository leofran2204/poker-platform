"""EP17 — Multiway e set mining (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S4. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("MULTIWAY: 3+ NO FLOP", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("episodio 17: disciplina", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("blefe aqui e dinheiro fora", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Valor(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("acertou? cobre sem do", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["trinca / 2p: 50-70%", "sempre tem quem pague pior"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(17.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Mining(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("sete mining: a conta", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["par baixo / 20x o call", "55 pagando 1 com 100 atras"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        trig = Text("fora de posicao cobra mais caro", font=MONO, font_size=23, color=CREAM)
        trig.next_to(rows, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        self.wait(26.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("menos blefe, mais valor", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo modulo: o turn", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(16.5)
