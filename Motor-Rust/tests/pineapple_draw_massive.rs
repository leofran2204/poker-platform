//! Suíte massiva dedicada ao Brazilian Pineapple (`pineapple_draw_v4`) 6-max,
//! com o Loss Deflator LIGADO (ele vale em todas as modalidades; BUSINESS_RULES 11).
//!
//! Cobertura:
//! - 100.000 avaliações aleatórias contra uma referência por força bruta
//!   (melhor mão de 5 com no máximo 2 privadas), com 3 e com 2 privadas, e o
//!   descarte automático (fica com as duas melhores; no empate descarta a menor);
//! - 100.000 mãos de cash 6-max no `GameLoop` com a mesa do catálogo
//!   (R$ 0,50/R$ 0,50, rake 5% com cap de R$ 100), stacks variados e troca na
//!   vez do jogador no flop, no turn e no river, descarte aberto no river e
//!   all-in que não troca mais; 1 mão em cada 100 em "modo pressão" (muito
//!   all-in de vários jogadores) para estressar o Loss Deflator;
//! - cartas únicas: carta trocada nunca volta e as 52 cartas fecham no fim;
//! - potes paralelos e vencedores recalculados por fora, rake conferido e
//!   fichas conservadas ao centavo;
//! - Loss Deflator recalculado por fora: a foto do all-in pago (fase, mesa e
//!   privadas de todos) é tirada pela própria suíte; a equity é enumerada por
//!   completo no flop e no turn e conferida por um Monte Carlo independente no
//!   pré-flop (amostra); faixa, teto por pote, rateio e débito dos vencedores
//!   são refeitos ao centavo;
//! - relatório final: quantas mãos tiveram devolução, faixas, fases, quanto do
//!   pote voltou, mano a mano × 3 ou mais jogadores e quantas vezes um
//!   adversário ainda trocou carta depois da foto do all-in.
//!
//! A suíte usa o rake do motor (`deduct_rake_for_hand_with_player_count`), que
//! tem suítes próprias (`rake_regressions`, `rake_cap_schedule`).
//!
//! Rodar (lenta em debug):
//!   cargo test --release --features massive-tests --test pineapple_draw_massive -- --nocapture
//! Ensaio curto: PINEAPPLE_MASSIVE_HANDS=2000 PINEAPPLE_MASSIVE_EVALS=5000.

use poker_engine::deck::{
    compare_hands, create_deck, evaluate_hand, evaluate_hand_pineapple_draw,
    pineapple_draw_best_discard, Card, HandResult, Rank, Suit,
};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::{EndReason, GameType};
use poker_engine::loss_deflator::get_multiway_win_probability_for_variant;
use poker_engine::pineapple_draw::suggested_swap;
use poker_engine::rake::{deduct_rake_for_hand_with_player_count, RakeRounding};
use poker_engine::types::{GamePhase, PokerVariant, Pot, TableConfig};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

const RANDOM_EVALUATIONS: u32 = 100_000;
const CASH_HANDS: u32 = 100_000;
const PLAYERS: usize = 6;
const SMALL_BLIND: u64 = 50;
const BIG_BLIND: u64 = 50;
const BUY_IN: u64 = 7_500;
const MIN_STACK: u64 = 2_000;
const REBUY_BELOW: u64 = 500;
const RAKE_BPS: u16 = 500;
const RAKE_CAP: u64 = 10_000;
/// Amostras do Monte Carlo do motor no pré-flop (`TWO_PLUS_THREE_MC_SAMPLES`).
const ENGINE_MC_SAMPLES: f64 = 120_000.0;
const REFERENCE_MC_SAMPLES: u32 = 20_000;
const REFERENCE_MC_CASES: u32 = 120;

type CardKey = (Rank, Suit);

fn key(card: &Card) -> CardKey {
    (card.rank, card.suit)
}

