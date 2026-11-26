# Flow 08 — T-Spin Detection

Guideline **§9 / §9.1**. Only T tetrimino. Full vs **Mini**. Detect at **lock**,
score at **Eliminate**. See appendix A; FR-SP1..5.

## T-Slot (§9)
A block formation such that when T is spun in, **any 3 of the 4 cells diagonally
adjacent to T's center** are occupied (by existing blocks, walls, floor count as
occupied for corner testing — engine checks matrix bounds/occupancy). 

## Corner labeling (§9.1) — depends on facing

For T facing **N/E/S/W**, four diagonal corners around center mino are labeled
**A, B, C, D** as in guideline diagrams. Front corners vs back corners flip with
facing. We store per-facing corner offsets; test `occupied(center + off)` (out of
bounds/wall/floor = occupied). 

| Facing | Front pair | Back pair | (guideline letters) |
|--------|-----------|-----------|---------------------|
| North | A,B | C,D | North row: A B / C D (§9.1) |
| East | B,C→... | ... | per diagram |
| South | C,D | A,B | |
| West | ... | ... | per diagram |

(Exact per-facing letter placement comes from guideline §9.1 diagrams; engine uses
**front/back pair counts**, which is equivalent; appendix A has reference. Verified
with fixtures; can't machine-read diagram pixels.)

## Recognition algorithm (§9.1.1 + §9.1.2)

```mermaid
flowchart TD
  L[piece locked] --> T{kind == T?}
  T -->|no| NoTS[not a T-Spin]
  T -->|yes| Rot{last successful action was a ROTATION §9\nnot mere move/drop?}
  Rot -->|no| NoTS
  Rot -->|yes| Count[count occupied of 4 diagonal corners\n(pre-clear matrix; walls/floor count)]
  Count -->|<3 of 4| NoTS
  Count -->|3 or 4 occupied -> T-Slot| Kind{classify}
  Kind -->|A&B + (C or D)  [front 2 + 1 back]| Full1[T-SPIN §9.1.1]
  Kind -->|fills T-Slot completely, no holes| Full2[T-SPIN §9.1.1]
  Kind -->|Rotation POINT 5 used A1.4 §9.1.1| Full3[T-SPIN not mini §9.1.1]
  Kind -->|C&D + (A or B)  [back 2 + 1 front]| Mini1[MINI T-SPIN §9.1.2]
  Kind -->|creates holes in T-Slot| Mini2[MINI T-SPIN §9.1.2]
  Kind -->|mini BUT point 5 used| Full3
  Full1 --> Score[scoring §8 flows/07]
  Full2 --> Score
  Full3 --> Score
  Mini1 --> Score
  Mini2 --> Score
```

## Precise rules (§9.1.1 / §9.1.2)

**T-Spin (full) if ANY:**
- Sides **A & B + (C or D)** touching a surface at lock; **or**
- T fills T-Slot **completely with no holes**; **or**
- **Rotation Point 5** used to rotate into slot (then T-Spin, **not** Mini); any
  further rotation still T-Spin. (A1.4.1 kick #5; our `last_kick_index==5`.)

**Mini T-Spin if EITHER:**
- Sides **C & D + (A or B)** touching at lock; **or**
- T creates **holes** in a T-Slot; **unless** Point 5 used → then T-Spin. (§9.1.2)

**Also required (§9):** T must **spin clockwise or counterclockwise** into slot;
last action **rotation**, not move/drop. T-Spin no-line still scored; B2B rules
(§8n1); Mini no-line (§8). 

## State tracked per piece (for detection)
`last_was_rotation`, `last_kick_index`, `last_rotation_from/to`, current `facing`,
`origin`, and **pre-clear matrix snapshot**. Reset appropriately on move/drop/lock.
A subsequent move after rotation cancels T-Spin (must be last action = rotation).

## Scoring consequences (§8 → flows/07)
T-Spin 400·L; TSS 800·L; TSD 1200·L; TST 1600·L; Mini 100·L; MiniSingle 200·L;
B2B for line-clearing T/Mini + Tetris; no-line T-Spin doesn't start/break B2B (§8n1).
Notifications "T-Spin Single"/"Mini T-Spin"/"Back-to-Back T-Spin Double" (§13.1.5).

## Guideline example scenarios (§9.1.1–9.1.2 diagrams)
T-Spin E-S/N-W/N-E; T-Spin Single; T-Spin Double; T-Spin Triple; Mini T-Spin; Mini
T-Spin Single variants. We recreate these as **hand-built matrix fixtures** in tests
(see 08-testing). We cannot machine-read diagram pixels; fixtures are derived from
the textual rules. 
