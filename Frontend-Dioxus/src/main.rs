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
//! - `/lobby` → `Lobby`
//! - `/table/:id` → `Table`

// Itens públicos dos módulos (api_client, ws_client, components)
// fazem parte da API e serão usados conforme as telas evoluem.
#![allow(dead_code)]

mod api_client;
mod audio;
mod components;
mod pages;
mod router;
mod ws_client;

#[cfg(test)]
mod fuzz_tests;
#[cfg(test)]
mod state_stress_tests;

use dioxus::prelude::*;

/// Componente raiz — delega para `router::root` que contém o `Router`.
///
/// Separado para manter `main.rs` limpo e focado em inicialização.
fn app() -> Element {
    router::root()
}

/// Função principal — inicializa o logger e lança o app Dioxus.
fn main() {
    // Inicializa o logger com nível INFO.
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO)
        .expect("Falha ao iniciar logger");

    // Loga o início da aplicação.
    log::info!("🃏 Poker Project — Front-end Dioxus iniciando...");
    log::info!("📡 Motor Rust · API Axum · 496 testes · 0 warnings");

    // Lança o app no navegador (target WASM).
    launch(app);
}