fn env_count(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

// ─── Referência de avaliação ───

/// Referência explícita da regra: entre as comunitárias e no máximo duas
/// privadas, testa toda mão de 5 cartas possível (46 com 3 privadas, 21 com 2).
fn reference_pineapple(hole: &[Card], board: &[Card]) -> HandResult {
    assert!(matches!(hole.len(), 2 | 3), "Pineapple tem 2 ou 3 privadas");
    assert_eq!(board.len(), 5, "referência exige mesa completa");
    let mut best: Option<HandResult> = None;
    for hole_mask in 0u32..(1 << hole.len()) {
        let from_hole = hole_mask.count_ones() as usize;
        if from_hole > 2 {
            continue;
        }
        let selected_hole: Vec<Card> = (0..hole.len())
            .filter(|i| hole_mask >> i & 1 == 1)
            .map(|i| hole[i])
            .collect();
        for board_mask in 0u32..32 {
            if board_mask.count_ones() as usize != 5 - from_hole {
                continue;
            }
            let selected_board: Vec<Card> = (0..5)
                .filter(|i| board_mask >> i & 1 == 1)
                .map(|i| board[i])
                .collect();
            let candidate = evaluate_hand(&selected_hole, &selected_board);
            if best
                .as_ref()
                .is_none_or(|current| compare_hands(&candidate, current) == Ordering::Greater)
            {
                best = Some(candidate);
            }
        }
    }
    best.expect("há pelo menos a mão só da mesa")
}

fn assert_same_hand(actual: &HandResult, expected: &HandResult, context: &str) {
    assert_eq!(actual.rank, expected.rank, "rank divergente: {context}");
    assert_eq!(actual.value, expected.value, "value divergente: {context}");
    assert_eq!(
        compare_hands(actual, expected),
        Ordering::Equal,
        "desempate divergente: {context}"
    );
}

/// O descarte automático guarda as duas melhores; entre descartes igualmente
/// bons, joga fora a carta de menor valor.
fn assert_optimal_discard(before: &[Card], kept: &[Card], board: &[Card], context: &str) {
    assert_eq!(before.len(), 3, "descarte parte de 3 privadas: {context}");
    assert_eq!(kept.len(), 2, "descarte deixa 2 privadas: {context}");
    let best = reference_pineapple(before, board);
    assert_same_hand(&reference_pineapple(kept, board), &best, context);
    let spare = before
        .iter()
        .find(|card| !kept.contains(card))
        .expect("carta descartada");
    let lowest_optimal = (0..3)
        .filter(|&out| {
            let pair: Vec<Card> = (0..3).filter(|&i| i != out).map(|i| before[i]).collect();
            compare_hands(&reference_pineapple(&pair, board), &best) == Ordering::Equal
        })
        .map(|out| before[out].rank)
        .min()
        .expect("há descarte ótimo");
    assert_eq!(
        spare.rank, lowest_optimal,
        "empate descarta a menor: {context}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "massive-tests"),
    ignore = "suíte massiva; rode com --release --features massive-tests"
)]
fn pineapple_one_hundred_thousand_evaluations_match_brute_force() {
    let evaluations = env_count("PINEAPPLE_MASSIVE_EVALS", RANDOM_EVALUATIONS);
    let mut rng = StdRng::seed_from_u64(0x00B4_A21A_2026);
    let mut rank_histogram: BTreeMap<String, u64> = BTreeMap::new();
    let mut board_only_or_one_card = 0u64;
    let started = Instant::now();

    for iteration in 0..evaluations {
        let mut deck = create_deck();
        deck.shuffle(&mut rng);
        let hole = &deck[..3];
        let board = &deck[3..8];
        let context = format!("avaliação {iteration}");

        let expected_three = reference_pineapple(hole, board);
        assert_same_hand(
            &evaluate_hand_pineapple_draw(hole, board),
            &expected_three,
            &context,
        );
        let expected_two = reference_pineapple(&hole[..2], board);
        assert_same_hand(
            &evaluate_hand_pineapple_draw(&hole[..2], board),
            &expected_two,
            &context,
        );
        assert_ne!(
            compare_hands(&expected_three, &expected_two),
            Ordering::Less,
            "3 privadas nunca valem menos que 2 delas: {context}"
        );

        let spare = pineapple_draw_best_discard(hole, board).expect("3 privadas");
        let kept: Vec<Card> = (0..3).filter(|&i| i != spare).map(|i| hole[i]).collect();
        assert_optimal_discard(hole, &kept, board, &context);

        let board_hand = reference_pineapple(&hole[..2], board);
        let with_board_only = evaluate_hand(&[], board);
        if compare_hands(&board_hand, &with_board_only) == Ordering::Equal {
            board_only_or_one_card += 1;
        }
        *rank_histogram
            .entry(format!("{:?}", expected_three.rank))
            .or_insert(0) += 1;
    }

    let elapsed = started.elapsed();
    eprintln!(
        "Pineapple eval DONE evaluations={evaluations} elapsed={elapsed:?} eps={:.1} board_plays={board_only_or_one_card} ranks={rank_histogram:?}",
        evaluations as f64 / elapsed.as_secs_f64().max(0.001)
    );
    assert!(
        rank_histogram.len() >= 7 || evaluations < 50_000,
        "amostra massiva deveria produzir pelo menos sete classes de mãos"
    );
}

// ─── Foto do all-in e equity de referência ───

#[derive(Clone, Debug)]
struct Snapshot {
    phase: GamePhase,
    board: Vec<Card>,
    hero: Vec<Card>,
    opponents: Vec<(String, Vec<Card>)>,
}

struct PreState {
    phase: GamePhase,
    board: Vec<Card>,
    holes: HashMap<String, Vec<Card>>,
}

impl PreState {
    fn of(game: &GameLoop) -> Self {
        Self {
            phase: game.state.phase,
            board: game.state.community_cards.clone(),
            holes: game
                .state
                .players
                .iter()
                .map(|player| (player.id.clone(), player.hole_cards.clone()))
                .collect(),
        }
    }
}

/// BUSINESS_RULES 11.2: a foto é fixada no primeiro pagamento do all-in
/// (inclusive pagamento all-in menor) e guarda todos os adversários ainda na
/// mão. Lida do estado público logo depois de cada ação de aposta; fase, mesa e
/// privadas são as de antes da ação (a ação de aposta não mexe nas cartas, mas
/// pode fechar a rodada e abrir a street seguinte).
fn capture_paid_all_ins(
    pre: &PreState,
    game: &GameLoop,
    snapshots: &mut HashMap<String, Snapshot>,
) {
    for hero in &game.state.players {
        if !hero.is_all_in
            || hero.has_folded
            || hero.total_bet == 0
            || snapshots.contains_key(&hero.id)
        {
            continue;
        }
        let paid = game.state.players.iter().any(|other| {
            other.id != hero.id
                && other.is_in_hand()
                && other.total_bet > 0
                && (other.total_bet >= hero.total_bet || other.is_all_in)
        });
        if paid {
            snapshots.insert(
                hero.id.clone(),
                Snapshot {
                    phase: pre.phase,
                    board: pre.board.clone(),
                    hero: pre.holes[&hero.id].clone(),
                    opponents: game
                        .state
                        .players
                        .iter()
                        .filter(|other| other.id != hero.id && other.is_in_hand())
                        .map(|other| (other.id.clone(), pre.holes[&other.id].clone()))
                        .collect(),
                },
            );
        }
    }
}

/// Parcela do jogador 0 num desfecho: 1/k se está entre os k vencedores.
fn hero_share(hands: &[&[Card]], board: &[Card]) -> f64 {
    let evaluations: Vec<HandResult> = hands
        .iter()
        .map(|hand| evaluate_hand_pineapple_draw(hand, board))
        .collect();
    let mut best = 0;
    for index in 1..evaluations.len() {
        if compare_hands(&evaluations[index], &evaluations[best]) == Ordering::Greater {
            best = index;
        }
    }
    let winners = (0..evaluations.len())
        .filter(|&index| compare_hands(&evaluations[index], &evaluations[best]) == Ordering::Equal)
        .count();
    if compare_hands(&evaluations[0], &evaluations[best]) == Ordering::Equal {
        1.0 / winners as f64
    } else {
        0.0
    }
}

fn unseen_cards(hands: &[&[Card]], board: &[Card]) -> Vec<Card> {
    let known: HashSet<CardKey> = hands
        .iter()
        .flat_map(|hand| hand.iter())
        .chain(board.iter())
        .map(key)
        .collect();
    create_deck()
        .into_iter()
        .filter(|card| !known.contains(&key(card)))
        .collect()
}

/// Enumeração completa dos desfechos (flop, turn e river).
fn exact_equity(hands: &[&[Card]], board: &[Card]) -> f64 {
    let rest = unseen_cards(hands, board);
    let mut full = board.to_vec();
    let (mut sum, mut total) = (0.0f64, 0u64);
    match 5 - board.len() {
        0 => {
            sum += hero_share(hands, &full);
            total += 1;
        }
        1 => {
            for card in &rest {
                full.push(*card);
                sum += hero_share(hands, &full);
                total += 1;
                full.pop();
            }
        }
        2 => {
            for first in 0..rest.len() {
                for second in (first + 1)..rest.len() {
                    full.push(rest[first]);
                    full.push(rest[second]);
                    sum += hero_share(hands, &full);
                    total += 1;
                    full.truncate(board.len());
                }
            }
        }
        other => panic!("enumeração completa só a partir do flop (faltam {other})"),
    }
    sum / total as f64
}

/// Monte Carlo independente do motor: outra semente, outro avaliador.
fn independent_mc_equity(hands: &[&[Card]], board: &[Card], samples: u32, seed: u64) -> f64 {
    let mut pool = unseen_cards(hands, board);
    let needed = 5 - board.len();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut full = Vec::with_capacity(5);
    let mut sum = 0.0f64;
    for _ in 0..samples {
        for slot in 0..needed {
            let pick = rng.gen_range(slot..pool.len());
            pool.swap(slot, pick);
        }
        full.clear();
        full.extend_from_slice(board);
        full.extend_from_slice(&pool[..needed]);
        sum += hero_share(hands, &full);
    }
    sum / samples as f64
}

/// Faixas da BUSINESS_RULES 11.1, escritas aqui de novo de propósito.
fn tier_basis_points(equity: f64) -> Option<u16> {
    if !(0.0..1.0).contains(&equity) {
        None
    } else if equity >= 0.86 {
        Some(3_500)
    } else if equity >= 0.76 {
        Some(2_500)
    } else if equity >= 0.66 {
        Some(1_500)
    } else if equity >= 0.56 {
        Some(700)
    } else {
        None
    }
}

fn near_tier_edge(equity: f64) -> bool {
    [0.56, 0.66, 0.76, 0.86]
        .iter()
        .any(|edge| (equity - edge).abs() < 1e-9)
}

// ─── Estatísticas ───

#[derive(Default)]
struct Stats {
    showdowns: u64,
    fold_wins: u64,
    side_pot_hands: u64,
    rebuys: u64,
    pressure_hands: u64,
    decisions: u64,
    keeps: u64,
    swaps: u64,
    chosen_discards: u64,
    auto_discards: u64,
    folds_without_drawing: u64,
    rake: u64,
    net_pot_showdowns: u64,
    snapshots: u64,
    deflator_candidates: u64,
    deflator_hands: u64,
    deflator_payments: u64,
    tiers: [u64; 4],
    phases: [u64; 3],
    heads_up: u64,
    multiway: u64,
    multiway_opponent_swapped_after_snapshot: u64,
    shared_pot_caps: u64,
    cashback: u64,
    net_pot_deflator_hands: u64,
    exact_checks: u64,
    replay_checks: u64,
    independent_mc_checks: u64,
    max_mc_gap: f64,
    engine_resolve_secs: f64,
    slowest_resolve_secs: f64,
    replay_secs: f64,
    exact_secs: f64,
    independent_mc_secs: f64,
}

// ─── Referência de pagamento e de Loss Deflator ───

struct ExpectedDeflator {
    loser: String,
    equity: f64,
    basis_points: u16,
    phase: GamePhase,
    opponents: usize,
    entries: Vec<(usize, u64)>,
}

struct Candidate {
    loser: String,
    equity: f64,
    basis_points: u16,
    phase: GamePhase,
    opponents: usize,
    requests: Vec<(usize, u64)>,
}

/// Divide o teto do pote entre os pedidos (maior resto; empate pela ordem dos
/// assentos a partir do botão, que é a ordem dos pedidos).
fn share_cap(requests: &[(usize, u64)], cap: u64) -> Vec<(usize, u64)> {
    let requested: u128 = requests.iter().map(|(_, amount)| *amount as u128).sum();
    if requested == 0 || cap == 0 {
        return requests.iter().map(|(index, _)| (*index, 0)).collect();
    }
    if requested <= cap as u128 {
        return requests.to_vec();
    }
    let mut shares: Vec<(usize, u64, u128)> = requests
        .iter()
        .map(|(index, amount)| {
            let numerator = *amount as u128 * cap as u128;
            (
                *index,
                (numerator / requested) as u64,
                numerator % requested,
            )
        })
        .collect();
    let given: u64 = shares.iter().map(|(_, amount, _)| *amount).sum();
    let mut order: Vec<usize> = (0..shares.len()).collect();
    order.sort_by(|&a, &b| shares[b].2.cmp(&shares[a].2).then(a.cmp(&b)));
    for &position in order.iter().take((cap - given) as usize) {
        shares[position].1 += 1;
    }
    shares
        .into_iter()
        .map(|(index, amount, _)| (index, amount))
        .collect()
}

/// Debita o cashback dos vencedores do pote, em rodadas iguais, sem passar da
/// parte de cada um naquele pote; centavo ímpar pela ordem dos assentos.
fn debit_winners(
    payouts: &mut HashMap<String, u64>,
    pot_left: &mut HashMap<String, u64>,
    winners: &[String],
    requested: u64,
) -> u64 {
    let available: u64 = winners
        .iter()
        .map(|id| {
            pot_left
                .get(id)
                .copied()
                .unwrap_or(0)
                .min(payouts.get(id).copied().unwrap_or(0))
        })
        .sum();
    let target = requested.min(available);
    let mut pending = target;
    while pending > 0 {
        let active: Vec<&String> = winners
            .iter()
            .filter(|id| {
                pot_left.get(*id).copied().unwrap_or(0) > 0
                    && payouts.get(*id).copied().unwrap_or(0) > 0
            })
            .collect();
        if active.is_empty() {
            break;
        }
        let share = pending / active.len() as u64;
        let odd = pending % active.len() as u64;
        let mut round = 0u64;
        for (position, id) in active.iter().enumerate() {
            let want = share + u64::from((position as u64) < odd);
            let debit = want
                .min(pot_left.get(*id).copied().unwrap_or(0))
                .min(payouts.get(*id).copied().unwrap_or(0));
            if debit == 0 {
                continue;
            }
            *pot_left.get_mut(*id).expect("parte no pote") -= debit;
            *payouts.get_mut(*id).expect("pagamento") -= debit;
            round += debit;
        }
        if round == 0 {
            break;
        }
        pending -= round;
    }
    target - pending
}

#[allow(clippy::too_many_arguments)]
fn reference_loss_deflator(
    game: &GameLoop,
    net_pots: &[Pot],
    pot_winners: &[Vec<String>],
    pot_shares: &[HashMap<String, u64>],
    payouts: &mut HashMap<String, u64>,
    snapshots: &HashMap<String, Snapshot>,
    seat_order: &[String],
    stats: &mut Stats,
    hand_index: u32,
) -> Vec<ExpectedDeflator> {
    let live = |id: &String| {
        game.state
            .players
            .iter()
            .any(|player| &player.id == id && player.is_in_hand())
    };
    let seat_position = |id: &String| {
        seat_order
            .iter()
            .position(|seat| seat == id)
            .unwrap_or(usize::MAX)
    };

    let mut candidates: Vec<Candidate> = Vec::new();
    for player in &game.state.players {
        if player.has_folded || !player.is_all_in {
            continue;
        }
        let Some(snapshot) = snapshots.get(&player.id) else {
            continue;
        };
        if snapshot.phase == GamePhase::Showdown {
            continue;
        }
        if payouts.get(&player.id).copied().unwrap_or(0) >= player.total_bet {
            continue;
        }
        let shared: Vec<usize> = (0..net_pots.len())
            .filter(|&index| net_pots[index].eligible_players.contains(&player.id))
            .collect();
        let has_opponent = shared.iter().any(|&index| {
            net_pots[index]
                .eligible_players
                .iter()
                .any(|other| other != &player.id && live(other))
        });
        let has_winner = shared
            .iter()
            .any(|&index| pot_winners[index].iter().any(|winner| winner != &player.id));
        if !has_opponent || !has_winner {
            continue;
        }
        stats.deflator_candidates += 1;

        let mut hands: Vec<&[Card]> = vec![snapshot.hero.as_slice()];
        hands.extend(snapshot.opponents.iter().map(|(_, cards)| cards.as_slice()));
        let timer = Instant::now();
        let replay = get_multiway_win_probability_for_variant(
            &snapshot.hero,
            &hands[1..],
            &snapshot.board,
            PokerVariant::PineappleDraw,
        )
        .expect("equity do Pineapple com a foto do all-in");
        stats.replay_secs += timer.elapsed().as_secs_f64();
        stats.replay_checks += 1;
        let timer = Instant::now();
        if snapshot.board.len() >= 3 {
            let exact = exact_equity(&hands, &snapshot.board);
            stats.exact_secs += timer.elapsed().as_secs_f64();
            stats.exact_checks += 1;
            assert!(
                (exact - replay).abs() < 1e-9,
                "equity exata divergente na mão {hand_index}: referência {exact} × motor {replay} ({snapshot:?})"
            );
            if !near_tier_edge(exact) {
                assert_eq!(tier_basis_points(exact), tier_basis_points(replay));
            }
        } else if stats.independent_mc_checks < u64::from(REFERENCE_MC_CASES) {
            let seed = 0x5EED_0000_u64 ^ (u64::from(hand_index) << 8) ^ stats.independent_mc_checks;
            let independent =
                independent_mc_equity(&hands, &snapshot.board, REFERENCE_MC_SAMPLES, seed);
            stats.independent_mc_secs += timer.elapsed().as_secs_f64();
            let tolerance = 5.0
                * 0.5
                * (1.0 / ENGINE_MC_SAMPLES + 1.0 / f64::from(REFERENCE_MC_SAMPLES)).sqrt();
            let gap = (independent - replay).abs();
            stats.independent_mc_checks += 1;
            stats.max_mc_gap = stats.max_mc_gap.max(gap);
            assert!(
                gap <= tolerance,
                "equity pré-flop fora da margem na mão {hand_index}: referência {independent} × motor {replay} (margem {tolerance})"
            );
        }

        let Some(basis_points) = tier_basis_points(replay) else {
            continue;
        };
        let eligible_total: u64 = shared.iter().map(|&index| net_pots[index].amount).sum();
        if eligible_total == 0 {
            continue;
        }
        let base = ((eligible_total as u128 * basis_points as u128) / 10_000) as u64;
        let mut requests = Vec::new();
        let mut given = 0u64;
        for (position, &index) in shared.iter().enumerate() {
            let amount = if position + 1 == shared.len() {
                base - given
            } else {
                ((base as u128 * net_pots[index].amount as u128) / eligible_total as u128) as u64
            };
            given += amount;
            requests.push((index, amount));
        }
        candidates.push(Candidate {
            loser: player.id.clone(),
            equity: replay,
            basis_points,
            phase: snapshot.phase,
            opponents: snapshot.opponents.len().max(1),
            requests,
        });
    }

    let mut pot_left: Vec<HashMap<String, u64>> = pot_shares.to_vec();
    let mut entries: Vec<Vec<(usize, u64)>> = vec![Vec::new(); candidates.len()];
    for (pot_index, pot) in net_pots.iter().enumerate() {
        let winners = &pot_winners[pot_index];
        let mut requests: Vec<(usize, u64)> = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| !winners.contains(&candidate.loser))
            .filter_map(|(index, candidate)| {
                candidate
                    .requests
                    .iter()
                    .find(|(pot, amount)| *pot == pot_index && *amount > 0)
                    .map(|(_, amount)| (index, *amount))
            })
            .collect();
        if requests.is_empty() {
            continue;
        }
        requests.sort_by_key(|(index, _)| seat_position(&candidates[*index].loser));
        let highest = requests
            .iter()
            .map(|(index, _)| candidates[*index].basis_points)
            .max()
            .unwrap_or(0);
        let cap = ((pot.amount as u128 * highest as u128) / 10_000) as u64;
        if requests.len() > 1 && requests.iter().map(|(_, amount)| *amount).sum::<u64>() > cap {
            stats.shared_pot_caps += 1;
        }
        for (index, amount) in share_cap(&requests, cap) {
            if amount == 0 {
                continue;
            }
            let loser = &candidates[index].loser;
            let ordered: Vec<String> = seat_order
                .iter()
                .filter(|id| winners.contains(id) && *id != loser)
                .cloned()
                .collect();
            let debited = debit_winners(payouts, &mut pot_left[pot_index], &ordered, amount);
            if debited > 0 {
                entries[index].push((pot_index, debited));
            }
        }
    }

    let mut expected = Vec::new();
    for (candidate, entries) in candidates.into_iter().zip(entries) {
        let cashback: u64 = entries.iter().map(|(_, amount)| *amount).sum();
        if cashback == 0 {
            continue;
        }
        *payouts.entry(candidate.loser.clone()).or_insert(0) += cashback;
        expected.push(ExpectedDeflator {
            loser: candidate.loser,
            equity: candidate.equity,
            basis_points: candidate.basis_points,
            phase: candidate.phase,
            opponents: candidate.opponents,
            entries,
        });
    }
    expected
}

