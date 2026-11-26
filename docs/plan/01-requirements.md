# 01 — Requirements

Functional requirements (FR) extracted from the **2009 Tetris Design Guideline**, with
section references, plus non-functional requirements (NFR). IDs are stable for use in
tests and reviews. "Must" = guideline requirement; "Should" = recommended; "May" =
optional. Items marked **[v1]** are in the first release; **[later]** deferred.

## Core gameplay

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-G1 | Matrix is **10 cells wide × 20 cells high**. | §1.4, §2.4.1, A1.2.1 | must **[v1]** |
| FR-G2 | **Buffer Zone** 10×20 above Skyline; Top Out Line 20 rows above Skyline. | §1.4, §10 | must **[v1]** |
| FR-G3 | Only **one** tetrimino falls at a time. | §5 | must **[v1]** |
| FR-G4 | Clearing a full horizontal row removes it; blocks above shift down. | §1.4, A1.2.4–7 | must **[v1]** |
| FR-G5 | Scoring for singles/doubles/triples/tetris + T-Spins + B2B + drops. | §8 | must **[v1]** |
| FR-G6 | Level affects fall speed and scoring. | §6, §7, §8 | must **[v1]** |

## Tetriminos (§3)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-T1 | Seven tetriminos: **I, O, T, L, J, S, Z**; each 4 minos, one mino = one cell. | §1.4, §3 | must **[v1]** |
| FR-T2 | Standard colors: O=yellow, I=light blue, T=purple, L=orange, J=dark blue, S=green, Z=red. | §3.1 | must **[v1]** |
| FR-T3 | Monochrome fallback: distinct hues/patterns. | §3.2 | should **[later]** |
| FR-T4 | **7-bag**: shuffle 7, deal all 7 before refilling & reshuffling. | §3.3, A1.2.1 | must **[v1]** |
| FR-T5 | Next Queue fed by bag; 1–6 visible (ideally 6); next-to-play closest to top; North-facing orientation shown. | §2.4.3, §2.5.1 | must **[v1]** (default 6, configurable 1–6) |
| FR-T6 | North-facing spawn on rows 21–22 (21st/22nd); 3-wide pieces on cols 4–6; I on row 21 cols 4–7; O on row 21 cols 5–6; center mino at (5,21). | §3.4, A1.2.1 | must **[v1]** |
| FR-T7 | On generation: drop one row if clear; immediate manipulation; ghost appears; generation time **0.2 s** after Completion Phase. | §3.4, A1.2.1 | must **[v1]** |
| FR-T8 | Hold Queue: hold once per lock; held piece North-facing at spawn on re-entry; swap with previously held; no holding same piece twice in a row; lock between holds. | §2.4.8, §2.5.2, §5.6 | must **[v1]** (default on) |
| FR-T9 | Hold Queue can be turned off. | §2.5.2 | should **[later]** |
| FR-T10 | Ghost Piece: shows landing position; outline or translucent; North-facing at generation; default **on**; can be disabled. | §2.4.4, §2.5.3 | must **[v1]** |
| FR-T11 | Starting Blocks: optionally pre-fill N rows at bottom at game start. | §2.4.6 | should **[later]** |

