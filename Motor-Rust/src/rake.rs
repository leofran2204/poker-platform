// rake.rs — Módulo de Rake (Taxa da Casa)
// Migrado de TypeScript (rake.ts) para Rust em 2026-07-02
// Refatorado em 2026-07-06: usa types.rs e utils.rs compartilhados
//
// O rake é a comissão que a plataforma cobra por cada mão jogada.
// Regras:
//   1. Percentual do pote total (rakePercent)
//   2. Nunca ultrapassa o cap (rakeCap)
//   3. Descontado ANTES de distribuir aos vencedores
//   4. Arredondado para baixo (floor) — casa nunca favorece jogador
//   5. Pote mínimo = 2× big blind para cobrar rake

use crate::types::{Pot, TableConfig};
use crate::utils::{soma_total_pots, truncar_2_casas};

// ─── Tipos locais ───

/// Resultado do cálculo de rake
#[derive(Debug, Clone)]
pub struct RakeResult {
    pub total_rake: f64,            // rake total deduzido
    pub per_pot: Vec<PotRakeEntry>, // rateio por pote
    pub pots_after_rake: Vec<Pot>,  // pots após dedução
    pub total_pot_before_rake: f64, // soma de todos os pots antes do rake
}

/// Entrada individual do rateio: quanto cada pote contribuiu
#[derive(Debug, Clone)]
pub struct PotRakeEntry {
    pub pot_index: usize,
    pub rake: f64,
}

// ─── Funções públicas ───

/// Calcula o rake para UM pote individual.
///
/// Fórmula: rake = min(pote × rakePercent / 100, rakeCap)
/// Depois: trunca para 2 casas decimais (regra fundamental do software)
///
/// Retorna 0 se rakePercent ou rakeCap forem zero.
pub fn calculate_rake_for_pot(pot_amount: f64, rake_percent: f64, rake_cap: f64) -> f64 {
    if rake_percent <= 0.0 || rake_cap == 0.0 {
        return 0.0;
    }

    let raw_rake = (pot_amount * rake_percent) / 100.0;
    let capped_rake = raw_rake.min(rake_cap);
    truncar_2_casas(capped_rake)
}

/// Aplica o rake sobre todos os pots de uma mão.
///
/// O rake é distribuído PROPORCIONALMENTE entre os pots.
/// O último pote absorve o resto da divisão para evitar erro de arredondamento.
///
/// Se o pote total for menor que `min_pot_for_rake` (default: 2× big blind),
/// não cobra rake.
pub fn deduct_rake(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<f64>,
) -> RakeResult {
    let min_pot = min_pot_for_rake.unwrap_or(config.big_blind * 2.0);
    let total_pot = soma_total_pots(pots);

    // Pote muito pequeno → sem rake
    if total_pot < min_pot {
        return zero_rake_result(pots, total_pot);
    }

    let total_rake = calculate_rake_for_pot(total_pot, config.rake_percent, config.rake_cap);
    if total_rake == 0.0 {
        return zero_rake_result(pots, total_pot);
    }

    let per_pot = distribute_rake_proportionally(pots, total_pot, total_rake);
    let pots_after_rake = apply_rake_to_pots(pots, &per_pot);

    RakeResult {
        total_rake,
        per_pot,
        pots_after_rake,
        total_pot_before_rake: total_pot,
    }
}

/// Constrói um RakeResult sem rake (pote abaixo do mínimo ou rake zero)
fn zero_rake_result(pots: &[Pot], total_pot: f64) -> RakeResult {
    RakeResult {
        total_rake: 0.0,
        per_pot: pots
            .iter()
            .enumerate()
            .map(|(i, _)| PotRakeEntry {
                pot_index: i,
                rake: 0.0,
            })
            .collect(),
        pots_after_rake: pots.to_vec(),
        total_pot_before_rake: total_pot,
    }
}

/// Rateia o rake total proporcionalmente entre os pots.
/// O último pote absorve o resto para evitar erro de arredondamento.
fn distribute_rake_proportionally(
    pots: &[Pot],
    total_pot: f64,
    total_rake: f64,
) -> Vec<PotRakeEntry> {
    let mut per_pot: Vec<PotRakeEntry> = Vec::with_capacity(pots.len());
    let mut distributed_rake: f64 = 0.0;

    for (i, pot) in pots.iter().enumerate() {
        let is_last = i == pots.len() - 1;
        let pot_rake = if is_last {
            total_rake - distributed_rake
        } else {
            let proportion = pot.amount / total_pot;
            let raw = total_rake * proportion;
            truncar_2_casas(raw)
        };

        distributed_rake += pot_rake;
        per_pot.push(PotRakeEntry {
            pot_index: i,
            rake: pot_rake,
        });
    }

    per_pot
}

