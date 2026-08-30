//! Suíte massiva Short Deck / Six Plus Hold'em (sempre on — sem feature gate).
//!
//! Escala (iters ≈ 1,2M+ nesta crate de teste):
//! - Regras determinísticas (wheel, flush>boat, ladder, SF)
//! - 1.000.000 avaliações aleatórias
//! - 100.000 shuffles com composição intacta
//! - 100.000 mãos 6-max GameLoop (conservação de fichas + sem cartas 2–5)
//! - Smoke 2/3/6 jogadores
//!
//! Complementar com a feature do motor:
//!   cargo test --features massive-tests short_deck_massive -- --nocapture
//!   → +100k×2 proptest + 500k fairness + 200k pipeline + 50k gameloop
//!
//! Rodar só esta suíte:
//!   cargo test --test short_deck_massive -- --nocapture

use poker_engine::deck::{
    compare_hands, create_short_deck, evaluate_hand_short_deck, shuffle_deck, Card, HandRank, Rank,
    Suit,
};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::{PokerVariant, TableConfig};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Instant;

const ALL_SUITS: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
const SD_RANKS: [Rank; 9] = [
    Rank::Ace,
    Rank::King,
    Rank::Queen,
    Rank::Jack,
    Rank::Ten,
    Rank::Nine,
    Rank::Eight,
    Rank::Seven,
    Rank::Six,
];

fn c(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn sd_value(rank: HandRank) -> u8 {
    match rank {
        HandRank::HighCard => 1,
        HandRank::OnePair => 2,
        HandRank::TwoPair => 3,
        HandRank::ThreeOfAKind => 4,
        HandRank::Straight => 5,
        HandRank::FullHouse => 6,
        HandRank::Flush => 7,
        HandRank::FourOfAKind => 8,
        HandRank::StraightFlush => 9,
        HandRank::RoyalFlush => 10,
    }
}

fn pick_unique_cards(rng: &mut impl Rng, n: usize) -> Vec<Card> {
    let mut deck = create_short_deck();
    // Fisher-Yates via rand (harness) — independente do CSPRNG de produção
    for i in (1..deck.len()).rev() {
        let j = rng.gen_range(0..=i);
        deck.swap(i, j);
    }
    assert!(n <= deck.len());
    deck.into_iter().take(n).collect()
}

// ─── Regras determinísticas ─────────────────────────────────────────

#[test]
fn short_deck_rules_deck_size_and_no_low_cards() {
    let deck = create_short_deck();
    assert_eq!(deck.len(), 36);
    assert_eq!(deck.len(), ALL_SUITS.len() * SD_RANKS.len());
    for card in &deck {
        assert!(
            (card.rank as u8) >= 6,
            "carta ilegal no short deck: {:?}",
            card
        );
        assert!(!matches!(
            card.rank,
            Rank::Two | Rank::Three | Rank::Four | Rank::Five
        ));
    }
    // Unicidade
    let mut seen = std::collections::HashSet::new();
    for card in &deck {
        assert!(seen.insert((card.rank as u8, format!("{:?}", card.suit))));
    }
}

#[test]
fn short_deck_rules_wheel_a6789_is_straight() {
    let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Six, Suit::Diamonds)];
    let board = vec![
        c(Rank::Seven, Suit::Clubs),
        c(Rank::Eight, Suit::Spades),
        c(Rank::Nine, Suit::Hearts),
        c(Rank::King, Suit::Diamonds),
        c(Rank::Jack, Suit::Clubs),
    ];
    let result = evaluate_hand_short_deck(&hole, &board);
    assert_eq!(result.rank, HandRank::Straight);
    assert_eq!(result.value, sd_value(HandRank::Straight));
}

