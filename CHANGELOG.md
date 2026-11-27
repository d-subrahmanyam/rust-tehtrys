# Changelog

All notable changes. **Non-commercial educational.** Project original work **CC-BY-NC-4.0**; **third-party `korobeiniki.wav`** separate **CC-BY-NC-4.0** (classicals.de; see `assets/audio/CREDITS.md`; can't verify; not legal advice). 

## [Unreleased]

- (next) 

## [1.0.1] — installer-quality fixes

- robust `korobeiniki.wav` discovery (data/music; cwd; `<exe>`; `<exe>/music`; `.app` `Contents/Resources`; `/usr/share/tehtrys[/music]`; `.msi` `assets/audio`; flatpak `/app/share`); `.msi` `korobeiniki.wav`→`assets/audio`.

## [1.0.0] — first stable release

*(tehtrys v1.0.0; built on v0.1.0.)*

## [0.1.0] — first release

**Tetris, faithful 2009 Guideline mechanics, non-commercial educational.**

### Engine (`core`, pure, `forbid(unsafe_code)`, no external deps, no wall-clock, integer-ms, seedable)
- 10×20 + 20 buffer; 7 pieces; North; spawn; drop; **SRS** 5-kick + T exceptions + Point-5; **7-bag**; Next 1-6; **Hold** once/lock (toggle); **Ghost**. 
- **Lock Down** 0.5 s; **Extended default**/Infinite/Classic; 15-cap; reset; hard-drop instant. 
- **Scoring** §8 + level; soft+1/hard+2; **T-Spin/Mini**; **Back-to-Back**; **no combo scoring**; §8.1=5400. 
- **Levels 1-15**; Fixed/Variable goals; fall-speed; start level. **8 phases.** 
- **Game over** LockOut/BlockOut/TopOut(seam). 
- **Variants** Marathon(default)/Sprint-40/Ultra-2min. 

### App (SDL2, `app`/`render`/`input`/`sound`/`persist`/`replay`)
- fixed 60 Hz; keymap §4.1; **DAS/ARR**; pause; focus auto-pause; restart; mono. 
- **Phase 3** HUD §2.4.7/Next/Hold/Ghost/notifications §13.1.5/flashes/hard-drop trail/pause §13.1.6/GameOver; embedded 8×8 font; **mirror-text fixed (LSB-left)**. 
- **Phase 4** state machine + menus; persistence; **high scores** §11.2 + name entry; variant/lock/next-length/hold/ghost/mono/goal. 
- **Phase 5** sound §13.2 (`Audio`; **procedural synth**; no `sdl2_mixer`; volume/toggle/mute); **Korobeiniki** §13.3 `MusicSlot` **`korobeiniki.wav`** (classicals.de CC-BY-NC; converted PCM16; separate; can't verify; not The Tetris Company); **key remapping** §4.1 persisted; **seeded randomness + replay**; help/demo; robustness. 
- **Phase 6** multiplayer seams **document-only**. 

### Project
- **117 tests**; `fmt`/`clippy -D warnings`; `#![forbid(unsafe_code)]`. 
- **Cross-platform** Linux/macOS/Windows; **Docker/Kiri**; **`Dockerfile`**; **GitHub Actions CI** matrix + `docker` + smoke + `audit` + `deny`; **`cargo-deny` `CC-BY-NC-4.0`**. 
- **Release** `.github/workflows/release.yml` tag `v*`; `bundled`; package; see `docs/RELEASE.md`. 
- **CC BY-NC 4.0** + disclaimer; source guideline not distributed; no The-Tetris-Company protected assets; `korobeiniki.wav` third-party CC-BY-NC. 

[Unreleased]: https://github.com/d-subrahmanyam/rust-tehtrys/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/d-subrahmanyam/rust-tehtrys/releases/tag/v1.0.1
[1.0.0]: https://github.com/d-subrahmanyam/rust-tehtrys/releases/tag/v1.0.0
[0.1.0]: https://github.com/d-subrahmanyam/rust-tehtrys/releases/tag/v0.1.0
