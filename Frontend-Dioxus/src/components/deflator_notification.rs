use dioxus::prelude::*;

/// Payload para exibir a notificação global do Loss Deflator
#[derive(Debug, Clone, PartialEq)]
pub struct DeflatorPayload {
    pub loser_name: String,
    pub winner_name: String,
    pub cashback_amount: u64,
    pub odds_broken: u8,
    pub prevented_elimination: bool,
    pub is_tournament: bool,
}

#[component]
pub fn DeflatorNotification(payload: Option<DeflatorPayload>) -> Element {
    if payload.is_none() {
        return rsx! { div { display: "none" } };
    }

    let p = payload.unwrap();
    let cashback_str = if p.is_tournament {
        format!("{} fichas", p.cashback_amount)
    } else {
        format!("R$ {:.2}", (p.cashback_amount as f64) / 100.0).replace('.', ",")
    };

    let title;
    let description;
    let color_classes;

    // Nível 0 (<= 7%): Deflator Mínimo (7%) / Ajuste Mínimo
    // Nível 1 (<= 15%): Pré-flop (15%) / Variância Amenizada
    // Nível 2 (<= 25%): Flop (25%) / Reviravolta Moderada
    // Nível 3 (> 25%): Turn (35%) / Bad Beat Severa (Milagre no River)
    
    let loser_equity = 100 - p.odds_broken;
    let winner_odds = p.odds_broken;

    if p.odds_broken <= 7 {
        // NÍVEL 0 (7%): Verde Teal / Bad Beat em Ajuste Leve
        color_classes = "from-emerald-950 to-teal-900 border-teal-500 shadow-[0_0_35px_rgba(20,184,166,0.4)]";
        if p.prevented_elimination {
            title = "🛡️ PROTEÇÃO DE BAD BEAT (7%)".to_string();
            description = format!(
                "{} aplicou uma Bad Beat em {} (venceu com apenas {}% de chance contra {}%). O Loss Deflator devolveu {} para evitar que o perdedor ficasse zerado!",
                p.winner_name, p.loser_name, winner_odds, loser_equity, cashback_str
            );
        } else {
            title = "🛡️ PROTEÇÃO DE BAD BEAT (7%)!".to_string();
            description = format!(
                "{} aplicou uma Bad Beat em {} (venceu com apenas {}% de chance contra {}%). O sistema devolveu {} ao saldo de {} para amortecer a pancada.",
                p.winner_name, p.loser_name, winner_odds, loser_equity, cashback_str, p.loser_name
            );
        }
    } else if p.odds_broken <= 15 {
        // NÍVEL 1 (15%): Azul / Bad Beat Pré-Flop
        color_classes = "from-slate-900 to-blue-900 border-blue-500 shadow-[0_0_40px_rgba(59,130,246,0.4)]";
        if p.prevented_elimination {
            title = "⚖️ PROTEÇÃO DE BAD BEAT (15%)".to_string();
            description = format!(
                "{} virou a mão com {}% de chance contra os {}% de {}. O Loss Deflator devolveu {} para mantê-lo vivo na mesa!",
                p.winner_name, winner_odds, loser_equity, p.loser_name, cashback_str
            );
        } else {
            title = "⚖️ PROTEÇÃO DE BAD BEAT (15%)!".to_string();
            description = format!(
                "{} acertou uma virada no Pré-Flop ({}% de chance vs {}% de {}). O sistema devolveu {} ao stack do perdedor.",
                p.winner_name, winner_odds, loser_equity, p.loser_name, cashback_str
            );
        }
    } else if p.odds_broken <= 25 {
        // NÍVEL 2 (25%): Amarelo / Bad Beat no Flop
        color_classes = "from-yellow-900 to-orange-950 border-orange-500 shadow-[0_0_50px_rgba(249,115,22,0.5)]";
        if p.prevented_elimination {
            title = "⚠️ BAD BEAT NO FLOP! (DEFLATOR 25%)".to_string();
            description = format!(
                "{} virou o jogo com apenas {}% de chance contra os {}% de {}! {} foram devolvidos para salvar o oponente da eliminação.",
                p.winner_name, winner_odds, loser_equity, p.loser_name, cashback_str
            );
        } else {
            title = "⚠️ BAD BEAT NO FLOP! (DEFLATOR 25%)".to_string();
            description = format!(
                "{} acertou uma virada no Flop com {}% de chance contra {}% de {}. Nossa proteção devolveu {} para amortecer a Bad Beat.",
                p.winner_name, winner_odds, loser_equity, p.loser_name, cashback_str
            );
        }
    } else {
        // NÍVEL 3 (35%): Vermelho Neon / Bad Beat Severa (River)
        color_classes = "from-red-950 to-rose-900 border-red-500 shadow-[0_0_60px_rgba(239,68,68,0.7)] text-red-50";
        if p.prevented_elimination {
            title = "🚨 BAD BEAT SEVERA NO RIVER! (DEFLATOR 35%)".to_string();
            description = format!(
                "{} achou o milagre no River com apenas {}% de chance contra os {}% de {}! O sistema devolveu {} de fôlego para evitar a eliminação!",
                p.winner_name, winner_odds, loser_equity, p.loser_name, cashback_str
            );
        } else {
            title = "🚨 BAD BEAT SEVERA NO RIVER! (DEFLATOR 35%)".to_string();
            description = format!(
                "{} achou um milagre no River com apenas {}% de chance contra os {}% de {}. A plataforma atuou pesadamente devolvendo {} ao saldo de {}!",
                p.winner_name, winner_odds, loser_equity, p.loser_name, cashback_str, p.loser_name
            );
        }
    }

    rsx! {
        div {
            class: "fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-md animate-in fade-in duration-300",
            div {
                class: "bg-gradient-to-br border-4 rounded-2xl p-10 max-w-2xl text-center animate-in zoom-in-95 duration-500 transform transition-all {color_classes}",
                h2 {
                    class: "text-3xl font-extrabold text-white mb-6 tracking-wider drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)]",
                    "{title}"
                }
                p {
                    class: "text-xl leading-relaxed font-semibold px-4 drop-shadow-md text-gray-100",
                    "{description}"
                }
                div {
                    class: "mt-8 text-sm font-bold opacity-80 animate-pulse text-gray-300",
                    "A próxima mão iniciará em instantes..."
                }
            }
        }
    }
}
