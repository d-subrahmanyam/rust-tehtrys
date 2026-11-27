//! Input layer. (§4/§4.1/§5.2; flows/04) key→command; custom DAS/ARR.
//!
//! SDL key events are fed in; we **ignore SDL auto-repeat** and run our own DAS/ARR
//! (delayed auto-shift / auto-repeat rate) for Left/Right only. Remapping-ready.
//! Core receives discrete `GameCommand`s. App-level keys (pause/restart/quit)
//! are returned separately.

use crate::core::constants::*;
use crate::core::types::GameCommand;

/// DAS initial delay. (§5.2 ~0.3s)
pub const DAS_MS: u32 = 300;
/// ARR per cell after DAS. (§5.2 full width in ~0.5s => ~33.3ms; 10 cells)
pub const ARR_MS: u32 = 33;

/// App-level (non-game) keys.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppKey {
    Pause,
    Restart,
    Quit,
}

/// A physical key, abstracted from SDL so input is testable. We map SDL Keycodes
/// in `app`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Left,
    Right,
    Down,
    Space,
    Up,
    X,
    Z,
    LeftCtrl,
    C,
    LeftShift,
    Numpad0,
    Numpad2,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad8,
    Numpad1,
    Numpad3,
    Numpad7,
    Numpad9,
    Escape,
    F1,
    R,
    Q,
    M,
    /// confirm / select. (menus, name entry)
    Enter,
    /// delete previous char. (name entry)
    Backspace,
}

/// Key bindings. (§4.1; our defaults; remappable)
#[derive(Clone, Debug)]
pub struct KeyMap {
    // store lists per action for secondary bindings
    pub move_left: Vec<Key>,
    pub move_right: Vec<Key>,
    pub soft_drop: Vec<Key>,
    pub hard_drop: Vec<Key>,
    pub rotate_cw: Vec<Key>,
    pub rotate_ccw: Vec<Key>,
    pub hold: Vec<Key>,
    pub pause: Vec<Key>,
    pub restart: Vec<Key>,
    pub quit: Vec<Key>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            move_left: vec![Key::Left, Key::Numpad4],
            move_right: vec![Key::Right, Key::Numpad6],
            soft_drop: vec![Key::Down, Key::Numpad2],
            hard_drop: vec![Key::Space, Key::Numpad8], // §4.1 hard drop Space/Numpad8
            rotate_cw: vec![Key::Up, Key::X],          // §4.1 CW Up/X (numpad also via app map)
            rotate_ccw: vec![Key::Z, Key::LeftCtrl],   // §4.1 CCW Z/Ctrl
            hold: vec![Key::C, Key::LeftShift, Key::Numpad0], // §4.1 hold C/Shift/Numpad0
            pause: vec![Key::Escape, Key::F1],         // §4.1 pause Esc/F1
            restart: vec![Key::R],
            quit: vec![Key::Q],
        }
    }
}

/// Result of classifying a key press.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyAction {
    /// discrete game command (press edge).
    Command(GameCommand),
    /// soft drop level (true=held).
    SoftDrop(bool),
    /// movement key edge for DAS.
    MoveEdge(Direction, bool),
    /// app-level.
    App(AppKey),
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

impl KeyMap {
    /// Classify a physical key edge. `pressed` true=down,false=up.
    pub fn classify(&self, key: Key, pressed: bool) -> KeyAction {
        // movement edges always tracked for DAS
        if self.move_left.contains(&key) {
            return KeyAction::MoveEdge(Direction::Left, pressed);
        }
        if self.move_right.contains(&key) {
            return KeyAction::MoveEdge(Direction::Right, pressed);
        }
        // soft drop level
        if self.soft_drop.contains(&key) {
            return KeyAction::SoftDrop(pressed);
        }
        // app keys on press edge
        if pressed {
            if self.pause.contains(&key) {
                return KeyAction::App(AppKey::Pause);
            }
            if self.restart.contains(&key) {
                return KeyAction::App(AppKey::Restart);
            }
            if self.quit.contains(&key) {
                return KeyAction::App(AppKey::Quit);
            }
            // discrete game commands on press edge only
            if self.hard_drop.contains(&key) {
                return KeyAction::Command(GameCommand::HardDrop);
            }
            if self.rotate_cw.contains(&key) {
                return KeyAction::Command(GameCommand::RotateCw);
            }
            if self.rotate_ccw.contains(&key) {
                return KeyAction::Command(GameCommand::RotateCcw);
            }
            if self.hold.contains(&key) {
                return KeyAction::Command(GameCommand::Hold);
            }
        }
        KeyAction::None
    }
}

