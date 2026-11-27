//! Replay / determinism. (Phase 5; plan Phase-5 "replay/determinism: seed + input
//! log; record/export"; NFR-1)
//!
//! `core` is pure & deterministic: same `seed` + same ordered `GameCommand`s at
//! same `tick` => identical `Game`. [`Replay`] records that input log; [`Replay::run`]
//! re-simulates headlessly; **no wall-clock** in core. Custom text format (no serde).
//!
//! Honest scope: records commands we feed the engine + total ticks; DAS/ARR and
//! app-level pause are app concerns — replay re-feeds exactly what reached `core`,
//! which is what determinism requires.

use crate::core::game::{Game, Variant};
use crate::core::types::{GameCommand, LockRuleset};

/// REPLAY text format version. (robustness; backwards-compatible)
pub const REPLAY_VERSION: u32 = 1;
pub const REPLAY_FILE: &str = "tehtrys_replay.txt";

/// One recorded command at a tick boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReplayCommand {
    pub tick: u64,
    pub cmd: GameCommand,
}

/// A deterministic replayable session. (§3.3 seed + ordered input)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Replay {
    pub seed: u64,
    pub variant: Variant,
    pub start_level: u32,
    pub ruleset: LockRuleset,
    pub commands: Vec<ReplayCommand>,
    /// total ticks advanced. (`tick(TICK_MS)` count)
    pub ticks: u64,
}

impl Replay {
    pub fn new(seed: u64, variant: Variant, start_level: u32, ruleset: LockRuleset) -> Self {
        Self {
            seed,
            variant,
            start_level,
            ruleset,
            commands: Vec::new(),
            ticks: 0,
        }
    }

    /// Record a command applied at current `tick`.
    pub fn record(&mut self, cmd: GameCommand) {
        self.commands.push(ReplayCommand {
            tick: self.ticks,
            cmd,
        });
    }

    /// Advance one tick (caller ticks the real game in parallel).
    pub fn advance_tick(&mut self) {
        self.ticks += 1;
    }

    /// Re-simulate deterministically; returns a fresh `Game`. (pure)
    /// `dt_ms` per tick.
    pub fn run(&self, dt_ms: u32) -> Game {
        let mut g = Game::new(self.seed, self.variant, self.start_level, self.ruleset);
        let mut idx = 0;
        // commands sorted by tick; recorded in order.
        for t in 0..self.ticks {
            while idx < self.commands.len() && self.commands[idx].tick == t {
                g.apply(self.commands[idx].cmd);
                idx += 1;
            }
            g.tick(dt_ms);
        }
        // any trailing commands at final tick
        while idx < self.commands.len() {
            g.apply(self.commands[idx].cmd);
            idx += 1;
        }
        g
    }

    /// Determinism check: re-run and compare key state. (for tests/app verify)
    pub fn is_deterministic(&self, dt_ms: u32) -> bool {
        let a = self.run(dt_ms);
        let b = self.run(dt_ms);
        a.score() == b.score()
            && a.lines() == b.lines()
            && a.engine.pieces_placed == b.engine.pieces_placed
    }

    // ---- disk (robust; atomic) ----

    pub fn path() -> std::path::PathBuf {
        crate::persist::data_dir_public().join(REPLAY_FILE)
    }
    pub fn save_to_disk(&self) -> std::io::Result<()> {
        crate::persist::atomic_write(&Self::path(), self.to_text().as_bytes())
    }
    pub fn load_from_disk() -> Option<Self> {
        let text = std::fs::read_to_string(Self::path()).ok()?;
        Self::from_text(&text)
    }

