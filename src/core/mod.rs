//! Pure Tetris core. No rendering / OS / time-source / external crates.
//!
//! Conventions (see `docs/plan/appendices/A-srs-kick-tables.md`):
//! - engine **y-down**, 0-based; `x in 0..10`, `y in 0..40`;
//! - buffer rows `0..20`, visible rows `20..40`, skyline boundary `y = 20`;
//! - guideline y-up mapping `engine_y = 40 - guideline_row`, `engine_x = col - 1`;
//! - times are integer **milliseconds**; no floats in game logic.

pub mod constants;
pub mod types;

pub mod bag;
pub mod engine;
pub mod game;
pub mod levels;
pub mod lockdown;
pub mod matrix;
pub mod piece;
pub mod rng;
pub mod rotation;
pub mod scoring;
pub mod tspin;

pub use constants::*;
pub use types::*;
