//! Scoring. (§8, §8.1, §8n1; A1.2.7; appendix C; §6.1 Variable Goal extra clears)
//!
//! Integer `u64`. Level-scaled base + Back-to-Back +0.5x + drop points. 2009 has
//! **no combo scoring** (combo = stat only). (FR-S*)

use crate::core::constants::*;
use crate::core::tspin::TSpinKind;
use crate::core::types::ScoreAction;

/// Scoring + Back-to-Back + combo-stat + totals. (§8)
#[derive(Clone, Debug, Default)]
pub struct Scoring {
    pub score: u64,
    /// Back-to-Back active. (§8.1)
    pub b2b: bool,
    /// length of current B2B sequence. (stat)
    pub b2b_count: u32,
    /// consecutive clears combo. (stat only, §2.4.7; not scored in 2009)
    pub combo: u32,
    // stats (A1.2.7, §13):
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub tetrises: u32,
    pub t_spins: u32,
    pub mini_t_spins: u32,
    pub total_lines: u32,
    /// drop points accumulated for current piece; flushed at lock. (§8)
    pub pending_drop_points: u64,
}

/// Base points for an action at `level`. (§8) Not incl B2B.
pub fn base_points(action: ScoreAction, level: u32) -> u64 {
    let l = level.max(LEVEL_MIN) as u64;
    match action {
        ScoreAction::NoClear => 0,
        ScoreAction::Single => SCORE_SINGLE * l,
        ScoreAction::Double => SCORE_DOUBLE * l,
        ScoreAction::Triple => SCORE_TRIPLE * l,
        ScoreAction::Tetris => SCORE_TETRIS * l,
        ScoreAction::TSpin => SCORE_T_SPIN * l,
        ScoreAction::TSpinSingle => SCORE_T_SPIN_SINGLE * l,
        ScoreAction::TSpinDouble => SCORE_T_SPIN_DOUBLE * l,
        ScoreAction::TSpinTriple => SCORE_T_SPIN_TRIPLE * l,
        ScoreAction::MiniTSpin => SCORE_MINI_T_SPIN * l,
        ScoreAction::MiniTSpinSingle => SCORE_MINI_T_SPIN_SINGLE * l,
    }
}

/// Classify a lock into a `ScoreAction`. (§8, §9.1)
/// `lines` 0..=4; `tspin` from `tspin::detect`.
pub fn classify(lines: u32, tspin: TSpinKind) -> ScoreAction {
    match (lines, tspin) {
        (0, TSpinKind::None) => ScoreAction::NoClear,
        (1, TSpinKind::None) => ScoreAction::Single,
        (2, TSpinKind::None) => ScoreAction::Double,
        (3, TSpinKind::None) => ScoreAction::Triple,
        (4, TSpinKind::None) => ScoreAction::Tetris,
        // T-Spin (full)
        (0, TSpinKind::Full) => ScoreAction::TSpin,
        (1, TSpinKind::Full) => ScoreAction::TSpinSingle,
        (2, TSpinKind::Full) => ScoreAction::TSpinDouble,
        (3, TSpinKind::Full) => ScoreAction::TSpinTriple,
        (4, TSpinKind::Full) => ScoreAction::Tetris, // T can't clear 4; safety
        // Mini T-Spin
        (0, TSpinKind::Mini) => ScoreAction::MiniTSpin,
        (1, TSpinKind::Mini) => ScoreAction::MiniTSpinSingle,
        (_, TSpinKind::Mini) => ScoreAction::MiniTSpinSingle, // mini max 1; safety
        _ => ScoreAction::NoClear,                            // lines>4 impossible; safety
    }
}