    // ---- custom text format ----

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("# tehtrys replay v{REPLAY_VERSION}\n"));
        s.push_str(&format!("seed={}\n", self.seed));
        s.push_str(&format!("variant={}\n", variant_name(self.variant)));
        s.push_str(&format!(
            "target_lines={}\n",
            variant_target_lines(self.variant)
        ));
        s.push_str(&format!(
            "duration_ms={}\n",
            variant_duration_ms(self.variant)
        ));
        s.push_str(&format!("start_level={}\n", self.start_level));
        s.push_str(&format!("ruleset={}\n", ruleset_name(self.ruleset)));
        s.push_str(&format!("ticks={}\n", self.ticks));
        s.push_str("commands=\n");
        for c in &self.commands {
            s.push_str(&format!("{} {}\n", c.tick, cmd_name(c.cmd)));
        }
        s
    }

    pub fn from_text(text: &str) -> Option<Self> {
        let mut seed = None;
        let mut variant_name_s = "marathon".to_string();
        let mut target_lines = 40u32;
        let mut duration_ms = 120_000u32;
        let mut start_level = 1u32;
        let mut ruleset = LockRuleset::Extended;
        let mut ticks = 0u64;
        let mut commands = Vec::new();
        let mut in_commands = false;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if in_commands {
                let mut it = line.split_whitespace();
                if let (Some(t), Some(c)) = (it.next(), it.next())
                    && let (Ok(tick), Some(cmd)) = (t.parse::<u64>(), parse_cmd(c))
                {
                    commands.push(ReplayCommand { tick, cmd });
                }
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "seed" => seed = v.trim().parse().ok(),
                    "variant" => variant_name_s = v.trim().into(),
                    "target_lines" => target_lines = v.trim().parse().unwrap_or(40),
                    "duration_ms" => duration_ms = v.trim().parse().unwrap_or(120_000),
                    "start_level" => start_level = v.trim().parse().unwrap_or(1),
                    "ruleset" => ruleset = parse_ruleset(v.trim()),
                    "ticks" => ticks = v.trim().parse().unwrap_or(0),
                    "commands" => in_commands = true,
                    _ => {}
                }
            }
        }
        let variant = parse_variant(&variant_name_s, target_lines, duration_ms);
        Some(Self {
            seed: seed?,
            variant,
            start_level,
            ruleset,
            commands,
            ticks,
        })
    }
}

// ---- name mapping (pure) ----

