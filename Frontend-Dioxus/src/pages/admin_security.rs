//! Painel de Segurança e Antifraude Administrativo — Dioxus Component
use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[component]
pub fn AdminSecurityPage() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-slate-950 text-white p-6 font-sans",
            div {
                class: "max-w-6xl mx-auto space-y-6",
                div {
                    class: "flex items-center justify-between border-b border-slate-800 pb-4",
                    div {
                        h1 {
                            class: "text-2xl font-bold text-emerald-400 flex items-center gap-2",
                            "🛡️ Painel de Segurança & Antifraude (Real-Time)"
                        }
                        p {
                            class: "text-xs text-slate-400 mt-1",
                            "Monitoramento de integridade do motor de poker, detecção de bots e prevenções de fraude."
                        }
                    }
                    Link {
                        to: Route::Lobby {},
                        class: "text-xs px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition-colors",
                        "← Voltar ao Lobby"
                    }
                }

                // Grid de métricas
                div {
                    class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                    div {
                        class: "bg-slate-900 border border-slate-800 rounded-lg p-4 shadow-sm",
                        div { class: "text-xs text-slate-400 font-semibold", "STATUS DO MOTOR" }
                        div { class: "text-lg font-bold text-emerald-400 mt-1 flex items-center gap-1.5", "🟢 SAUDÁVEL" }
                        div { class: "text-[10px] text-slate-500 mt-1", "0 erros detectados nas últimas 24h" }
                    }
                    div {
                        class: "bg-slate-900 border border-slate-800 rounded-lg p-4 shadow-sm",
                        div { class: "text-xs text-slate-400 font-semibold", "SUSPEITAS DE BOTS" }
                        div { class: "text-lg font-bold text-yellow-400 mt-1", "0" }
                        div { class: "text-[10px] text-slate-500 mt-1", "Análise de variância de tempo < 0.2s" }
                    }
                    div {
                        class: "bg-slate-900 border border-slate-800 rounded-lg p-4 shadow-sm",
                        div { class: "text-xs text-slate-400 font-semibold", "ALERTAS DE COLUSÃO" }
                        div { class: "text-lg font-bold text-emerald-400 mt-1", "0" }
                        div { class: "text-[10px] text-slate-500 mt-1", "Monitoramento de duplas em mesas" }
                    }
                    div {
                        class: "bg-slate-900 border border-slate-800 rounded-lg p-4 shadow-sm",
                        div { class: "text-xs text-slate-400 font-semibold", "CHIP DUMPING" }
                        div { class: "text-lg font-bold text-emerald-400 mt-1", "0" }
                        div { class: "text-[10px] text-slate-500 mt-1", "Filtro de transferências atípicas" }
                    }
                }

                // Tabela de alertas recentes
                div {
                    class: "bg-slate-900 border border-slate-800 rounded-lg overflow-hidden shadow-sm",
                    div {
                        class: "px-4 py-3 border-b border-slate-800 flex items-center justify-between",
                        h3 { class: "text-sm font-semibold text-slate-200", "📋 Logs de Auditoria & Registro de Alertas" }
                        span { class: "text-[10px] text-slate-400 bg-slate-800 px-2 py-0.5 rounded", "Sincronizado" }
                    }
                    div {
                        class: "p-4 text-center text-xs text-slate-500 italic",
                        "Nenhum alerta de alta severidade registrado nas últimas execuções."
                    }
                }
            }
        }
    }
}
