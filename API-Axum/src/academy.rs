//! Mesas de estudo sem carteira, SQL, settlement financeiro ou estado persistente.
//! Cada requisição reproduz uma mão limitada, usando o mesmo GameLoop das mesas.
use axum::{extract::Json, http::StatusCode};
use poker_engine::{
    deck::{self, Card, HandRank, HandResult, Suit},
    game_loop::{GameLoop, PlayerMove},
    hand_history::GameType,
    types::{GamePhase, PokerVariant, TableConfig},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};
use tokio::sync::Semaphore;

const CATALOG: &str = include_str!("../../Frontend-Web/src/data/courseTraining.json");
static CAPACITY: Semaphore = Semaphore::const_new(4);

#[derive(Clone, Deserialize)]
struct Scenario {
    id: String,
    module: String,
    variant: PokerVariant,
    phase: String,
    hole: Vec<String>,
    board: Vec<String>,
    topic: String,
    hint: String,
    profiles: Vec<String>,
    opening: u64,
    stack: u64,
    #[serde(default)]
    stacks: Vec<u64>,
    #[serde(default)]
    dealer: usize,
    #[serde(default)]
    setup: String,
    #[serde(default)]
    opponent_holes: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct Catalog {
    version: u32,
    scenarios: Vec<Scenario>,
}

fn catalog() -> &'static Catalog {
    static DATA: OnceLock<Catalog> = OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(CATALOG).expect("validated Academy catalog"))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StudyAction {
    pub action: String,
    #[serde(default)]
    pub amount: u64,
    #[serde(default)]
    pub answer: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyRequest {
    pub scenario: String,
    pub seed: u32,
    pub version: u32,
    #[serde(default)]
    pub actions: Vec<StudyAction>,
}

#[derive(Clone, Serialize)]
pub struct StudyPlayer {
    id: String,
    name: String,
    position: String,
    stack: u64,
    bet: u64,
    total: u64,
    folded: bool,
    all_in: bool,
    cards: Vec<String>,
    card_count: usize,
    category: Option<String>,
    best_five: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct StudySnapshot {
    phase: String,
    board: Vec<String>,
    players: Vec<StudyPlayer>,
    pot: u64,
    ante: u64,
    ante_paid: u64,
    ante_player_id: Option<String>,
    acting: usize,
    finished: bool,
}

#[derive(Serialize)]
pub struct StudyEvent {
    actor: String,
    action: String,
    amount: u64,
    setup: bool,
    state: StudySnapshot,
}

#[derive(Serialize)]
pub struct StudyQuestion {
    prompt: String,
    options: Vec<String>,
    unit: String,
}

#[derive(Serialize)]
pub struct StudyReview {
    decision: usize,
    action: String,
    prompt: String,
    answer: String,
    expected: String,
    correct: bool,
    explanation: String,
}

#[derive(Serialize)]
pub struct StudyLegal {
    can_check: bool,
    to_call: u64,
    min_total: u64,
    max_total: u64,
    can_raise: bool,
    can_all_in: bool,
    betting_structure: String,
    bet_exists: bool,
    call_price_percent: f64,
    eligible_pot_after_call: u64,
}

#[derive(Serialize)]
pub struct StudyResponse {
    version: u32,
    scenario: String,
    module: String,
    variant: String,
    state: StudySnapshot,
    legal: StudyLegal,
    question: Option<StudyQuestion>,
    hint: String,
    events: Vec<StudyEvent>,
    reviews: Vec<StudyReview>,
    payouts: HashMap<String, u64>,
    pots: Vec<poker_engine::types::Pot>,
}

type StudyError = (StatusCode, Json<serde_json::Value>);
fn error(message: impl Into<String>) -> StudyError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error":message.into()})),
    )
}

pub async fn play(Json(request): Json<StudyRequest>) -> Result<Json<StudyResponse>, StudyError> {
    let permit = CAPACITY.try_acquire().map_err(|_| (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({"error":"Mesas de estudo ocupadas. Tente novamente em instantes."})),
    ))?;
    let result = tokio::task::spawn_blocking(move || {
        // A tarefa mantém a vaga mesmo se o cliente cancelar a requisição.
        let _permit = permit;
        replay(request)
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"Não foi possível reconstruir o treino."})),
        )
    })?;
    result.map(Json).map_err(error)
}

