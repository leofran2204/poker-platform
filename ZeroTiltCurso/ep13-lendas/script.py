"""EP13 — Lendas do poker (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S5. Áudio por cena (segN.mp3), waits casados."""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("LENDAS DO POKER", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episodio 13: Moss, Brunson, Ungar", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("eleito em 70, venceu em 71", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(16.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_USA(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("Brunson e Ungar", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["Brunson: 10 + Super System", "o 10-2 leva o nome dele", "Ungar: 3 Mains, genio tragico"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(15.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Modernos(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a era moderna", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["Aivi / Helmuth 17 / Negrianu", "Holz: aposentou antes dos 30"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(10.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Brasil(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("o Brasil no mapa", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=23, color=CREAM)
            for s in ["2004 CPH / BSOP maior fora de Vegas", "Gomes 2008: 2317, 770 mil", "Akkari 2011: 675 mil, 2o do pais"]
        ]).arrange(DOWN, buff=0.4).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        self.wait(20.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("Yuri 6 / Botteon vice 2020", font=MONO, font_size=25, color=CREAM)
        mid = Text("uma ideia por sessao", font=MONO, font_size=25, color=CREAM)
        nxt = Text("proximo modulo: ranges", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, mid, nxt).arrange(DOWN, buff=0.5)
        self.play(Write(line), run_time=1.0)
        self.play(FadeIn(mid, shift=RIGHT * 0.3), run_time=0.7)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(14.5)
