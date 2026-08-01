// rake.rs — Módulo de Rake (Taxa da Casa)
// Migrado de TypeScript (rake.ts) para Rust em 2026-07-02
// Refatorado em 2026-07-24: Arquitetura u64 centavos inteiros
//
// O rake é a comissão que a plataforma cobra por cada mão jogada.
// Regras:
//   1. Percentual em pontos-base inteiros
//   2. Cap único por mão, consumido na ordem main pot → side pots
//   3. Potes com um único participante são devoluções de aposta não coberta
//      e nunca pagam rake
//   4. Descontado ANTES de distribuir aos vencedores
//   5. Arredondamento configurável; o padrão é half-to-even
//   6. No flop, no drop é aplicado pelo contexto da mão

use crate::types::{Pot, TableConfig};

// ─── Tipos locais ───

/// Política de arredondamento da fração de centavo gerada pelo percentual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RakeRounding {
    /// Descarta a fração de centavo.
    Floor,
    /// Arredonda a metade para o inteiro par mais próximo.
    #[default]
    HalfToEven,
}

/// Resultado do cálculo de rake em centavos inteiros
#[derive(Debug, Clone)]
pub struct RakeResult {
    pub total_rake: u64,
    pub club_rake: u64,
    pub platform_fee: u64,
    pub per_pot: Vec<PotRakeEntry>,
    pub pots_after_rake: Vec<Pot>,
    pub total_pot_before_rake: u64,
    /// Soma dos potes disputados por pelo menos dois jogadores.
    pub total_rakeable_pot: u64,
    /// Apostas não cobertas, representadas por potes de um único jogador.
    pub uncalled_amount: u64,
}

/// Entrada individual: quanto cada pote contribuiu para o rake.
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

/// Calcula o rake para UM pote individual usando half-to-even.
///
/// Fórmula: rake = min(round_even(pote × basis_points / 10_000), rake_cap)
pub fn calculate_rake_for_pot(pot_amount: u64, rake_basis_points: u16, rake_cap: u64) -> u64 {
    calculate_rake_for_pot_with_rounding(
        pot_amount,
        rake_basis_points,
        rake_cap,
        RakeRounding::HalfToEven,
    )
}

/// Calcula o rake de um pote com política de arredondamento explícita.
pub fn calculate_rake_for_pot_with_rounding(
    pot_amount: u64,
    rake_basis_points: u16,
    rake_cap: u64,
    rounding: RakeRounding,
) -> u64 {
    if rake_basis_points == 0 || rake_cap == 0 || pot_amount == 0 {
        return 0;
    }

    // Limitar a 100% mantém o resultado significativo mesmo quando callers
    // constroem configurações manualmente fora do limite operacional.
    let effective_basis_points = rake_basis_points.min(10_000);
    let numerator = u128::from(pot_amount) * u128::from(effective_basis_points);
    let quotient = numerator / 10_000;
    let remainder = numerator % 10_000;
    let rounded = match rounding {
        RakeRounding::Floor => quotient,
        RakeRounding::HalfToEven => {
            let above_half = remainder * 2 > 10_000;
            let exactly_half = remainder * 2 == 10_000;
            if above_half || (exactly_half && quotient % 2 == 1) {
                quotient + 1
            } else {
                quotient
            }
        }
    };

    (rounded as u64).min(rake_cap).min(pot_amount)
}

/// Aplica rake inferindo a quantidade de jogadores pelo maior pote.
///
/// Callers que conhecem todos os jogadores que receberam cartas devem usar
/// [`deduct_rake_with_rounding_for_players`].
pub fn deduct_rake(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<u64>,
) -> RakeResult {
    deduct_rake_with_rounding(pots, config, min_pot_for_rake, RakeRounding::HalfToEven)
}

/// Aplica o rake com arredondamento explícito e player count inferido.
pub fn deduct_rake_with_rounding(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<u64>,
    rounding: RakeRounding,
) -> RakeResult {
    let players_dealt = inferred_players_dealt(pots);
    deduct_rake_with_rounding_for_players(pots, config, min_pot_for_rake, rounding, players_dealt)
}

