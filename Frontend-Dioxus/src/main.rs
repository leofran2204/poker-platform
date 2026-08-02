//! Ponto de entrada do Front-end Dioxus (WebAssembly).
//!
//! Inicializa o logger e monta o componente raiz (`Root`) que contém
//! o `Router` com todas as rotas da aplicação.
//!
//! # Estrutura
//!
//! - `Root` (em `router.rs`) → Navbar + Router
//! - `Router` → renderiza a página correspondente à rota atual
//!
//! # Rotas
//!
//! - `/` → `Home`
//! - `/login` → `Login`
//! - `/register` → `Register`
//! - `/lobby` → `Lobby`
//! - `/table/:id` → `Table`
//! - `/tournament/:id` → `TournamentLobby` (MTT)
//! - `/admin/security` → antifraude
//! - `/admin/clubs` → dashboard B2B (HTTPS + JWT admin)

// Itens públicos dos módulos (api_client, ws_client, components)
// fazem parte da API e serão usados conforme as telas evoluem.
#![allow(dead_code)]

mod api_client;
mod audio;
mod components;
mod pages;
mod router;
mod ws_client;

use dioxus::prelude::*;

/// Resolve o tema baseado no domínio (Multi-tenant White Label).
fn use_theme_injector() {
    use_effect(|| {
        if let Some(window) = web_sys::window() {
            if let Ok(location) = window.location().hostname() {
                // Mock da chamada à API para pegar o custom_theme_json do clube.
                // Na prática: fetch(`/api/public/clubs?domain=${location}`)
                let (primary_color, bg_color) = if location.contains("luxurypoker") {
                    ("#D4AF37", "#0a0a0a") // Dourado e preto
                } else if location.contains("neonpoker") {
                    ("#00FF00", "#111122") // Verde neon e azul escuro
                } else {
                    ("#3b82f6", "#0f172a") // Padrão azul e slate
                };

                // Injeta as variáveis de CSS root dinamicamente.
                if let Some(document) = window.document() {
                    if let Ok(style) = document.create_element("style") {
                        style.set_inner_html(&format!(
                            ":root {{ --primary-color: {}; --bg-color: {}; }}",
                            primary_color, bg_color
                        ));
                        if let Some(head) = document.head() {
                            let _ = head.append_child(&style);
                        }
                    }
                }
            }
        }
    });
}

/// Componente raiz — delega para `router::root` que contém o `Router`.
///
/// Separado para manter `main.rs` limpo e focado em inicialização.
fn app() -> Element {
    // Aplica o tema B2B dinâmico.
    use_theme_injector();
    
    router::root()
}

/// Função principal — inicializa o logger e lança o app Dioxus.
fn main() {
    // Não panica se o logger falhar (evita tela em branco no browser).
    let _ = dioxus_logger::init(dioxus_logger::tracing::Level::INFO);

    log::info!("🃏 Poker Project — Front-end Dioxus iniciando...");
    log::info!("📡 Motor Rust · API Axum · B2B White-Label Ready");

    // Lança o app no navegador (target WASM).
    launch(app);
}
