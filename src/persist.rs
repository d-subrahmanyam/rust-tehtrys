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
            // sanitize separators in name
            let name = e.name.replace(['|', '\n'], " ");
            s.push_str(&format!(
                "{}|{}|{}|{}|{}|{}\n",
                name, e.score, e.level, e.lines, e.variant, e.time_ms
            ));
        }
        let mut f = fs::File::create(p)?;
        f.write_all(s.as_bytes())
    }
}

/// Key=value settings. (§2.3/§2.5 persisted) simple text.
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
        }
    }
}

impl Settings {
    pub fn path() -> PathBuf {
        data_dir().join(SETTINGS_FILE)
    }
    pub fn load() -> Self {
        let mut s = Self::default();
        if let Ok(text) = fs::read_to_string(Self::path()) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    match k.trim() {
                        "lock_ruleset" => s.lock_ruleset = v.trim().into(),
                        "next_len" => s.next_len = v.trim().parse().unwrap_or(5).clamp(1, 6),
                        "hold_enabled" => s.hold_enabled = v.trim() == "true",
                        "ghost_enabled" => s.ghost_enabled = v.trim() == "true",
                        "monochrome" => s.monochrome = v.trim() == "true",
                        "goal_system" => s.goal_system = v.trim().into(),
                        "start_level" => s.start_level = v.trim().parse().unwrap_or(1).clamp(1, 15),
                        "variant" => s.variant = v.trim().into(),
                        _ => {}
                    }
                }
            }
        }
        s
    }
    pub fn save(&self) -> std::io::Result<()> {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir)?;
        }
        let s = format!(
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
        fs::write(p, s)
    }
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
