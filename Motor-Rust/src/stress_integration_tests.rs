// stress_integration_tests.rs — Testes de Integração MASSIVOS (Nível 2 + Nível 6)
//
// Cobertura pesada da fronteira entre módulos sob variação real de entradas:
// cada cenário roda 200.000 iterações com inputs ALEATÓRIOS (mas válidos),
// afirmando INVARIANTES EXATOS (conservação de fichas, rake ≤ cap, ausência
// de cartas duplicadas, prêmios ≤ prize pool). Não afirma estatística — essa
// responsabilidade fica nos testes de fairness (card_fairness_tests.rs).
//
// RNG com SEED FIXO (StdRng) para reprodutibilidade: uma falha pode ser
// reproduzida exatamente. As cartas são embaralhadas com o mesmo rng semeado.
//
// Cenários (200k cada = 1M iterações):
//   1. Mão completa:       deck → side_pots → rake → hand_history
//   2. Side pots multi-way: all-ins/folds aleatórios, conservação + fold excluído
//   3. Torneio completo:    config/registro/blinds/eliminação aleatórios
//   4. Loss deflator+rake:  pots/phase aleatórios, cashback por tier + conservação
//   5. RNG + deck:          200k embaralhamentos, 52 únicas (permutação exata)

use crate::deck::{create_deck, deal_cards, Card, HandRank, HandResult, Rank, Suit};
use crate::hand_history::{
    create_hand_history, finalize_hand, record_action, set_community_cards, to_json, from_json,
    Action, EndReason, PlayerAction, PlayerResult,
};
use crate::loss_deflator::{
    calculate_progressive_loss_deflator, LossDeflatorTier, ProgressiveLossDeflatorParams,
};
use crate::rake::deduct_rake;
use crate::side_pots::{resolve_side_pots, PlayerForPots};
use crate::tournament_engine::{
    advance_blinds, create_tournament, eliminate_player, finish_tournament, register_player,
    start_tournament, BlindLevel, TournamentConfig, TournamentSpeed, TournamentStatus,
};
use crate::types::{GamePhase as TypesGamePhase, Pot, TableConfig};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

const ITER: u64 = 200_000;
const SEED: u64 = 0xDEAD_BEEF_CAFE_1234;

