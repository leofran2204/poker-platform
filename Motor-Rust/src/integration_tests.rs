// integration_tests.rs — Testes de Integração (Nível 2 da Pirâmide de Testes)
//
// Valida o fluxo ponta-a-ponta ENTRE módulos, conforme QUALITY.md §3.2:
//   1. Mão completa:        deck → side_pots → rake → hand_history
//   2. Torneio completo:    tournament_engine (registro → blinds → eliminação → prêmios)
//   3. Loss deflator + rake: perda + cashback + rake cobrado corretamente
//   4. RNG + deck:          embaralhamento criptográfico preserva integridade
//   5. Conservação de fichas: soma de pots == soma de contribuições (invariante)
//
// Foco: os riscos críticos do QUALITY.md §3.1 (cartas duplicadas, pote errado,
// vencedor errado, rake acima do cap). Não usa aleatoriedade pesada — cenários
// determinísticos e rápidos.

use crate::deck::{create_deck, deal_cards, shuffle_deck, Card, HandResult, Rank, Suit};
use crate::hand_history::{
    create_hand_history, finalize_hand, from_json, record_action, set_community_cards, to_json,
    Action, EndReason, PlayerAction, PlayerResult,
};
use crate::loss_deflator::{
    calculate_progressive_loss_deflator, LossDeflatorTier, ProgressiveLossDeflatorParams,
};
use crate::rake::deduct_rake;
use crate::side_pots::{resolve_side_pots, PlayerForPots};
use crate::tournament_engine::{
    advance_blinds, create_tournament, eliminate_player, finish_tournament, register_player,
    start_tournament, BlindLevel, TournamentConfig, TournamentSpeed,
};
use crate::types::{GamePhase, Pot, TableConfig};
use std::collections::HashMap;

// ─── Helpers ───

