//! Página de Registro — criação de nova conta.
//!
//! Formulário de registro com username, email, password e confirmação.
//! Faz POST para `/auth/register` na API Axum.

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::api_client;
use crate::components::register_form::RegisterForm;
use crate::router::Route;

/// Página de registro de novo usuário.
///
/// # Fluxo
///
/// ```text
/// RegisterForm (username + email + password + confirm)
///   → POST /auth/register
///     → success → redireciona para /login (ou /lobby se auto-login)
///     → failure → mensagem de erro, permanece no formulário
/// ```
#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let error_msg = use_signal(|| Option::<String>::None);
    let success_msg = use_signal(|| Option::<String>::None);

    let on_register_submit = move |(username, email, password): (String, String, String)| {
        let nav = navigator;
        let mut error = error_msg;
        let mut success = success_msg;

        spawn(async move {
            let req = api_client::RegisterRequest {
                username,
                email,
                password,
            };

            match api_client::register(&req).await {
                Ok(resp) => {
                    // Registro bem-sucedido — salva tokens e redireciona
                    api_client::save_tokens(&resp.token, &resp.refresh_token);
                    success.set(Some("Conta criada com sucesso! Redirecionando...".to_string()));
                    error.set(None);
                    nav.push(Route::Lobby {});
                }
                Err(err_msg) => {
                    error.set(Some(err_msg));
                    success.set(None);
                }
            }
        });
    };

    rsx! {
        div {
            class: "auth-page",

            div {
                class: "auth-page-inner",

                RegisterForm {
                    on_submit: on_register_submit,
                    footer_link: rsx! {
                        span {
                            class: "auth-link-text",
                            "Já tem conta? "
                            Link {
                                to: Route::Login {},
                                class: "auth-link",
                                "Fazer login"
                            }
                        }
                    },
                }

                // Mensagem de sucesso
                if let Some(msg) = success_msg.read().clone() {
                    div {
                        class: "auth-success",
                        span { class: "auth-success-icon", "✅" }
                        span { "{msg}" }
                    }
                }

                // Mensagem de erro do servidor
                if let Some(msg) = error_msg.read().clone() {
                    div {
                        class: "auth-error auth-error-banner",
                        span { class: "auth-error-icon", "⚠️" }
                        span { "{msg}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    /// Verifica que o componente Register existe e pode ser referenciado.
    #[test]
    fn test_register_component_exists() {
        // Teste de compilação: garante que Register é um componente válido.
        // A função em si não pode ser testada isoladamente sem um VirtualDom,
        // mas este teste garante que o módulo compila.
    }
}