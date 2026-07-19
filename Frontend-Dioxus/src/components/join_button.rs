//! Botão de entrada em mesa.
//!
//! Exibe o estado da mesa com três variantes visuais:
//! - Disponível: botão verde sólido.
//! - Cheia: botão cinza desabilitado.
//! - Assistindo: botão dourado.

use dioxus::prelude::*;

/// Estado do botão de entrada conforme a situação da mesa.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinButtonState {
    Available,
    Full,
    Spectating,
}

impl JoinButtonState {
    /// Rótulo textual do botão.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "Entrar",
            Self::Full => "Mesa cheia",
            Self::Spectating => "Assistir",
        }
    }

    /// Classe CSS correspondente ao estado.
    #[must_use]
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Available => "join-btn join-btn-available",
            Self::Full => "join-btn join-btn-full",
            Self::Spectating => "join-btn join-btn-spectating",
        }
    }

    /// Indica se o botão deve permanecer desabilitado.
    #[must_use]
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::Full)
    }
}

/// Botão de entrada exibido no card de cada mesa.
///
/// Estilo Full Tilt Poker: botões sólidos, sem animações de escala,
/// cantos sutis, cores clássicas de poker online.
#[allow(dead_code)]
#[component]
pub fn JoinButton(
    state: JoinButtonState,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let disabled = state.is_disabled();

    rsx! {
        button {
            class: "{state.css_class()}",
            disabled: "{disabled}",
            onclick: move |evt| {
                if let Some(ref cb) = onclick {
                    cb.call(evt);
                }
            },
            "{state.label()}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_button_state_label_retorna_strings_nao_vazias() {
        assert!(!JoinButtonState::Available.label().is_empty());
        assert!(!JoinButtonState::Full.label().is_empty());
        assert!(!JoinButtonState::Spectating.label().is_empty());
    }

    #[test]
    fn join_button_state_available_nao_desabilitado() {
        assert!(!JoinButtonState::Available.is_disabled());
    }

    #[test]
    fn join_button_state_full_desabilitado() {
        assert!(JoinButtonState::Full.is_disabled());
    }

    #[test]
    fn join_button_state_spectating_nao_desabilitado() {
        assert!(!JoinButtonState::Spectating.is_disabled());
    }

    #[test]
    fn join_button_state_css_class_diferentes_por_estado() {
        let available = JoinButtonState::Available.css_class();
        let full = JoinButtonState::Full.css_class();
        let spectating = JoinButtonState::Spectating.css_class();
        assert_ne!(available, full);
        assert_ne!(available, spectating);
        assert_ne!(full, spectating);
    }
}
