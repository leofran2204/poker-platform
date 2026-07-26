use poker_engine::deck::{create_deck, deal_cards, evaluate_hand, get_hand_name, shuffle_deck};
use poker_engine::loss_deflator::{
    calculate_progressive_loss_deflator, ProgressiveLossDeflatorParams,
};
use poker_engine::rake::deduct_rake;
use poker_engine::side_pots::{calculate_side_pots, PlayerForPots};
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
        big_blind: 1000,
        rake_percent: 3.5,
        rake_cap: 500,
    };

    let pots = vec![
        Pot {
            amount: 10000,
            eligible_players: vec!["alice".into(), "bob".into()],
        },
        Pot {
            amount: 5000,
            eligible_players: vec!["alice".into()],
        },
    ];

    let rake_result = deduct_rake(&pots, &config, None);
    println!(
        "Pote total antes do rake (centavos): {}",
        rake_result.total_pot_before_rake
    );
    println!("Rake total deduzido (centavos): {}", rake_result.total_rake);
    for entry in &rake_result.per_pot {
        println!("  Pote {}: rake = {} centavos", entry.pot_index, entry.rake);
    }
    for (i, pot) in rake_result.pots_after_rake.iter().enumerate() {
        println!("  Pote {} após rake: {} centavos", i, pot.amount);
    }

    // ─── Demonstração de Side Pots ───
    println!("\n=== SIDE POTS ===");
    let players = vec![
        PlayerForPots {
            id: "p1".into(),
            total_bet: 10000,
            has_folded: false,
            cards: vec![],
        },
        PlayerForPots {
            id: "p2".into(),
            total_bet: 20000,
            has_folded: false,
            cards: vec![],
        },
        PlayerForPots {
            id: "p3".into(),
            total_bet: 20000,
            has_folded: false,
            cards: vec![],
        },
    ];
    let pots_created = calculate_side_pots(&players);
    println!("Pots criados: {}", pots_created.len());
    for (i, pot) in pots_created.iter().enumerate() {
        println!(
            "  Pote {}: {} centavos, elegíveis: {:?}",
            i, pot.amount, pot.eligible_players
        );
    }

    // ─── Demonstração do Loss Deflator ───
    println!("\n=== LOSS DEFLATOR ===");
    let pots_after_rake = vec![
        Pot {
            amount: 20000,
            eligible_players: vec!["loser".into(), "winner".into()],
        },
        Pot {
            amount: 10000,
            eligible_players: vec!["winner".into()],
        },
    ];
    let deflator_result = calculate_progressive_loss_deflator(ProgressiveLossDeflatorParams {
        pots: pots_after_rake,
        loser_id: "loser".into(),
        winner_id: "winner".into(),
        phase: GamePhase::Flop,
    });

    if let Some(d) = deflator_result {
        println!("Loss Deflator ativado (centavos): {}", d.cashback);
    }
}