// PRNG de estudo reproduzível, sem uso em mesas com saldo. Nenhuma carta é
// reordenada depois da distribuição inicial. A seed nunca entra na política do
// bot: a fonte aleatória de decisões é separada da fonte da distribuição.
fn next_random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut x = *state;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn card_code(card: &Card) -> String {
    let rank = match card.rank as u8 {
        14 => "A".into(),
        13 => "K".into(),
        12 => "Q".into(),
        11 => "J".into(),
        10 => "T".into(),
        n => n.to_string(),
    };
    let suit = match card.suit {
        Suit::Spades => 's',
        Suit::Hearts => 'h',
        Suit::Diamonds => 'd',
        Suit::Clubs => 'c',
    };
    format!("{rank}{suit}")
}

fn parse_card(code: &str) -> Result<Card, String> {
    deck::create_deck()
        .into_iter()
        .find(|c| card_code(c) == code)
        .ok_or_else(|| format!("Carta inválida no cenário: {code}"))
}

fn category(rank: HandRank) -> &'static str {
    match rank {
        HandRank::HighCard => "Carta alta",
        HandRank::OnePair => "Um par",
        HandRank::TwoPair => "Dois pares",
        HandRank::ThreeOfAKind => "Trinca",
        HandRank::Straight => "Sequência",
        HandRank::Flush => "Flush",
        HandRank::FullHouse => "Full house",
        HandRank::FourOfAKind => "Quadra",
        HandRank::StraightFlush => "Straight flush",
        HandRank::RoyalFlush => "Royal flush",
    }
}

fn evaluate(variant: PokerVariant, hole: &[Card], board: &[Card]) -> HandResult {
    match variant {
        PokerVariant::Holdem => deck::evaluate_hand(hole, board),
        PokerVariant::ShortDeck => deck::evaluate_hand_short_deck(hole, board),
        PokerVariant::Omaha => deck::evaluate_hand_omaha(hole, board),
        PokerVariant::BrazilianPineapple => deck::evaluate_hand_brazilian_pineapple(hole, board),
    }
}

fn make_deck(s: &Scenario, seed: u32) -> Result<Vec<Card>, String> {
    let count = s.profiles.len() + 1;
    if !(2..=6).contains(&count) {
        return Err("Número de participantes inválido".into());
    }
    let mut random = u64::from(seed);
    let cards = |codes: &[String]| -> Result<Vec<Card>, String> {
        codes.iter().map(|c| parse_card(c)).collect()
    };
    let hero = cards(&s.hole)?;
    let board = cards(&s.board)?;
    let opponents = s
        .opponent_holes
        .iter()
        .map(|h| cards(h))
        .collect::<Result<Vec<_>, _>>()?;
    let mut reserved = hero.clone();
    reserved.extend(&board);
    for h in &opponents {
        reserved.extend(h);
    }
    let full = if s.variant.uses_short_deck() {
        deck::create_short_deck()
    } else {
        deck::create_deck()
    };
    for (i, c) in reserved.iter().enumerate() {
        if !full.contains(c) || reserved[..i].contains(c) {
            return Err("Cartas repetidas ou fora da variante".into());
        }
    }
    let mut remaining: Vec<_> = full.into_iter().filter(|c| !reserved.contains(c)).collect();
    for i in (1..remaining.len()).rev() {
        let j = next_random(&mut random) as usize % (i + 1);
        remaining.swap(i, j);
    }
    let mut pick = || remaining.pop().expect("scenario validated: enough cards");
    let mut ordered = Vec::new();
    let initial = s.variant.hole_card_count();
    for i in 0..initial {
        for seat in 0..count {
            ordered.push(if seat == 0 {
                *hero.get(i).ok_or("Privadas insuficientes")?
            } else {
                opponents
                    .get(seat - 1)
                    .and_then(|h| h.get(i))
                    .copied()
                    .unwrap_or_else(&mut pick)
            });
        }
    }
    // Preencher o baralho inteiro antes da primeira ação. Extras de quem folda
    // permanecem no baralho: o GameLoop decide quem efetivamente recebe.
    let mut b = 0;
    for street in 0..3 {
        ordered.push(pick()); // burn
        for _ in 0..if street == 0 { 3 } else { 1 } {
            ordered.push(board.get(b).copied().unwrap_or_else(&mut pick));
            b += 1;
        }
        if s.variant == PokerVariant::BrazilianPineapple {
            for seat in 0..count {
                ordered.push(if seat == 0 {
                    hero.get(initial + street)
                        .copied()
                        .unwrap_or_else(&mut pick)
                } else {
                    pick()
                });
            }
        }
    }
    ordered.extend(remaining);
    Ok(ordered)
}