fn c(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn card_index(card: &Card) -> usize {
    (card.suit as usize) * 13 + (card.rank as usize - 2)
}

/// Embaralha com rng semeado (reprodutível) e distribui 2 hole + board 5 (com burns).
fn deal_random_hand(rng: &mut StdRng, n_players: usize) -> (Vec<Vec<Card>>, Vec<Card>) {
    let full = create_deck();
    let mut deck = full;
    deck.shuffle(rng);
    let mut holes: Vec<Vec<Card>> = vec![Vec::with_capacity(2); n_players];
    for _round in 0..2 {
        for hole in &mut holes {
            let (cards, rest) = deal_cards(&deck, 1);
            deck = rest;
            hole.extend(cards);
        }
    }
    let (_b, d) = deal_cards(&deck, 1); deck = d;
    let (flop, d) = deal_cards(&deck, 3); deck = d;
    let (_b, d) = deal_cards(&deck, 1); deck = d;
    let (turn, d) = deal_cards(&deck, 1); deck = d;
    let (_b, d) = deal_cards(&deck, 1); deck = d;
    let (river, _d) = deal_cards(&deck, 1);
    let mut board = flop;
    board.extend(turn);
    board.extend(river);
    (holes, board)
}

fn sum_pots(pots: &[Pot]) -> u64 {
    pots.iter().map(|p| p.amount).sum()
}

fn rake_cfg() -> TableConfig {
    TableConfig::new(1000, 5.0, 600)
}

// ─── Cenário 1: Mão completa (deck → side_pots → rake → hand_history) ───

#[test]
fn test_stress_integration_full_hand() {
    let mut rng = StdRng::seed_from_u64(SEED);
    let cfg = rake_cfg();

    for _ in 0..ITER {
        let n = rng.gen_range(2usize..=9);
        let (holes, board) = deal_random_hand(&mut rng, n);

        // Integridade do baralho usado: todas as cartas distintas
        let mut seen = [false; 52];
        for hand in &holes {
            for card in hand {
                let idx = card_index(card);
                assert!(!seen[idx], "carta duplicada no deal (integridade)");
                seen[idx] = true;
            }
        }
        for card in &board {
            let idx = card_index(card);
            assert!(!seen[idx], "carta duplicada no board (integridade)");
            seen[idx] = true;
        }

        // Apostas aleatórias em u64 centavos inteiros e folds aleatórios
        let mut players: Vec<PlayerForPots> = Vec::with_capacity(n);
        for (p, hole) in holes.iter().enumerate() {
            let bet = rng.gen_range(0u64..=10_000);
            let folded = rng.gen_bool(0.15);
            players.push(PlayerForPots {
                id: format!("p{}", p),
                total_bet: bet,
                has_folded: folded,
                cards: hole.clone(),
            });
        }

        let side = resolve_side_pots(&players, &board);
        let contributions: u64 = players.iter().map(|p| p.total_bet).sum();
        let total_pots_sum = sum_pots(&side.pots);

        // Invariante 1: soma dos pots == soma das contribuições (fichas conservadas exatas)
        assert_eq!(total_pots_sum, contributions, "fichas não conservadas: pots={} contrib={}", total_pots_sum, contributions);

        // Invariante 2: rake <= cap e <= total
        let rake = deduct_rake(&side.pots, &cfg, None);
        assert!(rake.total_rake <= cfg.rake_cap, "rake acima do cap");
        assert!(rake.total_rake <= total_pots_sum, "rake acima do total");

        // Invariante 3: pots após rake + total_rake == antes
        let before = total_pots_sum;
        let after = sum_pots(&rake.pots_after_rake);
        assert_eq!(after + rake.total_rake, before, "pots após rake incoerentes");

        // Invariante 4: payouts (pré-rake) somam os pots ou conservam com resto indivisível WSOP.
        let payout_sum: u64 = side.payouts.values().sum();
        assert!(payout_sum <= before, "payouts excederam os pots");
        let folded: std::collections::HashSet<String> =
            players.iter().filter(|p| p.has_folded).map(|p| p.id.clone()).collect();
        let residual = before.saturating_sub(payout_sum);
        let mut max_residual = 0u64;
        for pot in &side.pots {
            let all_folded = pot.eligible_players.iter().all(|id| folded.contains(id));
            if all_folded {
                max_residual += pot.amount;
            } else {
                let n_win = pot.eligible_players.iter().filter(|id| !folded.contains(*id)).count() as u64;
                max_residual += n_win;
            }
        }
        assert!(residual <= max_residual, "payouts muito abaixo dos pots (truncagem excessiva): residual={} max={}", residual, max_residual);

        // Invariante 5: foldado não recebe payout
        for p in &players {
            if p.has_folded {
                let got = side.payouts.get(&p.id).copied().unwrap_or(0);
                assert_eq!(got, 0, "jogador foldado recebeu payout");
            }
        }

        // Invariante 6: hand_history reflete os totais e roundtrip JSON preserva
        let total_u64 = before as u64;
        let rake_u64 = rake.total_rake as u64;

        let mut stacks = HashMap::new();
        for p in &players {
            stacks.insert(p.id.clone(), 1000u64);
        }
        let names: Vec<String> = players.iter().map(|p| p.id.clone()).collect();
        let mut history = create_hand_history(
            "stress-full".into(),
            crate::hand_history::TableConfig {
                table_name: "Stress".into(),
                small_blind: 5,
                big_blind: 10,
                ante: None,
                max_players: 9,
                game_type: crate::hand_history::GameType::Cash,
            },
            names,
            stacks,
        );
        for (i, p) in players.iter().enumerate() {
            record_action(&mut history, PlayerAction {
                player_id: p.id.clone(),
                action: if p.has_folded { Action::Fold } else { Action::Call },
                amount: p.total_bet as u64,
                phase: TypesGamePhase::Preflop,
                timestamp_ms: (i as u64) * 10,
            });
        }
        set_community_cards(&mut history, TypesGamePhase::Flop, board[0..3].to_vec());
        set_community_cards(&mut history, TypesGamePhase::Turn, vec![board[3]]);
        set_community_cards(&mut history, TypesGamePhase::River, vec![board[4]]);

        let mut results: Vec<PlayerResult> = players
            .iter()
            .map(|p| {
                let won = *side.payouts.get(&p.id).unwrap_or(&0);
                PlayerResult {
                    player_id: p.id.clone(),
                    finish_position: 0,
                    hole_cards: p.cards.clone(),
                    best_hand: Some(HandResult { rank: HandRank::HighCard, cards: p.cards.clone(), kickers: vec![], value: 0 }),
                    best_hand_name: Some("High Card".into()),
                    chips_won: won,
                    chips_lost: p.total_bet,
                    folded: p.has_folded,
                    was_all_in: p.total_bet > 0,
                }
            })
            .collect();
        let mut order: Vec<usize> = (0..results.len()).collect();
        order.sort_by(|&a, &b| {
            let pa = *side.payouts.get(&players[a].id).unwrap_or(&0);
            let pb = *side.payouts.get(&players[b].id).unwrap_or(&0);
            pb.cmp(&pa)
        });
        for (pos, &idx) in order.iter().enumerate() {
            results[idx].finish_position = (pos + 1) as u8;
        }
        finalize_hand(&mut history, results, total_u64, rake_u64, TypesGamePhase::River, EndReason::Showdown);

        assert_eq!(history.total_pot, total_u64, "hand_history.total_pot diverge");
        assert_eq!(history.rake, rake_u64, "hand_history.rake diverge");

        let json = to_json(&history).expect("serialização");
        let restored = from_json(&json).expect("desserialização");
        assert_eq!(restored.total_pot, total_u64, "roundtrip perdeu total_pot");
        assert_eq!(restored.rake, rake_u64, "roundtrip perdeu rake");
        assert_eq!(restored.players.len(), n);
    }
}

// ─── Cenário 2: Side pots multi-way (all-ins/folds aleatórios) ───

#[test]
fn test_stress_integration_sidepots_multiway() {
    let mut rng = StdRng::seed_from_u64(SEED ^ 0x1111);

    for _ in 0..ITER {
        let n = rng.gen_range(2usize..=9);
        let board = vec![
            c(Rank::Ace, Suit::Hearts),
            c(Rank::King, Suit::Hearts),
            c(Rank::Queen, Suit::Hearts),
            c(Rank::Jack, Suit::Hearts),
            c(Rank::Ten, Suit::Hearts),
        ];

        // Apostas em centavos inteiros
        let allin_levels = [5000u64, 12000u64, 25000u64, 50000u64, 100000u64];
        let mut players: Vec<PlayerForPots> = Vec::with_capacity(n);
        for p in 0..n {
            let bet = *allin_levels.choose(&mut rng).unwrap();
            let folded = rng.gen_bool(0.2);
            // Cartas dummy (não importa para os invariantes de conservação)
            let cards = vec![c(Rank::Two, Suit::Clubs), c(Rank::Three, Suit::Clubs)];
            players.push(PlayerForPots { id: format!("p{}", p), total_bet: bet, has_folded: folded, cards });
        }

        let side = resolve_side_pots(&players, &board);
        let contributions: u64 = players.iter().map(|p| p.total_bet).sum();
        let total_pots_sum = sum_pots(&side.pots);
        assert_eq!(total_pots_sum, contributions, "side pots não conservam: pots={} contrib={}", total_pots_sum, contributions);

        // Cada pote tem valor positivo e eligible_players não vazios
        for pot in &side.pots {
            assert!(pot.amount > 0, "pote com valor zero");
            assert!(!pot.eligible_players.is_empty(), "pote sem elegíveis");
        }

        // Foldado não recebe payout; payouts só para elegíveis não-foldados
        let payout_sum: u64 = side.payouts.values().sum();
        for p in &players {
            let got = side.payouts.get(&p.id).copied().unwrap_or(0);
            if p.has_folded {
                assert_eq!(got, 0, "foldado recebeu payout");
            }
            if got > 0 {
                assert!(!p.has_folded, "payout para foldado");
            }
        }
        assert!(payout_sum <= total_pots_sum, "payouts excederam pots");
    }
}

// ─── Cenário 3: Torneio completo (config/registro/blinds/eliminação) ───

#[test]
fn test_stress_integration_tournament() {
    let mut rng = StdRng::seed_from_u64(SEED ^ 0x2222);

    for _ in 0..ITER {
        let n_players = rng.gen_range(2u32..=40);
        let n_levels = rng.gen_range(1usize..=5);
        let mut blind_levels = Vec::with_capacity(n_levels);
        let mut sb = rng.gen_range(1u64..=50);
        let mut bb = sb * 2;
        for lvl in 1..=n_levels {
            blind_levels.push(BlindLevel {
                level: lvl as u32,
                small_blind: sb,
                big_blind: bb,
                ante: if rng.gen_bool(0.3) { rng.gen_range(0u64..=sb) } else { 0 },
                duration_minutes: rng.gen_range(5u32..=30),
            });
            sb = bb;
            bb *= 2;
        }

        // Prize distribution: k pesos aleatórios normalizados para somar ~1.0
        let k = rng.gen_range(1usize..=5);
        let mut weights: Vec<f64> = (0..k).map(|_| rng.gen_range(0.1f64..=1.0)).collect();
        let wsum: f64 = weights.iter().sum();
        for w in &mut weights {
            *w /= wsum;
        }

        let buy_in = rng.gen_range(100u64..=2000);
        let starting_stack = rng.gen_range(1000u64..=100000);
        let prize_pool_pct = rng.gen_range(0.5f64..=1.0);

        let config = TournamentConfig {
            name: "Stress MTT".into(),
            game_type: "Holdem".into(),
            buy_in,
            starting_stack,
            max_players: n_players,
            speed: if rng.gen_bool(0.5) { TournamentSpeed::Turbo } else { TournamentSpeed::Normal },
            blind_levels,
            prize_pool_pct,
            prize_distribution: weights,
            late_registration: false,
            late_registration_max_level: 0,
            allow_rebuy: false,
            allow_addon: false,
            rebuy_max_level: 0,
        };

        let mut state = create_tournament(config);
        let registered = rng.gen_range(2u32..=n_players);
        for i in 0..registered {
            register_player(&mut state, &format!("t{}", i), &format!("P{}", i)).expect("registro");
        }
        let expected_pool = (state.total_buyins as f64 * prize_pool_pct) as u64;
        assert_eq!(state.prize_pool, expected_pool, "prize pool incompatível com buy-ins");

        start_tournament(&mut state).expect("início");
        assert_eq!(state.status, TournamentStatus::Running);

        // Avança blinds aleatoriamente (respeitando limite de níveis)
        let advances = rng.gen_range(0..=n_levels as u32);
        for _ in 0..advances {
            if advance_blinds(&mut state).is_err() {
                break;
            }
        }

        // Elimina jogadores em ordem aleatória até sobrar 0 ou 1
        let mut ids: Vec<String> = (0..registered).map(|i| format!("t{}", i)).collect();
        ids.shuffle(&mut rng);
        let keep = rng.gen_range(0u32..=1); // 0 ou 1 restantes
        let to_eliminate = registered.saturating_sub(keep);
        for (pos, id) in ids.into_iter().take(to_eliminate as usize).enumerate() {
            eliminate_player(&mut state, &id, Some((pos + 1) as u32)).expect("elimina");
        }
        assert_eq!(state.players_remaining, keep, "players_remaining incompatível");

        let result = finish_tournament(&mut state).expect("finaliza");
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(result.total_players, registered);
        assert_eq!(result.total_prize_pool, state.prize_pool);

        // Prêmios pagos não podem exceder o prize pool
        let sum_prizes: u64 = result.winners.iter().map(|w| w.prize).sum();
        assert!(sum_prizes <= state.prize_pool, "prêmios ({}) excedem prize pool ({})", sum_prizes, state.prize_pool);

        // Se sobrou 1 jogador, ele é o campeão (pos 1)
        if keep == 1 {
            let champ = result.winners.iter().find(|w| w.position == 1).expect("campeão");
            assert_eq!(champ.player_id, ids_restante(&state, registered), "campeão incorreto");
        }
    }
}

/// Descobre o ID do único jogador não eliminado (para checar o campeão).
fn ids_restante(state: &crate::tournament_engine::TournamentState, registered: u32) -> String {
    for i in 0..registered {
        let id = format!("t{}", i);
        if let Some(e) = state.players.get(&id) {
            if e.eliminated_at.is_none() {
                return id;
            }
        }
    }
    String::new()
}

// ─── Cenário 4: Loss deflator + rake (pots/phase aleatórios) ───

#[test]
fn test_stress_integration_loss_deflator_plus_rake() {
    let mut rng = StdRng::seed_from_u64(SEED ^ 0x3333);
    let cfg = rake_cfg();
    let phases = [TypesGamePhase::Preflop, TypesGamePhase::Flop, TypesGamePhase::Turn];

    for _ in 0..ITER {
        // 1 a 4 pots com valores inteiros e elegíveis aleatórios
        let n_pots = rng.gen_range(1usize..=4);
        let mut pots = Vec::with_capacity(n_pots);
        let mut all_ids: Vec<String> = Vec::new();
        for _i in 0..n_pots {
            let amount = rng.gen_range(1000u64..=500_000u64);
            // 2 a 3 elegíveis por pote
            let n_elig = rng.gen_range(2usize..=3);
            let mut elig = Vec::with_capacity(n_elig);
            for _ in 0..n_elig {
                let pid = format!("pl{}", rng.gen_range(0usize..=5));
                if !elig.contains(&pid) {
                    elig.push(pid.clone());
                }
                if !all_ids.contains(&pid) {
                    all_ids.push(pid);
                }
            }
            pots.push(Pot { amount, eligible_players: elig });
        }

        // Rake
        let rake = deduct_rake(&pots, &cfg, None);
        assert!(rake.total_rake <= cfg.rake_cap, "rake acima do cap");
        let total = sum_pots(&pots);
        assert!(rake.total_rake <= total, "rake acima do total");

        // Loss deflator: sorteia loser/winner entre os elegíveis do pote 0
        let loser = pots[0].eligible_players[rng.gen_range(0..pots[0].eligible_players.len())].clone();
        let winner = pots[0].eligible_players[rng.gen_range(0..pots[0].eligible_players.len())].clone();
        let phase = phases[rng.gen_range(0..phases.len())];

        if let Some(deflator) = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
            pots: pots.clone(),
            loser_id: loser.clone(),
            winner_id: winner.clone(),
            phase,
        }) {
            // Tier e percentual corretos por fase
            let expected_tier = match phase {
                TypesGamePhase::Preflop => LossDeflatorTier::FifteenPercent,
                TypesGamePhase::Flop => LossDeflatorTier::TwentyFivePercent,
                TypesGamePhase::Turn => LossDeflatorTier::ThirtyFivePercent,
                _ => unreachable!(),
            };
            assert_eq!(deflator.tier, expected_tier, "tier incompatível com fase");

            // Cashback = 15/25/35% do total elegível ao perdedor, em centavos inteiros (floor)
            let pct = match phase {
                TypesGamePhase::Preflop => 0.15,
                TypesGamePhase::Flop => 0.25,
                TypesGamePhase::Turn => 0.35,
                _ => 0.0,
            };
            let expected_cb = ((deflator.eligible_pot_total as f64) * pct).floor() as u64;
            assert_eq!(deflator.cashback, expected_cb, "cashback incompatível: {} vs {}", deflator.cashback, expected_cb);

            // Conservação: rake + cashback não excedem o pote total
            assert!(rake.total_rake + deflator.cashback <= total, "rake+cashback excedem o pote");
            // Cashback incide apenas sobre pots onde o perdedor é elegível
            assert!(deflator.eligible_pot_total <= total);
        }
    }
}

// ─── Cenário 5: RNG + deck (200k embaralhamentos) ───

#[test]
fn test_stress_integration_rng_deck_integrity() {
    let mut rng = StdRng::seed_from_u64(SEED ^ 0x4444);
    let original = create_deck();
    assert_eq!(original.len(), 52);

    for _ in 0..ITER {
        let mut deck = original.clone();
        deck.shuffle(&mut rng);

        // 52 únicas (permutação exata do baralho original)
        assert_eq!(deck.len(), 52);
        let mut seen = [false; 52];
        let mut present = [false; 52];
        for card in &deck {
            let idx = card_index(card);
            assert!(!seen[idx], "shuffle produziu carta duplicada");
            seen[idx] = true;
            present[idx] = true;
        }
        for p in &present {
            assert!(*p, "carta faltando no shuffle");
        }
    }
}
