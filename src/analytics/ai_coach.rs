use crate::analytics::equity::{EquityCalculator, EquityResult};
use crate::engine::evaluator::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GtoRecommendation {
    Fold,
    Check,
    Call,
    ValueBet(f64),
    BluffRaise(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachAdvice {
    pub equity: EquityResult,
    pub pot_odds_percentage: f64,
    pub expected_value: f64,
    pub recommendation: GtoRecommendation,
    pub reasoning: String,
}

pub struct AiCoach;

impl AiCoach {
    /// Calcula as Pot Odds percentuais: (aposta a pagar / (pote total + aposta a pagar)) * 100%
    pub fn calculate_pot_odds(call_amount: f64, current_pot: f64) -> f64 {
        if call_amount <= 0.0 {
            0.0
        } else {
            (call_amount / (current_pot + call_amount)) * 100.0
        }
    }

    /// Calcula o Valor Esperado (EV) de pagar uma aposta:
    /// EV = (P(Win) * PoteGanho) - (P(Loss) * ApostaPaga)
    pub fn calculate_expected_value(equity: &EquityResult, current_pot: f64, call_amount: f64) -> f64 {
        let win_prob = (equity.win_percentage + (equity.tie_percentage * 0.5)) / 100.0;
        let loss_prob = equity.loss_percentage / 100.0;

        (win_prob * current_pot) - (loss_prob * call_amount)
    }

    /// Gera uma recomendação estratégica GTO baseada em equidade, Pot Odds e EV.
    pub fn analyze_hand(
        player_hole_cards: &[Card],
        community_cards: &[Card],
        num_opponents: usize,
        current_pot: f64,
        call_amount: f64,
        num_simulations: u32,
    ) -> CoachAdvice {
        let equity = EquityCalculator::calculate_equity(
            player_hole_cards,
            community_cards,
            num_opponents,
            num_simulations,
        );

        let pot_odds = Self::calculate_pot_odds(call_amount, current_pot);
        let ev = Self::calculate_expected_value(&equity, current_pot, call_amount);

        let (recommendation, reasoning) = if call_amount == 0.0 {
            if equity.win_percentage >= 70.0 {
                (
                    GtoRecommendation::ValueBet(current_pot * 0.66),
                    format!(
                        "Equidade excelente ({:.1}%). Recomendado Value Bet de R$ {:.2} para extrair valor.",
                        equity.win_percentage,
                        current_pot * 0.66
                    ),
                )
            } else {
                (
                    GtoRecommendation::Check,
                    format!("Equidade moderada ({:.1}%). Passar a vez (Check) sem custo adicional.", equity.win_percentage),
                )
            }
        } else {
            let win_equity = equity.win_percentage + (equity.tie_percentage * 0.5);
            if win_equity >= pot_odds && ev > 0.0 {
                if win_equity >= 75.0 {
                    (
                        GtoRecommendation::BluffRaise(call_amount * 2.5),
                        format!(
                            "Mão forte ({:.1}% equidade > {:.1}% pot odds). Aposta altamente lucrativa (EV: +R$ {:.2}). Aumentar!",
                            win_equity, pot_odds, ev
                        ),
                    )
                } else {
                    (
                        GtoRecommendation::Call,
                        format!(
                            "Equidade ({:.1}%) é superior às Pot Odds ({:.1}%). EV Positivo (+R$ {:.2}). Pagar (Call).",
                            win_equity, pot_odds, ev
                        ),
                    )
                }
            } else {
                (
                    GtoRecommendation::Fold,
                    format!(
                        "Equidade ({:.1}%) é inferior às Pot Odds necessárias ({:.1}%). EV Negativo (-R$ {:.2}). Correto é Foldar.",
                        win_equity, pot_odds, ev.abs()
                    ),
                )
            }
        };

        CoachAdvice {
            equity,
            pot_odds_percentage: pot_odds,
            expected_value: ev,
            recommendation,
            reasoning,
        }
    }
}
