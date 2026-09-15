"""EP15 — C-bet: seco x molhado (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("C-BET: SECO X MOLHADO", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("episodio 15: a textura manda", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("olhe o board antes de apostar", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(3.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Seco(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("seco: K 7 2 / A 8 3", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["quase ninguem acerta", "frequente e barata: 25-33%", "range com todos os pares altos"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(24.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Molhado(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("molhado: 9 8 7 / J T 5 flush", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["acerta pares e draws", "menos vezes, 60-75% por valor"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(18.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_MeioTermo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("meio-termo: Q 9 2", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["manda a vantagem de range", "bate em voce: bet pequeno", "bate nele: pot control"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(18.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("seco barato sempre, molhado caro as vezes", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: check-raise e float", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(13.5)
