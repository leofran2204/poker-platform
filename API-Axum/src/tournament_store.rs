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
    /// Number of seats at each physical tournament table before the final table.
    pub table_max_players: u8,
    /// Optional variant activated when the final table threshold is reached.
    pub final_table_variant: Option<String>,
    /// Remaining-player threshold that activates `final_table_variant`.
    pub final_table_max_players: Option<u8>,
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
            final_table_variant: None,
            final_table_max_players: None,
        }
    }

    /// Returns the dealing/evaluation variant for the current tournament phase.
    pub fn active_poker_variant(&self) -> &str {
        let final_table_started = matches!(
            self.state.status,
            poker_engine::tournament_engine::TournamentStatus::Running
                | poker_engine::tournament_engine::TournamentStatus::Paused
        ) && self.state.players_remaining > 0
            && self
                .final_table_max_players
                .is_some_and(|limit| self.state.players_remaining <= u32::from(limit));

        if final_table_started {
            self.final_table_variant
                .as_deref()
                .unwrap_or(&self.poker_variant)
        } else {
            &self.poker_variant
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

    #[test]
    fn long_short_switches_only_when_final_table_starts() {
        let mut tournament = TournamentStore::with_mode_and_variant(
            "long-short".into(),
            TournamentConfig::default(),
            "play".into(),
            "holdem".into(),
        );
        tournament.final_table_variant = Some("short_deck".into());
        tournament.final_table_max_players = Some(6);

        tournament.state.status = poker_engine::tournament_engine::TournamentStatus::Running;
        tournament.state.players_remaining = 7;
        assert_eq!(tournament.active_poker_variant(), "holdem");

        tournament.state.players_remaining = 6;
        assert_eq!(tournament.active_poker_variant(), "short_deck");

        tournament.state.status = poker_engine::tournament_engine::TournamentStatus::Registering;
        assert_eq!(tournament.active_poker_variant(), "holdem");
    }
}
