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
