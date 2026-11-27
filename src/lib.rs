//! tehtrys — a Tetris game in Rust.
//!
//! The [`core`] module is a pure, deterministic, headless-testable Tetris engine
//! implementing the official *2009 Tetris Design Guideline* mechanics
//! (see `docs/plan/`; source guideline is local-only, not distributed).
//! It has **no** rendering, OS, wall-clock, or external-crate dependencies.

#![forbid(unsafe_code)]

pub mod app;
pub mod core;
pub mod font;
pub mod input;
pub mod persist;
pub mod render;
pub mod replay;
pub mod sound;
