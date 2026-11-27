# Release process — `tehtrys`

**Not verified on GitHub** (can't run Actions here). **Not legal advice.** 

## Version & changelog
- **Semver**; bump `Cargo.toml` `version`. Current `1.0.2`. **MSRV `rust-version="1.100"`.**
- `cargo check` to sync `Cargo.lock` (committed). 
- Add `CHANGELOG.md` entry under new version. 
- **tag `vX.Y.Z` or plain `X.Y.Z`** matching `Cargo.toml`. **first stable `v1.0.0`; `v1.0.1` installer-quality fixes; `1.0.2` Windows packaging fixes.**

## Gate (CI + local)
```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked                 # 117
cargo test --locked --features bundled   # release also tests bundled SDL2 (needs cmake)
cargo deny check                    # deny.toml (incl CC-BY-NC-4.0)
cargo audit
```
**RUSTFLAGS=-Dwarnings.** 

## Tag + push (GitHub token needs `workflow` scope + `contents: write`)
```sh
git add -A && git commit -m "release vX.Y.Z"
git tag -a vX.Y.Z -m "tehtrys vX.Y.Z"   # e.g. v1.0.0
git push origin master
git push origin vX.Y.Z              # triggers .github/workflows/release.yml
```
**Hit invalid token?** `unset GITHUB_TOKEN` / `env -u GITHUB_TOKEN`, or `gh auth refresh -s workflow`, `gh auth setup-git`. 

## `release.yml`
- `on push tag v*` + plain `1.0.2` + `workflow_dispatch`; `contents: write`.
- matrix linux/macos/windows; `rm .cargo/config.toml`; `lukka/get-cmake`. 
- `cargo fmt --check`; `cargo test --locked --features bundled`; `cargo build --release --locked --target … --features bundled`. 
- package `tehtrys-<ver>-<target>.{tar.gz,zip}` + `LICENSE`+`README`+`CHANGELOG`+`korobeiniki.wav`+`CREDITS`; `upload-artifact`+`gh-release`. 
- Windows `.zip` artifacts are real ZIP files (`Compress-Archive`, verified non-empty); do not rename a `tar` archive to `.zip`.
- Windows `.msi` (`cargo-wix`, `wix/main.wxs`) installs `tehtrys.exe`, `korobeiniki.wav`, credits, license, and a `tehtrys` **Start Menu** shortcut; installers are not code-signed, so Windows SmartScreen/Defender may warn.
- **`--features bundled`** SDL2 self-contained (not verified); else system SDL2 + ship shared lib. 

## Artifacts / contents
- native `tehtrys`/`tehtrys.exe`. 
- **`korobeiniki.wav`** (PCM16; third-party CC-BY-NC classicals.de; `CREDITS.md` + attribution; non-commercial; can't verify). 

## Not shipped / note
- no The-Tetris-Company protected art/audio; **not affiliated**; own SFX synth; **non-commercial**. 
- `cargo-dist`, app stores, code signing/notarization **not** shipped. `.dmg`/`.msi`/`.dpkg` are built by `release.yml`.
