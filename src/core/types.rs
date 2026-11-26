//! Core value types. Guideline §1.4 / §3 / A1.3.

use crate::core::constants::*;

/// The seven tetriminos. (§1.4, §3.1) One unique color each.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Tetrimino {
    /// Light blue, 4×1 line. (§3.1)
    I,
    /// Yellow 2×2 square.
    O,
    /// Purple, row-3 + top-center.
    T,
    /// Orange, row-3 + top-right.
    L,
    /// Dark blue, row-3 + top-left.
    J,
    /// Green, top domino right.
    S,
    /// Red, top domino left.
    Z,
}

/// All seven, in a stable order. (§3.3 bag contents)
pub const ALL_TETRIMINOS: [Tetrimino; 7] = [
    Tetrimino::I,
    Tetrimino::O,
    Tetrimino::T,
    Tetrimino::L,
    Tetrimino::J,
    Tetrimino::S,
    Tetrimino::Z,
];

/// RGB color. (§3.1) Presentation reads this; core defines standard colors.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Tetrimino {
    /// Standard color scheme. (§3.1)
    /// O=yellow, I=light blue, T=purple, L=orange, J=dark blue, S=green, Z=red.
    pub fn color(self) -> Color {
        match self {
            Tetrimino::O => Color::new(255, 220, 0), // yellow
            Tetrimino::I => Color::new(0, 240, 240), // light blue
            Tetrimino::T => Color::new(160, 0, 240), // purple
            Tetrimino::L => Color::new(255, 140, 0), // orange
            Tetrimino::J => Color::new(0, 0, 240),   // dark blue
            Tetrimino::S => Color::new(0, 240, 0),   // green
            Tetrimino::Z => Color::new(240, 0, 0),   // red
        }
    }
}

/// Facing. (A1.3) North at generation / in Next & Hold. CW +1, CCW -1 mod 4.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Facing {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Facing {
    pub const NORTH: Facing = Facing::North;

    pub fn index(self) -> usize {
        self as usize
    }

    /// Rotate clockwise 90°. (§5.3, A1.4)
    pub fn cw(self) -> Facing {
        match self {
            Facing::North => Facing::East,
            Facing::East => Facing::South,
            Facing::South => Facing::West,
            Facing::West => Facing::North,
        }
    }

    /// Rotate counter-clockwise 90°. (§5.3, A1.4)
    pub fn ccw(self) -> Facing {
        match self {
            Facing::North => Facing::West,
            Facing::West => Facing::South,
            Facing::South => Facing::East,
            Facing::East => Facing::North,
        }
    }
}

/// Rotation direction. (§5.3)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rotation {
    Cw,
    Ccw,
}

/// Horizontal move direction. (§5.1)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

/// Integer grid point. Engine y-down. May be transiently out of bounds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
    pub fn plus(self, o: Point) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }
}

/// A single matrix cell. `None` = empty; `Some(kind)` = locked Block. (§1.4)
pub type Cell = Option<Tetrimino>;

/// The tetrimino currently in play. (§5, A1.2.2) Origin = bounding-box top-left.
#[derive(Clone, Copy, Debug)]
pub struct ActivePiece {
    pub kind: Tetrimino,
    pub facing: Facing,
    pub origin: Point,
    /// Last successful action was a rotation (for T-Spin §9.1).
    pub last_was_rotation: bool,
    /// Which kick point (1..=5) was used in last rotation; `Some(5)` => Point-5. (§9.1.1, A1.4.1)
    pub last_kick_index: Option<usize>,
    /// Facing before last rotation (diagnostic).
    pub last_rotation_from: Facing,
}

impl ActivePiece {
    pub fn new(kind: Tetrimino, origin: Point) -> Self {
        Self {
            kind,
            facing: Facing::North,
            origin,
            last_was_rotation: false,
            last_kick_index: None,
            last_rotation_from: Facing::North,
        }
    }
}

/// Game-over reason. (§10)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameOverReason {
    /// Whole tetrimino locked down above the skyline. (§10.6)
    LockOut,
    /// Part of a newly generated tetrimino is blocked. (§10.7)
    BlockOut,
    /// (Multiplayer) line attacks push blocks past top-out line. (§10.5) seam.
    TopOut,
}

/// Which lock-down ruleset. (§2.5.4, §5.7–5.9) Extended default.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LockRuleset {
    /// Extended Placement. (§5.7) **default**; required for multiplayer/arcade.
    #[default]
    Extended,
    /// Infinite Placement. (§5.8)
    Infinite,
    /// Classic Lock Down. (§5.9)
    Classic,
}

