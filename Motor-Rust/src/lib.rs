// Library root for poker-engine.
// This file exposes all modules so that integration tests (under tests/)
// can import them via `use poker_engine::...`.
//
// The binary entry point remains in main.rs.

// ─── Módulos compartilhados (tipos e utilitários) ───
pub mod types;
pub mod utils;

// ─── Módulos do motor ───
pub mod antifraud;
pub mod auth;
pub mod deck;
pub mod game_loop;
pub mod hand_history;
pub mod lobby;
pub mod loss_deflator;
pub mod rake;
pub mod rng_crypto;
pub mod provably_fair;
pub mod side_pots;
pub mod tournament_engine;

// ─── Test modules (inline, avoid linker issues with GNU toolchain) ───
#[cfg(test)]
mod antifraud_tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod loss_deflator_tests;
#[cfg(test)]
mod motor_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod game_loop_tests;
#[cfg(test)]
mod tournament_engine_tests;
#[cfg(test)]
mod side_pots_tests;
#[cfg(test)]
mod lobby_tests;
#[cfg(test)]
mod hand_history_tests;
#[cfg(test)]
mod stress_tests;
#[cfg(test)]
mod stress_integration_tests;
#[cfg(test)]
mod card_fairness_tests;
#[cfg(test)]
mod fuzz_tests;
#[cfg(test)]
mod tournament_fuzz_tests;
#[cfg(test)]
mod extreme_fuzz_tests;
