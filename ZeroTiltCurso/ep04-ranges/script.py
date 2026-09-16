"""EP04 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S6. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, deal_in

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
        table = MiniTable().scale(0.45)
        table.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.wait(7.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Cedo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("UTG: so premiums", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        prem = VGroup(*[Card("A", "s", height=0.85), Card("K", "s", height=0.85)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("8 olhando voce agir", font=MONO, font_size=22, color=CREAM)
        both = VGroup(prem, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(prem), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(13.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Tarde(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("botao: oceano", font=MONO, font_size=30,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        wide = VGroup(*[Card(r, s, height=0.8) for r, s in
                        [("7", "h"), ("6", "h"), ("A", "c"), ("5", "d")]]
                      ).arrange(RIGHT, buff=0.12)
        cap = Text("pares, ases, conectores, blefes", font=MONO, font_size=22, color=CREAM)
        both = VGroup(wide, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(wide), run_time=1.0)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(13.7)
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
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        backs = VGroup(*[CardBack(height=0.75) for _ in range(3)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 2.2)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        deal_in(self, list(backs), run_time=0.7)
        self.wait(12.7)
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
        self.wait(12.2)
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
        self.wait(12.5)
