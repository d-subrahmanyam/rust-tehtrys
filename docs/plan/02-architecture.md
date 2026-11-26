# 02 — Architecture

## Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│ app            SDL2 event pump, fixed-timestep loop, app state      │
│                machine, screen routing (main.rs / app/)             │
├─────────────────────────────────────────────────────────────────────┤
│ input          key→action mapping, DAS/ARR state, pause/focus,      │
│                remapping persistence (input/)                       │
├─────────────────────────────────────────────────────────────────────┤
│ render         SDL2 drawing: matrix, pieces, ghost, HUD, Next/Hold, │
│                notifications, animation; reads snapshots (render/)  │
├─────────────────────────────────────────────────────────────────────┤
│ audio          SFX + BGM queue (sdl2_mixer); optional (audio/)      │
├─────────────────────────────────────────────────────────────────────┤
│ core           PURE tetris engine — no SDL, no I/O, no wall clock   │
│   types, matrix, piece+SRS, bag RNG, lockdown, engine(8 phases),    │
│   scoring, tspin, game(variants/levels), events (core/)             │
├─────────────────────────────────────────────────────────────────────┤
│ persist        settings, high scores, replays (JSON on disk)        │
│                (persist/)                                           │
└─────────────────────────────────────────────────────────────────────┘
```

**Hard rule:** `core` depends on nothing in `render`/`input`/`audio`/`app`, and no
external crates. Everything observable flows out as **events**; everything actionable
flows in as **commands**.

## Module layout (proposed)

```
src/
  main.rs                 // SDL2 init; build App; run loop
  app/
    mod.rs                // App struct; wiring
    state.rs              // AppState machine (see flows/14)
    loop_runner.rs        // fixed timestep accumulator; render interp; quit
  core/
    mod.rs
    types.rs              // Cell, Point, Facing, TetriminoKind, Color, ... 
    matrix.rs             // Matrix { w:10, h:40 }; occupancy; collision; clear; render snapshot
    piece.rs              // tetrimino data; orientation tables; spawn; cells_at(facing)
    rotation.rs           // SRS kick tables; try_rotate(...) -> Option<Placement>
    bag.rs                // Rng trait; SplitMix64; Bag7; NextQueue
    rng.rs                // internal PRNG (no external crate)
    lockdown.rs           // LockDown ruleset; timer; counters; reset rules (§5.7–5.9, A1.2.3)
    engine.rs             // TetrisEngine; 8 phases; process(inputs); emit events; GameCommand
    scoring.rs            // ScoreTable; B2B; combo-stat; drop points; level multiplier
    tspin.rs              // T-spin recognition (§9.1) from last-move info
    levels.rs             // Level; GoalSystem (Fixed/Variable); fall speed (§7); start level
    game.rs               // GameState; variants (Marathon/Sprint/Ultra); ending; stats
    replay.rs             // seed + input log; deterministic re-simulation (optional)
  input/
    mod.rs
    keymap.rs             // SDL key ↔ GameCommand; remappable; defaults (§4.1)
    das.rs                // DAS/ARR state machine (§5.2)
  render/
    mod.rs
    renderer.rs           // SDL2 canvas; rects; colors; scaling
    layout.rs             // computed rects: matrix, next, hold, HUD
    hud.rs
    notifications.rs
    anim.rs               // line clear, lock flash, hard-drop trail, fades
  audio/
    mod.rs                // trait Audio; NullAudio; SdlAudio [later]
  persist/
    mod.rs                // Settings; HighScoreTable; load/save
