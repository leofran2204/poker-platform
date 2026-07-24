//! Componente de Notificações Toast amigáveis de Confiança ao Jogador.
//!
//! Exibe alertas visuais animados para reconexão de rede, avisos de saldo,
//! bloqueios explicativos de antifraude e notificações de torneio.

use dioxus::prelude::*;

/// Severidade da notificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    /// Ícone associado ao nível de severidade.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Success => "✅",
            Self::Warning => "⚠️",
            Self::Error => "❌",
        }
    }

    /// Estilo CSS com gradiente e borda de alta visibilidade.
    #[must_use]
    pub const fn style_class(self) -> &'static str {
        match self {
            Self::Info => "bg-slate-900/90 border-blue-500/50 text-blue-200 shadow-blue-900/30",
            Self::Success => "bg-slate-900/90 border-emerald-500/50 text-emerald-200 shadow-emerald-900/30",
            Self::Warning => "bg-slate-900/90 border-amber-500/50 text-amber-200 shadow-amber-900/30",
            Self::Error => "bg-slate-900/90 border-red-500/50 text-red-200 shadow-red-900/30",
        }
    }
}

/// Dados de uma notificação ao jogador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerNotification {
    pub id: String,
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub timestamp: String,
}

/// Componente visual Toast de notificação para a UI.
#[component]
pub fn NotificationToast(
    notification: PlayerNotification,
    on_dismiss: EventHandler<String>,
) -> Element {
    let notif_id = notification.id.clone();
    rsx! {
        div {
            class: "flex items-start gap-3 p-4 rounded-xl border backdrop-blur-md shadow-2xl \
                    max-w-md w-full transform hover:scale-[1.02] transition-all duration-300 \
                    animate-slide-in select-none {notification.level.style_class()}",
            div {
                class: "text-2xl flex-shrink-0 mt-0.5",
                "{notification.level.icon()}"
            }
            div {
                class: "flex-1 min-w-0",
                h4 {
                    class: "font-bold text-sm text-white tracking-wide mb-0.5",
                    "{notification.title}"
                }
                p {
                    class: "text-xs leading-relaxed text-slate-300 break-words",
                    "{notification.message}"
                }
                span {
                    class: "text-[10px] text-slate-400 mt-1 block opacity-80",
                    "{notification.timestamp}"
                }
            }
            button {
                class: "text-slate-400 hover:text-white font-bold text-sm px-1.5 py-0.5 rounded \
                        hover:bg-slate-800 transition-colors self-start",
                onclick: move |_| on_dismiss.call(notif_id.clone()),
                "✕"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_levels() {
        assert_eq!(NotificationLevel::Info.icon(), "ℹ️");
        assert_eq!(NotificationLevel::Success.icon(), "✅");
        assert_eq!(NotificationLevel::Warning.icon(), "⚠️");
        assert_eq!(NotificationLevel::Error.icon(), "❌");
    }

    #[test]
    fn test_notification_styles() {
        assert!(NotificationLevel::Info.style_class().contains("blue"));
        assert!(NotificationLevel::Success.style_class().contains("emerald"));
        assert!(NotificationLevel::Warning.style_class().contains("amber"));
        assert!(NotificationLevel::Error.style_class().contains("red"));
    }
}
