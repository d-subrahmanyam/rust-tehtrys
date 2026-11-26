//! tehtrys binary. Phase 2: SDL2 desktop shell over the pure `core` engine.

fn main() {
    if let Err(e) = tehtrys::app::run() {
        eprintln!("tehtrys error: {e}");
        std::process::exit(1);
    }
}
