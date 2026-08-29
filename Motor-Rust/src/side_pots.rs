// side-pots.rs — Calculadora de Side Pots em Centavos Inteiros (u64)
// Migrado de TypeScript (side-pots.ts) para Rust em 2026-07-02
// Refatorado em 2026-07-24: Arquitetura u64 centavos inteiros (Zero Float Errors)

use crate::deck::{
    compare_hands, evaluate_hand, evaluate_hand_short_deck, evaluate_hand_short_deck_omaha, Card,
    HandResult,
};
use crate::types::{PokerVariant, Pot};
use crate::utils::dividir_pote_empatado;
use std::collections::HashMap;

/// Contribuição de um jogador para o pote em centavos inteiros
#[derive(Debug, Clone)]
pub struct PlayerContribution {
    pub player_id: String,
    pub amount: u64,
}

/// Resultado do cálculo de side pots em centavos
#[derive(Debug, Clone)]
pub struct SidePotsResult {
    pub pots: Vec<Pot>,
    pub payouts: HashMap<String, u64>,
    pub contributions: Vec<PlayerContribution>,
}

/// Jogador simplificado para cálculo de side pots com total_bet em centavos
#[derive(Debug, Clone)]
pub struct PlayerForPots {
    pub id: String,
    pub total_bet: u64,
    pub has_folded: bool,
    pub cards: Vec<Card>,
}

/// Calcula os side pots em centavos inteiros a partir das contribuições dos jogadores.
pub fn calculate_side_pots(players: &[PlayerForPots]) -> Vec<Pot> {
    // 1. Coletar contribuições únicas em centavos (apenas jogadores que colocaram fichas)
    let mut contributions: Vec<PlayerContribution> = players
        .iter()
        .filter(|p| p.total_bet > 0)
        .map(|p| PlayerContribution {
            player_id: p.id.clone(),
            amount: p.total_bet,
        })
        .collect();

    contributions.sort_by_key(|c| c.amount);

    if contributions.is_empty() {
        return Vec::new();
    }

    let mut pots = Vec::new();
    let mut previous_level: u64 = 0;

    // 2. Para cada nível distinto de aposta em centavos, criar um pote
    let mut i = 0;
    while i < contributions.len() {
        let current_level = contributions[i].amount;
        let level_diff = current_level.saturating_sub(previous_level);

        if level_diff > 0 {
            let eligible_players: Vec<String> = contributions[i..]
                .iter()
                .map(|c| c.player_id.clone())
                .collect();

            let pot_amount = level_diff * eligible_players.len() as u64;

            if pot_amount > 0 {
                pots.push(Pot {
                    amount: pot_amount,
                    eligible_players,
                });
            }
        }

        previous_level = current_level;
        while i < contributions.len() && contributions[i].amount == current_level {
            i += 1;
        }
    }

    pots
}

/// Distribui cada pote em centavos entre as melhores mãos elegíveis.
///
/// Para compatibilidade, a ordem recebida em `players` define a prioridade do
/// centavo ímpar. O GameLoop deve preferir
/// [`distribute_pots_with_seat_order`], que recebe explicitamente a ordem a
/// partir do botão.
pub fn distribute_pots(
    pots: &[Pot],
    players: &[PlayerForPots],
    community_cards: &[Card],
) -> HashMap<String, u64> {
    let player_order: Vec<String> = players.iter().map(|player| player.id.clone()).collect();
    distribute_pots_with_seat_order(pots, players, community_cards, &player_order)
}

/// Distribui potes aplicando a regra WSOP/TDA do centavo ímpar.
///
/// `seat_order_from_button` deve começar no primeiro assento à esquerda do
/// botão e seguir a ordem dos assentos. Entradas ausentes são completadas pela
/// ordem dos vencedores para preservar a conservação do pote.
pub fn distribute_pots_with_seat_order(
    pots: &[Pot],
    players: &[PlayerForPots],
    community_cards: &[Card],
    seat_order_from_button: &[String],
) -> HashMap<String, u64> {
    distribute_pots_with_seat_order_for_variant(
        pots,
        players,
        community_cards,
        seat_order_from_button,
        PokerVariant::Holdem,
    )
}

