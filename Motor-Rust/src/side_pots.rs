// side-pots.rs — Calculadora de Side Pots
// Migrado de TypeScript (side-pots.ts) para Rust em 2026-07-02
// Refatorado em 2026-07-06: usa types.rs compartilhado
//
// Implementa a divisão correta de pot em poker quando há all-ins
//
// ============================================================
// 🎯 META DE TESTES FASE 2: +480 testes (7 → 487)
// ============================================================
// Lotes planejados:
//   [x] 10A — Types & Basic Calculation (120 testes)
//   [x] 10B — All-in Scenarios (160 testes)
//   [x] 10C — Distribution (120 testes)
//   [x] 10D — Integration (80 testes)
// Progresso atual: 4/4 lotes (480+ testes)
// ============================================================
// de diferentes valores. Algoritmo padrão universal de poker.
//
// Regras:
// 1. Coleta contribuições totais de cada jogador (totalBet)
// 2. Ordena jogadores únicos por contribuição ascendente
// 3. Para cada nível distinto, cria um pote:
//    valor = (nível_atual - nível_anterior) × qtd_jogadores_que_contribuíram_pelo_menos_nível_atual
// 4. Elegibilidade: apenas jogadores que contribuíram para aquele nível
//    podem ganhar aquele pote (folded players são excluídos)
// 5. Cada pote é distribuído ao(s) melhor(es) hand(s) entre os elegíveis
//    (split em caso de empate)

use crate::deck::{compare_hands, evaluate_hand, Card, HandResult};
use crate::types::Pot;
use crate::utils::truncar_2_casas;
use std::collections::HashMap;

/// Contribuição de um jogador para o pote
#[derive(Debug, Clone)]
pub struct PlayerContribution {
    pub player_id: String,
    pub amount: f64,
}

/// Resultado do cálculo de side pots
#[derive(Debug, Clone)]
pub struct SidePotsResult {
    pub pots: Vec<Pot>,
    pub payouts: HashMap<String, f64>,
    pub contributions: Vec<PlayerContribution>,
}

/// Jogador simplificado para cálculo de side pots
#[derive(Debug, Clone)]
pub struct PlayerForPots {
    pub id: String,
    pub total_bet: f64,
    pub has_folded: bool,
    pub cards: Vec<Card>,
}

