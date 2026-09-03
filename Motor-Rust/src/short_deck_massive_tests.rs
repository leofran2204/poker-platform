//! Short Deck — suíte MASSIVA (feature `massive-tests`).
//!
//! Escala alinhada à documentação do motor (`QUALITY.md` / `DASHBOARD.md`):
//! - Extreme fuzz proptest: 100.000 casos no avaliador SD
//! - Fairness: 500.000 deals 6-max no baralho de 36 (sem duplicata + qui-quadrado)
//! - Stress integração: 200.000 pipelines deal→eval→side_pots→rake
//! - Stress game loop: 50.000 mãos 6-max com conservação de fichas
//!
//! Total ≈ 850k+ iterações dedicadas SD neste módulo (+ suíte
//! `tests/short_deck_massive.rs` com +1.2M).
//!
//! Rodar:
//!   cargo test --features massive-tests short_deck_massive -- --nocapture

use crate::deck::{
    compare_hands, create_short_deck, deal_cards, evaluate_hand_short_deck, shuffle_deck, Card,
    HandRank, Rank, Suit,
};
use crate::game_loop::{GameLoop, PlayerMove};
use crate::hand_history::GameType;
use crate::rake::deduct_rake;
use crate::side_pots::{calculate_side_pots, PlayerForPots};
use crate::types::{PokerVariant, TableConfig};
use proptest::prelude::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
use std::time::Instant;

const SEED: u64 = 0x5D_5D_CAFE_BEEF_36;
const FAIRNESS_ITERS: u64 = 500_000;
const STRESS_PIPELINE_ITERS: u64 = 200_000;
const STRESS_GAMELOOP_HANDS: u32 = 50_000;

fn sd_extreme_config() -> ProptestConfig {
    let cases = std::env::var("EXTREME_FUZZ_CASES")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(100_000);
    ProptestConfig {
        cases,
        max_shrink_iters: 100,
        ..ProptestConfig::default()
    }
}

fn sd_value(rank: HandRank) -> u8 {
    match rank {
        HandRank::HighCard => 1,
        HandRank::OnePair => 2,
        HandRank::TwoPair => 3,
        HandRank::Straight => 4,
        HandRank::ThreeOfAKind => 5,
        HandRank::FullHouse => 6,
        HandRank::Flush => 7,
        HandRank::FourOfAKind => 8,
        HandRank::StraightFlush => 9,
        HandRank::RoyalFlush => 10,
    }
}

fn card_index_sd(c: &Card) -> usize {
    // ranks 6..=14 → 0..=8 ; suits 0..=3 → índice 0..36
    let rank_i = (c.rank as usize).saturating_sub(6);
    (c.suit as usize) * 9 + rank_i
}

fn deal_sd_hand(rng: &mut StdRng, n_players: usize) -> (Vec<Vec<Card>>, Vec<Card>) {
    let mut deck = create_short_deck();
    deck.shuffle(rng);
    let mut holes: Vec<Vec<Card>> = vec![Vec::with_capacity(2); n_players];
    for _round in 0..2 {
        for hole in &mut holes {
            let (cards, rest) = deal_cards(&deck, 1);
            deck = rest;
            hole.extend(cards);
        }
    }
    let (_b, d) = deal_cards(&deck, 1);
    deck = d;
    let (flop, d) = deal_cards(&deck, 3);
    deck = d;
    let (_b, d) = deal_cards(&deck, 1);
    deck = d;
    let (turn, d) = deal_cards(&deck, 1);
    deck = d;
    let (_b, d) = deal_cards(&deck, 1);
    deck = d;
    let (river, _d) = deal_cards(&deck, 1);
    let mut board = flop;
    board.extend(turn);
    board.extend(river);
    (holes, board)
}

// ─── 1. Extreme fuzz avaliador Short Deck (100k) ────────────────────

