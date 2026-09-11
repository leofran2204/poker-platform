"""EP02 — Posição. Sem LaTeX. Render: manim -ql script.py S1..S5"""
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


def seat_dot(label, pos, dealer=False, hero=False):
    c = Circle(radius=0.32, fill_opacity=1,
               fill_color=GOLD if dealer else INK,
               stroke_color=GOLD, stroke_width=3)
    t = Text(label, font=MONO, font_size=22, color=CREAM if not dealer else INK,
             weight=BOLD)
    if hero:
        c.set_stroke(CREAM, width=5)
    return VGroup(c, t).move_to(pos)


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("POSIÇÃO", font=MONO, font_size=44, color=GOLD, weight=BOLD),
            Text("episódio 2: falar por último", font=MONO, font_size=26, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        table = Ellipse(width=5.5, height=3.0, color=GOLD, stroke_width=4)
        table.move_to(DOWN * 0.8)
        dots = VGroup(
            seat_dot("D", table.point_at_angle(-90 * DEGREES), dealer=True),
            seat_dot("SB", table.point_at_angle(-30 * DEGREES)),
            seat_dot("BB", table.point_at_angle(30 * DEGREES)),
            seat_dot("BTN", table.point_at_angle(210 * DEGREES), hero=True),
        )
        arrow = CurvedArrow(table.point_at_angle(250 * DEGREES),
                            table.point_at_angle(290 * DEGREES),
                            color=CREAM)
        tag = Text("ordem de fala →", font=MONO, font_size=20, color=CREAM)
        tag.next_to(table, DOWN, buff=0.4)
        self.play(Write(head), run_time=1.2)
        self.play(Create(table), run_time=1.0)
        self.play(FadeIn(dots, shift=UP * 0.2), run_time=0.8)
        self.play(Create(arrow), FadeIn(tag), run_time=0.8)
        self.wait(5.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_OOP(Scene):
    def construct(self):
        self.camera.background_color = FELT
        seat = Text("VOCÊ no BIG BLIND", font=MONO, font_size=26, color=RED, weight=BOLD)
        seat.to_edge(UP, buff=0.8)
        hero = VGroup(card("K", "♠"), card("Q", "♠")).arrange(RIGHT, buff=0.15)
        hero.move_to(UP * 0.6)
        flop = VGroup(card("K", "♥"), card("8", "♦"), card("4", "♣")).arrange(RIGHT, buff=0.15)
        flop.next_to(hero, DOWN, buff=0.6)
        q = Text("???", font=MONO, font_size=40, color=GOLD, weight=BOLD)
        q.next_to(flop, DOWN, buff=0.5)
        warn = Text("você fala PRIMEIRO — no escuro", font=MONO, font_size=24, color=CREAM)
        warn.to_edge(DOWN, buff=0.9)
        self.play(FadeIn(seat), run_time=0.6)
        self.play(FadeIn(hero, shift=DOWN * 0.3), run_time=0.8)
        self.wait(0.6)
        self.play(FadeIn(flop, shift=UP * 0.3), run_time=0.8)
        self.wait(0.6)
        self.play(FadeIn(q, scale=1.4), run_time=0.8)
        self.play(FadeIn(warn), run_time=0.8)
        self.wait(6.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_IP(Scene):
    def construct(self):
        self.camera.background_color = FELT
        seat = Text("VOCÊ no BOTÃO", font=MONO, font_size=26, color=GOLD, weight=BOLD)
        seat.to_edge(UP, buff=0.8)
        hero = VGroup(card("K", "♠"), card("Q", "♠")).arrange(RIGHT, buff=0.15)
        hero.move_to(UP * 1.2)
        flop = VGroup(card("K", "♥"), card("8", "♦"), card("4", "♣")).arrange(RIGHT, buff=0.15)
        flop.next_to(hero, DOWN, buff=0.6)
        check = Text("vilão: CHECK ✓", font=MONO, font_size=24, color="#6BCB77", weight=BOLD)
        check.next_to(flop, RIGHT, buff=0.5)
        info = Text("informação primeiro, dinheiro depois", font=MONO, font_size=24, color=CREAM)
        info.to_edge(DOWN, buff=0.6)
        opt1 = Text("→ apostar e extrair", font=MONO, font_size=22, color=CREAM)
        opt2 = Text("→ mesa e ver o turn grátis", font=MONO, font_size=22, color=CREAM)
        opts = VGroup(opt1, opt2).arrange(DOWN, aligned_edge=LEFT, buff=0.2)
        opts.next_to(info, UP, buff=0.4)
        self.play(FadeIn(seat), run_time=0.6)
        self.play(FadeIn(hero, shift=DOWN * 0.3), FadeIn(flop, shift=UP * 0.3), run_time=1.0)
        self.wait(0.6)
        self.play(FadeIn(check, shift=LEFT * 0.3), run_time=0.8)
        self.wait(0.6)
        self.play(FadeIn(info), run_time=0.8)
        self.play(FadeIn(opts), run_time=0.8)
        self.wait(6.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Regra(Scene):
    def construct(self):
        self.camera.background_color = FELT
        late = Text("EM POSIÇÃO: mais mãos, ataque", font=MONO, font_size=30,
                    color="#6BCB77", weight=BOLD)
        early = Text("FORA DE POSIÇÃO: só as fortes", font=MONO, font_size=30,
                     color=RED, weight=BOLD)
        grp = VGroup(late, early).arrange(DOWN, buff=0.6)
        self.play(FadeIn(late, shift=RIGHT * 0.3), run_time=1.0)
        self.wait(0.6)
        self.play(FadeIn(early, shift=RIGHT * 0.3), run_time=1.0)
        self.wait(5.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        line = Text("falar por último = decidir com informação",
                    font=MONO, font_size=28, color=CREAM)
        nxt = Text("próximo episódio: aumento ou descarte", font=MONO, font_size=26,
                   color=GOLD, weight=BOLD)
        grp = VGroup(line, nxt).arrange(DOWN, buff=0.6)
        self.play(Write(line), run_time=1.5)
        self.wait(0.5)
        self.play(FadeIn(nxt), run_time=1.0)
        self.wait(5.0)
