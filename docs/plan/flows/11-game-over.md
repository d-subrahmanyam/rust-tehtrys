# Flow 11 — Game Over

§10 conditions; §10.1 diagrams; §11 events; buffer zone. 

```mermaid
flowchart TD
  %% NON-OVER first
  subgraph NotOver[NOT game over §10.1-4]
    N1[Lock Down normal: whole piece below skyline]
    N2[Lock Down peeking: some blocks above + below skyline]
    N3[Playing above skyline: move/rotate in buffer]
    N4[Existing blocks forced above skyline but within 20-row buffer]
  end
  %% OVER
  LO[Lock Out §10.6\nwhole tetrimino LOCKS DOWN above skyline] --> Over
  BO[Block Out §10.7\nstarting cell(s) of NEW tetrimino occupied] --> Over
  TO[Top Out §10.5 MULTIPLAYER\nopponent line attacks push past top-out line] --> Over
  Over[GAME OVER] --> End[stop engine; emit GameOver reason]
  End --> Events[Game Over Events §11\nend rewards §11.1; high score §11.2]
  Events --> Menu[Main Menu]
```

## Conditions (§10)

**Game over when any:**
- **(a) Top Out** — (multiplayer) opponent line attacks push blocks past top of
  **20-row buffer**; *not v1* (§10.5, Appendix B seam).
- **(b) Lock Out** — a **whole** tetrimino **locks down above the Skyline** (every
  cell in buffer; §10.6). Peeking (straddling skyline) is **not** over (§10.2).
- **(c) Block Out** — **part of a newly generated tetrimino is blocked** by an
  existing block (§10.7).

**Buffer Zone** (10×20 above skyline) used to detect Lock Out/Block Out/Top Out
(§1.4, §10); **Top Out Line** = top of buffer, 20 above skyline. 

## Non-over situations (§10.1–4)
1. Normal lock below skyline. 2. Peeking lock (≥1 block above & below skyline). 
3. Playing/rotating above skyline. 4. Existing blocks above skyline but inside buffer.
Plus a generated piece shown partially above skyline (§3.4) isn't itself over; only
blocked spawn (Block Out) or full lock above (Lock Out) is.

## When checked
- **Block Out:** at **Generation** (§10.7/§3.4) after spawn placement.
- **Lock Out:** at **Lock** commit (§10.6/A1.2.3).
- **Top Out:** at multiplayer line-attack application (§10.5/B1.3.3); seam only. 
- (Variant endings §12 are *not* game-over; reported separately.)

## After game over (§11)
- Stop core; emit `GameOver{reason}`; 
- **End Rewards** if above-average/high level (§11.1) *[optional]*; 
- **High Score Table** (§11.2/A1.2.7): qualify→enter name; per variant; persist;
  show top; menu access. Save stats. 
- Variant endings (Sprint clear / Ultra time / Marathon 15) → completion screen +
  same §11 events, distinguished from Game Over.

## Reason enum
`GameOverReason::{ LockOut, BlockOut, TopOut }`.

## Phase-6 (not built)

`TopOut` (§10.5) is a **seam**, not emitted in v1 single-player; multiplayer
garbage → top-out would set it. See [appendices/E-multiplayer-seams.md](../appendices/E-multiplayer-seams.md).
