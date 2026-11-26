# Flow 07 — Line Clear & Scoring

Pattern→hit-list→Animate→Eliminate→scoring. §1.4/§8 + A1.2.4–8. See
[05-scoring](../05-scoring-and-progression.md).

```mermaid
flowchart TD
  Commit[piece locked] --> Snap[snapshot matrix for T-Spin analysis §9.1\n(3-of-4 corners; pre-clear)]
  Snap --> Pattern[Pattern Phase A1.2.4\nscan rows; full 10-cell rows -> HIT LIST\nno time]
  Pattern -->|any hit?| Animate[Animate Phase A1.2.6\nline-clear FX intensity scales w/ count §13.1.1\nno time; cosmetic]
  Pattern -->|no hit| NoClear[no line clear]
  Animate --> Elim[Eliminate Phase A1.2.7]
  Elim --> Remove[remove hit-list cells; collapse above; no time]
  Elim --> Classify[classify action §8:\ncount + tspin/mini + last-move + B2B state]
  Classify --> Score[SCORE:\nbase=table·level + 0.5x if B2B qualifying & active\n+ accumulated drop points\ninteger; A1.2.7 update stats §13.1.5 event]
  Score --> UpdateB2B[update Back-to-Back §8.1 + combo-stat + variant clears §6.1]
  NoClear --> UpdateB2B2[B2B unchanged unless non-qualifying clear...\nno-line T-Spin doesn't break §8n1; lock-without-clear doesn't break §8.1\ncombo resets]
  UpdateB2B --> Completion[Completion A1.2.8\nScore/Time update; LEVEL UP §6 check; variant end §12; +0.2s next gen]
  UpdateB2B2 --> Completion
```

## Hit list (A1.2.4)
Full row = 10 cells occupied. Mark all cells in matched rows. Multiple rows in one
lock → count N (1..4). Cleared simultaneously. All above shift down (Eliminate).

## Classification (§8 + §9)

```
action =
  lines==0:
     t==T && tspin_full   -> TSpin (no line)        // §9.1.1
     t==T && tspin_mini   -> MiniTSpin (no line)    // §9.1.2
     else                 -> no line
  lines==4                 -> Tetris                 // (never T-Spin; T-Spin max 3)
  t==T && tspin_full:
     lines==1 -> TSpinSingle; lines==2 -> TSpinDouble; lines==3 -> TSpinTriple
  t==T && tspin_mini:
     lines==1 -> MiniTSpinSingle; (mini max 1 line; §9.1.2)
  else: lines==1 Single; 2 Double; 3 Triple
```
T-Spin requires last action was a **rotation** into slot (§9). T-Spin detection uses
**pre-clear** matrix + facing. See [08-t-spin-detection](08-t-spin-detection.md).

## Scoring (§8) — integer, level-scaled except drops

```
points = base(action) * level
if qualifying(action) && b2b_active: points += base(action) * level / 2   // +0.5x action total
score += points + drop_points_accumulated_this_piece
```
See table in [05](../05-scoring-and-progression.md#scoring-table-§8--base--level-l).

## Back-to-Back (§8, §8.1)

- qualifying = Tetris OR T-Spin line clear OR Mini-T-Spin line clear. (§8n1 mini line
  qualifies; no-line doesn't.)
- on qualifying: `b2b_active ? add 0.5x : (start; no bonus on first)`.
- **break** on non-qualifying Single/Double/Triple. no-line/no-clear & hold don't
  break; no-line T-Spin doesn't start; counts inside. 
- **§8.1 verified test ⇒ 5400.**

## Combos (2009)

`combo` counts consecutive pieces that cleared ≥1 line; resets on a lock-without-
clear; **no scoring**; displayed stat. §8 has no combo points; §2.4.7 shows COMBOS.
*(Some Tetris variants add combo scoring; 2009 guideline does not.)*

## Drop scoring (§8)
Accumulated during piece; soft +1/cell; hard +2/cell; not level-scaled; added at
lock. Hard-drop distance = cells fallen by hard drop. Soft-drop cells counted while
soft-dropping. 

## Stats (A1.2.7)
Update at Eliminate: singles/doubles/triples/tetrises, t-spins, mini-t-spins, b2b,
combos, lines, pieces, TPM/LPM; score; level/goal handled in Completion (§A1.2.8).

## Events emitted
`PatternMatched`, `Animate`, `Eliminated`, `LinesCleared`, `Scored`, `TSpin`,
`LevelUp`, `Completed`. (flows/12)
