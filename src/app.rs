//! SDL2 application shell. (flows/14 state machine; flows/03/04/12/13; §2 screens;
//! §11.2 high score; §2.5 options) Phase 4.

use crate::core::constants::*;
use crate::core::engine::Phase;
use crate::core::game::{Game, Variant};
use crate::core::levels::GoalSystem;
use crate::core::types::{EngineEvent, GameCommand, LockRuleset, VariantEndReason};
use crate::input::{AppKey, DasArr, Key, KeyAction, KeyMap};
use crate::persist::{HighScoreEntry, HighScoreTable, Settings};
use crate::render::{self, Fx, RenderSnapshot};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// Application state. (flows/14)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AppState {
    Title,     // §2.2
    MainMenu,  // §2.3
    Options,   // §2.5
    HighScore, // §2.3.4/§11.2
    Playing,   // §2.4 + pause
    GameOver,  // §11
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
    "HIGH SCORE",
    "QUIT",
];

struct App {
    state: AppState,
    game: Game,
    settings: Settings,
    keymap: KeyMap,
    das: DasArr,
    paused: bool,
    running: bool,
    accumulator_ms: u32,
    last_ticks: u32,
    fx: Fx,
    high_score: u64,
    scores: HighScoreTable,
    menu_index: usize,
    score_recorded: bool,
    // pending game-over info
    pending_score: u64,
    pending_lines: u32,
    pending_level: u32,
    pending_time: u64,
    pending_reason: String,
}

impl App {
    fn new() -> Self {
        let settings = Settings::load();
        let variant = variant_from(&settings.variant);
        let mut app = Self {
            state: AppState::Title,
            settings: settings.clone(),
            game: Game::new(
                0x7E77_21F5,
                variant,
                settings.start_level,
                ruleset_from(&settings.lock_ruleset),
            ),
            keymap: KeyMap::default(),
            das: DasArr::new(),
            paused: false,
            running: true,
            accumulator_ms: 0,
            last_ticks: 0,
            fx: Fx::default(),
            high_score: 0,
            scores: HighScoreTable::load(),
            menu_index: 0,
            score_recorded: false,
            pending_score: 0,
            pending_lines: 0,
            pending_level: 1,
            pending_time: 0,
            pending_reason: String::new(),
        };
        app.apply_settings_to_engine();
        app
    }

    // convenience
    fn variant(&self) -> Variant {
        variant_from(&self.settings.variant)
    }

    fn apply_settings_to_engine(&mut self) {
        self.game.engine.hold_enabled = self.settings.hold_enabled;
        let gs = goal_from(&self.settings.goal_system);
        self.game.engine.goal_system = gs;
        self.game.engine.goal = crate::core::levels::starting_goal(gs, self.game.engine.level);
    }

    fn start_game(&mut self) {
        let variant = self.variant();
        let seed = 0x7E77_21F5; // fixed; seeded RNG UI later. (replay Phase 5)
        self.game = Game::new(
            seed,
            variant,
            self.settings.start_level,
            ruleset_from(&self.settings.lock_ruleset),
        );
        self.apply_settings_to_engine();
        self.fx = Fx::default();
        self.das.clear();
        self.paused = false;
        self.score_recorded = false;
        self.state = AppState::Playing;
    }

    fn persist_settings(&self) {
        let _ = self.settings.save();
    }

    // ---------- input ----------

    fn on_key(&mut self, key: Key, pressed: bool) {
        match self.state {
            AppState::Title => {
                if pressed {
                    self.state = AppState::MainMenu;
                    self.menu_index = 0;
                }
            }
            AppState::MainMenu => self.on_menu_key(key, pressed),
            AppState::Options => self.on_options_key(key, pressed),
            AppState::HighScore => {
                if pressed && (key == Key::Escape || key == Key::Q) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::Playing => self.on_playing_key(key, pressed),
            AppState::GameOver => {
                if pressed {
                    match key {
                        Key::R | Key::Space => self.start_game(),
                        Key::Escape | Key::Q => self.state = AppState::MainMenu,
                        _ => {}
                    }
                }
            }
        }
    }

    fn on_menu_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        let n = MAIN_MENU.len();
        match key {
            Key::Up => self.menu_index = (self.menu_index + n - 1) % n,
            Key::Down => self.menu_index = (self.menu_index + 1) % n,
            Key::Space | Key::X => self.activate_menu(),
            Key::Escape => self.state = AppState::Title,
            _ => {}
        }
    }

