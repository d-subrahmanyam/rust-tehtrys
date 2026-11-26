# 07 — Implementation Phases

Ordered milestones. Each phase has **goal**, **deliverables**, **acceptance criteria**
(done = code + tests + docs updated + `cargo fmt/clippy/test` clean). Phases are
sized to be independently reviewable. No phase starts until the previous is green.
(The user asked: plan first; implementation starts after this plan is approved.)

```
Phase 0  Scaffolding & conventions
Phase 1  Pure core engine  ▶ rules-correct, headless, fully tested
Phase 2  SDL2 shell + game loop + input
Phase 3  Full presentation (HUD, next/hold, ghost, notifications, pause, animation)
Phase 4  Variants, levels/goals, endings, high scores
Phase 5  Polish: sound, music, settings, help, replay/determinism
Phase 6  (optional) multiplayer seams → Appendix B  [not v1]
```

---

## Phase 0 — Scaffolding & conventions  ✅ **done**

**Goal.** Project hygiene; conventions; no behavior yet.

- Confirm Cargo layout; `#![forbid(unsafe_code)]`; `clippy`, `fmt`, `test` wired.
- Keep `sdl2`; decide on removing `cgmath` (keep minimal); no other deps for `core`.
- Create `src/core/…` module skeleton; `app`, `render`, `input`, `persist` skeletons.
- CI script (`cargo fmt --check`, `clippy -D warnings`, `test`).
- `docs` kept in sync; ADR-style notes optional.
- **Sanity:** `cargo build`, `cargo test` green; baseline `Hello world` SDL window opens
  (later replaced) OR headless core test harness.

**Acceptance.** Clean tree; toolchain 1.100; `forbid(unsafe_code)`; conventions doc;
all commands green.

---

## Phase 1 — Pure core engine (headless, rules-correct)  ✅ **done**

**Goal.** Complete, deterministic `tetris` logic, no rendering. The heart.
*(Shipped: 86 tests; fmt/clippy/test clean.)*

Deliverables (modules in `core`):

1. **types** + constants (W10/V20/B20; `Tetrimino`; `Facing`; `Rotation`; `Point`;
   `Cell`; ms timing consts).
2. **matrix** (occupancy; `is_free`; `lock` cells; `find_full_rows`; `clear_rows`;
   collapse; render snapshot; buffer handling; game-over predicates).
3. **piece** + **orientation tables** (appendix B) + **spawn** (§3.4/A1.2.1) +
   drop-one-row + colors.
4. **rotation / SRS** (appendix A; kick tables; `try_rotate`; Point-5 flag;
   last-move tracking; O no-op; I/3×3; **guideline→engine coordinate conversion**).
5. **bag / RNG** (internal PRNG; **7-bag**; `NextQueue`; seedable; reproducible).
6. **lockdown** (three rulesets; 0.5 s; surface tracking; reset semantics; **15-cap**
   Extended; hard-drop instant; Classic/Infinite; A1.2.3 lift-off note).
7. **scoring** (table; level mult; B2B; drop pts; combos-stat; §8.1 test; integer).
8. **tspin** (§9.1 recognition; corners; A/B/C/D; Point-5; last-was-rotation; mini vs
   full; no-line nuances §8n1).
9. **engine** (**8 phases**; generation 0.2 s; hold; events; `tick(dt_ms)`; commands;
   **game-over** Lock Out/Block Out/Top-Out-seam; resting/lock FSM).
10. **levels/goals** (1–15; Fixed/Variable; level-up; start level; fall speed §7;
    appendix C table).
11. **game** (Marathon/Sprint/Ultra shells; ending; stats; time).

**Tests (see 08):** every rule above; guideline test vectors; §8.1=5400; SRS kicks vs
tables; spawn; lock timing w/ `ManualClock`; B2B; T-Spin; game-over; 7-bag; determinism.

**Acceptance.** `cargo test` headless green; no SDL in `core`; a tiny headless binary
can play scripted inputs & print events. **No rendering yet.**

---

## Phase 2 — SDL2 shell + game loop + input

**Goal.** Playable window; fixed timestep; input; minimal render.

- SDL2 init; window; canvas; `forbid(unsafe)`.
- **Fixed-timestep** accumulator (60 Hz); `dt` clamp; render interpolation optional.
- **Keymap** (§4.1; our defaults; remappable later); edge vs level; ignore SDL repeat.
- **DAS/ARR** FSM (§5.2); soft drop; hold; pause; focus handling.
- Minimal render: filled cells; active + ghost; clear color; matrix border.
- App state machine skeleton; Esc pause; R restart; Q quit. **Marathon** only for now.
- **Events** from core → drive (no-op) audio/render hooks.

