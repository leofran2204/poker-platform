//! 10.000 mãos por configuração do catálogo cash oficial.
//!
//! Configs (blinds/frente/max alinhados à produção):
//! - NLHE 0,25/0,25 · 9-max · frente R$25
//! - NLHE 0,25/0,50 · 9-max · frente R$50
//! - Short Deck 0,50/0,50 · 6-max · frente R$75
//! - SD Omaha 0,50/1 · 4-max · frente R$100
//!
//! Rodar:
//!   cargo test --test cash_catalog_10k_hands -- --nocapture

use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::{PokerVariant, TableConfig};
use std::time::Instant;

const HANDS: u32 = 10_000;

struct CatalogTable {
    name: &'static str,
    small_blind: u64,
    big_blind: u64,
    max_players: usize,
    starting_stack: u64,
    rake_bps: u16,
    rake_cap: u64,
    variant: PokerVariant,
}

const CATALOG: &[CatalogTable] = &[
    CatalogTable {
        name: "NLHE 0,25/0,25",
        small_blind: 25,
        big_blind: 25,
        max_players: 9,
        starting_stack: 2_500,
        rake_bps: 500,
        rake_cap: 250,
        variant: PokerVariant::Holdem,
    },
    CatalogTable {
        name: "NLHE 0,25/0,50",
        small_blind: 25,
        big_blind: 50,
        max_players: 9,
        starting_stack: 5_000,
        rake_bps: 500,
        rake_cap: 250,
        variant: PokerVariant::Holdem,
    },
    CatalogTable {
        name: "SD 0,50/0,50",
        small_blind: 50,
        big_blind: 50,
        max_players: 6,
        starting_stack: 7_500,
        rake_bps: 500,
        rake_cap: 500,
        variant: PokerVariant::ShortDeck,
    },
    CatalogTable {
        name: "SD Omaha 0,50/1",
        small_blind: 50,
        big_blind: 100,
        max_players: 4,
        starting_stack: 10_000,
        rake_bps: 500,
        rake_cap: 1_000,
        variant: PokerVariant::ShortDeckOmaha,
    },
];

fn auto_play(gl: &mut GameLoop, big_blind: u64) {
    let mut steps = 0u32;
    while !gl.state.is_finished && steps < 3_000 {
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
        let roll = steps.wrapping_mul(37).wrapping_add(active.len() as u32) % 100;
        let mv = if to_call == 0 {
            if roll > 93 {
                PlayerMove::Raise(big_blind * 2)
            } else {
                PlayerMove::Check
            }
        } else if roll < 12 {
            PlayerMove::Fold
        } else if roll > 98 {
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
    assert!(
        gl.state.is_finished,
        "mão não terminou steps={steps} phase={:?}",
        gl.state.phase
    );
}

fn run_catalog_table(cfg: &CatalogTable) {
    let config = TableConfig::new(cfg.big_blind, cfg.rake_bps, cfg.rake_cap)
        .with_small_blind(cfg.small_blind)
        .with_poker_variant(cfg.variant);

    let hole_expected = cfg.variant.hole_card_count();
    let short = cfg.variant.uses_short_deck();

    let mut stacks: Vec<(String, u64)> = (0..cfg.max_players)
        .map(|i| (format!("p{i}"), cfg.starting_stack))
        .collect();

    let mut total_rake = 0u64;
    let mut showdowns = 0u32;
    let mut fold_wins = 0u32;
    let mut rebuys = 0u32;
    let t0 = Instant::now();

    for hand_idx in 0..HANDS {
        for (_, s) in stacks.iter_mut() {
            if *s < cfg.big_blind * 10 {
                *s = cfg.starting_stack;
                rebuys += 1;
            }
        }
        let before: u64 = stacks.iter().map(|(_, s)| *s).sum();

        let mut gl = GameLoop::new(
            config.clone(),
            format!("{}-{hand_idx}", cfg.name),
            cfg.name.to_string(),
            GameType::Cash,
        )
        .with_skip_loss_deflator(true);
        for (id, stack) in &stacks {
            gl.add_player(id.clone(), *stack);
        }
        gl.set_dealer((hand_idx as usize) % cfg.max_players);
        gl.start_hand()
            .unwrap_or_else(|e| panic!("{} start_hand: {e:?}", cfg.name));

        assert_eq!(gl.state.small_blind, cfg.small_blind, "{} SB", cfg.name);
        assert_eq!(gl.state.big_blind, cfg.big_blind, "{} BB", cfg.name);

        for p in &gl.state.players {
            assert_eq!(
                p.hole_cards.len(),
                hole_expected,
                "{} hole cards",
                cfg.name
            );
            if short {
                for c in &p.hole_cards {
                    assert!(
                        (c.rank as u8) >= 6,
                        "{} carta baixa no hole {:?}",
                        cfg.name,
                        c
                    );
                }
            }
        }

        auto_play(&mut gl, cfg.big_blind);

        if short {
            for c in &gl.state.community_cards {
                assert!(
                    (c.rank as u8) >= 6,
                    "{} carta baixa no board {:?}",
                    cfg.name,
                    c
                );
            }
        }

        let res = gl
            .resolve_hand()
            .unwrap_or_else(|e| panic!("{} resolve: {e:?}", cfg.name));
        total_rake += res.rake;
        let platform = (res.rake * 15) / 100;
        let club = res.rake.saturating_sub(platform);
        assert_eq!(platform + club, res.rake, "{} B2B split", cfg.name);

        match res.end_reason {
            poker_engine::hand_history::EndReason::Showdown => showdowns += 1,
            poker_engine::hand_history::EndReason::AllFolded => fold_wins += 1,
            _ => {}
        }

        for p in &gl.state.players {
            let pay = res.payouts.get(&p.id).copied().unwrap_or(0);
            if let Some((_, s)) = stacks.iter_mut().find(|(id, _)| id == &p.id) {
                *s = p.stack + pay;
            }
        }
        let after: u64 = stacks.iter().map(|(_, s)| *s).sum();
        assert_eq!(
            after + res.rake,
            before,
            "{} conservação mão {hand_idx}: before={before} after={after} rake={}",
            cfg.name,
            res.rake
        );

        if hand_idx > 0 && hand_idx % 2_500 == 0 {
            eprintln!(
                "[{}] progress {hand_idx}/{HANDS} elapsed={:.1}s",
                cfg.name,
                t0.elapsed().as_secs_f64()
            );
        }
    }

    let elapsed = t0.elapsed();
    eprintln!(
        "[{}] DONE hands={HANDS} elapsed={elapsed:?} hps={:.1} showdowns={showdowns} folds={fold_wins} rake={total_rake} rebuys={rebuys}",
        cfg.name,
        HANDS as f64 / elapsed.as_secs_f64().max(0.001)
    );
}

#[test]
fn cash_catalog_ten_thousand_hands_each_table() {
    for cfg in CATALOG {
        eprintln!("=== START {} ===", cfg.name);
        run_catalog_table(cfg);
    }
}
