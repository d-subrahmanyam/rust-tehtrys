# Appendix A — Super Rotation System (SRS) Reference

Guideline **A1.3 + A1.4**. Exact conformance is a **test** (08-testing). 

> **Honesty.** Tables below reproduce the published **Super Rotation System** kick
> data that guideline A1.4.2 depicts in its Super Rotation Diagrams. The PDF renders
> those as vector diagrams; this model cannot read image pixels, so conformance
> against the diagrams is verified by fixtures during implementation, not asserted
> here. Semantics (5 ordered tests, O no kick, I vs 3×3, Point-5, **T-piece
> omissions**) are taken from the guideline **text** (§A1.4, §A1.4.1, §9.1.1, §15.4).
> A local-only text-only conversion of the source (not distributed) confirms the
> tables; our implementation is also tested. **T-piece exceptions** ("Not Used"):
> T from North omits Test 4 (N→E CW & N→W CCW); T from South omits Test 3 (S→W CW &
> S→E CCW); **point numbering preserved** (Point 5 stays Point 5).

## Facings (A1.3)

`Facing { North=0, East=1, South=2, West=3 }`. CW `+1 mod 4`; CCW `−1 mod 4`.
Generated North (§3.4); North in Next/Hold (§2.4.3/§2.4.8, A1.3). 

## Rotation mechanic (A1.4)

- Rotate 90° CW/CCW (§5.3). 
- **5 ordered rotation points** (kicks); try Point 1..5; first **non-colliding**
  wins; else rotation fails. (A1.4, A1.4.2)
- **Point 1 = visual center** (no shift); always first. (A1.4.1 A)
- **O**: no kicks; all points same; always visual. (A1.4.1, A1.4.2)
- **I** (4×1): points 2/3 off walls; 4/5 off floor/out of well. (A1.4.1)
- **3×2 T/L/J/S/Z**: 2 off walls; 3 off floor; 4/5 out of wells. (A1.4.1)
- Rotation can move piece any direction; can lift off surface (A1.2.3 note).
- **Point-5** success → T-Spin not Mini (§9.1.1/§9.1.2). 

## Coordinate conventions

- **Guideline native:** origin bottom-left, **1-based**, **y-up**; W=10; total H=40;
  visible rows 1–20; buffer rows 21–40; skyline between 20/21; Top-Out 40. (§1.4,
  §10, A1.2.1)
- **Engine (ours):** origin top-left, **0-based**, **y-down**; `x:0..=9`, `y:0..=39`;
  buffer `y:0..=19`; visible `y:20..=39`; skyline boundary `y=20`.
- **Conversions:** `engine_x = guideline_col - 1`; `engine_y = 40 - guideline_row`.
- **Kick tables below use guideline convention (y-up).** Engine **negates `dy`**
  (`engine_dx=dx`, `engine_dy=-dy`) because engine y is down. *(We verify signs with
  tests; e.g. "off the floor" kick `+y` up in guideline = engine `−y`.)*

## Kick tables (ordered test lists; offsets `(dx, dy)` added to piece origin)

Test 1 always `(0,0)`. **y-up.** `L`=CCW, `R`=CW. (A1.4.2)

### O-Tetrimino (2×2)
All facings; both directions; all 5 points `(0,0)`. No kicks. 

### J, L, S, T, Z (3×3 box; guideline "3×2")

**Clockwise:**

| From→To | Test 1 | Test 2 | Test 3 | Test 4 | Test 5 |
|---------|--------|--------|--------|--------|--------|
| 0→1 (N→E) | (0,0) | (-1,0) | (-1,1) | (0,-2) | (-1,-2) |
| 1→2 (E→S) | (0,0) | (1,0) | (1,-1) | (0,2) | (1,2) |
| 2→3 (S→W) | (0,0) | (1,0) | (1,1) | (0,-2) | (1,-2) |
| 3→0 (W→N) | (0,0) | (-1,0) | (-1,-1) | (0,2) | (-1,2) |

**Counterclockwise:**

| From→To | Test 1 | Test 2 | Test 3 | Test 4 | Test 5 |
|---------|--------|--------|--------|--------|--------|
| 0→3 (N→W) | (0,0) | (1,0) | (1,1) | (0,-2) | (1,-2) |
| 3→2 (W→S) | (0,0) | (-1,0) | (-1,-1) | (0,2) | (-1,2) |
| 2→1 (S→E) | (0,0) | (-1,0) | (-1,1) | (0,-2) | (-1,-2) |
| 1→0 (E→N) | (0,0) | (1,0) | (1,-1) | (0,2) | (1,2) |

### I-Tetrimino (4×4 box; 4×1 piece)

**Clockwise:**

| From→To | Test 1 | Test 2 | Test 3 | Test 4 | Test 5 |
|---------|--------|--------|--------|--------|--------|
| 0→1 (N→E) | (0,0) | (-2,0) | (1,0) | (-2,-1) | (1,2) |
| 1→2 (E→S) | (0,0) | (-1,0) | (2,0) | (-1,2) | (2,-1) |
| 2→3 (S→W) | (0,0) | (2,0) | (-1,0) | (2,1) | (-1,-2) |
| 3→0 (W→N) | (0,0) | (1,0) | (-2,0) | (1,-2) | (-2,1) |

**Counterclockwise:**

| From→To | Test 1 | Test 2 | Test 3 | Test 4 | Test 5 |
|---------|--------|--------|--------|--------|--------|
| 0→3 (N→W) | (0,0) | (-1,0) | (2,0) | (-1,2) | (2,-1) |
| 3→2 (W→S) | (0,0) | (-2,0) | (1,0) | (-2,-1) | (1,2) |
| 2→1 (S→E) | (0,0) | (1,0) | (-2,0) | (1,-2) | (-2,1) |
| 1→0 (E→N) | (0,0) | (2,0) | (-1,0) | (2,1) | (-1,-2) |

## Rotation Point purposes (A1.4.1) (summary)

Point1 visual; Point2 off right/left wall; Point3 off floor (3×2) / reverse (I);
Point4/5 out of left/right well / reverse; off floor (I). (See guideline diagrams.)

## Spawn mapping (§3.4 + A1.2.1) → engine

- North. 3-wide (T/L/J/S/Z): guideline cols 4–6 row21 (+row22, §3.4); → engine trio
  **cols 3–5 row19**, extra **row18**; **center mino engine (4,19)**; 3×3 origin
  **(3,18)**. 
- I: guideline row21 cols4–7 → engine **row19 cols3–6**; 4×4 origin **(3,19)**;
  Point-1 center at box center (not a cell). 
- O: guideline row21 cols5–6 → engine **row19 cols4–5**; 2×2 origin **(4,19)**. 
- On generation: drop 1 row if clear; blocked→no; Block-Out check; ghost; +0.2 s. 

Exact per-orientation cells: **[appendix B](B-piece-data.md)**. 

## T-Spin recognition reference (§9.1) → flows/08

3-of-4 diagonal corners; facing-specific A/B/C/D; last action rotation; **full** vs
**mini**; Point-5; no-line nuances. (Recreated as fixtures; can't machine-read
diagrams.)
