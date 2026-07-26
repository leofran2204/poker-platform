// utils.rs — Funções utilitárias compartilhadas
//
// Este módulo centraliza funções utilitárias que são usadas em múltiplos
// módulos do motor de poker, eliminando duplicação de código.
//
// Arquitetura Monetária:
// - Todos os valores monetários usam `u64` centavos inteiros (Zero Float Errors).

use crate::types::Pot;
use std::collections::HashMap;

/// Rateia um valor total em centavos inteiros proporcionalmente entre múltiplos pots.
/// O último pote absorve o resto da divisão inteira para garantir conservação exata.
pub fn ratear_proporcional(pots: &[Pot], valor_total_centavos: u64) -> Vec<u64> {
    if pots.is_empty() {
        return vec![];
    }
    if valor_total_centavos == 0 {
        return vec![0; pots.len()];
    }

    let total_valor_pots: u64 = pots.iter().map(|p| p.amount).sum();
    if total_valor_pots == 0 {
        return vec![0; pots.len()];
    }

    let mut resultado = Vec::with_capacity(pots.len());
    let mut distribuido: u64 = 0;

    for (i, pot) in pots.iter().enumerate() {
        let eh_ultimo = i == pots.len() - 1;
        let valor = if eh_ultimo {
            valor_total_centavos.saturating_sub(distribuido)
        } else {
            let proporcao = pot.amount as f64 / total_valor_pots as f64;
            ((valor_total_centavos as f64 * proporcao).floor()) as u64
        };

        distribuido += valor;
        resultado.push(valor);
    }

    resultado
}

/// Calcula a soma total de todos os pots em centavos
pub fn soma_total_pots(pots: &[Pot]) -> u64 {
    pots.iter().map(|p| p.amount).sum()
}

/// Filtra pots onde um jogador específico é elegível
pub fn pots_elegeiveis<'a>(pots: &'a [Pot], player_id: &str) -> Vec<(usize, &'a Pot)> {
    pots.iter()
        .enumerate()
        .filter(|(_, pot)| pot.is_eligible(player_id))
        .collect()
}

/// Margem de erro (bound) de uma estimativa Monte Carlo por amostragem sem
/// reposição sobre uma população finita.
pub fn mc_error_bound(samples: u64, max_boards: u64) -> f64 {
    crate::loss_deflator::mc_error_bound(samples, max_boards)
}

/// Divide um valor de pote em centavos igualmente entre N vencedores empatados (split pot).
///
/// ### Decisão Arquitetural & Regras Oficiais:
/// 1. **Aritmética Inteira de Centavos (`u64`):** O valor em centavos é dividido inteiramente por N (`total_centavos / n`).
/// 2. **Regra Oficial do Poker Internacional (WSOP / TDA Regra 68):**
///    - O resto indivisível (`total_centavos % n`, em centavos de R$ 0,01) é distribuído 1 a 1
///      aos vencedores empatados na ordem dos assentos a partir do botão.
pub fn dividir_pote_empatado(
    total_centavos: u64,
    vencedores_ids: &[String],
    ordem_assentos: &[String],
) -> HashMap<String, u64> {
    let mut payouts = HashMap::new();
    let n = vencedores_ids.len() as u64;
    if n == 0 || total_centavos == 0 {
        return payouts;
    }

    let base_centavos = total_centavos / n;
    let mut resto_centavos = total_centavos % n;

    // Inicializa todos com o valor base em centavos
    for id in vencedores_ids {
        payouts.insert(id.clone(), base_centavos);
    }

    // Ordena os vencedores pela posição dos assentos para distribuição do resto em centavos
    if resto_centavos > 0 && !ordem_assentos.is_empty() {
        for seat_player_id in ordem_assentos {
            if resto_centavos == 0 {
                break;
            }
            if payouts.contains_key(seat_player_id) {
                if let Some(val) = payouts.get_mut(seat_player_id) {
                    *val += 1;
                    resto_centavos -= 1;
                }
            }
        }
    }

    payouts
}
