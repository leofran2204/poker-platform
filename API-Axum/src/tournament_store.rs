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
        let poker_variant = if v == "short_deck_omaha" || v == "sd_omaha" {
            "short_deck_omaha".into()
        } else if v == "short_deck" || v == "sd" {
            "short_deck".into()
        } else {
            "holdem".into()
        };
        Self {
            id,
            state,
            money_mode,
            poker_variant,
        }
    }
}
