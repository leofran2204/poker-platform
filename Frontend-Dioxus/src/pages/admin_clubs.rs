//! Página de Dashboard Administrativo B2B para Gestão de Clubes
//!
//! Consome a API Axum via **HTTPS** (same-origin + JWT admin):
//! financials, saque, tema white-label e agentes/rakeback.

use dioxus::prelude::*;

use crate::api_client::{
    self, ClubAgentResponse, ClubFinancialsResponse, ClubResponse, CreateClubAgentRequest,
    UpdateClubThemeRequest, WithdrawClubBalanceRequest,
};

/// Agente do clube (espelha o contrato `ClubAgent` da API).
#[derive(Clone, PartialEq)]
struct AgentRow {
    agent_id: String,
    name: String,
    rakeback_percentage: u8,
    total_players_referred: u32,
    /// Comissão acumulada em centavos inteiros.
    total_commission_earned: u64,
}

fn format_cents_brl(cents: i64) -> String {
    let negative = cents < 0;
    let abs = cents.unsigned_abs();
    let reais = abs / 100;
    let frac = abs % 100;
    if negative {
        format!("-R$ {reais},{frac:02}")
    } else {
        format!("R$ {reais},{frac:02}")
    }
}

fn format_cents_brl_u64(cents: u64) -> String {
    format_cents_brl(cents as i64)
}

/// Converte texto de reais (`1500`, `1500.50`, `1.500,50`) em centavos.
fn parse_reais_to_cents(input: &str) -> Result<u64, String> {
    let s = input.trim().replace("R$", "").replace(' ', "");
    if s.is_empty() {
        return Err("Valor vazio".to_string());
    }
    // Aceita "1500.50" ou "1500,50"
    let normalized = if s.contains(',') && s.contains('.') {
        // 1.500,50 → remove pontos de milhar
        s.replace('.', "").replace(',', ".")
    } else if s.contains(',') {
        s.replace(',', ".")
    } else {
        s
    };
    let parts: Vec<&str> = normalized.split('.').collect();
    match parts.as_slice() {
        [whole] => {
            let reais: u64 = whole
                .parse()
                .map_err(|_| "Valor inválido".to_string())?;
            Ok(reais.saturating_mul(100))
        }
        [whole, frac] => {
            let reais: u64 = if whole.is_empty() {
                0
            } else {
                whole.parse().map_err(|_| "Valor inválido".to_string())?
            };
            let frac_digits = if frac.len() >= 2 {
                &frac[..2]
            } else {
                frac
            };
            let mut frac_val: u64 = frac_digits
                .parse()
                .map_err(|_| "Centavos inválidos".to_string())?;
            if frac_digits.len() == 1 {
                frac_val *= 10;
            }
            Ok(reais.saturating_mul(100).saturating_add(frac_val))
        }
        _ => Err("Formato de valor inválido".to_string()),
    }
}

fn agent_from_api(a: ClubAgentResponse) -> AgentRow {
    AgentRow {
        agent_id: a.agent_id,
        name: a.name,
        rakeback_percentage: a.rakeback_percentage,
        total_players_referred: a.total_players_referred,
        total_commission_earned: a.total_commission_earned,
    }
}

fn demo_agents() -> Vec<AgentRow> {
    vec![
        AgentRow {
            agent_id: "ag_101".to_string(),
            name: "Carlos (Agente SP)".to_string(),
            rakeback_percentage: 20,
            total_players_referred: 14,
            total_commission_earned: 85_000,
        },
        AgentRow {
            agent_id: "ag_102".to_string(),
            name: "Mariana (Agente RJ)".to_string(),
            rakeback_percentage: 25,
            total_players_referred: 22,
            total_commission_earned: 142_000,
        },
    ]
}

fn demo_financials() -> ClubFinancialsResponse {
    ClubFinancialsResponse {
        club_id: "demo".to_string(),
        name: "Clube Demo".to_string(),
        balance: 425_000,
        total_rake_generated: 500_000,
        net_club_rake: 425_000,
        platform_fee_paid: 75_000,
    }
}