## Controls (§4, §5)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-C1 | Move left/right one cell; no movement into occupied cells/walls/floor; snap to cells. | §5.1 | must **[v1]** |
| FR-C2 | Rotate 90° CW & CCW using **SRS**; no auto-repeat for rotation; rotatable while auto-repeating. | §5.3, A1.4 | must **[v1]** |
| FR-C3 | **DAS**: ~0.3 s initial delay; **ARR**: full width in ~0.5 s; left/right only; continues to next piece; opposite-direction reset; re-apply delay on release. | §5.2 | must **[v1]** |
| FR-C4 | **Hard Drop**: instantly drop & lock on first surface; no auto-repeat; +2/line; immediate lock (0.0001 s). | §5.4, §7.1, §8 | must **[v1]** |
| FR-C5 | **Soft Drop**: 20× fall speed while held; normal speed on release; no lock until timer; +1/line; continues to next piece. | §5.5, §7.1, §8 | must **[v1]** |
| FR-C6 | **Pause**: stop instantly; hide matrix/next/hold; show "Pause"; resume. | §13.1.6 | must **[v1]** |
| FR-C7 | Key bindings per §4.1 keyboard scheme; remappable. | §4, §4.1 | must **[v1]** (scheme) / should (remap) |
| FR-C8 | Default keyboard: ←/→ move, ↓ soft, Space hard, ↑/X CW, Z/Ctrl CCW, C/Shift hold, Esc/F1 pause (guideline-compatible; see [04](04-controls.md)). | §4.1 | must **[v1]** |

## Super Rotation System (A1.4)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-R1 | Four facings N/E/S/W; CW/CCW 90°. | A1.3 | must **[v1]** |
| FR-R2 | Five rotation points tried in order; first non-colliding wins; else rotation fails. | A1.4 | must **[v1]** |
| FR-R3 | Exact kick tables for O (none), I (4×4), T/L/J/S/Z (3×3); verify vs diagrams. | A1.4.2 | must **[v1]** |
| FR-R4 | Point 1 = visual center; Point-5 kick counts for T-Spin (Rotation Point 5). | A1.4.1, §9.1.1 | must **[v1]** |

## Lock Down (§5.4, §5.7–5.9, A1.2.3)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-L1 | Hard Drop locks immediately. | §5.4, A1.2.3 | must **[v1]** |
| FR-L2 | Natural fall / soft drop onto surface starts **0.5 s** Lock Down Timer. | §5.7–5.9, A1.2.3 | must **[v1]** |
| FR-L3 | **Extended Placement** (default; required for multiplayer/arcade): timer resets on move/rotate; **15** moves/rotations cap; reset cap on falling one row below lowest reached. | §5.7 | must **[v1]** |
| FR-L4 | **Infinite Placement**: timer resets on any real move/rotate. | §5.8 | must **[v1]** |
| FR-L5 | **Classic**: timer resets only when y-coordinate decreases (piece falls further). | §5.9 | must **[v1]** |
| FR-L6 | Rotating can lift piece off surface; timer doesn't reset; pauses until re-landing on same/lower... (guideline: same-or-higher y / not lower) surface; re-landing lower resets. | A1.2.3 | must **[v1]** |
| FR-L7 | Moving/rotating so piece falls again returns to Falling Phase. | A1.2.3 | must **[v1]** |
| FR-L8 | Lock-down visual feedback (flash/darken) & locked blocks differ from in-play. | §13.1.3 | should **[v1/later]** |

## Levels & goals (§6)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-V1 | 15 levels (1–15); default level 1; starting level selectable 1–15. | §6, §2.3.3 | must **[v1]** |
| FR-V2 | **Fixed Goal** (10 lines/level; start-level adjusted cumulative goal) *or* **Variable Goal** (5·level lines; start level goal = level·5) choose one. | §6 | must pick one **[v1]** → **Variable Goal** for Marathon? *(decision below)* |
| FR-V3 | Level Up on goal reached; checked in Completion Phase. | §6, A1.2.8 | must **[v1]** |

**Decision FR-V2:** implement **both**; default **Fixed Goal** (10 lines/level, simplest
matching §12.1 Marathon "15 levels"; cumulative start-level rule per §6) — *or* Variable.
Final pick deferred to implementation; engine supports both. Sprint/Ultra don't level up.
See [05](05-scoring-and-progression.md).

