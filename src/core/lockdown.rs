//! Lock Down. (§5.4, §5.7–5.9 + §8.1–8.4 in converted ref; A1.2.3; §2.5.4)
//!
//! Hard Drop locks immediately (§5.4). Natural fall / soft drop onto a Surface
//! starts a **0.5 s** Lock Down Timer. Three rulesets. Surface tracking A1.2.3.
//! Reset policy is applied by `engine` (which knows resting state); this struct
//! holds state + pure predicates. See flows/06.

use crate::core::constants::{EXTENDED_MOVE_CAP, LOCK_DOWN_MS};
use crate::core::types::LockRuleset;

#[derive(Clone, Debug)]
pub struct LockDown {
    pub ruleset: LockRuleset,
    pub timer_ms: u32,
    /// Extended successful moves/rotations used. (§5.7/8.1)
    pub moves_used: u32,
    /// lowest row reached (engine y-down; larger = lower). (§5.7/8.1)
    pub lowest_row: i32,
    /// currently resting on a surface (no fall)?
    pub resting: bool,
    /// currently off-surface (timer paused). (A1.2.3/8.4)
    pub off_surface: bool,
    /// Extended cap exhausted. (§5.7/8.1)
    pub cap_exhausted: bool,
}

impl LockDown {
    pub fn new(ruleset: LockRuleset, start_lowest_row: i32) -> Self {
        Self {
            ruleset,
            timer_ms: LOCK_DOWN_MS,
            moves_used: 0,
            lowest_row: start_lowest_row,
            resting: false,
            off_surface: true,
            cap_exhausted: false,
        }
    }

    pub fn reset_timer(&mut self) {
        self.timer_ms = LOCK_DOWN_MS;
    }

    pub fn reset_for_new_piece(&mut self, ruleset: LockRuleset, lowest_row: i32) {
        *self = LockDown::new(ruleset, lowest_row);
    }

    /// Would a successful move/rotation reset the timer? Depends on ruleset +
    /// whether piece stays resting vs lifts off / re-enters falling. (8.1–8.4)
    pub fn resets_on_action(&self) -> bool {
        match self.ruleset {
            // 8.4: rotation/move causing lift-off does NOT reset; handled by engine resting check.
            LockRuleset::Extended => !self.cap_exhausted, // 8.1 (+15 cap)
            LockRuleset::Infinite => true,                // 8.2 any real change
            LockRuleset::Classic => false,                // 8.3 only falling further resets
        }
    }

    /// Register a successful move/rotation. `still_resting` after the action.
    /// (engine applies 8.4 lift-off by calling on_leave_surface instead.)
    pub fn on_action_while_resting(&mut self, still_resting: bool) {
        if !still_resting {
            return; // lifted off / re-entered falling; no reset. (8.4, A1.2.3)
        }
        match self.ruleset {
            LockRuleset::Extended => {
                if !self.cap_exhausted {
                    self.moves_used += 1;
                    self.reset_timer();
                    if self.moves_used >= EXTENDED_MOVE_CAP {
                        self.cap_exhausted = true; // after 15, no reset; lock on first surface (8.1)
                    }
                }
            }
            LockRuleset::Infinite => self.reset_timer(),
            LockRuleset::Classic => {} // 8.3
        }
    }

    /// Called when piece becomes resting (lands on a surface). `y`: piece y.
    /// Implements 8.4 re-landing + 8.1/8.3 new-lowest.
    pub fn on_landing(&mut self, y: i32, resting: bool) {
        self.resting = resting;
        self.off_surface = !resting;
        if !resting {
            return;
        }
        // re-landing resets timer if NOT landing lower than before (8.4);
        // Classic resets only if y increases (falls further) (8.3/§5.9 engine-y-down).
        let new_lowest = y > self.lowest_row;
        match self.ruleset {
            LockRuleset::Classic => {
                if new_lowest {
                    self.lowest_row = y;
                    self.reset_timer();
                }
            }
            LockRuleset::Extended | LockRuleset::Infinite => {
                self.reset_timer();
                if new_lowest {
                    self.lowest_row = y;
                    if self.ruleset == LockRuleset::Extended {
                        self.moves_used = 0; // 8.1 reset 15-cap on new lowest
                        self.cap_exhausted = false;
                    }
                }
            }
        }
    }