    fn activate_menu(&mut self) {
        match self.menu_index {
            0 => self.start_game(),                // NEW GAME
            1 => self.cycle_variant(),             // VARIATION §2.3.2
            2 => self.cycle_start_level(),         // START LEVEL §2.3.3
            3 => self.cycle_lock_ruleset(),        // LOCK DOWN §2.5.4
            4 => self.state = AppState::Options,   // OPTIONS §2.5
            5 => self.state = AppState::HighScore, // HIGH SCORE §2.3.4
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

    fn on_options_key(&mut self, key: Key, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            Key::Escape => self.state = AppState::MainMenu,
            Key::Left | Key::Right => {
                // toggle options with left/right on a simple list
            }
            _ => {}
        }
        // simple option toggles via keys:
        match key {
            Key::Down => {
                // NEXT length. (§2.5.1 1-6)
                self.settings.next_len = if self.settings.next_len >= 6 {
                    1
                } else {
                    self.settings.next_len + 1
                };
                self.persist_settings();
            }
            Key::Up => {
                self.settings.next_len = if self.settings.next_len <= 1 {
                    6
                } else {
                    self.settings.next_len - 1
                };
                self.persist_settings();
            }
            Key::C => {
                self.settings.hold_enabled = !self.settings.hold_enabled;
                self.persist_settings();
            }
            Key::X => {
                self.settings.ghost_enabled = !self.settings.ghost_enabled;
                self.persist_settings();
            }
            Key::M => {
                self.settings.monochrome = !self.settings.monochrome;
                self.persist_settings();
            }
            Key::Space => {
                self.settings.goal_system = if self.settings.goal_system == "fixed" {
                    "variable".into()
                } else {
                    "fixed".into()
                };
                self.persist_settings();
            }
            _ => {}
        }
    }

    fn on_playing_key(&mut self, key: Key, pressed: bool) {
        // monochrome toggle
        if key == Key::M && pressed {
            self.settings.monochrome = !self.settings.monochrome;
            self.persist_settings();
            return;
        }
        match self.keymap.classify(key, pressed) {
            KeyAction::Command(cmd) => self.dispatch(cmd),
            KeyAction::SoftDrop(held) => {
                if !self.paused {
                    self.game.apply(GameCommand::SoftDrop(held));
                }
            }
            KeyAction::MoveEdge(dir, p) => {
                if let Some(cmd) = self.das.on_edge(dir, p)
                    && !self.paused
                {
                    self.game.apply(cmd);
                }
            }
            KeyAction::App(app) => match app {
                AppKey::Pause => {
                    self.paused = !self.paused;
                    if self.paused {
                        self.das.clear();
                        self.game.apply(GameCommand::SoftDrop(false));
                    }
                }
                AppKey::Restart => self.start_game(),
                AppKey::Quit => self.state = AppState::MainMenu,
            },
            KeyAction::None => {}
        }
    }

    fn dispatch(&mut self, cmd: GameCommand) {
        if self.paused {
            return;
        }
        self.game.apply(cmd);
    }

    // ---------- core events → effects ----------

    fn process_events(&mut self, evs: Vec<EngineEvent>) {
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
        self.state = AppState::GameOver;
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

    /// §11.2 record high score (once). name defaults; full name entry UI later.
    fn record_score(&mut self) {
        if self.score_recorded {
            return;
        }
        self.score_recorded = true;
        let variant = self.variant().name().to_string();
        let entry = HighScoreEntry {
            name: "YOU".to_string(),
            score: self.pending_score,
            level: self.pending_level,
            lines: self.pending_lines,
            variant,
            time_ms: self.pending_time,
        };
        let _ = self.scores.insert(entry);
        let _ = self.scores.save();
    }

    // ---------- snapshot ----------

    fn snapshot(&self) -> RenderSnapshot<'_> {
        let e = &self.game.engine;
        RenderSnapshot {
            matrix: &e.matrix,
            active_cells: e.active_cells(),
            active_kind: e.active.map(|p| p.kind),
            ghost_cells: if self.settings.ghost_enabled {
                e.ghost_cells()
            } else {
                Vec::new()
            },
            hold: e.hold,
            hold_used: e.hold_used,
            next: e.next_queue(self.settings.next_len),
            score: e.score(),
            high_score: self.high_score.max(e.score()),
            level: e.level,
            lines: e.lines_total,
            goal: e.goal,
            time_ms: self.game.elapsed_ms,
            phase: phase_str(e.phase).to_string(),
            variant_name: self.variant().name().to_string(),
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
            paused: self.paused,
            game_over: e.is_game_over(),
            variant_end: None,
            monochrome: self.settings.monochrome,
        }
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
        _ => return None,
    })
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

    let mut app = App::new();
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
                    app.game.apply(GameCommand::SoftDrop(false));
                }
                _ => {}
            }
        }

        let now = timer.ticks();
        let mut frame_ms = now.wrapping_sub(app.last_ticks);
        app.last_ticks = now;
        frame_ms = frame_ms.min(MAX_FRAME_MS);

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
            }
            let evs = app.game.engine.take_events();
            app.process_events(evs);
            app.check_variant_end();
            app.fx.tick(TICK_MS);
            app.accumulator_ms -= TICK_MS;
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
            AppState::Options => {
                render::draw_options(&mut canvas, &app.settings, app.settings.monochrome)
            }
            AppState::HighScore => render::draw_high_score(
                &mut canvas,
                &app.scores,
                app.variant().name(),
                app.settings.monochrome,
            ),
            AppState::Playing => {
                let snap = app.snapshot();
                render::draw(&mut canvas, &snap, &app.fx);
            }
            AppState::GameOver => {
                app.record_score();
                render::draw_game_over(
                    &mut canvas,
                    app.pending_score,
                    app.pending_lines,
                    app.pending_level,
                    app.pending_time,
                    &app.pending_reason,
                    app.variant().name(),
                    app.high_score,
                    app.settings.monochrome,
                );
            }
        }
    }

    app.persist_settings();
    Ok(())
}

fn menu_values(s: &Settings) -> Vec<String> {
    vec![
        String::new(),
        s.variant.clone().to_uppercase(),
        s.start_level.to_string(),
        s.lock_ruleset.to_uppercase(),
        String::new(),
        String::new(),
        String::new(),
    ]
}

pub const MAX_FRAME_MS: u32 = 250;
