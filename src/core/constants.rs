//! Global constants. Guideline references in comments. All times in ms.

/// Matrix width in cells. (§1.4, §2.4.1, A1.2.1)
pub const MATRIX_WIDTH: i32 = 10;
/// Visible matrix height in cells. (§1.4, A1.2.1)
pub const MATRIX_HEIGHT_VISIBLE: i32 = 20;
/// Hidden buffer-zone height above the skyline. (§1.4, §10)
pub const BUFFER_HEIGHT: i32 = 20;
/// Total grid height (visible + buffer).
pub const MATRIX_HEIGHT_TOTAL: i32 = MATRIX_HEIGHT_VISIBLE + BUFFER_HEIGHT;
/// Engine y-coordinate of the skyline boundary; buffer rows are `0..SKYLINE`.
pub const SKYLINE_Y: i32 = BUFFER_HEIGHT;

/// Lock Down Timer. (§5.4, §5.7–5.9, A1.2.3)
pub const LOCK_DOWN_MS: u32 = 500;
/// Generation time after the Completion Phase. (A1.2.1)
pub const GENERATION_MS: u32 = 200;
/// Hard drop time. (§5.4, §7.1) Guideline says 0.0001 s; we treat as instant.
pub const HARD_DROP_MS: u32 = 0;
/// Soft drop speed multiplier. (§5.5, §7.1)
pub const SOFT_DROP_MULTIPLIER: u32 = 20;
/// Extended Placement move/rotation cap. (§5.7)
pub const EXTENDED_MOVE_CAP: u32 = 15;

/// Fall speed formula base & slope. (§7) `(0.8 - (level-1)*0.007)^(level-1)`
pub const FALL_SPEED_BASE: f64 = 0.8;
pub const FALL_SPEED_SLOPE: f64 = 0.007;
/// Minimum fall time per line (avoid 0). ms.
pub const MIN_FALL_MS: u32 = 1;
/// Fixed update tick used by the app loop. ms. (presentation; core takes dt)
pub const TICK_MS: u32 = 1000 / 60;

/// Levels. (§6, §8)
pub const LEVEL_MIN: u32 = 1;
pub const LEVEL_MAX: u32 = 15;
/// Fixed Goal lines per level. (§6)
pub const FIXED_GOAL_LINES_PER_LEVEL: u32 = 10;
/// Variable Goal increment per level. (§6)
pub const VARIABLE_GOAL_PER_LEVEL: u32 = 5;

/// Scoring. (§8) base points; multiplied by level.
pub const SCORE_SINGLE: u64 = 100;
pub const SCORE_DOUBLE: u64 = 300;
pub const SCORE_TRIPLE: u64 = 500;
pub const SCORE_TETRIS: u64 = 800;
pub const SCORE_MINI_T_SPIN: u64 = 100;
pub const SCORE_MINI_T_SPIN_SINGLE: u64 = 200;
pub const SCORE_T_SPIN: u64 = 400;
pub const SCORE_T_SPIN_SINGLE: u64 = 800;
pub const SCORE_T_SPIN_DOUBLE: u64 = 1200;
pub const SCORE_T_SPIN_TRIPLE: u64 = 1600;
/// Back-to-Back bonus = +50% of action total. (§8) `+0.5x`.
pub const B2B_NUM: u64 = 1;
pub const B2B_DEN: u64 = 2;
/// Soft drop +1 per cell; hard drop +2 per cell. (§8) not level-scaled.
pub const SCORE_SOFT_DROP_PER_CELL: u64 = 1;
pub const SCORE_HARD_DROP_PER_CELL: u64 = 2;