#[component]
pub fn AdminClubsPage() -> Element {
    let mut selected_primary_color = use_signal(|| "#3b82f6".to_string());
    let mut selected_bg_color = use_signal(|| "#0f172a".to_string());
    let mut withdraw_amount = use_signal(|| "".to_string());
    let mut pix_key = use_signal(|| "".to_string());
    let mut feedback_msg = use_signal(|| "".to_string());
    let mut loading = use_signal(|| true);
    let mut live_https = use_signal(|| false);
    let mut club_id = use_signal(|| Option::<String>::None);
    let mut club_name = use_signal(|| "—".to_string());
    let mut financials = use_signal(demo_financials);
    let mut agents = use_signal(demo_agents);
    let mut agent_name = use_signal(|| "".to_string());
    let mut agent_rakeback = use_signal(|| "20".to_string());
    let mut show_agent_form = use_signal(|| false);
    let mut busy = use_signal(|| false);

    // Carrega clubes + financials + agents via HTTPS ao montar.
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            if !api_client::is_authenticated() {
                live_https.set(false);
                feedback_msg.set(
                    "⚠️ Sem JWT: exibindo dados demo. Faça login como admin para carregar via HTTPS."
                        .to_string(),
                );
                loading.set(false);
                return;
            }

            match api_client::list_admin_clubs().await {
                Ok(clubs) => {
                    if let Some(club) = pick_first_club(&clubs) {
                        let id = club.id.clone().unwrap_or_default();
                        club_id.set(Some(id.clone()));
                        club_name.set(club.name.clone());
                        apply_theme_from_json(&club.custom_theme_json, &mut selected_primary_color, &mut selected_bg_color);

                        match api_client::get_club_financials(&id).await {
                            Ok(fin) => financials.set(fin),
                            Err(e) => feedback_msg.set(format!(
                                "⚠️ Clube carregado; falha nos financials HTTPS: {e}"
                            )),
                        }

                        match api_client::list_club_agents(&id).await {
                            Ok(list) => {
                                agents.set(list.into_iter().map(agent_from_api).collect());
                            }
                            Err(e) => feedback_msg.set(format!(
                                "⚠️ Clube carregado; falha ao listar agentes HTTPS: {e}"
                            )),
                        }

                        live_https.set(true);
                        feedback_msg.set(format!(
                            "✅ Dados do clube \"{}\" carregados via HTTPS.",
                            club.name
                        ));
                    } else {
                        live_https.set(false);
                        feedback_msg.set(
                            "ℹ️ Nenhum clube cadastrado. Exibindo demo local."
                                .to_string(),
                        );
                    }
                }
                Err(e) => {
                    live_https.set(false);
                    feedback_msg.set(format!(
                        "⚠️ Falha HTTPS ao listar clubes ({e}). Exibindo demo local."
                    ));
                }
            }
            loading.set(false);
        });
    });

    let apply_theme = move |_| {
        let primary = selected_primary_color.read().clone();
        let bg = selected_bg_color.read().clone();
        inject_css_vars(&primary, &bg);

        let Some(id) = club_id.read().clone() else {
            feedback_msg.set(
                "✅ Tema aplicado localmente (sem clube HTTPS selecionado).".to_string(),
            );
            return;
        };
        if !*live_https.read() {
            feedback_msg.set(
                "✅ Tema aplicado localmente (modo demo).".to_string(),
            );
            return;
        }

        busy.set(true);
        spawn(async move {
            let body = UpdateClubThemeRequest {
                custom_theme_json: serde_json::json!({
                    "primary_color": primary,
                    "bg_color": bg,
                }),
            };
            match api_client::update_club_theme(&id, &body).await {
                Ok(_) => feedback_msg
                    .set("✅ Tema salvo via HTTPS e aplicado no browser.".to_string()),
                Err(e) => feedback_msg.set(format!("❌ Falha ao salvar tema HTTPS: {e}")),
            }
            busy.set(false);
        });
    };

    let handle_withdraw = move |_| {
        let amount_raw = withdraw_amount.read().clone();
        let key = pix_key.read().trim().to_string();
        if amount_raw.is_empty() || key.is_empty() {
            feedback_msg
                .set("⚠️ Preencha o valor e a chave PIX para solicitar o saque.".to_string());
            return;
        }
        let cents = match parse_reais_to_cents(&amount_raw) {
            Ok(c) if c > 0 => c,
            Ok(_) => {
                feedback_msg.set("⚠️ Valor deve ser maior que zero.".to_string());
                return;
            }
            Err(e) => {
                feedback_msg.set(format!("⚠️ {e}"));
                return;
            }
        };

        let Some(id) = club_id.read().clone() else {
            feedback_msg.set(format!(
                "🚀 Demo: saque de {} solicitado via PIX para {} (sem HTTPS).",
                format_cents_brl_u64(cents),
                key
            ));
            withdraw_amount.set("".to_string());
            pix_key.set("".to_string());
            return;
        };
        if !*live_https.read() {
            feedback_msg.set(format!(
                "🚀 Demo: saque de {} para {} (sem ligação HTTPS).",
                format_cents_brl_u64(cents),
                key
            ));
            withdraw_amount.set("".to_string());
            pix_key.set("".to_string());
            return;
        }

        busy.set(true);
        spawn(async move {
            let body = WithdrawClubBalanceRequest {
                amount: cents,
                pix_key: key.clone(),
            };
            match api_client::withdraw_club_balance(&id, &body).await {
                Ok(_) => {
                    feedback_msg.set(format!(
                        "✅ Saque de {} solicitado via HTTPS para {key}",
                        format_cents_brl_u64(cents)
                    ));
                    withdraw_amount.set("".to_string());
                    pix_key.set("".to_string());
                    // Atualiza financials
                    if let Ok(fin) = api_client::get_club_financials(&id).await {
                        financials.set(fin);
                    }
                }
                Err(e) => feedback_msg.set(format!("❌ Falha no saque HTTPS: {e}")),
            }
            busy.set(false);
        });
    };

    let handle_create_agent = move |_| {
        let name = agent_name.read().trim().to_string();
        if name.is_empty() || name.len() > 100 {
            feedback_msg.set("⚠️ Nome do agente deve ter entre 1 e 100 caracteres.".to_string());
            return;
        }
        let pct: u8 = match agent_rakeback.read().trim().parse() {
            Ok(v) if v <= 50 => v,
            _ => {
                feedback_msg.set(
                    "⚠️ Percentual de rakeback deve ser um número inteiro de 0 a 50.".to_string(),
                );
                return;
            }
        };

        let Some(id) = club_id.read().clone() else {
            // Demo local
            let new_id = format!("ag_{:x}", agents.read().len() + 200);
            agents.write().insert(
                0,
                AgentRow {
                    agent_id: new_id.clone(),
                    name: name.clone(),
                    rakeback_percentage: pct,
                    total_players_referred: 0,
                    total_commission_earned: 0,
                },
            );
            agent_name.set("".to_string());
            agent_rakeback.set("20".to_string());
            show_agent_form.set(false);
            feedback_msg.set(format!(
                "✅ Demo: agente \"{name}\" com {pct}% (id {new_id}) — sem HTTPS."
            ));
            return;
        };

        if !*live_https.read() {
            feedback_msg.set(
                "⚠️ Sem sessão HTTPS ativa; cadastro de agente só em demo local.".to_string(),
            );
            return;
        }

        busy.set(true);
        spawn(async move {
            let body = CreateClubAgentRequest {
                name: name.clone(),
                rakeback_percentage: pct,
            };
            match api_client::create_club_agent(&id, &body).await {
                Ok(created) => {
                    agents.write().insert(0, agent_from_api(created));
                    agent_name.set("".to_string());
                    agent_rakeback.set("20".to_string());
                    show_agent_form.set(false);
                    feedback_msg.set(format!(
                        "✅ Agente \"{name}\" cadastrado via HTTPS com {pct}% de rakeback."
                    ));
                }
                Err(e) => feedback_msg.set(format!("❌ Falha ao cadastrar agente HTTPS: {e}")),
            }
            busy.set(false);
        });
    };

    let fin = financials.read().clone();
    let status_badge = if *live_https.read() {
        "HTTPS · Live"
    } else {
        "Demo local"
    };

    rsx! {
        div { class: "max-w-6xl mx-auto p-6 space-y-8",
            // Cabeçalho
            div { class: "flex justify-between items-center bg-gray-900/80 backdrop-blur p-6 rounded-2xl border border-gray-800 shadow-xl flex-wrap gap-3",
                div {
                    h1 { class: "text-3xl font-extrabold text-transparent bg-clip-text bg-gradient-to-r from-yellow-400 to-amber-500",
                        "🏛️ Dashboard B2B — Gestão do Clube"
                    }
                    p { class: "text-gray-400 text-sm mt-1",
                        "Clube: {club_name} · Painel financeiro, agentes e White-Label via HTTPS"
                    }
                }
                span {
                    class: if *live_https.read() {
                        "px-4 py-1.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-xs font-semibold uppercase tracking-wider"
                    } else {
                        "px-4 py-1.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 text-xs font-semibold uppercase tracking-wider"
                    },
                    "{status_badge}"
                }
            }

            if *loading.read() {
                div { class: "text-center py-6 text-gray-400", "Carregando dados B2B via HTTPS..." }
            }

            // Cards de Métricas Financeiras (KPIs)
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-6",
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Saldo Acumulado" }
                    p { class: "text-3xl font-black text-emerald-400 mt-2",
                        "{format_cents_brl(fin.balance)}"
                    }
                    p { class: "text-xs text-gray-500 mt-1", "Disponível para saque" }
                }
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Rake Bruto Gerado" }
                    p { class: "text-3xl font-black text-white mt-2",
                        "{format_cents_brl(fin.total_rake_generated)}"
                    }
                    p { class: "text-xs text-gray-500 mt-1", "Total de mesas do clube" }
                }
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Lucro do Clube (85%)" }
                    p { class: "text-3xl font-black text-yellow-400 mt-2",
                        "{format_cents_brl(fin.net_club_rake)}"
                    }
                    p { class: "text-xs text-gray-500 mt-1", "Sua fatia B2B" }
                }
                div { class: "bg-gray-900/60 p-5 rounded-xl border border-gray-800 shadow-lg",
                    p { class: "text-gray-400 text-xs uppercase font-medium", "Fee Plataforma (15%)" }
                    p { class: "text-3xl font-black text-blue-400 mt-2",
                        "{format_cents_brl(fin.platform_fee_paid)}"
                    }
                    p { class: "text-xs text-gray-500 mt-1", "Repasse Zerotilt" }
                }
            }

            // Bloco de Ações
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-8",
                div { class: "bg-gray-900/80 p-6 rounded-2xl border border-gray-800 shadow-xl space-y-4",
                    h2 { class: "text-xl font-bold text-white flex items-center gap-2",
                        "💸 Solicitar Saque de Comissões"
                    }
                    p { class: "text-gray-400 text-sm",
                        "Transfira o saldo do clube via PIX (valor em R$; API recebe centavos por HTTPS)."
                    }

                    div { class: "space-y-3 pt-2",
                        div {
                            label { class: "text-xs text-gray-400 font-medium block mb-1", "Valor (R$)" }
                            input {
                                class: "w-full bg-gray-950 border border-gray-800 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:border-yellow-500",
                                placeholder: "ex: 1500.00",
                                value: "{withdraw_amount}",
                                oninput: move |e| withdraw_amount.set(e.value())
                            }
                        }
                        div {
                            label { class: "text-xs text-gray-400 font-medium block mb-1", "Chave PIX (CPF/CNPJ/E-mail/EVP)" }
                            input {
                                class: "w-full bg-gray-950 border border-gray-800 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:border-yellow-500",
                                placeholder: "Sua chave PIX",
                                value: "{pix_key}",
                                oninput: move |e| pix_key.set(e.value())
                            }
                        }
                        button {
                            class: "w-full bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-bold py-3 rounded-lg shadow-lg transition-all disabled:opacity-50",
                            disabled: *busy.read(),
                            onclick: handle_withdraw,
                            "Confirmar Solicitação de Saque"
                        }
                    }
                }

                div { class: "bg-gray-900/80 p-6 rounded-2xl border border-gray-800 shadow-xl space-y-4",
                    h2 { class: "text-xl font-bold text-white flex items-center gap-2",
                        "🎨 Personalização White-Label"
                    }
                    p { class: "text-gray-400 text-sm", "Ajuste a identidade visual e persista via HTTPS no clube." }

                    div { class: "space-y-4 pt-2",
                        div { class: "flex items-center justify-between",
                            span { class: "text-sm text-gray-300", "Cor Primária (Destaques)" }
                            input {
                                r#type: "color",
                                class: "bg-transparent cursor-pointer h-10 w-16 rounded border border-gray-700",
                                value: "{selected_primary_color}",
                                oninput: move |e| selected_primary_color.set(e.value())
                            }
                        }
                        div { class: "flex items-center justify-between",
                            span { class: "text-sm text-gray-300", "Cor de Fundo (Background)" }
                            input {
                                r#type: "color",
                                class: "bg-transparent cursor-pointer h-10 w-16 rounded border border-gray-700",
                                value: "{selected_bg_color}",
                                oninput: move |e| selected_bg_color.set(e.value())
                            }
                        }
                        button {
                            class: "w-full bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white font-bold py-3 rounded-lg shadow-lg transition-all disabled:opacity-50",
                            disabled: *busy.read(),
                            onclick: apply_theme,
                            "Salvar e Aplicar Tema"
                        }
                    }
                }
            }

            // Gestão de Agentes & Rakeback
            div { class: "bg-gray-900/80 p-6 rounded-2xl border border-gray-800 shadow-xl space-y-6",
                div { class: "flex justify-between items-center flex-wrap gap-3",
                    div {
                        h2 { class: "text-xl font-bold text-white flex items-center gap-2", "👥 Agentes do Clube & Comissões (Rakeback)" }
                        p { class: "text-gray-400 text-sm", "Cadastro e listagem via HTTPS (GET/POST /api/admin/clubs/:id/agents)." }
                    }
                    button {
                        class: "px-4 py-2 bg-amber-500/10 text-amber-400 border border-amber-500/30 rounded-lg hover:bg-amber-500/20 text-sm font-semibold transition-all",
                        onclick: move |_| {
                            // Split read/write across statements to avoid E0502.
                            let next = !*show_agent_form.read();
                            show_agent_form.set(next);
                        },
                        if *show_agent_form.read() { "✖ Fechar formulário" } else { "➕ Cadastrar Novo Agente" }
                    }
                }

                if *show_agent_form.read() {
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 p-4 rounded-xl bg-gray-950/80 border border-gray-800",
                        div {
                            label { class: "text-xs text-gray-400 font-medium block mb-1", "Nome / Região do Agente" }
                            input {
                                class: "w-full bg-gray-900 border border-gray-800 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:border-amber-500",
                                placeholder: "ex: Ana (Agente BH)",
                                value: "{agent_name}",
                                oninput: move |e| agent_name.set(e.value())
                            }
                        }
                        div {
                            label { class: "text-xs text-gray-400 font-medium block mb-1", "Rakeback (%)" }
                            input {
                                class: "w-full bg-gray-900 border border-gray-800 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:border-amber-500",
                                placeholder: "0–50",
                                r#type: "number",
                                min: "0",
                                max: "50",
                                value: "{agent_rakeback}",
                                oninput: move |e| agent_rakeback.set(e.value())
                            }
                        }
                        div { class: "flex items-end",
                            button {
                                class: "w-full bg-gradient-to-r from-amber-500 to-yellow-600 hover:from-amber-400 hover:to-yellow-500 text-black font-bold py-2.5 rounded-lg shadow-lg transition-all disabled:opacity-50",
                                disabled: *busy.read(),
                                onclick: handle_create_agent,
                                "Registrar Agente"
                            }
                        }
                    }
                }

                div { class: "overflow-x-auto",
                    table { class: "w-full text-left text-sm text-gray-300",
                        thead { class: "bg-gray-950 text-gray-400 uppercase text-xs border-b border-gray-800",
                            tr {
                                th { class: "p-3", "ID Agente" }
                                th { class: "p-3", "Nome / Região" }
                                th { class: "p-3", "Comissão (Rakeback)" }
                                th { class: "p-3", "Jogadores Indicados" }
                                th { class: "p-3", "Comissão Gerada" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-800",
                            if agents.read().is_empty() {
                                tr {
                                    td { class: "p-4 text-gray-500", colspan: "5", "Nenhum agente cadastrado neste clube." }
                                }
                            }
                            for agent in agents.read().clone() {
                                tr { class: "hover:bg-gray-800/40",
                                    key: "{agent.agent_id}",
                                    td { class: "p-3 font-mono text-gray-400", "{agent.agent_id}" }
                                    td { class: "p-3 font-semibold text-white", "{agent.name}" }
                                    td { class: "p-3 text-yellow-400 font-bold", "{agent.rakeback_percentage}%" }
                                    td { class: "p-3", "{agent.total_players_referred} ativos" }
                                    td { class: "p-3 text-emerald-400 font-semibold",
                                        "{format_cents_brl_u64(agent.total_commission_earned)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Feedback Toast
            if !feedback_msg.read().is_empty() {
                div { class: "p-4 rounded-xl bg-yellow-500/10 border border-yellow-500/30 text-yellow-300 text-sm font-semibold text-center animate-fade-in",
                    "{feedback_msg}"
                }
            }
        }
    }
}

fn pick_first_club(clubs: &[ClubResponse]) -> Option<ClubResponse> {
    clubs
        .iter()
        .find(|c| c.id.as_ref().map(|id| !id.is_empty()).unwrap_or(false))
        .cloned()
        .or_else(|| clubs.first().cloned())
}

fn inject_css_vars(primary: &str, bg: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(style) = document.create_element("style") {
                style.set_inner_html(&format!(
                    ":root {{ --primary-color: {}; --bg-color: {}; }}",
                    primary, bg
                ));
                if let Some(head) = document.head() {
                    let _ = head.append_child(&style);
                }
            }
        }
    }
}

fn apply_theme_from_json(
    json: &serde_json::Value,
    primary: &mut Signal<String>,
    bg: &mut Signal<String>,
) {
    if let Some(p) = json.get("primary_color").and_then(|v| v.as_str()) {
        primary.set(p.to_string());
    }
    if let Some(b) = json.get("bg_color").and_then(|v| v.as_str()) {
        bg.set(b.to_string());
    }
    let p = primary.read().clone();
    let b = bg.read().clone();
    inject_css_vars(&p, &b);
}
