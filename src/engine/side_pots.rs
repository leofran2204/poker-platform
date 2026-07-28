use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contribution {
    pub player_id: String,
    pub total_bet: f64,
    pub has_folded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SidePot {
    pub amount: f64,
    pub eligible_players: Vec<String>,
}

/// Calculates main and side pots for a poker hand.
///
/// FIX CRÍTICO:
/// Jogadores que deram fold (`has_folded = true`) contribuíram para o pote,
/// mas NÃO são elegíveis para ganhar NENHUM pote (principal ou secundário).
/// A elegibilidade é estritamente restrita a jogadores ativos que NÃO foldaram.
pub fn calculate_side_pots(contributions: &[Contribution]) -> Vec<SidePot> {
    if contributions.is_empty() {
        return Vec::new();
    }

    // Identificar os níveis de aposta dos jogadores que foram all-in ou contribuíram
    let mut levels: Vec<f64> = contributions
        .iter()
        .map(|c| c.total_bet)
        .filter(|&bet| bet > 0.0)
        .collect();

    if levels.is_empty() {
        return Vec::new();
    }

    levels.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    levels.dedup();

    let mut side_pots = Vec::new();
    let mut previous_level = 0.0;

    for &level in &levels {
        let cap_delta = level - previous_level;
        if cap_delta <= 0.0 {
            continue;
        }

        let mut pot_amount = 0.0;
        let mut eligible_players = Vec::new();

        for c in contributions {
            if c.total_bet > previous_level {
                let contribution_at_tier = (c.total_bet - previous_level).min(cap_delta);
                pot_amount += contribution_at_tier;

                // CORREÇÃO CRÍTICA: Apenas jogadores que NÃO foldaram entram no pot elegível
                if !c.has_folded {
                    eligible_players.push(c.player_id.clone());
                }
            }
        }

        if pot_amount > 0.0 && !eligible_players.is_empty() {
            side_pots.push(SidePot {
                amount: pot_amount,
                eligible_players,
            });
        } else if pot_amount > 0.0 && side_pots.last().is_some() {
            // Se não sobrou nenhum jogador elegível nesta faixa (raro, ex: todos que apostaram nessa faixa deram fold),
            // o pote acumula no pote anterior elegível.
            let last_idx = side_pots.len() - 1;
            side_pots[last_idx].amount += pot_amount;
        }

        previous_level = level;
    }

    side_pots
}
