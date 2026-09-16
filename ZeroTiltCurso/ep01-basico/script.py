"""EP01 v3 visual (versão aprofundada ~85s). Sem LaTeX. Render: manim -ql script.py S1..S6. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, ChipStack, glow, deal_in

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("POKER DO ZERO", font=MONO, font_size=42, color=GOLD, weight=BOLD),
            Text("episodio 1: como funciona", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("2 blinds, 2 suas, botao que anda", font=MONO, font_size=24, color=CREAM)
        hook.move_to(DOWN * 0.5)
        hole = VGroup(*[CardBack(height=0.8) for _ in range(2)]).arrange(RIGHT, buff=0.12)
        hole.move_to(DOWN * 2.3)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(hole), run_time=0.8)
        self.wait(13.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Rodadas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("4 rodadas: flop 3, turn 1, river 1", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        backs = VGroup(*[CardBack(height=0.8) for _ in range(5)]).arrange(RIGHT, buff=0.12)
        backs.move_to(DOWN * 0.4)
        cap = Text("pre-flop: fold, call ou raise", font=MONO, font_size=22, color=CREAM)
        cap.next_to(backs, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(backs), run_time=1.0)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Jogo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.6)
        table.move_to(DOWN * 0.5)
        hero = [Card("A", "s", height=0.75), Card("K", "d", height=0.75)]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.45 + RIGHT * 0.85 * i + DOWN * 1.35)
        cap = Text("2 suas + 5 da mesa", font=MONO, font_size=22, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, hero, run_time=0.9)
        self.wait(11.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Royal(Scene):
    def construct(self):
        self.camera.background_color = FELT
        royal = VGroup(*[Card(r, "s", height=0.9) for r in ["A", "K", "Q", "J", "T"]]
                       ).arrange(RIGHT, buff=0.12)
        cap = Text("royal flush: o jogo imbativel", font=MONO, font_size=22, color=CREAM)
        both = VGroup(royal, cap).arrange(DOWN, buff=0.4).move_to(DOWN * 0.2)
        tag = Text("A-K + Q-J-T de espadas", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.7)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(royal), run_time=1.0)
        ring = glow(royal)
        self.play(Create(ring), FadeIn(cap), run_time=0.8)
        self.wait(10.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Blefe(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("ninguem pagou? e seu", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        pot = ChipStack("pote seu", n=4)
        pot.move_to(DOWN * 0.3)
        cap = Text("blefe: vencer sem mostrar", font=MONO, font_size=22, color=CREAM)
        cap.next_to(pot, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(pot), run_time=0.8)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S6_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("showdown: melhor 5 leva, empate divide", font=MONO, font_size=24, color=CREAM)
        nxt = Text("proximo: posicao, a maior vantagem", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(15.5)
