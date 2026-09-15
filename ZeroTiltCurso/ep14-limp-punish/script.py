"""EP14 — Puna o limp (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("PUNA O LIMP", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 14: convite ou armadilha", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("SB so pagou: leia antes de punir", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Leia(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("primeiro: leia o vilao", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["fraco: isole 50%+", "equilibrado: 40-45% forte"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Armadilha(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("limp-reraise: alarme", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        ex = Text("voltou aumentando? quase sempre monstro", font=MONO, font_size=24, color=CREAM)
        ex.move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.wait(15.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Tamanho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o tamanho do castigo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["regular: 3,5x a 4x", "fraco: 4,5x, pote grande em posicao", "+1 por limper"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(18.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("complete barato, puna sempre", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: continuation bet", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(19.0)