/// Player command into the core. (§4/§5; presentation-agnostic)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameCommand {
    MoveLeft,
    MoveRight,
    RotateCw,
    RotateCcw,
    /// Soft drop state. `true` = pressed/held. (§5.5)
    SoftDrop(bool),
    HardDrop,
    Hold,
    // Pause/Resume handled at app layer; kept here for completeness.
    Pause,
    Resume,
}

/// Events emitted by the core for rendering/audio/stats. Not needed for logic.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EngineEvent {
    PieceGenerated {
        kind: Tetrimino,
    },
    Moved {
        dir: Direction,
    },
    MoveFailed,
    Rotated {
        from: Facing,
        to: Facing,
        /// 1..=5 kick point used. (A1.4)
        kick: usize,
    },
    RotationFailed,
    HardDropped {
        cells: u32,
        /// for hard-drop trail. (§13.1.2)
        kind: Tetrimino,
        facing: Facing,
        from: Point,
        to: Point,
    },
    SoftDropped {
        cells: u32,
    },
    /// Piece resting; lock-down timer started. (A1.2.3)
    LockStarted,
    /// Re-entered falling (moved off surface / falls again). (A1.2.3)
    LockCancelled,
    LockedDown {
        kind: Tetrimino,
        /// locked cells (engine space). for lock-down flash. (§13.1.3)
        cells: Vec<Point>,
    },
    /// Full rows found. (A1.2.4)
    PatternMatched {
        rows: Vec<i32>,
    },
    /// (A1.2.5) unused hook.
    Iterate,
    /// (A1.2.6)
    Animate,
    /// Lines removed + collapsed. (A1.2.7)
    LinesCleared {
        count: u32,
        rows: Vec<i32>,
    },
    /// Points awarded. (§8, A1.2.7)
    Scored {
        action: ScoreAction,
        points: u64,
        b2b: bool,
        total: u64,
        level: u32,
    },
    /// (§9.1)
    TSpin {
        kind: TSpinKindRepr,
    },
    HoldUsed {
        swapped: bool,
    },
    LevelUp {
        level: u32,
    },
    /// (A1.2.8)
    Completed,
    GameOver {
        reason: GameOverReason,
    },
    /// Variant ending (§12) distinct from game over.
    VariantEnd {
        reason: VariantEndReason,
    },
}

/// Score action. (§8) Used for scoring + notifications. (§13.1.5)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScoreAction {
    /// No line clear (no points base). Combo breaks.
    NoClear,
    Single,
    Double,
    Triple,
    Tetris,
    /// T-Spin no line. (§8) scored; no B2B start/break.
    TSpin,
    TSpinSingle,
    TSpinDouble,
    TSpinTriple,
    /// Mini T-Spin no line. (§8)
    MiniTSpin,
    /// Mini T-Spin Single. (§8)
    MiniTSpinSingle,
}

impl ScoreAction {
    /// Is this a Back-to-Back-qualifying action? (§8, §8n1)
    /// Tetris + T-Spin line clears + Mini-T-Spin line clears. No-line T-Spin: no.
    pub fn is_b2b_qualifying(self) -> bool {
        matches!(
            self,
            ScoreAction::Tetris
                | ScoreAction::TSpinSingle
                | ScoreAction::TSpinDouble
                | ScoreAction::TSpinTriple
                | ScoreAction::MiniTSpinSingle
        )
    }

    /// Does this break an existing Back-to-Back sequence? (§8.1)
    /// Only a non-qualifying Single/Double/Triple line clear breaks. No-line &
    /// no-clear & hold don't break; no-line T-Spin doesn't break (§8n1).
    pub fn breaks_b2b(self) -> bool {
        matches!(
            self,
            ScoreAction::Single | ScoreAction::Double | ScoreAction::Triple
        )
    }

    /// Did any line get cleared? (for combo stat)
    pub fn clears_line(self) -> bool {
        !matches!(
            self,
            ScoreAction::NoClear | ScoreAction::TSpin | ScoreAction::MiniTSpin
        )
    }
}

/// T-Spin kind. (§9.1) (mirrors `tspin::TSpinKind` for events)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TSpinKindRepr {
    Full,
    Mini,
}

/// Variant ending. (§12) distinct from game over (§10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VariantEndReason {
    /// Marathon level 15 complete. (§12.1)
    MarathonComplete,
    /// Sprint reached target lines. (§12.2)
    SprintComplete,
    /// Ultra time expired. (§12.3)
    UltraTimeUp,
}

/// Sanity: total rows. (compile-time-ish)
pub const _ASSERT_HEIGHT: i32 = MATRIX_HEIGHT_TOTAL;
