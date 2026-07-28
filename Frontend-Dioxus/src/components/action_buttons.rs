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

    /// Cor CSS com gradiente e animação de hover premium.
    #[must_use]
    pub const fn color_class(self) -> &'static str {
        match self {
            Self::Fold => {
                "bg-gradient-to-r from-red-600 to-red-800 hover:from-red-500 hover:to-red-700 border-red-500/50 shadow-red-900/40"
            }
            Self::Check => {
                "bg-gradient-to-r from-blue-600 to-blue-800 hover:from-blue-500 hover:to-blue-700 border-blue-500/50 shadow-blue-900/40"
            }
            Self::Call => {
                "bg-gradient-to-r from-emerald-600 to-emerald-800 hover:from-emerald-500 hover:to-emerald-700 border-emerald-500/50 shadow-emerald-900/40"
            }
            Self::Raise => {
                "bg-gradient-to-r from-amber-500 to-amber-700 hover:from-amber-400 hover:to-amber-600 border-amber-400/50 shadow-amber-900/40 text-gray-950"
            }
            Self::AllIn => {
                "bg-gradient-to-r from-purple-600 to-indigo-800 hover:from-purple-500 hover:to-indigo-700 border-purple-400/50 shadow-purple-900/40 animate-pulse"
            }
        }
    }
}

/// Componente de botões de ação com design premium e micro-animações.
#[component]
pub fn ActionButtons(available: Vec<ActionKind>, on_action: EventHandler<ActionKind>) -> Element {
    rsx! {
        div {
            class: "flex flex-wrap gap-4 justify-center items-center p-6 bg-slate-950/90 \
                    backdrop-blur-md border-t border-amber-500/30 shadow-2xl rounded-b-2xl max-w-4xl mx-auto",
            for action in available.into_iter() {
                button {
                    key: "{action.label()}",
                    class: "px-8 py-3.5 rounded-xl font-extrabold text-white text-base tracking-wider uppercase border shadow-lg transform hover:-translate-y-1 active:translate-y-0 transition-all duration-200 ease-out focus:outline-none focus:ring-2 focus:ring-amber-400/50",
                    onclick: move |_| {
                        match action {
                            ActionKind::Fold => crate::audio::SoundManager::play(crate::audio::SoundEvent::Fold),
                            ActionKind::Check => crate::audio::SoundManager::play(crate::audio::SoundEvent::Check),
                            ActionKind::Call | ActionKind::Raise | ActionKind::AllIn => crate::audio::SoundManager::play(crate::audio::SoundEvent::ChipBet),
                        }
                        on_action.call(action);
                    },
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
        assert!(ActionKind::Call.color_class().contains("emerald"));
        assert!(ActionKind::Raise.color_class().contains("amber"));
        assert!(ActionKind::AllIn.color_class().contains("purple"));
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(ActionKind::Fold, ActionKind::Fold);
        assert_ne!(ActionKind::Fold, ActionKind::Call);
    }
}
