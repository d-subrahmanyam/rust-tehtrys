# Flow 06 — Lock Down

Guideline §5.4/§5.7–5.9 + A1.2.3. Hard-drop instant; else 0.5 s; three rulesets;
**Extended default**. (Requirement FR-L1..L8.)

```mermaid
flowchart TD
  Rest[piece resting\nstep-down would collide] -->|hard drop| Immediate[lock immediately §5.4]
  Rest -->|natural fall / soft drop| Start[start Lock Down Timer 0.5s §5.7-5.9\nstate=Locking]
  Start --> Check{tick / events}
  Check -->|timer 0.5s elapsed AND on surface| Commit[LOCK §A1.2.3]
  Check -->|timer expired but NOT on surface?| Note1[guideline: only locks while...\nre-check resting; engine tracks surface]
  Check -->|player MOVE success| ResetRule{ruleset}
  Check -->|player ROTATE success| ResetRule
  Check -->|rotation lifts piece OFF surface| LiftOff[A1.2.3: timer NOT reset;\nPAUSED until re-landing on same/lower... surface]
  Check -->|move/rotate causes piece to FALL again| ReenterFall[re-enter Falling §A1.2.3\n→ re-establish resting]
  Check -->|cap reached Extended| Cap[15 moves/rotations used:\ntimer NOT reset; lock on first surface touched §5.7\n(if not resting, can still act; no more resets)]
  Check -->|falls one row BELOW lowest reached| ResetCap[reset 15-cap §5.7\n(also reset timer per ruleset)]
  LiftOff -->|re-land same-or-lower... surface| ResetRule
  ResetRule -->|Extended| Ext[timer→0.5s; (cap not reset here)]
  ResetRule -->|Infinite| Inf[timer→0.5s any real change]
  ResetRule -->|Classic| Cls[timer→0.5s ONLY if falls further §5.9]
  ResetRule --> Start
  ReenterFall --> Rest
  Immediate --> Commit
  Commit --> Locked[commit cells; lock flash; Lock-Out §10.6; → Pattern]
```

## Rulesets (§2.5.4)

### Extended Placement — **default**; required multiplayer/arcade (§5.7)
- 0.5 s timer once resting.
- Timer **resets to 0.5 s** on simply moving or rotating (successful change).
- **15** total left/right movements + rotations; once used, timer **not reset**;
  **lock on first surface touched**; if not resting, can still move/rotate further
  (and falling to a **new lowest line** resets cap & grants 15 again, §5.7).
- "Falls one row below lowest row yet reached" resets the 15-cap (not necessarily
  the timer).
- A1.2.3 lift-off note applies (below).

### Infinite Placement (§5.8)
- 0.5 s timer; resets on any **actual change in position or orientation**; can keep
  moving/rotating while real change occurs; no cap.

### Classic (§5.9)
- 0.5 s timer; resets **only if y-coordinate decreases** (piece falls further down;
  guideline y-up). Rotation/move-not-down doesn't reset. (In engine y-down: y
  increases when falling.)

## A1.2.3 surface / lift-off semantics (must implement exactly)

- Rotating often moves piece **up** off its surface ("lifts up"). 
- **Timer does NOT reset** in that case.
- Timer **stops counting down** while piece is off-surface.
- **Verbatim (guideline A1.2.3, y-up):** *"rotating a Tetrimino often causes the
  y-coordinate of the Tetrimino to increase, i.e. it 'lifts up' off the Surface it
  landed on. The Lock Down Timer does not reset in this case, but it does stop
  counting down until the Tetrimino lands again on a Surface that has the same (or
  higher) y-coordinate as it did before it was rotated. Only if it lands on a
  Surface with a lower y-coordinate will the timer reset."*
- **Engine mapping (y-down):** signs flip; semantic preserved — timer not reset on
  lift-off; paused while off-surface; re-landing on a surface **no lower** than
  before doesn't reset; re-landing on a **lower** surface resets. **Pinned by
  tests**; not re-stated with raw signs here to avoid error.
- Moving/rotating so piece **falls again** → **Falling** (not still Locking) (A1.2.3).
- Hard Drop bypasses all of this → immediate lock.

## Failed actions
Failed move / failed rotation = no-op; **no reset**; no cap consumption (only
successful changes count; §5.7). 

## After lock
→ Pattern (A1.2.4); scoring etc.; next piece 0.2 s later. Lock-Out if whole piece
above skyline (§10.6).
