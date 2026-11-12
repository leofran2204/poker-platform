//! Roteiros dos vídeos do Brazilian Pineapple (filme de regras, trailer e aulas
//! m5l3–m5l5) conferidos contra o motor: cartas sem repetição, apostas, trocas e
//! descartes na ordem e nos limites do jogo, stacks e pote de cada cena, mãos
//! destacadas e vencedores.
use poker_engine::deck::{
    compare_hands, create_deck, evaluate_hand, evaluate_hand_pineapple_draw,
    pineapple_draw_best_discard, Card, Rank, Suit,
};
use poker_engine::game_loop::{GameLoop, PlayerMove};
use poker_engine::hand_history::GameType;
use poker_engine::types::{GamePhase, PokerVariant, TableConfig};
use serde_json::Value;
use std::cmp::Ordering;

fn scripts() -> Vec<Value> {
    [
        include_str!("../../ZeroTiltCurso/editorial/motion/src/data/pineapple-draw-rules.json"),
        include_str!("../../ZeroTiltCurso/editorial/motion/src/data/pineapple-draw-trailer.json"),
        include_str!("../../ZeroTiltCurso/editorial/motion/src/data/academy-m5l3.json"),
        include_str!("../../ZeroTiltCurso/editorial/motion/src/data/academy-m5l4.json"),
        include_str!("../../ZeroTiltCurso/editorial/motion/src/data/academy-m5l5.json"),
    ]
    .iter()
    .map(|raw| serde_json::from_str(raw).unwrap())
    .collect()
}

