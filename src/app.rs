//! SDL2 application shell. (flows/14 state machine; flows/03/04/12/13; §2 screens;
//! §11.2 high score + name entry; §2.5 options; §13.2/§13.3 sound; §4.1 remapping;
//! Phase 4 + Phase 5) Sound is presentation-only; `core` stays pure.

use crate::core::constants::*;
use crate::core::engine::Phase;
use crate::core::game::{Game, Variant};
use crate::core::levels::GoalSystem;
use crate::core::types::{EngineEvent, GameCommand, LockRuleset, VariantEndReason};
use crate::input::{AppKey, DasArr, Key, KeyAction, KeyMap, RemapTarget};
use crate::persist::{self, HighScoreEntry, HighScoreTable, Settings};
use crate::render::{self, Fx, RenderSnapshot};
use crate::replay::Replay;
use crate::sound::{Audio, MusicSlot, NullAudio, SdlAudio, SoundCue, SoundSettings};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// Application state. (flows/14) Phase-4 + Phase-5 screens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AppState {
    Title,     // §2.2
    MainMenu,  // §2.3
    Options,   // §2.5
    KeyMap,    // §4.1 remapping (Phase 5)
    HighScore, // §2.3.4/§11.2
    Help,      // §2.6 help / scoring demo / credits (Phase 5)
    Playing,   // §2.4 + pause
    NameEntry, // §11.2 high-score name entry (Phase 5)
    GameOver,  // §11
    Demo,      // attract-mode demo (Phase 5)
}

/// Which game variant. (§12)
fn variant_from(s: &str) -> Variant {
    match s {
        "sprint" => Variant::Sprint { target_lines: 40 },
        "ultra" => Variant::Ultra {
            duration_ms: 120_000,
        },
        _ => Variant::Marathon,
    }
}
fn ruleset_from(s: &str) -> LockRuleset {
    match s {
        "infinite" => LockRuleset::Infinite,
        "classic" => LockRuleset::Classic,
        _ => LockRuleset::Extended,
    }
}
fn goal_from(s: &str) -> GoalSystem {
    match s {
        "variable" => GoalSystem::Variable,
        _ => GoalSystem::Fixed,
    }
}

const MAIN_MENU: &[&str] = &[
    "NEW GAME",
    "VARIATION",
    "START LEVEL",
    "LOCK DOWN",
    "OPTIONS",
    "KEY MAPPING",
    "HIGH SCORE",
    "HELP",
    "DEMO",
    "QUIT",
];

/// Options rows. (§2.5 + Phase-5 sound/remap/seed)
const OPTIONS: &[&str] = &[
    "NEXT QUEUE (1-6)",
    "HOLD",
    "GHOST",
    "MONOCHROME",
    "LOCK DOWN",
    "GOAL SYSTEM",
    "SFX",
    "SFX VOLUME",
    "MUSIC",
    "MUSIC VOLUME",
    "MUTE ALL",
    "RANDOM SEED",
    "SEED",
    "KEY MAPPING",
];

/// fixed deterministic seed value. (Phase-4; §3.3)
const FIXED_SEED_DEFAULT: u64 = 0x7E77_21F5;
/// auto demo interval ms.
const DEMO_DROP_MS: u32 = 350;

struct App {
    state: AppState,
    game: Game,
    settings: Settings,
    das: DasArr,
    paused: bool,
    running: bool,
    accumulator_ms: u32,
    last_ticks: u32,
    fx: Fx,
    high_score: u64,
    scores: HighScoreTable,
    menu_index: usize,
    options_index: usize,
    // key remapping (Phase 5)
    remap_index: usize,
    remap_capture: bool,
    // high-score name entry (Phase 5)
    name_input: String,
    score_recorded: bool,
    // pending game-over info
    pending_score: u64,
    pending_lines: u32,
    pending_level: u32,
    pending_time: u64,
    pending_reason: String,
    // seeded randomness + replay (Phase 5)
    current_seed: u64,
    replay: Replay,
    replay_tick_count: u64,
    entropy_counter: u64,
    // sound (Phase 5; presentation-only)
    sound: Box<dyn Audio>,
    // demo (Phase 5)
    demo_game: Game,
    demo_timer_ms: u32,
    demo_seed: u64,
}