impl Scoring {
    /// Register a lock + cleared `lines`. `tspin`, `level`. Returns points added. (§8, §8.1)
    pub fn register_action(
        &mut self,
        lines: u32,
        tspin: TSpinKind,
        level: u32,
    ) -> (ScoreAction, u64) {
        let action = classify(lines, tspin);
        let base = base_points(action, level);

        // Back-to-Back. (§8.1)
        let qualifying = action.is_b2b_qualifying();
        let mut points = base;
        if qualifying {
            if self.b2b {
                points += base * B2B_NUM / B2B_DEN; // +0.5x action total
            }
            self.b2b = true;
            self.b2b_count += 1;
        } else if action.breaks_b2b() {
            self.b2b = false;
            self.b2b_count = 0;
        } // no-line / no-clear / no-line T-Spin / hold: unchanged (§8n1, §8.1)

        // combo stat (not scored). resets on lock-without-clear. (§2.4.7)
        if lines > 0 {
            self.combo += 1;
        } else {
            self.combo = 0;
        }

        // stats
        match action {
            ScoreAction::Single => self.singles += 1,
            ScoreAction::Double => self.doubles += 1,
            ScoreAction::Triple => self.triples += 1,
            ScoreAction::Tetris => self.tetrises += 1,
            ScoreAction::TSpin
            | ScoreAction::TSpinSingle
            | ScoreAction::TSpinDouble
            | ScoreAction::TSpinTriple => self.t_spins += 1,
            ScoreAction::MiniTSpin | ScoreAction::MiniTSpinSingle => self.mini_t_spins += 1,
            _ => {}
        }
        self.total_lines += lines;

        // drop points accrued this piece
        points += self.pending_drop_points;
        self.pending_drop_points = 0;
        self.score += points;
        (action, points)
    }

    // drop scoring (§8) accumulated during piece
    pub fn add_soft_drop(&mut self, cells: u32) {
        self.pending_drop_points += SCORE_SOFT_DROP_PER_CELL * cells as u64;
    }
    pub fn add_hard_drop(&mut self, cells: u32) {
        self.pending_drop_points += SCORE_HARD_DROP_PER_CELL * cells as u64;
    }
    pub fn reset_pending_drop(&mut self) {
        self.pending_drop_points = 0;
    }

    /// Holding doesn't break B2B. (§8.1) no-op provided for clarity.
    pub fn register_hold(&mut self) {}