proptest! {
    #![proptest_config(sd_extreme_config())]
    #[test]
    fn extreme_fuzz_short_deck_evaluator(seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut deck = create_short_deck();
        deck.shuffle(&mut rng);
        prop_assert_eq!(deck.len(), 36);
        prop_assert!(deck.iter().all(|c| (c.rank as u8) >= 6));

        let hole = vec![deck[0], deck[1]];
        let board = vec![deck[2], deck[3], deck[4], deck[5], deck[6]];
        let res = evaluate_hand_short_deck(&hole, &board);
        prop_assert_eq!(res.value, sd_value(res.rank));
        prop_assert!(res.value >= 1 && res.value <= 10);

        // Flush sempre vence Full House sob regras SD
        let flush = evaluate_hand_short_deck(
            &[Card { rank: Rank::Ace, suit: Suit::Hearts }, Card { rank: Rank::King, suit: Suit::Hearts }],
            &[
                Card { rank: Rank::Queen, suit: Suit::Hearts },
                Card { rank: Rank::Jack, suit: Suit::Hearts },
                Card { rank: Rank::Nine, suit: Suit::Hearts },
                Card { rank: Rank::Eight, suit: Suit::Clubs },
                Card { rank: Rank::Seven, suit: Suit::Diamonds },
            ],
        );
        let boat = evaluate_hand_short_deck(
            &[Card { rank: Rank::Six, suit: Suit::Clubs }, Card { rank: Rank::Six, suit: Suit::Diamonds }],
            &[
                Card { rank: Rank::Six, suit: Suit::Spades },
                Card { rank: Rank::King, suit: Suit::Clubs },
                Card { rank: Rank::King, suit: Suit::Diamonds },
                Card { rank: Rank::Nine, suit: Suit::Spades },
                Card { rank: Rank::Eight, suit: Suit::Spades },
            ],
        );
        prop_assert_eq!(compare_hands(&flush, &boat), Ordering::Greater);
    }
}

proptest! {
    #![proptest_config(sd_extreme_config())]
    #[test]
    fn extreme_fuzz_short_deck_shuffle_composition(seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let base = create_short_deck();
        let mut expected = base.clone();
        expected.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.suit.cmp(&b.suit)));

        let shuffled = shuffle_deck(&base);
        prop_assert_eq!(shuffled.len(), 36);
        let mut got = shuffled.clone();
        got.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.suit.cmp(&b.suit)));
        prop_assert_eq!(got, expected.clone());

        // Segundo shuffle via StdRng também preserva composição
        let mut deck2 = create_short_deck();
        deck2.shuffle(&mut rng);
        let mut got2 = deck2;
        got2.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.suit.cmp(&b.suit)));
        prop_assert_eq!(got2, expected);
    }
}

// ─── 2. Fairness 500k deals Short Deck ──────────────────────────────