fn position(seat: usize, dealer: usize, n: usize) -> String {
    if n == 2 {
        return if seat == dealer { "BTN / SB" } else { "BB" }.into();
    }
    match (seat + n - dealer) % n {
        0 => "BTN".into(),
        1 => "SB".into(),
        2 => "BB".into(),
        _ => "UTG".into(),
    }
}

fn snapshot(game: &GameLoop, reveal: bool) -> StudySnapshot {
    let st = &game.state;
    StudySnapshot {
        phase: st.phase.as_str().into(),
        board: st.community_cards.iter().map(card_code).collect(),
        pot: st.total_pot(),
        ante: game.nominal_ante(),
        ante_paid: game.ante_paid(),
        ante_player_id: game.ante_player_id().map(str::to_owned),
        acting: st.active_player_index,
        finished: st.is_finished,
        players: st
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let visible = i == 0 || (reveal && !p.has_folded);
                let result = (visible && st.community_cards.len() >= 3).then(|| {
                    evaluate(
                        game.config.poker_variant,
                        &p.hole_cards,
                        &st.community_cards,
                    )
                });
                StudyPlayer {
                    id: p.id.clone(),
                    name: if i == 0 {
                        "Você".into()
                    } else {
                        format!("Bot {i}")
                    },
                    position: position(i, st.dealer_index, st.players.len()),
                    stack: p.stack,
                    bet: p.current_bet,
                    total: p.total_bet,
                    folded: p.has_folded,
                    all_in: p.is_all_in,
                    card_count: p.hole_cards.len(),
                    cards: if visible {
                        p.hole_cards.iter().map(card_code).collect()
                    } else {
                        vec![]
                    },
                    category: result.as_ref().map(|r| category(r.rank).into()),
                    best_five: result
                        .map(|r| {
                            r.cards
                                .iter()
                                .chain(&r.kickers)
                                .take(5)
                                .map(card_code)
                                .collect()
                        })
                        .unwrap_or_default(),
                }
            })
            .collect(),
    }
}

fn legal(game: &GameLoop) -> StudyLegal {
    let st = &game.state;
    let p = &st.players[0];
    let to_call = st
        .current_bet_to_match
        .saturating_sub(p.current_bet)
        .min(p.stack);
    let eligible_pot_after_call = game.eligible_pot_after_call("hero", to_call);
    let legal = game.legal_actions("hero");
    let min_total = legal.minimum_wager;
    let max_total = legal.maximum_wager;
    StudyLegal {
        can_check: to_call == 0,
        to_call,
        min_total,
        max_total,
        can_raise: legal.allows("raise") || legal.allows("bet"),
        can_all_in: legal.allows("allin"),
        betting_structure: game.betting_structure().into(),
        bet_exists: st.current_bet_to_match > 0,
        call_price_percent: if eligible_pot_after_call > 0 {
            100.0 * to_call as f64 / eligible_pot_after_call as f64
        } else {
            0.0
        },
        eligible_pot_after_call,
    }
}

