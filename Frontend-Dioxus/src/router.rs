//! Módulo de roteamento do Front-end Dioxus
//!
//! Define as rotas da aplicação usando `dioxus-router` 0.6.
//! Cada rota corresponde a uma página (tela) da aplicação de poker.

use dioxus::prelude::*;
use dioxus_router::prelude::*;

// Re-exporta os componentes de página para uso externo.
// Os nomes devem coincidir com as variantes do enum `Route` (convenção do
// `dioxus-router` 0.6: a macro `Routable` chama `Home()`, `Login()`, etc.).
pub use crate::pages::{
    admin_security::AdminSecurityPage as AdminSecurity, home::Home, lobby::Lobby, login::Login,
    register::Register, table::Table,
};

/// Enum que define todas as rotas disponíveis na aplicação.
#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    /// Rota raiz — landing page com botões Jogar/Configurar.
    #[route("/")]
    Home {},

    /// Tela de login/cadastro — autenticação de usuário.
    #[route("/login")]
    Login {},

    /// Tela de registro — criação de nova conta.
    #[route("/register")]
    Register {},

    /// Tela de lobby — listagem de mesas disponíveis.
    #[route("/lobby")]
    Lobby {},

    /// Tela de mesa de poker — recebe `id` da mesa como parâmetro de path.
    #[route("/table/:id")]
    Table { id: String },

    /// Painel de segurança e antifraude administrativo.
    #[route("/admin/security")]
    AdminSecurity {},
}

/// Componente raiz que monta o `Router` com todas as rotas da aplicação.
///
/// Este é o componente que deve ser passado para `launch()` no `main.rs`.
///
/// # Estrutura
///
/// ```text
/// Router
/// ├── navbar (sempre visível)
/// └── <Outlet /> (renderiza a página da rota atual)
/// ```
pub fn root() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-green-900 via-green-800 to-emerald-900 text-white",
            Navbar {}
            Router::<Route> {}
        }
    }
}

/// Barra de navegação superior — sempre visível em todas as rotas.
///
/// Usa `Link` do `dioxus-router` para navegação client-side (sem reload).
#[allow(non_snake_case)]
fn Navbar() -> Element {
    rsx! {
        nav {
            class: "bg-green-950/80 backdrop-blur-sm border-b border-green-700/50 px-6 py-3 flex items-center justify-between shadow-lg",
            div {
                class: "flex items-center gap-2",
                span { class: "text-2xl", "🃏" }
                Link {
                    to: Route::Home {},
                    class: "text-xl font-bold text-yellow-400 hover:text-yellow-300 transition-colors",
                    "Poker Project"
                }
            }
            div {
                class: "flex gap-4 text-sm font-semibold",
                Link {
                    to: Route::Home {},
                    class: "text-green-200 hover:text-white transition-colors",
                    "🏠 Início"
                }
                Link {
                    to: Route::Lobby {},
                    class: "text-green-200 hover:text-white transition-colors",
                    "🎪 Lobby"
                }
                Link {
                    to: Route::Login {},
                    class: "text-green-200 hover:text-white transition-colors",
                    "🔐 Login"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Testa que o enum `Route` pode ser instanciado para todas as variantes.
    #[test]
    fn test_route_variants() {
        let _home = Route::Home {};
        let _login = Route::Login {};
        let _lobby = Route::Lobby {};
        let _table = Route::Table {
            id: "abc-123".to_string(),
        };
    }

    /// Testa que `Route` implementa `Clone` e `PartialEq`.
    #[test]
    fn test_route_clone_eq() {
        let r1 = Route::Table {
            id: "x".to_string(),
        };
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }
}
