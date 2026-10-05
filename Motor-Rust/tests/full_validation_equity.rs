//! Independent runout enumeration vs the production equity estimator.
//! Ranking is shared with the separately checked deterministic rank fixtures.
use poker_engine::deck::{compare_hands, create_deck, evaluate_hand, Card, Rank, Suit};
use poker_engine::loss_deflator::get_multiway_win_probability_for_variant;
use poker_engine::types::PokerVariant;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde_json::json;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
fn cards(s: &str) -> Vec<Card> {
    s.split_whitespace()
        .map(|s| Card {
            rank: match s.as_bytes()[0] {
                b'A' => Rank::Ace,
                b'K' => Rank::King,
                b'Q' => Rank::Queen,
                b'J' => Rank::Jack,
                b'T' => Rank::Ten,
                n => Rank::try_from(n - b'0').unwrap(),
            },
            suit: match s.as_bytes()[1] {
                b'h' => Suit::Hearts,
                b'd' => Suit::Diamonds,
                b'c' => Suit::Clubs,
                _ => Suit::Spades,
            },
        })
        .collect()
}
fn exact(hands: &[Vec<Card>], board: &[Card]) -> (f64, u64) {
    let known: Vec<_> = hands.iter().flatten().chain(board).copied().collect();
    let deck: Vec<_> = create_deck()
        .into_iter()
        .filter(|c| !known.contains(c))
        .collect();
    fn visit(
        deck: &[Card],
        start: usize,
        board: &mut Vec<Card>,
        hands: &[Vec<Card>],
        sum: &mut f64,
        total: &mut u64,
    ) {
        if board.len() == 5 {
            let values: Vec<_> = hands.iter().map(|h| evaluate_hand(h, board)).collect();
            let hero = &values[0];
            if !values
                .iter()
                .skip(1)
                .any(|v| compare_hands(v, hero) == Ordering::Greater)
            {
                let tied = values
                    .iter()
                    .filter(|v| compare_hands(v, hero) == Ordering::Equal)
                    .count();
                *sum += 1.0 / tied as f64;
            }
            *total += 1;
            return;
        }
        for i in start..=deck.len() - (5 - board.len()) {
            board.push(deck[i]);
            visit(deck, i + 1, board, hands, sum, total);
            board.pop();
        }
    }
    let (mut sum, mut count) = (0.0, 0);
    visit(&deck, 0, &mut board.to_vec(), hands, &mut sum, &mut count);
    (sum / count as f64, count)
}
#[test]
#[ignore = "authorized equity precision campaign"]
fn equity_accuracy_and_uncertainty() {
    assert_eq!(
        std::env::var("FULL_VALIDATION_APPROVED").as_deref(),
        Ok("1")
    );
    let dir = PathBuf::from(std::env::var("FULL_VALIDATION_REPORT_DIR").unwrap());
    let mut log = File::create(dir.join("equity-progress.jsonl")).unwrap();
    let cases = [
        ("dominant_pair", vec!["As Ah", "Ks Kh"], "", None),
        ("coin_flip", vec!["2c 2d", "As Kh"], "", None),
        ("dominated", vec!["As Qh", "Ac Kd"], "", None),
        ("multiway", vec!["As Ah", "Ks Kh", "Qc Qd"], "", None),
        ("draw_flop", vec!["Ah Kh", "Qc Qd"], "Jh Th 2c", None),
        (
            "turn_known_36_of_44",
            vec!["Ad Ac", "Jh Th"],
            "Qs 9c 4d 2s",
            Some(36.0 / 44.0),
        ),
        (
            "board_tie",
            vec!["9c 9d", "8c 8d", "7c 7d"],
            "Ah Kh Qh Jh Th",
            Some(1.0 / 3.0),
        ),
    ];
    let mut results = Vec::new();
    for (name, hole, board, known) in cases {
        let hands: Vec<_> = hole.iter().map(|s| cards(s)).collect();
        let board = cards(board);
        let opponents: Vec<&[Card]> = hands.iter().skip(1).map(Vec::as_slice).collect();
        writeln!(
            log,
            "{}",
            json!({"event":"start","case":name,"hands":hands,"board":board})
        )
        .unwrap();
        log.flush().unwrap();
        let estimate = get_multiway_win_probability_for_variant(
            &hands[0],
            &opponents,
            &board,
            PokerVariant::Holdem,
        )
        .unwrap();
        // Exactly one repeat, solely to check determinism, not counted as a new sample.
        if name == "dominant_pair" {
            assert_eq!(
                estimate,
                get_multiway_win_probability_for_variant(
                    &hands[0],
                    &opponents,
                    &board,
                    PokerVariant::Holdem
                )
                .unwrap()
            );
        }
        let (reference, population) = exact(&hands, &board);
        if let Some(expected) = known {
            assert!((reference - expected).abs() < 1e-12);
        }
        let samples = population.min(500_000);
        // Two-sided Hoeffding bound for bounded [0,1] pot shares, also valid
        // for sampling without replacement. Conservative 99%, no normal approximation.
        let radius = if samples == population {
            0.0
        } else {
            (200.0f64.ln() / (2.0 * samples as f64)).sqrt()
        };
        let boundary = [0.56, 0.66, 0.76, 0.86]
            .iter()
            .any(|b| (estimate - b).abs() <= radius);
        let error = (estimate - reference).abs();
        let result = json!({"case":name,"estimate":estimate,"reference":reference,
            "absolute_error_pp":error*100.0,"population":population,"samples":samples,
            "ci99":[(estimate-radius).max(0.0),(estimate+radius).min(1.0)],
            "ci_method":"Hoeffding, fixed deterministic pseudorandom sample; confidence is a design bound",
            "tier_status":if boundary {"inconclusive"}else{"resolved"},
            "tolerance_pp":0.5,"oracle":"exhaustive runouts; rank evaluator shared and checked by rank regressions"});
        writeln!(log, "{result}").unwrap();
        log.flush().unwrap();
        results.push(result);
        fs::write(
            dir.join("equity.json"),
            serde_json::to_vec_pretty(&results).unwrap(),
        )
        .unwrap();
        assert!(error <= 0.005, "equity error exceeds 0.5 pp for {name}");
        assert!(
            error <= radius + 1e-10,
            "reference outside 99% bound for {name}"
        );
    }
    // Explicit threshold neighbours on exact turn inputs; systematically select
    // witnesses from independent legal boards rather than altering the financial tiers.
    // Flop references have enough distinct runouts to witness each threshold;
    // contiguous turn cards cannot in general approach 56%/76% within 0.5 pp.
    type Witness = (f64, Vec<Vec<Card>>, Vec<Card>, f64);
    let mut rng = StdRng::seed_from_u64(0x261002);
    let mut nearest: [Option<Witness>; 4] = [None, None, None, None];
    for _ in 0..1000 {
        let mut deck = create_deck();
        deck.shuffle(&mut rng);
        let hands = vec![deck[..2].to_vec(), deck[2..4].to_vec()];
        let board = deck[4..7].to_vec();
        let (value, _) = exact(&hands, &board);
        for (j, threshold) in [0.56, 0.66, 0.76, 0.86].iter().enumerate() {
            let distance = (value - threshold).abs();
            if nearest[j].as_ref().is_none_or(|old| distance < old.0) {
                nearest[j] = Some((distance, hands.clone(), board.clone(), value));
            }
        }
        if nearest
            .iter()
            .all(|w| w.as_ref().is_some_and(|w| w.0 <= 0.005))
        {
            break;
        }
    }
    let mut thresholds = Vec::new();
    for (threshold, witness) in [0.56, 0.66, 0.76, 0.86].iter().zip(nearest) {
        let (distance, hands, board, reference) = witness.unwrap();
        let estimate = get_multiway_win_probability_for_variant(
            &hands[0],
            &[&hands[1]],
            &board,
            PokerVariant::Holdem,
        )
        .unwrap();
        assert!((estimate - reference).abs() < 1e-10);
        thresholds.push(
            json!({"threshold":threshold,"distance":distance,"board":board,"hands":hands,
            "equity":estimate,"status":if distance<=0.005 {"covered"}else{"gap"},
            "precision":"exact flop enumeration; deterministic witness selection"}),
        );
    }
    fs::write(
        dir.join("equity-thresholds.json"),
        serde_json::to_vec_pretty(&thresholds).unwrap(),
    )
    .unwrap();
}
