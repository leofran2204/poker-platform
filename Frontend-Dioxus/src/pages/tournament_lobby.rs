//! Página de Lobby de Torneios Multimesas (MTT)
//!
//! Exibe a estrutura de blinds, premiação (Prizepool), lista de participantes
//! e botões de inscrição/início para o jogador.

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct BlindLevelRow {
    level: u32,
    small_blind: u64,
    big_blind: u64,
    ante: u64,
    duration_minutes: u32,
}

#[derive(Clone, PartialEq)]
struct PrizeRow {
    position_label: String,
    percentage: String,
    amount_cents: u64,
    highlight: PrizeHighlight,
}

#[derive(Clone, Copy, PartialEq)]
enum PrizeHighlight {
    Gold,
    Silver,
    Bronze,
    Normal,
}

#[derive(Clone, PartialEq)]
struct RegisteredPlayer {
    seat_or_order: u32,
    nickname: String,
    stack: u64,
    status: &'static str,
}

fn format_cents_brl(cents: u64) -> String {
    let reais = cents / 100;
    let frac = cents % 100;
    format!("R$ {reais},{frac:02}")
}

fn format_chips(n: u64) -> String {
    if n >= 1_000 {
        format!("{}", n) // keep plain for stacks; UI can grow later
    } else {
        n.to_string()
    }
}

fn demo_blinds() -> Vec<BlindLevelRow> {
    vec![
        BlindLevelRow {
            level: 1,
            small_blind: 100,
            big_blind: 200,
            ante: 0,
            duration_minutes: 10,
        },
        BlindLevelRow {
            level: 2,
            small_blind: 150,
            big_blind: 300,
            ante: 0,
            duration_minutes: 10,
        },
        BlindLevelRow {
            level: 3,
            small_blind: 200,
            big_blind: 400,
            ante: 0,
            duration_minutes: 10,
        },
        BlindLevelRow {
            level: 4,
            small_blind: 300,
            big_blind: 600,
            ante: 50,
            duration_minutes: 10,
        },
        BlindLevelRow {
            level: 5,
            small_blind: 400,
            big_blind: 800,
            ante: 100,
            duration_minutes: 10,
        },
        BlindLevelRow {
            level: 6,
            small_blind: 500,
            big_blind: 1_000,
            ante: 100,
            duration_minutes: 10,
        },
    ]
}

fn demo_prizes() -> Vec<PrizeRow> {
    vec![
        PrizeRow {
            position_label: "🥇 1º Lugar".to_string(),
            percentage: "30%".to_string(),
            amount_cents: 300_000,
            highlight: PrizeHighlight::Gold,
        },
        PrizeRow {
            position_label: "🥈 2º Lugar".to_string(),
            percentage: "20%".to_string(),
            amount_cents: 200_000,
            highlight: PrizeHighlight::Silver,
        },
        PrizeRow {
            position_label: "🥉 3º Lugar".to_string(),
            percentage: "14%".to_string(),
            amount_cents: 140_000,
            highlight: PrizeHighlight::Bronze,
        },
        PrizeRow {
            position_label: "4º ao 8º Lugar".to_string(),
            percentage: "5% cada".to_string(),
            amount_cents: 50_000,
            highlight: PrizeHighlight::Normal,
        },
        PrizeRow {
            position_label: "9º ao 15º Lugar".to_string(),
            percentage: "2% cada".to_string(),
            amount_cents: 20_000,
            highlight: PrizeHighlight::Normal,
        },
    ]
}

fn demo_registered() -> Vec<RegisteredPlayer> {
    vec![
        RegisteredPlayer {
            seat_or_order: 1,
            nickname: "ZeroTiltHero".to_string(),
            stack: 20_000,
            status: "Registered",
        },
        RegisteredPlayer {
            seat_or_order: 2,
            nickname: "RiverShark".to_string(),
            stack: 20_000,
            status: "Registered",
        },
        RegisteredPlayer {
            seat_or_order: 3,
            nickname: "BluffMaster".to_string(),
            stack: 20_000,
            status: "Registered",
        },
        RegisteredPlayer {
            seat_or_order: 4,
            nickname: "Ana_MTT".to_string(),
            stack: 20_000,
            status: "Registered",
        },
        RegisteredPlayer {
            seat_or_order: 5,
            nickname: "ClubAgent_SP".to_string(),
            stack: 20_000,
            status: "Registered",
        },
        RegisteredPlayer {
            seat_or_order: 6,
            nickname: "LateReg_Pro".to_string(),
            stack: 20_000,
            status: "Registered",
        },
    ]
}