/// Subtrai o rake de cada pot, preservando a lista de elegíveis
fn apply_rake_to_pots(pots: &[Pot], per_pot: &[PotRakeEntry]) -> Vec<Pot> {
    pots.iter()
        .enumerate()
        .map(|(i, pot)| Pot {
            amount: pot.amount - per_pot[i].rake,
            eligible_players: pot.eligible_players.clone(),
        })
        .collect()
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pot(amount: f64) -> Pot {
        Pot {
            amount,
            eligible_players: vec!["p1".into(), "p2".into()],
        }
    }

    fn default_config() -> TableConfig {
        TableConfig {
            big_blind: 10.0,
            rake_percent: 5.0,
            rake_cap: 10.0,
        }
    }

    // ─── calculate_rake_for_pot ───

    #[test]
    fn test_rake_below_cap() {
        // 5% de 100 = 5, abaixo do cap de 10
        let rake = calculate_rake_for_pot(100.0, 5.0, 10.0);
        assert!((rake - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rake_at_cap() {
        // 5% de 300 = 15, mas cap é 10
        let rake = calculate_rake_for_pot(300.0, 5.0, 10.0);
        assert!((rake - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rake_truncation() {
        // 5% de 30 = 1.5, trunc = 1.5 (2 casas decimais)
        let rake = calculate_rake_for_pot(30.0, 5.0, 10.0);
        assert!((rake - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rake_zero_percent() {
        let rake = calculate_rake_for_pot(100.0, 0.0, 10.0);
        assert!((rake - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rake_zero_cap() {
        let rake = calculate_rake_for_pot(100.0, 5.0, 0.0);
        assert!((rake - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rake_small_pot() {
        // 5% de 1 = 0.05, trunc = 0.05
        let rake = calculate_rake_for_pot(1.0, 5.0, 10.0);
        assert!((rake - 0.05).abs() < f64::EPSILON);
    }

    // ─── deduct_rake ───

    #[test]
    fn test_deduct_single_pot() {
        let pots = vec![make_pot(200.0)];
        let config = default_config();
        let result = deduct_rake(&pots, &config, None);

        // 5% de 200 = 10, cap = 10 → rake = 10
        assert!((result.total_rake - 10.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 190.0).abs() < f64::EPSILON);
        assert!((result.total_pot_before_rake - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduct_multiple_pots_proportional() {
        // main pot = 100, side pot = 50, total = 150
        // 5% de 150 = 7.5, trunc = 7.5
        // main: trunc(7.5 × 100/150) = trunc(5.0) = 5.0
        // side: 7.5 - 5.0 = 2.5 (último absorve resto)
        let pots = vec![make_pot(100.0), make_pot(50.0)];
        let config = default_config();
        let result = deduct_rake(&pots, &config, None);

        assert!((result.total_rake - 7.5).abs() < f64::EPSILON);
        assert!((result.per_pot[0].rake - 5.0).abs() < f64::EPSILON);
        assert!((result.per_pot[1].rake - 2.5).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 95.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[1].amount - 47.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduct_below_minimum() {
        // BB=10, minPot=20. Pote total=15 → sem rake
        let pots = vec![make_pot(15.0)];
        let config = default_config();
        let result = deduct_rake(&pots, &config, None);

        assert!((result.total_rake - 0.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduct_exactly_at_minimum() {
        // BB=10, minPot=20. Pote total=20 → cobra rake
        let pots = vec![make_pot(20.0)];
        let config = default_config();
        let result = deduct_rake(&pots, &config, None);

        // 5% de 20 = 1.0
        assert!((result.total_rake - 1.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 19.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduct_custom_min_pot() {
        let pots = vec![make_pot(50.0)];
        let config = default_config();
        // minPot custom = 100 → pote de 50 não qualifica
        let result = deduct_rake(&pots, &config, Some(100.0));

        assert!((result.total_rake - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduct_three_pots_last_absorbs_remainder() {
        // 3 pots: 100, 60, 40 → total = 200
        // 5% de 200 = 10.0
        // pot0: trunc(10 × 100/200) = 5.0
        // pot1: trunc(10 × 60/200) = 3.0
        // pot2: 10 - 5 - 3 = 2.0 (último absorve)
        let pots = vec![make_pot(100.0), make_pot(60.0), make_pot(40.0)];
        let config = default_config();
        let result = deduct_rake(&pots, &config, None);

        assert!((result.total_rake - 10.0).abs() < f64::EPSILON);
        assert!((result.per_pot[0].rake - 5.0).abs() < f64::EPSILON);
        assert!((result.per_pot[1].rake - 3.0).abs() < f64::EPSILON);
        assert!((result.per_pot[2].rake - 2.0).abs() < f64::EPSILON);
        // Soma dos rakes = total
        let sum: f64 = result.per_pot.iter().map(|e| e.rake).sum();
        assert!((sum - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deduct_eligible_players_preserved() {
        let pot = Pot {
            amount: 200.0,
            eligible_players: vec!["alice".into(), "bob".into(), "charlie".into()],
        };
        let config = default_config();
        let result = deduct_rake(&[pot], &config, None);

        assert_eq!(result.pots_after_rake[0].eligible_players.len(), 3);
        assert!(result.pots_after_rake[0]
            .eligible_players
            .contains(&"alice".into()));
    }
}
