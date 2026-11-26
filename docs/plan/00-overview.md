# 00 — Overview

## Vision

Build **tehtrys**: a faithful, polished single-player Tetris in Rust, implementing the
official **2009 Tetris Design Guideline** mechanics, with a clean, deterministic core
engine that is fully unit-tested and a simple SDL2 presentation layer.

The name `tehtrys` is the existing crate name; the plan keeps it (a licensed release
would require using the proper "Tetris" name under license — see trademark note in
[README](README.md)).

## Goals

1. **Rules-correct core.** Bit-for-bit faithful to the guideline where it specifies
   behavior: 10×20 matrix + 20-row buffer zone, 7 tetriminos, North-facing spawn,
   **Super Rotation System** with 5 kick tests, **7-bag** randomization, **0.5 s
   lock-down timer** with three rulesets (**Extended** default), hold, ghost,
   hard/soft drop, exact **scoring**, **T-Spin/Mini T-Spin** recognition,
   **Back-to-Back**, levels & goals, three **game-over** conditions, eight engine
   phases, 0.2 s generation delay, fall-speed curve.
2. **Deterministic & testable.** Pure `core` with no rendering/OS dependencies;
   injectable clock + seedable PRNG; replayable. Same inputs ⇒ same outputs.
3. **Faithful feel.** Fixed-timestep loop; **DAS/ARR** auto-repeat; snappy lock down;
   on-screen notifications; smooth rendering; pause hides matrix (§13.1.6).
4. **Three variants.** **Marathon** (default, 15 levels), **Sprint/40 Lines**,
   **Ultra** (time-limited); starting-level selection; configurable starting blocks.
5. **Extensible.** Clean seams for sound, music, high scores, remapping, replay,
   and (later) multiplayer per guideline Appendix B.

## Non-goals (v1)

- Multiplayer / line attacks (guideline Appendix B documented; engine events designed
  not to block it, but not built).
- Official trademark/licensing work: no official Tetris logo, no confidential marketing
  assets, no protected BGM files; BGM hook provided, Korobeiniki left to a licensed
  asset (§13.3).
- Netcode, online services, accounts.
- Custom variants beyond the three main ones (Iterate Phase hook noted, not used —
  matching guideline A1.2.5).
- Mobile/touch/console input schemes (keyboard first; input layer is abstracted).

See [09-variants-and-modes.md](09-variants-and-modes.md#scope-decisions).

## Key design decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust (edition 2024; toolchain 1.100) | Existing project |
| Rendering | **SDL2** (`sdl2` crate, already in `Cargo.toml`; system `libSDL2` 2.32 present) | Simple 2D rect rendering, keyboard, audio later |
| Architecture | `core` (pure logic) + `render` + `input` + `app`; core has **zero** SDL dependency | Determinism & testability |
| Core dependencies | none for logic; internal PRNG (SplitMix64 + Fisher–Yates) | Offline reproducibility; no `rand` needed for game logic |
| Presentation dependency | `sdl2`; drop `cgmath` unless rendering needs it | Keep it minimal |
| Timing | integer **milliseconds** accumulator; fixed 60 Hz update; timers in `Duration`/`u64 ms` | No float drift; reproducible |
| Time injection | `Clock` trait; `ManualClock` for tests | Testable lock-down/DAS/generation |
| RNG | `Rng` trait; `Xoshiro`/`SplitMix` impl; seedable; `SeededRng` for tests | 7-bag reproducibility |
| Orientation | explicit per-orientation cell tables (no runtime rotation math) + kick tables | Exactness; easy testing |
| Coordinates | engine **y-down** grid; convert guideline's y-up tables once | Matches rendering; documented mapping |

### Coordinate system (implementation convention)

The guideline uses **y-up, origin bottom-left, 1-based** cells *(§A1.2.1)*. Our engine
uses **y-down, 0-based** cells to match rendering. A single conversion module maps
guideline positions/kicks to engine space; tests assert equivalence. See
[appendix A](appendices/A-srs-kick-tables.md#coordinate-conventions).

- Grid width `W = 10`, visible height `V = 20`, buffer `B = 20`, total height `H = 40`.
- Skyline = boundary between row `B-1` (last buffer row) and row `B` (top visible row).
- **Guideline "y-coordinate decreases toward the floor"** maps to **engine y increases
  toward the floor**; semantic names ("lowest row reached", "timer reset on landing on
  lower surface") are preserved, not raw signs.

## Definition of done (v1)

- `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check` all clean.
- Marathon/Sprint/Ultra playable end-to-end on keyboard.
- All scoring/lock-down/SRS/game-over behaviors verified by tests derived from the
  guideline (incl. §8.1 Back-to-Back example ⇒ 5400 at level 1).
- All flows in [flows/](flows/README.md) match implemented behavior.
