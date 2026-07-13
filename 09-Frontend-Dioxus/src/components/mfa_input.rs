//! Componente de input de código MFA/TOTP.
//!
//! Campo de 6 dígitos para verificação de autenticação de dois fatores
//! (TOTP — Time-based One-Time Password, RFC 6238).
//!
//! Usado após login bem-sucedido quando o servidor retorna `mfa_required`.

use dioxus::prelude::*;

/// Props do input MFA.
#[derive(Props, Clone, PartialEq)]
pub struct MfaInputProps {
    /// Callback chamado quando o código de 6 dígitos é preenchido e confirmado.
    /// Recebe o código como String (ex: "123456").
    pub on_verify: EventHandler<String>,
    /// Callback opcional para cancelar/reiniciar o fluxo MFA.
    #[props(default = None)]
    pub on_cancel: Option<EventHandler<()>>,
    /// Texto opcional do título (default: "🔐 Verificação em 2 Fatores").
    #[props(default = "🔐 Verificação em 2 Fatores".to_string())]
    pub title: String,
    /// Texto opcional de instrução (default explica TOTP).
    #[props(default = "Digite o código de 6 dígitos do seu aplicativo autenticador (Google Authenticator, Authy, etc.).".to_string())]
    pub instruction: String,
}

/// Input de código MFA de 6 dígitos com validação.
///
/// # Comportamento
///
/// - Aceita apenas dígitos numéricos (0-9)
/// - Máximo de 6 caracteres
/// - Botão "Verificar" só habilita quando 6 dígitos preenchidos
/// - Botão "Cancelar" opcional para voltar ao login
///
/// # Exemplo
///
/// ```rust,ignore
/// use crate::components::mfa_input::MfaInput;
///
/// rsx! {
///     MfaInput {
///         on_verify: move |code: String| {
///             // enviar POST /auth/mfa/verify
///         },
///         on_cancel: move |_| {
///             // voltar para tela de login
///         },
///     }
/// }
/// ```
#[component]
pub fn MfaInput(props: MfaInputProps) -> Element {
    let mut code = use_signal(String::new);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    let is_valid = code.read().len() == 6
        && code.read().chars().all(|c| c.is_ascii_digit());

    let on_input = move |e: FormEvent| {
        let raw = e.value();
        // Filtra apenas dígitos e limita a 6 caracteres
        let filtered: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
        code.set(filtered);
        error_msg.set(None);
    };

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if !is_valid {
            error_msg.set(Some("Digite exatamente 6 dígitos.".to_string()));
            return;
        }
        is_loading.set(true);
        error_msg.set(None);
        props.on_verify.call(code.read().clone());
    };

    let on_cancel_click = move |_| {
        if let Some(ref cancel_handler) = props.on_cancel {
            cancel_handler.call(());
        }
    };

    rsx! {
        div {
            class: "auth-card",
            h2 {
                class: "auth-title",
                "{props.title}"
            }

            div {
                class: "mfa-container",

                // --- Ícone MFA ---
                div {
                    class: "mfa-icon",
                    "🔐"
                }

                // --- Instrução ---
                p {
                    class: "mfa-instruction",
                    "{props.instruction}"
                }

                // --- Campo de código ---
                form {
                    class: "auth-form",
                    onsubmit: on_submit,

                    div {
                        class: "auth-field",
                        label {
                            class: "auth-label",
                            r#for: "mfa-code",
                            "🔢 Código de 6 dígitos"
                        }
                        input {
                            id: "mfa-code",
                            r#type: "text",
                            class: "mfa-code-input",
                            inputmode: "numeric",
                            pattern: "[0-9]*",
                            maxlength: "6",
                            placeholder: "000000",
                            value: "{code}",
                            autocomplete: "one-time-code",
                            oninput: on_input,
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

                    // --- Botões ---
                    div {
                        class: "mfa-buttons",

                        button {
                            r#type: "submit",
                            class: "auth-submit-btn",
                            disabled: !is_valid || is_loading(),
                            if is_loading() {
                                "⏳ Verificando..."
                            } else {
                                "✅ Verificar"
                            }
                        }

                        if props.on_cancel.is_some() {
                            button {
                                r#type: "button",
                                class: "auth-cancel-btn",
                                onclick: on_cancel_click,
                                "↩️ Cancelar"
                            }
                        }
                    }
                }
            }
        }
    }
}