# tehtrys — Tetris in Rust: Project Plan

This folder contains the complete plan for building **tehtrys**, a Tetris game in Rust,
and documentation of all major game/system **flows**.

## Source of truth

All gameplay rules are derived from the official design document, **used here purely
as a reference**:

- ***Tetris® Design Guideline*, March 2009**, © 2009 Blue Planet Software, Inc.
  (96 pages; marked **CONFIDENTIAL**; "Tetris®" trademarked; intended for licensed
  products). We **used it for reference** to extract mechanics; it is **not included
  or redistributed in this repo**. If you want it locally, keep your own copy — it
  is git-ignored at `docs/2009 Tetris Design Guideline.pdf` (optionally a local
  text-only restatement at `docs/tetris-design-guideline.md`, also git-ignored);
  nothing here assumes either is present.

Section references like *(§5.7)* or *(A1.4.2)* point into that guideline. My notes,
plan, and appendices are **original writing** (factual rules restated); no protected
artwork/audio; no rights to the source document or trademark are granted (see
[LICENSE](../../LICENSE)). Where this plan adds implementation choices not fixed by
the guideline, that is called out explicitly.

> **Note on reading the PDF.** Text was extracted for analysis. The guideline's
> diagrams (tetrimino shapes, T-Spin examples, game-over diagrams, **Super Rotation
> System kick diagrams**, multiplayer layouts) are vector art; this model cannot view
> images. A **local-only** text-only conversion of the source (not distributed;
> diagrams→tables/ASCII) confirms spawn cells (§5.2),
> orientation tables (§3.2), SRS kick tables (§15.4 incl. **T-piece exceptions**),
> T-Spin corner rules (§11), and B2B (§10). Our rotation kick tables reproduce those
> and are **tested**; conformance is verified in [testing](08-testing.md). Nothing
> asserts having read protected/pixel art; no protected assets are used.

> **Trademark / licensing note.** "Tetris®" is a registered trademark and the
> guideline is confidential, intended for officially licensed products. This project
> (binary crate `tehtrys`, no commits yet) appears to be a **learning exercise**;
> the plan implements the *gameplay mechanics* faithfully but scopes out
> trademark/marketing requirements (official logo, legal screen wording, Korobeiniki
> licensing) as "only needed for a licensed release" — see
> [requirements](01-requirements.md#non-functional-requirements) and
> [variants](09-variants-and-modes.md#scope-decisions). We do not reproduce any
> protected artwork or audio assets.

## What is being built

A single-player desktop Tetris game in Rust:

- **Engine:** pure, deterministic, headless-testable core implementing the guideline's
  8-phase Tetris Engine (Appendix A1), **Super Rotation System**, **7-bag** random
  generation, three **Lock Down** rulesets (Extended default), full **scoring**
  (line clears, T-Spins/Mini T-Spins, Back-to-Back), levels & goal systems,
  all three game-over conditions, and the three main variants (**Marathon**,
  **Sprint/40 Lines**, **Ultra**).
- **Presentation:** **SDL2** rendering (crate `sdl2` already in `Cargo.toml`),
  keyboard controls with **DAS/ARR**, Next + Hold queues, **Ghost Piece**, on-screen
  action notifications, pause, scoring/stats HUD.
- **Polish (later phase):** sound effects, background music, high-score table,
  settings persistence, replay/determinism.

## Document index

### Planning

| Doc | Contents |
|-----|----------|
| [00-overview.md](00-overview.md) | Vision, scope, non-goals, key decisions, definition of done |
| [01-requirements.md](01-requirements.md) | Functional requirements mapped to guideline sections; non-functional requirements |
| [02-architecture.md](02-architecture.md) | Crate/module layout, core types, engine/presentation split, event model, determinism |
| [03-game-engine.md](03-game-engine.md) | The 8-phase engine; matrix & buffer zone; spawn; 7-bag; SRS; lock down; line clear; game over |
| [04-controls.md](04-controls.md) | Key bindings; movement; DAS/ARR; rotation; drop; hold; pause; remapping |
| [05-scoring-and-progression.md](05-scoring-and-progression.md) | Scoring tables; Back-to-Back; T-Spin rules; levels; goals; fall speeds; starting level |
| [06-rendering-and-ui.md](06-rendering-and-ui.md) | Screen layout; HUD; Next/Hold; Ghost; colors; notifications; pause; animation |
| [07-phases.md](07-phases.md) | **Implementation phases** (milestones) with deliverables & acceptance criteria |
| [08-testing.md](08-testing.md) | Testing strategy; guideline-derived test vectors; determinism; CI |
| [09-variants-and-modes.md](09-variants-and-modes.md) | Marathon/Sprint/Ultra; lock-down modes; options; multiplayer scope note |

### Flows — [flows/README.md](flows/README.md)

Game flow, engine phases, game loop, input, piece lifecycle, lock down, line clear &
scoring, T-Spin detection, hold & next, level progression, game over, pause, rendering,
and the application state machine.

### Appendices

| Doc | Contents |
|-----|----------|
| [appendices/A-srs-kick-tables.md](appendices/A-srs-kick-tables.md) | SRS facings, kick test order, full kick tables (O/I/3×3), coordinate conventions, T-Spin recognition reference |
| [appendices/B-piece-data.md](appendices/B-piece-data.md) | Explicit per-orientation cell data for all 7 tetriminos; colors; spawn placement |
| [appendices/C-fall-speed-table.md](appendices/C-fall-speed-table.md) | Fall speed formula + level table; soft/hard drop; DAS/ARR; timing constants |
| [appendices/D-glossary.md](appendices/D-glossary.md) | Tetris terminology used by this plan (from guideline §1.4/§14) |

## Implementation status

- [x] Source guideline ingested; rules extracted with section references. Source
      PDF + any converted restatement are **local-only, not distributed**
      (git-ignored); see trademark/source note above.
- [x] Plan & flow documentation (this folder).
- [x] **Phase 0** scaffolding; **Phase 1** pure `core` engine + 86 tests
      (fmt/clippy/test clean); `#![forbid(unsafe_code)]`.
- [x] **Phase 2** SDL2 shell + fixed-timestep + input/DAS + render (pause/restart).
- [x] **Phase 3** full presentation (HUD/Next/Hold/Ghost/notifications/flashes/
      hard-drop trail/pause/game-over/monochrome); embedded font; see [07](07-phases.md).
- [x] **Phase 4** app state machine + menus; variants (Marathon/Sprint/Ultra);
      start level; lock-down mode; next-length/hold/ghost/mono; goal system;
      **high-score table + settings persistence** (custom text; no serde).
- [ ] **Phase 5** sound/music + remapping UI + replay.

## Recommended reading order

1. [00-overview.md](00-overview.md)
2. [03-game-engine.md](03-game-engine.md)
3. [flows/02-engine-phases.md](flows/02-engine-phases.md)
4. [05-scoring-and-progression.md](05-scoring-and-progression.md)
5. [07-phases.md](07-phases.md)