## Fall & drop speeds (§7)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-F1 | `seconds_per_line = (0.8 - (level-1)·0.007)^(level-1)`. | §7 | must **[v1]** |
| FR-F2 | Table levels 1–15. | §7 | must **[v1]** |
| FR-F3 | Soft drop 20×; hard drop 0.0001 s. | §7.1 | must **[v1]** |
| FR-F4 | Piece falls at normal speed even while manipulating; generated piece drops one row immediately. | §3.4, §7 | must **[v1]** |

## Scoring (§8)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-S1 | Single 100·L, Double 300·L, Triple 500·L, Tetris 800·L. | §8 | must **[v1]** |
| FR-S2 | Mini T-Spin 100·L, Mini T-Spin Single 200·L. | §8 | must **[v1]** |
| FR-S3 | T-Spin 400·L, T-Spin Single 800·L, T-Spin Double 1200·L, T-Spin Triple 1600·L. | §8 | must **[v1]** |
| FR-S4 | **Back-to-Back Bonus**: +50% (0.5×action total) for consecutive qualifying actions; only T-Spins/Mini line clears + Tetris qualify; Single/Double/Triple break; no-line T-Spin doesn't start B2B, doesn't break, counts inside; lock w/o clear & hold don't break. | §8, §8.1 | must **[v1]** |
| FR-S5 | Soft drop +1·n; hard drop +2·m; not level-scaled. | §8 | must **[v1]** |
| FR-S6 | §8.1 example verified: Tetris,TSD,T-Spin,Tetris,TSS @L1 ⇒ **5400**. | §8.1 | must **[v1 test]** |
| FR-S7 | (2009 has **no combo score**; COMBOS tracked as stat.) | §8 + UI §2.4.7 | note **[v1 stat]** |

## T-Spins (§9)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-SP1 | T-Spin = T spun CW/CCW into T-Slot before lock; 3-of-4 diagonal corners occupied. | §9, §9.1.1 | must **[v1]** |
| FR-SP2 | Must rotate (not just move/drop) into slot. | §9 | must **[v1]** |
| FR-SP3 | **T-Spin** if: (A&B + (C|D)) touching at lock; or fills T-Slot w/o holes; or **Rotation Point 5** used (then T-Spin not mini). | §9.1.1 | must **[v1]** |
| FR-SP4 | **Mini T-Spin** if: (C&D + (A|B)) touching at lock; or creates holes in T-Slot; unless Point 5 used → T-Spin. | §9.1.2 | must **[v1]** |
| FR-SP5 | T-Spin + no line = T-Spin score but not B2B-start; doesn't break B2B. | §8 n1 | must **[v1]** |

## Game over (§10)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-O1 | **Lock Out**: whole tetrimino locks down above Skyline. | §10.6 | must **[v1]** |
| FR-O2 | **Block Out**: part of newly generated tetrimino blocked by existing block. | §10.7 | must **[v1]** |
| FR-O3 | **Top Out**: (multiplayer) opponent line attacks push blocks past top of buffer. | §10.5 | n/a **[later]** |
| FR-O4 | Non-over: peeking lock-down; playing above skyline; existing blocks above skyline but in buffer. | §10.1–4 | must **[v1]** |
| FR-O5 | Buffer zone used to detect lock-out/block-out/top-out. | §1.4, §10 | must **[v1]** |

## Game over events (§11)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-E1 | Game Over screen; end rewards if above average/high level. | §11.1 | should **[v1/later]** |
| FR-E2 | **High Score Table** per variant; name/initials; level/score/lines; persisted; accessible from menu. | §11.2, A1.2.7 | should **[v1]** |

## Variants (§12)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-M1 | **Marathon**: 15 levels; ends at level 15 completion or game over; scoring. Default variant. | §12.1, §2.3.2 | must **[v1]** |
| FR-M2 | **Sprint (40 Lines)**: choose start level; clear N lines (typ 40) fastest; ends on goal or game over; no level-up. | §12.2 | must **[v1]** |
| FR-M3 | **Ultra**: maximize score or lines in 2–3 min; ends on timer or game over; no level-up. | §12.3 | must **[v1]** (2:00) |
| FR-M4 | Game name begins with "Tetris" for licensed release; variants needn't. | §12 | note |

