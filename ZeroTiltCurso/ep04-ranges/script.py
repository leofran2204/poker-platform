"""EP04 — Ranges v2 (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S6. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("RANGES: O MAPA", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 4: cada assento, um mapa", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("o mapa muda com o assento", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Cedo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("UTG: so premiums", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["pares altos / AK / AQ suited", "8 olhando voce agir", "meio: abre um pouco"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(14.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Tarde(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("CO rouba, botao e oceano", font=MONO, font_size=27,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["qualquer par, qualquer As", "blefes pagam a entrada"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(14.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Contra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("contra quem?", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["apertado: abra mais e roube", "pagador: valor e cobranca"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(13.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Erro(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o erro classico", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.move_to(UP * 1.4)
        ex = Text("UTG largo: doar fichas", font=MONO, font_size=25, color=CREAM)
        ex.move_to(DOWN * 0.2)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(ex), run_time=0.8)
        self.wait(13.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S6_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("logica, nao lista", font=MONO, font_size=27, color=CREAM)
        nxt = Text("proximo: tamanho do aumento", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.0)