/// Potes recalculados por fora, pelo nível de aposta de quem ainda está na
/// mão; o dinheiro de quem desistiu entra como dinheiro morto.
fn reference_live_pots(game: &GameLoop) -> Vec<(u64, Vec<String>)> {
    let mut levels: Vec<u64> = game
        .state
        .players
        .iter()
        .filter(|player| player.is_in_hand() && player.total_bet > 0)
        .map(|player| player.total_bet)
        .collect();
    levels.sort_unstable();
    levels.dedup();
    let mut pots = Vec::new();
    let mut previous = 0u64;
    for level in levels {
        let amount: u64 = game
            .state
            .players
            .iter()
            .map(|player| player.total_bet.min(level) - player.total_bet.min(previous))
            .sum();
        let mut eligible: Vec<String> = game
            .state
            .players
            .iter()
            .filter(|player| player.is_in_hand() && player.total_bet >= level)
            .map(|player| player.id.clone())
            .collect();
        eligible.sort();
        pots.push((amount, eligible));
        previous = level;
    }
    let beyond: u64 = game
        .state
        .players
        .iter()
        .map(|player| player.total_bet.saturating_sub(previous))
        .sum();
    assert_eq!(beyond, 0, "ninguém que desistiu pôs mais que o maior vivo");
    pots
}