    /// Variable Goal extra line clears. (§6.1) returns clears to add (caller rounds).
    pub fn variable_goal_clears(
        action: ScoreAction,
        b2b_added: bool,
        level_at_1_value: u64,
    ) -> f64 {
        // §6.1 table; we return as f64 to carry 0.5; engine accumulates. (tested)
        let _ = level_at_1_value;
        let base = match action {
            ScoreAction::Single | ScoreAction::MiniTSpin => 1.0,
            ScoreAction::MiniTSpinSingle => 2.0,
            ScoreAction::Double => 3.0,
            ScoreAction::TSpin => 4.0,
            ScoreAction::Triple => 5.0,
            ScoreAction::Tetris | ScoreAction::TSpinSingle => 8.0,
            ScoreAction::TSpinDouble => 12.0,
            ScoreAction::TSpinTriple => 16.0,
            _ => 0.0,
        };
        let b2b = if b2b_added { base * 0.5 } else { 0.0 };
        base + b2b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoring_table_level_scaled() {
        assert_eq!(base_points(ScoreAction::Single, 1), 100);
        assert_eq!(base_points(ScoreAction::Single, 5), 500);
        assert_eq!(base_points(ScoreAction::Double, 1), 300);
        assert_eq!(base_points(ScoreAction::Triple, 1), 500);
        assert_eq!(base_points(ScoreAction::Tetris, 1), 800);
        assert_eq!(base_points(ScoreAction::MiniTSpin, 1), 100);
        assert_eq!(base_points(ScoreAction::MiniTSpinSingle, 1), 200);
        assert_eq!(base_points(ScoreAction::TSpin, 1), 400);
        assert_eq!(base_points(ScoreAction::TSpinSingle, 1), 800);
        assert_eq!(base_points(ScoreAction::TSpinDouble, 1), 1200);
        assert_eq!(base_points(ScoreAction::TSpinTriple, 1), 1600);
        // level multiplier; drops NOT level-scaled handled separately
        assert_eq!(base_points(ScoreAction::Tetris, 10), 8000);
    }

    #[test]
    fn classify_actions() {
        assert_eq!(classify(0, TSpinKind::None), ScoreAction::NoClear);
        assert_eq!(classify(1, TSpinKind::None), ScoreAction::Single);
        assert_eq!(classify(2, TSpinKind::None), ScoreAction::Double);
        assert_eq!(classify(3, TSpinKind::None), ScoreAction::Triple);
        assert_eq!(classify(4, TSpinKind::None), ScoreAction::Tetris);
        assert_eq!(classify(0, TSpinKind::Full), ScoreAction::TSpin);
        assert_eq!(classify(1, TSpinKind::Full), ScoreAction::TSpinSingle);
        assert_eq!(classify(2, TSpinKind::Full), ScoreAction::TSpinDouble);
        assert_eq!(classify(3, TSpinKind::Full), ScoreAction::TSpinTriple);
        assert_eq!(classify(0, TSpinKind::Mini), ScoreAction::MiniTSpin);
        assert_eq!(classify(1, TSpinKind::Mini), ScoreAction::MiniTSpinSingle);
    }

    /// §8.1 worked example: Tetris, T-Spin Double, T-Spin, Tetris, T-Spin Single @L1 => 5400.
    #[test]
    fn back_to_back_example_8_1_is_5400() {
        let mut s = Scoring::default();
        let mut total = 0;
        total += s.register_action(4, TSpinKind::None, 1).1; // Tetris 800
        total += s.register_action(2, TSpinKind::Full, 1).1; // TSD 1200+600
        total += s.register_action(0, TSpinKind::Full, 1).1; // T-Spin no line 400
        total += s.register_action(4, TSpinKind::None, 1).1; // Tetris 800+400
        total += s.register_action(1, TSpinKind::Full, 1).1; // TSS 800+400
        assert_eq!(total, 5400);
        assert_eq!(s.score, 5400);
    }

    #[test]
    fn b2b_breaking_and_noline_rules() {
        let mut s = Scoring::default();
        s.register_action(4, TSpinKind::None, 1); // start B2B
        assert!(s.b2b);
        // no-line T-Spin doesn't break, no bonus, stays inside. (§8n1)
        let (a, p) = s.register_action(0, TSpinKind::Full, 1);
        assert_eq!(a, ScoreAction::TSpin);
        assert_eq!(p, 400);
        assert!(s.b2b);
        // no-clear lock doesn't break. (§8.1)
        s.register_action(0, TSpinKind::None, 1);
        assert!(s.b2b);
        // plain Single breaks. (§8.1)
        s.register_action(1, TSpinKind::None, 1);
        assert!(!s.b2b);
    }

    #[test]
    fn drop_points_added_not_scaled() {
        let mut s = Scoring::default();
        s.add_soft_drop(3); // +3
        s.add_hard_drop(5); // +10
        let before = s.score;
        let (_, p) = s.register_action(0, TSpinKind::None, 7); // no clear
        assert_eq!(p, 13);
        assert_eq!(s.score, before + 13);
    }

    #[test]
    fn combo_stat_resets_on_no_clear() {
        let mut s = Scoring::default();
        s.register_action(1, TSpinKind::None, 1);
        s.register_action(1, TSpinKind::None, 1);
        assert_eq!(s.combo, 2);
        s.register_action(0, TSpinKind::None, 1);
        assert_eq!(s.combo, 0);
    }

    #[test]
    fn variable_goal_clears_6_1() {
        assert_eq!(
            Scoring::variable_goal_clears(ScoreAction::Single, false, 100),
            1.0
        );
        assert_eq!(
            Scoring::variable_goal_clears(ScoreAction::Double, false, 300),
            3.0
        );
        assert_eq!(
            Scoring::variable_goal_clears(ScoreAction::TSpinDouble, false, 1200),
            12.0
        );
        assert_eq!(
            Scoring::variable_goal_clears(ScoreAction::Tetris, true, 800),
            8.0 + 4.0
        ); // +0.5x
    }
}
