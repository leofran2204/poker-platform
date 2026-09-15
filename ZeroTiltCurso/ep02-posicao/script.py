"""EP02 — Posição v2 (versão aprofundada ~85s). Sem LaTeX. Render: manim -ql script.py S1..S6. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("POSICAO E INFORMACAO", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("episodio 2: dinheiro e ordem", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("agir por ultimo e ver tudo", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(10.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Mesa(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("9 lugares, 1 botao", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["SB BB pagam no escuro", "UTG fala primeiro", "botao anda toda mao"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(13.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Cedo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("cedo: fechado e forte", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["UTG: so premiums", "8 olhando voce agir"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(9.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Tarde(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("tarde: largo e agressivo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["CO rouba, BTN e rei", "lucro nasce aqui"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(12.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Regra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a regra de bolso", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["fora de posicao: pote pequeno", "em posicao: molde o pote"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(15.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S6_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("mesmas cartas, outro resultado", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo: aumento ou descarte", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(9.0)
