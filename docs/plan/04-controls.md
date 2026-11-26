# 04 — Controls

Keyboard-first; abstracted `GameCommand` so input source is pluggable. Guideline
§4 + §4.1 + §5. Flows: [flows/04-input-flow.md](flows/04-input-flow.md).

## Default keyboard mapping

Chosen to be **guideline-compatible** (§4.1 keyboard scheme) with a common clean
default; **all remappable** (§4). We ship this default:

| Action | Primary | Secondary (guideline §4.1) | `GameCommand` |
|--------|---------|----------------------------|---------------|
| Move left | ← ArrowLeft | Numpad 4 | `MoveLeft` |
| Move right | → ArrowRight | Numpad 6 | `MoveRight` |
| Soft Drop | ↓ ArrowDown | Numpad 2 | `SoftDrop(true/false)` |
| Hard Drop | Space | Numpad 8 | `HardDrop` |
| Rotate Clockwise | ↑ ArrowUp | X, Numpad 1/5/9 *(guideline)* | `RotateCw` |
| Rotate Counterclockwise | Z | Left Ctrl, Numpad 3/7 *(guideline)* | `RotateCcw` |
| Hold | C | Left Shift, Numpad 0 *(guideline)* | `Hold` |
| Pause | Escape | F1 *(guideline)* | `Pause`/`Resume` |

Notes:
- Guideline table (§4.1) lists Hard Drop = **Space / Numpad 8** and Rotate-CW =
  **↑ / X / Numpad 1,5,9** on keyboard; its keyboard *diagram* differs. We follow
  the **table**, and make it remappable. Our default binds ↑ to rotate-CW (hard drop
  = Space), matching the guideline table. *(§4.1)*
- Hard drop no auto-repeat (§5.4). Rotation no auto-repeat (§5.3). Hold one per lock
  (§5.6). Pause toggles.
- Menu navigation keys separate (arrows/Enter/Esc), not game commands.

## Movement & auto-repeat (§5.1, §5.2)

**DAS/ARR** ("Delayed Auto-Shift / Auto-Repeat Rate"), implemented in `input::das`,
emitting discrete `MoveLeft/Right` to core. (Auto-repeat **only** left/right.)

| Parameter | Guideline | Value |
|-----------|-----------|-------|
| DAS initial delay | ~0.3 s (§5.2) | `DAS_MS = 300` |
| ARR | full width in ~0.5 s total (§5.2) | `ARR_MS = 500.0/10 ≈ 33.3 ms` per cell after DAS |

Behavior (§5.2):
- **Tap** = single cell.
- **Hold** = after `DAS`, repeat every `ARR`.
- Auto-repeat **continues to next piece** after lock-down while button held (§5.2,
  §5.5).
- **Opposite direction:** if repeating one way then press opposite, begin opposite
  with **fresh DAS delay**; releasing one while other still held resumes held
  direction with DAS applied once more. (§5.2)
- Rotation allowed while auto-repeating. (§5.3)
- DAS/ARR while piece resting: moves reset lock-down per ruleset. (§5.7–5.9)
- We choose sensible repeat for a 10-wide matrix; guideline only mandates DAS ~0.3 s
  + ~0.5 s full traversal.

## Soft drop (§5.5, §7.1)

- Holding Soft Drop: `fall_ms/20` (`SOFT_DROP_MULTIPLIER=20`).
- Release → normal speed. Lock-down timer **not** cancelled by reaching surface; it
  runs once resting (§5.5). +1/cell (§8). Continues to next piece while held (§5.5).
- No extra auto-repeat logic needed (continuous while held).

## Rotation (§5.3, A1.4)

- CW/CCW 90°; **SRS** kicks; no auto-repeat; works while auto-repeating; works
  against walls/floor/wells. Failed rotation = no-op (`RotationFailed`). Rotation
  resets lock-down per ruleset. (appendix A)

## Hold (§5.6, §2.5.2)

- Once per lock; swap with held; incoming North-facing spawn; no hold-twice; lock
  between holds; default on; (guideline) doesn't break B2B. (flows/13)

## Pause (§13.1.6)

- Instant stop; **hide matrix/next/hold**; prominent "Pause"; toggle; pausing halts
  all timers (lock, generation, gravity, DAS, variant clock).

## Lock-down modes (§5.7–5.9, §2.5.4)

Selectable; **Extended default**; hard-drop always instant. See
[03](03-game-engine.md#lock-down-§5457959-a123).

## Input → command mapping

```
SDL KeyboardEvent
  └─ keymap (KeyCode → Option<GameCommand>, with press/release + repeat flag)
       ├─ discrete (press edge): RotateCw/Ccw, HardDrop, Hold, Pause   (ignore SDL auto-repeat)
       ├─ level-triggered:       SoftDrop(held)
       └─ movement:              feed DAS/ARR FSM → emits MoveLeft/Right repeats
            └─ core.apply(GameCommand, dt)
```

- We **ignore SDL key-repeat** for our own DAS (full control, §5.2).
- Swallow input while paused / during app-state transitions; while paused only
  Resume/quit/menu work.
- On focus loss, auto-pause (we won't miss releases); clear held keys on resume.

## Remapping & persistence

- `KeyMap` serializable; UI later; reset-to-default. Conflicts rejected. Stored in
  settings (`persist`). (§4 "player is allowed to change the control keys".)

## Controller / touch (later)

Guideline gives joystick/mobile/touch schemes (§4.1 + touch §…). Input abstraction
allows adding; not v1. (e.g. touch rotate dividing line, etc. — not required.)
