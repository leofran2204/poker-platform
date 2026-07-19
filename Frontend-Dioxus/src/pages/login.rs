//! Página de Login — autenticação de usuário.
//!
//! Gerencia o fluxo completo de autenticação:
//! 1. Login (username + password)
//! 2. MFA (código TOTP de 6 dígitos, se exigido pelo servidor)
//! 3. Redirecionamento para /lobby em caso de sucesso
//!
//! Usa os componentes `LoginForm`, `MfaInput` e `RegisterForm`.

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::api_client;
use crate::components::login_form::LoginForm;
use crate::components::mfa_input::MfaInput;
use crate::router::Route;

/// Estados do fluxo de autenticação.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum AuthFlow {
    /// Tela inicial de login (username + password).
    Login,
    /// Servidor exigiu MFA — mostrar input de código TOTP.
    MfaRequired { username: String, password: String },
    /// Login concluído com sucesso — redirecionando.
    Authenticated,
}

/// Página de login com fluxo completo (Login → MFA → Redirecionamento).
///
/// # Fluxo
///
/// ```text
/// LoginForm (username+password)
///   → POST /auth/login
///     → success → redireciona para /lobby
///     → mfa_required → MfaInput (código TOTP)
///       → POST /auth/mfa/verify
///         → success → redireciona para /lobby
///         → failure → mensagem de erro, volta para MfaInput
///     → failure → mensagem de erro, permanece no LoginForm
/// ```
#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let mut flow = use_signal(|| AuthFlow::Login);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // --- Handler: submit do login ---
    let on_login_submit = move |(username, password): (String, String)| {
        let nav = navigator;
        let mut flow_clone = flow;
        let mut error_clone = error_msg;

        spawn(async move {
            let req = api_client::LoginRequest {
                email: username.clone(),
                password: password.clone(),
            };

            match api_client::login(&req).await {
                Ok(resp) => {
                    // Salva tokens
                    api_client::save_tokens(&resp.token, &resp.refresh_token);
                    flow_clone.set(AuthFlow::Authenticated);
                    nav.push(Route::Lobby {});
                }
                Err(err) => {
                    // Verifica se é erro de MFA
                    if err.contains("mfa") || err.contains("MFA") {
                        flow_clone.set(AuthFlow::MfaRequired {
                            username: username.clone(),
                            password: password.clone(),
                        });
                        error_clone.set(None);
                    } else {
                        error_clone.set(Some(err));
                    }
                }
            }
        });
    };

    // --- Handler: verificação MFA ---
    let on_mfa_verify = move |code: String| {
        let nav = navigator;
        let mut error_clone = error_msg;

        spawn(async move {
            let req = api_client::MfaVerifyRequest { code };

            match api_client::mfa_verify(&req).await {
                Ok(resp) => {
                    api_client::save_tokens(&resp.token, &resp.refresh_token);
                    nav.push(Route::Lobby {});
                }
                Err(err) => {
                    error_clone.set(Some(err));
                }
            }
        });
    };

    // --- Handler: cancelar MFA (voltar ao login) ---
    let on_mfa_cancel = move |_| {
        flow.set(AuthFlow::Login);
        error_msg.set(None);
    };

    rsx! {
        div {
            class: "auth-page",

            div {
                class: "auth-page-inner",

                match flow.read().clone() {
                    AuthFlow::Login => {
                        rsx! {
                            LoginForm {
                                on_submit: on_login_submit,
                                footer_link: rsx! {
                                    span {
                                        class: "auth-link-text",
                                        "Não tem conta? "
                                        Link {
                                            to: Route::Register {},
                                            class: "auth-link",
                                            "Criar conta"
                                        }
                                    }
                                },
                            }

                            // Mensagem de erro do servidor (se houver)
                            if let Some(msg) = error_msg.read().clone() {
                                div {
                                    class: "auth-error auth-error-banner",
                                    span { class: "auth-error-icon", "⚠️" }
                                    span { "{msg}" }
                                }
                            }
                        }
                    }

                    AuthFlow::MfaRequired { .. } => {
                        rsx! {
                            MfaInput {
                                on_verify: on_mfa_verify,
                                on_cancel: on_mfa_cancel,
                            }

                            if let Some(msg) = error_msg.read().clone() {
                                div {
                                    class: "auth-error auth-error-banner",
                                    span { class: "auth-error-icon", "⚠️" }
                                    span { "{msg}" }
                                }
                            }
                        }
                    }

                    AuthFlow::Authenticated => {
                        rsx! {
                            div {
                                class: "auth-card",
                                h2 {
                                    class: "auth-title",
                                    "✅ Autenticado!"
                                }
                                p {
                                    class: "auth-success-text",
                                    "Redirecionando para o lobby..."
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifica que AuthFlow implementa Clone + Debug + PartialEq.
    #[test]
    fn test_auth_flow_clone_eq() {
        let a = AuthFlow::Login;
        let b = a.clone();
        assert_eq!(a, b);

        let mfa = AuthFlow::MfaRequired {
            username: "player1".into(),
            password: "secret123".into(),
        };
        let mfa2 = mfa.clone();
        assert_eq!(mfa, mfa2);
    }

    /// Verifica que AuthFlow::Login != AuthFlow::Authenticated.
    #[test]
    fn test_auth_flow_variants_differ() {
        assert_ne!(AuthFlow::Login, AuthFlow::Authenticated);
    }

    /// Verifica que MfaRequired com dados diferentes não são iguais.
    #[test]
    fn test_mfa_required_different_data() {
        let a = AuthFlow::MfaRequired {
            username: "a".into(),
            password: "p1".into(),
        };
        let b = AuthFlow::MfaRequired {
            username: "b".into(),
            password: "p2".into(),
        };
        assert_ne!(a, b);
    }
}
