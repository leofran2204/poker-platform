// Library root for poker-engine.
// This file exposes all modules so that integration tests (under tests/)
// can import them via `use poker_engine::...`.
//
// The binary entry point remains in main.rs.

// O código legado usa `format!` extensivamente em mensagens e fixtures. A
// interpolação posicional não altera comportamento nem segurança; mantemos a
// exceção limitada a este lint para que `-D warnings` continue bloqueando os
// demais problemas do Clippy.
#![allow(clippy::uninlined_format_args)]

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
pub mod provably_fair;
pub mod rake;
pub mod rng_crypto;
pub mod side_pots;
pub mod tournament_engine;

// ─── Test modules (inline, avoid linker issues with GNU toolchain) ───
#[cfg(test)]
mod antifraud_tests;
#[cfg(all(test, feature = "massive-tests"))]
mod card_fairness_tests;
#[cfg(all(test, feature = "massive-tests"))]
mod extreme_fuzz_tests;
#[cfg(all(test, feature = "massive-tests"))]
mod fuzz_tests;
#[cfg(test)]
mod game_loop_tests;
#[cfg(test)]
mod hand_history_tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod lobby_tests;
#[cfg(test)]
mod loss_deflator_tests;
#[cfg(test)]
mod motor_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod side_pots_tests;
#[cfg(all(test, feature = "massive-tests"))]
mod stress_integration_tests;
#[cfg(all(test, feature = "massive-tests"))]
mod stress_tests;
#[cfg(test)]
mod tournament_engine_tests;
#[cfg(all(test, feature = "massive-tests"))]
mod tournament_fuzz_tests;
