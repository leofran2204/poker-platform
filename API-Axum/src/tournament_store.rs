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
}

impl TournamentStore {
    pub fn new(id: String, config: TournamentConfig) -> Self {
        Self::with_money_mode(id, config, "play".into())
    }

    pub fn with_money_mode(id: String, config: TournamentConfig, money_mode: String) -> Self {
        let mut state = poker_engine::tournament_engine::create_tournament(config);
        state.tournament_id = id.clone();
        Self {
            id,
            state,
            money_mode,
        }
    }
}
