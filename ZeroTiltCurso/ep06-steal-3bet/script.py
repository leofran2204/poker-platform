"""EP06 v3 visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S4. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, ChipStack, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("ROUBOS E 3-BET", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 6: salario sem showdown", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("fim da fila: crime perfeito", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        table = MiniTable().scale(0.45)
        table.move_to(DOWN * 2.4)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.wait(11.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Roubo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.6)
        table.move_to(DOWN * 0.5)
        hero = [Card(r, s, height=0.75) for r, s in [("A", "h"), ("9", "d")]]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.45 + RIGHT * 0.85 * i + DOWN * 1.35)
        folds = VGroup(*[CardBack(height=0.7) for _ in range(3)]).arrange(RIGHT, buff=0.1)
        folds.move_to(table.get_center() + UP * 1.9)
        cap = Text("botao abre largo, resto larga", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero, run_time=0.9)
        self.play(FadeIn(folds), run_time=0.7)
        self.wait(16.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Camera(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("restile: aperte e puna", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        rows = VGroup(*[
            Text(s, font=MONO, font_size=24, color=CREAM)
            for s in ["frequente? range menor", "QQ+ AK adoram"]
        ]).arrange(DOWN, buff=0.4).move_to(UP * 0.1)
        prem = VGroup(*[Card("Q", "h", height=0.85), Card("Q", "d", height=0.85)]
                      ).arrange(RIGHT, buff=0.15)
        prem.move_to(DOWN * 2.1)
        self.play(FadeIn(tag), run_time=0.7)
        for r in rows:
            self.play(FadeIn(r, shift=RIGHT * 0.3), run_time=0.6)
        deal_in(self, list(prem), run_time=0.8)
        self.wait(19.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("defesa + squeeze", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        pot = ChipStack("3x / 4x + squeeze", n=5)
        pot.move_to(DOWN * 0.4)
        nxt = Text("proximo: pot odds", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        nxt.next_to(pot, DOWN, buff=0.5)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(pot), run_time=0.9)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(26.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)
