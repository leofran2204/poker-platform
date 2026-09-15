"""EP01 — Poker do zero v2 (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S6. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("POKER DO ZERO", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 1: como funciona", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("2 blinds, 2 suas, botao que anda", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(13.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Rodadas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("4 rodadas de aposta", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["pre-flop: fold / call / raise", "flop 3, turn 1, river 1"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(12.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Jogo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("melhor jogo de 5 leva", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["2 suas + 5 da mesa", "par ate royal flush"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(11.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Royal(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("A-K + Q-J-T de espadas", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.4)
        ex = Text("royal flush: o jogo imbatível", font=MONO, font_size=25, color=CREAM)
        ex.move_to(DOWN * 0.2)
        frame = SurroundingRectangle(ex, color=GOLD, buff=0.2, stroke_width=4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.play(Create(frame), run_time=0.6)
        self.wait(10.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Blefe(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("ninguem pagou? e seu", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        ex = Text("blefe: vencer sem o melhor jogo", font=MONO, font_size=24, color=CREAM)
        ex.move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S6_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("showdown: melhor 5 leva, empate divide", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: posicao, a maior vantagem", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(16.0)
