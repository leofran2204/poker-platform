"""EP01 — Poker do zero. Sem LaTeX: só Text + formas. Render: manim -ql script.py"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
INK = "#1A1A1A"
BACK = "#1A2A5A"
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


def chip_stack(n=5):
    chips = VGroup(*[
        Circle(radius=0.22, fill_opacity=1, fill_color=GOLD,
               stroke_color=CREAM, stroke_width=2).shift(DOWN * 0.09 * i)
        for i in range(n)
    ])
    return chips


def title_card(text, sub=""):
    t = Text(text, font=MONO, font_size=44, color=GOLD, weight=BOLD)
    if not sub:
        return t
    s = Text(sub, font=MONO, font_size=26, color=CREAM)
    return VGroup(t, s).arrange(DOWN, buff=0.3)


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = title_card("POKER DO ZERO", "episódio 1: seu objetivo")
        head.move_to(UP * 1.5)
        deck = VGroup(*[card(face_down=True, w=0.7, h=1.0) for _ in range(3)])
        deck.arrange(RIGHT, buff=-0.55).next_to(head, DOWN, buff=0.8)
        label = Text("52 cartas", font=MONO, font_size=26, color=CREAM)
        label.next_to(deck, DOWN, buff=0.4)
        self.play(Write(head), run_time=1.5)
        self.wait(0.5)
        self.play(FadeIn(deck, shift=UP * 0.3), run_time=1.0)
        self.play(FadeIn(label), run_time=0.6)
        self.wait(3.5)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_SuasCartas(Scene):
    def construct(self):
        self.camera.background_color = FELT
        you = Text("VOCÊ", font=MONO, font_size=30, color=GOLD, weight=BOLD)
        you.to_edge(DOWN, buff=1.2)
        c1 = card(face_down=True).move_to(UP * 2.5 + LEFT * 2)
        c2 = card(face_down=True).move_to(UP * 2.5 + RIGHT * 2)
        tag = Text("2 cartas só suas", font=MONO, font_size=26, color=CREAM)
        tag.to_edge(UP, buff=0.8)
        self.play(FadeIn(you), run_time=0.6)
        self.play(FadeIn(c1, shift=DOWN), FadeIn(c2, shift=DOWN), run_time=1.0)
        self.play(c1.animate.move_to(LEFT * 0.55 + DOWN * 0.4),
                  c2.animate.move_to(RIGHT * 0.55 + DOWN * 0.4), run_time=1.2)
        self.wait(0.4)
        open1 = card("A", "♠").move_to(c1.get_center())
        open2 = card("K", "♠").move_to(c2.get_center())
        self.play(FadeOut(c1), FadeOut(c2),
                  FadeIn(open1), FadeIn(open2), FadeIn(tag), run_time=1.0)
        self.wait(5.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Mesa(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("5 cartas da mesa (para todos)", font=MONO, font_size=26, color=CREAM)
        tag.to_edge(UP, buff=0.8)
        flop = [card("Q", "♠"), card("J", "♠"), card("T", "♥")]
        turn = card("3", "♦")
        river = card("2", "♣")
        board = VGroup(*flop, turn, river).arrange(RIGHT, buff=0.15)
        self.play(FadeIn(tag), run_time=0.6)
        for i, c in enumerate(flop):
            self.play(FadeIn(c, shift=UP * 0.3), run_time=0.6)
            self.wait(0.3)
        flop_tag = Text("flop", font=MONO, font_size=20, color=GOLD)
        flop_tag.next_to(board, DOWN, buff=0.4)
        self.play(FadeIn(flop_tag), run_time=0.4)
        self.wait(1.5)
        self.play(FadeIn(turn, shift=UP * 0.3), run_time=0.6)
        self.wait(1.5)
        self.play(FadeIn(river, shift=UP * 0.3), run_time=0.6)
        self.wait(3.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_MelhorJogo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        hero = VGroup(card("A", "♠"), card("K", "♠")).arrange(RIGHT, buff=0.15)
        hero.move_to(UP * 2.2)
        hero_tag = Text("suas 2", font=MONO, font_size=22, color=CREAM)
        hero_tag.next_to(hero, LEFT, buff=0.5)
        board = VGroup(card("Q", "♠"), card("J", "♠"), card("T", "♥"),
                       card("3", "♦"), card("2", "♣")).arrange(RIGHT, buff=0.15)
        board.next_to(hero, DOWN, buff=0.7)
        self.play(FadeIn(hero), FadeIn(hero_tag), FadeIn(board), run_time=1.2)
        self.wait(2.0)
        royal = Text("SEQUÊNCIA REAL — o jogo mais forte", font=MONO,
                     font_size=26, color=GOLD, weight=BOLD)
        royal.to_edge(DOWN, buff=0.9)
        win_five = VGroup(hero[0], hero[1], board[0], board[1], board[2])
        frame = SurroundingRectangle(win_five, color=GOLD, buff=0.12, stroke_width=6)
        self.play(Create(frame),
                  FadeIn(royal), run_time=1.5)
        self.wait(1.0)
        for m in (board[3], board[4]):
            self.play(m.animate.set_opacity(0.25), run_time=0.5)
        self.wait(0.5)
        pot = chip_stack(6).move_to(DOWN * 2.6 + RIGHT * 3.2)
        arrow = Arrow(royal.get_top(), pot.get_left(), color=GOLD, buff=0.2)
        take = Text("leva o pote", font=MONO, font_size=24, color=CREAM)
        take.next_to(pot, DOWN, buff=0.3)
        self.play(FadeIn(pot), Create(arrow), FadeIn(take), run_time=1.2)
        self.wait(5.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("2 suas + 5 da mesa = melhor jogo de 5 leva tudo",
                    font=MONO, font_size=28, color=CREAM)
        nxt = Text("próximo episódio: a POSIÇÃO", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.5)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=1.0)
        self.wait(5.5)
