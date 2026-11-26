# tehtrys

A **Tetris** game in Rust (edition 2024), built to faithfully implement the official
**2009 Tetris Design Guideline** mechanics. **Phase 0 + Phase 1 done**: pure, deterministic, fully-tested core engine;
**Phase 2 + Phase 3 done**: SDL2 desktop shell + full presentation. See [`docs/plan/`](docs/plan/README.md).

> ## ⚠️ Disclaimer
>
> This work is provided **purely for educational and study purposes** — to learn
> Rust, game design, and the published Tetris gameplay rules — and is **never for
> commercial use**. See [LICENSE](LICENSE).
>
> - Not affiliated with, endorsed by, or licensed by **The Tetris Company** /
>   Tetris Holding, LLC. "Tetris®" is a registered trademark; no trademark or
>   protected-asset rights are claimed or granted.
> - Gameplay mechanics are implemented from the published rules in the included
>   **2009 Tetris Design Guideline** PDF; no proprietary artwork, audio, or marks
>   are reproduced. A commercial/official product would require separate licensing.

## License

**Free to use for non-commercial + educational purposes** under
**[CC BY-NC 4.0](LICENSE)** (attribution; **no commercial use**). See [LICENSE](LICENSE).

## What this is

- **`core`** (`src/core/`): pure, deterministic, headless-testable Tetris engine —
  no rendering, OS, wall-clock, or external-crate dependencies.
  - 10×20 matrix + 20-row buffer zone; 7 tetriminos; North-facing spawn;
  - **Super Rotation System** (5 kicks, T exceptions, O/I/3×3);
  - **7-bag** randomization + Next queue; **Hold** (once/lock); **Ghost** data;
  - **Lock Down**: hard-drop instant + 0.5 s; **Extended** (default)/**Infinite**/
    **Classic**; A1.2.3 surface tracking;
  - **scoring**: line clears, **T-Spin/Mini**, **Back-to-Back**, soft/hard drop;
  - **levels 1–15** + Fixed/Variable goals + fall-speed curve;
  - **game over**: Lock Out / Block Out / Top Out (seam);
  - **8 engine phases** (A1.2); **variants**: Marathon / Sprint / Ultra;
  - deterministic (seedable PRNG; integer ms; injectable time).
- `#![forbid(unsafe_code)]`.

## Build & test

```sh
cargo build
cargo test          # 70 unit + 16 integration = 86 tests (guideline-derived)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Requires a Rust toolchain (1.100+) and **SDL2** system library. On Debian/Ubuntu:
`sudo apt install libsdl2-dev` (provides `libSDL2.so`). This repo also ships
`.cargo/config.toml` `[env]` pointing at a linuxbrew SDL2; override `LIBRARY_PATH`
if needed. Run with `cargo run`.

## Docs

- [docs/plan/README.md](docs/plan/README.md) — complete plan, phases, flows, appendices.
- Source rules: the **2009 Tetris Design Guideline** (© Blue Planet Software;
  CONFIDENTIAL; "Tetris®"). **Not distributed in this repo**; kept **local-only**
  and git-ignored (`docs/2009 Tetris Design Guideline.pdf`, plus optional local
  restatement). Mechanics are restated in my own `docs/plan/`.

## Status

**Phase 0 + Phase 1 complete** (scaffolding + pure engine; fmt/clippy/test clean;
`#![forbid(unsafe_code)]`). **Phase 2 + Phase 3 complete**: SDL2 shell + fixed-timestep + keymap/DAS + full
presentation (HUD/Next/Hold/Ghost/notifications/flashes/pause/game-over/mono);
**95 tests** (79 + 16). **Phase 4 complete**; Phase 5 next. See
[docs/plan/07-phases.md](docs/plan/07-phases.md).
