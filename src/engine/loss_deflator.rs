use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLossStats {
    pub player_id: String,
    pub total_bet: f64,
    pub amount_won: f64,
    pub cashback_tier_rate: f64, // e.g., 0.05 (5%) to 0.20 (20%)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossDeflatorPayout {
    pub player_id: String,
    pub net_loss: f64,
    pub cashback_amount: f64,
}

/// Calculates progressive cashback (Loss Deflator) for players based on hand results.
///
/// FIX CRÍTICO:
/// Previne o bug de cashback negativo em side pots.
/// O cashback é calculado exclusivamente sobre perdas líquidas REAIS (`net_loss > 0`).
/// Se o jogador teve lucro ou empatou (`amount_won >= total_bet`), o cashback é EXATAMENTE 0.0.
pub fn calculate_loss_deflators(stats: &[PlayerLossStats]) -> Vec<LossDeflatorPayout> {
    let mut results = Vec::new();

    for player in stats {
        // Cálculo seguro da perda líquida
        let net = player.amount_won - player.total_bet;

        // Se o resultado líquido for negativo, o jogador teve uma PERDA LÍQUIDA real
        let net_loss = if net < 0.0 { -net } else { 0.0 };

        // O cashback jamais pode ser negativo
        let cashback_amount = if net_loss > 0.0 {
            (net_loss * player.cashback_tier_rate).max(0.0)
        } else {
            0.0
        };

        results.push(LossDeflatorPayout {
            player_id: player.player_id.clone(),
            net_loss,
            cashback_amount,
        });
    }

    results
}
