"""EP16 — Check-raise e float (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S4. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("CHECK-RAISE E FLOAT", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("episodio 16: forca ou plano", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("tomou raise em cima: e agora?", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Significado(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("raise no flop: valor ou semi", font=MONO, font_size=27,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["trinca / 2p / TPTK", "NFD / OESD com 2 jeitos de ganhar"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(21.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Float(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("floute: o alvo certo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=23, color=CREAM)
            for s in ["c-bet 70%+ e desiste no turn", "sempre em posicao", "bordo seco alto"]
        ]).arrange(DOWN, buff=0.35).move_to(DOWN * 0.1)
        trig = Text("checa? aposta e leva", font=MONO, font_size=24, color=CREAM)
        trig.next_to(rows, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.play(FadeIn(trig), run_time=0.7)
        self.wait(23.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("topo, draw ou fold: sem meio-termo", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: multiway", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(17.0)