    /// Piece lifted off surface / re-entered falling. Timer NOT reset; paused. (8.4, A1.2.3)
    pub fn on_leave_surface(&mut self) {
        self.resting = false;
        self.off_surface = true;
    }

    /// Gravity step down one row while falling. resting recomputed by engine.
    pub fn on_fall_step(&mut self, y: i32) {
        // 8.1 reset 15-cap when falls to a row lower than lowest reached.
        if self.ruleset == LockRuleset::Extended && y > self.lowest_row {
            self.lowest_row = y;
            self.moves_used = 0;
            self.cap_exhausted = false;
        }
    }

    /// Advance timer; true => lock now. Only when resting & on-surface. (8.1–8.3, A1.2.3)
    pub fn tick(&mut self, dt_ms: u32) -> bool {
        if !self.resting || self.off_surface {
            return false;
        }
        self.timer_ms = self.timer_ms.saturating_sub(dt_ms);
        self.timer_ms == 0
    }

    /// Extended cap exhausted: lock immediately on first surface touched. (8.1)
    pub fn cap_locks_immediately(&self) -> bool {
        self.ruleset == LockRuleset::Extended && self.cap_exhausted
    }
}

pub const LOCK_MS: u32 = LOCK_DOWN_MS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_only_when_resting_on_surface() {
        let mut ld = LockDown::new(LockRuleset::Extended, 0);
        assert!(!ld.tick(1000));
        ld.on_landing(30, true);
        assert!(!ld.tick(499));
        assert!(ld.tick(1));
    }

    #[test]
    fn off_surface_pauses_timer_8_4() {
        let mut ld = LockDown::new(LockRuleset::Infinite, 0);
        ld.on_landing(30, true);
        ld.tick(400);
        ld.on_leave_surface();
        assert!(!ld.tick(10_000));
        assert_eq!(ld.timer_ms, LOCK_DOWN_MS - 400);
    }

    #[test]
    fn infinite_resets_on_real_change_8_2() {
        let mut ld = LockDown::new(LockRuleset::Infinite, 0);
        ld.on_landing(30, true);
        ld.tick(400);
        ld.on_action_while_resting(true);
        assert_eq!(ld.timer_ms, LOCK_DOWN_MS);
    }

    #[test]
    fn no_reset_when_lifted_off_8_4() {
        let mut ld = LockDown::new(LockRuleset::Infinite, 0);
        ld.on_landing(30, true);
        ld.tick(400);
        ld.on_action_while_resting(false); // action lifted piece off
        assert!(ld.timer_ms < LOCK_DOWN_MS); // not reset
    }

    #[test]
    fn classic_only_falling_further_8_3() {
        let mut ld = LockDown::new(LockRuleset::Classic, 0);
        ld.on_landing(30, true);
        ld.tick(400);
        ld.on_action_while_resting(true); // move/rotate in place: no reset
        assert!(ld.timer_ms < LOCK_DOWN_MS);
        ld.on_landing(31, true); // falls further (engine y-down +1)
        assert_eq!(ld.timer_ms, LOCK_DOWN_MS);
    }

    #[test]
    fn extended_15_cap_and_new_lowest_8_1() {
        let mut ld = LockDown::new(LockRuleset::Extended, 0);
        ld.on_landing(30, true);
        for _ in 0..EXTENDED_MOVE_CAP {
            ld.on_action_while_resting(true);
        }
        assert!(ld.cap_exhausted);
        assert!(ld.cap_locks_immediately());
        let t = ld.timer_ms;
        ld.on_action_while_resting(true);
        assert_eq!(ld.timer_ms, t); // no reset after cap
        // new lowest resets cap
        ld.on_landing(31, true);
        assert!(!ld.cap_exhausted);
        assert_eq!(ld.moves_used, 0);
    }
}
