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
    admin_clubs::AdminClubsPage as AdminClubs,
    admin_security::AdminSecurityPage as AdminSecurity, home::Home, lobby::Lobby, login::Login,
    register::Register, table::Table, tournament_lobby::TournamentLobbyPage as TournamentLobby,
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

    /// Tela de lobby de torneio multimesas — recebe `id` do torneio como parâmetro.
    #[route("/tournament/:id")]
    TournamentLobby { id: String },

    /// Painel de segurança e antifraude administrativo.
    #[route("/admin/security")]
    AdminSecurity {},

    /// Painel B2B de gestão do clube.
    #[route("/admin/clubs")]
    AdminClubs {},
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
    // Estilos inline: o index.html usa CSS puro (sem Tailwind CDN).
    rsx! {
        div {
            style: "min-height: 100vh; background: linear-gradient(180deg, #1a3a1a 0%, #0f2a0f 100%); color: #e8e0d0;",
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
            style: "background: #0f2a0f; border-bottom: 2px solid #8b6914; padding: 12px 24px; display: flex; align-items: center; justify-content: space-between;",
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                span { style: "font-size: 1.5rem;", "🃏" }
                Link {
                    to: Route::Home {},
                    style: "font-size: 1.25rem; font-weight: 700; color: #d4a843; text-decoration: none;",
                    "Poker Project"
                }
            }
            div {
                style: "display: flex; gap: 16px; font-size: 0.875rem; font-weight: 600;",
                Link {
                    to: Route::Home {},
                    style: "color: #7ab87a; text-decoration: none;",
                    "🏠 Início"
                }
                Link {
                    to: Route::Lobby {},
                    style: "color: #7ab87a; text-decoration: none;",
                    "🎪 Lobby"
                }
                Link {
                    to: Route::AdminClubs {},
                    style: "color: #d4a843; text-decoration: none;",
                    "🏛️ Gestão B2B"
                }
                Link {
                    to: Route::Login {},
                    style: "color: #7ab87a; text-decoration: none;",
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
        let _register = Route::Register {};
        let _lobby = Route::Lobby {};
        let _table = Route::Table {
            id: "abc-123".to_string(),
        };
        let _mtt = Route::TournamentLobby {
            id: "demo-1".to_string(),
        };
        let _admin_sec = Route::AdminSecurity {};
        let _admin_clubs = Route::AdminClubs {};
    }

    /// Testa que `Route` implementa `Clone` e `PartialEq`.
    #[test]
    fn test_route_clone_eq() {
        let r1 = Route::Table {
            id: "x".to_string(),
        };
        let r2 = r1.clone();
        assert_eq!(r1, r2);

        let t1 = Route::TournamentLobby {
            id: "1".to_string(),
        };
        let t2 = t1.clone();
        assert_eq!(t1, t2);
    }
}