fn question(game: &GameLoop, s: &Scenario) -> (StudyQuestion, String, String) {
    let p = &game.state.players[0];
    let board = &game.state.community_cards;
    let limits = legal(game);
    let mut options = vec![];
    let (prompt, expected, unit, explanation) = match s.topic.as_str() {
        "ranking" if board.len()>=3 => {
            options = [HandRank::HighCard,HandRank::OnePair,HandRank::TwoPair,HandRank::ThreeOfAKind,HandRank::Straight,HandRank::Flush,HandRank::FullHouse,HandRank::FourOfAKind,HandRank::StraightFlush,HandRank::RoyalFlush].into_iter().map(|r| category(r).into()).collect();
            let result = evaluate(s.variant,&p.hole_cards,board);
            ("Qual é a categoria da sua melhor mão agora?".into(),category(result.rank).into(),"".into(),"A categoria foi calculada pelo avaliador da modalidade usando somente suas cartas e o board atual. Ela não mede equity contra um range.".into())
        },
        "selection" => ("Quantas cartas privadas você deve usar no showdown desta modalidade?".into(),"2".into(),"cartas".into(),"Omaha e Brazilian Pineapple exigem exatamente duas privadas e três comunitárias. Nenhuma sexta carta desempata a mão.".into()),
        "combinations" if board.len()>=3 => {
            let h=p.hole_cards.len(); let b=board.len(); let combinations=h*(h-1)/2*b*(b-1)*(b-2)/6;
            ("Quantas candidatas 2+3 existem neste estado?".into(),combinations.to_string(),"combinações".into(),format!("C({h},2) × C({b},3) = {combinations}. As candidatas compartilham cartas e não são vitórias independentes."))
        },
        "sizing" => ("Qual é o menor total para uma aposta/aumento completo nesta rodada?".into(),limits.min_total.to_string(),"fichas".into(),format!("Aposta máxima atual {} + incremento mínimo {} = {} fichas. Um all-in curto pode não atingir esse total; confira seu stack.",game.state.current_bet_to_match,game.state.min_raise,limits.min_total)),
        "odds" if limits.to_call>0 => ("Qual é o preço do call como percentual do pote elegível após pagar?".into(),format!("{:.2}",limits.call_price_percent),"%".into(),format!("{} ÷ {} × 100 = {:.2}%. Sem rake no treino. Esse preço é exato; equity e apostas futuras exigem hipóteses adicionais.",limits.to_call,limits.eligible_pot_after_call,limits.call_price_percent)),
        "legal" | "odds" => ("Quanto falta pagar para acompanhar agora?".into(),limits.to_call.to_string(),"fichas".into(),"A diferença entre a maior aposta e sua contribuição nesta rodada, limitada ao seu stack, é o valor do call. Se for zero, use check.".into()),
        "position" => ("Quantos adversários ainda estão nesta mão?".into(),(game.state.players_in_hand_count()-1).to_string(),"adversários".into(),format!("Sua posição é {}. Conte quem ainda disputa o pote, inclusive all-ins; jogadores que foldaram não continuam.",position(0,game.state.dealer_index,game.state.players.len()))),
        "pots" => ("Qual é seu total já comprometido nesta mão?".into(),p.total_bet.to_string(),"fichas".into(),"A contribuição acumulada determina até que faixa de potes você participa. Pote paralelo não pertence automaticamente ao vencedor do pote principal.".into()),
        _ => { let n=if s.variant.uses_short_deck() {36} else {52}; let unknown=n-p.hole_cards.len()-board.len();
            ("Quantas cartas você ainda não conhece?".into(),unknown.to_string(),"cartas".into(),format!("{n} menos suas {} privadas e as {} comunitárias. Cartas dos rivais e queimadas permanecem desconhecidas.",p.hole_cards.len(),board.len())) },
    };
    (
        StudyQuestion {
            prompt,
            options,
            unit,
        },
        expected,
        explanation,
    )
}

fn record_action(
    game: &mut GameLoop,
    action: PlayerMove,
    setup: bool,
    events: &mut Vec<StudyEvent>,
) -> Result<(), String> {
    let actor = game
        .state
        .active_player()
        .ok_or("Sem jogador ativo")?
        .id
        .clone();
    let before = game.state.active_player().expect("checked").total_bet;
    let label = match action {
        PlayerMove::Fold => "fold",
        PlayerMove::Check => "check",
        PlayerMove::Call => "call",
        PlayerMove::Bet(_) => "bet",
        PlayerMove::Raise(_) => "raise",
        PlayerMove::AllIn => "all_in",
    }
    .to_string();
    game.player_action(&actor, action)
        .map_err(|e| e.to_string())?;
    let amount = game
        .state
        .players
        .iter()
        .find(|p| p.id == actor)
        .expect("existing player")
        .total_bet
        - before;
    events.push(StudyEvent {
        actor,
        action: label,
        amount,
        setup,
        state: snapshot(game, false),
    });
    Ok(())
}

// Política recebe SOMENTE cartas próprias, board, preço e estado público.
// Nem baralho restante, nem hole cards de terceiros, nem seed da distribuição.
struct BotView<'a> {
    hole: &'a [Card],
    board: &'a [Card],
    variant: PokerVariant,
    stack: u64,
    current: u64,
    to_match: u64,
    min_raise: u64,
    pot: u64,
    acted: bool,
}

