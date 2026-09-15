"""EP08 — EV e fold equity v2 (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("EV: A MEDIA MANDA", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episodio 8: jogue a media", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("perder hoje, lucrar em mil", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Pernas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("2 pernas: equidade + fold", font=MONO, font_size=27,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["equidade: chance ate o fim", "fold equity: largam agora"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(12.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Semi(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("semi-blefe: 2 jeitos de ganhar", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["foldam? leva agora", "pagam? 36% de melhorar"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(18.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Erro(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("leia antes de blefar", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["rocha folda: blefe", "apaixonado: so valor"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(13.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("soma a seu favor? entre", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo: banca, o cinto", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.5)
