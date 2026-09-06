use serde::{Deserialize, Serialize};

/// Entrada do helper legado. O valor elegível deve chegar já líquido de rake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLossStats {
    pub player_id: String,
    pub eligible_loss_after_rake: f64,
    /// Equity do perdedor no instante em que o all-in foi pago (0.0 a 1.0).
    pub loser_equity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossDeflatorPayout {
    pub player_id: String,
    pub net_loss: f64,
    pub cashback_amount: f64,
}

/// Retorna a taxa normativa para a equity do perdedor no instante do all-in.
///
/// - abaixo de 56%: 0%
/// - de 56% (inclusive) a 66% (exclusive): 7%
/// - de 66% (inclusive) a 76% (exclusive): 15%
/// - de 76% (inclusive) a 86% (exclusive): 25%
/// - 86% ou mais: 35%
pub fn cashback_rate_for_equity(loser_equity: f64) -> f64 {
    if !loser_equity.is_finite() || !(0.0..=1.0).contains(&loser_equity) {
        0.0
    } else if loser_equity >= 0.86 {
        0.35
    } else if loser_equity >= 0.76 {
        0.25
    } else if loser_equity >= 0.66 {
        0.15
    } else if loser_equity >= 0.56 {
        0.07
    } else {
        0.0
    }
}

/// Calcula o Loss Deflator exclusivamente sobre a perda elegível após o rake.
///
/// Este helper é mantido para simuladores legados; o fluxo principal vive no
/// `Motor-Rust` e calcula os potes e side pots antes de aplicar a mesma tabela.
pub fn calculate_loss_deflators(stats: &[PlayerLossStats]) -> Vec<LossDeflatorPayout> {
    stats
        .iter()
        .map(|player| {
            let net_loss = player.eligible_loss_after_rake.max(0.0);
            let rate = cashback_rate_for_equity(player.loser_equity);
            LossDeflatorPayout {
                player_id: player.player_id.clone(),
                net_loss,
                cashback_amount: net_loss * rate,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_equity_boundaries_match_the_normative_rule() {
        let cases = [
            (0.559_999, 0.0),
            (0.56, 0.07),
            (0.659_999, 0.07),
            (0.66, 0.15),
            (0.759_999, 0.15),
            (0.76, 0.25),
            (0.859_999, 0.25),
            (0.86, 0.35),
            (1.0, 0.35),
        ];

        for (equity, expected) in cases {
            assert_eq!(cashback_rate_for_equity(equity), expected);
        }
        assert_eq!(cashback_rate_for_equity(f64::NAN), 0.0);
    }
}
