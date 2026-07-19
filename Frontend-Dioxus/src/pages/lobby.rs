//! Página de Lobby — listagem de mesas disponíveis.
//!
//! Mostra mesas de poker abertas para o usuário entrar.
//! Faz GET para `/api/lobby/tables` na API Axum.
//!
//! Refatorada em 3.7 para usar os novos componentes:
//! - `LobbyFilters` para filtros visuais
//! - `LobbyList` para renderizar a lista de mesas
//! - `TableCard` + `PlayerCount` + `JoinButton` por mesa

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::api_client;
use crate::components::lobby_filters::{BlindsFilter, GameTypeFilter, LobbyFilters};
use crate::components::lobby_list::LobbyList;
use crate::components::table_card::{GameType, TableData};
use crate::router::Route;

/// Componente da página de lobby.
#[component]
pub fn Lobby() -> Element {
    let mut tables = use_signal(Vec::<TableData>::new);
    let mut loading = use_signal(|| true);
    let mut fetch_error = use_signal(|| Option::<String>::None);

    // Filtros visuais (mock — não aplicados à lista nesta versão).
    let game_filter = use_signal(|| GameTypeFilter::All);
    let blinds_filter = use_signal(|| BlindsFilter::All);
    let only_available = use_signal(|| false);

    // Busca mesas da API ao montar o componente
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            match api_client::list_tables().await {
                Ok(api_tables) => {
                    let mapped: Vec<TableData> = api_tables
                        .into_iter()
                        .map(|t| TableData {
                            id: t.id,
                            name: t.name,
                            game_type: parse_game_type(&t.game_type),
                            small_blind: t.small_blind as u32,
                            big_blind: t.big_blind as u32,
                            players: t.players as u32,
                            max_players: t.max_players as u32,
                        })
                        .collect();
                    tables.set(mapped);
                    fetch_error.set(None);
                }
                Err(e) => {
                    fetch_error.set(Some(e));
                }
            }
            loading.set(false);
        });
    });

    rsx! {
        main {
            class: "container mx-auto px-6 py-8 max-w-5xl",
            div {
                class: "flex items-center justify-between mb-6",
                h2 {
                    class: "text-3xl font-bold text-yellow-400",
                    "🎪 Lobby de Mesas"
                }
                Link {
                    to: Route::Home {},
                    class: "text-sm text-green-300 hover:text-white underline",
                    "← Voltar"
                }
            }

            LobbyFilters {
                game_type: *game_filter.read(),
                blinds: *blinds_filter.read(),
                only_available: *only_available.read(),
            }

            if *loading.read() {
                div {
                    class: "text-center py-8 text-green-300/70",
                    "Carregando mesas..."
                }
            } else if let Some(err) = fetch_error.read().clone() {
                div {
                    class: "text-center py-8 text-red-400",
                    "Erro ao carregar mesas: {err}"
                }
            } else {
                LobbyList {
                    tables: tables.read().clone(),
                }
            }
        }
    }
}

/// Converte a string de tipo de jogo da API em enum GameType.
fn parse_game_type(s: &str) -> GameType {
    match s.to_lowercase().as_str() {
        "texas_holdem" | "texas-holdem" | "holdem" => GameType::TexasHoldem,
        "omaha" => GameType::Omaha,
        "omaha_hilo" | "omaha-hilo" => GameType::OmahaHiLo,
        "seven_card_stud" | "seven-card-stud" => GameType::SevenCardStud,
        "tournament" | "torneio" => GameType::Tournament,
        "freeroll" => GameType::Freeroll,
        _ => GameType::TexasHoldem,
    }
}