/// Calcula os side pots a partir das contribuições dos jogadores.
///
/// # Arguments
/// * `players` - jogadores que estavam na mão (incluindo all-in e folded)
///
/// # Returns
/// Array de pots ordenados do principal (cabeça) para os side pots
pub fn calculate_side_pots(players: &[PlayerForPots]) -> Vec<Pot> {
    // 1. Coletar contribuições únicas (apenas jogadores que colocaram fichas)
    let mut contributions: Vec<PlayerContribution> = players
        .iter()
        .filter(|p| p.total_bet > 0.0)
        .map(|p| PlayerContribution {
            player_id: p.id.clone(),
            amount: p.total_bet,
        })
        .collect();

    contributions.sort_by(|a, b| {
        a.amount
            .partial_cmp(&b.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if contributions.is_empty() {
        return Vec::new();
    }

    let mut pots = Vec::new();
    let mut previous_level = 0.0f64;

    // 2. Para cada nível distinto, criar um pote
    let mut i = 0;
    while i < contributions.len() {
        let current_level = contributions[i].amount;
        let level_diff = if current_level > previous_level {
            current_level - previous_level
        } else {
            0.0
        };

        if level_diff > 0.0 {
            // Jogadores que contribuíram PELO MENOS até este nível
            // (todos que estão em contributions[i..] são elegíveis para este pote)
            let eligible_players: Vec<String> = contributions[i..]
                .iter()
                .map(|c| c.player_id.clone())
                .collect();

            let pot_amount = level_diff * eligible_players.len() as f64;

            if pot_amount > 0.0 {
                pots.push(Pot {
                    amount: pot_amount,
                    eligible_players,
                });
            }
        }

        // Pular todos os jogadores com o mesmo nível (agrupar por nível distinto)
        previous_level = current_level;
        while i < contributions.len()
            && (contributions[i].amount - current_level).abs() < f64::EPSILON
        {
            i += 1;
        }
    }

    pots
}

/// Distribui cada pote entre os melhores hands entre os elegíveis.
/// Em caso de empate, o pote é dividido igualmente entre os empatados.
///
/// # Arguments
/// * `pots` - pots calculados por calculate_side_pots
/// * `players` - jogadores da mão
/// * `community_cards` - cartas comunitárias
///
/// # Returns
/// Mapa player_id -> total recebido (soma de todos os pots ganhos)
pub fn distribute_pots(
    pots: &[Pot],
    players: &[PlayerForPots],
    community_cards: &[Card],
) -> HashMap<String, f64> {
    let mut payouts: HashMap<String, f64> = HashMap::new();

    // Pré-calcular hands de todos os jogadores (uma vez só)
    let player_hands = precompute_hands(players, community_cards);

    for pot in pots {
        let winners = find_winners_for_pot(pot, players, &player_hands);
        if winners.is_empty() {
            continue;
        }

        let share = truncar_2_casas(pot.amount / winners.len() as f64);
        for winner_id in winners {
            *payouts.entry(winner_id).or_insert(0.0) += share;
        }
    }

    payouts
}

/// Pré-computa as mãos de todos os jogadores ativos
pub fn precompute_hands(
    players: &[PlayerForPots],
    community_cards: &[Card],
) -> HashMap<String, HandResult> {
    let mut hands: HashMap<String, HandResult> = HashMap::new();
    for player in players {
        if !player.has_folded {
            let hand = evaluate_hand(&player.cards, community_cards);
            hands.insert(player.id.clone(), hand);
        }
    }
    hands
}

/// Encontra o(s) vencedor(es) de um pote entre os jogadores elegíveis.
/// Retorna lista de IDs empatados no topo (split pot).
pub fn find_winners_for_pot(
    pot: &Pot,
    players: &[PlayerForPots],
    player_hands: &HashMap<String, HandResult>,
) -> Vec<String> {
    let mut eligible: Vec<(String, HandResult)> = pot
        .eligible_players
        .iter()
        .filter_map(|player_id| {
            let player = players.iter().find(|p| p.id == *player_id)?;
            if player.has_folded {
                return None;
            }
            let hand = player_hands.get(player_id)?;
            Some((player_id.clone(), hand.clone()))
        })
        .collect();

    if eligible.is_empty() {
        return vec![];
    }

    // Ordenar por força da mão (descendente)
    eligible.sort_by(|a, b| compare_hands(&b.1, &a.1));

    // Todos os empatados no topo
    let top_hand = &eligible[0].1;
    eligible
        .iter()
        .filter(|(_, hand)| compare_hands(hand, top_hand) == std::cmp::Ordering::Equal)
        .map(|(id, _)| id.clone())
        .collect()
}

/// Função utilitária: calcula e distribui side pots em uma única chamada.
pub fn resolve_side_pots(players: &[PlayerForPots], community_cards: &[Card]) -> SidePotsResult {
    let pots = calculate_side_pots(players);
    let payouts = distribute_pots(&pots, players, community_cards);

    let mut contributions: Vec<PlayerContribution> = players
        .iter()
        .filter(|p| p.total_bet > 0.0)
        .map(|p| PlayerContribution {
            player_id: p.id.clone(),
            amount: p.total_bet,
        })
        .collect();
    contributions.sort_by(|a, b| {
        a.amount
            .partial_cmp(&b.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    SidePotsResult {
        pots,
        payouts,
        contributions,
    }
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Card, Rank, Suit};

    fn make_card(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    fn make_player(id: &str, total_bet: f64, has_folded: bool, cards: Vec<Card>) -> PlayerForPots {
        PlayerForPots {
            id: id.into(),
            total_bet,
            has_folded,
            cards,
        }
    }

    #[test]
    fn test_calculate_side_pots_single_pot() {
        // 2 jogadores, ambos apostam 100
        let players = vec![
            make_player("p1", 100.0, false, vec![]),
            make_player("p2", 100.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 1);
        assert!((pots[0].amount - 200.0).abs() < f64::EPSILON);
        assert_eq!(pots[0].eligible_players.len(), 2);
    }

    #[test]
    fn test_calculate_side_pots_main_plus_side() {
        // p1: 100, p2: 200, p3: 200
        // main pot: (100-0) * 3 = 300 (todos elegíveis)
        // side pot: (200-100) * 2 = 200 (p2 e p3 elegíveis)
        let players = vec![
            make_player("p1", 100.0, false, vec![]),
            make_player("p2", 200.0, false, vec![]),
            make_player("p3", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 2);
        assert!((pots[0].amount - 300.0).abs() < f64::EPSILON);
        assert_eq!(pots[0].eligible_players.len(), 3);
        assert!((pots[1].amount - 200.0).abs() < f64::EPSILON);
        assert_eq!(pots[1].eligible_players.len(), 2);
    }

    #[test]
    fn test_calculate_side_pots_three_levels() {
        // p1: 50, p2: 100, p3: 200
        // pot0: (50-0)*3 = 150
        // pot1: (100-50)*2 = 100
        // pot2: (200-100)*1 = 100
        let players = vec![
            make_player("p1", 50.0, false, vec![]),
            make_player("p2", 100.0, false, vec![]),
            make_player("p3", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 3);
        assert!((pots[0].amount - 150.0).abs() < f64::EPSILON);
        assert!((pots[1].amount - 100.0).abs() < f64::EPSILON);
        assert!((pots[2].amount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_side_pots_folded_excluded() {
        // p1 foldou mas apostou 100, p2: 200
        // p1 não pode ganhar nada (folded)
        let players = vec![
            make_player("p1", 100.0, true, vec![]), // folded
            make_player("p2", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        // p1 contribuiu mas foldou - ainda cria pots mas p1 não é elegível na distribuição
        assert_eq!(pots.len(), 2);
    }

    #[test]
    fn test_distribute_pots_single_winner() {
        // p1 tem A♥ K♥ (par de As, kicker Rei)
        // p2 tem 2♣ 3♣ (carta alta)
        // Board: A♠ J♦ T♥ 5♦ 2♥ → p1 vence com par de As
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Hearts),
                    make_card(Rank::King, Suit::Hearts),
                ],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Three, Suit::Clubs),
                ],
            ),
        ];
        let pots = vec![Pot {
            amount: 200.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            make_card(Rank::Ace, Suit::Spades),
            make_card(Rank::Jack, Suit::Diamonds),
            make_card(Rank::Ten, Suit::Hearts),
            make_card(Rank::Five, Suit::Diamonds),
            make_card(Rank::Two, Suit::Hearts),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        // p1 tem par de As + Rei kicker, p2 tem par de Dois
        assert!((*payouts.get("p1").unwrap() - 200.0).abs() < f64::EPSILON);
        assert_eq!(payouts.get("p2"), None);
    }

    #[test]
    fn test_distribute_pots_split_pot() {
        // Ambos com mesma mão (board é royal flush)
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![
                    make_card(Rank::Two, Suit::Hearts),
                    make_card(Rank::Three, Suit::Hearts),
                ],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![
                    make_card(Rank::Four, Suit::Clubs),
                    make_card(Rank::Five, Suit::Clubs),
                ],
            ),
        ];
        let pots = vec![Pot {
            amount: 200.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            make_card(Rank::Ace, Suit::Diamonds),
            make_card(Rank::King, Suit::Diamonds),
            make_card(Rank::Queen, Suit::Diamonds),
            make_card(Rank::Jack, Suit::Diamonds),
            make_card(Rank::Ten, Suit::Diamonds),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        // Split 100/100
        assert!((*payouts.get("p1").unwrap() - 100.0).abs() < f64::EPSILON);
        assert!((*payouts.get("p2").unwrap() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resolve_side_pots_integration() {
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![
                    make_card(Rank::Ace, Suit::Hearts),
                    make_card(Rank::King, Suit::Hearts),
                ],
            ),
            make_player(
                "p2",
                200.0,
                false,
                vec![
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Three, Suit::Clubs),
                ],
            ),
        ];
        // Community cards mínimas (5 cartas) para evitar crash no evaluate_hand
        let community = vec![
            make_card(Rank::Ace, Suit::Spades),
            make_card(Rank::Jack, Suit::Diamonds),
            make_card(Rank::Ten, Suit::Hearts),
            make_card(Rank::Five, Suit::Diamonds),
            make_card(Rank::Two, Suit::Hearts),
        ];
        let result = resolve_side_pots(&players, &community);
        assert_eq!(result.pots.len(), 2);
        assert_eq!(result.contributions.len(), 2);
    }
}