/// Potes do motor agrupados por quem ainda está vivo neles (o motor também
/// abre níveis com a aposta de quem desistiu).
fn engine_live_pots(game: &GameLoop, pots: &[Pot]) -> Vec<(u64, Vec<String>)> {
    let mut merged: Vec<(u64, Vec<String>)> = Vec::new();
    for pot in pots {
        let mut eligible: Vec<String> = pot
            .eligible_players
            .iter()
            .filter(|id| {
                game.state
                    .players
                    .iter()
                    .any(|player| &player.id == *id && player.is_in_hand())
            })
            .cloned()
            .collect();
        eligible.sort();
        match merged.last_mut() {
            Some((amount, last)) if *last == eligible => *amount += pot.amount,
            _ => merged.push((pot.amount, eligible)),
        }
    }
    merged
}

fn without_zeros(map: &HashMap<String, u64>) -> BTreeMap<String, u64> {
    map.iter()
        .filter(|(_, amount)| **amount > 0)
        .map(|(id, amount)| (id.clone(), *amount))
        .collect()
}

// ─── Política de jogo (aleatória e reproduzível) ───

/// Estilo da mesa: `CALM` é jogo comum; `PRESSURE` (1 mão em cada
/// `PRESSURE_EVERY`) empurra all-ins de vários jogadores para estressar o
/// Loss Deflator, o teto dividido e os potes paralelos.
struct Style {
    check: f64,
    fold: f64,
    call: f64,
    postflop_shove: f64,
    half_pot: bool,
}