/// DAS/ARR auto-repeat for Left/Right. (§5.2)
#[derive(Clone, Debug, Default)]
pub struct DasArr {
    left_held: bool,
    right_held: bool,
    /// active repeating direction + its timer.
    active: Option<Direction>,
    /// ms until next event (DAS then ARR).
    timer_ms: u32,
    /// have we emitted the initial tap; next is DAS-wait.
    in_das: bool,
}

impl DasArr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Key edge. Returns a `MoveLeft/Right` to emit immediately on press. (§5.2 tap)
    pub fn on_edge(&mut self, dir: Direction, pressed: bool) -> Option<GameCommand> {
        match dir {
            Direction::Left => self.left_held = pressed,
            Direction::Right => self.right_held = pressed,
        }
        let opposite_held = match dir {
            Direction::Left => self.right_held,
            Direction::Right => self.left_held,
        };

        if pressed {
            // press: emit one cell immediately; begin DAS for this dir. (§5.2)
            // if opposite was active, cancel it; this dir wins with fresh DAS. (§5.2)
            self.active = Some(dir);
            self.timer_ms = DAS_MS;
            self.in_das = true;
            Some(match dir {
                Direction::Left => GameCommand::MoveLeft,
                Direction::Right => GameCommand::MoveRight,
            })
        } else {
            // release: if this dir was active, resume other if still held w/ fresh DAS. (§5.2)
            if self.active == Some(dir) {
                if opposite_held {
                    let other = match dir {
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                    };
                    self.active = Some(other);
                    self.timer_ms = DAS_MS;
                    self.in_das = true;
                    // (re-apply DAS; don't emit immediate tap here; it already... §5.2)
                } else {
                    self.active = None;
                }
            }
            None
        }
    }

    /// Advance time; emit repeats. (§5.2 continues across lock-down; engine decides)
    pub fn tick(&mut self, dt_ms: u32) -> Vec<GameCommand> {
        let mut out = Vec::new();
        let Some(dir) = self.active else { return out };
        // re-check still held
        let held = match dir {
            Direction::Left => self.left_held,
            Direction::Right => self.right_held,
        };
        if !held {
            self.active = None;
            return out;
        }
        if self.timer_ms > dt_ms {
            self.timer_ms -= dt_ms;
            return out;
        }
        // fire; if in DAS, switch to ARR
        let overflow = dt_ms - self.timer_ms;
        if self.in_das {
            self.in_das = false;
            self.timer_ms = ARR_MS.saturating_sub(overflow);
        } else {
            self.timer_ms = ARR_MS.saturating_sub(overflow);
        }
        out.push(match dir {
            Direction::Left => GameCommand::MoveLeft,
            Direction::Right => GameCommand::MoveRight,
        });
        // avoid infinite catchup; clamp
        out
    }
}

// ---------- persistence + remapping (Phase 5; §4.1 remapping; §2.5 persisted) ----------

/// Remappable actions. (§4.1) each has a list of bound `Key`s.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemapTarget {
    MoveLeft,
    MoveRight,
    SoftDrop,
    HardDrop,
    RotateCw,
    RotateCcw,
    Hold,
    Pause,
    Restart,
    Quit,
}

