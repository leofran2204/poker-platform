//! Brazilian Pineapple: simulação de referência (6 jogadores até o showdown).
//!
//! Mede a força das mãos vencedoras contra Hold'em e Omaha 4 com a mesma
//! política passiva, a frequência de trocas da heurística pública e o número
//! de decisões por mão. Também confere baralho, cartas e fichas em cada mão.
//! Lento em debug: rode com `--release --features massive-tests -- --nocapture`.
use poker_engine::deck::{
    create_deck, evaluate_hand, evaluate_hand_omaha, evaluate_hand_pineapple_draw, Card, HandRank,
    HandResult,
};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::pineapple_draw::suggested_swap;
use poker_engine::types::{PokerVariant, TableConfig};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

const PLAYERS: usize = 6;

struct Report {
    winners: [u64; 11],
    hands: u64,
    decisions: u64,
    draw_decisions: u64,
    swaps: u64,
}

fn evaluate(variant: PokerVariant, hole: &[Card], board: &[Card]) -> HandResult {
    match variant {
        PokerVariant::Omaha => evaluate_hand_omaha(hole, board),
        PokerVariant::PineappleDraw => evaluate_hand_pineapple_draw(hole, board),
        _ => evaluate_hand(hole, board),
    }
}

fn simulate(variant: PokerVariant, hands: u64, seed: u64) -> Report {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut report = Report {
        winners: [0; 11],
        hands,
        decisions: 0,
        draw_decisions: 0,
        swaps: 0,
    };
    for hand in 0..hands {
        let mut game = GameLoop::new(
            TableConfig::new(100, 0, 0)
                .with_small_blind(50)
                .with_poker_variant(variant),
            format!("sim-{hand}"),
            "Simulação".into(),
            GameType::Cash,
        )
        .with_skip_loss_deflator(true);
        for seat in 0..PLAYERS {
            game.add_player(format!("p{seat}"), 1_000_000);
        }
        game.set_dealer((hand % PLAYERS as u64) as usize);
        let mut deck = create_deck();
        deck.shuffle(&mut rng);
        game.start_hand_with_deck(deck).unwrap();
        while !game.state.is_finished {
            let player = game.state.active_player().unwrap();
            let id = player.id.clone();
            let mv = if game.state.discarding {
                PlayerMove::Discard(None)
            } else if game.state.drawing {
                report.draw_decisions += 1;
                match suggested_swap(&player.hole_cards, &game.state.community_cards) {
                    Some(position) => {
                        report.swaps += 1;
                        PlayerMove::Swap(position)
                    }
                    None => PlayerMove::Keep,
                }
            } else if game.legal_actions(&id).allows("check") {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            };
            report.decisions += 1;
            game.player_action(&id, mv).unwrap();
        }
        // Cartas únicas e completas: privadas, mesa, queimadas, descartes, baralho.
        let mut seen: Vec<Card> = game.state.community_cards.clone();
        seen.extend(&game.state.burn_pile);
        seen.extend(&game.state.discard_pile);
        seen.extend(&game.state.deck);
        for p in &game.state.players {
            // Brazilian Pineapple: três privadas no deal, duas depois do descarte do river.
            let expected = if variant == PokerVariant::PineappleDraw {
                2
            } else {
                variant.hole_card_count()
            };
            assert_eq!(p.hole_cards.len(), expected);
            seen.extend(&p.hole_cards);
        }
        assert_eq!(seen.len(), 52);
        let best = game
            .state
            .players
            .iter()
            .map(|p| evaluate(variant, &p.hole_cards, &game.state.community_cards))
            .max_by(poker_engine::deck::compare_hands)
            .unwrap();
        report.winners[best.rank as usize] += 1;
        let resolution = game.resolve_hand().unwrap();
        let stacks: u64 = game.state.players.iter().map(|p| p.stack).sum();
        assert_eq!(
            stacks + resolution.payouts.values().sum::<u64>() + resolution.rake,
            PLAYERS as u64 * 1_000_000
        );
    }
    report
}

fn print(name: &str, report: &Report) {
    let pct = |count: u64| 100.0 * count as f64 / report.hands as f64;
    let at_least = |rank: HandRank| pct(report.winners[rank as usize..].iter().sum::<u64>());
    println!(
        "{name}: vencedora sequência+ {:.1}% · flush+ {:.1}% · full house+ {:.1}% · \
         trinca ou menos {:.1}% · decisões/mão {:.1}",
        at_least(HandRank::Straight),
        at_least(HandRank::Flush),
        at_least(HandRank::FullHouse),
        100.0 - at_least(HandRank::Straight),
        report.decisions as f64 / report.hands as f64,
    );
    if report.draw_decisions > 0 {
        println!(
            "{name}: trocas {:.1}% das decisões de troca · {:.2} trocas por jogador por mão",
            100.0 * report.swaps as f64 / report.draw_decisions as f64,
            report.swaps as f64 / (report.hands * PLAYERS as u64) as f64,
        );
    }
}

#[test]
#[cfg_attr(
    not(feature = "massive-tests"),
    ignore = "simulação de referência; habilite a feature massive-tests"
)]
fn pineapple_draw_reference_simulation() {
    let hands = 20_000;
    let holdem = simulate(PokerVariant::Holdem, hands, 11);
    let omaha = simulate(PokerVariant::Omaha, hands, 12);
    let draw = simulate(PokerVariant::PineappleDraw, hands, 13);
    print("Hold'em", &holdem);
    print("Omaha 4", &omaha);
    print("Brazilian Pineapple", &draw);
    // Referência medida em 11/11/2026, com troca depois das apostas do flop e
    // do turn e descarte no river (20 mil mãos por modalidade, sementes fixas):
    // vencedora sequência ou melhor em 40,1% (Hold'em), 78,7% (Omaha 4) e 73,0%
    // (Brazilian Pineapple); flush ou melhor em 22,9%, 52,4% e 55,0%. Três
    // privadas com trocas ficam perto do Omaha, bem acima do Hold'em.
    let strong = |r: &Report| r.winners[HandRank::Straight as usize..].iter().sum::<u64>();
    assert!(strong(&draw) > strong(&holdem));
    assert!(draw.swaps > 0);
}

#[test]
fn pineapple_draw_quick_simulation_keeps_cards_and_chips_consistent() {
    let report = simulate(PokerVariant::PineappleDraw, 300, 99);
    assert_eq!(report.hands, 300);
    assert!(report.draw_decisions > 0);
    assert!(report.swaps > 0 && report.swaps < report.draw_decisions);
}
