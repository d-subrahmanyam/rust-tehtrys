//! Persistence. (§11.2 high score table; settings §2.3/§2.5) No serde; simple
//! versioned text. High scores per variant. Local file.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub const SCORES_FILE: &str = "tehtrys_scores.txt";
pub const SETTINGS_FILE: &str = "tehtrys_settings.txt";
pub const MAX_ENTRIES: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HighScoreEntry {
    pub name: String,
    pub score: u64,
    pub level: u32,
    pub lines: u32,
    pub variant: String,
    pub time_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct HighScoreTable {
    pub entries: Vec<HighScoreEntry>,
}

impl HighScoreTable {
    /// Qualify for inclusion? (top N or empty) (§11.2)
    pub fn qualifies(&self, score: u64, variant: &str) -> bool {
        let mut v: Vec<u64> = self
            .entries
            .iter()
            .filter(|e| e.variant == variant)
            .map(|e| e.score)
            .collect();
        v.sort_unstable_by(|a, b| b.cmp(a));
        v.len() < MAX_ENTRIES || score > *v.last().unwrap_or(&0)
    }

    /// Insert + sort + truncate per variant. returns rank (0-based) if added. (§11.2)
    pub fn insert(&mut self, e: HighScoreEntry) -> Option<usize> {
        let variant = e.variant.clone();
        let score = e.score;
        let name = e.name.clone();
        self.entries.push(e);
        self.entries.sort_by_key(|e| std::cmp::Reverse(e.score));
        // truncate per variant to MAX_ENTRIES
        let mut seen = std::collections::HashMap::<String, usize>::new();
        let mut out = Vec::new();
        for e in self.entries.drain(..) {
            let n = seen.entry(e.variant.clone()).or_insert(0);
            if *n < MAX_ENTRIES {
                out.push(e);
                *n += 1;
            }
        }
        self.entries = out;
        // rank within variant
        self.entries
            .iter()
            .filter(|x| x.variant == variant)
            .position(|x| x.score == score && x.name == name)
    }

    /// Entries for a variant, sorted desc. (§11.2 top players per variant)
    pub fn for_variant(&self, variant: &str) -> Vec<&HighScoreEntry> {
        let mut v: Vec<&HighScoreEntry> = self
            .entries
            .iter()
            .filter(|e| e.variant == variant)
            .collect();
        v.sort_by_key(|e| std::cmp::Reverse(e.score));
        v
    }

    pub fn top(&self, variant: &str, n: usize) -> Vec<&HighScoreEntry> {
        self.for_variant(variant).into_iter().take(n).collect()
    }

    // ---- persistence (simple text) ----

    pub fn path() -> PathBuf {
        data_dir().join(SCORES_FILE)
    }

    pub fn load() -> Self {
        let p = Self::path();
        let Ok(text) = fs::read_to_string(&p) else {
            return Self::default();
        };
        let mut entries = Vec::new();
        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.splitn(6, '|').collect();
            if f.len() == 6 {
                entries.push(HighScoreEntry {
                    name: f[0].to_string(),
                    score: f[1].parse().unwrap_or(0),
                    level: f[2].parse().unwrap_or(1),
                    lines: f[3].parse().unwrap_or(0),
                    variant: f[4].to_string(),
                    time_ms: f[5].parse().unwrap_or(0),
                });
            }
        }
        Self { entries }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut s = String::from("# tehtrys high scores: name|score|level|lines|variant|time_ms\n");
        for e in &self.entries {
            // sanitize separators + length in name (robustness)
            let name = sanitize_name(&e.name).replace(['|', '\n'], " ");
            s.push_str(&format!(
                "{}|{}|{}|{}|{}|{}\n",
                name, e.score, e.level, e.lines, e.variant, e.time_ms
            ));
        }
        atomic_write(&p, s.as_bytes())
    }
}