#[test]
fn short_deck_fairness_500k_no_duplicates_and_chi_squared() {
    let t0 = Instant::now();
    let n_players = 6;
    let mut hole_hist = [0u64; 36];
    let mut board_hist = [0u64; 36];
    let mut rng = StdRng::seed_from_u64(SEED ^ 0xFA17);

    for i in 0..FAIRNESS_ITERS {
        let (holes, board) = deal_sd_hand(&mut rng, n_players);
        let mut seen = [false; 36];
        for hand in &holes {
            assert_eq!(hand.len(), 2);
            for card in hand {
                assert!((card.rank as u8) >= 6, "carta baixa no hole iter {i}");
                let idx = card_index_sd(card);
                assert!(idx < 36);
                assert!(!seen[idx], "duplicata hole/board iter {i}");
                seen[idx] = true;
                hole_hist[idx] += 1;
            }
        }
        assert_eq!(board.len(), 5);
        for card in &board {
            assert!((card.rank as u8) >= 6, "carta baixa no board iter {i}");
            let idx = card_index_sd(card);
            assert!(!seen[idx], "duplicata board iter {i}");
            seen[idx] = true;
            board_hist[idx] += 1;
        }
    }

    // Qui-quadrado hole: cada carta aparece 2*6*ITERS / 36 vezes em média
    let hole_total = FAIRNESS_ITERS * (n_players as u64) * 2;
    let expected_hole = hole_total as f64 / 36.0;
    let mut chi_hole = 0.0f64;
    for &c in &hole_hist {
        let d = c as f64 - expected_hole;
        chi_hole += d * d / expected_hole;
    }
    // df=35; limiar bem folgado (~4σ agregado) — evita flake sem relaxar demais
    assert!(
        chi_hole < 80.0,
        "qui-quadrado hole SD fora do esperado: {chi_hole} (esperado≈35)"
    );

    let board_total = FAIRNESS_ITERS * 5;
    let expected_board = board_total as f64 / 36.0;
    let mut chi_board = 0.0f64;
    for &c in &board_hist {
        let d = c as f64 - expected_board;
        chi_board += d * d / expected_board;
    }
    assert!(
        chi_board < 80.0,
        "qui-quadrado board SD fora do esperado: {chi_board}"
    );

    eprintln!(
        "SD fairness DONE iters={FAIRNESS_ITERS} elapsed={:?} chi_hole={chi_hole:.2} chi_board={chi_board:.2}",
        t0.elapsed()
    );
}

// ─── 3. Stress integração 200k (deal→eval→side_pots→rake) ───────────

#[test]
fn short_deck_stress_integration_200k_pipeline() {
    let t0 = Instant::now();
    let mut rng = StdRng::seed_from_u64(SEED);
    let cfg = TableConfig::new(200, 500, 1000).with_poker_variant(PokerVariant::ShortDeck);

    for i in 0..STRESS_PIPELINE_ITERS {
        let n = rng.gen_range(2usize..=6);
        let (holes, board) = deal_sd_hand(&mut rng, n);

        let mut seen = [false; 36];
        for hand in &holes {
            for card in hand {
                let idx = card_index_sd(card);
                assert!(!seen[idx], "dup pipeline {i}");
                seen[idx] = true;
            }
        }
        for card in &board {
            let idx = card_index_sd(card);
            assert!(!seen[idx], "dup board pipeline {i}");
            seen[idx] = true;
        }

        let mut evals = Vec::with_capacity(n);
        for hole in &holes {
            let r = evaluate_hand_short_deck(hole, &board);
            assert_eq!(r.value, sd_value(r.rank));
            evals.push(r);
        }
        // compare_hands transitivo fraco: max existe
        let mut best = 0usize;
        for j in 1..evals.len() {
            if compare_hands(&evals[j], &evals[best]) == Ordering::Greater {
                best = j;
            }
        }
        assert!(best < n);

        let mut players: Vec<PlayerForPots> = Vec::with_capacity(n);
        for (p, hole) in holes.iter().enumerate() {
            players.push(PlayerForPots {
                id: format!("p{p}"),
                total_bet: rng.gen_range(0u64..=10_000),
                has_folded: rng.gen_bool(0.12),
                cards: hole.clone(),
            });
        }
        // Garante ≥1 ativo
        if players.iter().all(|p| p.has_folded) {
            players[0].has_folded = false;
        }
        let pots = calculate_side_pots(&players);
        let contributed: u64 = players.iter().map(|p| p.total_bet).sum();
        let pot_sum: u64 = pots.iter().map(|p| p.amount).sum();
        assert_eq!(contributed, pot_sum, "side pots SD iter {i}");

        let rake = deduct_rake(&pots, &cfg, None);
        assert!(rake.total_rake <= cfg.rake_cap_for_players(n));
        assert!(rake.total_rake <= pot_sum);
        let after: u64 = rake.pots_after_rake.iter().map(|p| p.amount).sum();
        assert_eq!(after + rake.total_rake, pot_sum, "rake split SD iter {i}");
    }

    eprintln!(
        "SD stress pipeline DONE iters={STRESS_PIPELINE_ITERS} elapsed={:?}",
        t0.elapsed()
    );
}