fn hand_scenes(script: &Value) -> Vec<Value> {
    script["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|scene| scene["visual"]["mode"] == "hand")
        .cloned()
        .collect()
}

fn card(code: &str) -> Card {
    let rank = match code.as_bytes()[0] {
        b'A' => Rank::Ace,
        b'K' => Rank::King,
        b'Q' => Rank::Queen,
        b'J' => Rank::Jack,
        b'T' => Rank::Ten,
        n => Rank::try_from(n - b'0').unwrap(),
    };
    let suit = match code.as_bytes()[1] {
        b'h' => Suit::Hearts,
        b'd' => Suit::Diamonds,
        b'c' => Suit::Clubs,
        b's' => Suit::Spades,
        _ => panic!("naipe desconhecido"),
    };
    Card { rank, suit }
}

fn cards(value: &Value) -> Vec<Card> {
    value
        .as_array()
        .map(|list| {
            list.iter()
                .map(|code| card(code.as_str().unwrap()))
                .collect()
        })
        .unwrap_or_default()
}

fn events(scene: &Value) -> Vec<Value> {
    scene["visual"]["events"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn index(value: &Value) -> usize {
    value.as_u64().unwrap() as usize
}

fn game() -> GameLoop {
    let mut game = GameLoop::new(
        TableConfig::new(100, 0, 0)
            .with_small_blind(50)
            .with_poker_variant(PokerVariant::PineappleDraw),
        "film".into(),
        "Film".into(),
        GameType::Cash,
    )
    .with_skip_loss_deflator(true);
    for i in 0..4 {
        game.add_player(format!("p{i}"), 10000);
    }
    // Duda (assento 3) é o botão: Ana paga o small blind e Beto o big blind.
    game.set_dealer(3);
    game.start_hand_with_deck(create_deck()).unwrap();
    game
}

/// Aplica no motor a ação de um evento do roteiro e confere que é a vez daquele jogador.
fn play(game: &mut GameLoop, event: &Value, scene: &str) {
    let kind = event["type"].as_str().unwrap();
    let Some(player) = event["player"].as_u64() else {
        return;
    };
    let id = format!("p{player}");
    let to = event["to"].as_u64().unwrap_or(0);
    let mv = match kind {
        "bet" => PlayerMove::Bet(to),
        "raise" => PlayerMove::Raise(to),
        "call" => PlayerMove::Call,
        "check" => PlayerMove::Check,
        "fold" => PlayerMove::Fold,
        "keep" => PlayerMove::Keep,
        "swap" => PlayerMove::Swap(index(&event["position"])),
        "discard" => PlayerMove::Discard(Some(index(&event["position"]))),
        _ => return,
    };
    assert_eq!(
        game.state.active_player().map(|p| p.id.clone()),
        Some(id.clone()),
        "{scene}: vez errada em {kind}"
    );
    game.player_action(&id, mv)
        .unwrap_or_else(|e| panic!("{scene}: {kind} de {id} recusado pelo motor: {e}"));
    if matches!(kind, "bet" | "raise" | "call") && !game.state.drawing {
        // O total do jogador na rodada é o que a tela mostra na frente dele
        // (zero quando a ação fechou a rodada e as fichas foram ao pote).
        let bet = game.state.players[player as usize].current_bet;
        assert!(bet == to || bet == 0, "{scene}: aposta {bet} x {to}");
    }
}

fn assert_table(game: &GameLoop, scene: &Value) {
    let id = scene["id"].as_str().unwrap();
    let v = &scene["visual"];
    let players = v["players"].as_array().unwrap();
    for (i, player) in players.iter().enumerate() {
        let engine = &game.state.players[i];
        assert_eq!(
            engine.stack,
            player["stack"].as_u64().unwrap(),
            "{id}: stack de p{i}"
        );
        assert_eq!(
            engine.current_bet,
            player["bet"].as_u64().unwrap_or(0),
            "{id}: aposta de p{i}"
        );
        assert_eq!(
            engine.has_folded,
            player["folded"].as_bool().unwrap_or(false),
            "{id}: fold de p{i}"
        );
    }
    let bets: u64 = game.state.players.iter().map(|p| p.current_bet).sum();
    assert_eq!(
        game.state.total_pot() - bets,
        v["pot"].as_u64().unwrap(),
        "{id}: pote no centro"
    );
    // O motor abre a próxima carta assim que a rodada fecha; no vídeo ela aparece
    // no começo da cena seguinte.
    assert!(
        game.state.community_cards.len() >= v["board"].as_array().unwrap().len(),
        "{id}: cartas na mesa"
    );
}

#[test]
fn every_hand_scene_uses_each_card_once() {
    for script in scripts() {
        for scene in hand_scenes(&script) {
            let v = &scene["visual"];
            let mut seen = cards(&v["board"]);
            for player in v["players"].as_array().unwrap() {
                seen.extend(cards(&player["cards"]));
            }
            for event in events(&scene) {
                seen.extend(cards(&event["cards"]));
                if let Some(code) = event["receive"].as_str() {
                    seen.push(card(code));
                }
            }
            for (i, c) in seen.iter().enumerate() {
                assert!(
                    !seen[..i].contains(c),
                    "{}: carta repetida {c:?}",
                    scene["id"]
                );
            }
        }
    }
}

/// A mão-modelo (trailer, filme de regras e aula 1) inteira no motor: mesma ordem
/// de ações, mesmos valores, troca só depois das apostas do flop e do turn e
/// descarte no river antes da última rodada.
#[test]
fn model_hand_is_legal_in_the_engine_from_deal_to_showdown() {
    let mut found = 0;
    for script in scripts() {
        let scenes = hand_scenes(&script);
        let Some(start) = scenes
            .iter()
            .position(|scene| events(scene).iter().any(|e| e["type"] == "deal"))
        else {
            continue;
        };
        found += 1;
        let mut game = game();
        let name = script["id"].as_str().unwrap();
        for (n, scene) in scenes[start..].iter().enumerate() {
            let id = format!("{name}/{}", scene["id"].as_str().unwrap());
            if n > 0 {
                assert_table(&game, scene);
            }
            for event in events(scene) {
                match event["type"].as_str().unwrap() {
                    // Blinds e ante são cobrados pelo motor no início da mão.
                    "post" => {
                        let player = index(&event["player"]);
                        assert_eq!(
                            game.state.players[player].current_bet,
                            event["to"].as_u64().unwrap(),
                            "{id}: blind"
                        );
                    }
                    "ante" => {
                        assert_eq!(
                            game.ante_paid(),
                            event["to"].as_u64().unwrap(),
                            "{id}: ante"
                        );
                        assert_eq!(game.ante_player_id(), Some("p1"), "{id}: ante do big blind");
                    }
                    "board" => {
                        let total =
                            cards(&scene["visual"]["board"]).len() + cards(&event["cards"]).len();
                        assert!(
                            game.state.community_cards.len() >= total,
                            "{id}: a mesa só abre depois das apostas e da troca"
                        );
                        if total == 5 {
                            assert!(game.state.discarding, "{id}: o river abre o descarte");
                        }
                    }
                    "swap" | "keep" => {
                        assert!(game.state.drawing, "{id}: troca só depois das apostas");
                        play(&mut game, &event, &id);
                    }
                    "discard" => {
                        assert!(game.state.discarding, "{id}: descarte só no river");
                        assert_eq!(game.state.phase, GamePhase::River);
                        play(&mut game, &event, &id);
                    }
                    _ => play(&mut game, &event, &id),
                }
            }
            if scene["id"].as_str().unwrap().ends_with("show") {
                break;
            }
        }
        assert!(game.state.is_finished, "{name}: a mão vai até o showdown");
        assert_eq!(game.state.total_pot(), 5800, "{name}: pote final");
        assert!(game
            .state
            .players
            .iter()
            .filter(|p| !p.has_folded)
            .all(|p| p.hole_cards.len() == 2));
    }
    assert_eq!(
        found, 3,
        "trailer, filme de regras e aula 1 mostram a mão-modelo"
    );
}

/// Cartas da mão-modelo: quem vence, as cinco cartas destacadas e os descartes.
#[test]
fn highlighted_hands_winners_and_discards_match_the_evaluator() {
    let mut highlights = 0;
    for script in scripts() {
        for scene in hand_scenes(&script) {
            let id = scene["id"].as_str().unwrap();
            let v = &scene["visual"];
            let mut board = cards(&v["board"]);
            let mut holes: Vec<Vec<Card>> = v["players"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| cards(&p["cards"]))
                .collect();
            let mut folded: Vec<bool> = v["players"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["folded"].as_bool().unwrap_or(false))
                .collect();
            for event in events(&scene) {
                match event["type"].as_str().unwrap() {
                    "board" => board.extend(cards(&event["cards"])),
                    "fold" => folded[index(&event["player"])] = true,
                    "swap" => {
                        let player = index(&event["player"]);
                        assert!(
                            matches!(board.len(), 3 | 4),
                            "{id}: troca no flop ou no turn"
                        );
                        assert_eq!(holes[player].len(), 3);
                        holes[player][index(&event["position"])] =
                            card(event["receive"].as_str().unwrap());
                    }
                    "discard" => {
                        let player = index(&event["player"]);
                        assert_eq!(board.len(), 5, "{id}: descarte com a mesa completa");
                        let position = index(&event["position"]);
                        // O jogador do vídeo fica sempre com a melhor dupla.
                        let best = evaluate_hand_pineapple_draw(&holes[player], &board);
                        let mut kept = holes[player].clone();
                        kept.remove(position);
                        assert_eq!(
                            compare_hands(&evaluate_hand_pineapple_draw(&kept, &board), &best),
                            Ordering::Equal,
                            "{id}: descarte de p{player} piora a mão"
                        );
                        assert!(pineapple_draw_best_discard(&holes[player], &board).is_some());
                        holes[player] = kept;
                    }
                    "highlight" => {
                        highlights += 1;
                        let picked_board: Vec<Card> = event["boardPick"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|i| board[index(i)])
                            .collect();
                        let Some(player) = event["player"].as_u64().map(|p| p as usize) else {
                            // Mesa inteira destacada: é a melhor mão de todos.
                            assert_eq!(picked_board.len(), 5, "{id}");
                            for (i, hole) in holes.iter().enumerate().filter(|(i, _)| !folded[*i]) {
                                assert_eq!(
                                    compare_hands(
                                        &evaluate_hand_pineapple_draw(hole, &board),
                                        &evaluate_hand(&[], &picked_board)
                                    ),
                                    Ordering::Equal,
                                    "{id}: p{i} teria mão melhor que a mesa"
                                );
                            }
                            continue;
                        };
                        let picked_hole: Vec<Card> = event["hole"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|i| holes[player][index(i)])
                            .collect();
                        assert!(picked_hole.len() <= 2, "{id}: no máximo duas privadas");
                        assert_eq!(
                            picked_hole.len() + picked_board.len(),
                            5,
                            "{id}: cinco cartas"
                        );
                        assert_eq!(
                            compare_hands(
                                &evaluate_hand(&picked_hole, &picked_board),
                                &evaluate_hand_pineapple_draw(&holes[player], &board)
                            ),
                            Ordering::Equal,
                            "{id}: as cartas destacadas não são a melhor mão de p{player}"
                        );
                    }
                    "award" => {
                        assert_eq!(board.len(), 5, "{id}");
                        let winners: Vec<usize> = match event["players"].as_array() {
                            Some(list) => list.iter().map(index).collect(),
                            None => vec![index(&event["player"])],
                        };
                        let best = winners
                            .iter()
                            .map(|w| evaluate_hand_pineapple_draw(&holes[*w], &board))
                            .next()
                            .unwrap();
                        for (i, hole) in holes.iter().enumerate().filter(|(i, _)| !folded[*i]) {
                            let order =
                                compare_hands(&evaluate_hand_pineapple_draw(hole, &board), &best);
                            let expected = if winners.contains(&i) {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            };
                            assert_eq!(order, expected, "{id}: resultado de p{i}");
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    assert_eq!(highlights, 7, "mãos destacadas conferidas");
}

/// Cena “O limite do pote, na prática”: aposta máxima, aumento máximo e pote final.
#[test]
fn pot_limit_scene_matches_engine_limits() {
    let script = &scripts()[0];
    let scene = script["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["title"] == "O limite do pote, na prática")
        .expect("cena do limite do pote")
        .clone();
    let mut game = game();
    for (id, mv) in [
        ("p2", PlayerMove::Call),
        ("p3", PlayerMove::Call),
        ("p0", PlayerMove::Call),
        ("p1", PlayerMove::Check),
    ] {
        game.player_action(id, mv).unwrap();
    }
    assert_eq!(game.state.phase, GamePhase::Flop);
    assert_table(&game, &scene);
    for event in events(&scene) {
        if let Some(player) = event["player"].as_u64() {
            let legal = game.legal_actions(&format!("p{player}"));
            match event["type"].as_str().unwrap() {
                // A cena mostra o máximo permitido em cada passo.
                "bet" | "raise" => assert_eq!(legal.maximum_wager, event["to"].as_u64().unwrap()),
                _ => {}
            }
        }
        play(&mut game, &event, "limite do pote");
    }
    // Apostas fechadas no flop com dois jogadores: abre a troca, com R$ 35 no pote.
    assert!(game.state.drawing);
    assert_eq!(game.state.total_pot(), 3500);
}