fn bot_move(view: BotView<'_>, profile: &str, random: &mut u64) -> PlayerMove {
    let to_call = view.to_match.saturating_sub(view.current);
    let roll = next_random(random) % 100;
    let rank = if view.board.len() >= 3 {
        evaluate(view.variant, view.hole, view.board).rank
    } else {
        HandRank::HighCard
    };
    let paired = view
        .hole
        .iter()
        .enumerate()
        .any(|(i, card)| view.hole[..i].iter().any(|other| other.rank == card.rank));
    let strong = rank >= HandRank::TwoPair || paired;
    let passive = if to_call > 0 {
        PlayerMove::Call
    } else {
        PlayerMove::Check
    };
    if to_call > 0 {
        if profile == "value" && strong && !view.acted {
            let total = view.to_match + (view.to_match * 2).max(view.min_raise);
            if total <= view.current + view.stack {
                return PlayerMove::Raise(total);
            }
        }
        if profile == "tight" && !strong && (roll < 65 || to_call > view.pot / 2) {
            return PlayerMove::Fold;
        }
        if to_call > view.pot && !strong && profile != "pressure" {
            return PlayerMove::Fold;
        }
        return passive;
    }
    if view.acted {
        return PlayerMove::Check;
    }
    let attack = match profile {
        "pressure" => true,
        "opener" => view.board.is_empty() || strong,
        "value" => false, // check-raise; só aumenta com força demonstrada
        "tight" => strong && roll < 70,
        _ => false,
    };
    if attack {
        let fraction = [33, 50, 75][(roll % 3) as usize];
        let total = (view.pot * fraction / 100).max(view.to_match + view.min_raise);
        if total >= view.current + view.stack {
            PlayerMove::AllIn
        } else if view.to_match > 0 {
            PlayerMove::Raise(total)
        } else {
            PlayerMove::Bet(total)
        }
    } else {
        PlayerMove::Check
    }
}

fn advance_bots(
    game: &mut GameLoop,
    s: &Scenario,
    random: &mut u64,
    events: &mut Vec<StudyEvent>,
) -> Result<(), String> {
    while !game.state.is_finished && game.state.active_player_index != 0 {
        if events.len() >= 256 {
            return Err("Limite de ações da mão atingido".into());
        }
        let i = game.state.active_player_index;
        let p = &game.state.players[i];
        let action = bot_move(
            BotView {
                hole: &p.hole_cards,
                board: &game.state.community_cards,
                variant: s.variant,
                stack: p.stack,
                current: p.current_bet,
                to_match: game.state.current_bet_to_match,
                min_raise: game.state.min_raise,
                pot: game.state.total_pot(),
                acted: p.has_acted,
            },
            &s.profiles[i - 1],
            random,
        );
        let action = game.legal_actions(&p.id).constrain_move(action);
        record_action(game, action, false, events)?;
        game.run_out_stalled_hand();
    }
    Ok(())
}

