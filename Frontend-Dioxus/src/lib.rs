//! Library root for Frontend Dioxus — re-exports modules for unit testing and fuzzing.

#![allow(dead_code)]

pub mod api_client;
pub mod audio;
pub mod components;
pub mod pages;
pub mod router;
pub mod ws_client;

#[cfg(test)]
mod fuzz_tests;
#[cfg(test)]
mod state_stress_tests;