impl RemapTarget {
    pub const ALL: &'static [RemapTarget] = &[
        RemapTarget::MoveLeft,
        RemapTarget::MoveRight,
        RemapTarget::SoftDrop,
        RemapTarget::HardDrop,
        RemapTarget::RotateCw,
        RemapTarget::RotateCcw,
        RemapTarget::Hold,
        RemapTarget::Pause,
        RemapTarget::Restart,
        RemapTarget::Quit,
    ];
    pub fn label(self) -> &'static str {
        match self {
            RemapTarget::MoveLeft => "MOVE LEFT",
            RemapTarget::MoveRight => "MOVE RIGHT",
            RemapTarget::SoftDrop => "SOFT DROP",
            RemapTarget::HardDrop => "HARD DROP",
            RemapTarget::RotateCw => "ROTATE CW",
            RemapTarget::RotateCcw => "ROTATE CCW",
            RemapTarget::Hold => "HOLD",
            RemapTarget::Pause => "PAUSE",
            RemapTarget::Restart => "RESTART",
            RemapTarget::Quit => "QUIT",
        }
    }
    pub fn key_name(self) -> &'static str {
        match self {
            RemapTarget::MoveLeft => "move_left",
            RemapTarget::MoveRight => "move_right",
            RemapTarget::SoftDrop => "soft_drop",
            RemapTarget::HardDrop => "hard_drop",
            RemapTarget::RotateCw => "rotate_cw",
            RemapTarget::RotateCcw => "rotate_ccw",
            RemapTarget::Hold => "hold",
            RemapTarget::Pause => "pause",
            RemapTarget::Restart => "restart",
            RemapTarget::Quit => "quit",
        }
    }
    pub fn from_key_name(s: &str) -> Option<RemapTarget> {
        RemapTarget::ALL.iter().copied().find(|t| t.key_name() == s)
    }
}

impl Key {
    /// stable text name for persistence.
    pub fn name(self) -> &'static str {
        match self {
            Key::Left => "Left",
            Key::Right => "Right",
            Key::Down => "Down",
            Key::Space => "Space",
            Key::Up => "Up",
            Key::X => "X",
            Key::Z => "Z",
            Key::LeftCtrl => "LeftCtrl",
            Key::C => "C",
            Key::LeftShift => "LeftShift",
            Key::Numpad0 => "Numpad0",
            Key::Numpad2 => "Numpad2",
            Key::Numpad4 => "Numpad4",
            Key::Numpad5 => "Numpad5",
            Key::Numpad6 => "Numpad6",
            Key::Numpad8 => "Numpad8",
            Key::Numpad1 => "Numpad1",
            Key::Numpad3 => "Numpad3",
            Key::Numpad7 => "Numpad7",
            Key::Numpad9 => "Numpad9",
            Key::Escape => "Escape",
            Key::F1 => "F1",
            Key::R => "R",
            Key::Q => "Q",
            Key::M => "M",
            Key::Enter => "Enter",
            Key::Backspace => "Backspace",
        }
    }
    pub fn from_name(s: &str) -> Option<Key> {
        Some(match s {
            "Left" => Key::Left,
            "Right" => Key::Right,
            "Down" => Key::Down,
            "Space" => Key::Space,
            "Up" => Key::Up,
            "X" => Key::X,
            "Z" => Key::Z,
            "LeftCtrl" => Key::LeftCtrl,
            "C" => Key::C,
            "LeftShift" => Key::LeftShift,
            "Numpad0" => Key::Numpad0,
            "Numpad2" => Key::Numpad2,
            "Numpad4" => Key::Numpad4,
            "Numpad5" => Key::Numpad5,
            "Numpad6" => Key::Numpad6,
            "Numpad8" => Key::Numpad8,
            "Numpad1" => Key::Numpad1,
            "Numpad3" => Key::Numpad3,
            "Numpad7" => Key::Numpad7,
            "Numpad9" => Key::Numpad9,
            "Escape" => Key::Escape,
            "F1" => Key::F1,
            "R" => Key::R,
            "Q" => Key::Q,
            "M" => Key::M,
            "Enter" => Key::Enter,
            "Backspace" => Key::Backspace,
            _ => return None,
        })
    }
    /// human label for display.
    pub fn label(self) -> &'static str {
        self.name()
    }
}