/// Aplica o rake usando a quantidade exata de jogadores que receberam cartas.
///
/// Os potes devem estar na ordem main pot → side pots. Cada pote disputado
/// contribui com seu percentual até que o cap único da mão seja atingido.
/// Potes com menos de dois participantes representam aposta não coberta e são
/// preservados integralmente.
pub fn deduct_rake_with_rounding_for_players(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<u64>,
    rounding: RakeRounding,
    players_dealt: usize,
) -> RakeResult {
    let min_pot = min_pot_for_rake.unwrap_or(0);
    let total_pot = soma_total_pots_centavos(pots);
    let total_rakeable_pot: u64 = pots
        .iter()
        .filter(|pot| pot.eligible_players.len() >= 2)
        .map(|pot| pot.amount)
        .sum();
    let uncalled_amount = total_pot.saturating_sub(total_rakeable_pot);
    let rake_cap = config.rake_cap_for_players(players_dealt);

    if total_rakeable_pot < min_pot
        || total_rakeable_pot == 0
        || config.rake_basis_points == 0
        || rake_cap == 0
    {
        return zero_rake_result(pots, total_pot, total_rakeable_pot, uncalled_amount);
    }

    let mut cap_remaining = rake_cap;
    let mut per_pot = Vec::with_capacity(pots.len());

    for (pot_index, pot) in pots.iter().enumerate() {
        let pot_rake = if pot.eligible_players.len() < 2 || cap_remaining == 0 {
            0
        } else {
            calculate_rake_for_pot_with_rounding(
                pot.amount,
                config.rake_basis_points,
                cap_remaining,
                rounding,
            )
        };
        cap_remaining = cap_remaining.saturating_sub(pot_rake);
        per_pot.push(PotRakeEntry {
            pot_index,
            rake: pot_rake,
        });
    }

    let effective_total_rake: u64 = per_pot.iter().map(|entry| entry.rake).sum();
    
    // FASE 2: B2B Split - 15% Platform Fee, 85% Club Rake
    // Arredonda a favor da plataforma (math.ceil-ish) ou floor. Usaremos floor pro platform e resto pro clube.
    let platform_fee = (effective_total_rake * 15) / 100;
    let club_rake = effective_total_rake.saturating_sub(platform_fee);

    let pots_after_rake = apply_rake_to_pots(pots, &per_pot);

    RakeResult {
        total_rake: effective_total_rake,
        club_rake,
        platform_fee,
        per_pot,
        pots_after_rake,
        total_pot_before_rake: total_pot,
        total_rakeable_pot,
        uncalled_amount,
    }
}

/// Aplica no-flop-no-drop inferindo a quantidade de jogadores pelo maior pote.
pub fn deduct_rake_for_hand(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<u64>,
    flop_was_dealt: bool,
    rounding: RakeRounding,
) -> RakeResult {
    deduct_rake_for_hand_with_player_count(
        pots,
        config,
        min_pot_for_rake,
        flop_was_dealt,
        rounding,
        inferred_players_dealt(pots),
    )
}

/// Aplica no-flop-no-drop e o cap do número exato de jogadores que receberam cartas.
pub fn deduct_rake_for_hand_with_player_count(
    pots: &[Pot],
    config: &TableConfig,
    min_pot_for_rake: Option<u64>,
    flop_was_dealt: bool,
    rounding: RakeRounding,
    players_dealt: usize,
) -> RakeResult {
    if !flop_was_dealt {
        let total_pot = soma_total_pots_centavos(pots);
        let total_rakeable_pot: u64 = pots
            .iter()
            .filter(|pot| pot.eligible_players.len() >= 2)
            .map(|pot| pot.amount)
            .sum();
        return zero_rake_result(
            pots,
            total_pot,
            total_rakeable_pot,
            total_pot.saturating_sub(total_rakeable_pot),
        );
    }

    deduct_rake_with_rounding_for_players(pots, config, min_pot_for_rake, rounding, players_dealt)
}

fn inferred_players_dealt(pots: &[Pot]) -> usize {
    pots.iter()
        .map(|pot| pot.eligible_players.len())
        .max()
        .unwrap_or(0)
}

fn zero_rake_result(
    pots: &[Pot],
    total_pot: u64,
    total_rakeable_pot: u64,
    uncalled_amount: u64,
) -> RakeResult {
    RakeResult {
        total_rake: 0,
        club_rake: 0,
        platform_fee: 0,
        per_pot: pots
            .iter()
            .enumerate()
            .map(|(pot_index, _)| PotRakeEntry { pot_index, rake: 0 })
            .collect(),
        pots_after_rake: pots.to_vec(),
        total_pot_before_rake: total_pot,
        total_rakeable_pot,
        uncalled_amount,
    }
}

fn apply_rake_to_pots(pots: &[Pot], per_pot: &[PotRakeEntry]) -> Vec<Pot> {
    pots.iter()
        .enumerate()
        .map(|(pot_index, pot)| Pot {
            amount: pot.amount.saturating_sub(per_pot[pot_index].rake),
            eligible_players: pot.eligible_players.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Pot, TableConfig};

    #[test]
    fn b2b_rake_split_always_totals_100_percent() {
        let config = TableConfig::new(200, 500, 3000); // Rake 5%, cap 30.00
        let pots = vec![
            Pot {
                amount: 1573, // Pote arbitrário
                eligible_players: vec![uuid::Uuid::new_v4(), uuid::Uuid::new_v4()],
            },
        ];

        let result = deduct_rake_with_rounding(
            &pots,
            &config,
            None,
            RakeRounding::HalfToEven,
        );

        // A soma do fee da plataforma (15%) + rake do clube (85%) DEVE ser idêntica ao total coletado.
        assert_eq!(
            result.platform_fee + result.club_rake,
            result.total_rake,
            "Rake math violation! Platform {} + Club {} != Total {}",
            result.platform_fee, result.club_rake, result.total_rake
        );
        
        // Assert de sanidade dos valores (exato 15% via floor)
        // O total_rake deve ser min(round(1573 * 500 / 10000), 3000) = round(78.65) = 79
        assert_eq!(result.total_rake, 79);
        assert_eq!(result.platform_fee, (79 * 15) / 100); // floor(11.85) = 11
        assert_eq!(result.club_rake, 79 - 11); // 68
    }
}