// ─── 4. Stress game loop 50k mãos 6-max ──────────────────────────────

fn auto_play(gl: &mut GameLoop, bb: u64) {
    let mut steps = 0u32;
    while !gl.state.is_finished && steps < 2_500 {
        steps += 1;
        let Some(active) = gl.state.active_player().map(|p| p.id.clone()) else {
            break;
        };
        let to_call = {
            let p = gl.state.players.iter().find(|p| p.id == active).unwrap();
            gl.state.current_bet_to_match.saturating_sub(p.current_bet)
        };
        let roll = steps.wrapping_mul(41).wrapping_add(active.len() as u32) % 100;
        let mv = if to_call == 0 {
            if roll > 93 {
                PlayerMove::Raise(bb * 2)
            } else {
                PlayerMove::Check
            }
        } else if roll < 20 {
            PlayerMove::Fold
        } else if roll > 96 {
            PlayerMove::AllIn
        } else {
            PlayerMove::Call
        };
        if gl.player_action(&active, mv).is_err() {
            let fb = if to_call == 0 {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            };
            if gl.player_action(&active, fb).is_err() {
                let _ = gl.player_action(&active, PlayerMove::Fold);
            }
        }
    }
    assert!(gl.state.is_finished, "mão SD não terminou steps={steps}");
}

#[test]
fn short_deck_stress_gameloop_50k_six_max_chip_conservation() {
    const PLAYERS: usize = 6;
    const START: u64 = 10_000;
    const BB: u64 = 200;
    let t0 = Instant::now();
    let config = TableConfig::new(BB, 500, 1000).with_poker_variant(PokerVariant::ShortDeck);
    let mut stacks: Vec<(String, u64)> = (0..PLAYERS).map(|i| (format!("p{i}"), START)).collect();
    let mut total_rake = 0u64;
    let mut low_violations = 0u64;

    for hand_idx in 0..STRESS_GAMELOOP_HANDS {
        for (_, s) in stacks.iter_mut() {
            if *s < BB * 10 {
                *s = START;
            }
        }
        let before: u64 = stacks.iter().map(|(_, s)| *s).sum();
        let mut gl = GameLoop::new(
            config.clone(),
            format!("sd-m-{hand_idx}"),
            "SD massive".into(),
            GameType::Cash,
        );
        for (id, stack) in &stacks {
            gl.add_player(id.clone(), *stack);
        }
        gl.set_dealer((hand_idx as usize) % PLAYERS);
        gl.start_hand().expect("start");
        for p in &gl.state.players {
            for c in &p.hole_cards {
                if (c.rank as u8) < 6 {
                    low_violations += 1;
                }
            }
        }
        auto_play(&mut gl, BB);
        for c in &gl.state.community_cards {
            assert!((c.rank as u8) >= 6);
        }
        let res = gl.resolve_hand().expect("resolve");
        total_rake += res.rake;
        for p in &gl.state.players {
            let pay = res.payouts.get(&p.id).copied().unwrap_or(0);
            if let Some((_, s)) = stacks.iter_mut().find(|(id, _)| id == &p.id) {
                *s = p.stack + pay;
            }
        }
        let after: u64 = stacks.iter().map(|(_, s)| *s).sum();
        assert_eq!(after + res.rake, before, "chip leak mão {hand_idx}");
        if hand_idx > 0 && hand_idx % 10_000 == 0 {
            eprintln!(
                "SD gameloop progress {hand_idx}/{STRESS_GAMELOOP_HANDS} elapsed={:.1}s",
                t0.elapsed().as_secs_f64()
            );
        }
    }

    assert_eq!(low_violations, 0);
    eprintln!(
        "SD gameloop DONE hands={STRESS_GAMELOOP_HANDS} elapsed={:?} rake={total_rake}",
        t0.elapsed()
    );
}
