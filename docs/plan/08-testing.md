# 08 — Testing Strategy

Core is pure ⇒ everything testable. Honesty: tests must **could have failed**; no
self-comparisons. Guideline-derived vectors are the gold standard.

## Layers

1. **Unit** (`core`): pure fns — collision, clear, rotate, kicks, scoring, B2B,
   t-spin, lock rules, goal math, fall speed, bag.
2. **Engine integration**: scripted command sequences + `ManualClock` ⇒ assert
   events/state after each phase; lock-down timing; hold; game-over.
3. **Guideline conformance**: vectors cited to §/A-section.
4. **Determinism/replay**: same seed + inputs ⇒ identical state & events.
5. **Render** (light): snapshot → layout sanity; no logic. *(not pixel-golden v1)*
6. **Input**: keymap; DAS/ARR FSM timing with `ManualClock`; edge vs repeat; opposite
   reset; focus loss.
7. **Property/fuzz** (optional): random ops never panic; invariants (matrix stays
   valid; no overlapping cells; piece always in some valid state); no `unsafe`.

## Testing tools

- `cargo test`; `cargo clippy -D warnings`; `cargo fmt --check`.
- `ManualClock`, `SeededRng`, `ScriptedInput`. No real time in core tests.
- Headless `core` tests need no SDL.

## Guideline-derived test vectors (must-have)

### Scoring (§8/§8.1)
- Single/Double/Triple/Tetris = 100/300/500/800·L.
- Mini T-Spin 100·L; Mini T-Spin Single 200·L; T-Spin 400·L; TSS 800·L; TSD 1200·L;
  TST 1600·L.
- B2B +50%; **§8.1 ⇒ 5400 @L1**; first not counted; break rules; no-line T-Spin
  rules (§8n1); lock-without-clear & hold don't break.
- Soft +1n; hard +2m; not level-scaled. Integer score. Level scaling.
- **No combo score** (§8); combo stat increments on consecutive clears, resets on
  non-clear.

### T-Spin (§9.1) — construct matrix fixtures
- 3-of-4 corners; N/E/S/W; A/B/C/D patterns; must-be-rotation; not move/drop;
  full vs mini; holes; **Rotation Point 5**; T-Spin no-line; Mini no-line; B2B
  interaction. Verify against guideline T-Spin diagram scenarios conceptually
  (exact diagram pixels not machine-readable — see README note; hand-built fixtures).

### SRS (A1.4.2)
- Exact kick tables (appendix A) per piece/facing/rotation; **order**; first-free
  wins; all-fail ⇒ no rotation; O no kick; I vs 3×3 differences; **Point-5** flag.
- **Verify against PDF Appendix A1.4.2 diagrams** (manual/cross-check test; cannot
  machine-read images here — honesty).
- Spawn positions §3.4/A1.2.1; North-facing; drop-one-row; I on row21; O cells;
  center mino (5,21).
- Facings N/E/S/W; CW/CCW; orientation tables (appendix B) consistent (rotation
  composition returns to start; kicks invert).

### Lock Down (§5.4–5.9, A1.2.3)
- Hard drop instant. 0.5 s resting timer. ManualClock at 499/500 ms.
- Extended: reset on move/rotate; **15 cap**; reset cap on new lowest row; after cap
  timer not reset; lock on first surface; note behavior §5.7. **default.**
- Infinite: reset on any real change. Classic: reset only on falling lower.
- A1.2.3 lift-off: no reset when lifted; pause; re-landing rule. Re-enter Falling if
  falls. Failed move doesn't reset. 

### Game over (§10)
- Lock Out whole-piece-above-skyline; Block Out starting cell occupied; peeking OK;
  above-skyline play OK; blocks above skyline in buffer OK; Top-Out seam. Buffer 20.

### Levels/goals (§6) & speed (§7)
- Fixed 10/level; L4→5 total 40 (§6); start-level adjust; Variable 5n; §6.1 table;
  B2B 0.5x clears; level-up timing; 15 levels; **fall speed formula + table 1–15**;
  soft20x; hard; clamps.

### Engine phases (A1.2)
- Order; no-time phases; hit list; eliminate+collapse; completion; **gen 0.2 s**;
  one piece; only-one-at-a-time; hold reset; events emitted.

### 7-bag (§3.3)
- 7 distinct before repeat; refill+reshuffle; order feeds next queue; seedable;
  reproducible; next-queue North-facing; length 1–6; reproducibility snapshot.

### Timing (appendix C)
- `LOCK=500ms`, `GEN=200ms`, `SOFT=20x`, `HARD=0.0001s`, `DAS=300ms`, `ARR≈33.3`,
  fixed-step; no float drift; integer ms.

### Variants (§12)
- Marathon 15; Sprint N=40; no level-up; Ultra time; endings; default Marathon.

## Property invariants

- Matrix width/height constant; no cell outside; no two cells overlap; active piece
  never overlaps locked; after clear, no full rows; level≥1; score monotonic
  non-decreasing; B2B consistent; hold ≤1 & once-per-lock; NextQueue reflects bag;
  phase transitions legal.

## What we explicitly cannot auto-test (honest)

- **Visual rendering** correctness (we snapshot layout, not pixels) — manual review.
- **Sound/music** (licensed assets absent).
- **PDF diagram conformance** for SRS/T-Spin kicks — we encode published tables &
  hand-build fixtures; a manual checklist against PDF pages 22–25 & A1.4.2 remains.
- **Feel** (DAS tuning) — constants from §5.2; manual.

## CI

`cargo fmt --check && cargo clippy --all-targets -D warnings && cargo test` on push;
core tests headless; no network.

## Definition of tested

A behavior counts only if a test could have shown it wrong. No `assert!(true)`, no
self-comparison, no re-encoding the same table twice. Fixtures hand-computed from the
guideline.