**Acceptance.** `cargo run` opens window; you can play Marathon; pause; restart; keys
match §4.1; core still 100% headless-testable; no logic in `render`.

---

## Phase 3 — Full presentation

**Goal.** Faithful look & feel per §2/§13.

- **HUD** all fields (§2.4.7); **Next 1–6** (§2.4.3/§2.5.1); **Hold** (§2.4.8);
  **Ghost** (§2.4.4); colors (§3.1); skyline/buffer hint.
- **Action notifications** (§13.1.5); placement rule L10; names.
- **Animation**: line-clear (intensity), lock flash/darken (§13.1.3), hard-drop trail
  (§13.1.2), level-up cue; **Animate Phase** wired (§A1.2.6).
- **Pause** hides matrix/next/hold + "Pause" (§13.1.6).
- **Layout scaling**; monochrome patterns (§3.2); not relying on color.
- **Menus**: Title, Main Menu, Options, Help (§2.3/§2.5/§2.6); Game Over screen.

**Acceptance.** Looks/feels faithful; notifications correct; pause hides playfield;
menus navigate; cosmetic only (logic unchanged; tests still green).

---

## Phase 4 — Variants, progression, persistence

**Goal.** Full ruleset & retention.

- **Variants**: Marathon (default), **Sprint/40 Lines**, **Ultra** (2:00), (§12);
  endings; no level-up for Sprint/Ultra; start-level for Sprint (§12.2).
- **Levels/goals**: Fixed + Variable (§6); 1–15; start level; fall-speed (§7);
  appendix C; scoring level multiplier.
- **Lock-down mode** option (§2.5.4); Next length (§2.5.1); Hold/Ghost toggles (§2.5).
- **Starting Blocks** (§2.4.6) **[if time]**.
- **High Score Table** (§11.2/A1.2.7): per variant; name; level/score/lines; persisted
  JSON; menu access. **End rewards** (§11.1) **[if time]**.
- **Game Over** full screen; stats; replay-this-seed **[if time]**.

**Acceptance.** All three variants correct; progression correct; scores persist;
guideline goal examples verified (§6 L4→5=40 etc.); tests green.

---

## Phase 5 — Polish

- **Sound effects** (§13.2 min + extra); `sdl2_mixer`; `Audio` trait; toggle/volume.
- **BGM**: Korobeiniki slot for licensed asset; toggle/volume; **[not shipping
  protected audio]**; random/other **[if licensed]** (§13.3).
- **Settings** persistence (keys, options, volumes); **remapping UI**.
- **Help/demo** incl. scoring demo (§8 note); correct terminology (§2.6); credits.
- **Replay/determinism** (seed + input log); **record/export**.
- **Robustness**: focus, resize, frame pacing, no panics; `cargo clippy`.
- **Docs**: sync flows; README; controls; build instructions; licensing note.

**Acceptance.** Feature-complete vs requirements checklist; tests + manual pass.

---

## Phase 6 — Multiplayer seams (optional, not v1)

Document-only for v1; design keeps seams. Guideline **Appendix B**:

- Line Attack tables (§B1.3.5); Counter Attacks (§B1.3.6); Receiving Queue;
  broken/solid lines; gap randomness (8-line); Top Out (§10.5); Extended required
  (§5.7); shared tetrimino order (§B1.2); practice mode; UI (§B1.4). **Not built.**

---

## Cross-cutting

- **Testing** every phase ([08-testing.md](08-testing.md)); guideline-derived vectors.
- **Docs** updated per phase; flows stay accurate.
- **Commits** small; `fmt/clippy/test` before commit.
- **No shortcuts**: tests prove behavior; we don't mark done without green tests.

## Phase → guideline coverage

| Phase | Covers |
|-------|--------|
| 1 | §1–§12 core, §A1, scoring, SRS, lock, game-over, 7-bag, spawn, hold |
| 2 | §4/§4.1, §5.1–5.5, §5.2 DAS, §13.1.6 pause (basic) |
| 3 | §2.4, §2.5, §3.1/3.2, §13.1, notifications |
| 4 | §2.3, §6, §7, §11, §12, §2.4.6, §2.5.4 |
| 5 | §13.2, §13.3, §4 remap, §8 demo, §11.1 |
| 6 | Appendix B |

## Rough ordering & rationale

Core-first (Phase 1) so all rules are proven before any rendering; SDL shell second;
presentation third; variants/persistence fourth; polish last. Mirrors guideline's own
"Tetris Engine" (A1) being independent of presentation/sound (§13). 