impl App {
    fn new(mut sound: Box<dyn Audio>) -> Self {
        let settings = Settings::load();
        let variant = variant_from(&settings.variant);
        let ruleset = ruleset_from(&settings.lock_ruleset);
        let seed = FIXED_SEED_DEFAULT;
        let replay = Replay::new(seed, variant, settings.start_level, ruleset);
        sound.apply_settings(settings.sound);
        let mut app = Self {
            state: AppState::Title,
            settings: settings.clone(),
            game: Game::new(seed, variant, settings.start_level, ruleset),
            das: DasArr::new(),
            paused: false,
            running: true,
            accumulator_ms: 0,
            last_ticks: 0,
            fx: Fx::default(),
            high_score: 0,
            scores: HighScoreTable::load(),
            menu_index: 0,
            options_index: 0,
            remap_index: 0,
            remap_capture: false,
            name_input: String::new(),
            score_recorded: false,
            pending_score: 0,
            pending_lines: 0,
            pending_level: 1,
            pending_time: 0,
            pending_reason: String::new(),
            current_seed: seed,
            replay,
            replay_tick_count: 0,
            entropy_counter: 0,
            sound,
            demo_game: Game::new(seed ^ 0xDEAD, Variant::Marathon, 1, LockRuleset::Extended),
            demo_timer_ms: 0,
            demo_seed: seed ^ 0xDEAD,
        };
        app.apply_settings_to_engine();
        app
    }

    fn keymap(&self) -> &KeyMap {
        &self.settings.keymap
    }

    fn variant(&self) -> Variant {
        variant_from(&self.settings.variant)
    }
    fn ruleset(&self) -> LockRuleset {
        ruleset_from(&self.settings.lock_ruleset)
    }

    fn apply_settings_to_engine(&mut self) {
        self.game.engine.hold_enabled = self.settings.hold_enabled;
        let gs = goal_from(&self.settings.goal_system);
        self.game.engine.goal_system = gs;
        self.game.engine.goal = crate::core::levels::starting_goal(gs, self.game.engine.level);
    }

    fn apply_sound_settings(&mut self) {
        self.sound.apply_settings(self.settings.sound);
    }

    fn persist_settings(&self) {
        let _ = self.settings.save();
    }

    // ---- seed ----