pub fn distribute_pots_with_seat_order_for_variant(
    pots: &[Pot],
    players: &[PlayerForPots],
    community_cards: &[Card],
    seat_order_from_button: &[String],
    variant: PokerVariant,
) -> HashMap<String, u64> {
    let mut payouts: HashMap<String, u64> = HashMap::new();
    let player_hands = precompute_hands_for_variant(players, community_cards, variant);

    for pot in pots {
        let winners = find_winners_for_pot(pot, players, &player_hands);
        if winners.is_empty() {
            continue;
        }

        let mut ordered_winners: Vec<String> = seat_order_from_button
            .iter()
            .filter(|player_id| winners.contains(*player_id))
            .cloned()
            .collect();
        for winner_id in &winners {
            if !ordered_winners.contains(winner_id) {
                ordered_winners.push(winner_id.clone());
            }
        }

        let shares = dividir_pote_empatado(pot.amount, &winners, &ordered_winners);
        for (winner_id, share) in shares {
            *payouts.entry(winner_id).or_insert(0) += share;
        }
    }

    payouts
}

/// Pré-computa as mãos de todos os jogadores ativos (Hold'em).
pub fn precompute_hands(
    players: &[PlayerForPots],
    community_cards: &[Card],
) -> HashMap<String, HandResult> {
    precompute_hands_for_variant(players, community_cards, PokerVariant::Holdem)
}

pub fn precompute_hands_for_variant(
    players: &[PlayerForPots],
    community_cards: &[Card],
    variant: PokerVariant,
) -> HashMap<String, HandResult> {
    let mut hands: HashMap<String, HandResult> = HashMap::new();
    for player in players {
        if !player.has_folded {
            let hand = match variant {
                PokerVariant::ShortDeckOmaha => {
                    evaluate_hand_short_deck_omaha(&player.cards, community_cards)
                }
                PokerVariant::ShortDeck => {
                    evaluate_hand_short_deck(&player.cards, community_cards)
                }
                PokerVariant::Holdem => evaluate_hand(&player.cards, community_cards),
            };
            hands.insert(player.id.clone(), hand);
        }
    }
    hands
}

/// Encontra o(s) vencedor(es) de um pote entre os jogadores elegíveis.
pub fn find_winners_for_pot(
    pot: &Pot,
    players: &[PlayerForPots],
    player_hands: &HashMap<String, HandResult>,
) -> Vec<String> {
    let eligible: Vec<(String, HandResult)> = pot
        .eligible_players
        .iter()
        .filter_map(|player_id| {
            let player = players.iter().find(|p| p.id == *player_id)?;
            if player.has_folded {
                return None;
            }
            let hand = player_hands.get(player_id)?;
            Some((player_id.clone(), hand.clone()))
        })
        .collect();

    if eligible.is_empty() {
        return vec![];
    }

    let mut best_winners: Vec<String> = vec![eligible[0].0.clone()];
    let mut best_hand = &eligible[0].1;

    for (player_id, hand) in eligible.iter().skip(1) {
        let cmp = compare_hands(hand, best_hand);
        if cmp == std::cmp::Ordering::Greater {
            best_winners = vec![player_id.clone()];
            best_hand = hand;
        } else if cmp == std::cmp::Ordering::Equal {
            best_winners.push(player_id.clone());
        }
    }

    best_winners
}

/// Resolve side pots e redistribui payouts retornando SidePotsResult.
pub fn resolve_side_pots(players: &[PlayerForPots], community_cards: &[Card]) -> SidePotsResult {
    let pots = calculate_side_pots(players);
    let payouts = distribute_pots(&pots, players, community_cards);
    let contributions = players
        .iter()
        .map(|p| PlayerContribution {
            player_id: p.id.clone(),
            amount: p.total_bet,
        })
        .collect();
    SidePotsResult {
        pots,
        payouts,
        contributions,
    }
}
