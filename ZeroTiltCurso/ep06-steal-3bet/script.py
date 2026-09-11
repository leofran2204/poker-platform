"""EP06 — Roubos e 3-bets. Sem LaTeX. Render: manim -ql script.py S1..S5"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
BACK = "#1A2A5A"
GREEN = "#6BCB77"
MONO = "DejaVu Sans Mono"
RED_SUITS = ("♥", "♦")


def card(rank="", suit="", face_down=False, w=0.9, h=1.3):
    rect = RoundedRectangle(
        corner_radius=0.08, width=w, height=h,
        fill_opacity=1, fill_color=CREAM, stroke_color=GOLD, stroke_width=3,
    )
    if face_down:
        rect.set_fill(BACK, opacity=1)
        return VGroup(rect)
    color = RED if suit in RED_SUITS else INK
    r = Text(rank, font=MONO, font_size=28, color=color, weight=BOLD)
    s = Text(suit, font=MONO, font_size=30, color=color)
    return VGroup(rect, VGroup(r, s).arrange(DOWN, buff=0.05))


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("ROUBOS E 3-BETS", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("episódio 6: o contra-ataque", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.0)
        line = Text("blind parado é dinheiro morto", font=MONO, font_size=26, color=CREAM)
        line.move_to(DOWN * 0.5)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(line), run_time=0.8)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_Steal(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("BTN aumenta, blinds largam ~70%", font=MONO, font_size=25,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        btn = Text("BTN: raise", font=MONO, font_size=26, color=CREAM)
        btn.move_to(UP * 0.8)
        f1 = Text("SB: fold", font=MONO, font_size=24, color=CREAM)
        f2 = Text("BB: fold", font=MONO, font_size=24, color=CREAM)
        folds = VGroup(f1, f2).arrange(DOWN, buff=0.3).move_to(DOWN * 0.5)
        take = Text("recolhe sem ver o flop", font=MONO, font_size=24, color=GREEN, weight=BOLD)
        take.to_edge(DOWN, buff=0.9)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(btn), run_time=0.7)
        self.play(FadeIn(f1, shift=LEFT * 0.3), run_time=0.6)
        self.play(FadeIn(f2, shift=LEFT * 0.3), run_time=0.6)
        self.play(FadeIn(take), run_time=0.7)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Defesa(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("defenda seu blind com re-aumento", font=MONO, font_size=25,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        hero = VGroup(card("J", "♠"), card("J", "♦")).arrange(RIGHT, buff=0.15)
        hero.move_to(UP * 0.5)
        raid = Text("roubo 0,25 → 3-bet 0,90", font=MONO, font_size=28,
                    color=CREAM, weight=BOLD)
        raid.next_to(hero, DOWN, buff=0.6)
        warn = Text("fora de posição não se paga: se ataca", font=MONO, font_size=23, color=CREAM)
        warn.to_edge(DOWN, buff=0.9)
        self.play(FadeIn(tag), run_time=0.7)
        self.play(FadeIn(hero, shift=DOWN * 0.3), run_time=0.8)
        self.play(FadeIn(raid), run_time=0.8)
        self.play(FadeIn(warn), run_time=0.7)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Efeito(Scene):
    def construct(self):
        self.camera.background_color = FELT
        a = Text("desistiu → pote seu na hora", font=MONO, font_size=27, color=GREEN)
        b = Text("pagou → pote grande com a melhor", font=MONO, font_size=27, color=CREAM)
        grp = VGroup(a, b).arrange(DOWN, buff=0.5)
        self.play(FadeIn(a, shift=RIGHT * 0.3), run_time=0.8)
        self.play(FadeIn(b, shift=RIGHT * 0.3), run_time=0.8)
        self.wait(7.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("ataque os blinds alheios, re-aumente nos seus",
                    font=MONO, font_size=26, color=CREAM)
        nxt = Text("próximo episódio: pot odds", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.2)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=0.8)
        self.wait(7.0)
