# 05 — Scoring & Progression

All scoring from guideline **§8**; levels & goals **§6**; speeds **§7**. Exact
integer math. Cross-refs: [03](03-game-engine.md), [flows/07](flows/07-line-clear-and-scoring.md),
[appendix C](appendices/C-fall-speed-table.md).

## Scoring table (§8) — base × level `L`

| Action | Points | Description |
|--------|--------|-------------|
| Single | `100·L` | 1 line cleared |
| Double | `300·L` | 2 lines |
| Triple | `500·L` | 3 lines |
| **Tetris** | `800·L` | 4 lines |
| **Mini T-Spin** | `100·L` | T-Spin, no line (§8 "Mini T-Spin") |
| **Mini T-Spin Single** | `200·L` | Mini T-Spin clearing 1 line |
| **T-Spin** | `400·L` | T-Spin, no line |
| **T-Spin Single** | `800·L` | 1 line |
| **T-Spin Double** | `1200·L` | 2 lines |
| **T-Spin Triple** | `1600·L` | 3 lines |
| **Back-to-Back Bonus** | `+0.5 × action total` | on qualifying consecutive actions |
| **Soft Drop** | `1 × n` | n cells soft-dropped |
| **Hard Drop** | `2 × m` | m cells hard-dropped |

Notes:
- Line-clear, T-Spin, Mini T-Spin scoring is **level dependent**; drop scoring is
  **not** (§8).
- Levels start at 1; typical max 15 (§8).
- 2009 guideline has **no combo scoring** (§8); `COMBOS` is a **displayed stat**
  (§2.4.7). We track a combo counter for UI/stats only.
- A 2009 requirement noted in §8: ship an in-app **demonstration** of all scoring
  actions. We scope this as a **Help/Demo** screen item (not core engine).

## Back-to-Back (B2B) (§8, §8.1, §9)

**Qualifying (B2B) actions:** Tetris; T-Spin line clears (T-Spin Single/Double/
Triple); Mini-T-Spin line clears (Mini T-Spin Single; §8 n1 says "Mini T-Spin
Line Clears" qualify); i.e. `line_clears>0 && (tetris || t_spin || mini_t_spin)`.

**Breaking:** §8 says a B2B sequence is "only broken by a Single, Double, or Triple
Line Clear" and lists which actions qualify for B2B; combined with §8 n1 (no-line
T-Spins don't break), a breaker is a **non-qualifying** Single/Double/Triple (a plain
line clear that is not a T-Spin/Mini and not a Tetris). §8.1 example has
`... Tetris, + B2B, T-Spin Single, + B2B ...`, i.e. T-Spin Single does **not** break.
Precisely:

- If action is **qualifying**: if B2B already active ⇒ add `+0.5×` (and extend); else
  start B2B. 
- If action is **non-qualifying line clear** (Single/Double/Triple that isn't a
  T-Spin/Mini): **break** B2B.
- If action is **no-line-clear** (incl. T-Spin no-line & Mini T-Spin no-line, §8 n1):
  **no change** (doesn't start, doesn't break; counts inside sequence).
- **No-line action / lock without clear / hold:** doesn't break (§8.1).

**T-Spin no-line nuance (§8 n1):** T-Spin/Mini with no line: scored normally; does
**not** receive B2B bonus; **cannot start** B2B; **does not break**; remains inside.

### B2B example (§8.1) — required test

Level 1 sequence: **Tetris, T-Spin Double, T-Spin, Tetris, T-Spin Single** ⇒
```
 800 + (1200+600) + 400 + (800+400) + (800+400) = 5400
```
(T-Spin no-line `400` doesn't break B2B nor get bonus; note in guideline the `+B2B`
after T-Spin Single example totals accordingly; our rule reproduces **5400**.)

## Levels & goals (§6)

- **Levels 1–15.** Level affects **fall speed** & **scoring**. (§6)
- **Starting level 1–15** from menu; default 1. (§2.3.3, §6)
- Level Up when **goal** reached; checked in Completion Phase (A1.2.8).

### Goal systems (§6) — pick one; engine supports both

**Fixed Goal** (§6): 10 lines/level through level 15; total 150. Starting level `L`:
**Goal = 10·L cumulative** to reach next level (i.e. sum prior + current; example L4→5
needs 40 total). Per-level goal shown = 10. (§6)

**Variable Goal** (§6 + §6.1): level `n` requires `5·n` lines; level 1=5 … level 15=
600 total; starting at level ≥2 ⇒ **goal = level·5**. Plus **awarded line clears**
from scoring (§6.1):

| Action | Awarded Line Clears | | Action | Awarded Line Clears |
|---|---|---|---|---|
| Single / Mini T-Spin | 1 | | T-Spin Double | 12 |
| Mini T-Spin Single | 2 | | T-Spin Triple | 16 |
| Double | 3 | | Back-to-Back Bonus | 0.5 × total clears |
| T-Spin | 4 | | | |
| Triple | 5 | | | |
| Tetris / T-Spin Single | 8 | | | |

**Decision (FR-V2):** ship **both**; **default Marathon = Fixed Goal** (simplest,
15-level match §12.1); Variable available. *(Marked deferred final pick; engine
plumbing identical.)*

## Fall & drop speeds (§7, appendix C)

```
seconds_per_line(level) = (0.8 - (level - 1) · 0.007)^(level - 1)     // §7
```
Stored as integer ms; `fall_ms = round(seconds·1000)`, clamp `>= 1`. Table levels
1–15 in appendix C (1.0, 0.793, 0.618, 0.473, 0.355, 0.262, 0.190, 0.135, 0.094,
0.064, 0.043, 0.028, 0.018, 0.011, 0.007 s).

- Piece falls at normal speed **even while manipulated**; generated piece drops one
  row immediately if clear. (§3.4, §7)
- **Soft Drop** `fall_ms/20`; **Hard Drop** `0.0001 s` (instant). (§7.1)
- `LOCK_DOWN=0.5 s`, `GENERATION=0.2 s`, `DAS=0.3 s`, `ARR≈0.5 s width`. (§5.4,5.7,
  A1.2.1, §5.2; appendix C)

## Scoring flow at Eliminate

```
action = classify(locked_piece, lines, last_move, tspin, matrix_before_clear)
base   = table_points(action) · level
if b2b_qualifying(action) && b2b_active { bonus = base / 2; }   // 0.5x action total
score += base + bonus + drop_points_this_piece
update B2B; update combos/stats; (Variable-Goal extra clears §6.1)
```

`score` integer `u64`. Drop points accrued during the piece, added at lock.
See [flows/07-line-clear-and-scoring.md](flows/07-line-clear-and-scoring.md).

## Statistics (§2.4.7, A1.2.7, §13)

Score, High, Time (elapsed/remaining), Lines, Level, Goal, Tetrises, T-Spins,
Combos, TPM (tetriminos/min), LPM (lines/min), Singles/Doubles/Triples/Tetrises,
Mini-T-Spin/T-Spin counts, B2B count, pieces placed. Recorded at Eliminate (A1.2.7).
