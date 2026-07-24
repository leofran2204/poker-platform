//! Componente de Pote central.
//!
//! Mostra o valor acumulado do pote principal e dos side pots
//! no centro visual da mesa.

use dioxus::prelude::*;

/// Pote individual (principal ou side pot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotEntry {
    pub label: String,
    pub amount: u32,
}

impl PotEntry {
    /// Cria um novo pote.
    #[must_use]
    pub const fn new(label: String, amount: u32) -> Self {
        Self { label, amount }
    }
}

/// Componente visual do pote.
///
/// # Props
///
/// - `pots`: lista de potes (principal + side pots)
/// - `odd_cent_notice`: notificação opcional da regra do centavo ímpar no Showdown (TDA Regra 68)
#[component]
pub fn Pot(pots: Vec<PotEntry>, odd_cent_notice: Option<String>) -> Element {
    let total: u32 = pots.iter().map(|p| p.amount).sum();

    rsx! {
        div {
            class: "flex flex-col items-center gap-1",
            div {
                class: "w-32 h-32 rounded-full bg-gradient-to-br from-yellow-600 to-yellow-800 \
                        border-4 border-yellow-300 shadow-2xl flex flex-col items-center \
                        justify-center",
                div {
                    class: "text-xs uppercase tracking-wider text-yellow-100 font-semibold",
                    "POTE"
                }
                div {
                    class: "text-2xl font-bold text-white font-mono",
                    "💰 {total}"
                }
            }
            if pots.len() > 1 {
                div {
                    class: "flex flex-col items-center gap-0.5 text-xs text-yellow-100 \
                            bg-green-950/80 border border-yellow-300/30 rounded px-2 py-1",
                    for p in pots.iter() {
                        div {
                            class: "font-mono",
                            "{p.label}: {p.amount}"
                        }
                    }
                }
            }
            if let Some(notice) = odd_cent_notice {
                div {
                    class: "text-xs text-amber-300 bg-black/75 border border-amber-500/50 rounded px-2 py-0.5 shadow font-semibold animate-pulse",
                    "🪙 {notice}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pot_entry_new() {
        let entry = PotEntry::new("Main".to_string(), 1000);
        assert_eq!(entry.label, "Main");
        assert_eq!(entry.amount, 1000);
    }

    #[test]
    fn test_pot_total_sum() {
        let pots = [
            PotEntry::new("Main".to_string(), 500),
            PotEntry::new("Side 1".to_string(), 300),
            PotEntry::new("Side 2".to_string(), 200),
        ];
        let total: u32 = pots.iter().map(|p| p.amount).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_empty_pot() {
        let pots: Vec<PotEntry> = vec![];
        let total: u32 = pots.iter().map(|p| p.amount).sum();
        assert_eq!(total, 0);
    }
}
