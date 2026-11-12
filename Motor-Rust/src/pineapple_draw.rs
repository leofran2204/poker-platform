// pineapple_draw.rs — Apoio de decisão para a rodada de troca do Brazilian
// Pineapple (identificador `pineapple_draw`).
//
// A regra do jogo vive em `game_loop.rs` (troca depois das apostas do flop e do
// turn, descarte no river, all-in não troca). Este módulo só sugere uma troca
// para bots, simulações e o laboratório
// da Academy. É uma heurística simples, pública e determinística; não é
// estratégia ótima e não participa da liquidação nem do Loss Deflator.

use crate::deck::{compare_hands, evaluate_hand_pineapple_draw, Card, HandRank, HandResult};
use std::cmp::Ordering;

/// Sugere qual privada trocar (posição 0 a 2) ou `None` para manter as três.
///
/// A troca só existe com o flop ou o turn na mesa (3 ou 4 comunitárias).
///
/// 1. Encontra o par de privadas que forma a melhor mão com a mesa agora; a
///    terceira carta é a candidata à troca.
/// 2. Mantém quando a mão já é sequência ou melhor.
/// 3. Mantém quando a candidata ainda trabalha: forma par com a mesa ou com
///    outra privada, ou completa quatro cartas do mesmo naipe.
/// 4. Caso contrário, troca a candidata.
///
/// O descarte do river é `deck::pineapple_draw_best_discard`.
pub fn suggested_swap(hole: &[Card], board: &[Card]) -> Option<usize> {
    if hole.len() != 3 || !matches!(board.len(), 3 | 4) {
        return None;
    }
    let mut best: Option<(HandResult, usize)> = None;
    for (i, j, spare) in [(0, 1, 2), (0, 2, 1), (1, 2, 0)] {
        let hand = evaluate_hand_pineapple_draw(&[hole[i], hole[j]], board);
        let better = best
            .as_ref()
            .is_none_or(|(current, _)| compare_hands(&hand, current) == Ordering::Greater);
        if better {
            best = Some((hand, spare));
        }
    }
    let (hand, spare_index) = best?;
    if hand.rank >= HandRank::Straight {
        return None;
    }
    let spare = hole[spare_index];
    let others = hole
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != spare_index)
        .map(|(_, card)| card);
    if board
        .iter()
        .chain(others)
        .any(|card| card.rank == spare.rank)
    {
        return None;
    }
    let same_suit = hole
        .iter()
        .chain(board)
        .filter(|card| card.suit == spare.suit)
        .count();
    if same_suit >= 4 {
        return None;
    }
    Some(spare_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Rank, Suit};

    fn card(text: &str) -> Card {
        let bytes = text.as_bytes();
        let rank = match bytes[0] {
            b'A' => Rank::Ace,
            b'K' => Rank::King,
            b'Q' => Rank::Queen,
            b'J' => Rank::Jack,
            b'T' => Rank::Ten,
            b'9' => Rank::Nine,
            b'8' => Rank::Eight,
            b'7' => Rank::Seven,
            b'6' => Rank::Six,
            b'5' => Rank::Five,
            b'4' => Rank::Four,
            b'3' => Rank::Three,
            _ => Rank::Two,
        };
        let suit = match bytes[1] {
            b's' => Suit::Spades,
            b'h' => Suit::Hearts,
            b'd' => Suit::Diamonds,
            _ => Suit::Clubs,
        };
        Card { rank, suit }
    }

    fn cards(text: &str) -> Vec<Card> {
        text.split_whitespace().map(card).collect()
    }

    #[test]
    fn swaps_the_card_outside_the_best_pair() {
        // A K formam top pair com a mesa; o 2 isolado não trabalha.
        assert_eq!(
            suggested_swap(&cards("Ah Kd 2c"), &cards("As 9h 5d")),
            Some(2)
        );
    }

    #[test]
    fn keeps_a_made_straight_or_better() {
        assert_eq!(suggested_swap(&cards("9h 8d 2c"), &cards("7s 6h 5d")), None);
    }

    #[test]
    fn keeps_a_spare_that_pairs_or_draws_to_a_flush() {
        // O par de ases é a melhor dupla; o 9 de sobra forma par com a mesa e
        // ainda pode virar dois pares ou trinca.
        assert_eq!(suggested_swap(&cards("Ah Ad 9c"), &cards("9h 5s 2d")), None);
        // Quatro copas antes do river: a candidata faz parte do projeto.
        assert_eq!(suggested_swap(&cards("Ah Kd 7h"), &cards("Qh 5h 2c")), None);
        // No turn, sem projeto: a carta de fora da melhor dupla é trocada.
        assert_eq!(
            suggested_swap(&cards("Ah Kd 7c"), &cards("Qh 5h 2c Js")),
            Some(2)
        );
        // Com o river na mesa não há troca, só descarte.
        assert_eq!(
            suggested_swap(&cards("Ah Kd 7h"), &cards("Qh 5h 2c Js 3d")),
            None
        );
    }

    #[test]
    fn needs_three_private_cards_and_a_flop() {
        assert_eq!(suggested_swap(&cards("Ah Kd"), &cards("As 9h 5d")), None);
        assert_eq!(suggested_swap(&cards("Ah Kd 2c"), &[]), None);
    }
}
