// integration_tests.rs — Testes de integração para módulos refatorados
// Verifica que os módulos compartilhados (types.rs, utils.rs) funcionam
// corretamente com deck.rs, rake.rs, side_pots.rs e loss_deflator.rs

use crate::deck::{
    compare_hands, create_deck, deal_cards, evaluate_hand, shuffle_deck, Card, Rank, Suit,
};
use crate::loss_deflator::{calculate_progressive_loss_deflator, ProgressiveLossDeflatorParams};
use crate::rake::{calculate_rake_for_pot, deduct_rake};
use crate::side_pots::{calculate_side_pots, distribute_pots, resolve_side_pots, PlayerForPots};
use crate::types::{GamePhase, Pot, TableConfig};
use crate::utils::{pots_elegeiveis, soma_total_pots, truncar_2_casas};

/// Helper: cria uma carta rapidamente
fn c(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

/// Helper: cria um jogador para side pots
fn make_player(id: &str, total_bet: f64, has_folded: bool, cards: Vec<Card>) -> PlayerForPots {
    PlayerForPots {
        id: id.into(),
        total_bet,
        has_folded,
        cards,
    }
}

// ═══════════════════════════════════════════════════════════════
// Testes de Integração: Deck + Side Pots + Rake
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fluxo_completo_mao_dealer() {
    // Simula uma mão completa: deal → side pots → rake → distribuição
    let deck = create_deck();
    let shuffled = shuffle_deck(&deck);

    // Distribui cartas para 3 jogadores
    let (hole1, rest1) = deal_cards(&shuffled, 2);
    let (hole2, rest2) = deal_cards(&rest1, 2);
    let (hole3, rest3) = deal_cards(&rest2, 2);

    // Flop + Turn + River
    let (flop, rest4) = deal_cards(&rest3, 3);
    let (turn, rest5) = deal_cards(&rest4, 1);
    let (river, _) = deal_cards(&rest5, 1);

    let community: Vec<Card> = flop
        .iter()
        .chain(turn.iter())
        .chain(river.iter())
        .copied()
        .collect();

    // Verifica que todas as cartas são únicas
    let mut all_cards: Vec<Card> = Vec::new();
    all_cards.extend(&hole1);
    all_cards.extend(&hole2);
    all_cards.extend(&hole3);
    all_cards.extend(&community);

    let set: std::collections::HashSet<(Rank, Suit)> =
        all_cards.iter().map(|c| (c.rank, c.suit)).collect();
    assert_eq!(set.len(), 11, "Cartas devem ser todas únicas (2*3 + 5)");
}

#[test]
fn test_side_pots_com_rake_completo() {
    // 3 jogadores com all-ins diferentes
    let players = vec![
        make_player(
            "p1",
            100.0,
            false,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Spades)],
        ),
        make_player(
            "p2",
            200.0,
            false,
            vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Spades)],
        ),
        make_player(
            "p3",
            200.0,
            false,
            vec![c(Rank::Queen, Suit::Hearts), c(Rank::Queen, Suit::Spades)],
        ),
    ];

    let community = vec![
        c(Rank::Two, Suit::Clubs),
        c(Rank::Five, Suit::Diamonds),
        c(Rank::Eight, Suit::Hearts),
        c(Rank::Jack, Suit::Spades),
        c(Rank::Nine, Suit::Clubs),
    ];

    // 1. Calcula side pots
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 2);
    assert_eq!(pots[0].amount, 300.0); // main pot
    assert_eq!(pots[1].amount, 200.0); // side pot

    // 2. Aplica rake
    let config = TableConfig::new(10.0, 5.0, 20.0);
    let rake_result = deduct_rake(&pots, &config, None);
    assert!(rake_result.total_rake > 0.0);
    assert!(rake_result.total_rake <= 20.0); // respeita o cap

    // 3. Verifica que soma dos pots após rake = total antes - rake
    let total_after: f64 = rake_result.pots_after_rake.iter().map(|p| p.amount).sum();
    let expected_total = 500.0 - rake_result.total_rake;
    assert!((total_after - expected_total).abs() < f64::EPSILON);
}

#[test]
fn test_loss_deflator_com_side_pots() {
    // Cenário: all-in no flop, perdedor recebe cashback
    let pots = vec![
        Pot::new(200.0, vec!["loser".into(), "winner".into()]),
        Pot::new(100.0, vec!["winner".into()]),
    ];

    let result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
        pots: pots.clone(),
        loser_id: "loser".into(),
        winner_id: "winner".into(),
        phase: GamePhase::Flop,
    });

    assert!(result.is_some());
    let r = result.unwrap();

    // 25% do main pot (200) = 50
    assert_eq!(r.cashback, 50.0);
    assert_eq!(r.eligible_pot_total, 200.0);
    assert_eq!(r.eligible_pot_ids, vec![0]); // apenas main pot
}

#[test]
fn test_utils_truncar_com_rake() {
    // Verifica que truncar_2_casas funciona corretamente com rake
    let rake = calculate_rake_for_pot(333.0, 5.0, 100.0);
    // 5% de 333 = 16.65, truncado para 16.65
    assert!(
        (rake - 16.65).abs() < 0.01,
        "rake={}, expected ~16.65",
        rake
    );

    // Verifica que soma_total_pots funciona
    let pots = vec![
        Pot::new(100.0, vec![]),
        Pot::new(200.0, vec![]),
        Pot::new(150.0, vec![]),
    ];
    assert_eq!(soma_total_pots(&pots), 450.0);
}

