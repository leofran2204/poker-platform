// utils.rs — Funções utilitárias compartilhadas
//
// Este módulo centraliza funções utilitárias que são usadas em múltiplos
// módulos do motor de poker, eliminando duplicação de código.

use crate::types::Pot;

/// Trunca um valor f64 para 2 casas decimais
/// Regra fundamental do software: casa nunca favorece jogador
///
/// # Exemplo
/// ```
/// use poker_engine::utils::truncar_2_casas;
///
/// assert_eq!(truncar_2_casas(5.678), 5.67);
/// assert_eq!(truncar_2_casas(10.0), 10.0);
/// ```
#[inline]
pub fn truncar_2_casas(valor: f64) -> f64 {
    (valor * 100.0).trunc() / 100.0
}

/// Rateia um valor total proporcionalmente entre múltiplos pots
/// O último pote absorve o resto para evitar erro de arredondamento
///
/// # Arguments
/// * `pots` - lista de pots com seus valores
/// * `valor_total` - valor a ser rateado
///
/// # Returns
/// Vec com o valor rateado para cada pote (mantém a ordem)
///
/// # Exemplo
/// ```
/// use poker_engine::utils::ratear_proporcional;
/// use poker_engine::types::Pot;
///
/// let pots = vec![Pot::new(100.0, vec![]), Pot::new(200.0, vec![])];
/// let rateio = ratear_proporcional(&pots, 30.0);
/// // rateio[0] = 10.0 (30 * 100/300)
/// // rateio[1] = 20.0 (30 - 10, último absorve)
/// ```
pub fn ratear_proporcional(pots: &[Pot], valor_total: f64) -> Vec<f64> {
    if pots.is_empty() {
        return vec![];
    }
    if valor_total == 0.0 {
        return vec![0.0; pots.len()];
    }

    let total_valor_pots: f64 = pots.iter().map(|p| p.amount).sum();
    if total_valor_pots == 0.0 {
        return vec![0.0; pots.len()];
    }

    let mut resultado = Vec::with_capacity(pots.len());
    let mut distribuido = 0.0;

    for (i, pot) in pots.iter().enumerate() {
        let eh_ultimo = i == pots.len() - 1;
        let valor = if eh_ultimo {
            valor_total - distribuido
        } else {
            let proporcao = pot.amount / total_valor_pots;
            let raw = valor_total * proporcao;
            truncar_2_casas(raw)
        };

        distribuido += valor;
        resultado.push(valor);
    }

    resultado
}

