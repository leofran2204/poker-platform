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
}

impl TournamentStore {
    pub fn new(id: String, config: TournamentConfig) -> Self {
        let mut state = poker_engine::tournament_engine::create_tournament(config);
        // Override the generated tournament_id with our explicit ID
        state.tournament_id = id.clone();
        Self { id, state }
    }
}