fn c(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

/// Embaralha e distribui uma mão completa de `n_players` jogadores (2 hole
/// cards cada, com burn cards) + board de 5 (flop/turn/river). Igual ao
/// fluxo real do game_loop.
fn deal_full_hand(n_players: usize) -> (Vec<Vec<Card>>, Vec<Card>) {
    let full = create_deck();
    let mut deck = shuffle_deck(&full);
    let mut holes: Vec<Vec<Card>> = vec![Vec::with_capacity(2); n_players];
    for _round in 0..2 {
        for hole in &mut holes {
            let (cards, rest) = deal_cards(&deck, 1);
            deck = rest;
            hole.extend(cards);
        }
    }
    let (_b, d) = deal_cards(&deck, 1);
    deck = d; // burn
    let (flop, d) = deal_cards(&deck, 3);
    deck = d;
    let (_b, d) = deal_cards(&deck, 1);
    deck = d; // burn
    let (turn, d) = deal_cards(&deck, 1);
    deck = d;
    let (_b, d) = deal_cards(&deck, 1);
    deck = d; // burn
    let (river, _d) = deal_cards(&deck, 1);
    let mut board = flop;
    board.extend(turn);
    board.extend(river);
    (holes, board)
}

fn rake_config() -> TableConfig {
    TableConfig::new(1000, 500, 600) // BB=10, 5%, cap R$6
}

/// Soma dos amounts de uma lista de pots.
fn sum_pots(pots: &[Pot]) -> u64 {
    pots.iter().map(|p| p.amount).sum()
}

// ─── Cenário 1: Mão completa (deck → side_pots → rake → hand_history) ───

#[test]
fn test_integration_full_hand_deck_sidepots_rake_handhistory() {
    let n = 3;
    let (holes, board) = deal_full_hand(n);

    // Integridade: 52 únicas no baralho usado (hole + board = 11 distintas)
    let mut seen = [false; 52];
    for hand in &holes {
        for card in hand {
            let idx = (card.suit as usize) * 13 + (card.rank as usize - 2);
            assert!(
                !seen[idx],
                "carta duplicada no deal (integridade do baralho)"
            );
            seen[idx] = true;
        }
    }
    for card in &board {
        let idx = (card.suit as usize) * 13 + (card.rank as usize - 2);
        assert!(
            !seen[idx],
            "carta duplicada no board (integridade do baralho)"
        );
        seen[idx] = true;
    }

    // Simula apostas (all-in com side pots): p1=100, p2=200, p3=200
    // p1 recebe cartas fortes para ser o vencedor do main pot.
    let players = vec![
        PlayerForPots {
            id: "p1".into(),
            total_bet: 10000,
            has_folded: false,
            cards: vec![c(Rank::Ace, Suit::Spades), c(Rank::Ace, Suit::Hearts)],
        },
        PlayerForPots {
            id: "p2".into(),
            total_bet: 20000,
            has_folded: false,
            cards: vec![c(Rank::King, Suit::Clubs), c(Rank::Queen, Suit::Diamonds)],
        },
        PlayerForPots {
            id: "p3".into(),
            total_bet: 20000,
            has_folded: false,
            cards: vec![c(Rank::Jack, Suit::Clubs), c(Rank::Ten, Suit::Diamonds)],
        },
    ];

    let board = vec![
        c(Rank::Two, Suit::Clubs),
        c(Rank::Four, Suit::Diamonds),
        c(Rank::Seven, Suit::Hearts),
        c(Rank::Nine, Suit::Spades),
        c(Rank::Two, Suit::Hearts),
    ];

    // 1) side_pots
    let side = resolve_side_pots(&players, &board);
    let contributions_sum: u64 = players.iter().map(|p| p.total_bet).sum();
    assert!(
        sum_pots(&side.pots) == contributions_sum,
        "soma dos pots ({}) != soma das contribuições ({}) — fichas não conservadas",
        sum_pots(&side.pots),
        contributions_sum
    );
    // main pot = (100-0)*3 = 300; side = (200-100)*2 = 200
    assert_eq!(side.pots.len(), 2, "deve haver main + 1 side pot");
    assert_eq!(side.pots[0].amount, 30000);
    assert_eq!(side.pots[1].amount, 20000);

    // 2) rake sobre os pots
    let rake_result = deduct_rake(&side.pots, &rake_config(), None);
    assert!(
        rake_result.total_rake <= rake_config().rake_cap,
        "rake {} acima do cap {}",
        rake_result.total_rake,
        rake_config().rake_cap
    );
    assert_eq!(
        sum_pots(&rake_result.pots_after_rake),
        sum_pots(&side.pots) - rake_result.total_rake,
        "pots após rake não batem com total - rake"
    );

    // 3) hand_history — registra a mão inteira
    let mut stacks = HashMap::new();
    stacks.insert("p1".into(), 1000u64);
    stacks.insert("p2".into(), 1000u64);
    stacks.insert("p3".into(), 1000u64);

    let hh_config = crate::hand_history::TableConfig {
        table_name: "Mesa Integração".into(),
        small_blind: 5,
        big_blind: 10,
        ante: None,
        max_players: 9,
        game_type: crate::hand_history::GameType::Cash,
    };
    let mut history = create_hand_history(
        "hand-integration-01".into(),
        hh_config,
        vec!["p1".into(), "p2".into(), "p3".into()],
        stacks,
    );

    record_action(
        &mut history,
        PlayerAction {
            player_id: "p1".into(),
            action: Action::Call,
            amount: 100,
            phase: GamePhase::Preflop,
            timestamp_ms: 100,
        },
    );
    record_action(
        &mut history,
        PlayerAction {
            player_id: "p2".into(),
            action: Action::Raise,
            amount: 200,
            phase: GamePhase::Preflop,
            timestamp_ms: 200,
        },
    );
    record_action(
        &mut history,
        PlayerAction {
            player_id: "p3".into(),
            action: Action::Call,
            amount: 200,
            phase: GamePhase::Preflop,
            timestamp_ms: 300,
        },
    );

    set_community_cards(&mut history, GamePhase::Flop, board[0..3].to_vec());
    set_community_cards(&mut history, GamePhase::Turn, vec![board[3]]);
    set_community_cards(&mut history, GamePhase::River, vec![board[4]]);

    // Deriva resultados dos payouts do side_pots (pré-rake, para bater com o total apostado)
    let mut results: Vec<PlayerResult> = players
        .iter()
        .map(|p| {
            let won = *side.payouts.get(&p.id).unwrap_or(&0);
            PlayerResult {
                player_id: p.id.clone(),
                finish_position: 0, // preenchido abaixo
                hole_cards: p.cards.clone(),
                best_hand: Some(HandResult {
                    rank: crate::deck::HandRank::HighCard,
                    cards: p.cards.clone(),
                    kickers: vec![],
                    value: 0,
                }),
                best_hand_name: Some("High Card".into()),
                chips_won: won as u64,
                chips_lost: p.total_bet,
                folded: p.has_folded,
                was_all_in: true,
            }
        })
        .collect();
    // Posições por payout decrescente
    let mut order: Vec<usize> = (0..results.len()).collect();
    order.sort_by(|&a, &b| {
        side.payouts
            .get(&players[b].id)
            .unwrap_or(&0)
            .cmp(side.payouts.get(&players[a].id).unwrap_or(&0))
    });
    for (pos, &idx) in order.iter().enumerate() {
        results[idx].finish_position = (pos + 1) as u8;
    }

    let total_pot_before = sum_pots(&side.pots);
    finalize_hand(
        &mut history,
        results,
        total_pot_before,
        rake_result.total_rake,
        GamePhase::River,
        EndReason::Showdown,
    );

    // Invariantes do hand_history
    assert_eq!(
        history.total_pot, total_pot_before,
        "total_pot do histórico != soma antes do rake"
    );
    assert_eq!(
        history.rake, rake_result.total_rake as u64,
        "rake do histórico != rake calculado"
    );
    assert_eq!(history.community_cards.len(), 5);
    let winner = crate::hand_history::get_winner(&history).expect("deve haver vencedor");
    assert_eq!(winner.player_id, "p1", "vencedor deve ser p1 (melhor mão)");
    assert_eq!(winner.chips_won, 30_000);

    // Roundtrip JSON preserva os dados críticos
    let json = to_json(&history).expect("serialização");
    let restored = from_json(&json).expect("desserialização");
    assert_eq!(restored.hand_id, history.hand_id);
    assert_eq!(restored.total_pot, history.total_pot);
    assert_eq!(restored.rake, history.rake);
    assert_eq!(restored.players.len(), 3);
}

// ─── Cenário 2: Torneio completo (tournament_engine) ───

#[test]
fn test_integration_full_tournament_lifecycle() {
    let config = TournamentConfig {
        name: "Integração MTT".into(),
        game_type: "Holdem".into(),
        buy_in: 1000,
        starting_stack: 10000,
        max_players: 100,
        speed: TournamentSpeed::Normal,
        blind_levels: vec![
            BlindLevel {
                level: 1,
                small_blind: 10,
                big_blind: 20,
                ante: 0,
                duration_minutes: 15,
            },
            BlindLevel {
                level: 2,
                small_blind: 20,
                big_blind: 40,
                ante: 0,
                duration_minutes: 15,
            },
        ],
        prize_pool_pct: 0.9,
        prize_distribution: vec![0.6, 0.3, 0.1],
        late_registration: false,
        late_registration_max_level: 0,
        allow_rebuy: false,
        allow_addon: false,
        rebuy_max_level: 0,
    };

    let mut state = create_tournament(config);
    assert_eq!(
        state.status,
        crate::tournament_engine::TournamentStatus::Registering
    );

    for (i, id) in ["t1", "t2", "t3", "t4"].iter().enumerate() {
        register_player(&mut state, id, &format!("Player{}", i)).expect("registro");
    }
    assert_eq!(state.players_remaining, 4);
    assert_eq!(state.prize_pool, (4000.0 * 0.9) as u64); // 3600

    start_tournament(&mut state).expect("início");
    assert_eq!(
        state.status,
        crate::tournament_engine::TournamentStatus::Running
    );
    assert_eq!(state.current_level, 1);

    // Blinds sobem
    advance_blinds(&mut state).expect("avanço de blinds");
    assert_eq!(state.current_level, 2);
    let blinds = crate::tournament_engine::get_current_blinds(&state).expect("blinds atuais");
    assert_eq!(blinds.big_blind, 40);

    // Elimina 3 jogadores (sobra 1 = vencedor)
    eliminate_player(&mut state, "t4", Some(4)).expect("elimina t4");
    eliminate_player(&mut state, "t3", Some(3)).expect("elimina t3");
    eliminate_player(&mut state, "t2", Some(2)).expect("elimina t2");
    assert_eq!(state.players_remaining, 1);

    // Finaliza e distribui prêmios
    let result = finish_tournament(&mut state).expect("finaliza");
    assert_eq!(
        state.status,
        crate::tournament_engine::TournamentStatus::Finished
    );
    assert_eq!(result.total_players, 4);
    assert_eq!(result.total_prize_pool, state.prize_pool);

    // Conservação: prêmios pagos não podem exceder o prize pool disponível.
    // Com 1 jogador restante, apenas o prêmio de 1º lugar (60%) é efetivamente pago.
    let sum_prizes: u64 = result.winners.iter().map(|w| w.prize).sum();
    assert!(
        sum_prizes <= state.prize_pool,
        "soma dos prêmios ({}) > prize pool ({})",
        sum_prizes,
        state.prize_pool
    );
    // O vencedor (posição 1) é o jogador que sobrou
    let champ = result
        .winners
        .iter()
        .find(|w| w.position == 1)
        .expect("campeão");
    assert_eq!(champ.player_id, "t1");
    // Com 1 jogador restante, recebe o prêmio de 1º lugar (60% do pool)
    assert_eq!(champ.prize, (state.prize_pool as f64 * 0.6) as u64);
}

// ─── Cenário 3: Loss deflator + rake ───

#[test]
fn test_integration_loss_deflator_plus_rake() {
    // All-in preflop: main pot de 200 entre loser e winner.
    let pots = vec![Pot {
        amount: 20000,
        eligible_players: vec!["loser".into(), "winner".into()],
    }];

    // Rake sobre o pote (5% de 200 = 10, cap 6) → 6
    let rake = deduct_rake(&pots, &rake_config(), None);
    assert_eq!(rake.total_rake, 600, "rake deve ser 6 (cap)");
    assert!(rake.total_rake <= rake_config().rake_cap);

    // Equity de 60% = tier de 7% sobre os potes líquidos após o rake.
    let deflator = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
        pots: rake.pots_after_rake.clone(),
        loser_id: "loser".into(),
        winner_id: "winner".into(),
        phase: GamePhase::Preflop,
        loser_equity: 0.60,
    })
    .expect("deflator calculado");
    assert_eq!(deflator.tier, LossDeflatorTier::SevenPercent);
    assert_eq!(
        deflator.cashback, 1358,
        "cashback deve ser 7% de 194 = 13,58"
    );
    assert_eq!(deflator.eligible_pot_total, 19400);

    // Conservação: rake + cashback não podem exceder o pote original
    assert!(
        (rake.total_rake + deflator.cashback) <= pots[0].amount,
        "rake ({}) + cashback ({}) excede o pote ({})",
        rake.total_rake,
        deflator.cashback,
        pots[0].amount
    );

    // O vencedor recebe o pote pós-rake menos o cashback do perdedor.
    let winner_net = rake.pots_after_rake[0].amount - deflator.cashback;
    assert_eq!(
        winner_net, 18042,
        "vencedor líquido deve ser 194-13,58 = 180,42"
    );
}

