"""EP11 v3 piloto visual (versão aprofundada ~90s). Sem LaTeX. Render: manim -ql script.py S1..S7. Mesmos waits da v2 (áudio reaproveitado)."""
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.getcwd(), "..")))
from manim import *
from shared import Card, CardBack, MiniTable, ChipStack, glow, deal_in
from shared import (SuitTile, AsNasTile, CoinMark, StringMark, CrownMark,
                    SabreMark, ServantMark, HarpMark, DocumentMark,
                    BolsterMark, CupMark, PoloMark, SunMark, FlowerMark,
                    NoteMark, SwordsMark, DARK, CREAM)

FELT = "#0A2E1A"
GOLD = "#C9A227"
CREAM = "#F4F0E6"
MONO = "DejaVu Sans Mono"


def money_strip(symbol, label):
    """Tira de papel chinesa: símbolo do naipe + etiqueta."""
    base = Rectangle(width=2.6, height=0.5, fill_color=CREAM, fill_opacity=1,
                     stroke_color="#C9A227", stroke_width=2)
    symbol.move_to(base.get_center() + LEFT * 0.85)
    tag = Text(label, font=MONO, font_size=20, color=DARK)
    tag.move_to(base.get_center() + LEFT * 0.85 + RIGHT * 1.05)
    return VGroup(base, symbol, tag)


