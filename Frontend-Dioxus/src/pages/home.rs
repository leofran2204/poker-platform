//! Página inicial (Home) — landing page da aplicação.
//!
//! Apresenta o projeto e oferece botões de navegação para Lobby e Login.

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::router::Route;

/// Componente da página inicial.
#[component]
pub fn Home() -> Element {
    rsx! {
        main {
            class: "container mx-auto px-6 py-12 max-w-4xl",
            div {
                class: "text-center space-y-8",
                div {
                    class: "space-y-4",
                    h1 {
                        class: "text-6xl font-bold text-yellow-400 drop-shadow-lg",
                        "🃏 Poker Project"
                    }
                    p {
                        class: "text-xl text-green-100",
                        "Front-end Rust com Dioxus + WebAssembly"
                    }
                    p {
                        class: "text-sm text-green-300/70 italic",
                        "Motor de poker 100% Rust · 496 testes · 0 warnings"
                    }
                }

                div {
                    class: "flex flex-col sm:flex-row gap-4 justify-center pt-8",
                    Link {
                        to: Route::Lobby {},
                        class: "px-8 py-4 bg-yellow-500 hover:bg-yellow-400 text-green-950 font-bold rounded-lg shadow-lg transition-all hover:scale-105",
                        "🎪 Entrar no Lobby"
                    }
                    Link {
                        to: Route::Login {},
                        class: "px-8 py-4 bg-green-700 hover:bg-green-600 text-white font-bold rounded-lg shadow-lg transition-all hover:scale-105",
                        "🔐 Fazer Login"
                    }
                }

                div {
                    class: "pt-12 grid grid-cols-1 md:grid-cols-3 gap-6 text-left",
                    FeatureCard {
                        icon: "⚡",
                        title: "Motor Rust",
                        description: "Engine de poker de alta performance com 484 testes unitários e de propriedade."
                    }
                    FeatureCard {
                        icon: "🌐",
                        title: "API Axum",
                        description: "Backend REST + WebSocket com autenticação JWT e persistência PostgreSQL."
                    }
                    FeatureCard {
                        icon: "🛡️",
                        title: "Antifraude",
                        description: "Detecção de bots, chip dumping, conluio e multi-contas em tempo real."
                    }
                }
            }
        }
    }
}

/// Card de feature reutilizável na home.
#[component]
fn FeatureCard(icon: String, title: String, description: String) -> Element {
    rsx! {
        div {
            class: "bg-green-950/50 backdrop-blur-sm border border-green-700/50 rounded-lg p-6 hover:border-yellow-500/50 transition-colors",
            div {
                class: "text-4xl mb-3",
                "{icon}"
            }
            h3 {
                class: "text-lg font-bold text-yellow-400 mb-2",
                "{title}"
            }
            p {
                class: "text-sm text-green-200",
                "{description}"
            }
        }
    }
}
