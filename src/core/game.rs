//! Variant wrapper. (§12 Marathon/Sprint/Ultra; §11 endings; flows/01, flows/11)
//!
//! The pure [`TetrisEngine`] is variant-agnostic; [`Game`] applies variant rules:
//! leveling on/off, target lines, time limit, ending. Game Over (§10) always ends.

use crate::core::constants::LEVEL_MAX;
use crate::core::engine::{Phase, TetrisEngine};
use crate::core::levels::GoalSystem;
use crate::core::types::{GameCommand, LockRuleset, VariantEndReason};

/// Main Tetris variant. (§12) Marathon default. (§2.3.2)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Variant {
    /// 15 levels; score; level-up. (§12.1) **default**.
    #[default]
    Marathon,
    /// clear N lines (typically 40) fastest; no level-up. (§12.2)
    Sprint { target_lines: u32 },
    /// maximize score/lines in time; no level-up. (§12.3)
    Ultra { duration_ms: u32 },
}

impl Variant {
    pub fn sprint_40() -> Self {
        Variant::Sprint { target_lines: 40 }
    }
    pub fn ultra_2min() -> Self {
        Variant::Ultra {
            duration_ms: 120_000,
        }
    }
    pub fn levels_up(self) -> bool {
        matches!(self, Variant::Marathon)
    }
    pub fn name(self) -> &'static str {
        match self {
            Variant::Marathon => "Marathon",
            Variant::Sprint { .. } => "Sprint",
            Variant::Ultra { .. } => "Ultra",
        }
    }
}

pub struct Game {
    pub engine: TetrisEngine,
    pub variant: Variant,
    /// elapsed active (non-paused) time. ms.
    pub elapsed_ms: u64,
    pub ended: Option<VariantEndReason>,
    /// starting level. (§2.3.3, §12.2)
    pub start_level: u32,
}

impl Game {
    pub fn new(seed: u64, variant: Variant, start_level: u32, ruleset: LockRuleset) -> Self {
        let goal_system = GoalSystem::Fixed;
        let mut engine = TetrisEngine::with_config(
            seed,
            ruleset,
            crate::core::bag::DEFAULT_NEXT_LEN,
            start_level,
            goal_system,
        );
        engine.level_up_enabled = variant.levels_up();
        // Sprint/Ultra don't level up; Marathon max 15.
        Self {
            engine,
            variant,
            elapsed_ms: 0,
            ended: None,
            start_level: start_level.clamp(1, LEVEL_MAX),
        }
    }

    pub fn apply(&mut self, cmd: GameCommand) {
        if self.ended.is_some() || self.engine.is_game_over() {
            return;
        }
        self.engine.apply(cmd);
    }

    pub fn tick(&mut self, dt_ms: u32) {
        if self.ended.is_some() || self.engine.is_game_over() {
            return;
        }
        self.engine.tick(dt_ms);
        self.elapsed_ms += dt_ms as u64;
        self.check_variant_end();
    }

    fn check_variant_end(&mut self) {
        if self.engine.is_game_over() {
            return; // game over separate. (§10)
        }
        match self.variant {
            Variant::Marathon => {
                // "Level 15 is completed, or game over." (§12.1)
                if self.engine.level >= LEVEL_MAX
                    && self.engine.lines_this_level >= self.engine.goal
                    && self.engine.pieces_placed > 0
                {
                    self.ended = Some(VariantEndReason::MarathonComplete);
                }
            }
            Variant::Sprint { target_lines } => {
                if self.engine.lines_total >= target_lines {
                    self.ended = Some(VariantEndReason::SprintComplete);
                }
            }
            Variant::Ultra { duration_ms } => {
                if self.elapsed_ms >= duration_ms as u64 {
                    self.ended = Some(VariantEndReason::UltraTimeUp);
                }
            }
        }
    }

    pub fn is_over(&self) -> bool {
        self.ended.is_some() || self.engine.is_game_over()
    }
    pub fn phase(&self) -> Phase {
        self.engine.phase
    }
    pub fn score(&self) -> u64 {
        self.engine.score()
    }
    pub fn lines(&self) -> u32 {
        self.engine.lines_total
    }
    pub fn level(&self) -> u32 {
        self.engine.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_defaults_and_flags() {
        assert!(Variant::Marathon.levels_up());
        assert!(!Variant::sprint_40().levels_up());
        assert!(!Variant::ultra_2min().levels_up());
        assert_eq!(Variant::Marathon.name(), "Marathon");
    }

    #[test]
    fn sprint_no_level_up() {
        let g = Game::new(1, Variant::sprint_40(), 1, LockRuleset::Extended);
        assert!(!g.engine.level_up_enabled);
    }

    #[test]
    fn sprint_ends_at_target_lines() {
        let mut g = Game::new(
            1,
            Variant::Sprint { target_lines: 0 },
            1,
            LockRuleset::Extended,
        );
        // target 0 -> already met after first tick check
        g.tick(1);
        assert_eq!(g.ended, Some(VariantEndReason::SprintComplete));
    }

    #[test]
    fn ultra_ends_on_time() {
        let mut g = Game::new(
            1,
            Variant::Ultra { duration_ms: 100 },
            1,
            LockRuleset::Extended,
        );
        g.tick(50);
        assert!(g.ended.is_none());
        g.tick(60);
        assert_eq!(g.ended, Some(VariantEndReason::UltraTimeUp));
    }

    #[test]
    fn marathon_levels_up() {
        let g = Game::new(1, Variant::Marathon, 1, LockRuleset::Extended);
        assert!(g.engine.level_up_enabled);
    }
}