const CALM: Style = Style {
    check: 0.70,
    fold: 0.45,
    call: 0.87,
    postflop_shove: 0.03,
    half_pot: true,
};
const PRESSURE: Style = Style {
    check: 0.66,
    fold: 0.16,
    call: 0.72,
    postflop_shove: 0.05,
    half_pot: false,
};
const PRESSURE_EVERY: u32 = 100;

fn choose_move(game: &GameLoop, id: &str, rng: &mut StdRng, style: &Style) -> PlayerMove {
    let legal = game.legal_actions(id);
    let player = game.state.active_player().expect("jogador da vez");
    let roll: f64 = rng.gen();
    // No jogo comum, apostas entre o mínimo e metade do intervalo permitido.
    let sized = |rng: &mut StdRng| {
        let low = legal.minimum_wager;
        let span = legal.maximum_wager.saturating_sub(low);
        let high = low + if style.half_pot { span / 2 } else { span };
        rng.gen_range(low..=high)
    };
    let proposed = if game.state.discarding {
        if roll < 0.03 && legal.call_amount > 0 {
            PlayerMove::Fold
        } else if roll < 0.30 {
            PlayerMove::Discard(None)
        } else {
            PlayerMove::Discard(Some(rng.gen_range(0..3)))
        }
    } else if game.state.drawing {
        if roll < 0.03 && legal.call_amount > 0 {
            PlayerMove::Fold
        } else if roll < 0.55 {
            suggested_swap(&player.hole_cards, &game.state.community_cards)
                .map(PlayerMove::Swap)
                .unwrap_or(PlayerMove::Keep)
        } else if roll < 0.80 {
            PlayerMove::Swap(rng.gen_range(0..3))
        } else {
            PlayerMove::Keep
        }
    } else {
        // All-in pré-flop é raro com stack fundo e comum com stack curto.
        let preflop = game.state.phase == GamePhase::Preflop;
        let short = player.stack <= 15 * BIG_BLIND;
        let shove = match (preflop, short) {
            (true, false) => 0.01,
            (true, true) => 0.10,
            (false, _) => style.postflop_shove,
        };
        if legal.allows("check") {
            if roll < style.check {
                PlayerMove::Check
            } else if roll < 1.0 - shove {
                PlayerMove::Bet(sized(rng))
            } else {
                PlayerMove::AllIn
            }
        } else if roll < style.fold {
            PlayerMove::Fold
        } else if roll < style.call {
            PlayerMove::Call
        } else if roll < 1.0 - shove {
            PlayerMove::Raise(sized(rng))
        } else {
            PlayerMove::AllIn
        }
    };
    legal.constrain_move(proposed)
}

fn cards_of(game: &GameLoop, id: &str) -> Vec<Card> {
    game.state
        .players
        .iter()
        .find(|player| player.id == id)
        .expect("jogador existe")
        .hole_cards
        .clone()
}

fn difference(left: &[Card], right: &[Card]) -> Vec<Card> {
    left.iter()
        .filter(|card| !right.contains(card))
        .copied()
        .collect()
}

