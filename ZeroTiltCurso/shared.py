"""Biblioteca visual compartilhada dos vídeos Zero Tilt (Manim, sem LaTeX).

Componentes reutilizáveis com a identidade do curso:
feltro #0A2E1A, dourado #C9A227, creme #F4F0E6, monospace.
Uso: from shared import Card, MiniTable, ChipStack, EquityBar, glow, deal_in
(Testado com Manim 0.21 em vid-env.)
"""
from manim import *

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
RED = "#B33A3A"
MONO = "DejaVu Sans Mono"

SUIT_SYMBOL = {"s": "♠", "h": "♥", "d": "♦", "c": "♣"}
RED_SUITS = ("h", "d")


class Card(VGroup):
    """Carta de baralho: retângulo arredondado + rank/naipe. Ex: Card('A', 's')."""

    def __init__(self, rank: str, suit: str, height: float = 1.1, **kwargs):
        super().__init__(**kwargs)
        color = RED if suit in RED_SUITS else "#1A1A1A"
        back = RoundedRectangle(
            corner_radius=0.12, height=height, width=height * 0.72,
            fill_color="#F4F0E6", fill_opacity=1,
            stroke_color=GOLD, stroke_width=3,
        )
        top = Text(f"{rank}{SUIT_SYMBOL[suit]}", font=MONO, font_size=26, color=color)
        top.move_to(back.get_center() + UP * height * 0.18)
        bottom = Text(f"{rank}{SUIT_SYMBOL[suit]}", font=MONO, font_size=26, color=color)
        bottom.move_to(back.get_center() + DOWN * height * 0.18)
        self.add(back, top, bottom)


class CardBack(VGroup):
    """Verso fechado (padrão dourado sobre feltro escuro)."""

    def __init__(self, height: float = 1.1, **kwargs):
        super().__init__(**kwargs)
        self.add(
            RoundedRectangle(
                corner_radius=0.12, height=height, width=height * 0.72,
                fill_color="#1A2A5A", fill_opacity=1,
                stroke_color=GOLD, stroke_width=3,
            )
        )


class MiniTable(VGroup):
    """Mesa oval de feltro com pote central. seats: [(nome, pos)] — use .seat(i)."""

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        felt = Ellipse(
            width=9.5, height=4.6,
            fill_color=FELT, fill_opacity=1,
            stroke_color=GOLD, stroke_width=6,
        )
        self.pot = Text("POTE", font=MONO, font_size=22, color=GOLD)
        self.pot.move_to(felt.get_center() + UP * 1.1)
        self.add(felt, self.pot)

    def seat_pos(self, i: int, n: int = 2):
        """Posição do assento i (0 = herói embaixo)."""
        spots = [
            DOWN * 3.1,
            UP * 3.1,
            LEFT * 4.6,
            RIGHT * 4.6,
            LEFT * 3.4 + DOWN * 2.2,
            RIGHT * 3.4 + DOWN * 2.2,
        ]
        return spots[i % len(spots)]


class ChipStack(VGroup):
    """Pilha de fichas douradas com valor. Ex: ChipStack('R$ 200')."""

    def __init__(self, label: str = "", n: int = 3, **kwargs):
        super().__init__(**kwargs)
        chips = VGroup(*[
            Circle(
                radius=0.32, fill_color=GOLD, fill_opacity=1,
                stroke_color="#3A2A08", stroke_width=4,
            ).shift(UP * 0.14 * k)
            for k in range(max(1, n))
        ])
        self.add(chips)
        if label:
            tag = Text(label, font=MONO, font_size=22, color=CREAM)
            tag.next_to(chips, DOWN, buff=0.25)
            self.add(tag)


class EquityBar(VGroup):
    """Barra de equidade herói x vilão (0-100). Ex: EquityBar(82)."""

    def __init__(self, hero_pct: float, width: float = 6.0, **kwargs):
        super().__init__(**kwargs)
        hero_pct = max(0.0, min(100.0, hero_pct))
        back = Rectangle(
            width=width, height=0.55,
            fill_color="#3A3A3A", fill_opacity=1, stroke_width=0,
        )
        fill = Rectangle(
            width=width * hero_pct / 100.0, height=0.55,
            fill_color=GOLD, fill_opacity=1, stroke_width=0,
        )
        fill.align_to(back, LEFT)
        label = Text(f"você {hero_pct:.0f}%", font=MONO, font_size=24, color=CREAM)
        label.next_to(back, UP, buff=0.25)
        self.add(back, fill, label)


def glow(mobj: VMobject, color: str = GOLD) -> SurroundingRectangle:
    """Destaque dourado ao redor do jogo vencedor."""
    return SurroundingRectangle(mobj, color=color, buff=0.15, stroke_width=5)


