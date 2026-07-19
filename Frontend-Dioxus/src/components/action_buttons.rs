//! Componente de Botões de Ação.
//!
//! Renderiza os botões disponíveis para o jogador agir:
//! Fold, Check, Call, Raise, All-in. A disponibilidade de cada
//! botão depende do estado da rodada.

use dioxus::prelude::*;

/// Tipo de ação disponível.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Fold,
    Check,
    Call,
    Raise,
    AllIn,
}

impl ActionKind {
    /// Rótulo do botão.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fold => "Fold",
            Self::Check => "Check",
            Self::Call => "Call",
            Self::Raise => "Raise",
            Self::AllIn => "All-in",
        }
    }

    /// Cor CSS do botão.
    #[must_use]
    pub const fn color_class(self) -> &'static str {
        match self {
            Self::Fold => "bg-red-600 hover:bg-red-700",
            Self::Check => "bg-blue-600 hover:bg-blue-700",
            Self::Call => "bg-green-600 hover:bg-green-700",
            Self::Raise => "bg-yellow-600 hover:bg-yellow-700",
            Self::AllIn => "bg-purple-600 hover:bg-purple-700",
        }
    }
}

/// Componente de botões de ação.
///
/// # Props
///
/// - `available`: lista de ações disponíveis nesta rodada
/// - `on_action`: callback chamado quando uma ação é selecionada
#[component]
pub fn ActionButtons(
    available: Vec<ActionKind>,
    on_action: EventHandler<ActionKind>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-wrap gap-3 justify-center p-4 bg-green-950/80 \
                    border-t border-green-700/50",
            for action in available.into_iter() {
                button {
                    key: "{action.label()}",
                    class: "px-6 py-3 rounded-lg font-bold text-white shadow-lg \
                            transition-colors {action.color_class()}",
                    onclick: move |_| on_action.call(action),
                    "{action.label()}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_labels() {
        assert_eq!(ActionKind::Fold.label(), "Fold");
        assert_eq!(ActionKind::Check.label(), "Check");
        assert_eq!(ActionKind::Call.label(), "Call");
        assert_eq!(ActionKind::Raise.label(), "Raise");
        assert_eq!(ActionKind::AllIn.label(), "All-in");
    }

    #[test]
    fn test_action_colors() {
        assert!(ActionKind::Fold.color_class().contains("red"));
        assert!(ActionKind::Check.color_class().contains("blue"));
        assert!(ActionKind::Call.color_class().contains("green"));
        assert!(ActionKind::Raise.color_class().contains("yellow"));
        assert!(ActionKind::AllIn.color_class().contains("purple"));
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(ActionKind::Fold, ActionKind::Fold);
        assert_ne!(ActionKind::Fold, ActionKind::Call);
    }
}