impl KeyMap {
    fn binding(&self, t: RemapTarget) -> &Vec<Key> {
        match t {
            RemapTarget::MoveLeft => &self.move_left,
            RemapTarget::MoveRight => &self.move_right,
            RemapTarget::SoftDrop => &self.soft_drop,
            RemapTarget::HardDrop => &self.hard_drop,
            RemapTarget::RotateCw => &self.rotate_cw,
            RemapTarget::RotateCcw => &self.rotate_ccw,
            RemapTarget::Hold => &self.hold,
            RemapTarget::Pause => &self.pause,
            RemapTarget::Restart => &self.restart,
            RemapTarget::Quit => &self.quit,
        }
    }
    fn binding_mut(&mut self, t: RemapTarget) -> &mut Vec<Key> {
        match t {
            RemapTarget::MoveLeft => &mut self.move_left,
            RemapTarget::MoveRight => &mut self.move_right,
            RemapTarget::SoftDrop => &mut self.soft_drop,
            RemapTarget::HardDrop => &mut self.hard_drop,
            RemapTarget::RotateCw => &mut self.rotate_cw,
            RemapTarget::RotateCcw => &mut self.rotate_ccw,
            RemapTarget::Hold => &mut self.hold,
            RemapTarget::Pause => &mut self.pause,
            RemapTarget::Restart => &mut self.restart,
            RemapTarget::Quit => &mut self.quit,
        }
    }
    /// comma-joined key names for a target. (display + persistence)
    pub fn binding_names(&self, t: RemapTarget) -> String {
        let v = self.binding(t);
        if v.is_empty() {
            return "(none)".into();
        }
        v.iter().map(|k| k.name()).collect::<Vec<_>>().join(",")
    }
    /// **Remap**: set a single primary binding for `target`. (§4.1 remapping)
    pub fn set_binding(&mut self, t: RemapTarget, key: Key) {
        let b = self.binding_mut(t);
        // avoid duplicate within action
        b.retain(|k| *k != key);
        b.insert(0, key);
    }
    /// add a secondary binding.
    pub fn add_binding(&mut self, t: RemapTarget, key: Key) {
        let b = self.binding_mut(t);
        if !b.contains(&key) {
            b.push(key);
        }
    }
    pub fn clear_binding(&mut self, t: RemapTarget) {
        self.binding_mut(t).clear();
    }
    pub fn reset_defaults(&mut self) {
        *self = KeyMap::default();
    }

    // ---- persistence (custom text; no serde) ----

    /// `key.move_left=Left,Numpad4` lines.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for t in RemapTarget::ALL {
            s.push_str(&format!(
                "key.{}={}\n",
                t.key_name(),
                self.binding_names(*t).replace("(none)", "")
            ));
        }
        s
    }
    /// merge `key.*` lines into self. unknown keys ignored.
    pub fn apply_text_line(&mut self, k: &str, v: &str) {
        if let Some(name) = k.trim().strip_prefix("key.")
            && let Some(t) = RemapTarget::from_key_name(name.trim())
        {
            let keys: Vec<Key> = v
                .split(',')
                .filter_map(|n| Key::from_name(n.trim()))
                .collect();
            *self.binding_mut(t) = keys;
        }
    }
}

#[cfg(test)]
mod remap_tests {
    use super::*;

    #[test]
    fn key_name_roundtrip_all() {
        // every constructible Key roundtrips through name.
        let all = [
            Key::Left,
            Key::Right,
            Key::Down,
            Key::Space,
            Key::Up,
            Key::X,
            Key::Z,
            Key::LeftCtrl,
            Key::C,
            Key::LeftShift,
            Key::Numpad0,
            Key::Numpad1,
            Key::Numpad2,
            Key::Numpad3,
            Key::Numpad4,
            Key::Numpad5,
            Key::Numpad6,
            Key::Numpad7,
            Key::Numpad8,
            Key::Numpad9,
            Key::Escape,
            Key::F1,
            Key::R,
            Key::Q,
            Key::M,
            Key::Enter,
            Key::Backspace,
        ];
        for k in all {
            assert_eq!(Key::from_name(k.name()), Some(k), "roundtrip {k:?}");
        }
        assert_eq!(Key::from_name("nope"), None);
    }

    #[test]
    fn remap_set_and_clear() {
        let mut km = KeyMap::default();
        km.set_binding(RemapTarget::HardDrop, Key::Enter);
        assert!(km.classify(Key::Enter, true) == KeyAction::Command(GameCommand::HardDrop));
        assert!(km.hard_drop.contains(&Key::Enter));
        km.clear_binding(RemapTarget::HardDrop);
        assert_eq!(km.classify(Key::Enter, true), KeyAction::None);
    }

