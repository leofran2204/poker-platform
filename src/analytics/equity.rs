use crate::engine::evaluator::{evaluate_hand, Card};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EquityResult {
    pub win_percentage: f64,
    pub tie_percentage: f64,
    pub loss_percentage: f64,
    pub total_simulations: u32,
}

pub struct EquityCalculator;

impl EquityCalculator {
    /// Gera o baralho padrão de 52 cartas excluindo cartas conhecidas (mão do jogador + comunitárias).
    fn get_remaining_deck(known_cards: &[Card]) -> Vec<Card> {
        let full_deck = crate::crypto::DeckShuffler::generate_standard_deck();
        full_deck
            .into_iter()
            .filter(|c| !known_cards.contains(c))
            .collect()
    }

    /// Executa uma simulação Monte Carlo para determinar a equidade de uma mão de poker.
    pub fn calculate_equity(
        player_hole_cards: &[Card],
        community_cards: &[Card],
        num_opponents: usize,
        num_simulations: u32,
    ) -> EquityResult {
        if player_hole_cards.len() != 2 || num_opponents == 0 || num_simulations == 0 {
            return EquityResult {
                win_percentage: 0.0,
                tie_percentage: 0.0,
                loss_percentage: 100.0,
                total_simulations: 0,
            };
        }

        let mut known_cards = Vec::new();
        known_cards.extend_from_slice(player_hole_cards);
        known_cards.extend_from_slice(community_cards);

        let remaining_deck_base = Self::get_remaining_deck(&known_cards);
        let cards_needed_for_board = 5usize.saturating_sub(community_cards.len());

        let mut wins = 0u32;
        let mut ties = 0u32;
        let mut losses = 0u32;

        let mut rng = ChaCha8Rng::from_entropy();

        for _ in 0..num_simulations {
            let mut deck = remaining_deck_base.clone();
            deck.shuffle(&mut rng);

            // 1. Completar as cartas comunitárias restantes
            let mut sim_board = community_cards.to_vec();
            for _ in 0..cards_needed_for_board {
                if let Some(card) = deck.pop() {
                    sim_board.push(card);
                }
            }

            // 2. Distribuir 2 cartas para cada oponente
            let mut opponent_ranks = Vec::with_capacity(num_opponents);
            let mut valid_sim = true;

            for _ in 0..num_opponents {
                if deck.len() >= 2 {
                    let opp_c1 = deck.pop().unwrap();
                    let opp_c2 = deck.pop().unwrap();
                    let mut opp_full_cards = sim_board.clone();
                    opp_full_cards.push(opp_c1);
                    opp_full_cards.push(opp_c2);
                    opponent_ranks.push(evaluate_hand(&opp_full_cards));
                } else {
                    valid_sim = false;
                    break;
                }
            }

            if !valid_sim {
                continue;
            }

            // 3. Avaliar mão do jogador principal
            let mut player_full_cards = sim_board.clone();
            player_full_cards.extend_from_slice(player_hole_cards);
            let player_rank = evaluate_hand(&player_full_cards);

            // 4. Comparar rank do jogador contra o melhor rank dos oponentes
            let max_opponent_rank = opponent_ranks.into_iter().max();

            match max_opponent_rank {
                Some(best_opp_rank) => {
                    if player_rank > best_opp_rank {
                        wins += 1;
                    } else if player_rank == best_opp_rank {
                        ties += 1;
                    } else {
                        losses += 1;
                    }
                }
                None => wins += 1,
            }
        }

        let total = (wins + ties + losses) as f64;
        let win_pct = if total > 0.0 { (wins as f64 / total) * 100.0 } else { 0.0 };
        let tie_pct = if total > 0.0 { (ties as f64 / total) * 100.0 } else { 0.0 };
        let loss_pct = if total > 0.0 { (losses as f64 / total) * 100.0 } else { 0.0 };

        EquityResult {
            win_percentage: win_pct,
            tie_percentage: tie_pct,
            loss_percentage: loss_pct,
            total_simulations: num_simulations,
        }
    }
}