#[test]
fn short_deck_rules_flush_beats_full_house() {
    let flush = evaluate_hand_short_deck(
        &[c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Hearts)],
        &[
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight, Suit::Hearts),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Six, Suit::Clubs),
            c(Rank::Jack, Suit::Diamonds),
        ],
    );
    let boat = evaluate_hand_short_deck(
        &[c(Rank::Ace, Suit::Clubs), c(Rank::Ace, Suit::Diamonds)],
        &[
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Nine, Suit::Spades),
            c(Rank::Eight, Suit::Clubs),
        ],
    );
    assert_eq!(flush.rank, HandRank::Flush);
    assert_eq!(boat.rank, HandRank::FullHouse);
    assert_eq!(compare_hands(&flush, &boat), Ordering::Greater);
    assert!(flush.value > boat.value);
}

#[test]
fn short_deck_rules_ranking_ladder() {
    // Hierarchy SD (valores): HC < 1P < 2P < trips < straight < boat < flush < quads < SF < royal
    let samples: Vec<(HandRank, Vec<Card>, Vec<Card>)> = vec![
        (
            HandRank::HighCard,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Diamonds)],
            vec![
                c(Rank::Queen, Suit::Clubs),
                c(Rank::Jack, Suit::Spades),
                c(Rank::Nine, Suit::Hearts),
                c(Rank::Eight, Suit::Diamonds),
                c(Rank::Six, Suit::Clubs),
            ],
        ),
        (
            HandRank::OnePair,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            vec![
                c(Rank::King, Suit::Clubs),
                c(Rank::Queen, Suit::Spades),
                c(Rank::Jack, Suit::Hearts),
                c(Rank::Nine, Suit::Diamonds),
                c(Rank::Eight, Suit::Clubs),
            ],
        ),
        (
            HandRank::TwoPair,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            vec![
                c(Rank::King, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::Queen, Suit::Hearts),
                c(Rank::Nine, Suit::Diamonds),
                c(Rank::Eight, Suit::Clubs),
            ],
        ),
        (
            HandRank::ThreeOfAKind,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            vec![
                c(Rank::Ace, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::Queen, Suit::Hearts),
                c(Rank::Nine, Suit::Diamonds),
                c(Rank::Eight, Suit::Clubs),
            ],
        ),
        (
            HandRank::Straight,
            vec![c(Rank::Ten, Suit::Hearts), c(Rank::Nine, Suit::Diamonds)],
            vec![
                c(Rank::Eight, Suit::Clubs),
                c(Rank::Seven, Suit::Spades),
                c(Rank::Six, Suit::Hearts),
                c(Rank::Ace, Suit::Diamonds),
                c(Rank::King, Suit::Clubs),
            ],
        ),
        (
            HandRank::FullHouse,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            vec![
                c(Rank::Ace, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::King, Suit::Hearts),
                c(Rank::Nine, Suit::Diamonds),
                c(Rank::Eight, Suit::Clubs),
            ],
        ),
        (
            HandRank::Flush,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Hearts)],
            vec![
                c(Rank::Queen, Suit::Hearts),
                c(Rank::Jack, Suit::Hearts),
                c(Rank::Nine, Suit::Hearts),
                c(Rank::Eight, Suit::Clubs),
                c(Rank::Seven, Suit::Diamonds),
            ],
        ),
        (
            HandRank::FourOfAKind,
            vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            vec![
                c(Rank::Ace, Suit::Clubs),
                c(Rank::Ace, Suit::Spades),
                c(Rank::King, Suit::Hearts),
                c(Rank::Nine, Suit::Diamonds),
                c(Rank::Eight, Suit::Clubs),
            ],
        ),
    ];

    let mut prev_value = 0u8;
    for (expected, hole, board) in samples {
        let r = evaluate_hand_short_deck(&hole, &board);
        assert_eq!(r.rank, expected, "esperava {:?}", expected);
        assert_eq!(r.value, sd_value(expected));
        assert!(
            r.value >= prev_value,
            "ranking SD não monotônico: {:?} value={} prev={}",
            expected,
            r.value,
            prev_value
        );
        prev_value = r.value;
    }
}

