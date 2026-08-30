//! Suíte massiva dedicada ao Omaha Short Deck 4-max.
//!
//! Cobertura:
//! - 100.000 avaliações aleatórias comparadas com uma referência 2+3;
//! - 100.000 mãos de torneio 4-max no `GameLoop`;
//! - somente cartas 6–A e quatro hole cards por jogador;
//! - leitura independente dos vencedores, inclusive side pots e centavo ímpar;
//! - conservação exata das fichas e ausência de rake/deflator na mão de torneio.
//!
//! Rodar:
//!   cargo test --release --test short_deck_omaha_massive -- --nocapture

use poker_engine::deck::{
    compare_hands, create_short_deck, evaluate_hand_short_deck, evaluate_hand_short_deck_omaha,
    Card, HandResult,
};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::{EndReason, GameType};
use poker_engine::types::{PokerVariant, Pot, TableConfig};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const RANDOM_EVALUATIONS: u32 = 100_000;
const TOURNAMENT_HANDS: u32 = 100_000;
const PLAYERS: usize = 4;
const STARTING_STACK: u64 = 10_000;
const SMALL_BLIND: u64 = 50;
const BIG_BLIND: u64 = 100;

/// Referência deliberadamente explícita da regra Omaha: escolhe exatamente
/// duas das quatro hole cards e exatamente três das cinco cartas do board.
fn reference_omaha_short_deck(hole: &[Card], board: &[Card]) -> HandResult {
    assert_eq!(hole.len(), 4, "Omaha Short Deck exige quatro hole cards");
    assert_eq!(board.len(), 5, "showdown exige board completo");

    let mut best: Option<HandResult> = None;
    for first_hole in 0..hole.len() {
        for second_hole in (first_hole + 1)..hole.len() {
            let selected_hole = [hole[first_hole], hole[second_hole]];
            for first_board in 0..board.len() {
                for second_board in (first_board + 1)..board.len() {
                    for third_board in (second_board + 1)..board.len() {
                        let selected_board =
                            [board[first_board], board[second_board], board[third_board]];
                        let candidate = evaluate_hand_short_deck(&selected_hole, &selected_board);
                        if best.as_ref().is_none_or(|current| {
                            compare_hands(&candidate, current) == Ordering::Greater
                        }) {
                            best = Some(candidate);
                        }
                    }
                }
            }
        }
    }

    best.expect("há 60 combinações válidas em quatro hole cards e cinco cartas do board")
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

#[test]
fn short_deck_omaha_one_hundred_thousand_evaluations_match_reference() {
    let mut rng = StdRng::seed_from_u64(0x005D_0AA4_2026);
    let mut rank_histogram = HashMap::new();
    let started = Instant::now();

    for iteration in 0..RANDOM_EVALUATIONS {
        let mut deck = create_short_deck();
        deck.shuffle(&mut rng);
        let hole = &deck[..4];
        let board = &deck[4..9];

        let actual = evaluate_hand_short_deck_omaha(hole, board);
        let expected = reference_omaha_short_deck(hole, board);
        assert_same_hand(&actual, &expected, &format!("avaliação {iteration}"));
        *rank_histogram.entry(actual.value).or_insert(0u64) += 1;

        assert!(
            hole.iter().chain(board).all(|card| (card.rank as u8) >= 6),
            "carta abaixo de seis na avaliação {iteration}"
        );
    }

    let elapsed = started.elapsed();
    eprintln!(
        "SD Omaha eval DONE evaluations={RANDOM_EVALUATIONS} elapsed={elapsed:?} eps={:.1} ranks={rank_histogram:?}",
        RANDOM_EVALUATIONS as f64 / elapsed.as_secs_f64().max(0.001)
    );
    assert!(
        rank_histogram.len() >= 5,
        "amostra massiva deveria produzir pelo menos cinco classes de mãos"
    );
}

fn auto_play_until_finished(game: &mut GameLoop, hand_index: u32) {
    let mut steps = 0u32;
    while !game.state.is_finished && steps < 3_000 {
        steps += 1;
        let Some(active_player) = game.state.active_player().map(|player| player.id.clone()) else {
            break;
        };
        let player = game
            .state
            .players
            .iter()
            .find(|player| player.id == active_player)
            .expect("jogador ativo existe");
        let to_call = game
            .state
            .current_bet_to_match
            .saturating_sub(player.current_bet);
        let roll = steps
            .wrapping_mul(37)
            .wrapping_add(hand_index.wrapping_mul(17))
            .wrapping_add(active_player.len() as u32)
            % 100;

        let requested_move = if to_call == 0 {
            if roll > 92 {
                PlayerMove::Raise(BIG_BLIND * 2)
            } else {
                PlayerMove::Check
            }
        } else if roll < 12 {
            PlayerMove::Fold
        } else if roll > 96 {
            PlayerMove::AllIn
        } else {
            PlayerMove::Call
        };

        if game.player_action(&active_player, requested_move).is_err() {
            let fallback = if to_call == 0 {
                PlayerMove::Check
            } else {
                PlayerMove::Call
            };
            if game.player_action(&active_player, fallback).is_err() {
                let _ = game.player_action(&active_player, PlayerMove::Fold);
            }
        }
    }

    assert!(
        game.state.is_finished,
        "mão Omaha Short Deck não terminou: hand={hand_index} steps={steps} phase={:?}",
        game.state.phase
    );
}

fn reference_showdown_payouts(game: &GameLoop, pots: &[Pot]) -> HashMap<String, u64> {
    let mut expected = HashMap::new();
    let seat_order: Vec<String> = (1..=game.state.players.len())
        .map(|offset| {
            let seat = (game.state.dealer_index + offset) % game.state.players.len();
            game.state.players[seat].id.clone()
        })
        .collect();

    for pot in pots {
        let mut winners: Vec<String> = Vec::new();
        let mut best: Option<HandResult> = None;

        for player_id in &pot.eligible_players {
            let player = game
                .state
                .players
                .iter()
                .find(|player| &player.id == player_id)
                .expect("jogador elegível existe");
            if player.has_folded {
                continue;
            }

            let hand = reference_omaha_short_deck(&player.hole_cards, &game.state.community_cards);
            match best.as_ref().map(|current| compare_hands(&hand, current)) {
                None | Some(Ordering::Greater) => {
                    best = Some(hand);
                    winners.clear();
                    winners.push(player.id.clone());
                }
                Some(Ordering::Equal) => winners.push(player.id.clone()),
                Some(Ordering::Less) => {}
            }
        }

        assert!(
            !winners.is_empty(),
            "pote de showdown sem vencedor elegível"
        );
        let base_share = pot.amount / winners.len() as u64;
        let odd_chips = pot.amount % winners.len() as u64;
        for winner in &winners {
            *expected.entry(winner.clone()).or_insert(0) += base_share;
        }

        let ordered_winners: Vec<&String> = seat_order
            .iter()
            .filter(|player_id| winners.contains(player_id))
            .collect();
        for winner in ordered_winners.into_iter().take(odd_chips as usize) {
            *expected.entry(winner.clone()).or_insert(0) += 1;
        }
    }

    expected
}

#[test]
fn short_deck_omaha_one_hundred_thousand_tournament_hands_are_exact() {
    let config = TableConfig::new(BIG_BLIND, 0, 0)
        .with_small_blind(SMALL_BLIND)
        .with_poker_variant(PokerVariant::ShortDeckOmaha);
    let mut stacks: Vec<(String, u64)> = (0..PLAYERS)
        .map(|seat| (format!("sdo{seat}"), STARTING_STACK))
        .collect();
    let mut showdowns = 0u32;
    let mut fold_wins = 0u32;
    let mut side_pot_hands = 0u32;
    let mut reentries = 0u32;
    let started = Instant::now();

    for hand_index in 0..TOURNAMENT_HANDS {
        for (_, stack) in &mut stacks {
            if *stack < BIG_BLIND * 10 {
                *stack = STARTING_STACK;
                reentries += 1;
            }
        }
        let chips_before: u64 = stacks.iter().map(|(_, stack)| *stack).sum();

        let mut game = GameLoop::new(
            config.clone(),
            format!("sdo-tournament-{hand_index}"),
            "Omaha Short Deck 100 GTD 4-max".to_string(),
            GameType::Tournament,
        );
        for (player_id, stack) in &stacks {
            game.add_player(player_id.clone(), *stack);
        }
        game.set_dealer((hand_index as usize) % PLAYERS);
        game.start_hand().expect("iniciar mão Omaha Short Deck");

        let mut dealt_cards = HashSet::new();
        for player in &game.state.players {
            assert_eq!(
                player.hole_cards.len(),
                4,
                "quantidade de hole cards na mão {hand_index}"
            );
            for card in &player.hole_cards {
                assert!((card.rank as u8) >= 6, "carta baixa no hole: {card:?}");
                assert!(
                    dealt_cards.insert((card.rank, card.suit)),
                    "carta duplicada no hole: {card:?}"
                );
            }
        }

        auto_play_until_finished(&mut game, hand_index);
        for card in &game.state.community_cards {
            assert!((card.rank as u8) >= 6, "carta baixa no board: {card:?}");
            assert!(
                dealt_cards.insert((card.rank, card.suit)),
                "carta duplicada entre hole e board: {card:?}"
            );
        }

        let resolution = game.resolve_hand().expect("resolver mão Omaha Short Deck");
        assert_eq!(resolution.rake, 0, "torneio não deve cobrar rake por mão");
        assert!(
            resolution.loss_deflator.is_none() && resolution.loss_deflators.is_empty(),
            "Omaha não deve aplicar o loss deflator de Hold'em"
        );

        match resolution.end_reason {
            EndReason::Showdown => {
                showdowns += 1;
                assert_eq!(game.state.community_cards.len(), 5);
                assert_eq!(
                    resolution.payouts,
                    reference_showdown_payouts(&game, &resolution.pots),
                    "pagamento divergente na mão {hand_index}"
                );
                for result in &resolution.player_results {
                    if result.folded {
                        continue;
                    }
                    let player = game
                        .state
                        .players
                        .iter()
                        .find(|player| player.id == result.player_id)
                        .expect("jogador do resultado existe");
                    let expected =
                        reference_omaha_short_deck(&player.hole_cards, &game.state.community_cards);
                    let actual = result.best_hand.as_ref().expect("showdown tem best hand");
                    assert_same_hand(actual, &expected, &format!("mão {hand_index}"));
                }
            }
            EndReason::AllFolded => {
                fold_wins += 1;
                let survivor = game
                    .state
                    .players
                    .iter()
                    .find(|player| player.is_in_hand())
                    .expect("vitória por fold tem sobrevivente");
                let total_pot: u64 = resolution.pots.iter().map(|pot| pot.amount).sum();
                assert_eq!(resolution.payouts.len(), 1);
                assert_eq!(resolution.payouts.get(&survivor.id), Some(&total_pot));
            }
            other => panic!("fim inesperado na mão {hand_index}: {other:?}"),
        }
        if resolution.pots.len() > 1 {
            side_pot_hands += 1;
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
            chips_after, chips_before,
            "conservação de fichas falhou na mão {hand_index}"
        );

        if hand_index > 0 && hand_index % 10_000 == 0 {
            eprintln!(
                "SD Omaha progress {hand_index}/{TOURNAMENT_HANDS} elapsed={:.1}s showdowns={showdowns} side_pots={side_pot_hands}",
                started.elapsed().as_secs_f64()
            );
        }
    }

    let elapsed = started.elapsed();
    eprintln!(
        "SD Omaha DONE hands={TOURNAMENT_HANDS} elapsed={elapsed:?} hps={:.1} showdowns={showdowns} fold_wins={fold_wins} side_pot_hands={side_pot_hands} reentries={reentries}",
        TOURNAMENT_HANDS as f64 / elapsed.as_secs_f64().max(0.001)
    );
    assert!(showdowns > 0, "stress deveria alcançar showdowns");
    assert!(side_pot_hands > 0, "stress deveria alcançar side pots");
}
