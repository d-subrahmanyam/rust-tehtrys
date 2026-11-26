//! Levels, goals, fall speed. (§6, §6.1, §7, §8; appendix C; flows/10)
//!
//! Levels 1–15; Fixed (10/level) or Variable (5n) goal; fall-speed table.
//! Scoring multiplier = level. (§6, §7, §8)

use crate::core::constants::*;

/// Goal system. (§6) engine supports both; default Fixed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GoalSystem {
    /// 10 lines per level. (§6) **default**.
    #[default]
    Fixed,
    /// `5*level`. (§6 + §6.1)
    Variable,
}

/// Guideline fall-speed table, seconds/line -> ms. (§7; appendix C) levels 1..=15.
pub const FALL_MS_TABLE: [u32; 15] = [
    1000, 793, 618, 473, 355, 262, 190, 135, 94, 64, 43, 28, 18, 11, 7,
];

/// Fall time per line in ms. (§7) `(0.8-(L-1)*0.007)^(L-1)`; table; clamp level; min 1.
pub fn fall_ms(level: u32) -> u32 {
    let l = level.clamp(LEVEL_MIN, LEVEL_MAX);
    FALL_MS_TABLE[(l - 1) as usize]
}

/// Fall speed from the formula (reference; used to verify table). (§7)
pub fn fall_ms_formula(level: u32) -> u32 {
    let l = level.max(LEVEL_MIN);
    let base = FALL_SPEED_BASE - (l as f64 - 1.0) * FALL_SPEED_SLOPE;
    let secs = base.powi((l - 1) as i32);
    let ms = (secs * 1000.0).round() as i64;
    ms.max(MIN_FALL_MS as i64) as u32
}

/// Soft-drop effective fall time. (§5.5, §7.1)
pub fn soft_drop_ms(fall_ms: u32) -> u32 {
    (fall_ms / SOFT_DROP_MULTIPLIER).max(MIN_FALL_MS)
}

/// Clamp level. (§6)
pub fn clamp_level(level: u32) -> u32 {
    level.clamp(LEVEL_MIN, LEVEL_MAX)
}

/// Lines required to clear `level` (per-level goal). (§6)
pub fn goal_for_level(system: GoalSystem, level: u32) -> u32 {
    let l = clamp_level(level);
    match system {
        GoalSystem::Fixed => FIXED_GOAL_LINES_PER_LEVEL,
        GoalSystem::Variable => VARIABLE_GOAL_PER_LEVEL * l, // 5n. (§6)
    }
}

/// Cumulative lines to *reach* `level` from level 1 (start-level adjusted). (§6)
/// Fixed: `10*(level-1)`. Variable: sum 5n.
pub fn cumulative_to_reach_level(system: GoalSystem, level: u32) -> u32 {
    let l = clamp_level(level);
    match system {
        GoalSystem::Fixed => FIXED_GOAL_LINES_PER_LEVEL * (l - 1),
        GoalSystem::Variable => (0..l).map(|n| VARIABLE_GOAL_PER_LEVEL * n).sum::<u32>(),
    }
}

/// Starting-level goal. (§6)
/// Fixed starting level `L`: goal to advance = `10*L` cumulative. (example L4->5 = 40)
/// Variable starting level `L>=2`: `L*5`.
pub fn starting_goal(system: GoalSystem, start_level: u32) -> u32 {
    let l = clamp_level(start_level);
    match system {
        GoalSystem::Fixed => {
            // total to advance from start level: cumulative prior + current = 10*L. (§6)
            FIXED_GOAL_LINES_PER_LEVEL * l
        }
        GoalSystem::Variable => VARIABLE_GOAL_PER_LEVEL * l,
    }
}

/// Awarded line clears for Variable Goal. (§6.1) `b2b` adds 0.5x. Returns f64.
pub fn variable_goal_clears(action: crate::core::types::ScoreAction, b2b_added: bool) -> f64 {
    crate::core::scoring::Scoring::variable_goal_clears(action, b2b_added, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fall_speed_table_matches_guideline_7() {
        let expect = [
            1000, 793, 618, 473, 355, 262, 190, 135, 94, 64, 43, 28, 18, 11, 7,
        ];
        assert_eq!(FALL_MS_TABLE, expect);
        for l in 1..=15 {
            assert_eq!(fall_ms(l), expect[(l - 1) as usize]);
        }
    }

    #[test]
    fn fall_speed_formula_matches_table() {
        // §7 formula reproduces table. (verify honestly, not self-comparison)
        for l in 1..=15 {
            assert_eq!(fall_ms_formula(l), fall_ms(l), "level {l}");
        }
    }

    #[test]
    fn fall_speed_clamps_level() {
        assert_eq!(fall_ms(0), fall_ms(1));
        assert_eq!(fall_ms(99), fall_ms(15));
    }

    #[test]
    fn soft_drop_20x() {
        assert_eq!(soft_drop_ms(1000), 50);
        assert_eq!(soft_drop_ms(7), 1); // clamp
        assert_eq!(soft_drop_ms(100), 5);
    }

    #[test]
    fn levels_clamp_1_15() {
        assert_eq!(clamp_level(0), 1);
        assert_eq!(clamp_level(15), 15);
        assert_eq!(clamp_level(100), 15);
    }

    #[test]
    fn fixed_goal_10_per_level() {
        assert_eq!(goal_for_level(GoalSystem::Fixed, 1), 10);
        assert_eq!(goal_for_level(GoalSystem::Fixed, 15), 10);
        // starting level L4 -> advance to 5 needs cumulative 40. (§6 example)
        assert_eq!(starting_goal(GoalSystem::Fixed, 4), 40);
        assert_eq!(starting_goal(GoalSystem::Fixed, 1), 10);
    }

    #[test]
    fn variable_goal_5n() {
        assert_eq!(goal_for_level(GoalSystem::Variable, 1), 5);
        assert_eq!(goal_for_level(GoalSystem::Variable, 2), 10);
        assert_eq!(goal_for_level(GoalSystem::Variable, 15), 75);
        // starting level >=2 goal = level*5. (§6)
        assert_eq!(starting_goal(GoalSystem::Variable, 2), 10);
        assert_eq!(starting_goal(GoalSystem::Variable, 4), 20);
        // total to level15 = 600. (§6)
        let total: u32 = (1..=15).map(|n| VARIABLE_GOAL_PER_LEVEL * n).sum();
        assert_eq!(total, 600);
    }
}