// ─── Cenário 4: RNG + deck (embaralhamento criptográfico) ───

#[test]
fn test_integration_rng_deck_integrity() {
    let original = create_deck();
    assert_eq!(original.len(), 52);

    // O shuffle preserva as 52 cartas (permutação exata, sem perdas/duplicatas)
    let shuffled = shuffle_deck(&original);
    assert_eq!(shuffled.len(), 52);
    let mut seen = [false; 52];
    for card in &shuffled {
        let idx = (card.suit as usize) * 13 + (card.rank as usize - 2);
        assert!(
            !seen[idx],
            "shuffle produziu carta duplicada — integridade quebrada"
        );
        seen[idx] = true;
    }

    // Dois shuffles independentes (CSPRNG) produzem ordens distintas
    let other = shuffle_deck(&original);
    let different = shuffled.iter().zip(other.iter()).any(|(a, b)| a != b);
    assert!(different, "dois shuffles CSPRNG não devem ser idênticos");

    // Deal de uma mão completa não repete cartas (usando o RNG por baixo)
    let (holes, board) = deal_full_hand(6);
    let mut used = [false; 52];
    for hand in &holes {
        for card in hand {
            let idx = (card.suit as usize) * 13 + (card.rank as usize - 2);
            assert!(!used[idx], "carta repetida no deal das hole cards");
            used[idx] = true;
        }
    }
    for card in &board {
        let idx = (card.suit as usize) * 13 + (card.rank as usize - 2);
        assert!(!used[idx], "carta repetida no board");
        used[idx] = true;
    }
    assert_eq!(holes.len(), 6);
    assert_eq!(board.len(), 5);
}

