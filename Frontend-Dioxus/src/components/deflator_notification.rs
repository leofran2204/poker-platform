use dioxus::prelude::*;

/// Payload para exibir a notificação global do Loss Deflator
#[derive(Debug, Clone, PartialEq)]
pub struct DeflatorPayload {
    pub loser_name: String,
    pub winner_name: String,
    pub cashback_amount: u64,
    pub deflator_percent: u8,
    pub loser_equity_percent: Option<f64>,
    pub prevented_elimination: bool,
    pub is_tournament: bool,
}

fn build_notification_copy(
    payload: &DeflatorPayload,
    cashback: &str,
) -> (String, String, &'static str) {
    let percent = payload.deflator_percent;
    let (title, color_classes) = if percent <= 7 {
        (
            "🛡️ PROTEÇÃO DE BAD BEAT ATIVADA (7%)",
            "from-emerald-950 to-teal-900 border-teal-500 shadow-[0_0_35px_rgba(20,184,166,0.4)]",
        )
    } else if percent <= 15 {
        (
            "⚖️ PROTEÇÃO DE BAD BEAT ATIVADA (15%)",
            "from-slate-900 to-blue-900 border-blue-500 shadow-[0_0_40px_rgba(59,130,246,0.4)]",
        )
    } else if percent <= 25 {
        (
            "⚠️ PROTEÇÃO DE BAD BEAT ATIVADA (25%)",
            "from-yellow-900 to-orange-950 border-orange-500 shadow-[0_0_50px_rgba(249,115,22,0.5)]",
        )
    } else {
        (
            "🚨 PROTEÇÃO DE BAD BEAT ATIVADA (35%)",
            "from-red-950 to-rose-900 border-red-500 shadow-[0_0_60px_rgba(239,68,68,0.7)] text-red-50",
        )
    };

    let equity_context = payload
        .loser_equity_percent
        .map(|equity| format!(" Ele tinha {:.2}% de equity no momento do all-in.", equity))
        .unwrap_or_default();
    let description = if payload.prevented_elimination {
        format!(
            "{} sofreu uma Bad Beat contra {}.{} O Loss Deflator devolveu {} ao jogador pela faixa de proteção de {}%, calculada sobre o pote elegível após o rake, e evitou que ele ficasse sem saldo.",
            payload.loser_name, payload.winner_name, equity_context, cashback, percent
        )
    } else {
        format!(
            "{} sofreu uma Bad Beat contra {}.{} O Loss Deflator devolveu {} ao saldo do jogador pela faixa de proteção de {}%, calculada sobre o pote elegível após o rake.",
            payload.loser_name, payload.winner_name, equity_context, cashback, percent
        )
    };

    (title.to_string(), description, color_classes)
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

    let (title, description, color_classes) = build_notification_copy(&p, &cashback_str);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(percent: u8, prevented_elimination: bool) -> DeflatorPayload {
        DeflatorPayload {
            loser_name: "Alice".to_string(),
            winner_name: "Bob".to_string(),
            cashback_amount: 71,
            deflator_percent: percent,
            loser_equity_percent: Some(80.25),
            prevented_elimination,
            is_tournament: false,
        }
    }

    #[test]
    fn copy_explains_refund_after_rake_without_fake_odds() {
        let (title, description, _) = build_notification_copy(&payload(25, false), "R$ 0,71");

        assert_eq!(title, "⚠️ PROTEÇÃO DE BAD BEAT ATIVADA (25%)");
        assert!(description.contains("Alice sofreu uma Bad Beat contra Bob"));
        assert!(description.contains("devolveu R$ 0,71"));
        assert!(description.contains("faixa de proteção de 25%"));
        assert!(description.contains("80.25% de equity no momento do all-in"));
        assert!(description.contains("pote elegível após o rake"));
        assert!(!description.contains("chance"));
    }

    #[test]
    fn copy_reports_when_refund_prevents_zero_balance() {
        let (_, description, _) = build_notification_copy(&payload(35, true), "R$ 1,99");

        assert!(description.contains("devolveu R$ 1,99"));
        assert!(description.contains("evitou que ele ficasse sem saldo"));
    }

    #[test]
    fn copy_keeps_legacy_seven_percent_visual_tier() {
        let (title, description, _) = build_notification_copy(&payload(7, false), "R$ 0,07");

        assert_eq!(title, "🛡️ PROTEÇÃO DE BAD BEAT ATIVADA (7%)");
        assert!(description.contains("faixa de proteção de 7%"));
    }
}