    fn entropy_seed(&mut self) -> u64 {
        // app layer may use OS time; core stays pure.
        self.entropy_counter = self.entropy_counter.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5DEECE66D);
        nanos ^ self.entropy_counter ^ self.settings.seed.rotate_left(17)
    }

    fn pick_seed(&mut self) -> u64 {
        if self.settings.seed_random {
            self.entropy_seed()
        } else {
            self.settings.seed
        }
    }

    /// start a brand-new game. picks seed (random or fixed). (§3.3; Phase 5)
    fn start_new_game(&mut self) {
        let seed = self.pick_seed();
        self.start_game_with_seed(seed);
    }

    /// restart current game with **same seed** (deterministic).
    fn start_game(&mut self) {
        self.start_game_with_seed(self.current_seed);
    }

    fn start_game_with_seed(&mut self, seed: u64) {
        let variant = self.variant();
        let ruleset = self.ruleset();
        self.current_seed = seed;
        self.game = Game::new(seed, variant, self.settings.start_level, ruleset);
        self.apply_settings_to_engine();
        self.fx = Fx::default();
        self.das.clear();
        self.paused = false;
        self.score_recorded = false;
        // replay recording (Phase 5)
        self.replay = Replay::new(seed, variant, self.settings.start_level, ruleset);
        self.replay_tick_count = 0;
        self.state = AppState::Playing;
        self.sound.start_music(MusicSlot::Korobeiniki); // no asset shipped; slot
    }

    // ---------- input ----------

    fn on_key(&mut self, key: Key, pressed: bool) {
        match self.state {
            AppState::Title => {
                if pressed {
                    self.sound.play(SoundCue::MenuSelect);
                    self.state = AppState::MainMenu;
                    self.menu_index = 0;
                }
            }
            AppState::MainMenu => self.on_menu_key(key, pressed),
            AppState::Options => self.on_options_key(key, pressed),
            AppState::KeyMap => self.on_keymap_key(key, pressed),
            AppState::HighScore => {
                if pressed && (key == Key::Escape || key == Key::Q || key == Key::Enter) {
                    self.sound.play(SoundCue::MenuBack);
                    self.state = AppState::MainMenu;
                }
            }
            AppState::Help => {
                if pressed
                    && (key == Key::Escape
                        || key == Key::Q
                        || key == Key::Enter
                        || key == Key::Space)
                {
                    self.sound.play(SoundCue::MenuBack);
                    self.state = AppState::MainMenu;
                }
            }
            AppState::Playing => self.on_playing_key(key, pressed),
            AppState::NameEntry => self.on_name_entry_key(key, pressed),
            AppState::GameOver => self.on_game_over_key(key, pressed),
            AppState::Demo => {
                // any key exits demo
                if pressed {
                    self.sound.stop_music();
                    self.sound.play(SoundCue::MenuBack);
                    self.state = AppState::MainMenu;
                }
            }
        }
    }

    /// text input (for high-score name entry). app routes SDL TextInput here.
    fn on_text_input(&mut self, text: &str) {
        if self.state != AppState::NameEntry {
            return;
        }
        for ch in text.chars() {
            if (ch.is_ascii_alphanumeric() || ch == ' ')
                && self.name_input.chars().count() < persist::MAX_NAME_LEN
            {
                self.name_input.push(ch.to_ascii_uppercase());
            }
        }
    }

    fn on_game_over_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            Key::R | Key::Space | Key::Enter => self.start_game(),
            Key::Escape | Key::Q => {
                self.sound.stop_music();
                self.state = AppState::MainMenu;
            }
            _ => {}
        }
    }

    fn on_menu_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        let n = MAIN_MENU.len();
        match key {
            Key::Up => {
                self.menu_index = (self.menu_index + n - 1) % n;
                self.sound.play(SoundCue::MenuMove);
            }
            Key::Down => {
                self.menu_index = (self.menu_index + 1) % n;
                self.sound.play(SoundCue::MenuMove);
            }
            Key::Space | Key::X | Key::Enter => self.activate_menu(),
            Key::Escape => {
                self.sound.play(SoundCue::MenuBack);
                self.state = AppState::Title;
            }
            _ => {}
        }
    }

    fn activate_menu(&mut self) {
        self.sound.play(SoundCue::MenuSelect);
        match self.menu_index {
            0 => self.start_new_game(),          // NEW GAME
            1 => self.cycle_variant(),           // VARIATION §2.3.2
            2 => self.cycle_start_level(),       // START LEVEL §2.3.3
            3 => self.cycle_lock_ruleset(),      // LOCK DOWN §2.5.4
            4 => self.state = AppState::Options, // OPTIONS §2.5
            5 => {
                // KEY MAPPING §4.1
                self.state = AppState::KeyMap;
                self.remap_index = 0;
                self.remap_capture = false;
            }
            6 => self.state = AppState::HighScore, // HIGH SCORE §2.3.4
            7 => self.state = AppState::Help,      // HELP §2.6
            8 => self.start_demo(),                // DEMO
            _ => self.running = false,             // QUIT
        }
    }

    fn cycle_variant(&mut self) {
        self.settings.variant = match self.settings.variant.as_str() {
            "marathon" => "sprint",
            "sprint" => "ultra",
            _ => "marathon",
        }
        .to_string();
        self.persist_settings();
    }
    fn cycle_start_level(&mut self) {
        self.settings.start_level = if self.settings.start_level >= LEVEL_MAX {
            LEVEL_MIN
        } else {
            self.settings.start_level + 1
        };
        self.persist_settings();
    }
    fn cycle_lock_ruleset(&mut self) {
        self.settings.lock_ruleset = match self.settings.lock_ruleset.as_str() {
            "extended" => "infinite",
            "infinite" => "classic",
            _ => "extended",
        }
        .to_string();
        self.persist_settings();
    }

    // ---- options (§2.5 + Phase-5) ----

    fn on_options_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        let n = OPTIONS.len();
        match key {
            Key::Escape | Key::Q => {
                self.sound.play(SoundCue::MenuBack);
                self.state = AppState::MainMenu;
            }
            Key::Up => {
                self.options_index = (self.options_index + n - 1) % n;
                self.sound.play(SoundCue::MenuMove);
            }
            Key::Down => {
                self.options_index = (self.options_index + 1) % n;
                self.sound.play(SoundCue::MenuMove);
            }
            Key::Left => self.adjust_option(-1),
            Key::Right | Key::Space | Key::X => self.adjust_option(1),
            Key::Enter => self.activate_option(),
            _ => {}
        }
    }

    fn adjust_option(&mut self, dir: i32) {
        match self.options_index {
            0 => {
                // NEXT length 1-6 (§2.5.1)
                let cur = self.settings.next_len as i32;
                self.settings.next_len = (cur + dir).clamp(1, 6) as usize;
                self.persist_settings();
            }
            1 => self.settings.hold_enabled = !self.settings.hold_enabled,
            2 => self.settings.ghost_enabled = !self.settings.ghost_enabled,
            3 => self.settings.monochrome = !self.settings.monochrome,
            4 => self.cycle_lock_ruleset(),
            5 => {
                self.settings.goal_system = if self.settings.goal_system == "fixed" {
                    "variable".into()
                } else {
                    "fixed".into()
                }
            }
            6 => self.settings.sound.sfx_enabled = !self.settings.sound.sfx_enabled,
            7 => {
                self.settings.sound.sfx_volume =
                    SoundSettings::clamp_volume(self.settings.sound.sfx_volume as i32 + dir * 5);
            }
            8 => self.settings.sound.music_enabled = !self.settings.sound.music_enabled,
            9 => {
                self.settings.sound.music_volume =
                    SoundSettings::clamp_volume(self.settings.sound.music_volume as i32 + dir * 5);
            }
            10 => self.settings.sound.muted = !self.settings.sound.muted,
            11 => self.settings.seed_random = !self.settings.seed_random,
            12 => {
                // adjust seed (used when RANDOM SEED off). big steps with shift-like large dir.
                let step: i64 = if dir > 0 { 1 } else { -1 };
                self.settings.seed = self.settings.seed.wrapping_add(step as u64);
            }
            _ => return,
        }
        // persist + apply sound for non-cycle paths (cycle_* already persisted for 4)
        if self.options_index != 4 {
            self.persist_settings();
        }
        self.apply_sound_settings();
        // restart music state if music toggled
        if self.options_index == 8 || self.options_index == 10 {
            self.sound.stop_music();
        }
        self.sound.play(SoundCue::MenuMove);
    }

    fn activate_option(&mut self) {
        match self.options_index {
            13 => {
                // KEY MAPPING
                self.state = AppState::KeyMap;
                self.remap_index = 0;
                self.remap_capture = false;
                self.sound.play(SoundCue::MenuSelect);
            }
            _ => self.adjust_option(1),
        }
    }

    // ---- key remapping (§4.1; Phase 5) ----

    fn on_keymap_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        // capture mode: next key becomes the binding.
        if self.remap_capture {
            match key {
                Key::Escape => {
                    self.remap_capture = false;
                    self.sound.play(SoundCue::MenuBack);
                }
                _ => {
                    let target = RemapTarget::ALL[self.remap_index];
                    self.settings.keymap.set_binding(target, key);
                    self.remap_capture = false;
                    self.persist_settings();
                    self.sound.play(SoundCue::MenuSelect);
                }
            }
            return;
        }
        let n = RemapTarget::ALL.len();
        match key {
            Key::Escape | Key::Q => {
                self.sound.play(SoundCue::MenuBack);
                self.state = AppState::MainMenu;
            }
            Key::Up => {
                self.remap_index = (self.remap_index + n - 1) % n;
                self.sound.play(SoundCue::MenuMove);
            }
            Key::Down => {
                self.remap_index = (self.remap_index + 1) % n;
                self.sound.play(SoundCue::MenuMove);
            }
            Key::Enter | Key::Space | Key::X => {
                self.remap_capture = true;
                self.sound.play(SoundCue::MenuSelect);
            }
            Key::C | Key::Backspace => {
                // clear binding
                let target = RemapTarget::ALL[self.remap_index];
                self.settings.keymap.clear_binding(target);
                self.persist_settings();
            }
            Key::R => {
                // reset all to guideline defaults §4.1
                self.settings.keymap.reset_defaults();
                self.persist_settings();
                self.sound.play(SoundCue::MenuSelect);
            }
            _ => {}
        }
    }

    // ---- high-score name entry (§11.2; Phase 5) ----

    fn on_name_entry_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            Key::Enter => self.submit_name(),
            Key::Escape => {
                // skip name; record with default + exit
                self.name_input = self.settings.last_name.clone();
                self.submit_name();
            }
            Key::Backspace => {
                self.name_input.pop();
            }
            _ => {}
        }
    }

    fn submit_name(&mut self) {
        let name = persist::sanitize_name(if self.name_input.trim().is_empty() {
            &self.settings.last_name
        } else {
            &self.name_input
        });
        self.record_score_named(name.clone());
        self.settings.last_name = name;
        self.persist_settings();
        self.state = AppState::GameOver;
    }

    // ---- demo (Phase 5) ----

    fn start_demo(&mut self) {
        self.demo_seed = self.demo_seed.wrapping_add(0x9E37);
        self.demo_game = Game::new(self.demo_seed, Variant::Marathon, 1, LockRuleset::Extended);
        self.demo_timer_ms = 0;
        self.state = AppState::Demo;
        self.sound.start_music(MusicSlot::Korobeiniki);
    }

    // ---- playing ----

    fn on_playing_key(&mut self, key: Key, pressed: bool) {
        // monochrome toggle (§3.2) + mute toggle (§13.2) handled at app level
        if pressed && key == Key::M {
            self.settings.monochrome = !self.settings.monochrome;
            self.persist_settings();
            return;
        }
        match self.keymap().classify(key, pressed) {
            KeyAction::Command(cmd) => self.dispatch(cmd),
            KeyAction::SoftDrop(held) => {
                if !self.paused {
                    self.dispatch(GameCommand::SoftDrop(held));
                }
            }
            KeyAction::MoveEdge(dir, p) => {
                if let Some(cmd) = self.das.on_edge(dir, p)
                    && !self.paused
                {
                    self.dispatch(cmd);
                }
            }
            KeyAction::App(app) => match app {
                AppKey::Pause => {
                    self.paused = !self.paused;
                    if self.paused {
                        self.das.clear();
                        self.game.apply(GameCommand::SoftDrop(false)); // not recorded
                        self.sound.play(SoundCue::Pause);
                    } else {
                        self.sound.play(SoundCue::Resume);
                    }
                }
                AppKey::Restart => self.start_game(),
                AppKey::Quit => {
                    self.sound.stop_music();
                    self.state = AppState::MainMenu;
                }
            },
            KeyAction::None => {}
        }
    }

    fn dispatch(&mut self, cmd: GameCommand) {
        if self.paused {
            return;
        }
        // record for replay (Phase 5); DAS + discrete both. (replay what reaches core)
        self.replay.record(cmd);
        self.game.apply(cmd);
    }

    // ---------- core events → effects + sound ----------

    fn process_events(&mut self, evs: Vec<EngineEvent>) {
        // sound (§13.2) pure mapping + play.
        for cue in crate::sound::cues_from_events(&evs) {
            self.sound.play(cue);
        }
        for ev in evs {
            match ev {
                EngineEvent::LinesCleared { count, rows } => {
                    self.fx.clearing_rows = rows;
                    self.fx.clear_timer_ms = 180;
                    self.fx.clear_intensity = count.min(4);
                }
                EngineEvent::LockedDown { cells, .. } => {
                    self.fx.lock_flash_ms = 120;
                    self.fx.lock_flash_cells = cells;
                }
                EngineEvent::HardDropped {
                    kind,
                    facing,
                    from,
                    to,
                    ..
                } => {
                    let mut trail = Vec::new();
                    let mut y = from.y;
                    while y < to.y {
                        trail.extend(
                            kind.cells_at(facing, crate::core::types::Point::new(from.x, y)),
                        );
                        y += 1;
                    }
                    self.fx.hard_trail = trail;
                    self.fx.hard_trail_ms = 150;
                }
                EngineEvent::Scored { action, b2b, .. } => {
                    if let Some(text) = render::action_notification(action, b2b) {
                        self.fx.push_notification(text, 1200);
                    }
                }
                EngineEvent::LevelUp { level } => {
                    self.fx.level_up_ms = 800;
                    self.fx.push_notification(format!("LEVEL UP {level}"), 1000);
                }
                EngineEvent::GameOver { .. } => self.end_game("GAME OVER".to_string()),
                EngineEvent::VariantEnd { .. } => { /* handled below via game.ended */ }
                _ => {}
            }
        }
    }

    fn end_game(&mut self, reason: String) {
        if self.state != AppState::Playing {
            return;
        }
        let e = &self.game.engine;
        self.pending_score = e.score();
        self.pending_lines = e.lines_total;
        self.pending_level = e.level;
        self.pending_time = self.game.elapsed_ms;
        self.pending_reason = reason;
        if self.pending_score > self.high_score {
            self.high_score = self.pending_score;
        }
        self.score_recorded = false;
        self.sound.stop_music();
        // export replay of this session (Phase 5; robustness; ignore io error)
        let _ = self.replay.save_to_disk();
        // §11.2 name entry if it qualifies; else straight to GameOver.
        let variant_name = self.variant().name().to_string();
        if self.scores.qualifies(self.pending_score, &variant_name) {
            self.name_input = self.settings.last_name.clone();
            self.state = AppState::NameEntry;
        } else {
            self.state = AppState::GameOver;
        }
    }

    fn check_variant_end(&mut self) {
        if let Some(end) = self.game.ended {
            let reason = match end {
                VariantEndReason::MarathonComplete => "LEVEL 15 COMPLETED",
                VariantEndReason::SprintComplete => "SPRINT COMPLETE",
                VariantEndReason::UltraTimeUp => "TIME UP",
            }
            .to_string();
            self.end_game(reason);
        }
    }

    /// §11.2 record high score (once). `name` from name entry.
    fn record_score_named(&mut self, name: String) {
        if self.score_recorded {
            return;
        }
        self.score_recorded = true;
        let variant = self.variant().name().to_string();
        let entry = HighScoreEntry {
            name,
            score: self.pending_score,
            level: self.pending_level,
            lines: self.pending_lines,
            variant,
            time_ms: self.pending_time,
        };
        let _ = self.scores.insert(entry);
        let _ = self.scores.save(); // atomic
    }

    // ---------- snapshot ----------

    fn snapshot(&self) -> RenderSnapshot<'_> {
        snapshot_of(&self.game, &self.settings, self.high_score, self.paused)
    }

    fn demo_snapshot(&self) -> RenderSnapshot<'_> {
        let mut s = snapshot_of(&self.demo_game, &self.settings, 0, false);
        s.variant_name = "DEMO".into();
        s
    }
}

