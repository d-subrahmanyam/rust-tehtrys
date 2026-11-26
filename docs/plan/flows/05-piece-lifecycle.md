# Flow 05 — Piece Lifecycle

One piece at a time; full path through the engine. §3.4/A1.2 + §5. See
[02-engine-phases](02-engine-phases.md).

```mermaid
flowchart TD
  Spawn[Generation §3.4/A1.2.1\nfrom 7-bag; North; row21/22; center(5,21);\nspawn rules; +0.2s; Block-Out check]
  Spawn --> Drop1[drop 1 row if clear §3.4]
  Drop1 --> Falling[Falling §A1.2.2\ngravity fall_ms(level); move; rotate SRS; soft20x; hold]
  Falling -->|resting| Locking[Lock Phase §A1.2.3\n0.5s timer; rulesets; can still act]
  Falling -->|hard drop| LockNow[lock immediately]
  Locking -->|act: move/rotate + reset rules| Falling
  Locking -->|act causes fall again| Falling
  Locking -->|timer expires / cap / hard drop| Commit[commit cells to matrix\nlock flash; Lock-Out check §10.6]
  LockNow --> Commit
  Commit --> Pattern[Pattern §A1.2.4 hit list]
  Pattern --> Iterate[Iterate §A1.2.5 unused]
  Iterate --> Animate[Animate §A1.2.6 FX]
  Animate --> Eliminate[Eliminate §A1.2.7 clear+collapse+SCORE+stats]
  Eliminate --> Completion[Completion §A1.2.8 score/time; LEVEL UP; variant end]
  Completion -->|+0.2s| Spawn

  %% hold branch
  Falling -.->|Hold §5.6 once/lock| HoldOp
  Locking -.->|Hold §5.6| HoldOp
  HoldOp[hold swap\nincoming=previously held or next-from-bag;\nNorth; fresh spawn; lock reset] -.-> Spawn
```

## Per-piece state

```
struct ActivePiece { kind, facing, origin,
                     last_was_rotation, last_kick_index(1..=5), last_rotation_from }
struct LockState   { timer_ms, moves_used, lowest_row }
hold_used_this_piece: bool
```

## Lifecycle invariants

- One active piece; it never overlaps locked cells; always in a valid resting/falling
  position; spawn always validated (Block Out).
- Reaching **resting** doesn't end piece; only Lock-Phase completion does.
- Movement/rotation reset rules depend on lock ruleset (§5.7–5.9).
- Hold resets incoming piece state; once per lock (§5.6).
- After Eliminate/Completion, B2B/combo/stats/level updated; next piece spawns.

## What resets on new piece
lock timer; move-count; lowest-row; hold-used flag; last-move T-Spin flags; gravity
accumulator; (DAS in input persists across pieces, §5.2!). B2B persists. 

## Hold lifecycle detail → [09-hold-and-next](09-hold-and-next.md)
