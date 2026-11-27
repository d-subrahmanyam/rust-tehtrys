# tehtrys

<!-- status badges -->
[![CI](https://github.com/d-subrahmanyam/rust-tehtrys/actions/workflows/ci.yml/badge.svg)](https://github.com/d-subrahmanyam/rust-tehtrys/actions/workflows/ci.yml)
[![tests](https://img.shields.io/badge/tests-117%20passing-brightgreen)](https://github.com/d-subrahmanyam/rust-tehtrys)
[![clippy](https://img.shields.io/badge/clippy-clean-brightgreen)](https://github.com/d-subrahmanyam/rust-tehtrys)
[![rustfmt](https://img.shields.io/badge/rustfmt-clean-brightgreen)](https://github.com/d-subrahmanyam/rust-tehtrys)
[![forbid unsafe](https://img.shields.io/badge/forbid%20unsafe-code-brightgreen)](https://github.com/d-subrahmanyam/rust-tehtrys)


A **Tetris** game in Rust (edition 2024), built to faithfully implement the official
**2009 Tetris Design Guideline** mechanics. **Phases 0-5 done**: pure deterministic core engine + SDL2 desktop shell + full
presentation + variants/persistence + sound/remapping/replay. See [`docs/plan/`](docs/plan/README.md).

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
>   are reproduced. **One separate third-party music asset**
  (`assets/audio/korobeiniki.wav`, classicals.de **Korobeiniki, CC BY-NC 4.0**; see
  [`assets/audio/CREDITS.md`](assets/audio/CREDITS.md); **not** The Tetris Company;
  **separate**; can't verify) is shipped; original protected audio
  is not. A commercial/official product would require separate licensing.

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

## Screenshots

| | |
|---|---|
| **Opening / title**<br>![tehtrys opening screen](assets/images/tehtrys-opening-screen.png) | **Main menu**<br>![tehtrys main menu](assets/images/tehtrys-main-menu.png) |
| **Gameplay**<br>![tehtrys gameplay](assets/images/tehtrys-game-play-01.png) | **Gameplay**<br>![tehtrys gameplay](assets/images/tehtrys-game-play-02.png) |

## Build & test

```sh
cargo build
cargo test          # 117 tests (101 lib + 16 integration; guideline-derived)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### Requirements

- **Rust toolchain 1.100+** (edition 2024).
- **SDL2** system library. **Not** `sdl2_mixer` and **not** `sdl2_ttf`: sound uses
  SDL2 *built-in* audio (original procedural synth); UI text uses an embedded
  8×8 font. A pure-Rust core needs no system libs; only SDL2 must be findable.

### Cross-platform: no source changes needed

`core` is pure Rust; `app`/`render`/`sound` only use the `sdl2` crate. Building on
**macOS** or **Windows** needs **no code changes** — only the **system SDL2
library + linker environment** differs. `#![forbid(unsafe_code)]`; no unsafe.

This repo ships `.cargo/config.toml` with `[env] LIBRARY_PATH` /
`LD_LIBRARY_PATH = /home/linuxbrew/.linuxbrew/lib` **for this Linux box only**;
`force = false`, so real environment variables win. On **macOS/Windows remove or
override that block** (it injects a harmless nonexistent `/home/linuxbrew/...`
path but won't find SDL2 for you).

#### Linux (Debian/Ubuntu)

```sh
sudo apt install libsdl2-dev        # Fedora: SDL2-devel; Arch: sdl2
cargo run
```

`cargo test` needs SDL2 linkable even without a display; `sdl2` crate ships
bindings. `cargo test` does not need `libsdl2-dev` at *test-runtime* but does at
link-time.

#### macOS

```sh
brew install sdl2                   # https://brew.sh
cargo run
```

Homebrew prefix `/opt/homebrew` (Apple Silicon) or `/usr/local` (Intel). If the
linker can't find SDL2:

```sh
export LIBRARY_PATH="$(brew --prefix sdl2)/lib:${LIBRARY_PATH}"
export DYLD_FALLBACK_LIBRARY_PATH="$(brew --prefix sdl2)/lib:${DYLD_FALLBACK_LIBRARY_PATH}"  # runtime
```

Remove the committed linuxbrew `[env]`. macOS runtime uses `DYLD_*`, not
`LD_LIBRARY_PATH`. App bundling not covered. `cargo test` works headless;
sound headless `SDL_AUDIODRIVER=dummy`. **Not verified on macOS in this env.**

#### Windows

1. Install **SDL2 development binaries** (e.g. `SDL2-devel-2.x-VC.zip` /
   `-mingw` from libsdl.org). No header needed (`sdl2` crate ships bindings).
2. Put `SDL2.dll` next to `target\debug\tehtrys.exe` (and `target\release\...`)
   **and** on `PATH`.
3. Point the linker at `SDL2.lib`:
   - **MSVC (`x86_64-pc-windows-msvc`):** `set LIB=C:\SDL2\lib\x64;%LIB%` and
     `set PATH=C:\SDL2\lib\x64;%PATH%`. (`LIBRARY_PATH`/`LD_LIBRARY_PATH`
     are ignored by the MSVC linker.)
   - **MinGW (`x86_64-pc-windows-gnu`):**
     `set LIBRARY_PATH=C:\SDL2-mingw\x86_64-w64-mingw32\lib;%LIBRARY_PATH%`
     + `PATH`; SDL2 mingw also ships `SDL2main`. 
4. **Remove the committed linuxbrew `[env]`** in `.cargo/config.toml` (Unix paths).
5. `cargo run`. 

No source changes. (Alternatively enable `sdl2`'s `bundled` feature to vendor
SDL2 — **not enabled** here.) `cargo test` needs SDL2 linkable; `cargo test`
itself doesn't open a window; `cargo run` needs a display. **Not verified on
Windows in this env.**

### Release packaging

- **Local release:** `cargo build --release --locked`. MSRV `rust-version = 1.100`; `edition 2024`. Release profile: `opt-level=3`, `lto="thin"`, `codegen-units=1`, `strip="symbols"`; `panic` **not** `abort` (so `cargo test --release` works). 
- **Self-contained SDL2:** `cargo build --release --locked --features bundled` → `sdl2` **`bundled` + `static-link`** vendors/statically-links SDL2 (no system SDL2 at runtime); **needs cmake + C compiler**. **Not default; not verified here.** Default remains system SDL2 per-OS. 
- **CI gate** unchanged. 
- **GitHub Release workflow:** [`.github/workflows/release.yml`](.github/workflows/release.yml) on `git tag vX.Y.Z` (+ `workflow_dispatch`); matrix `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`; `--features bundled`; `cargo fmt --check`; packages `tehtrys-<ver>-<target>.{tar.gz,zip}` + `LICENSE`+`README`; `actions/upload-artifact` + `softprops/action-gh-release`; `permissions: contents: write`. **Not run here.** 
- **Richer packaging:** **`cargo-dist`** (`cargo dist init`, generates release workflow + installers `.dmg`/`.msi`/`.tar.gz`/shell installers); `cross` for cross-compile; Docker image (`Dockerfile`); `cargo wix` (MSI); `appimagetool`. **Not shipped/verified.** 
- **No The Tetris Company protected art/audio**; own synthesized SFX; **`korobeiniki.wav` shipped separately (classicals.de CC-BY-NC-4.0; attribution; see CREDITS; can't verify; not The Tetris Company)**. **Non-commercial CC BY-NC 4.0** (original work; third-party asset separate). 

### CI extras

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) jobs: `test` matrix, `docker`, `headless-smoke`, plus **`audit`** (`cargo-audit`, RUSTSEC), **`deny`** (`cargo-deny check`; [`deny.toml`](deny.toml): advisories, licenses, bans, sources; project `license=CC-BY-NC-4.0`, checks **dependency** licenses incl. `sdl2`→zlib), **`kiri`** (Docker-compatible; `docker` on runners), **`release`** ([`release.yml`](.github/workflows/release.yml)). `RUSTFLAGS=-Dwarnings`. **Not verified on GitHub.** 

### Cross-platform testing with containers (Docker / Kiri)

**Short answer:** **yes** — and I verified it here. A reproducible `Dockerfile`
is committed; `docker build` **and** `kiri build` both succeed, `cargo test`
inside is **117/117**. 

**`kiri` is a Docker-compatible container runtime/CLI** ("docker's commands, on
Kiri": `kiri run/build/compose`, `kiri build-with`, `kiri k8s`, …). Everything
below works with **`docker`** or **`kiri`** (or `podman`); `kiri` mirrors
`docker`'s interface. 

```sh
docker build -t tehtrys .      # or: kiri build -t tehtrys .
docker run --rm -it tehtrys    # needs a display + SDL2; headless test image works
```

```sh
# without the Dockerfile, e.g. quick Linux check on rust:slim:
docker run --rm -v "$PWD:/work" -w /work -e CARGO_TARGET_DIR=/tmp/ct \
  -e LIBRARY_PATH=/usr/lib/x86_64-linux-gnu -e LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu \
  rust:slim bash -c "apt-get update -qq && apt-get install -y -qq libsdl2-dev && cargo test"
```

**Honest scope — what containers can and can't verify:**
- **Linux builds + tests**: fully verified. `Dockerfile` uses `rust:slim` +
  `libsdl2-dev`, **removes the committed linuxbrew `[env]`**, `cargo build` +
  `cargo test`. `.dockerignore` excludes `/target`, `.git`, local-only reference
  docs, `assets`.
- **Containers test LINUX only.** Docker/Kiri run Linux VMs; they **cannot run
  macOS or Windows binaries**. On a Mac/Windows host they still test a *Linux*
  target. 
- **macOS/Windows**: cross-*compiling* can be checked with **`cross`** (cargo-cross)
  (`cross build --target x86_64-pc-windows-msvc` / `aarch64-apple-darwin`, etc.;
  prebuilt Docker images) — but that **only type-checks/links**, it can't *run*
  those binaries. Real macOS/Windows verification needs **native runners** or a
  **CI matrix**. 
- **CI matrix shipped:** [`.github/workflows/ci.yml`](.github/workflows/ci.yml)
  `ubuntu-latest` + `apt libsdl2-dev`, `macos-latest` + `brew sdl2`,
  `windows-latest` + SDL2 VC (`LIB`/`PATH` + `SDL2.dll`), removes committed
  linuxbrew `[env]`, each `cargo fmt --check`, `cargo clippy --all-targets --
  -D warnings`, `cargo test --locked`; plus `docker` build job + headless
  smoke. **Not run here; verify on first push.** 
- `cargo test` needs SDL2 linkable (no display); `cargo run` needs a display;
  headless smoke `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`. 

### Portability notes

- `.cargo/config.toml` `[env]` is **machine-specific**; committed only for this
  Linux box; `force=false`.
- **Persistence** (`tehtrys_settings.txt`, `tehtrys_scores.txt`,
  `tehtrys_replay.txt`, `music/`): `~/.local/share/tehtrys`. Uses `$HOME`.
  - **macOS:** `$HOME` set → `~/.local/share/tehtrys`. 
  - **Windows:** `$HOME` often **unset** → falls back to `%TEMP%\tehtrys`; 
    `USERPROFILE`/`APPDATA` are **not** currently checked. (No code change
    *needed* to build; persistence location differs.)
- Pure core, `forbid(unsafe_code)`, no extra deps → `core` portable everywhere.
- Sound/music no licensed assets shipped; works without an audio device
  (`NullAudio` fallback; `SDL_AUDIODRIVER=dummy`). 

#### Korobeiniki music slot — format, path, **license** 

- **`MusicSlot::Korobeiniki`**; ships **`korobeiniki.wav`**. 
- **Supported format: PCM16 (little-endian) WAV**, 44.1 kHz, mono preferred
  (stereo mix-down + naive resample; own `parse_wav`; **no `sdl2_mixer`**;
  **MP3 NOT supported** — convert). 
- **Filename:** `korobeiniki.wav`. **Search order:** `<data>/music/` override
  (`~/.local/share/tehtrys/music/korobeiniki.wav`; Windows `%TEMP%` fallback)
  **then bundled `assets/audio/korobeiniki.wav`.** 
- **Shipped asset:** `assets/audio/korobeiniki.wav` = **"Tetris Theme - Korobeiniki
  - Arranged for Music Box"** (classicals.de
  `Classicalsde-Tetris-Theme-Korobeiniki-Rearranged-Arranged-for-Music-Box-xcmx.mp3`)
  converted `ffmpeg … -ac 1 -ar 44100 -c:a pcm_s16le`. **`parse_wav` verified.**
  Original MP3 **not** committed. **Convert:** `ffmpeg -i in.mp3 -ac 1 -ar 44100
  -c:a pcm_s16le korobeiniki.wav`. 
- **Music toggle/volume:** Options → MUSIC / MUSIC VOLUME; `music_enabled`.
  `SDL_AUDIODRIVER=dummy` safe. 
- **Source + attribution (CC BY-NC 4.0 requires credit + link + changes note):**
  **`https://www.classicals.de/tetris-theme`** (website `https://www.classicals.de`;
  `https://www.gregorquendel.com`). **"Tetris Theme - Korobeiniki - Arranged for
  Music Box"**; **recorded, produced, published by Gregor Quendel**; **arrangement
  based on notes by Maru Piano**. **License terms (Standard - Non-Commercial):
  `CC BY-NC 4.0`.** Full attribution in
  [`assets/audio/CREDITS.md`](assets/audio/CREDITS.md). **Changes made:** MP3→PCM16
  mono 44.1k WAV. 
- **License reconciliation:** `korobeiniki.wav` **third-party, NOT original
  `tehtrys`, NOT d-subrahmanyam, NOT The Tetris Company, NOT covered by project
  CC-BY-NC**; keeps own **CC BY-NC 4.0**; **same license family as project**;
  **non-commercial**; no third-party trademark; `CC` doesn't grant trademark;
  **"Tetris"®**; **not affiliated**; not original protected audio. 
- ⚠️ **Honest:** user-confirmed + stated CC-BY-NC; can't verify full text +
  **embedded ID3 `© 2024 Gregor Quendel. All rights reserved.`** + `CC-BY-NC`
  conflict + The Tetris Company + Korobeiniki + third-party; can't verify
  **audible** output headless; **not legal advice**; check before shipping. 
- **Sound effects** are **original procedural synthesis**, no asset. 


## Docs

- [docs/plan/README.md](docs/plan/README.md) — complete plan, phases, flows, appendices.
- Source rules: the **2009 Tetris Design Guideline** (© Blue Planet Software;
  CONFIDENTIAL; "Tetris®"). **Not distributed in this repo**; kept **local-only**
  and git-ignored (`docs/2009 Tetris Design Guideline.pdf`, plus optional local
  restatement). Mechanics are restated in my own `docs/plan/`.

## Releases

**Prebuilt releases on GitHub:** [`d-subrahmanyam/rust-tehtrys` Releases](https://github.com/d-subrahmanyam/rust-tehtrys/releases). **`v1.0.1`** (installer-quality fixes; supersedes `v1.0.0`/`v0.1.0`). 

**Downloads per tag** (`v*`):
- **Archives**: `tehtrys-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz`, `tehtrys-X.Y.Z-x86_64-unknown-linux-musl.tar.gz` (**static, no glibc**), `tehtrys-X.Y.Z-aarch64-apple-darwin.tar.gz`, `tehtrys-X.Y.Z-x86_64-pc-windows-msvc.zip`; each + `LICENSE`+`README`+`CHANGELOG`+`korobeiniki.wav`+`CREDITS`. 
- **Installers**: **macOS `.dmg`**, **Windows `.msi`** (`cargo-wix`), **Ubuntu `.dpkg`** (`cargo-deb`; flatpak alt), built by `release.yml`. 
- **GitHub auto**: `Source code (zip)` + `Source code (tar.gz)`. 

**Build from source instead** (anywhere; no prebuilt needed): see [Build & test](#build--test). 
```sh
git clone https://github.com/d-subrahmanyam/rust-tehtrys && cd rust-tehtrys
cargo build --release --locked            # + SDL2; or `--features bundled` (cmake); `x86_64-unknown-linux-musl` static
cargo run
```
**Cutting a release:** [`docs/RELEASE.md`](docs/RELEASE.md) + `.github/workflows/release.yml`. **Not verified on GitHub; `v1.0.0`.** **Non-commercial CC-BY-NC-4.0**; `korobeiniki.wav` third-party CC-BY-NC; not legal advice. 

## Status

**Badge legend:** 🟢 **passing** (`cargo fmt` + `clippy -D warnings` + `cargo test` green; **117 tests**) · 🟡 **running / pending** (CI in progress, amber in the Actions tab; not a failure) · 🔴 **failing** · ⚪ **no status** (no runs yet). **`CI`** is a **live** GitHub Actions badge (`actions/workflows/ci.yml/badge.svg`); `tests`/`clippy`/`rustfmt` are static `shields.io` badges reflecting the current committed state. `tests` count = `cargo test` total (101 lib + 16 integration).


Implementation progress by phase. Commit ids link to this repo's `master`. See
[docs/plan/07-phases.md](docs/plan/07-phases.md) for full detail.

| Phase | Scope | State | Commit |
|-------|-------|-------|--------|
| **0 — Scaffolding** | Cargo layout; `#![forbid(unsafe_code)]`; fmt/clippy/test wired; `sdl2` kept; `cgmath` removed; `core` zero external deps | done | [`6f0c2d5`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/6f0c2d5) |
| **1 — Pure core engine** | 10×20+20 buffer; 7 pieces; North; SRS + kicks + T-spin/Point-5; 7-bag + seedable PRNG; Lock-Down (Extended/Infinite/Classic; 0.5 s); scoring + B2B; levels 1-15 + Fixed/Variable goals; game-over (LockOut/BlockOut/TopOut seam); 8 phases; Marathon/Sprint/Ultra shells; 86 tests | done | [`6f0c2d5`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/6f0c2d5) |
| **2 — SDL2 shell + input** | SDL2 window; fixed 60 Hz accumulator + dt clamp; keymap §4.1; own DAS/ARR; edge vs level; ignore SDL repeat; pause; focus auto-pause; restart; minimal render; event hooks | done | [`fb8571b`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/fb8571b) |
| **3 — Full presentation** | §2.4.7 HUD; Next 1-6; Hold; Ghost; colors; action notifications §13.1.5; line-clear + lock flash + hard-drop trail + level-up; Pause §13.1.6; Game Over; scaling; monochrome §3.2 (`M`); embedded 8×8 font (no `sdl2_ttf`); menu shells | done | [`40f4d41`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/40f4d41) |
| **4 — Variants, progression, persistence** | app state machine §2 + menus; Marathon/Sprint-40/Ultra-2:00; start level 1-15; Lock-Down mode §2.5.4; Next-length §2.5.1; Hold/Ghost toggles §2.5; Fixed/Variable goals §6; **high-score table** §11.2 + **settings persistence** (custom text, no serde); Game Over; end-rewards seam; 98 tests | done | [`7bcd4bb`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/7bcd4bb) |
| **5 — Polish** | **sound effects** §13.2 (`Audio` trait; procedural synth; no licensed assets; volume/toggle; mute); **music slot Korobeiniki §13.3** (shipped converted `korobeiniki.wav`; **separate classicals.de CC-BY-NC-4.0** + attribution; can't verify; not The Tetris Company; `<data>/music` override; PCM16 only); **key remapping** §4.1 UI + persisted; settings robustness + atomic write; **seeded randomness** + **replay/determinism**; **high-score name entry** §11.2; **help + scoring demo + credits** §2.6/§8; **demo**; robustness; 117 tests | done | [`f087754`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/f087754) |
| **6 — Multiplayer seams** *(optional, not v1, **document-only**; **not built**)* | guideline Appendix B restated (line attack, counter attack, receiving queue, broken/solid, 8-line gap, Top Out §10.5, Extended §5.7, shared 7-bag order, practice, UI) + seams already in pure `core`; no multiplayer gameplay/networking | done (docs) | [`699423f`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/699423f) |

### Cross-cutting status

| Item | Status | Commit |
|------|--------|--------|
| fmt/clippy(`-D warnings`)/test clean | ✅ (117 tests) | [`f087754`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/f087754) |
| `#![forbid(unsafe_code)]` | ✅ | [`6f0c2d5`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/6f0c2d5) |
| `core` pure / no rendering / no OS / no wall-clock / no external deps | ✅ | [`6f0c2d5`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/6f0c2d5) |
| Headless `cargo run` (dummy video/audio drivers) | ✅ smoke | [`f087754`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/f087754) |
| Reference source not distributed (git-ignored, not in history) | ✅ | [`a9b766e`](https://github.com/d-subrahmanyam/rust-tehtrys/commit/a9b766e) |

### Honest notes

- **Not pixel-verified** (headless env); HUD/menus use embedded font; layout not screenshot-checked.
- **Audio not verified audibly** in headless env; synth + event→cue mapping + WAV parsing are unit-tested; own sound effects are synthesized; `Korobeiniki` slot ships converted
  `assets/audio/korobeiniki.wav` (**separate classicals.de CC-BY-NC-4.0**; attribution; can't verify; see CREDITS)
  + `<data>/music` override; **not The Tetris Company**; can't verify audible.
- Phase-5 plan named `sdl2_mixer`, but that **system library isn't available here**; implemented `Audio` trait + procedural synth on **SDL2 built-in audio** (no extra dependency). 
- High-score name entry, sound volumes, key remapping, seed, replay are **persisted** under `~/.local/share/tehtrys`. 
- `restart` reuses current seed (deterministic); random-seed mode + replay supported.
- **Not legal advice.**