#[test]
#[cfg_attr(
    not(feature = "massive-tests"),
    ignore = "suíte massiva; rode com --release --features massive-tests"
)]
fn pineapple_one_hundred_thousand_cash_hands_with_loss_deflator_are_exact() {
    let hands = env_count("PINEAPPLE_MASSIVE_HANDS", CASH_HANDS);
    let config = TableConfig::new(BIG_BLIND, RAKE_BPS, RAKE_CAP)
        .with_small_blind(SMALL_BLIND)
        .with_poker_variant(PokerVariant::PineappleDraw);
    let mut deck_rng = StdRng::seed_from_u64(0x00B4_A21A_D3C1);
    let mut policy_rng = StdRng::seed_from_u64(0x00B4_A21A_9011);
    let mut stack_rng = StdRng::seed_from_u64(0x00B4_A21A_57AC);
    let mut stacks: Vec<(String, u64)> = (0..PLAYERS)
        .map(|seat| (format!("bp{seat}"), stack_rng.gen_range(MIN_STACK..=BUY_IN)))
        .collect();
    let mut stats = Stats::default();
    let started = Instant::now();
    let progress_every = (hands / 10).max(1);

    for hand_index in 0..hands {
        for (_, stack) in &mut stacks {
            if *stack < REBUY_BELOW {
                *stack = stack_rng.gen_range(MIN_STACK..=BUY_IN);
                stats.rebuys += 1;
            }
        }
        let chips_before: u64 = stacks.iter().map(|(_, stack)| *stack).sum();
        let style = if hand_index % PRESSURE_EVERY == PRESSURE_EVERY - 1 {
            stats.pressure_hands += 1;
            &PRESSURE
        } else {
            &CALM
        };

        let mut game = GameLoop::new(
            config.clone(),
            format!("bp-massive-{hand_index}"),
            "Brazilian Pineapple 6-max".to_string(),
            GameType::Cash,
        );
        for (player_id, stack) in &stacks {
            game.add_player(player_id.clone(), *stack);
        }
        game.set_dealer((hand_index as usize) % PLAYERS);
        let mut deck = create_deck();
        deck.shuffle(&mut deck_rng);
        game.start_hand_with_deck(deck)
            .expect("iniciar mão do Pineapple");

        let mut ever_seen: HashSet<CardKey> = HashSet::new();
        for player in &game.state.players {
            assert_eq!(
                player.hole_cards.len(),
                3,
                "3 privadas no deal: mão {hand_index}"
            );
            for card in &player.hole_cards {
                assert!(
                    ever_seen.insert(key(card)),
                    "carta repetida no deal: {card:?}"
                );
            }
        }
        let mut snapshots: HashMap<String, Snapshot> = HashMap::new();
        let mut all_in_cards: HashMap<String, Vec<Card>> = HashMap::new();
        let opening = PreState::of(&game);
        capture_paid_all_ins(&opening, &game, &mut snapshots);
        let mut drew_this_street: HashSet<String> = HashSet::new();
        let mut street = game.state.phase;
        let mut board_seen = 0usize;
        let mut steps = 0u32;

        while !game.state.is_finished {
            steps += 1;
            assert!(steps < 500, "mão {hand_index} não terminou");
            if game.state.phase != street {
                street = game.state.phase;
                drew_this_street.clear();
            }
            let id = game
                .state
                .active_player()
                .map(|player| player.id.clone())
                .expect("mão em andamento tem jogador da vez");
            let three_cards = cards_of(&game, &id).len() == 3;
            let postflop = matches!(
                game.state.phase,
                GamePhase::Flop | GamePhase::Turn | GamePhase::River
            );
            let first_turn = !drew_this_street.contains(&id);
            assert_eq!(
                game.state.drawing,
                postflop && three_cards && first_turn,
                "troca só na primeira vez da street: mão {hand_index} {:?}",
                game.state.phase
            );
            assert_eq!(
                game.state.discarding,
                game.state.phase == GamePhase::River && three_cards && !first_turn,
                "descarte do river depois da troca: mão {hand_index}"
            );

            let pre = PreState::of(&game);
            let was_drawing = game.state.drawing || game.state.discarding;
            let chosen = choose_move(&game, &id, &mut policy_rng, style);
            stats.decisions += 1;
            if let Err(error) = game.player_action(&id, chosen.clone()) {
                panic!("ação legal recusada na mão {hand_index}: {chosen:?} → {error:?}");
            }
            let before = &pre.holes[&id];
            let after = cards_of(&game, &id);
            let draw_move = matches!(
                chosen,
                PlayerMove::Keep | PlayerMove::Swap(_) | PlayerMove::Discard(_)
            );
            let actor = game
                .state
                .players
                .iter()
                .find(|player| player.id == id)
                .expect("jogador da vez existe");
            match chosen {
                PlayerMove::Keep => {
                    stats.keeps += 1;
                    drew_this_street.insert(id.clone());
                    assert_eq!(&after, before, "manter não muda as cartas");
                    assert_eq!(actor.draw_count, Some(0));
                }
                PlayerMove::Swap(position) => {
                    stats.swaps += 1;
                    drew_this_street.insert(id.clone());
                    assert_eq!(after.len(), 3, "troca mantém 3 privadas");
                    assert_eq!(difference(before, &after), vec![before[position]]);
                    let received = difference(&after, before);
                    assert_eq!(received.len(), 1, "troca recebe uma carta");
                    assert!(
                        ever_seen.insert(key(&received[0])),
                        "carta já vista voltou na troca: mão {hand_index}"
                    );
                    assert!(game.state.discard_pile.contains(&before[position]));
                    assert_eq!(actor.draw_count, Some(1));
                }
                PlayerMove::Discard(choice) => {
                    assert_eq!(after.len(), 2, "descarte deixa 2 privadas");
                    let removed = difference(before, &after);
                    assert_eq!(removed.len(), 1);
                    assert_eq!(actor.river_discard, Some(removed[0]), "descarte aberto");
                    match choice {
                        Some(position) => {
                            stats.chosen_discards += 1;
                            assert_eq!(removed[0], before[position]);
                        }
                        None => {
                            stats.auto_discards += 1;
                            assert_optimal_discard(
                                before,
                                &after,
                                &game.state.community_cards,
                                &format!("descarte automático, mão {hand_index}"),
                            );
                        }
                    }
                }
                PlayerMove::Fold if was_drawing => stats.folds_without_drawing += 1,
                _ => {}
            }

            if !draw_move {
                let known = snapshots.len();
                capture_paid_all_ins(&pre, &game, &mut snapshots);
                stats.snapshots += (snapshots.len() - known) as u64;
            }
            for player in &game.state.players {
                if player.is_all_in && !all_in_cards.contains_key(&player.id) {
                    all_in_cards.insert(player.id.clone(), pre.holes[&player.id].clone());
                }
                if draw_move && player.id == id {
                    continue;
                }
                let old = &pre.holes[&player.id];
                if old.len() == 3 && player.hole_cards.len() == 2 {
                    // Descarte automático na abertura do river (all-in) ou
                    // quando a mesa é completada direto até o showdown.
                    stats.auto_discards += 1;
                    let removed = difference(old, &player.hole_cards);
                    assert_eq!(removed.len(), 1);
                    assert_eq!(player.river_discard, Some(removed[0]), "descarte aberto");
                    assert_optimal_discard(
                        old,
                        &player.hole_cards,
                        &game.state.community_cards,
                        &format!("descarte automático, mão {hand_index}"),
                    );
                } else {
                    assert_eq!(&player.hole_cards, old, "cartas de outro jogador mudaram");
                }
            }
            for card in &game.state.community_cards[board_seen..] {
                assert!(
                    ever_seen.insert(key(card)),
                    "carta repetida na mesa: {card:?}"
                );
            }
            board_seen = game.state.community_cards.len();
        }

        // Fechamento do baralho: privadas, mesa, queimadas, descartes e resto.
        let mut accounted: Vec<Card> = game.state.community_cards.clone();
        accounted.extend(&game.state.burn_pile);
        accounted.extend(&game.state.discard_pile);
        accounted.extend(&game.state.deck);
        for player in &game.state.players {
            accounted.extend(&player.hole_cards);
            if let Some(card) = player.river_discard {
                assert!(
                    game.state.discard_pile.contains(&card),
                    "descarte vai para o muck"
                );
            }
            if let Some(cards) = all_in_cards.get(&player.id) {
                assert!(
                    player.hole_cards.iter().all(|card| cards.contains(card)),
                    "all-in não troca: mão {hand_index}"
                );
            }
        }
        assert_eq!(accounted.len(), 52, "52 cartas na mão {hand_index}");
        assert_eq!(accounted.iter().map(key).collect::<HashSet<_>>().len(), 52);

        let timer = Instant::now();
        let resolution = game.resolve_hand().expect("resolver mão do Pineapple");
        let resolve_secs = timer.elapsed().as_secs_f64();
        stats.engine_resolve_secs += resolve_secs;
        stats.slowest_resolve_secs = stats.slowest_resolve_secs.max(resolve_secs);
        let total_pot: u64 = resolution.pots.iter().map(|pot| pot.amount).sum();
        let total_bets: u64 = game
            .state
            .players
            .iter()
            .map(|player| player.total_bet)
            .sum();
        assert_eq!(
            total_pot, total_bets,
            "potes somam as apostas: mão {hand_index}"
        );
        let rake = deduct_rake_for_hand_with_player_count(
            &resolution.pots,
            &config,
            None,
            game.state.community_cards.len() >= 3,
            RakeRounding::HalfToEven,
            game.state.players.len(),
        );
        assert_eq!(
            rake.total_rake, resolution.rake,
            "rake divergente: mão {hand_index}"
        );
        assert!(resolution.rake <= RAKE_CAP);
        assert!(
            u128::from(resolution.rake) * 10_000
                <= u128::from(total_pot) * u128::from(RAKE_BPS)
                    + 10_000 * resolution.pots.len() as u128,
            "rake acima de 5%: mão {hand_index}"
        );
        stats.rake += resolution.rake;

        match resolution.end_reason {
            EndReason::Showdown => {
                stats.showdowns += 1;
                assert_eq!(game.state.community_cards.len(), 5);
                let live_pots = reference_live_pots(&game);
                assert_eq!(
                    engine_live_pots(&game, &resolution.pots),
                    live_pots,
                    "potes paralelos divergentes na mão {hand_index}"
                );
                if live_pots.len() > 1 {
                    stats.side_pot_hands += 1;
                }
                let seat_order: Vec<String> = (1..=game.state.players.len())
                    .map(|offset| {
                        let seat = (game.state.dealer_index + offset) % game.state.players.len();
                        game.state.players[seat].id.clone()
                    })
                    .collect();
                let mut showdown_hands: HashMap<String, HandResult> = HashMap::new();
                for player in game
                    .state
                    .players
                    .iter()
                    .filter(|player| player.is_in_hand())
                {
                    assert_eq!(player.hole_cards.len(), 2, "showdown com 2 privadas");
                    assert!(
                        player.river_discard.is_some(),
                        "todo vivo descartou no river"
                    );
                    showdown_hands.insert(
                        player.id.clone(),
                        reference_pineapple(&player.hole_cards, &game.state.community_cards),
                    );
                }
                for result in resolution
                    .player_results
                    .iter()
                    .filter(|result| !result.folded)
                {
                    let actual = result.best_hand.as_ref().expect("showdown tem best hand");
                    assert_same_hand(
                        actual,
                        &showdown_hands[&result.player_id],
                        &format!("mão {hand_index}"),
                    );
                }

                let net_pots = &rake.pots_after_rake;
                stats.net_pot_showdowns += net_pots.iter().map(|pot| pot.amount).sum::<u64>();
                let mut payouts: HashMap<String, u64> = HashMap::new();
                let mut pot_winners = Vec::new();
                let mut pot_shares = Vec::new();
                for pot in net_pots {
                    let mut winners: Vec<String> = Vec::new();
                    let mut best: Option<&HandResult> = None;
                    for id in &pot.eligible_players {
                        let Some(hand) = showdown_hands.get(id) else {
                            continue;
                        };
                        match best.map(|current| compare_hands(hand, current)) {
                            None | Some(Ordering::Greater) => {
                                best = Some(hand);
                                winners = vec![id.clone()];
                            }
                            Some(Ordering::Equal) => winners.push(id.clone()),
                            Some(Ordering::Less) => {}
                        }
                    }
                    assert!(!winners.is_empty(), "pote sem vencedor: mão {hand_index}");
                    let mut shares: HashMap<String, u64> = HashMap::new();
                    let base = pot.amount / winners.len() as u64;
                    let odd = pot.amount % winners.len() as u64;
                    for winner in &winners {
                        shares.insert(winner.clone(), base);
                    }
                    for winner in seat_order
                        .iter()
                        .filter(|id| winners.contains(id))
                        .take(odd as usize)
                    {
                        *shares.get_mut(winner).expect("vencedor") += 1;
                    }
                    for (winner, amount) in &shares {
                        *payouts.entry(winner.clone()).or_insert(0) += amount;
                    }
                    pot_winners.push(winners);
                    pot_shares.push(shares);
                }

                let expected = reference_loss_deflator(
                    &game,
                    net_pots,
                    &pot_winners,
                    &pot_shares,
                    &mut payouts,
                    &snapshots,
                    &seat_order,
                    &mut stats,
                    hand_index,
                );
                assert_eq!(
                    without_zeros(&resolution.payouts),
                    without_zeros(&payouts),
                    "pagamento divergente na mão {hand_index}"
                );
                assert_eq!(
                    resolution.loss_deflators.len(),
                    expected.len(),
                    "quantidade de Loss Deflators na mão {hand_index}"
                );
                for (actual, wanted) in resolution.loss_deflators.iter().zip(&expected) {
                    assert_eq!(actual.loser_id, wanted.loser, "perdedor: mão {hand_index}");
                    assert_eq!(
                        actual.loser_equity, wanted.equity,
                        "equity da foto: mão {hand_index}"
                    );
                    assert_eq!(actual.tier.basis_points(), wanted.basis_points);
                    assert_eq!(actual.phase, wanted.phase);
                    assert_eq!(usize::from(actual.opponents_counted), wanted.opponents);
                    let entries: Vec<(usize, u64)> = actual
                        .per_pot_cashback
                        .iter()
                        .map(|entry| (entry.pot_index, entry.amount))
                        .collect();
                    assert_eq!(entries, wanted.entries, "rateio por pote: mão {hand_index}");
                    let cashback: u64 = wanted.entries.iter().map(|(_, amount)| *amount).sum();
                    assert_eq!(actual.cashback, cashback);
                    assert!(
                        pot_winners
                            .iter()
                            .enumerate()
                            .any(|(index, winners)| winners.contains(&actual.winner_id)
                                && net_pots[index].eligible_players.contains(&actual.loser_id)),
                        "vencedor contábil ganhou um pote do perdedor"
                    );

                    stats.deflator_payments += 1;
                    stats.cashback += cashback;
                    let tier = match wanted.basis_points {
                        700 => 0,
                        1_500 => 1,
                        2_500 => 2,
                        _ => 3,
                    };
                    stats.tiers[tier] += 1;
                    let phase = match wanted.phase {
                        GamePhase::Preflop => 0,
                        GamePhase::Flop => 1,
                        _ => 2,
                    };
                    stats.phases[phase] += 1;
                    if wanted.opponents >= 2 {
                        stats.multiway += 1;
                        let snapshot = &snapshots[&wanted.loser];
                        let swapped = snapshot.opponents.iter().any(|(other, cards)| {
                            game.state.players.iter().any(|player| {
                                &player.id == other
                                    && player.is_in_hand()
                                    && player.hole_cards.iter().any(|card| !cards.contains(card))
                            })
                        });
                        if swapped {
                            stats.multiway_opponent_swapped_after_snapshot += 1;
                        }
                    } else {
                        stats.heads_up += 1;
                    }
                }
                assert_eq!(
                    resolution
                        .loss_deflator
                        .as_ref()
                        .map(|first| first.loser_id.clone()),
                    expected.first().map(|first| first.loser.clone())
                );
                if !expected.is_empty() {
                    stats.deflator_hands += 1;
                    stats.net_pot_deflator_hands +=
                        net_pots.iter().map(|pot| pot.amount).sum::<u64>();
                }
            }
            EndReason::AllFolded => {
                stats.fold_wins += 1;
                let survivor = game
                    .state
                    .players
                    .iter()
                    .find(|player| player.is_in_hand())
                    .expect("vitória por fold tem sobrevivente");
                assert_eq!(resolution.payouts.len(), 1);
                assert_eq!(
                    resolution.payouts.get(&survivor.id).copied().unwrap_or(0) + resolution.rake,
                    total_pot
                );
                assert!(
                    resolution.loss_deflators.is_empty(),
                    "fold não tem Loss Deflator"
                );
            }
            other => panic!("fim inesperado na mão {hand_index}: {other:?}"),
        }

        for player in &game.state.players {
            let payout = resolution.payouts.get(&player.id).copied().unwrap_or(0);
            let (_, stack) = stacks
                .iter_mut()
                .find(|(player_id, _)| player_id == &player.id)
                .expect("stack persistida existe");
            *stack = player.stack + payout;
        }
        let chips_after: u64 = stacks.iter().map(|(_, stack)| *stack).sum();
        assert_eq!(
            chips_after + resolution.rake,
            chips_before,
            "conservação de fichas falhou na mão {hand_index}"
        );

        if hand_index > 0 && hand_index % progress_every == 0 {
            eprintln!(
                "Pineapple progress {hand_index}/{hands} elapsed={:.1}s showdowns={} deflator_payments={}",
                started.elapsed().as_secs_f64(),
                stats.showdowns,
                stats.deflator_payments
            );
        }
    }

    let elapsed = started.elapsed();
    let percent = |part: u64, whole: u64| 100.0 * part as f64 / whole.max(1) as f64;
    eprintln!(
        "Pineapple DONE hands={hands} pressure_hands={} elapsed={elapsed:?} hps={:.1} decisions={} showdowns={} fold_wins={} side_pot_hands={} rebuys={}",
        stats.pressure_hands,
        hands as f64 / elapsed.as_secs_f64().max(0.001),
        stats.decisions,
        stats.showdowns,
        stats.fold_wins,
        stats.side_pot_hands,
        stats.rebuys
    );
    eprintln!(
        "Pineapple draws keeps={} swaps={} chosen_discards={} auto_discards={} folds_without_drawing={} rake_cents={}",
        stats.keeps,
        stats.swaps,
        stats.chosen_discards,
        stats.auto_discards,
        stats.folds_without_drawing,
        stats.rake
    );
    eprintln!(
        "Pineapple LossDeflator snapshots={} candidates={} payments={} hands_with_payment={} ({:.2}% of hands, {:.2}% of showdowns) tiers7/15/25/35={:?} phases_pre/flop/turn={:?} heads_up={} multiway={} multiway_opponent_swapped_after_snapshot={} shared_pot_caps={}",
        stats.snapshots,
        stats.deflator_candidates,
        stats.deflator_payments,
        stats.deflator_hands,
        percent(stats.deflator_hands, u64::from(hands)),
        percent(stats.deflator_hands, stats.showdowns),
        stats.tiers,
        stats.phases,
        stats.heads_up,
        stats.multiway,
        stats.multiway_opponent_swapped_after_snapshot,
        stats.shared_pot_caps
    );
    eprintln!(
        "Pineapple LossDeflator cashback_cents={} ({:.2}% of net pot in paying hands, {:.3}% of all showdown net pot) exact_checks={} replay_checks={} independent_mc_checks={} max_mc_gap={:.4}",
        stats.cashback,
        percent(stats.cashback, stats.net_pot_deflator_hands),
        percent(stats.cashback, stats.net_pot_showdowns),
        stats.exact_checks,
        stats.replay_checks,
        stats.independent_mc_checks,
        stats.max_mc_gap
    );
    eprintln!(
        "Pineapple timing engine_resolve={:.1}s slowest_resolve={:.3}s replay={:.1}s exact={:.1}s independent_mc={:.1}s",
        stats.engine_resolve_secs,
        stats.slowest_resolve_secs,
        stats.replay_secs,
        stats.exact_secs,
        stats.independent_mc_secs
    );
    assert!(stats.showdowns > 0, "stress deveria alcançar showdowns");
    assert!(
        stats.side_pot_hands > 0,
        "stress deveria alcançar potes paralelos"
    );
    assert!(
        stats.swaps > 0 && stats.keeps > 0,
        "stress deveria trocar e manter"
    );
    assert!(
        stats.chosen_discards > 0 && stats.auto_discards > 0,
        "stress deveria ter descarte escolhido e automático"
    );
    assert!(
        stats.deflator_payments > 0,
        "stress deveria acionar o Loss Deflator"
    );
    assert!(
        stats.exact_checks > 0,
        "stress deveria conferir equity exata"
    );
}