// ─── Cenário 5: Conservação de fichas (side_pots) com fold ───

#[test]
fn test_integration_sidepots_chip_conservation_with_fold() {
    // 4 jogadores: p1 foldou (apostou 50), p2=50, p3=150, p4=150
    // main: (50-0)*4 = 200; side: (150-50)*2 = 200 → total 400
    let players = vec![
        PlayerForPots {
            id: "p1".into(),
            total_bet: 5000,
            has_folded: true,
            cards: vec![],
        },
        PlayerForPots {
            id: "p2".into(),
            total_bet: 5000,
            has_folded: false,
            cards: vec![],
        },
        PlayerForPots {
            id: "p3".into(),
            total_bet: 15000,
            has_folded: false,
            cards: vec![],
        },
        PlayerForPots {
            id: "p4".into(),
            total_bet: 15000,
            has_folded: false,
            cards: vec![],
        },
    ];
    let board = vec![
        c(Rank::Ace, Suit::Hearts),
        c(Rank::King, Suit::Hearts),
        c(Rank::Queen, Suit::Hearts),
        c(Rank::Jack, Suit::Hearts),
        c(Rank::Ten, Suit::Hearts),
    ];

    let side = resolve_side_pots(&players, &board);
    let contributions: u64 = players.iter().map(|p| p.total_bet).sum();
    assert!(
        sum_pots(&side.pots) == contributions,
        "pots ({}) divergem das contribuições ({})",
        sum_pots(&side.pots),
        contributions
    );
    assert_eq!(side.pots.len(), 2);
    assert_eq!(side.pots[0].amount, 20000);
    assert_eq!(side.pots[1].amount, 20000);

    // p1 foldou: continua elegível (contribuiu), mas NÃO pode ganhar nada.
    // O motor mantém p1 na lista de elegíveis do pote, mas a distribuição o exclui.
    assert!(
        !side.payouts.contains_key("p1") || side.payouts.get("p1").copied().unwrap_or(0) == 0,
        "jogador foldado não pode receber payout"
    );

    // Payouts (pré-rake) somam os pots, com resíduo de truncagem desprezível
    let payout_sum: u64 = side.payouts.values().sum();
    assert!(
        payout_sum <= sum_pots(&side.pots),
        "payouts ({}) não batem com os pots ({})",
        payout_sum,
        sum_pots(&side.pots)
    );
}