    #[test]
    fn keymap_text_roundtrip() {
        let mut km = KeyMap::default();
        km.set_binding(RemapTarget::MoveLeft, Key::Enter);
        let text = km.to_text();
        let mut km2 = KeyMap::default();
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                km2.apply_text_line(k, v);
            }
        }
        assert_eq!(
            km.binding_names(RemapTarget::MoveLeft),
            km2.binding_names(RemapTarget::MoveLeft)
        );
        assert!(km2.move_left.contains(&Key::Enter));
    }

    #[test]
    fn remap_target_all_labels() {
        for t in RemapTarget::ALL {
            assert!(!t.label().is_empty());
            assert!(!t.key_name().is_empty());
        }
    }
}

/// Timing re-export for app.
pub const _TICK_MS: u32 = TICK_MS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keymap_defaults_guideline_4_1() {
        let km = KeyMap::default();
        assert!(km.move_left.contains(&Key::Left));
        assert!(km.hard_drop.contains(&Key::Space)); // §4.1
        assert!(km.rotate_cw.contains(&Key::Up)); // §4.1
        assert!(km.rotate_ccw.contains(&Key::Z)); // §4.1
        assert!(km.hold.contains(&Key::C)); // §4.1
        assert!(km.pause.contains(&Key::Escape)); // §4.1
        assert!(km.pause.contains(&Key::F1)); // §4.1
    }

    #[test]
    fn discrete_on_press_edge_only() {
        let km = KeyMap::default();
        assert_eq!(
            km.classify(Key::Space, true),
            KeyAction::Command(GameCommand::HardDrop)
        );
        assert_eq!(km.classify(Key::Space, false), KeyAction::None); // no release
        assert_eq!(
            km.classify(Key::Up, true),
            KeyAction::Command(GameCommand::RotateCw)
        );
    }

    #[test]
    fn soft_drop_is_level() {
        let km = KeyMap::default();
        assert_eq!(km.classify(Key::Down, true), KeyAction::SoftDrop(true));
        assert_eq!(km.classify(Key::Down, false), KeyAction::SoftDrop(false));
    }

    #[test]
    fn das_initial_tap_then_delay_then_arr() {
        let mut d = DasArr::new();
        // press: immediate one cell
        let first = d.on_edge(Direction::Left, true);
        assert_eq!(first, Some(GameCommand::MoveLeft));
        // nothing before DAS
        assert!(d.tick(DAS_MS - 1).is_empty());
        // after DAS, one repeat
        let r = d.tick(1);
        assert_eq!(r, vec![GameCommand::MoveLeft]);
        // then ARR repeats
        assert!(d.tick(ARR_MS - 1).is_empty());
        assert_eq!(d.tick(1), vec![GameCommand::MoveLeft]);
    }

    #[test]
    fn das_opposite_direction_reset() {
        let mut d = DasArr::new();
        d.on_edge(Direction::Left, true);
        d.tick(DAS_MS); // repeating left
        // press opposite: emits right, cancels left, fresh DAS
        assert_eq!(
            d.on_edge(Direction::Right, true),
            Some(GameCommand::MoveRight)
        );
        assert!(d.tick(DAS_MS - 1).is_empty());
        assert_eq!(d.tick(1), vec![GameCommand::MoveRight]);
    }

    #[test]
    fn das_release_resumes_other_with_delay() {
        let mut d = DasArr::new();
        d.on_edge(Direction::Left, true);
        d.on_edge(Direction::Right, true); // right wins
        // release right while left still held -> resume left w/ fresh DAS
        d.on_edge(Direction::Right, false);
        assert!(d.tick(DAS_MS - 1).is_empty());
        assert_eq!(d.tick(1), vec![GameCommand::MoveLeft]);
    }

    #[test]
    fn release_all_stops() {
        let mut d = DasArr::new();
        d.on_edge(Direction::Left, true);
        d.tick(DAS_MS);
        d.on_edge(Direction::Left, false);
        assert!(d.tick(10_000).is_empty());
    }
}
