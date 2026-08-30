// TournamentStore wraps poker_engine::tournament_engine::TournamentState
// so we can store tournaments in a HashMap keyed by ID.

use poker_engine::tournament_engine::{TournamentConfig, TournamentState};

/// Wrapper that pairs a TournamentState with its string ID for easy lookup.
/// The `id` field is kept for convenience even though TournamentState
/// already carries `tournament_id`, because the HashMap key and the
/// state's tournament_id may diverge when loading from DB.
pub struct TournamentStore {
    pub id: String,
    pub state: TournamentState,
    /// `play` | `real` — must match client wallet mode.
    pub money_mode: String,
    /// `holdem` | `short_deck` | `short_deck_omaha`
    pub poker_variant: String,
    /// Number of seats at each physical tournament table.
    pub table_max_players: u8,
}

impl TournamentStore {
    pub fn new(id: String, config: TournamentConfig) -> Self {
        Self::with_money_mode(id, config, "play".into())
    }

    pub fn with_money_mode(id: String, config: TournamentConfig, money_mode: String) -> Self {
        Self::with_mode_and_variant(id, config, money_mode, "holdem".into())
    }

    pub fn with_mode_and_variant(
        id: String,
        config: TournamentConfig,
        money_mode: String,
        poker_variant: String,
    ) -> Self {
        let mut state = poker_engine::tournament_engine::create_tournament(config);
        state.tournament_id = id.clone();
        let v = poker_variant.to_ascii_lowercase();
        let poker_variant: String = if v == "short_deck_omaha" || v == "sd_omaha" {
            "short_deck_omaha".into()
        } else if v == "short_deck" || v == "sd" {
            "short_deck".into()
        } else {
            "holdem".into()
        };
        let table_max_players = match poker_variant.as_str() {
            "short_deck_omaha" => 4,
            "short_deck" => 6,
            _ => 9,
        };
        Self {
            id,
            state,
            money_mode,
            poker_variant,
            table_max_players,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_defaults_use_expected_table_sizes() {
        let config = TournamentConfig::default();
        let omaha = TournamentStore::with_mode_and_variant(
            "omaha".into(),
            config.clone(),
            "play".into(),
            "short_deck_omaha".into(),
        );
        let short_deck = TournamentStore::with_mode_and_variant(
            "short-deck".into(),
            config.clone(),
            "play".into(),
            "short_deck".into(),
        );
        let holdem = TournamentStore::with_mode_and_variant(
            "holdem".into(),
            config,
            "play".into(),
            "holdem".into(),
        );

        assert_eq!(omaha.table_max_players, 4);
        assert_eq!(short_deck.table_max_players, 6);
        assert_eq!(holdem.table_max_players, 9);
    }
}
