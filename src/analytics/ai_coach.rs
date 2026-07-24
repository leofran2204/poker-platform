use crate::analytics::equity::{EquityCalculator, EquityResult};
use crate::engine::evaluator::Card;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SimpleAction {
    Desistir,         // Fold
    PassarAVez,       // Check
    PagarAposta,      // Call
    ApostarForte(f64),// ValueBet
    AumentarBlefe(f64),// BluffRaise
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpponentRangeEstimate {
    pub likely_hand_types: Vec<String>,
    pub range_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendlyCoachAdvice {
    pub headline: String,
    pub simple_action: SimpleAction,
    pub win_chance_label: String, // ex: "Muito Alta (~85%)", "Moderada (~45%)"
    pub friendly_explanation: String,
    pub opponent_range: OpponentRangeEstimate,
    pub math_detail: MathDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathDetail {
    pub win_percentage: f64,
    pub pot_odds_percentage: f64,
    pub expected_value: f64,
}

pub struct AiCoach;

impl AiCoach {
    pub fn calculate_pot_odds(call_amount: f64, current_pot: f64) -> f64 {
        if call_amount <= 0.0 {
            0.0
        } else {
            (call_amount / (current_pot + call_amount)) * 100.0
        }
    }

    pub fn calculate_expected_value(equity: &EquityResult, current_pot: f64, call_amount: f64) -> f64 {
        let win_prob = (equity.win_percentage + (equity.tie_percentage * 0.5)) / 100.0;
        let loss_prob = equity.loss_percentage / 100.0;
        (win_prob * current_pot) - (loss_prob * call_amount)
    }

    /// Estima o alcance (*range*) de mãos do oponente com base na textura da mesa e na ação.
    pub fn estimate_opponent_range(board: &[Card], is_aggressive: bool) -> OpponentRangeEstimate {
        let mut likely_types = Vec::new();

        if board.is_empty() {
            likely_types.push("Pares Médios e Altos (88-AA)".into());
            likely_types.push("Cartas Altas Conectadas (AK, AQ, KQ)".into());
            let desc = if is_aggressive {
                "Oponente demonstrando força pré-flop. Provável que tenha cartas altas ou par formado."
            } else {
                "Alcance amplo pré-flop. Oponente pode estar jogando com diversas mãos."
            };
            return OpponentRangeEstimate {
                likely_hand_types: likely_types,
                range_description: desc.into(),
            };
        }

        // Checar se há possíveis flushes ou straights no board
        let has_flush_draw = board.iter().filter(|c| c.suit == board[0].suit).count() >= 3;

        if is_aggressive {
            likely_types.push("Par Alto da Mesa ou Trinca".into());
            if has_flush_draw {
                likely_types.push("Pedida de Cor (Flush Draw) Forte".into());
            }
            likely_types.push("Blefe Oportunista".into());
        } else {
            likely_types.push("Par Médio ou Par Baixo".into());
            likely_types.push("Mão Fraca tentando ver carta barata".into());
        }

        let desc = format!(
            "Análise do Oponente: Provável que ele possua {}.",
            likely_types.join(" ou ")
        );

        OpponentRangeEstimate {
            likely_hand_types: likely_types,
            range_description: desc,
        }
    }

    /// Analisa a mão e gera um conselho amigável, humano e acessível para jogadores iniciantes/amadores.
    pub fn analyze_hand_friendly(
        player_hole_cards: &[Card],
        community_cards: &[Card],
        num_opponents: usize,
        current_pot: f64,
        call_amount: f64,
        num_simulations: u32,
    ) -> FriendlyCoachAdvice {
        let equity = EquityCalculator::calculate_equity(
            player_hole_cards,
            community_cards,
            num_opponents,
            num_simulations,
        );

        let pot_odds = Self::calculate_pot_odds(call_amount, current_pot);
        let ev = Self::calculate_expected_value(&equity, current_pot, call_amount);

        let win_equity = equity.win_percentage + (equity.tie_percentage * 0.5);

        // Classificação amigável de chance de vitória
        let win_chance_label = if win_equity >= 75.0 {
            format!("Excelente (~{:.0}%)", win_equity)
        } else if win_equity >= 55.0 {
            format!("Boa (~{:.0}%)", win_equity)
        } else if win_equity >= 35.0 {
            format!("Moderada (~{:.0}%)", win_equity)
        } else {
            format!("Baixa (~{:.0}%)", win_equity)
        };

        let opponent_range = Self::estimate_opponent_range(community_cards, call_amount > 0.0);

        let (headline, simple_action, explanation) = if call_amount == 0.0 {
            if win_equity >= 65.0 {
                (
                    "💡 Dica do Coach: Faça uma Aposta de Valor!".to_string(),
                    SimpleAction::ApostarForte(current_pot * 0.5),
                    format!(
                        "Sua mão está muito forte com cerca de {:.0}% de chance de vitória! Aconselhamos apostar R$ {:.2} para extrair fichas dos oponentes.",
                        win_equity, current_pot * 0.5
                    ),
                )
            } else {
                (
                    "💡 Dica do Coach: Passe a Vez (Check)".to_string(),
                    SimpleAction::PassarAVez,
                    format!(
                        "Sua chance de vitória é {:.0}%. Como ninguém apostou ainda, passe a vez de graça para ver a próxima carta sem arriscar fichas.",
                        win_equity
                    ),
                )
            }
        } else {
            if win_equity >= pot_odds && ev > 0.0 {
                if win_equity >= 75.0 {
                    (
                        "🔥 Dica do Coach: Mão Monstra! Aumente a Aposta!".to_string(),
                        SimpleAction::AumentarBlefe(call_amount * 2.5),
                        format!(
                            "Sua mão é dominante ({:.0}% de chance de vitória). Pagar a aposta de R$ {:.2} trará um retorno esperado médio de +R$ {:.2}. Aproveite para aumentar!",
                            win_equity, call_amount, ev
                        ),
                    )
                } else {
                    (
                        "✅ Dica do Coach: Vale a Pena Pagar (Call)".to_string(),
                        SimpleAction::PagarAposta,
                        format!(
                            "Sua chance de vitória ({:.0}%) compensa o valor cobrado (R$ {:.2}). A matemática está a seu favor com um ganho médio estimado de +R$ {:.2}!",
                            win_equity, call_amount, ev
                        ),
                    )
                }
            } else {
                (
                    "⚠️ Dica do Coach: Melhor Desistir (Fold)".to_string(),
                    SimpleAction::Desistir,
                    format!(
                        "A aposta cobrada (R$ {:.2}) está muito alta em relação às suas chances ({:.0}%). Pagar essa aposta traria um prejuízo estimado de -R$ {:.2} no longo prazo. O correto é economizar suas fichas!",
                        call_amount, win_equity, ev.abs()
                    ),
                )
            }
        };

        FriendlyCoachAdvice {
            headline,
            simple_action,
            win_chance_label,
            friendly_explanation: explanation,
            opponent_range,
            math_detail: MathDetail {
                win_percentage: equity.win_percentage,
                pot_odds_percentage: pot_odds,
                expected_value: ev,
            },
        }
    }
}