pub fn replay(request: StudyRequest) -> Result<StudyResponse, String> {
    if request.version != catalog().version {
        return Err(
            "O treino foi atualizado. Recarregue a página para iniciar uma nova sessão.".into(),
        );
    }
    if request.actions.len() > 64
        || request
            .actions
            .iter()
            .any(|a| a.answer.len() > 80 || a.action.len() > 12 || a.amount > 100000)
    {
        return Err("Sessão excede os limites do treino".into());
    }
    let s = catalog()
        .scenarios
        .iter()
        .find(|s| s.id == request.scenario)
        .ok_or("Cenário inexistente")?;
    let mut game = GameLoop::new(
        TableConfig::new(100, 0, 0).with_poker_variant(s.variant),
        "academy".into(),
        "Mesa de estudo".into(),
        GameType::Cash,
    )
    .with_skip_loss_deflator(true);
    for i in 0..=s.profiles.len() {
        game.add_player(
            if i == 0 {
                "hero".into()
            } else {
                format!("bot-{i}")
            },
            s.stacks.get(i).copied().unwrap_or(s.stack),
        );
    }
    game.set_dealer(s.dealer);
    game.start_hand_with_deck(make_deck(s, request.seed)?)
        .map_err(|e| e.to_string())?;
    let mut events = vec![StudyEvent {
        actor: "mesa".into(),
        action: "deal".into(),
        amount: 0,
        setup: true,
        state: snapshot(&game, false),
    }];
    // Prefixo público do exercício: raise/calls pré-flop e checks até a street
    // de estudo. Todas as transições passam pelo motor e aparecem no replay.
    while game.state.phase.as_str() != s.phase && !game.state.is_finished {
        if events.len() > 80 {
            return Err("Prefixo inválido".into());
        }
        let player = game.state.active_player().ok_or("Prefixo sem jogador")?;
        let action = if game.state.phase == GamePhase::Preflop
            && game.state.current_bet_to_match < s.opening
        {
            game.legal_actions(&player.id)
                .constrain_move(PlayerMove::Raise(s.opening))
        } else if player.current_bet < game.state.current_bet_to_match {
            PlayerMove::Call
        } else {
            PlayerMove::Check
        };
        record_action(&mut game, action, true, &mut events)?;
    }
    if s.setup == "limp" {
        record_action(&mut game, PlayerMove::Call, true, &mut events)?;
    }
    if s.setup == "open" {
        let action = game
            .legal_actions(&game.state.active_player().unwrap().id)
            .constrain_move(PlayerMove::Raise(s.opening));
        record_action(&mut game, action, true, &mut events)?;
    }
    let mut random = u64::from(request.seed) ^ 0x67ad_f010_873a_9765;
    advance_bots(&mut game, s, &mut random, &mut events)?;
    let mut reviews = vec![];
    for (i, input) in request.actions.iter().enumerate() {
        if game.state.is_finished {
            return Err("Há ações após o fim da mão".into());
        }
        let (q, expected, explanation) = question(&game, s);
        let parsed = input.answer.trim().replace(',', ".").parse::<f64>().ok();
        let correct = if let Ok(value) = expected.parse::<f64>() {
            parsed.is_some_and(|x| {
                x.is_finite() && (x - value).abs() <= if q.unit == "%" { 0.15 } else { 0.001 }
            })
        } else {
            input.answer.trim() == expected
        };
        let limits = legal(&game);
        let action = match input.action.as_str() {
            "fold" => PlayerMove::Fold,
            "check" => PlayerMove::Check,
            "call" => PlayerMove::Call,
            "all_in" => PlayerMove::AllIn,
            "bet"
                if !limits.bet_exists
                    && input.amount >= limits.min_total
                    && input.amount <= limits.max_total =>
            {
                PlayerMove::Bet(input.amount)
            }
            "raise"
                if limits.bet_exists
                    && input.amount >= limits.min_total
                    && input.amount <= limits.max_total =>
            {
                PlayerMove::Raise(input.amount)
            }
            _ => return Err("Ação ou tamanho inválido".into()),
        };
        record_action(&mut game, action, false, &mut events)?;
        reviews.push(StudyReview {
            decision: i + 1,
            action: input.action.clone(),
            prompt: q.prompt,
            answer: input.answer.clone(),
            expected,
            correct,
            explanation,
        });
        game.run_out_stalled_hand();
        advance_bots(&mut game, s, &mut random, &mut events)?;
    }
    let hint = if game.state.is_finished {
        "Confira os pagamentos e use o replay para rever cada decisão com a informação disponível naquele momento.".into()
    } else if game.state.phase.as_str() != s.phase {
        format!(
            "A mão avançou; refaça a leitura com as cartas atuais. {}",
            question(&game, s).2
        )
    } else {
        s.hint.clone()
    };
    let question = (!game.state.is_finished).then(|| question(&game, s).0);
    let mut payouts = HashMap::new();
    let mut pots = vec![];
    if game.state.is_finished {
        let resolution = game.resolve_hand().map_err(|e| e.to_string())?;
        payouts = resolution.payouts;
        pots = resolution.pots;
        events.push(StudyEvent {
            actor: "mesa".into(),
            action: "showdown".into(),
            amount: 0,
            setup: false,
            state: snapshot(&game, true),
        });
    }
    Ok(StudyResponse {
        version: catalog().version,
        scenario: s.id.clone(),
        module: s.module.clone(),
        variant: s.variant.as_str().into(),
        state: snapshot(&game, game.state.is_finished),
        legal: legal(&game),
        question,
        hint,
        events,
        reviews,
        payouts,
        pots,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn request(scenario: &str, seed: u32, actions: Vec<StudyAction>) -> StudyRequest {
        StudyRequest {
            scenario: scenario.into(),
            seed,
            version: catalog().version,
            actions,
        }
    }

    #[test]
    fn every_scenario_accepts_minimum_sizings_and_rejects_out_of_bounds() {
        for s in &catalog().scenarios {
            let mut actions = vec![];
            loop {
                let state = replay(request(&s.id, 11, actions.clone())).unwrap();
                if state.state.finished {
                    break;
                }
                assert!(actions.len() < 20, "stalled {}", s.id);
                let label = if state.legal.bet_exists {
                    "raise"
                } else {
                    "bet"
                };
                if state.legal.can_raise {
                    for amount in [state.legal.min_total - 1, state.legal.max_total + 1] {
                        let mut invalid = actions.clone();
                        invalid.push(StudyAction {
                            action: label.into(),
                            amount,
                            answer: String::new(),
                        });
                        assert!(
                            replay(request(&s.id, 11, invalid)).is_err(),
                            "{} {amount}",
                            s.id
                        );
                    }
                }
                actions.push(StudyAction {
                    action: if state.legal.can_raise {
                        label
                    } else if state.legal.can_check {
                        "check"
                    } else {
                        "call"
                    }
                    .into(),
                    amount: if state.legal.can_raise {
                        state.legal.min_total
                    } else {
                        0
                    },
                    answer: String::new(),
                });
            }
        }
    }

    #[test]
    fn hint_tracks_new_streets_instead_of_repeating_the_initial_answer() {
        let initial = replay(request("omaha-2mais3", 5, vec![])).unwrap();
        let turn = replay(request(
            "omaha-2mais3",
            5,
            vec![StudyAction {
                action: "check".into(),
                amount: 0,
                answer: "6".into(),
            }],
        ))
        .unwrap();
        assert_eq!(initial.state.phase, "flop");
        assert_eq!(turn.state.phase, "turn");
        assert_ne!(initial.hint, turn.hint);
        assert!(turn.hint.contains("C(4,2) × C(4,3) = 24"));
    }

    #[test]
    fn catalog_covers_all_modules_and_lessons_and_legal_decks() {
        let course: serde_json::Value = serde_json::from_str(include_str!(
            "../../Frontend-Web/src/data/courseContent.json"
        ))
        .unwrap();
        for m in course["modules"].as_array().unwrap() {
            assert!(catalog()
                .scenarios
                .iter()
                .any(|s| s.module == m["id"].as_str().unwrap()));
        }
        let raw: serde_json::Value = serde_json::from_str(CATALOG).unwrap();
        for lesson in course["modules"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|m| m["lessons"].as_array().unwrap())
        {
            assert!(
                raw["scenarios"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|s| s["lesson"] == lesson["id"]),
                "missing {}",
                lesson["id"]
            );
        }
        for s in &catalog().scenarios {
            for seed in 0..8 {
                let cards = make_deck(s, seed).unwrap();
                let unique: HashSet<_> = cards.iter().map(card_code).collect();
                assert_eq!(cards.len(), unique.len(), "{}", s.id);
                assert_eq!(
                    cards.len(),
                    if s.variant.uses_short_deck() { 36 } else { 52 }
                );
            }
        }
    }

    #[test]
    fn every_scenario_completes_with_calls_checks_and_conserves_chips() {
        for s in &catalog().scenarios {
            for seed in 0..4 {
                let mut actions = vec![];
                loop {
                    let result = replay(request(&s.id, seed, actions.clone()))
                        .unwrap_or_else(|e| panic!("{} seed {seed}: {e}", s.id));
                    for player in result.state.players.iter().skip(1) {
                        if !result.state.finished {
                            assert!(player.cards.is_empty(), "hidden cards leaked");
                        }
                    }
                    if result.state.finished {
                        let initial = (0..=s.profiles.len())
                            .map(|i| s.stacks.get(i).copied().unwrap_or(s.stack))
                            .sum::<u64>();
                        // resolve_hand retorna pagamentos sem aplicá-los ao stack.
                        let stacks = result.state.players.iter().map(|p| p.stack).sum::<u64>();
                        assert_eq!(
                            stacks + result.payouts.values().sum::<u64>(),
                            initial,
                            "{}",
                            s.id
                        );
                        assert_eq!(result.payouts.values().sum::<u64>(), result.state.pot);
                        break;
                    }
                    assert!(actions.len() < 40, "scenario stalled: {}", s.id);
                    actions.push(StudyAction {
                        action: if result.legal.can_check {
                            "check"
                        } else {
                            "call"
                        }
                        .into(),
                        amount: 0,
                        answer: String::new(),
                    });
                }
            }
        }
    }

    #[test]
    fn every_scenario_accepts_fold_and_only_legal_all_ins_without_financial_side_effects() {
        for s in &catalog().scenarios {
            for action in ["fold", "all_in"] {
                if action == "all_in"
                    && !replay(request(&s.id, 7, vec![])).unwrap().legal.can_all_in
                {
                    assert!(replay(request(
                        &s.id,
                        7,
                        vec![StudyAction {
                            action: action.into(),
                            amount: 0,
                            answer: String::new()
                        }]
                    ))
                    .is_err());
                    continue;
                }
                let result = replay(request(
                    &s.id,
                    7,
                    vec![StudyAction {
                        action: action.into(),
                        amount: 0,
                        answer: String::new(),
                    }],
                ))
                .unwrap();
                assert!(result.state.finished, "{} {action}", s.id);
                if s.variant == PokerVariant::BrazilianPineapple
                    && action == "all_in"
                    && result.state.board.len() == 5
                {
                    assert_eq!(result.state.players[0].cards.len(), 5);
                }
            }
        }
    }

    #[test]
    fn replay_is_deterministic_and_never_reveals_future_cards_in_earlier_events() {
        for s in &catalog().scenarios {
            let a = replay(request(&s.id, 314, vec![])).unwrap();
            let b = replay(request(&s.id, 314, vec![])).unwrap();
            assert_eq!(
                serde_json::to_value(&a).unwrap(),
                serde_json::to_value(&b).unwrap()
            );
            for event in &a.events {
                for p in event.state.players.iter().skip(1) {
                    assert!(p.cards.is_empty());
                }
            }
        }
    }

    #[test]
    fn objective_answers_are_graded_on_server_and_bad_inputs_rejected() {
        let a = replay(request(
            "pine-combinacoes",
            5,
            vec![StudyAction {
                action: "call".into(),
                amount: 0,
                answer: "24".into(),
            }],
        ))
        .unwrap();
        assert!(a.reviews[0].correct);
        let a = replay(request(
            "omaha-2mais3",
            5,
            vec![StudyAction {
                action: "check".into(),
                amount: 0,
                answer: "6".into(),
            }],
        ))
        .unwrap();
        assert!(a.reviews[0].correct);
        assert!(replay(request("unknown", 1, vec![])).is_err());
        assert!(replay(StudyRequest {
            version: 999,
            ..request("primeira-mao", 1, vec![])
        })
        .is_err());
        assert!(replay(request(
            "primeira-mao",
            1,
            vec![StudyAction {
                action: "check".into(),
                amount: 0,
                answer: "NaN".into()
            }]
        ))
        .is_err());
        assert!(replay(request(
            "primeira-mao",
            1,
            vec![StudyAction {
                action: "raise".into(),
                amount: 100001,
                answer: String::new()
            }]
        ))
        .is_err());
    }

    #[test]
    fn pineapple_v3_catalog_exposes_ante_and_rejects_previous_sessions() {
        let response = replay(request("pine-distribuicao", 7, vec![])).unwrap();
        assert_eq!(response.version, 3);
        assert_eq!(response.state.ante, 100);
        assert_eq!(response.state.ante_paid, 100);
        assert_eq!(response.state.ante_player_id.as_deref(), Some("bot-2"));
        assert_eq!(response.state.pot, 250);
        assert_eq!(response.legal.to_call, 100);
        assert_eq!(response.legal.max_total, 350);
        assert_eq!(response.legal.eligible_pot_after_call, 350);
        assert_eq!(
            response.legal.betting_structure,
            poker_engine::game_loop::PINEAPPLE_RULE_VERSION
        );
        assert!(replay(StudyRequest {
            version: 2,
            ..request("pine-distribuicao", 7, vec![])
        })
        .is_err());
    }

    #[test]
    fn bot_policy_has_no_access_to_opponents_or_undealt_cards() {
        let hole = vec![parse_card("8h").unwrap(), parse_card("8d").unwrap()];
        let board = vec![
            parse_card("Kc").unwrap(),
            parse_card("8s").unwrap(),
            parse_card("3d").unwrap(),
        ];
        let action = bot_move(
            BotView {
                hole: &hole,
                board: &board,
                variant: PokerVariant::Holdem,
                stack: 9700,
                current: 0,
                to_match: 200,
                min_raise: 200,
                pot: 800,
                acted: false,
            },
            "value",
            &mut 2,
        );
        assert!(matches!(action, PlayerMove::Raise(600)));
        let a = replay(request(
            "flop-checkraise",
            2,
            vec![StudyAction {
                action: "bet".into(),
                amount: 200,
                answer: "Um par".into(),
            }],
        ))
        .unwrap();
        assert!(a
            .events
            .iter()
            .any(|e| !e.setup && e.actor == "bot-1" && e.action == "raise"));
    }
}
