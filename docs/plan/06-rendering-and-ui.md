# 06 — Rendering & UI

SDL2. Pure read of `RenderSnapshot`; animation separate; cosmetic only. Guideline
**§2** (flowchart, layout, options) and **§13** (effects). Flow:
[flows/12-rendering.md](flows/12-rendering.md).

## Window & scaling

- Window e.g. 1000×800; **integer cell size**; letterbox; scale to fit; matrix
  centered with HUD side panels. No background art v1 (must not interfere, §13.1.4).
- `render::layout` computes rects: `matrix_rect`, `next_rect`, `hold_rect`,
  `hud_rect`, `notify_rect`.

## Colors (§3.1)

| Piece | Color | | Piece | Color |
|-------|-------|-|-------|-------|
| O | yellow | | L | orange |
| I | light blue (cyan) | | J | dark blue |
| T | purple | | S | green |
| | | | Z | red |

- Matrix empty cells dark; grid lines subtle; locked blocks slightly darkened vs
  in-play; **lock-down flash** (§13.1.3); ghost = outline/translucent; **monochrome
  patterns** fallback (§3.2) for accessibility (NFR-7).
- We ship no protected artwork; colors are plain.

## Game screen layout (§2.4) — order matches guideline numbering

```
        ┌──────────────┐   ┌────────────────────────────┐   ┌──────────────┐
        │ HOLD (8→topL)│   │        MATRIX 10×20 (1)    │   │ NEXT  (3→topR)│
        │  North-facing│   │  + active piece (2)        │   │  North-facing │
        └──────────────┘   │  + ghost piece (4)         │   │  1..6 visible │
                           │  skyline / buffer hint     │   └──────────────┘
   HUD / Game Information (7):
   TYPE · LINES / LEFT · LEVEL · TIME · SCORE · HIGH · PLAYER · GOAL ·
   TETRISES · T-SPINS · COMBOS · TPM · LPM · (starting blocks indicator)
                           └────────────────────────────┘
   Notifications (§13.1.5): quick text "Tetris", "T-Spin Double",
   "Back-to-Back T-Spin Double", "Mini T-Spin", ... inside matrix ≤L10, else outside.
```

- **Next Queue** (§2.4.3): top-right; North-facing; next-to-play first; **1–6**
  visible (default 6; §2.5.1). 
- **Hold** (§2.4.8): top-left; North-facing; dim when used-this-piece. (§5.6)
- **Ghost** (§2.4.4): landing position; default on. (§2.5.3)
- **Hold/Next shown North-facing** always. (§2.4.3, §2.4.8, A1.3)

## Screen flow (§2) — simplified v1; full in flows/14

```
PowerOn → Legal(optional) → Title/Logo → MainMenu
   MainMenu → NewGame → (Select #Players=1 | Select Variation | Select Start Level) → Game
            → Options → MainMenu
            → Help → MainMenu
            → HighScore → MainMenu
   Game → Pause → Game
        → GameOver → (EndRewards?) → HighScore(enter name) → MainMenu
```

- **Legal screen** (§2.1) & **Tetris logo** (§2.2) required for *licensed* release;
  simplified/optional for learning build; no protected logo asset. 
- **Main Menu** (§2.3): select variation (default **Marathon** §2.3.2), starting
  level (§2.3.3), High Score (§2.3.4), Help (§2.6), Options (keys/sound §2.3).
- **Help** (§2.6): how to play, scoring, terminology (use correct terms; call it
  "Matrix"/"Tetrimino" not "Grid"/"Block"-only), credits (licensed). (+ §8 demo.)

## Options (§2.5)

| Option | Default | Ref |
|--------|---------|-----|
| Next Queue length | 6 (1–6) | §2.5.1 |
| Hold Queue | on | §2.5.2 |
| Ghost Piece | on | §2.5.3 |
| Lock Down mode | **Extended** | §2.5.4 |
| BGM | on; volume; soundtrack; random | §2.5.5 **[later]** |
| SFX | on; volume | §2.5.6 **[later]** |
| Key configuration | defaults; remappable | §4 |
| Starting Blocks | 0 rows **[later]** | §2.4.6 |
| Goal system | Fixed **[v1]** | §6 |

## Effects & animation (§13.1) — cosmetic, engine-independent

- **Line clear** effect; intensity scales with count. (§13.1.1)
- **Hard-drop trail.** (§13.1.2)
- **Lock down** texture/darken/flash; locked vs in-play. (§13.1.3)
- **Background** not interfering; cue level-up & Tetris/T-Spin/B2B. (§13.1.4)
- **Action notifications** (§13.1.5): exact names; quick; after L10 place outside
  matrix. (e.g. "T-Spin Single", "Back-to-Back T-Spin Double", "Mini T-Spin".)
- **Pause** hides matrix/next/hold; "Pause". (§13.1.6)

## Sound (§13.2, §13.3) **[later phase]**

- Min SFX: rotate CW/CCW; move L/R; land/wall; lock; line clear; game over.
- Extra: per-count clears; T-Spin variants; B2B; soft/hard; hold; move/rotation
  failure; level up. Toggle+volume. (§13.2)
- BGM default on; **Korobeiniki** for licensed release (§13.3); soundtrack select;
  (audio added via `sdl2_mixer`; `Audio` trait + `NullAudio` now).

## HUD fields & stats (§2.4.7)

Game type; lines/left; level; time; score; high; player; goal; tetrises; t-spins;
combos; TPM; LPM. 

## Rendering tech

- `Canvas<Window>`, `draw_rect(Fill)`, integer coords. No texture assets v1; font
  later (bundled free font). `Present` each frame; `clear`.
- `forbid(unsafe_code)`. Animation list in `render::anim`, fed by `EngineEvent`.