#[test]
fn short_deck_rules_straight_flush_wheel() {
    // A-6-7-8-9 todos de hearts = SF (wheel SD)
    let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Six, Suit::Hearts)];
    let board = vec![
        c(Rank::Seven, Suit::Hearts),
        c(Rank::Eight, Suit::Hearts),
        c(Rank::Nine, Suit::Hearts),
        c(Rank::King, Suit::Clubs),
        c(Rank::Jack, Suit::Diamonds),
    ];
    let r = evaluate_hand_short_deck(&hole, &board);
    assert!(
        matches!(r.rank, HandRank::StraightFlush | HandRank::RoyalFlush),
        "wheel suited deve ser SF, got {:?}",
        r.rank
    );
    assert!(r.value >= sd_value(HandRank::StraightFlush));
}

// ─── Massivo: avaliações aleatórias ────────────────────────────────

#[test]
fn short_deck_one_million_random_evals_no_panic_and_sd_values() {
    const N: u32 = 1_000_000;
    let mut rng = rand::thread_rng();
    let mut hist: HashMap<u8, u64> = HashMap::new();
    let mut flush_beats_boat_checks = 0u64;
    let t0 = Instant::now();

    for i in 0..N {
        let cards = pick_unique_cards(&mut rng, 7);
        let hole = vec![cards[0], cards[1]];
        let board = cards[2..7].to_vec();
        let result = evaluate_hand_short_deck(&hole, &board);
        assert_eq!(
            result.value,
            sd_value(result.rank),
            "value SD inconsistente em iter {i}: {:?}",
            result.rank
        );
        // Nunca carta baixa nas 7
        for card in &cards {
            assert!((card.rank as u8) >= 6);
        }
        *hist.entry(result.value).or_insert(0) += 1;

        // Amostra cruzada: se tirarmos flush e boat nesta mesma board-ish, flush ganha
        if i % 50 == 0 {
            let flush = evaluate_hand_short_deck(
                &[c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Hearts)],
                &[
                    c(Rank::Queen, Suit::Hearts),
                    c(Rank::Jack, Suit::Hearts),
                    c(Rank::Nine, Suit::Hearts),
                    c(Rank::Eight, Suit::Clubs),
                    c(Rank::Seven, Suit::Diamonds),
                ],
            );
            let boat = evaluate_hand_short_deck(
                &[c(Rank::Six, Suit::Clubs), c(Rank::Six, Suit::Diamonds)],
                &[
                    c(Rank::Six, Suit::Spades),
                    c(Rank::King, Suit::Clubs),
                    c(Rank::King, Suit::Diamonds),
                    c(Rank::Nine, Suit::Spades),
                    c(Rank::Eight, Suit::Spades),
                ],
            );
            assert_eq!(compare_hands(&flush, &boat), Ordering::Greater);
            flush_beats_boat_checks += 1;
        }
    }

    let elapsed = t0.elapsed();
    eprintln!(
        "SD evals N={N} elapsed={elapsed:?} eps={:.0} hist={hist:?} flush>boat_checks={flush_beats_boat_checks}",
        N as f64 / elapsed.as_secs_f64().max(0.001)
    );
    assert!(hist.len() >= 5, "esperava diversidade de ranks SD");
    // High card / pair devem aparecer; flush e boat devem aparecer em 1M
    assert!(hist.get(&1).copied().unwrap_or(0) > 0 || hist.get(&2).copied().unwrap_or(0) > 0);
    assert!(
        hist.get(&6).copied().unwrap_or(0) > 0,
        "full house deveria aparecer em 1M evals"
    );
    assert!(
        hist.get(&7).copied().unwrap_or(0) > 0,
        "flush deveria aparecer em 1M evals"
    );
}

