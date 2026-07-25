//! Lista de mesas do lobby.
//!
//! Compõe `TableCard`, `PlayerCount` e `JoinButton` para cada mesa.
//! Exibe mensagem de estado vazio quando não há mesas.

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::router::Route;
use super::join_button::{JoinButton, JoinButtonState};
use super::player_count::PlayerCount;
use super::table_card::{GameType, TableCard, TableData};

/// Lista de mesas exibida no lobby.
///
/// Estilo Full Tilt Poker: felt verde escuro sólido, borda grossa dourada,
/// cantos sutis, sem animações de hover.
#[allow(dead_code)]
#[component]
pub fn LobbyList(tables: Vec<TableData>) -> Element {
    rsx! {
        div {
            class: "lobby-list",
            if tables.is_empty() {
                EmptyState {}
            } else {
                for table in tables.iter() {
                    LobbyListItem {
                        key: "{table.id}",
                        table: table.clone(),
                    }
                }
            }
        }
    }
}

/// Item individual da lista: card + contador + botão.
#[allow(dead_code)]
#[component]
fn LobbyListItem(table: TableData) -> Element {
    let navigator = use_navigator();
    let state = if table.is_full() {
        JoinButtonState::Full
    } else {
        JoinButtonState::Available
    };

    let table_id = table.id.clone();
    let on_join = move |_| {
        let nav = navigator;
        let target_id = table_id.clone();
        spawn(async move {
            match crate::api_client::join_table(&target_id).await {
                Ok(_) => {
                    nav.push(Route::Table { id: target_id });
                }
                Err(err) => {
                    log::warn!("Erro ao registrar entrada na mesa {target_id}: {err}");
                }
            }
        });
    };

    rsx! {
        div {
            class: "lobby-list-item",
            div {
                class: "lobby-list-item-row",
                div {
                    class: "lobby-list-item-card",
                    TableCard { table: table.clone() }
                }
                div {
                    class: "lobby-list-item-actions",
                    PlayerCount {
                        players: table.players,
                        max_players: table.max_players,
                    }
                    JoinButton { state, onclick: on_join }
                }
            }
        }
    }
}

/// Mensagem exibida quando não há mesas disponíveis.
#[allow(dead_code)]
#[component]
fn EmptyState() -> Element {
    rsx! {
        div {
            class: "lobby-empty",
            div {
                class: "lobby-empty-icon",
                "♠"
            }
            h3 {
                class: "lobby-empty-title",
                "Nenhuma mesa encontrada"
            }
            p {
                class: "lobby-empty-text",
                "Tente ajustar os filtros ou volte mais tarde."
            }
        }
    }
}

/// Dados mock de mesas para desenvolvimento e testes.
#[must_use]
#[allow(dead_code)]
pub fn mock_tables() -> Vec<TableData> {
    vec![
        TableData {
            id: "mesa-01".to_string(),
            name: "Mesa do João".to_string(),
            game_type: GameType::TexasHoldem,
            small_blind: 1,
            big_blind: 2,
            players: 4,
            max_players: 8,
        },
        TableData {
            id: "mesa-02".to_string(),
            name: "Omaha da galera".to_string(),
            game_type: GameType::Omaha,
            small_blind: 5,
            big_blind: 10,
            players: 6,
            max_players: 8,
        },
        TableData {
            id: "mesa-03".to_string(),
            name: "Freeroll do domingo".to_string(),
            game_type: GameType::Freeroll,
            small_blind: 0,
            big_blind: 0,
            players: 23,
            max_players: 100,
        },
        TableData {
            id: "mesa-04".to_string(),
            name: "Mesa High Roller".to_string(),
            game_type: GameType::TexasHoldem,
            small_blind: 50,
            big_blind: 100,
            players: 8,
            max_players: 8,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_tables_retorna_lista_nao_vazia() {
        let tables = mock_tables();
        assert!(!tables.is_empty());
        assert!(tables.len() >= 3);
    }

    #[test]
    fn mock_tables_tem_ids_unicos() {
        let tables = mock_tables();
        let mut ids: Vec<&String> = tables.iter().map(|t| &t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), tables.len(), "IDs devem ser únicos");
    }

    #[test]
    fn mock_tables_inclui_mesa_cheia() {
        let tables = mock_tables();
        let has_full = tables.iter().any(|t| t.is_full());
        assert!(has_full, "Mock deve incluir pelo menos uma mesa cheia");
    }

    #[test]
    fn mock_tables_inclui_mesa_com_vagas() {
        let tables = mock_tables();
        let has_available = tables.iter().any(|t| !t.is_full());
        assert!(has_available, "Mock deve incluir pelo menos uma mesa com vagas");
    }

    #[test]
    fn mock_tables_tem_tipos_diversos() {
        let tables = mock_tables();
        let types: Vec<GameType> = tables.iter().map(|t| t.game_type).collect();
        let mut unique = types.clone();
        unique.sort_by_key(|g| *g as u32);
        unique.dedup();
        assert!(
            unique.len() >= 2,
            "Mock deve incluir pelo menos 2 tipos de jogo diferentes"
        );
    }
}
