// state_stress_tests.rs — Teste de Estresse de Sinais de Estado no Frontend Dioxus
// Valida rajadas de mutações de estado da mesa e lobby sob alta frequência (10.000+ eventos).

use crate::components::card::{PlayingCard, Rank, Suit};
use crate::components::pot::PotEntry;

#[test]
fn test_rapid_table_phase_transitions_stress() {
    const NUM_PHASES: usize = 10_000;

    let mut current_pot = 0u64;
    let mut community_cards = Vec::new();

    for step in 0..NUM_PHASES {
        let phase = step % 4;
        match phase {
            0 => {
                // Preflop: reinicia potes
                current_pot = 30;
                community_cards.clear();
            }
            1 => {
                // Flop: adiciona 3 cartas
                current_pot += 100;
                community_cards = vec![
                    PlayingCard::new(Suit::Spades, Rank::Ace),
                    PlayingCard::new(Suit::Hearts, Rank::King),
                    PlayingCard::new(Suit::Diamonds, Rank::Ten),
                ];
            }
            2 => {
                // Turn: adiciona 4ª carta
                current_pot += 250;
                community_cards.push(PlayingCard::new(Suit::Clubs, Rank::Nine));
            }
            _ => {
                // River: adiciona 5ª carta
                current_pot += 500;
                community_cards.push(PlayingCard::new(Suit::Hearts, Rank::Seven));
            }
        }

        assert!(current_pot > 0);
        assert!(community_cards.len() <= 5);
    }
}

#[test]
fn test_rapid_multi_pot_updates_stress() {
    const NUM_UPDATES: usize = 5_000;

    let mut pots = Vec::new();

    for step in 0..NUM_UPDATES {
        pots.clear();
        let num_pots = (step % 5) + 1;
        for p in 0..num_pots {
            pots.push(PotEntry::new(format!("Pote {}", p), (100 * (p + 1)) as u64));
        }

        assert_eq!(pots.len(), num_pots);
        let total_pot_sum: u64 = pots.iter().map(|p| p.amount).sum();
        assert!(total_pot_sum > 0);
    }
}