pub fn variant_name(v: Variant) -> &'static str {
    match v {
        Variant::Marathon => "marathon",
        Variant::Sprint { .. } => "sprint",
        Variant::Ultra { .. } => "ultra",
    }
}
fn variant_target_lines(v: Variant) -> u32 {
    match v {
        Variant::Sprint { target_lines } => target_lines,
        _ => 0,
    }
}
fn variant_duration_ms(v: Variant) -> u32 {
    match v {
        Variant::Ultra { duration_ms } => duration_ms,
        _ => 0,
    }
}
fn parse_variant(name: &str, target_lines: u32, duration_ms: u32) -> Variant {
    match name {
        "sprint" => Variant::Sprint { target_lines },
        "ultra" => Variant::Ultra { duration_ms },
        _ => Variant::Marathon,
    }
}
fn ruleset_name(r: LockRuleset) -> &'static str {
    match r {
        LockRuleset::Extended => "extended",
        LockRuleset::Infinite => "infinite",
        LockRuleset::Classic => "classic",
    }
}
fn parse_ruleset(s: &str) -> LockRuleset {
    match s {
        "infinite" => LockRuleset::Infinite,
        "classic" => LockRuleset::Classic,
        _ => LockRuleset::Extended,
    }
}
pub fn cmd_name(c: GameCommand) -> &'static str {
    match c {
        GameCommand::MoveLeft => "MoveLeft",
        GameCommand::MoveRight => "MoveRight",
        GameCommand::RotateCw => "RotateCw",
        GameCommand::RotateCcw => "RotateCcw",
        GameCommand::SoftDrop(true) => "SoftDrop+",
        GameCommand::SoftDrop(false) => "SoftDrop-",
        GameCommand::HardDrop => "HardDrop",
        GameCommand::Hold => "Hold",
        GameCommand::Pause => "Pause",
        GameCommand::Resume => "Resume",
    }
}
fn parse_cmd(s: &str) -> Option<GameCommand> {
    Some(match s {
        "MoveLeft" => GameCommand::MoveLeft,
        "MoveRight" => GameCommand::MoveRight,
        "RotateCw" => GameCommand::RotateCw,
        "RotateCcw" => GameCommand::RotateCcw,
        "SoftDrop+" => GameCommand::SoftDrop(true),
        "SoftDrop-" => GameCommand::SoftDrop(false),
        "HardDrop" => GameCommand::HardDrop,
        "Hold" => GameCommand::Hold,
        "Pause" => GameCommand::Pause,
        "Resume" => GameCommand::Resume,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::TICK_MS;

    fn sample_replay() -> Replay {
        let mut r = Replay::new(0x7E77_21F5, Variant::Marathon, 1, LockRuleset::Extended);
        // drive: occasionally rotate + shift + hard drop. simulate concurrently to keep sync simple:
        r.commands = vec![
            ReplayCommand {
                tick: 5,
                cmd: GameCommand::RotateCw,
            },
            ReplayCommand {
                tick: 10,
                cmd: GameCommand::MoveLeft,
            },
            ReplayCommand {
                tick: 12,
                cmd: GameCommand::HardDrop,
            },
            ReplayCommand {
                tick: 40,
                cmd: GameCommand::MoveRight,
            },
            ReplayCommand {
                tick: 45,
                cmd: GameCommand::HardDrop,
            },
            ReplayCommand {
                tick: 80,
                cmd: GameCommand::Hold,
            },
            ReplayCommand {
                tick: 90,
                cmd: GameCommand::HardDrop,
            },
        ];
        r.ticks = 200;
        r
    }

    #[test]
    fn deterministic_same_inputs_same_result() {
        let r = sample_replay();
        let a = r.run(TICK_MS);
        let b = r.run(TICK_MS);
        assert_eq!(a.score(), b.score());
        assert_eq!(a.lines(), b.lines());
        assert_eq!(a.engine.pieces_placed, b.engine.pieces_placed);
        assert_eq!(a.engine.matrix, b.engine.matrix);
        assert!(r.is_deterministic(TICK_MS));
    }

    #[test]
    fn different_seed_can_differ() {
        let r1 = sample_replay();
        let mut r2 = r1.clone();
        r2.seed ^= 0xFFFF;
        // not guaranteed to differ in score, but pieces order should differ -> matrix likely differs;
        // assert at least runs deterministically per seed.
        assert!(r1.is_deterministic(TICK_MS));
        assert!(r2.is_deterministic(TICK_MS));
    }

    #[test]
    fn text_roundtrip() {
        let r = sample_replay();
        let text = r.to_text();
        let back = Replay::from_text(&text).expect("parse");
        assert_eq!(r, back);
        // re-run from parsed == original
        assert_eq!(r.run(TICK_MS).score(), back.run(TICK_MS).score());
    }

    #[test]
    fn parse_ignores_garbage_lines() {
        let mut t = sample_replay().to_text();
        t.push_str("garbage line without equals\n# comment\n");
        assert!(Replay::from_text(&t).is_some());
        assert!(Replay::from_text("no seed here").is_none()); // seed required
    }

    #[test]
    fn cmd_names_roundtrip() {
        let cmds = [
            GameCommand::MoveLeft,
            GameCommand::MoveRight,
            GameCommand::RotateCw,
            GameCommand::RotateCcw,
            GameCommand::SoftDrop(true),
            GameCommand::SoftDrop(false),
            GameCommand::HardDrop,
            GameCommand::Hold,
            GameCommand::Pause,
            GameCommand::Resume,
        ];
        for c in cmds {
            assert_eq!(parse_cmd(cmd_name(c)), Some(c));
        }
    }
}