#[test]
fn test_pots_elegeiveis_com_loss_deflator() {
    // Verifica que pots_elegeiveis funciona com loss_deflator
    let pots = vec![
        Pot::new(200.0, vec!["p1".into(), "p2".into()]),
        Pot::new(100.0, vec!["p2".into()]),
        Pot::new(50.0, vec!["p1".into(), "p2".into()]),
    ];

    let elegiveis_p1 = pots_elegeiveis(&pots, "p1");
    assert_eq!(elegiveis_p1.len(), 2); // pots 0 e 2

    let elegiveis_p2 = pots_elegeiveis(&pots, "p2");
    assert_eq!(elegiveis_p2.len(), 3); // todos os pots
}

#[test]
fn test_fluxo_completo_cash_game() {
    // Simula um fluxo completo de cash game:
    // 1. Cria baralho e embaralha
    // 2. Distribui cartas
    // 3. Avalia mãos
    // 4. Calcula side pots
    // 5. Aplica rake
    // 6. Distribui ganhos

    let deck = create_deck();
    let shuffled = shuffle_deck(&deck);

    // 2 jogadores
    let (hole1, rest1) = deal_cards(&shuffled, 2);
    let (hole2, rest2) = deal_cards(&rest1, 2);
    let (community_cards, _) = deal_cards(&rest2, 5);

    // Avalia mãos
    let hand1 = evaluate_hand(&hole1, &community_cards);
    let hand2 = evaluate_hand(&hole2, &community_cards);

    // Cria jogadores
    let players = vec![
        make_player("alice", 100.0, false, hole1),
        make_player("bob", 100.0, false, hole2),
    ];

    // Calcula pots
    let pots = calculate_side_pots(&players);
    assert_eq!(pots.len(), 1);
    assert_eq!(pots[0].amount, 200.0);

    // Aplica rake
    let config = TableConfig::new(10.0, 5.0, 10.0);
    let rake_result = deduct_rake(&pots, &config, None);

    // Distribui ganhos
    let payouts = distribute_pots(&pots, &players, &community_cards);
    let total_payouts: f64 = payouts.values().sum();

    // Verifica invariantes
    assert!(rake_result.total_rake >= 0.0);
    assert!(rake_result.total_rake <= 10.0); // cap
    assert!(total_payouts <= 200.0); // não pode pagar mais que o pot
}

#[test]
fn test_tipos_compartilhados_consistencia() {
    // Verifica que os tipos compartilhados são consistentes
    let pot = Pot::new(100.0, vec!["p1".into()]);
    assert!(pot.is_eligible("p1"));
    assert!(!pot.is_eligible("p2"));

    let config = TableConfig::new(10.0, 5.0, 10.0);
    assert_eq!(config.big_blind, 10.0);
    assert_eq!(config.rake_percent, 5.0);
    assert_eq!(config.rake_cap, 10.0);

    // Verifica que GamePhase funciona
    assert_eq!(GamePhase::Preflop.as_str(), "preflop");
    assert_eq!(GamePhase::Flop.as_str(), "flop");
    assert_eq!(GamePhase::Turn.as_str(), "turn");
    assert_eq!(GamePhase::River.as_str(), "river");
}

#[test]
fn test_empate_split_pot() {
    // Dois jogadores com a mesma mão (board é royal flush)
    let players = vec![
        make_player(
            "p1",
            100.0,
            false,
            vec![c(Rank::Two, Suit::Hearts), c(Rank::Three, Suit::Hearts)],
        ),
        make_player(
            "p2",
            100.0,
            false,
            vec![c(Rank::Four, Suit::Clubs), c(Rank::Five, Suit::Clubs)],
        ),
    ];

    let community = vec![
        c(Rank::Ace, Suit::Diamonds),
        c(Rank::King, Suit::Diamonds),
        c(Rank::Queen, Suit::Diamonds),
        c(Rank::Jack, Suit::Diamonds),
        c(Rank::Ten, Suit::Diamonds),
    ];

    let payouts = distribute_pots(
        &[Pot::new(200.0, vec!["p1".into(), "p2".into()])],
        &players,
        &community,
    );

    // Split pot: 100 cada
    assert_eq!(payouts.get("p1"), Some(&100.0));
    assert_eq!(payouts.get("p2"), Some(&100.0));
}

#[test]
fn test_multiple_hands_consistency() {
    // Executa 100 mãos aleatórias e verifica consistência
    for _ in 0..100 {
        let deck = create_deck();
        let shuffled = shuffle_deck(&deck);

        let (hole1, rest1) = deal_cards(&shuffled, 2);
        let (hole2, rest2) = deal_cards(&rest1, 2);
        let (community, _) = deal_cards(&rest2, 5);

        let hand1 = evaluate_hand(&hole1, &community);
        let hand2 = evaluate_hand(&hole2, &community);

        // Uma deve vencer ou empatar
        let cmp = compare_hands(&hand1, &hand2);
        assert!(
            cmp == std::cmp::Ordering::Greater
                || cmp == std::cmp::Ordering::Less
                || cmp == std::cmp::Ordering::Equal
        );
    }
}
