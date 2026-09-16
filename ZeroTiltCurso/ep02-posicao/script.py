"""EP02 v3 visual (versão aprofundada ~85s). Sem LaTeX. Render: manim -ql script.py S1..S6. Mesmos waits da v2 (áudio reaproveitado)."""
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
            Text("POSICAO E INFORMACAO", font=MONO, font_size=38, color=GOLD, weight=BOLD),
            Text("video 5: dinheiro e ordem", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("agir por ultimo e ver tudo", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        table = MiniTable().scale(0.45)
        table.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.wait(9.9)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Mesa(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.62)
        table.move_to(DOWN * 0.5)
        hero = [CardBack(height=0.75), CardBack(height=0.75)]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.45 + RIGHT * 0.85 * i + DOWN * 1.35)
        cap = Text("9 lugares, botao anda", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero, run_time=0.9)
        self.wait(12.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Cedo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("cedo: fechado e forte", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        prem = VGroup(*[Card("A", "s", height=0.85), Card("A", "d", height=0.85)]
                      ).arrange(RIGHT, buff=0.15)
        cap = Text("UTG: so premiums", font=MONO, font_size=22, color=CREAM)
        both = VGroup(prem, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(prem), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(8.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Tarde(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("tarde: largo e agressivo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        wide = VGroup(*[Card("7", "h", height=0.85), Card("6", "h", height=0.85),
                        Card("A", "c", height=0.85), Card("5", "d", height=0.85)]
                      ).arrange(RIGHT, buff=0.12)
        cap = Text("botao joga 3x mais maos", font=MONO, font_size=22, color=CREAM)
        both = VGroup(wide, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(wide), run_time=1.0)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.9)
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
        self.wait(14.9)
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