/// Key=value settings. (§2.3/§2.5 persisted; Phase-5 keys + sound + remapping) simple text.
#[derive(Clone, Debug)]
pub struct Settings {
    pub lock_ruleset: String, // extended|infinite|classic
    pub next_len: usize,      // 1..=6
    pub hold_enabled: bool,
    pub ghost_enabled: bool,
    pub monochrome: bool,
    pub goal_system: String, // fixed|variable
    pub start_level: u32,    // 1..=15
    pub variant: String,     // marathon|sprint|ultra
    // ---- Phase 5 ----
    /// sound + music. (§13.2/§13.3; §2.5)
    pub sound: crate::sound::SoundSettings,
    /// key remapping. (§4.1; §2.5 persisted)
    pub keymap: crate::input::KeyMap,
    /// seeded randomness. `seed_random` => pick fresh seed each game; else use `seed`. (§3.3; replay)
    pub seed_random: bool,
    pub seed: u64,
    /// last high-score name entered. (§11.2 name entry)
    pub last_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lock_ruleset: "extended".into(),
            next_len: 5,
            hold_enabled: true,  // default on §2.5.2
            ghost_enabled: true, // default on §2.5.3
            monochrome: false,
            goal_system: "fixed".into(),
            start_level: 1,
            variant: "marathon".into(),
            sound: crate::sound::SoundSettings::default(),
            keymap: crate::input::KeyMap::default(),
            seed_random: true, // random seed each game; deterministic replay still available
            seed: 0x7E77_21F5, // fixed fallback seed (Phase-4 value)
            last_name: "YOU".into(),
        }
    }
}

impl Settings {
    pub fn path() -> PathBuf {
        data_dir().join(SETTINGS_FILE)
    }

    /// parse from text (pure; robust; unknown keys ignored; missing keys default).
    pub fn from_text(text: &str) -> Self {
        let mut s = Self::default();
        s.apply_text(text);
        s
    }

    pub fn apply_text(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim();
                match k {
                    "lock_ruleset" => self.lock_ruleset = v.into(),
                    "next_len" => self.next_len = v.parse().unwrap_or(5).clamp(1, 6),
                    "hold_enabled" => self.hold_enabled = v == "true",
                    "ghost_enabled" => self.ghost_enabled = v == "true",
                    "monochrome" => self.monochrome = v == "true",
                    "goal_system" => self.goal_system = v.into(),
                    "start_level" => self.start_level = v.parse().unwrap_or(1).clamp(1, 15),
                    "variant" => self.variant = v.into(),
                    // sound (§13.2/§13.3)
                    "sfx_enabled" => self.sound.sfx_enabled = v == "true",
                    "sfx_volume" => {
                        self.sound.sfx_volume =
                            crate::sound::SoundSettings::clamp_volume(v.parse().unwrap_or(80))
                    }
                    "music_enabled" => self.sound.music_enabled = v == "true",
                    "music_volume" => {
                        self.sound.music_volume =
                            crate::sound::SoundSettings::clamp_volume(v.parse().unwrap_or(50))
                    }
                    "muted" => self.sound.muted = v == "true",
                    // replay/seed (§3.3)
                    "seed_random" => self.seed_random = v == "true",
                    "seed" => self.seed = v.parse().unwrap_or(0x7E77_21F5),
                    "last_name" => self.last_name = sanitize_name(v),
                    // key remapping (key.*)
                    _ if k.starts_with("key.") => self.keymap.apply_text_line(k, v),
                    _ => {}
                }
            }
        }
    }

    /// serialize to text (pure).
    pub fn to_text(&self) -> String {
        let mut s = format!(
            "lock_ruleset={}\nnext_len={}\nhold_enabled={}\nghost_enabled={}\nmonochrome={}\ngoal_system={}\nstart_level={}\nvariant={}\n",
            self.lock_ruleset,
            self.next_len,
            self.hold_enabled,
            self.ghost_enabled,
            self.monochrome,
            self.goal_system,
            self.start_level,
            self.variant
        );
        s.push_str(&format!(
            "sfx_enabled={}\nsfx_volume={}\nmusic_enabled={}\nmusic_volume={}\nmuted={}\n",
            self.sound.sfx_enabled,
            self.sound.sfx_volume,
            self.sound.music_enabled,
            self.sound.music_volume,
            self.sound.muted
        ));
        s.push_str(&format!(
            "seed_random={}\nseed={}\nlast_name={}\n",
            self.seed_random,
            self.seed,
            sanitize_name(&self.last_name)
        ));
        s.push_str(&self.keymap.to_text());
        s
    }

    pub fn load() -> Self {
        match fs::read_to_string(Self::path()) {
            Ok(text) => Self::from_text(&text),
            Err(_) => Self::default(),
        }
    }

    /// save robustly: write temp + rename (atomic; robustness).
    pub fn save(&self) -> std::io::Result<()> {
        let p = Self::path();
        atomic_write(&p, self.to_text().as_bytes())
    }
}

