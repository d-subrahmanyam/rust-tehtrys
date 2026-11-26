# 03 — Game Engine

Pure `core`. Implements guideline **A1.1/A1.2** eight phases. Cross-refs:
[flows/02-engine-phases.md](flows/02-engine-phases.md), [appendix A](appendices/A-srs-kick-tables.md).

## Matrix & cells

- `Matrix { width: 10, height: 40 }`, **y-down**, 0-based; cells `Option<Tetrimino>`.
- Visible rows `20..=39`; buffer rows `0..=19`; skyline boundary at y=20.
  (Guideline y-up row r ↔ engine y `40-r`; spawn rows 21/22 ↔ engine y 19/18;
  see conversion.)
- **Collision** `is_free(cells, origin)`: every target cell in `0<=x<10`, `y < 40`
  (cells with `y < 0` treated as outside-above, only allowed transiently in buffer),
  and target cell empty. Walls/floor/existing blocks block. (§5.1)
- Piece can extend into buffer & above skyline; **not** required to be fully inside.

## Spawn (§3.4, A1.2.1)

Guideline (y-up 1-based, 10-wide, rows 21/22, visible 1..20, buffer 21..40) → engine
(y-down 0-based) mapping in [appendix A](appendices/A-srs-kick-tables.md#spawn-mapping).
**Engine map: col→col−1; row→`40 − row`.** Skyline = engine y=20 boundary; buffer
rows 0..19; visible rows 20..39. Summary, engine space, **North-facing**, then
**drop-one-row-if-clear**:

| Piece | Guideline | Engine spawn origin (box top-left) + cells |
|-------|-----------|--------------------------------------------|
| T/L/J/S/Z | 3-wide on cols 4–6, row 21 (+row 22, §3.4); center (5,21) | 3×3 box origin (3,18); trio row 19 cols 3–5; extra row 18; **center mino engine (4,19)**; exact cells appendix B |
| I | row 21 cols 4–7; center (5,21)* | 4×4 box origin (3,19); cells row 19 cols 3–6 |
| O | row 21 cols 5–6 | 2×2; horizontally centered cols 4–5; vertical spawn + drop-1 rule per **appendix B O-note** (guideline row-21-only vs SRS 2×2 flagged for verification) |

\* I has no single center mino; Rotation Point 1 sits at the box center between cells
(A1.4.1). Exact orientation cells are **explicit tables** (appendix B), so spawn is
exact regardless.

Spawn rules (§3.4, A1.2.1):
- North facing.
- **On generation**, immediately drop one row if clear; if blocked, don't; show a few
  pixels; player can move/rotate immediately; ghost appears; piece in **Falling**.
- **Generation time 0.2 s** after Completion Phase of previous piece (A1.2.1).
- **Block Out** if a starting cell of the new piece is occupied (§10.7). **Lock Out**
  if a whole piece locks above skyline (§10.6). Piece may **not** spawn into an
  occupied cell. (Guideline peeking/lock-down-above-skyline are **not** game over.)

## Hold (§5.6, FR-T8)

- `Hold` in Falling/Lock: put current in hold; bring previously held (or next from
  bag) to spawn North-facing; **once per lock**; can't hold same piece twice; lock
  must occur between holds; holding **does not break B2B** (§8.1); generation spawn
  rules & Block-Out apply to swapped-in piece.
- Holding resets the incoming piece's lock state; no carryover of move-count/timer.

## Super Rotation System (A1.4) — see appendix A

- Facings N/E/S/W; CW `+1`, CCW `−1 mod 4`.
- `try_rotate(kind, facing, origin, rot, matrix)`:
  for `kick in kick_table[kind][from→to]` (5, ordered): candidate `origin + kick`;
  if `is_free` → success, **return kick index (1..=5)**. All fail → no rotation +
  `RotationFailed`. O never moves (kicks all (0,0)).
- `Point 1` first (visual). **Point 5** success recorded for T-Spin (§9.1.1).
- Rotation can move piece **down** off a surface → leaves Lock phase → Falling
  (A1.2.3); can also move up/right/left; re-check surface.

## Gravity & falling (§5.1, §5.5, §7)

- `fall_ms(level)` = `((0.8 - (level-1)*0.007)^(level-1)) * 1000`, clamped `>= 1`.
  (appendix C table.)
- accumulate `dt`; while `accum >= fall_ms_effective`: `accum -= ...; step_down()`;
  `step_down` moves +1 row if `is_free`; if blocked, enter Lock phase; if moved to a
  **new lowest row**, reset lock move-cap (§5.7) & handle lock-timer per ruleset.
- **Soft Drop** held: `fall_ms_effective = fall_ms / 20` (clamp `>= 1 ms`); each cell
  dropped this way **+1 score** (§8); soft drop to surface starts timer (§5.5); no
  auto-repeat issue (continuous while held); continues to next piece (§5.5).
- **Hard Drop**: `origin.y += drop_distance` to lowest `is_free`; **+2·m score**;
  immediate Lock (`0.0001 s` ~ instant) (§5.4, §7.1, §8); hard-drop trail cosmetic;
  no auto-repeat.

## Lock Down (§5.4, §5.7–5.9, A1.2.3) — see flows/06

Piece "rests" when `step_down` would collide. States:

```
enum LockState { Falling, Locking { timer_ms, moves_used, lowest_row } }
```

Constants: `LOCK_DOWN_MS = 500`. On rest → `Locking { timer=500, ... }` (not for
hard drop → lock immediately).

**Surface tracking (A1.2.3).** "Resting" = `step_down` collides. Timer **stops**
when piece is off-surface; **doesn't reset** when rotation lifts piece up; on
re-landing, resets per ruleset only when landing on a **lower** surface (guideline
A1.2.3 verbatim; y-up/y-down signs mapped + pinned by tests). Moving/rotating so
piece falls again → `Falling`.

Rulesets (selectable; **Extended default**; multiplayer/arcade require Extended, §5.7):

| Ruleset | Timer reset on | Move/rotate cap |
|---------|----------------|-----------------|
| **Extended** (§5.7) | any move/rotate while resting; re-land on surface; falling one row below **lowest row reached** resets cap; `timer→500ms` on reset | **15** total moves/rotations; once cap hit, timer not reset; lock on first surface touched (if cap exhausted); still can move/rotate while not resting |
| **Infinite** (§5.8) | any **real** position/orientation change; re-landing; `→500ms` | none |
| **Classic** (§5.9) | only when piece **falls further** (y increases in engine space / guideline y decreases); `→500ms` | none |

- Timer expiry in `Locking` ⇒ **Lock Phase**.
- Failed move/rotation does **not** reset anything (only real change counts; Extended
  counts successful moves/rotations, §5.7 "15 movements or rotations" — only actual
  changes). *(We count successful changes; guideline counts attempts? It says
  "15 left/right movements or rotations" — we count successful ones; tests will pin
  behavior.)*

## The eight phases (A1.2)

`Phase` enum; per tetrimino. Pattern/Iterate/Animate consume **no apparent game time**
(A1.2.4–6). Only one tetrimino manipulable (§5, A1.1).

1. **Generation** (A1.2.1). 7-bag next; spawn N/S; drop-one-row; block-out check;
   ghost; emit `PieceGenerated`. → Falling. (0.2 s delay before this phase, A1.2.1.)
2. **Falling** (A1.2.2). Accept input; gravity; soft/hard; hold. → Lock when resting
   (and timer expiry / hard drop).
3. **Lock** (A1.2.3). Piece can still be manipulated until locked; then commit cells
   into matrix; emit `LockedDown`; lock-out check. → Pattern. *(Falling & Lock both
   accept the same actions until actually locked.)*
4. **Pattern** (A1.2.4). Scan full rows; build **hit list**; classic pattern = full
   10-cell row; emit `PatternMatched`. No time.
5. **Iterate** (A1.2.5). Optional scan hook; **unused**. No time.
6. **Animate** (A1.2.6). Run animation; render/audio listen; engine itself no-ops;
   **line-clear animation** here (§13.1.1). → Eliminate. No correctness impact.
   *(We keep animation in render; core emits events.)*
7. **Eliminate** (A1.2.7). Remove hit-list cells; collapse above; **award scoring**
   (incl. T-Spin & B2B determined at lock); update **stats**; (multiplayer: line
   attacks would emit here); emit `LinesCleared`, `Scored`. No time.
8. **Completion** (A1.2.8). Update Score/Time/HUD; **Level-Up** check (goal); variant
   ending check; emit `Completed`, `LevelUp`; → Generation (with 0.2 s delay).

### T-Spin detection timing

Recognition needs **lock-time** matrix + last-move info; compute at Lock, apply at
Eliminate scoring. See [flows/08-t-spin-detection.md](flows/08-t-spin-detection.md)
and appendix A. Rules summary (§9.1): T only; last successful action was a rotation
into a T-Slot; 3-of-4 corners; A/B vs C/D; Point-5; "fills T-Slot without holes" vs
"creates holes" → T-Spin vs Mini.

## Scoring hookup (§8) — see [05](05-scoring-and-progression.md)

At Eliminate:
`base = table[action]·level; if B2B-qualifying && b2b_active { base += base/2 }`
(equivalently total ×1.5; guideline 0.5× action total; §8.1 verified).
Drop points added at drop time (not level-scaled).
**B2B** tracked across pieces; Single/Double/Triple reset; no-line T-Spin neither
starts nor breaks; lock-without-clear & hold don't break.
**COMBO** counted for stats only in 2009 (no combo scoring, FR-S7).

## Levels & goals (§6) — see [05](05-scoring-and-progression.md)

`Level 1..=15`. Goal system **Fixed** or **Variable** (FR-V2). Level Up in
Completion. Fall speed uses current level. Starting level adjusts initial goal &
speed (§6). Marathon ends after level 15. Sprint/Ultra don't level up.

## Game-over (§10) — see flows/10

Evaluate at **Lock** & **Generation**:
- **Lock Out**: whole piece locked above skyline (all cells y < 20).
- **Block Out**: new piece's starting cell(s) occupied.
- **Top Out**: (multiplayer seam) forced past top-out line. *(not v1)*
- Not over: peeking; above-skyline play; blocks above skyline inside buffer; piece in
  play above skyline.

## Stats (A1.2.7, §2.4.7, §8.1, §13)

Score, time, lines, level, goal, tetrises, t-spins, **combos**, TPM, LPM, singles/
doubles/triples/tetrises, t-spin counts, B2B count, pieces placed, PPM; per variant.

## Variant endings (§12)

- **Marathon**: level 15 reached / goal complete → end. (§12.1)
- **Sprint**: lines ≥ target (40) → end win; game over ends. (§12.2)
- **Ultra**: time expires (2 min) → end; maximize score/lines. (§12.3)

## Concurrency model

Single-threaded core; `app` feeds commands & `dt`; no locks. Render reads snapshot.
Optional replay thread later. Keep `EngineEvent` `Send` for future audio thread.

## Reference: exact constants (appendix C)

`W=10 V=20 B=20`, `LOCK=500ms`, `GEN=200ms`, `SOFT=20x`, `HARD=0.0001s→instant`,
`DAS=300ms`, `ARR≈33.3ms`, `FALL formula`, `EXTRA_POINTS soft1 hard2`, scoring table,
`LEVELS=1..15`, `EXTENDED_CAP=15`.
