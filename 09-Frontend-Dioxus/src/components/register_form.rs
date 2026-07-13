//! Componente de formulário de registro.
//!
//! Formulário reutilizável com campos de username, email, password
//! e confirmação de password, validação client-side e callback de submit.
//!
//! Usado em `pages/register.rs`.

use dioxus::prelude::*;

/// Props do formulário de registro.
#[derive(Props, Clone, PartialEq)]
pub struct RegisterFormProps {
    /// Callback chamado quando o formulário é submetido com sucesso na validação.
    /// Recebe (username, email, password).
    pub on_submit: EventHandler<(String, String, String)>,
    /// Texto opcional do título (default: "📝 Criar Conta").
    #[props(default = "📝 Criar Conta".to_string())]
    pub title: String,
    /// Texto opcional do botão submit (default: "Registrar").
    #[props(default = "Registrar".to_string())]
    pub submit_label: String,
    /// Link opcional para navegação alternativa (ex: "Já tem conta? Login").
    #[props(default = None)]
    pub footer_link: Option<Element>,
}

/// Formulário de registro com validação client-side.
///
/// # Validações
///
/// - Username: 3-30 caracteres, apenas letras/números/underscore
/// - Email: formato básico (contém @ e .)
/// - Password: mínimo 8 caracteres, pelo menos 1 número e 1 letra
/// - Confirm Password: deve ser igual ao password
///
/// # Exemplo
///
/// ```rust,ignore
/// use crate::components::register_form::RegisterForm;
///
/// rsx! {
///     RegisterForm {
///         on_submit: move |(user, email, pass): (String, String, String)| {
///             // enviar para API
///         },
///     }
/// }
/// ```
#[component]
pub fn RegisterForm(props: RegisterFormProps) -> Element {
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut show_password = use_signal(|| false);

    let on_submit_inner = move |evt: FormEvent| {
        evt.prevent_default();
        let u = username.read().trim().to_string();
        let e = email.read().trim().to_string();
        let p = password.read().clone();
        let cp = confirm_password.read().clone();

        // Validação: username
        if u.is_empty() {
            error_msg.set(Some("Usuário é obrigatório.".to_string()));
            return;
        }
        if u.len() < 3 || u.len() > 30 {
            error_msg.set(Some("Usuário deve ter entre 3 e 30 caracteres.".to_string()));
            return;
        }
        if !u.chars().all(|c| c.is_alphanumeric() || c == '_') {
            error_msg.set(Some(
                "Usuário só pode conter letras, números e underscore.".to_string(),
            ));
            return;
        }

        // Validação: email
        if e.is_empty() {
            error_msg.set(Some("Email é obrigatório.".to_string()));
            return;
        }
        if !e.contains('@') || !e.contains('.') {
            error_msg.set(Some("Formato de email inválido.".to_string()));
            return;
        }

        // Validação: password
        if p.is_empty() {
            error_msg.set(Some("Senha é obrigatória.".to_string()));
            return;
        }
        if p.len() < 8 {
            error_msg.set(Some("Senha deve ter pelo menos 8 caracteres.".to_string()));
            return;
        }
        if !p.chars().any(|c| c.is_ascii_digit()) {
            error_msg.set(Some("Senha deve conter pelo menos 1 número.".to_string()));
            return;
        }
        if !p.chars().any(|c| c.is_alphabetic()) {
            error_msg.set(Some("Senha deve conter pelo menos 1 letra.".to_string()));
            return;
        }

        // Validação: confirm password
        if cp != p {
            error_msg.set(Some("Senhas não conferem.".to_string()));
            return;
        }

        error_msg.set(None);
        props.on_submit.call((u, e, p));
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
                        r#for: "reg-username",
                        "👤 Usuário"
                    }
                    input {
                        id: "reg-username",
                        r#type: "text",
                        class: "auth-input",
                        placeholder: "seu_usuario",
                        value: "{username}",
                        autocomplete: "username",
                        oninput: move |e| username.set(e.value()),
                    }
                    span {
                        class: "auth-hint",
                        "3-30 caracteres (letras, números, _)"
                    }
                }

                // --- Campo Email ---
                div {
                    class: "auth-field",
                    label {
                        class: "auth-label",
                        r#for: "reg-email",
                        "📧 Email"
                    }
                    input {
                        id: "reg-email",
                        r#type: "email",
                        class: "auth-input",
                        placeholder: "voce@exemplo.com",
                        value: "{email}",
                        autocomplete: "email",
                        oninput: move |e| email.set(e.value()),
                    }
                }

                // --- Campo Password ---
                div {
                    class: "auth-field",
                    label {
                        class: "auth-label",
                        r#for: "reg-password",
                        "🔒 Senha"
                    }
                    div {
                        class: "auth-password-wrapper",
                        input {
                            id: "reg-password",
                            r#type: if show_password() { "text" } else { "password" },
                            class: "auth-input",
                            placeholder: "Mínimo 8 caracteres",
                            value: "{password}",
                            autocomplete: "new-password",
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
                    span {
                        class: "auth-hint",
                        "Mínimo 8 caracteres, 1 número + 1 letra"
                    }
                }

                // --- Campo Confirm Password ---
                div {
                    class: "auth-field",
                    label {
                        class: "auth-label",
                        r#for: "reg-confirm",
                        "🔒 Confirmar Senha"
                    }
                    input {
                        id: "reg-confirm",
                        r#type: "password",
                        class: "auth-input",
                        placeholder: "Repita a senha",
                        value: "{confirm_password}",
                        autocomplete: "new-password",
                        oninput: move |e| confirm_password.set(e.value()),
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