#[component]
pub fn TournamentLobbyPage(id: String) -> Element {
    let mut is_registered = use_signal(|| false);
    let mut status_msg = use_signal(|| "".to_string());
    let mut registered = use_signal(demo_registered);
    let blinds = use_signal(demo_blinds);
    let prizes = use_signal(demo_prizes);

    // Demo: buy-in R$50,00 + 7% fee = R$3,50 → 5000 + 350 centavos
    let buy_in_cents: u64 = 5_000;
    let fee_cents: u64 = 350;
    let starting_stack: u64 = 20_000;
    let prize_pool_cents: u64 = 1_000_000;
    let max_players: u32 = 500;
    let min_players: u32 = 20;
    let current_level: u32 = 1;

    let handle_register = move |_| {
        let current = *is_registered.read();
        if current {
            is_registered.set(false);
            // Remove demo self entry if present
            registered.write().retain(|p| p.nickname != "Você");
            // renumber
            for (i, p) in registered.write().iter_mut().enumerate() {
                p.seat_or_order = (i as u32) + 1;
            }
            status_msg.set(
                "Inscrição cancelada. O valor do buy-in foi devolvido ao seu saldo.".to_string(),
            );
        } else {
            is_registered.set(true);
            let next = (registered.read().len() as u32) + 1;
            registered.write().push(RegisteredPlayer {
                seat_or_order: next,
                nickname: "Você".to_string(),
                stack: starting_stack,
                status: "Registered",
            });
            status_msg.set("✅ Inscrição confirmada com sucesso no Torneio!".to_string());
        }
    };

    let registered_count = registered.read().len() as u32;

    rsx! {
        div { class: "max-w-6xl mx-auto p-6 space-y-8",
            // Header do Torneio
            div { class: "flex justify-between items-center bg-gray-900/80 backdrop-blur p-6 rounded-2xl border border-gray-800 shadow-xl flex-wrap gap-4",
                div {
                    h1 { class: "text-3xl font-extrabold text-transparent bg-clip-text bg-gradient-to-r from-yellow-400 to-amber-500",
                        "🏆 Torneio MTT #{id} — Sunday Grand Freezeout"
                    }
                    p { class: "text-gray-400 text-sm mt-1", "Texas Hold'em No-Limit · Multi-tables · Rebalanceamento Automático" }
                    p { class: "text-gray-500 text-xs mt-1",
                        "Blind levels: 10 min · Starting stack: {format_chips(starting_stack)} · Late reg até nível 4"
                    }
                }
                div { class: "flex items-center gap-4 flex-wrap",
                    span { class: "px-4 py-1.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/30 text-xs font-semibold uppercase tracking-wider",
                        "Inscrições Abertas"
                    }
                    button {
                        class: if *is_registered.read() {
                            "px-6 py-2.5 rounded-xl bg-red-600/80 hover:bg-red-600 text-white font-bold transition-all shadow-lg"
                        } else {
                            "px-6 py-2.5 rounded-xl bg-gradient-to-r from-yellow-500 to-amber-600 hover:from-yellow-400 hover:to-amber-500 text-black font-extrabold transition-all shadow-lg"
                        },
                        onclick: handle_register,
                        if *is_registered.read() {
                            "Cancelar Inscrição"
                        } else {
                            {format!("Inscrever-se ({} + 7% Fee)", format_cents_brl(buy_in_cents))}
                        }
                    }
                }
            }

            // Cards Informativos
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-6",
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Garantido / Prizepool" }
                    p { class: "text-3xl font-black text-emerald-400 mt-2", "{format_cents_brl(prize_pool_cents)}" }
                    p { class: "text-xs text-gray-500 mt-1", "Premiação total estimada" }
                }
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Buy-in + Taxa (7%)" }
                    p { class: "text-3xl font-black text-white mt-2",
                        "{format_cents_brl(buy_in_cents)} + {format_cents_brl(fee_cents)}"
                    }
                    p { class: "text-xs text-gray-500 mt-1", "Re-buys: mesmo buy-in (0% Fee no demo)" }
                }
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Stack Inicial" }
                    p { class: "text-3xl font-black text-yellow-400 mt-2", "{format_chips(starting_stack)}" }
                    p { class: "text-xs text-gray-500 mt-1", "Fichas no nível 1" }
                }
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Jogadores Inscritos" }
                    p { class: "text-3xl font-black text-blue-400 mt-2",
                        "{registered_count} / {max_players}"
                    }
                    p { class: "text-xs text-gray-500 mt-1", "Mínimo: {min_players} inscritos" }
                }
            }

            // Tabelas de Estrutura de Blinds & Premiações (Prizepool)
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-8",
                div { class: "bg-gray-900/80 p-6 rounded-2xl border border-gray-800 shadow-xl space-y-4",
                    h2 { class: "text-xl font-bold text-white flex items-center gap-2", "⏱️ Estrutura de Níveis (Blinds)" }
                    p { class: "text-gray-400 text-sm", "Níveis de 10 minutos com ante a partir do nível 4." }

                    div { class: "overflow-x-auto",
                        table { class: "w-full text-left text-sm text-gray-300",
                            thead { class: "bg-gray-950 text-gray-400 uppercase text-xs border-b border-gray-800",
                                tr {
                                    th { class: "p-3", "Nível" }
                                    th { class: "p-3", "Small / Big Blind" }
                                    th { class: "p-3", "Ante" }
                                    th { class: "p-3", "Duração" }
                                }
                            }
                            tbody { class: "divide-y divide-gray-800",
                                for level in blinds.read().clone() {
                                    tr {
                                        key: "{level.level}",
                                        class: if level.level == current_level {
                                            "hover:bg-gray-800/40 bg-amber-500/5 text-amber-300 font-semibold"
                                        } else {
                                            "hover:bg-gray-800/40"
                                        },
                                        td { class: "p-3", "Nível {level.level}" }
                                        td { class: "p-3", "{level.small_blind} / {level.big_blind}" }
                                        td { class: "p-3",
                                            {
                                                if level.ante == 0 {
                                                    "-".to_string()
                                                } else {
                                                    level.ante.to_string()
                                                }
                                            }
                                        }
                                        td { class: "p-3", "{level.duration_minutes} min" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "bg-gray-900/80 p-6 rounded-2xl border border-gray-800 shadow-xl space-y-4",
                    h2 { class: "text-xl font-bold text-white flex items-center gap-2", "🎁 Distribuição de Premiação (Prizepool)" }
                    p { class: "text-gray-400 text-sm", "Top 15% dos colocados entram na faixa de premiação (In The Money)." }

                    div { class: "overflow-x-auto",
                        table { class: "w-full text-left text-sm text-gray-300",
                            thead { class: "bg-gray-950 text-gray-400 uppercase text-xs border-b border-gray-800",
                                tr {
                                    th { class: "p-3", "Posição" }
                                    th { class: "p-3", "Percentual" }
                                    th { class: "p-3", "Prêmio Estimado" }
                                }
                            }
                            tbody { class: "divide-y divide-gray-800",
                                for (idx, prize) in prizes.read().clone().into_iter().enumerate() {
                                    tr {
                                        key: "{idx}",
                                        class: match prize.highlight {
                                            PrizeHighlight::Gold => "hover:bg-gray-800/40 font-bold text-yellow-400",
                                            PrizeHighlight::Silver => "hover:bg-gray-800/40 font-semibold text-gray-200",
                                            PrizeHighlight::Bronze => "hover:bg-gray-800/40 text-amber-400 font-semibold",
                                            PrizeHighlight::Normal => "hover:bg-gray-800/40",
                                        },
                                        td { class: "p-3", "{prize.position_label}" }
                                        td { class: "p-3", "{prize.percentage}" }
                                        td { class: "p-3 text-emerald-400", "{format_cents_brl(prize.amount_cents)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Lista de Jogadores Inscritos
            div { class: "bg-gray-900/80 p-6 rounded-2xl border border-gray-800 shadow-xl space-y-4",
                div { class: "flex justify-between items-center flex-wrap gap-2",
                    div {
                        h2 { class: "text-xl font-bold text-white flex items-center gap-2", "📋 Jogadores Inscritos" }
                        p { class: "text-gray-400 text-sm",
                            "Estado do torneio: Registering · {registered_count} de {max_players} vagas"
                        }
                    }
                    span { class: "px-3 py-1 rounded-full bg-blue-500/10 text-blue-300 border border-blue-500/20 text-xs font-semibold",
                        "Stack inicial: {format_chips(starting_stack)}"
                    }
                }

                div { class: "overflow-x-auto max-h-80 overflow-y-auto",
                    table { class: "w-full text-left text-sm text-gray-300",
                        thead { class: "bg-gray-950 text-gray-400 uppercase text-xs border-b border-gray-800 sticky top-0",
                            tr {
                                th { class: "p-3", "#" }
                                th { class: "p-3", "Nickname" }
                                th { class: "p-3", "Stack" }
                                th { class: "p-3", "Status" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-800",
                            for player in registered.read().clone() {
                                tr {
                                    key: "{player.seat_or_order}-{player.nickname}",
                                    class: if player.nickname == "Você" {
                                        "hover:bg-gray-800/40 bg-emerald-500/5"
                                    } else {
                                        "hover:bg-gray-800/40"
                                    },
                                    td { class: "p-3 font-mono text-gray-500", "{player.seat_or_order}" }
                                    td { class: "p-3 font-semibold text-white", "{player.nickname}" }
                                    td { class: "p-3 text-yellow-300", "{format_chips(player.stack)}" }
                                    td { class: "p-3",
                                        span { class: "px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 text-xs font-semibold",
                                            "{player.status}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Toast de Feedback
            if !status_msg.read().is_empty() {
                div { class: "p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 text-sm font-semibold text-center animate-fade-in",
                    "{status_msg}"
                }
            }
        }
    }
}