def deal_in(scene: Scene, mobjects, run_time: float = 0.5, shift=DOWN * 0.4):
    """Distribui cartas/objetos com o ritual do crupiê."""
    scene.play(
        LaggedStart(*[FadeIn(m, shift=shift) for m in mobjects],
                     lag_ratio=0.35),
        run_time=run_time,
    )


DARK = "#1A1A1A"
BRONZE = "#3A2A08"
LEAF_GREEN = "#3D9970"


def CoinMark(color: str = GOLD, r: float = 0.16) -> VGroup:
    """Moeda com furo quadrado (naipe de moedas: China, gandjifa)."""
    outer = Circle(radius=r, fill_color=color, fill_opacity=1, stroke_width=0)
    hole = Square(side_length=r * 0.85, stroke_color=BRONZE,
                  stroke_width=4, fill_opacity=0)
    return VGroup(outer, hole)


def StringMark(color: str = GOLD) -> VGroup:
    """Cordão de moedas com nós (naipe de cordões: China)."""
    bar = RoundedRectangle(corner_radius=0.08, width=0.52, height=0.15,
                           fill_color=color, fill_opacity=1, stroke_width=0)
    knots = VGroup(*[Dot(radius=0.045, color=BRONZE).shift(RIGHT * (i - 1) * 0.17)
                     for i in range(3)])
    return VGroup(bar, knots)


def CrownMark(color: str = GOLD) -> VGroup:
    """Coroa (taj: gandjifa; xá: aznás)."""
    pts = [LEFT * 0.22 + DOWN * 0.14, LEFT * 0.22 + UP * 0.08,
           LEFT * 0.07 + DOWN * 0.02, UP * 0.16,
           RIGHT * 0.07 + DOWN * 0.02, RIGHT * 0.22 + UP * 0.08,
           RIGHT * 0.22 + DOWN * 0.14]
    return VGroup(Polygon(*pts, fill_color=color, fill_opacity=1, stroke_width=0))


def SabreMark(color: str = GOLD) -> VGroup:
    """Sabre curvo (shamsher: gandjifa; cimitarra mameluca)."""
    blade = AnnularSector(inner_radius=0.2, outer_radius=0.29,
                          angle=PI * 0.55, fill_color=color, fill_opacity=1,
                          stroke_width=0)
    blade.rotate(PI * 0.72)
    grip = Line(ORIGIN, DOWN * 0.14, stroke_color=color, stroke_width=6)
    grip.next_to(blade, DOWN, buff=0.0)
    pommel = Dot(radius=0.045, color=color).next_to(grip, DOWN, buff=0.0)
    return VGroup(blade, grip, pommel)


def ServantMark(color: str = DARK) -> VGroup:
    """Servo (ghulam: gandjifa)."""
    head = Circle(radius=0.08, stroke_color=color, stroke_width=5,
                  fill_opacity=0).shift(UP * 0.2)
    body = Line(UP * 0.12, DOWN * 0.14, stroke_color=color, stroke_width=5)
    arms = Line(LEFT * 0.15 + UP * 0.02, RIGHT * 0.15 + UP * 0.02,
                stroke_color=color, stroke_width=5)
    leg_l = Line(DOWN * 0.14, LEFT * 0.11 + DOWN * 0.3,
                 stroke_color=color, stroke_width=5)
    leg_r = Line(DOWN * 0.14, RIGHT * 0.11 + DOWN * 0.3,
                 stroke_color=color, stroke_width=5)
    return VGroup(head, body, arms, leg_l, leg_r)


def HarpMark(color: str = GOLD) -> VGroup:
    """Harpa (chang: gandjifa)."""
    frame = Triangle(stroke_color=color, stroke_width=5,
                     fill_opacity=0).scale(0.24)
    strings = VGroup(*[Line(UP * 0.11, DOWN * 0.11, stroke_color=color,
                             stroke_width=2).shift(RIGHT * (i - 1) * 0.08)
                       for i in range(3)])
    return VGroup(frame, strings)


def DocumentMark() -> VGroup:
    """Documento (barat: gandjifa) — retângulo de contorno verde."""
    return VGroup(Rectangle(width=0.42, height=0.3, stroke_color=LEAF_GREEN,
                            stroke_width=5, fill_color=CREAM, fill_opacity=1))


def BolsterMark(color: str = GOLD) -> VGroup:
    """Tecido/almofada (qimash: gandjifa)."""
    body = RoundedRectangle(corner_radius=0.1, width=0.46, height=0.2,
                            fill_color=color, fill_opacity=1, stroke_width=0)
    ties = VGroup(*[Line(UP * 0.1, DOWN * 0.1, stroke_color=BRONZE,
                          stroke_width=3).shift(RIGHT * s)
                    for s in (-0.12, 0.12)])
    return VGroup(body, ties)