#[test]
fn short_deck_shuffle_preserves_composition_100k() {
    const N: u32 = 100_000;
    let base = create_short_deck();
    let mut base_sorted = base.clone();
    base_sorted.sort_by_key(|c| (c.rank as u8, format!("{:?}", c.suit)));

    for i in 0..N {
        let deck = shuffle_deck(&create_short_deck());
        assert_eq!(deck.len(), 36, "iter {i}");
        let mut sorted = deck.clone();
        sorted.sort_by_key(|c| (c.rank as u8, format!("{:?}", c.suit)));
        assert_eq!(sorted, base_sorted, "composição alterada no shuffle {i}");
        assert!(deck.iter().all(|c| (c.rank as u8) >= 6));
    }
}

// ─── Stress 6-max cash Short Deck ───────────────────────────────────

fn auto_play_until_finished(gl: &mut GameLoop, big_blind: u64) {
    let mut steps = 0u32;
    while !gl.state.is_finished && steps < 2_500 {
        steps += 1;
        let Some(active) = gl.state.active_player().map(|p| p.id.clone()) else {
            break;
        };
        let to_call = {
            let p = gl
                .state
                .players
                .iter()
                .find(|p| p.id == active)
                .expect("active");
            gl.state.current_bet_to_match.saturating_sub(p.current_bet)
        };
        let roll = steps.wrapping_mul(31).wrapping_add(active.len() as u32) % 100;
        let mv = if to_call == 0 {
            if roll > 94 {
                PlayerMove::Raise(big_blind * 2)
            } else {
                PlayerMove::Check
            }
        } else if roll < 18 {
            // Mais folds → exercita AllFolded além de showdown/eval SD
            PlayerMove::Fold
        } else if roll > 96 {
            PlayerMove::AllIn
        } else {
            PlayerMove::Call
        };
        if gl.player_action(&active, mv).is_err() {
            let fallback = if to_call == 0 {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            };
            if gl.player_action(&active, fallback).is_err() {
                let _ = gl.player_action(&active, PlayerMove::Fold);
            }
        }
    }
    assert!(
        gl.state.is_finished,
        "mão SD não terminou (steps={steps} phase={:?})",
        gl.state.phase
    );
}

