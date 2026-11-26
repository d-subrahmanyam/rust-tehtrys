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