/// build a `RenderSnapshot` from a `Game`. pure.
fn snapshot_of<'a>(
    game: &'a Game,
    settings: &Settings,
    high_score: u64,
    paused: bool,
) -> RenderSnapshot<'a> {
    let e = &game.engine;
    RenderSnapshot {
        matrix: &e.matrix,
        active_cells: e.active_cells(),
        active_kind: e.active.map(|p| p.kind),
        ghost_cells: if settings.ghost_enabled {
            e.ghost_cells()
        } else {
            Vec::new()
        },
        hold: e.hold,
        hold_used: e.hold_used,
        next: e.next_queue(settings.next_len),
        score: e.score(),
        high_score: high_score.max(e.score()),
        level: e.level,
        lines: e.lines_total,
        goal: e.goal,
        time_ms: game.elapsed_ms,
        phase: phase_str(e.phase).to_string(),
        variant_name: game.variant.name().to_string(),
        tetrises: e.scoring.tetrises,
        t_spins: e.scoring.t_spins,
        mini_t_spins: e.scoring.mini_t_spins,
        combos: e.scoring.combo,
        b2b: e.scoring.b2b,
        b2b_count: e.scoring.b2b_count,
        pieces_placed: e.pieces_placed,
        singles: e.scoring.singles,
        doubles: e.scoring.doubles,
        triples: e.scoring.triples,
        paused,
        game_over: e.is_game_over(),
        variant_end: None,
        monochrome: settings.monochrome,
    }
}

