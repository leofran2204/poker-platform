// rake.rs — Módulo de Rake (Taxa da Casa)
// Migrado de TypeScript (rake.ts) para Rust em 2026-07-02
// Refatorado em 2026-07-24: Arquitetura u64 centavos inteiros
//
// O rake é a comissão que a plataforma cobra por cada mão jogada.
// Regras:
//   1. Percentual do pote total em pontos-base inteiros
//   2. Nunca ultrapassa o cap (rakeCap) em centavos
//   3. Descontado ANTES de distribuir aos vencedores
//   4. Arredondado para baixo (floor) em centavos
//   5. Pote mínimo = 2× big blind para cobrar rake

use crate::types::{Pot, TableConfig};

// ─── Tipos locais ───

/// Resultado do cálculo de rake em centavos inteiros
#[derive(Debug, Clone)]
pub struct RakeResult {
    pub total_rake: u64,            // rake total deduzido em centavos
    pub per_pot: Vec<PotRakeEntry>, // rateio por pote em centavos
    pub pots_after_rake: Vec<Pot>,  // pots após dedução
    pub total_pot_before_rake: u64, // soma de todos os pots antes do rake
}

/// Entrada individual do rateio: quanto cada pote contribuiu (em centavos)
#[derive(Debug, Clone)]
pub struct PotRakeEntry {
    pub pot_index: usize,
    pub rake: u64,
}

// ─── Funções públicas ───

/// Soma o valor total de uma lista de pots em centavos
pub fn soma_total_pots_centavos(pots: &[Pot]) -> u64 {
    pots.iter().map(|p| p.amount).sum()
}

/// Calcula o rake para UM pote individual em centavos.
///
/// Fórmula: rake = min(floor(pote_centavos × rake_basis_points / 10_000), rake_cap)
///
/// Retorna 0 se o rake ou o cap forem zero. O produto usa `u128` para
/// preservar precisão e evitar overflow antes da divisão em centavos.
pub fn calculate_rake_for_pot(pot_amount: u64, rake_basis_points: u16, rake_cap: u64) -> u64 {
    if rake_basis_points == 0 || rake_cap == 0 || pot_amount == 0 {
        return 0;
    }

    // Callers normally enforce the operational limit of 1,000 bps (10%),
    // but this boundary must also remain safe for manually constructed configs.
    // At most 10,000 bps (100%) is meaningful and keeps the intermediate
    // quotient representable as u64.
    let effective_basis_points = rake_basis_points.min(10_000);
    let raw_rake = ((u128::from(pot_amount) * u128::from(effective_basis_points)) / 10_000) as u64;
    raw_rake.min(rake_cap).min(pot_amount)
}

/// Aplica o rake sobre todos os pots de uma mão.
///
/// O rake é distribuído PROPORCIONALMENTE entre os pots em centavos inteiros.
/// O último pote absorve o resto da divisão para evitar erro de arredondamento.
///
/// Se o pote total for menor que `min_pot_for_rake` (default: 2× big blind),
/// não cobra rake.
pub fn deduct_rake(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<u64>,
) -> RakeResult {
    let min_pot = min_pot_for_rake.unwrap_or(config.big_blind * 2);
    let total_pot = soma_total_pots_centavos(pots);

    // Pote muito pequeno → sem rake
    if total_pot < min_pot {
        return zero_rake_result(pots, total_pot);
    }

    let total_rake = calculate_rake_for_pot(total_pot, config.rake_basis_points, config.rake_cap);
    if total_rake == 0 {
        return zero_rake_result(pots, total_pot);
    }

    let per_pot = distribute_rake_proportionally(pots, total_pot, total_rake);
    let effective_total_rake: u64 = per_pot.iter().map(|p| p.rake).sum();
    let pots_after_rake = apply_rake_to_pots(pots, &per_pot);

    RakeResult {
        total_rake: effective_total_rake,
        per_pot,
        pots_after_rake,
        total_pot_before_rake: total_pot,
    }
}

/// Constrói um RakeResult sem rake (pote abaixo do mínimo ou rake zero)
fn zero_rake_result(pots: &[Pot], total_pot: u64) -> RakeResult {
    RakeResult {
        total_rake: 0,
        per_pot: pots
            .iter()
            .enumerate()
            .map(|(i, _)| PotRakeEntry {
                pot_index: i,
                rake: 0,
            })
            .collect(),
        pots_after_rake: pots.to_vec(),
        total_pot_before_rake: total_pot,
    }
}

/// Rateia o rake total proporcionalmente entre os pots (em centavos).
/// O último pote absorve o resto para garantir conservação perfeita de centavos.
fn distribute_rake_proportionally(
    pots: &[Pot],
    total_pot: u64,
    total_rake: u64,
) -> Vec<PotRakeEntry> {
    let mut per_pot: Vec<PotRakeEntry> = Vec::with_capacity(pots.len());
    let mut distributed_rake: u64 = 0;

    for (i, pot) in pots.iter().enumerate() {
        let is_last = i == pots.len() - 1;
        let raw_pot_rake = if is_last {
            total_rake.saturating_sub(distributed_rake)
        } else {
            ((u128::from(pot.amount) * u128::from(total_rake)) / u128::from(total_pot)) as u64
        };

        let pot_rake = raw_pot_rake.min(pot.amount);

        distributed_rake += pot_rake;
        per_pot.push(PotRakeEntry {
            pot_index: i,
            rake: pot_rake,
        });
    }

    per_pot
}

/// Subtrai o rake de cada pot em centavos, preservando a lista de elegíveis
fn apply_rake_to_pots(pots: &[Pot], per_pot: &[PotRakeEntry]) -> Vec<Pot> {
    pots.iter()
        .enumerate()
        .map(|(i, pot)| Pot {
            amount: pot.amount.saturating_sub(per_pot[i].rake),
            eligible_players: pot.eligible_players.clone(),
        })
        .collect()
}