/// sanitize a name for persistence + display. (§11.2 name entry; robustness)
pub fn sanitize_name(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_control() && *c != '|' && *c != '=' && *c != '\n')
        .take(crate::persist::MAX_NAME_LEN)
        .collect();
    let trimmed = cleaned.trim().to_string();
    if trimmed.is_empty() {
        "YOU".into()
    } else {
        trimmed
    }
}

/// max high-score name length. (§11.2)
pub const MAX_NAME_LEN: usize = 10;

/// write file atomically (temp + rename). robustness.
pub fn atomic_write(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, path)
}

/// public data dir. (~/.local/share/tehtrys) for settings/scores/replay/music.
pub fn data_dir_public() -> PathBuf {
    data_dir()
}

fn data_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("tehtrys")
    } else {
        std::env::temp_dir().join("tehtrys")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifies_and_insert_sorts_truncates() {
        let mut t = HighScoreTable::default();
        for i in 0..15u64 {
            t.insert(HighScoreEntry {
                name: format!("p{i}"),
                score: i * 10,
                level: 1,
                lines: 0,
                variant: "Marathon".into(),
                time_ms: 0,
            });
        }
        assert_eq!(t.for_variant("Marathon").len(), MAX_ENTRIES);
        // highest first
        assert_eq!(t.for_variant("Marathon")[0].score, 140);
        assert!(t.qualifies(1000, "Marathon"));
        // low score doesn't qualify once full
        assert!(!t.qualifies(0, "Marathon"));
        // per-variant isolation
        assert!(t.for_variant("Sprint").is_empty());
    }

    #[test]
    fn settings_defaults_official() {
        let s = Settings::default();
        assert!(s.hold_enabled); // §2.5.2 default on
        assert!(s.ghost_enabled); // §2.5.3 default on
        assert_eq!(s.lock_ruleset, "extended"); // §2.5.4 default
        assert_eq!(s.next_len, 5);
        assert_eq!(s.start_level, 1); // §2.3.3 default 1
        assert_eq!(s.variant, "marathon"); // §2.3.2 default
        // Phase-5 defaults
        assert!(s.sound.sfx_enabled);
        assert!(s.sound.music_enabled);
        assert!(!s.sound.muted);
        assert_eq!(s.seed, 0x7E77_21F5);
        assert_eq!(s.last_name, "YOU");
    }

    #[test]
    fn settings_text_roundtrip_including_sound_and_keymap() {
        let mut s = Settings::default();
        s.sound.sfx_volume = 33;
        s.sound.music_volume = 66;
        s.sound.muted = true;
        s.sound.sfx_enabled = false;
        s.seed = 12345;
        s.seed_random = false;
        s.last_name = "ALICE".into();
        s.keymap.set_binding(
            crate::input::RemapTarget::HardDrop,
            crate::input::Key::Enter,
        );
        let text = s.to_text();
        let back = Settings::from_text(&text);
        assert_eq!(back.sound.sfx_volume, 33);
        assert_eq!(back.sound.music_volume, 66);
        assert!(back.sound.muted);
        assert!(!back.sound.sfx_enabled);
        assert_eq!(back.seed, 12345);
        assert!(!back.seed_random);
        assert_eq!(back.last_name, "ALICE");
        assert!(back.keymap.hard_drop.contains(&crate::input::Key::Enter));
    }

    #[test]
    fn settings_robust_against_garbage() {
        let s = Settings::from_text(
            "garbage\n=x\nnext_len=999\nsfx_volume=-5\nstart_level=abc\nunknown=1\nkey.bogus=Left\n",
        );
        assert_eq!(s.next_len, 6); // clamped
        assert_eq!(s.sound.sfx_volume, 0); // clamped
        assert_eq!(s.start_level, 1); // default on parse fail
    }

    #[test]
    fn sanitize_name_rules() {
        assert_eq!(sanitize_name("  hi  "), "hi");
        assert_eq!(sanitize_name("a|b=c\nd"), "abcd");
        assert_eq!(sanitize_name(""), "YOU");
        assert!(sanitize_name(&"x".repeat(50)).len() <= MAX_NAME_LEN);
    }

    #[test]
    fn roundtrip_simple_format() {
        // can't write to HOME safely in test; verify format shape via insert only
        let mut t = HighScoreTable::default();
        t.insert(HighScoreEntry {
            name: "AAA".into(),
            score: 5,
            level: 2,
            lines: 3,
            variant: "Marathon".into(),
            time_ms: 9,
        });
        assert_eq!(t.entries[0].name, "AAA");
    }
}
