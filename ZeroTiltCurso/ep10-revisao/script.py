"""EP10 — Revisão v2 (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("O METODO COMPLETO", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episodio 10: 5 regras", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("17 episodios em 5 regras", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(14.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Regras(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("regras 1, 2 e 3", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["objetivo: melhor 5 ou blefe", "posicao: ultimo decide", "agressao seletiva"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(16.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Conta(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("regras 4 e 5", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["a conta: odds + EV", "banca: 30-50 buy-ins"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(10.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Check(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("checklist de mesa", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=23, color=CREAM)
            for s in ["posicao? range? tamanho?", "preco? banca?"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        trig = Text("nao em tudo? volte uma casa", font=MONO, font_size=24, color=CREAM)
        trig.next_to(rows, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        self.wait(22.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("repeticao lucra, ponto final", font=MONO, font_size=26, color=CREAM)
        nxt = Text("agora e mesa: te vejo la", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(10.0)