def CupMark(color: str = GOLD) -> VGroup:
    """Taça (tuman: mamelucos)."""
    bowl = Polygon(LEFT * 0.2 + UP * 0.16, RIGHT * 0.2 + UP * 0.16,
                   RIGHT * 0.09 + DOWN * 0.04, LEFT * 0.09 + DOWN * 0.04,
                   fill_color=color, fill_opacity=1, stroke_width=0)
    stem = Line(DOWN * 0.04, DOWN * 0.18, stroke_color=color, stroke_width=6)
    base = Line(LEFT * 0.14 + DOWN * 0.18, RIGHT * 0.14 + DOWN * 0.18,
                stroke_color=color, stroke_width=6)
    return VGroup(bowl, stem, base)


def PoloMark(color: str = GOLD) -> VGroup:
    """Tacos de polo cruzados (jawkān: mamelucos)."""
    def stick():
        shaft = Line(DOWN * 0.22, UP * 0.16, stroke_color=color, stroke_width=6)
        hook = Line(UP * 0.16, UP * 0.16 + RIGHT * 0.14,
                    stroke_color=color, stroke_width=6)
        return VGroup(shaft, hook)
    return VGroup(stick().rotate(PI / 7), stick().rotate(-PI / 7))


def SunMark(color: str = GOLD) -> VGroup:
    """Sol com raios (ás-leão: aznás)."""
    disc = Circle(radius=0.12, fill_color=color, fill_opacity=1, stroke_width=0)
    rays = VGroup(*[Line(UP * 0.16, UP * 0.24, stroke_color=color,
                          stroke_width=4).rotate(a * PI / 4, about_point=ORIGIN)
                    for a in range(8)])
    return VGroup(disc, rays)


def FlowerMark(color: str) -> VGroup:
    """Flor de 5 pétalas (dama: cartas florais do aznás)."""
    petals = VGroup(*[Dot(radius=0.07, color=color).shift(UP * 0.13)
                       .rotate(a * TAU / 5, about_point=ORIGIN)
                       for a in range(5)])
    heart = Dot(radius=0.055, color=color)
    return VGroup(petals, heart)


def NoteMark(color: str) -> VGroup:
    """Colcheia vetorial (bailarino/músico: aznás)."""
    head = Ellipse(width=0.17, height=0.13, fill_color=color, fill_opacity=1,
                   stroke_width=0).shift(LEFT * 0.07 + DOWN * 0.13)
    stem = Line(RIGHT * 0.015 + DOWN * 0.12, RIGHT * 0.015 + UP * 0.22,
                stroke_color=color, stroke_width=5)
    flag = Polygon(RIGHT * 0.015 + UP * 0.22, RIGHT * 0.17 + UP * 0.16,
                   RIGHT * 0.015 + UP * 0.1,
                   fill_color=color, fill_opacity=1, stroke_width=0)
    return VGroup(head, stem, flag)


def SwordsMark(color: str = GOLD) -> VGroup:
    """Espadas cruzadas (soldado: aznás)."""
    def sword():
        blade = Rectangle(width=0.05, height=0.4, fill_color=color,
                          fill_opacity=1, stroke_width=0)
        guard = Rectangle(width=0.16, height=0.045, fill_color=color,
                          fill_opacity=1, stroke_width=0).shift(DOWN * 0.2)
        return VGroup(blade, guard)
    return VGroup(sword().rotate(PI / 5.5), sword().rotate(-PI / 5.5))


class SuitTile(VGroup):
    """Mini-carta creme com pictograma + etiqueta. Ex: SuitTile(CoinMark(), 'moedas')."""

    def __init__(self, mark: VMobject, label: str = "",
                 width: float = 0.68, height: float = 0.95, **kwargs):
        super().__init__(**kwargs)
        back = RoundedRectangle(
            corner_radius=0.08, width=width, height=height,
            fill_color=CREAM, fill_opacity=1,
            stroke_color=GOLD, stroke_width=2,
        )
        self.add(back)
        if label:
            mark.move_to(back.get_center() + UP * 0.1)
            tag = Text(label, font=MONO, font_size=13, color=DARK)
            tag.move_to(back.get_center() + DOWN * (height / 2 - 0.13))
            self.add(mark, tag)
        else:
            mark.move_to(back.get_center())
            self.add(mark)


ASNAS_GROUNDS = {
    "as": "#202020",
    "xa": "#1E6B3A",
    "dama": "#D9A821",
    "soldado": "#C97B1A",
    "danca": "#B33A3A",
}


class AsNasTile(VGroup):
    """Mini-carta de aznás: fundo na cor da série + emblema. Ex: AsNasTile('xa', CrownMark(CREAM))."""

    def __init__(self, series: str, emblem: VMobject,
                 width: float = 0.66, height: float = 0.88, **kwargs):
        super().__init__(**kwargs)
        back = RoundedRectangle(
            corner_radius=0.08, width=width, height=height,
            fill_color=ASNAS_GROUNDS[series], fill_opacity=1,
            stroke_color=GOLD, stroke_width=2,
        )
        emblem.move_to(back.get_center())
        self.add(back, emblem)