#[test]
fn short_deck_hundred_thousand_hands_six_max_conserves_chips() {
    const HANDS: u32 = 100_000;
    const PLAYERS: usize = 6;
    const STARTING_STACK: u64 = 10_000; // stack de stress (centavos)
    const BIG_BLIND: u64 = 200;
    const RAKE_BPS: u16 = 500;
    const RAKE_CAP: u64 = 1_000;

    let config =
        TableConfig::new(BIG_BLIND, RAKE_BPS, RAKE_CAP).with_poker_variant(PokerVariant::ShortDeck);
    assert_eq!(config.poker_variant, PokerVariant::ShortDeck);

    let mut stacks: Vec<(String, u64)> = (0..PLAYERS)
        .map(|i| (format!("sd{i}"), STARTING_STACK))
        .collect();

    let mut hands_ok = 0u32;
    let mut total_rake = 0u64;
    let mut showdowns = 0u32;
    let mut folds_win = 0u32;
    let mut rebuy_events = 0u32;
    let mut dealt_low_card_violations = 0u64;

    let t0 = Instant::now();

    for hand_idx in 0..HANDS {
        for (_, stack) in stacks.iter_mut() {
            if *stack < BIG_BLIND * 10 {
                *stack = STARTING_STACK;
                rebuy_events += 1;
            }
        }

        let chips_before: u64 = stacks.iter().map(|(_, s)| *s).sum();

        let mut gl = GameLoop::new(
            config.clone(),
            format!("sd-hand-{hand_idx}"),
            "SD 1/2 6-max".to_string(),
            GameType::Cash,
        );
        for (id, stack) in &stacks {
            gl.add_player(id.clone(), *stack);
        }
        gl.set_dealer((hand_idx as usize) % PLAYERS);
        gl.start_hand().expect("start_hand SD");

        // Invariante: cartas dealadas só do short deck
        for p in &gl.state.players {
            for card in &p.hole_cards {
                if (card.rank as u8) < 6 {
                    dealt_low_card_violations += 1;
                }
            }
        }
        for card in &gl.state.community_cards {
            if (card.rank as u8) < 6 {
                dealt_low_card_violations += 1;
            }
        }

        auto_play_until_finished(&mut gl, BIG_BLIND);

        for card in &gl.state.community_cards {
            assert!(
                (card.rank as u8) >= 6,
                "board com carta baixa na mão {hand_idx}: {:?}",
                card
            );
        }

        let resolution = gl.resolve_hand().expect("resolve_hand SD");
        total_rake += resolution.rake;
        let platform_fee = (resolution.rake * 15) / 100;
        let club_rake = resolution.rake.saturating_sub(platform_fee);
        assert_eq!(platform_fee + club_rake, resolution.rake);

        match resolution.end_reason {
            poker_engine::hand_history::EndReason::Showdown => showdowns += 1,
            poker_engine::hand_history::EndReason::AllFolded => folds_win += 1,
            _ => {}
        }

        for p in &gl.state.players {
            let payout = resolution.payouts.get(&p.id).copied().unwrap_or(0);
            let new_stack = p.stack + payout;
            if let Some((_, s)) = stacks.iter_mut().find(|(id, _)| id == &p.id) {
                *s = new_stack;
            }
        }

        let chips_after: u64 = stacks.iter().map(|(_, s)| *s).sum();
        assert_eq!(
            chips_after + resolution.rake,
            chips_before,
            "conservação SD falhou mão {hand_idx}: before={chips_before} after={chips_after} rake={}",
            resolution.rake
        );

        hands_ok += 1;
        if hand_idx > 0 && hand_idx % 10_000 == 0 {
            eprintln!(
                "SD progress {hand_idx}/{HANDS} elapsed={:.1}s rake={total_rake} showdowns={showdowns}",
                t0.elapsed().as_secs_f64()
            );
        }
    }

    let elapsed = t0.elapsed();
    eprintln!(
        "SD DONE hands={hands_ok} elapsed={elapsed:?} hps={:.1} showdowns={showdowns} fold_wins={folds_win} rake={total_rake} rebuys={rebuy_events} low_card_violations={dealt_low_card_violations}",
        hands_ok as f64 / elapsed.as_secs_f64().max(0.001)
    );

    assert_eq!(hands_ok, HANDS);
    assert_eq!(dealt_low_card_violations, 0, "cartas 2–5 foram dealadas");
    assert!(showdowns > 0, "esperava showdowns em 100k mãos");
    // Bot de stress prioriza call/check; wins por fold podem ser raros — só reportamos.
    eprintln!("fold_wins observed={folds_win} (não é requisito rígido do stress SD)");
}

#[test]
fn short_deck_heads_up_and_full_ring_smoke() {
    // Smoke: 2 e 6 jogadores com variante SD concluem mãos sem panic
    for players in [2usize, 3, 6] {
        let config = TableConfig::new(200, 500, 1000).with_poker_variant(PokerVariant::ShortDeck);
        let mut gl = GameLoop::new(
            config,
            format!("smoke-{players}"),
            "SD smoke".into(),
            GameType::Cash,
        );
        for i in 0..players {
            gl.add_player(format!("p{i}"), 10_000);
        }
        gl.set_dealer(0);
        gl.start_hand().unwrap();
        auto_play_until_finished(&mut gl, 200);
        let res = gl.resolve_hand().unwrap();
        let chips: u64 = gl.state.players.iter().map(|p| p.stack).sum::<u64>()
            + res.payouts.values().sum::<u64>();
        assert_eq!(chips + res.rake, players as u64 * 10_000);
    }
}