fn phase_str(p: Phase) -> &'static str {
    match p {
        Phase::Generation => "GENERATION",
        Phase::Falling => "FALLING",
        Phase::Lock => "LOCK",
        Phase::Pattern => "PATTERN",
        Phase::Iterate => "ITERATE",
        Phase::Animate => "ANIMATE",
        Phase::Eliminate => "ELIMINATE",
        Phase::Completion => "COMPLETION",
        Phase::GameOver => "GAME OVER",
    }
}

fn sdl_to_key(k: Keycode) -> Option<Key> {
    Some(match k {
        Keycode::Left => Key::Left,
        Keycode::Right => Key::Right,
        Keycode::Down => Key::Down,
        Keycode::Space => Key::Space,
        Keycode::Up => Key::Up,
        Keycode::X => Key::X,
        Keycode::Z => Key::Z,
        Keycode::LCtrl => Key::LeftCtrl,
        Keycode::C => Key::C,
        Keycode::LShift => Key::LeftShift,
        Keycode::Kp0 => Key::Numpad0,
        Keycode::Kp2 => Key::Numpad2,
        Keycode::Kp4 => Key::Numpad4,
        Keycode::Kp5 => Key::Numpad5,
        Keycode::Kp6 => Key::Numpad6,
        Keycode::Kp8 => Key::Numpad8,
        Keycode::Kp1 => Key::Numpad1,
        Keycode::Kp3 => Key::Numpad3,
        Keycode::Kp7 => Key::Numpad7,
        Keycode::Kp9 => Key::Numpad9,
        Keycode::Escape => Key::Escape,
        Keycode::F1 => Key::F1,
        Keycode::R => Key::R,
        Keycode::Q => Key::Q,
        Keycode::M => Key::M,
        Keycode::Return => Key::Enter,
        Keycode::KpEnter => Key::Enter,
        Keycode::Backspace => Key::Backspace,
        // any other physical key -> Char (key remapping; not used by gameplay defaults).
        k => {
            let name = format!("{k:?}");
            // letters/digits: e.g. Keycode::A -> Char('A'); Keycode::Num1 -> Char('1')
            let mut ch = name.chars();
            match (ch.next(), ch.next()) {
                (Some(c), None) if c.is_ascii_alphanumeric() => Key::Char(c.to_ascii_uppercase()),
                _ if name.starts_with("Num") && name.len() == 4 => {
                    let c = name.chars().nth(3).unwrap();
                    if c.is_ascii_digit() {
                        Key::Char(c)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    })
}

fn video_text_input(sdl: &sdl2::Sdl) -> Option<sdl2::keyboard::TextInputUtil> {
    let v = sdl.video().ok()?;
    let ti = v.text_input();
    ti.start();
    Some(ti)
}

pub fn run() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("tehtrys", render::WIN_W, render::WIN_H)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl.event_pump()?;
    let timer = sdl.timer()?;

    // sound: SDL2 built-in audio + procedural synth; **fall back to silent** (robustness; headless).
    let music_dir = persist::data_dir_public().join("music");
    let sound: Box<dyn Audio> = match sdl.audio() {
        Ok(audio_sub) => match SdlAudio::open(&audio_sub, music_dir) {
            Ok(a) => Box::new(a),
            Err(_) => Box::new(NullAudio::new()),
        },
        Err(_) => Box::new(NullAudio::new()),
    };

    // enable text input for name entry. (§11.2)
    let text_input = video_text_input(&sdl);
    let _text_input_guard = text_input;

    let mut app = App::new(sound);
    app.last_ticks = timer.ticks();

    while app.running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => app.running = false,
                Event::KeyDown {
                    keycode: Some(k),
                    repeat,
                    ..
                } => {
                    if !repeat && let Some(key) = sdl_to_key(k) {
                        app.on_key(key, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    if let Some(key) = sdl_to_key(k) {
                        app.on_key(key, false);
                    }
                }
                Event::TextInput { text, .. } => app.on_text_input(&text),
                Event::Window { win_event, .. }
                    if app.state == AppState::Playing
                        && matches!(
                            win_event,
                            sdl2::event::WindowEvent::FocusLost
                                | sdl2::event::WindowEvent::Minimized
                        ) =>
                {
                    app.paused = true;
                    app.das.clear();
                    app.game.apply(GameCommand::SoftDrop(false)); // not recorded
                }
                _ => {}
            }
        }

        let now = timer.ticks();
        let mut frame_ms = now.wrapping_sub(app.last_ticks);
        app.last_ticks = now;
        frame_ms = frame_ms.min(MAX_FRAME_MS);

        app.sound.update(frame_ms);

        let playing = app.state == AppState::Playing && !app.paused;
        if playing {
            app.accumulator_ms += frame_ms;
        } else {
            app.accumulator_ms = 0;
        }

        while app.accumulator_ms >= TICK_MS {
            for cmd in app.das.tick(TICK_MS) {
                app.dispatch(cmd);
            }
            if playing {
                app.game.tick(TICK_MS);
                app.replay.advance_tick();
                app.replay_tick_count += 1;
            }
            let evs = app.game.engine.take_events();
            app.process_events(evs);
            app.check_variant_end();
            app.fx.tick(TICK_MS);
            app.accumulator_ms -= TICK_MS;
        }

        // demo auto-play (Phase 5): trivial attract mode (hard-drop each piece; restart on game over).
        if app.state == AppState::Demo {
            app.demo_timer_ms += frame_ms;
            if app.demo_timer_ms >= DEMO_DROP_MS {
                app.demo_timer_ms = 0;
                if app.demo_game.is_over() {
                    app.demo_game = Game::new(
                        app.demo_seed.wrapping_add(1),
                        Variant::Marathon,
                        1,
                        LockRuleset::Extended,
                    );
                } else {
                    app.demo_game.apply(GameCommand::HardDrop);
                    app.demo_game.engine.take_events();
                }
            }
            // advance demo time too
            app.demo_game.tick(TICK_MS);
            app.demo_game.engine.take_events();
        }

        // render per state
        match app.state {
            AppState::Title => render::draw_title(&mut canvas, app.settings.monochrome),
            AppState::MainMenu => render::draw_main_menu(
                &mut canvas,
                app.menu_index,
                &menu_values(&app.settings),
                app.settings.monochrome,
            ),
            AppState::Options => render::draw_options_v2(
                &mut canvas,
                &app.settings,
                app.options_index,
                OPTIONS,
                app.settings.monochrome,
            ),
            AppState::KeyMap => render::draw_keymap(
                &mut canvas,
                app.keymap(),
                app.remap_index,
                app.remap_capture,
                app.settings.monochrome,
            ),
            AppState::HighScore => render::draw_high_score(
                &mut canvas,
                &app.scores,
                app.variant().name(),
                app.settings.monochrome,
            ),
            AppState::Help => render::draw_help(&mut canvas, app.settings.monochrome),
            AppState::Playing => {
                let snap = app.snapshot();
                render::draw(&mut canvas, &snap, &app.fx);
            }
            AppState::NameEntry => render::draw_name_entry(
                &mut canvas,
                &app.name_input,
                app.pending_score,
                app.variant().name(),
                app.settings.monochrome,
            ),
            AppState::GameOver => render::draw_game_over(
                &mut canvas,
                app.pending_score,
                app.pending_lines,
                app.pending_level,
                app.pending_time,
                &app.pending_reason,
                app.variant().name(),
                app.high_score,
                app.current_seed,
                app.settings.monochrome,
            ),
            AppState::Demo => {
                let snap = app.demo_snapshot();
                render::draw(&mut canvas, &snap, &Fx::default());
            }
        }
    }

    app.sound.stop_music();
    app.persist_settings();
    Ok(())
}

fn menu_values(s: &Settings) -> Vec<String> {
    vec![
        String::new(),
        s.variant.clone().to_uppercase(),
        s.start_level.to_string(),
        s.lock_ruleset.to_uppercase(),
        String::new(), // OPTIONS
        String::new(), // KEY MAPPING
        String::new(), // HIGH SCORE
        String::new(), // HELP
        String::new(), // DEMO
        String::new(), // QUIT
    ]
}

pub const MAX_FRAME_MS: u32 = 250;