## Engine (Appendix A1)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-A1 | Eight phases per tetrimino: **Generation, Falling, Lock, Pattern, Iterate, Animate, Eliminate, Completion**. | A1.1, A1.2 | must **[v1]** |
| FR-A2 | Pattern phase matches full rows; mark hit list; no game time. | A1.2.4 | must **[v1]** |
| FR-A3 | Iterate phase hook; unused. | A1.2.5 | must **[v1 hook]** |
| FR-A4 | Animate phase; line-clear effects; no correctness impact. | A1.2.6, §13.1.1 | should **[v1]** |
| FR-A5 | Eliminate: remove hit list; collapse; **award points**; track stats. | A1.2.7 | must **[v1]** |
| FR-A6 | Completion: update Score/Time; **Level Up** check; back to Generation. | A1.2.8 | must **[v1]** |

## Presentation (§2, §13)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-U1 | Screen flow: Power-on → Legal → Title/Logo → Main Menu → New Game/Options/Help/High Score. | §2 | must **[v1 simplified]** (Legal optional learning) |
| FR-U2 | Game screen shows matrix, tetrimino, next, hold, ghost; info fields (type, lines/left, level, time, score, high, goal, stats). | §2.4 | must **[v1]** core fields |
| FR-U3 | Line-clear visual; intensity scales with lines. | §13.1.1 | should **[v1/later]** |
| FR-U4 | Hard-drop trail. | §13.1.2 | should **[later]** |
| FR-U5 | Action notifications: "T-Spin Single", "Back-to-Back T-Spin Double", "Tetris", "Mini T-Spin", ...; quick; outside matrix after level 10. | §13.1.5 | should **[v1]** |
| FR-U6 | Level-change & special-maneuver background cues. | §13.1.4 | should **[later]** |
| FR-U7 | Pause hides matrix/next/hold. | §13.1.6 | must **[v1]** |

## Sound (§13.2, §13.3)

| ID | Requirement | Ref | Pri |
|----|-------------|-----|-----|
| FR-AUD1 | Min SFX: rotate CW/CCW, move L/R, land/wall, lock down, line clear, game over. | §13.2 | should **[later]** |
| FR-AUD2 | Extra SFX: per-count clears, T-Spin variants, B2B, soft/hard drop, hold, move/rotation failure, level up. | §13.2 | should **[later]** |
| FR-AUD3 | BGM on by default; **Korobeiniki** for licensed release; volume; toggle; soundtrack select. | §13.3, §2.5.5 | should **[later]** |

## Non-functional requirements

- **NFR-1 Determinism.** Pure core; seedable PRNG; injectable clock; no wall-clock in
  `core`; replayable. *(§8 scoring exactness demands no float accumulation drift in
  score; score is integer.)*
- **NFR-2 Testability.** `core` has no rendering/OS; 100% of rules unit-testable;
  guideline-derived test vectors.
- **NFR-3 Performance.** 60 fps; tiny grid; trivial CPU; no allocations in hot loop ideally.
- **NFR-4 Correctness on timing.** Fixed timestep; integer-ms timers; no `f32` for
  game logic; exact 0.5 s lock, 0.2 s generation, DAS/ARR constants.
- **NFR-5 No external runtime deps for core;** SDL2 only for presentation.
- **NFR-6 Cross-platform.** Linux primary; Windows/macOS secondary (SDL2).
- **NFR-7 Accessibility.** Remappable keys; not relying on color alone (monochrome
  patterns §3.2 as fallback); clear notifications.
- **NFR-8 Honesty/maintainability.** Code documented; flows kept in sync; no fake
  "done"; tests prove behavior.
- **NFR-9 Licensing.** No protected assets shipped; trademark handling for release
  (separate from code).
