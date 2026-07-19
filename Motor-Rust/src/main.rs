use poker_engine::deck::{create_deck, deal_cards, evaluate_hand, get_hand_name, shuffle_deck};
use poker_engine::loss_deflator::{
    calculate_progressive_loss_deflator, ProgressiveLossDeflatorParams,
};
use poker_engine::rake::deduct_rake;
use poker_engine::side_pots::{resolve_side_pots, PlayerForPots};
use poker_engine::types::{GamePhase, Pot, TableConfig};

fn main() {
    // ─── Demonstração do motor de baralho ───
    let deck = create_deck();
    println!("=== BARALHO ===");
    println!("Baralho criado: {} cartas", deck.len());

    let shuffled = shuffle_deck(&deck);
    println!("Primeiras 5 cartas embaralhadas: {:?}", &shuffled[..5]);

    let (hand, _remaining) = deal_cards(&shuffled, 2);
    let (community, _rest) = deal_cards(&shuffled[2..], 5);
    println!("Mão do jogador: {:?}", hand);
    println!("Cartas comunitárias: {:?}", community);

    let result = evaluate_hand(&hand, &community);
    println!(
        "Melhor mão: {} (valor={})",
        get_hand_name(result.rank),
        result.value
    );
    println!("Cartas da mão: {:?}", result.cards);
    println!("Kickers: {:?}", result.kickers);

    // ─── Demonstração do módulo de rake ───
    println!("\n=== RAKE ===");
    let config = TableConfig {
        big_blind: 10.0,
        rake_percent: 5.0,
        rake_cap: 10.0,
    };

    let pots = vec![
        Pot {
            amount: 100.0,
            eligible_players: vec!["alice".into(), "bob".into()],
        },
        Pot {
            amount: 50.0,
            eligible_players: vec!["alice".into()],
        },
    ];

    let rake_result = deduct_rake(&pots, &config, None);
    println!(
        "Pote total antes do rake: {}",
        rake_result.total_pot_before_rake
    );
    println!("Rake total deduzido: {}", rake_result.total_rake);
    for entry in &rake_result.per_pot {
        println!("  Pote {}: rake = {}", entry.pot_index, entry.rake);
    }
    for (i, pot) in rake_result.pots_after_rake.iter().enumerate() {
        println!("  Pote {} após rake: {} fichas", i, pot.amount);
    }

    // ─── Demonstração de Side Pots ───
    println!("\n=== SIDE POTS ===");
    let players = vec![
        PlayerForPots {
            id: "p1".into(),
            total_bet: 100.0,
            has_folded: false,
            cards: vec![],
        },
        PlayerForPots {
            id: "p2".into(),
            total_bet: 200.0,
            has_folded: false,
            cards: vec![],
        },
        PlayerForPots {
            id: "p3".into(),
            total_bet: 200.0,
            has_folded: false,
            cards: vec![],
        },
    ];
    let sp_result = resolve_side_pots(&players, &[]);
    println!("Pots criados: {}", sp_result.pots.len());
    for (i, pot) in sp_result.pots.iter().enumerate() {
        println!(
            "  Pote {}: {} fichas, elegíveis: {:?}",
            i, pot.amount, pot.eligible_players
        );
    }

    // ─── Demonstração do Loss Deflator ───
    println!("\n=== LOSS DEFLATOR ===");
    let pots_after_rake = vec![
        Pot {
            amount: 200.0,
            eligible_players: vec!["loser".into(), "winner".into()],
        },
        Pot {
            amount: 100.0,
            eligible_players: vec!["winner".into()],
        },
    ];
    let deflator_result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
        pots: pots_after_rake,
        loser_id: "loser".into(),
        winner_id: "winner".into(),
        phase: GamePhase::Flop,
    });
    if let Some(r) = deflator_result {
        println!("Cashback total: {} fichas", r.cashback);
        println!("Tier: {}", r.tier.as_str());
        println!("Fase: {:?}", r.phase);
        println!("Cartas restantes: {}", r.cards_remaining);
        println!("Pots elegíveis: {:?}", r.eligible_pot_ids);
        for entry in r.per_pot_cashback {
            println!("  Pote {}: cashback = {}", entry.pot_index, entry.amount);
        }
    }
}
