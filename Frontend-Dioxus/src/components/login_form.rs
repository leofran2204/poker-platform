//! Componente de formulário de login.
//!
//! Formulário reutilizável com campos de username e password,
//! validação client-side e callback de submit.
//!
//! Usado em `pages/login.rs` e futuramente em modais de re-autenticação.

use dioxus::prelude::*;

/// Props do formulário de login.
#[derive(Props, Clone, PartialEq)]
pub struct LoginFormProps {
    /// Callback chamado quando o formulário é submetido com sucesso na validação.
    /// Recebe (username, password).
    pub on_submit: EventHandler<(String, String)>,
    /// Texto opcional do título (default: "🔐 Login").
    #[props(default = "🔐 Login".to_string())]
    pub title: String,
    /// Texto opcional do botão submit (default: "Entrar").
    #[props(default = "Entrar".to_string())]
    pub submit_label: String,
    /// Link opcional para navegação alternativa (ex: "Criar conta").
    #[props(default = None)]
    pub footer_link: Option<Element>,
}

/// Formulário de login com validação client-side.
///
/// # Validações
///
/// - Username: não pode estar vazio
/// - Password: não pode estar vazio
///
/// # Exemplo
///
/// ```rust,ignore
/// use crate::components::login_form::LoginForm;
///
/// rsx! {
///     LoginForm {
///         on_submit: move |(user, pass): (String, String)| {
///             // enviar para API
///         },
///     }
/// }
/// ```
#[component]
pub fn LoginForm(props: LoginFormProps) -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut show_password = use_signal(|| false);

    let on_submit_inner = move |evt: FormEvent| {
        evt.prevent_default();
        let u = username.read().trim().to_string();
        let p = password.read().clone();

        // Validações
        if u.is_empty() {
            error_msg.set(Some("Usuário é obrigatório.".to_string()));
            return;
        }
        if u.len() < 3 {
            error_msg.set(Some("Usuário deve ter pelo menos 3 caracteres.".to_string()));
            return;
        }
        if p.is_empty() {
            error_msg.set(Some("Senha é obrigatória.".to_string()));
            return;
        }
        if p.len() < 6 {
            error_msg.set(Some("Senha deve ter pelo menos 6 caracteres.".to_string()));
            return;
        }

        error_msg.set(None);
        props.on_submit.call((u, p));
    };

    rsx! {
        div {
            class: "auth-card",
            h2 {
                class: "auth-title",
                "{props.title}"
            }

            form {
                class: "auth-form",
                onsubmit: on_submit_inner,

                // --- Campo Username ---
                div {
                    class: "auth-field",
                    label {
                        class: "auth-label",
                        r#for: "login-username",
                        "👤 Usuário"
                    }
                    input {
                        id: "login-username",
                        r#type: "text",
                        class: "auth-input",
                        placeholder: "seu_usuario",
                        value: "{username}",
                        autocomplete: "username",
                        oninput: move |e| username.set(e.value()),
                    }
                }

                // --- Campo Password ---
                div {
                    class: "auth-field",
                    label {
                        class: "auth-label",
                        r#for: "login-password",
                        "🔒 Senha"
                    }
                    div {
                        class: "auth-password-wrapper",
                        input {
                            id: "login-password",
                            r#type: if show_password() { "text" } else { "password" },
                            class: "auth-input",
                            placeholder: "••••••••",
                            value: "{password}",
                            autocomplete: "current-password",
                            oninput: move |e| password.set(e.value()),
                        }
                        button {
                            r#type: "button",
                            class: "auth-password-toggle",
                            onclick: move |_| show_password.set(!show_password()),
                            aria_label: if show_password() { "Ocultar senha" } else { "Mostrar senha" },
                            if show_password() {
                                "🙈"
                            } else {
                                "👁️"
                            }
                        }
                    }
                }

                // --- Mensagem de erro ---
                if let Some(msg) = error_msg.read().clone() {
                    div {
                        class: "auth-error",
                        span { class: "auth-error-icon", "⚠️" }
                        span { "{msg}" }
                    }
                }

                // --- Botão submit ---
                button {
                    r#type: "submit",
                    class: "auth-submit-btn",
                    "{props.submit_label}"
                }
            }

            // --- Footer link opcional ---
            if let Some(footer) = props.footer_link {
                div {
                    class: "auth-footer",
                    {footer}
                }
            }
        }
    }
}