/// Calcula a soma total de todos os pots
pub fn soma_total_pots(pots: &[Pot]) -> f64 {
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
///
/// Re-exporta `crate::loss_deflator::mc_error_bound` para que qualquer módulo
/// da arquitetura possa calcular/validar o ruído de estimativas estocásticas
/// (Monte Carlo, amostragem, simulações) com um único ponto de verdade.
///
/// Fórmula (pior caso p = 0.5, margem de 3σ ≈ 99.7% de confiança):
///
///   bound = 3 · 0.5 · √( (1 - f) / samples ),   f = samples / max_boards
///
/// Retorna 0.0 quando a população toda é coberta (estimativa exata).
pub fn mc_error_bound(samples: u64, max_boards: u64) -> f64 {
    crate::loss_deflator::mc_error_bound(samples, max_boards)
}

/// Divide um valor de pote igualmente entre N vencedores empatados (split pot).
/// Aplica a regra oficial do Poker Internacional (WSOP / TDA Regra 68):
///   1. Trunca o valor base de cada jogador para 2 casas decimais.
///   2. Os centavos remanescentes indivisíveis (resto R$ 0,01) são atribuídos de 1 em 1
///      aos vencedores empatados na ordem dos assentos a partir do primeiro à esquerda do Botão (Dealer).
pub fn dividir_pote_empatado(
    pote_amount: f64,
    vencedores_ids: &[String],
    ordem_assentos: &[String],
) -> std::collections::HashMap<String, f64> {
    let mut payouts = std::collections::HashMap::new();
    let n = vencedores_ids.len();
    if n == 0 || pote_amount <= 0.0 {
        return payouts;
    }

    let valor_base = truncar_2_casas(pote_amount / n as f64);
    let total_base = truncar_2_casas(valor_base * n as f64);
    let mut centavos_restantes = ((pote_amount - total_base) * 100.0).round() as i32;

    for id in vencedores_ids {
        payouts.insert(id.clone(), valor_base);
    }

    // Atribui o centavo ímpar aos jogadores empatados na ordem dos assentos à esquerda do botão
    for id in ordem_assentos {
        if centavos_restantes <= 0 {
            break;
        }
        if vencedores_ids.contains(id) {
            if let Some(val) = payouts.get_mut(id) {
                *val = truncar_2_casas(*val + 0.01);
                centavos_restantes -= 1;
            }
        }
    }

    payouts
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Pot;

    #[test]
    fn test_dividir_pote_empatado_odd_cent_to_left_of_button() {
        // Pote de 10.05 para 2 jogadores (alice e bob). Assentos: alice, bob
        let vencedores = vec!["alice".to_string(), "bob".to_string()];
        let ordem_assentos = vec!["alice".to_string(), "bob".to_string()];
        let payouts = dividir_pote_empatado(10.05, &vencedores, &ordem_assentos);

        assert_eq!(*payouts.get("alice").unwrap(), 5.03);
        assert_eq!(*payouts.get("bob").unwrap(), 5.02);
        assert_eq!(payouts.get("alice").unwrap() + payouts.get("bob").unwrap(), 10.05);
    }

    #[test]
    fn test_dividir_pote_empatado_tres_jogadores() {
        // Pote de 10.00 para 3 jogadores (p1, p2, p3). 10.00 / 3 = 3.33 + 0.01 de resto
        let vencedores = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let ordem_assentos = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let payouts = dividir_pote_empatado(10.00, &vencedores, &ordem_assentos);

        assert_eq!(*payouts.get("p1").unwrap(), 3.34);
        assert_eq!(*payouts.get("p2").unwrap(), 3.33);
        assert_eq!(*payouts.get("p3").unwrap(), 3.33);
        let total: f64 = payouts.values().sum();
        assert_eq!(truncar_2_casas(total), 10.00);
    }

    #[test]
    fn test_truncar_2_casas() {
        assert_eq!(truncar_2_casas(5.678), 5.67);
        assert_eq!(truncar_2_casas(5.671), 5.67);
        assert_eq!(truncar_2_casas(10.0), 10.0);
        assert_eq!(truncar_2_casas(0.123), 0.12);
        assert_eq!(truncar_2_casas(0.0), 0.0);
    }

    #[test]
    fn test_ratear_proporcional_dois_pots() {
        let pots = vec![
            Pot::new(100.0, vec!["p1".into()]),
            Pot::new(200.0, vec!["p2".into()]),
        ];
        let rateio = ratear_proporcional(&pots, 30.0);
        assert_eq!(rateio.len(), 2);
        assert_eq!(rateio[0], 10.0); // 30 * 100/300
        assert_eq!(rateio[1], 20.0); // 30 - 10
    }

    #[test]
    fn test_ratear_proporcional_tres_pots() {
        let pots = vec![
            Pot::new(100.0, vec!["p1".into()]),
            Pot::new(60.0, vec!["p2".into()]),
            Pot::new(40.0, vec!["p3".into()]),
        ];
        let rateio = ratear_proporcional(&pots, 10.0);
        assert_eq!(rateio[0], 5.0); // 10 * 100/200
        assert_eq!(rateio[1], 3.0); // 10 * 60/200
        assert_eq!(rateio[2], 2.0); // 10 - 5 - 3
    }

    #[test]
    fn test_ratear_proporcional_vazio() {
        let rateio = ratear_proporcional(&[], 100.0);
        assert!(rateio.is_empty());
    }

    #[test]
    fn test_ratear_proporcional_zero() {
        let pots = vec![Pot::new(100.0, vec!["p1".into()])];
        let rateio = ratear_proporcional(&pots, 0.0);
        assert_eq!(rateio, vec![0.0]);
    }

    #[test]
    fn test_soma_total_pots() {
        let pots = vec![
            Pot::new(100.0, vec![]),
            Pot::new(200.0, vec![]),
            Pot::new(150.0, vec![]),
        ];
        assert_eq!(soma_total_pots(&pots), 450.0);
    }

    #[test]
    fn test_pots_elegeiveis() {
        let pots = vec![
            Pot::new(100.0, vec!["p1".into(), "p2".into()]),
            Pot::new(200.0, vec!["p2".into()]),
        ];
        let elegiveis = pots_elegeiveis(&pots, "p1");
        assert_eq!(elegiveis.len(), 1);
        assert_eq!(elegiveis[0].0, 0); // índice 0

        let elegiveis_p2 = pots_elegeiveis(&pots, "p2");
        assert_eq!(elegiveis_p2.len(), 2); // ambos os pots
    }
}