```

`core` is what unit/integration tests target; `render` gets `Snapshot` structs.

## Core types (sketch)

```rust
pub const MATRIX_WIDTH: u8 = 10;
pub const MATRIX_HEIGHT_VISIBLE: u8 = 20;
pub const BUFFER_HEIGHT: u8 = 20;
pub const MATRIX_HEIGHT_TOTAL: u8 = MATRIX_HEIGHT_VISIBLE + BUFFER_HEIGHT; // 40

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tetrimino { I, O, T, L, J, S, Z }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Facing { North = 0, East = 1, South = 2, West = 3 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Rotation { Cw, Ccw }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Direction { Left, Right }

/// A single occupied square. `kind: None` = empty.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Cell(pub Option<Tetrimino>);

/// Engine-space point: x: 0..=9, y: 0..=39, **y-down**. May temporarily be out of
/// bounds (buffer/spawn); collision checks clamp.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Point { pub x: i32, pub y: i32 }

/// The tetrimino currently in play.
#[derive(Clone, Copy, Debug)]
pub struct ActivePiece {
    pub kind: Tetrimino,
    pub facing: Facing,
    pub origin: Point,          // origin of local bounding box in engine space
    // transient last-move info for T-Spin recognition (§9.1):
    pub last_was_rotation: bool,
    pub last_rotation_kick_index: Option<usize>, // which kick point (1..=5); Point 5 check
    pub last_rotation_from: Facing,
}
```

### Commands in / events out

```rust
pub enum GameCommand {            // → core
    MoveLeft, MoveRight,          // one cell; also feeds DAS upstream in input layer
    RotateCw, RotateCcw,
    SoftDrop(bool),               // press/release; or SoftDropStep
    HardDrop,
    Hold,
    Pause, Resume,
    Restart, ...
}

pub enum EngineEvent {            // ← core (for render/audio/stats; not needed for logic)
    PieceGenerated { kind: Tetrimino },
    Moved { dir: Direction },
    Rotated { from: Facing, to: Facing, kick: usize },
    RotationFailed,
    MoveFailed,
    LockedDown { kind: Tetrimino, piece_cells: Vec<Point> },
    LinesCleared { count: u8, rows: Vec<u8>, .. },       // "hit list"
    Scored { action: ScoreAction, points: u64, b2b: bool, b2b_count: u32, combo: u32, level: u32 },
    TSpin { kind: TSpinKind },                            // Mini / Full
    LevelUp { level: u32 },
    HoldUsed { swapped: bool },
    GameOver { reason: GameOverReason },
    // line-clear animation hooks:
    PatternMatched { rows: Vec<u8> }, Animate, Eliminated, Completed,
    // soft/hard drop scoring handled in `Scored`
}
```

## Determinism

- `core` never reads real time: caller passes `dt_ms` to `tick(dt_ms)`.
- `Rng` injected; `SplitMix64` default; **seed recorded**; 7-bag reproducible.
- Integer **milliseconds**; no `f32`/`f64` in game logic; score integer `u64`.
- Fixed 60 Hz update in `app`; render interpolation optional and cosmetic only.
- Replay: `(seed, Vec<(t_ms, GameCommand)>)` re-simulates identically (NFR-1).

## Timing model (`core`)

Constants (appendix C): `LOCK_DOWN_MS = 500`, `HARD_DROP_MS = 0` (0.0001 s rounds to
instant; **documented**, no need to simulate 0.0001), `GENERATION_MS = 200`,
`SOFT_DROP_MULTIPLIER = 20`, `DAS_MS = 300`, `ARR_MS ≈ 33.3` (full width ~0.5 s).

- Gravity: accumulate `gravity_accum_ms`; each `fall_ms(level)` ⇒ move down 1; soft
  drop divides `fall_ms` by 20 (min clamp); hard drop instant.
- Lock Down Timer: `Option<LockTimer>`; starts when piece rests; **stops** when piece
  is off-surface (A1.2.3); resets per ruleset; `LOCK_DOWN_MS` ⇒ Lock Phase.
- Generation Timer: `GENERATION_MS` after Completion Phase before next Generation.
- DAS/ARR lives in **input** layer, emitting repeated `MoveLeft/Right`; core only
  applies single moves. (Continues across lock-down to next piece — engine keeps no
  per-piece DAS state.)

## Phasing

`TetrisEngine` runs an explicit state machine matching guideline **A1.1/A1.2**.
See [03-game-engine.md](03-game-engine.md) and [flows/02-engine-phases.md](flows/02-engine-phases.md).

```
Generation → Falling → Lock → Pattern → Iterate → Animate → Eliminate → Completion → (Generation)
```

- `Pattern/Iterate/Animate` take **no apparent game time** (A1.2.4–6).
- **Hold** can be triggered in Falling or Lock phase; resets piece to spawn; once per lock (§5.6).
- Multiplayer **Top Out** wiring left open (events + a `LineAttackQueue` seam; not built).

## Rendering snapshot

`render` consumes an immutable `RenderSnapshot`:

```rust
pub struct RenderSnapshot {
    pub cells: Vec<Cell>,                 // 40×10, y-down
    pub active: Option<ActivePieceSnap>,  // kind/facing/origin + ghost origin
    pub next: Vec<Tetrimino>,
    pub hold: Option<Tetrimino>,
    pub hold_used: bool,
    pub score: u64, pub level: u32, pub lines: u32, pub goal: u32,
    pub time_ms: u64, pub stats: Stats,
    pub phase: Phase, pub notifications: Vec<Notification>,
    pub clearing_rows: Vec<u8>,           // for animation
    pub paused: bool,
}
```

Rendering never mutates core; animation timing lives in `render::anim`, fed by events.

## Crate choice & build

- Keep **`sdl2`**. Rendering via `Canvas<Window>` drawing filled rects; integer cell
  size; letterboxed scaling. No external art needed for v1.
- **Remove `cgmath`** unless rendering needs vectors (grid math is integer).
- Core unit tests need **no SDL**; CI can run `cargo test -p tehtrys core::` headless.
- `sdl2` needs system SDL2 (present: 2.32.74); document `brew/apt/pacman` install +
  `SDL2` feature flags in build docs later. `sdl2_mixer` added in the sound phase.

## Error handling

- `core` returns `Result` only at construction; movement/rotation return
  `Option`/`bool` (failed move/rotation is **not** an error, just no-op + `*Failed`
  event, §13.2 "movement/rotation failure" SFX).
- `app` handles SDL errors; no panics in hot path; `unwrap` only on invariants in
  tests.

## Style

- `#![forbid(unsafe_code)]`.
- `cargo fmt`, `cargo clippy -D warnings`, `cargo test`.
- Small pure functions; tables as `const`.