class S1_Titulo(Scene):
    def construct(self):
        self.camera.background_color = FELT
        head = VGroup(
            Text("MIL ANOS DE BARALHO", font=MONO, font_size=40, color=GOLD, weight=BOLD),
            Text("video 1: de onde veio o baralho", font=MONO, font_size=24, color=CREAM),
        ).arrange(DOWN, buff=0.3).move_to(UP * 2.2)
        hook = Text("mil anos antes do seu all-in", font=MONO, font_size=26, color=CREAM)
        hook.move_to(DOWN * 0.5)
        deck = VGroup(*[CardBack(height=0.9) for _ in range(5)]).arrange(RIGHT, buff=0.15)
        deck.move_to(DOWN * 2.2)
        self.play(Write(head), run_time=1.2)
        self.play(FadeIn(hook, shift=RIGHT * 0.3), run_time=0.7)
        deal_in(self, list(deck), run_time=0.8)
        self.wait(5.6)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S2_China(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("China, sec. IX — dinastia Tang", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        strips = VGroup(*[
            money_strip(CoinMark(), "moedas"),
            money_strip(StringMark(), "cordoes"),
            money_strip(VGroup(*[CoinMark(r=0.1) for _ in range(3)]
                               ).arrange(RIGHT, buff=0.08), "miriades"),
        ]).arrange(DOWN, buff=0.25).move_to(DOWN * 0.6)
        cap = Text("tiras de papel » Rota da Seda", font=MONO, font_size=22, color=CREAM)
        cap.next_to(strips, DOWN, buff=0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(strips), run_time=0.9)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(9.7)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S3_Persia(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("Persia: dois jogos", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        g_row = VGroup(*[
            SuitTile(CrownMark(), width=0.5, height=0.62),
            SuitTile(CoinMark("#C0C0C0"), width=0.5, height=0.62),
            SuitTile(SabreMark(), width=0.5, height=0.62),
            SuitTile(ServantMark().scale(0.85), width=0.5, height=0.62),
            SuitTile(CoinMark(), width=0.5, height=0.62),
            SuitTile(HarpMark(), width=0.5, height=0.62),
            SuitTile(DocumentMark(), width=0.5, height=0.62),
            SuitTile(BolsterMark(), width=0.5, height=0.62),
        ]).arrange(RIGHT, buff=0.08)
        g_cap = Text("gandjifa: 96 em 8 naipes", font=MONO, font_size=22, color=CREAM)
        g_block = VGroup(g_row, g_cap).arrange(DOWN, buff=0.2)
        a_row = VGroup(*[
            AsNasTile("as", SunMark(CREAM), width=0.62, height=0.82),
            AsNasTile("xa", CrownMark(CREAM), width=0.62, height=0.82),
            AsNasTile("dama", FlowerMark(DARK), width=0.62, height=0.82),
            AsNasTile("soldado", SwordsMark(CREAM), width=0.62, height=0.82),
            AsNasTile("danca", NoteMark(CREAM), width=0.62, height=0.82),
        ]).arrange(RIGHT, buff=0.08)
        a_cap = Text("aznas: 25 em 5 naipes", font=MONO, font_size=22, color=CREAM)
        a_block = VGroup(a_row, a_cap).arrange(DOWN, buff=0.2)
        both = VGroup(g_block, a_block).arrange(DOWN, buff=0.5).move_to(DOWN * 0.3)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(g_row), run_time=0.8)
        self.play(FadeIn(g_cap), run_time=0.5)
        deal_in(self, list(a_row), run_time=0.8)
        self.play(FadeIn(a_cap), run_time=0.5)
        self.wait(10.4)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S4_Mamelucos(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("mamelucos para a Europa", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        fan = VGroup(*[
            SuitTile(CoinMark(), "moedas"),
            SuitTile(PoloMark(), "tacos"),
            SuitTile(CupMark(), "tacas"),
            SuitTile(SabreMark(), "espadas"),
        ]).arrange(RIGHT, buff=0.15)
        cap = Text("Egito: 52 em 4 naipes -> 1370 na Europa", font=MONO, font_size=22, color=CREAM)
        both = VGroup(fan, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(fan), run_time=0.9)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(12.2)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S5_Franca(Scene):
    def construct(self):
        self.camera.background_color = FELT
        tag = Text("a Franca barateou tudo", font=MONO, font_size=28,
                   color=GOLD, weight=BOLD)
        tag.to_edge(UP, buff=0.8)
        suits = VGroup(*[Card("K", s, height=0.95) for s in ["s", "h", "d", "c"]]
                       ).arrange(RIGHT, buff=0.15)
        cap = Text("Davi / Carlos Magno / Cesar / Alexandre", font=MONO, font_size=20, color=CREAM)
        both = VGroup(suits, cap).arrange(DOWN, buff=0.35).move_to(DOWN * 0.4)
        self.play(FadeIn(tag), run_time=0.7)
        deal_in(self, list(suits), run_time=0.9)
        self.play(FadeIn(cap), run_time=0.6)
        self.wait(11.0)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S6_52(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.72)
        table.move_to(DOWN * 0.4)
        board = [Card(r, s, height=0.8) for r, s in
                 [("Q", "s"), ("J", "s"), ("T", "s"), ("3", "d"), ("2", "c")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 1.9 + RIGHT * 0.95 * i + DOWN * 0.4)
        cap = Text("13 x 4 = 52, cruzou o oceano", font=MONO, font_size=24, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.6)
        self.play(FadeIn(cap), run_time=0.5)
        deal_in(self, board, run_time=1.0)
        self.wait(6.8)
        self.play(FadeOut(Group(*self.mobjects)), run_time=0.5)


class S7_Fecho(Scene):
    def construct(self):
        self.camera.background_color = FELT
        table = MiniTable().scale(0.62)
        table.move_to(LEFT * 2.6 + DOWN * 0.2)
        hero = [Card("A", "s", height=0.8), Card("K", "s", height=0.8)]
        for i, c in enumerate(hero):
            c.move_to(table.get_center() + LEFT * 0.5 + RIGHT * 0.9 * i + DOWN * 1.0)
        board = [Card(r, s, height=0.8) for r, s in
                 [("Q", "s"), ("J", "s"), ("T", "s"), ("3", "d"), ("2", "c")]]
        for i, c in enumerate(board):
            c.move_to(table.get_center() + LEFT * 1.6 + RIGHT * 0.8 * i + UP * 0.5)
        five = VGroup(hero[0], hero[1], board[0], board[1], board[2])
        pot = ChipStack("pote", n=3)
        pot.move_to(RIGHT * 3.4 + DOWN * 0.6)
        cap = Text("as 5 acendem, resto apaga", font=MONO, font_size=24, color=GOLD, weight=BOLD)
        cap.to_edge(UP, buff=0.7)
        self.play(FadeIn(table), run_time=0.5)
        deal_in(self, hero + board, run_time=1.2)
        self.play(FadeIn(cap), run_time=0.5)
        ring = SurroundingRectangle(five, color=GOLD, buff=0.15, stroke_width=8)
        self.play(
            Create(ring),
            FadeIn(pot),
            board[3].animate.set_opacity(0.25),
            board[4].animate.set_opacity(0.25),
            table.animate.set_opacity(0.35),
            run_time=0.8,
        )
        self.wait(13.7)
