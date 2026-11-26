//! SDL2 application shell. (flows/03 fixed timestep; flows/04 input; flows/13 pause;
//! flows/12 render; §13 effects) Phase 3: full presentation.

use crate::core::constants::*;
use crate::core::engine::{Phase, TetrisEngine};
use crate::core::game::{Game, Variant};
use crate::core::types::{EngineEvent, GameCommand, LockRuleset, VariantEndReason};
use crate::input::{AppKey, DasArr, Key, KeyAction, KeyMap};
use crate::render::{self, Fx, RenderSnapshot};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const SEED: u64 = 0x7E77_21F5; // fixed Phase 2/3; random/seeded replay later.

/// Map SDL keycode → abstract `Key`. (§4.1; remapping later)
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

struct App {
    game: Game,
    keymap: KeyMap,
    das: DasArr,
    paused: bool,
    running: bool,
    accumulator_ms: u32,
    last_ticks: u32,
    fx: Fx,
    high_score: u64,
    monochrome: bool,
    variant_name: String,
}

impl App {
    fn new() -> Self {
        let variant = Variant::Marathon;
        Self {
            variant_name: variant.name().to_string(),
            game: Game::new(SEED, variant, LEVEL_MIN, LockRuleset::Extended),
            keymap: KeyMap::default(),
            das: DasArr::new(),
            paused: false,
            running: true,
            accumulator_ms: 0,
            last_ticks: 0,
            fx: Fx::default(),
            high_score: 0,
            monochrome: false,
        }
    }

    fn restart(&mut self) {
        let variant = Variant::Marathon;
        self.variant_name = variant.name().to_string();
        self.game = Game::new(SEED, variant, LEVEL_MIN, LockRuleset::Extended);
        self.das.clear();
        self.paused = false;
        self.fx = Fx::default();
        // high_score retained across restart. (§11.2)
    }

    fn dispatch(&mut self, cmd: GameCommand) {
        if self.paused {
            return;
        }
        self.game.apply(cmd);
    }

    fn on_key(&mut self, key: Key, pressed: bool) {
        // monochrome toggle. (§3.2; not in keymap defaults)
        if key == Key::M && pressed {
            self.monochrome = !self.monochrome;
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
                AppKey::Restart => self.restart(),
                AppKey::Quit => self.running = false,
            },
            KeyAction::None => {}
        }
    }

    /// Consume core events → notifications + effects. (§13.1; flows/12)
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
                EngineEvent::Scored { action, b2b, .. } => {
                    if let Some(text) = render::action_notification(action, b2b) {
                        self.fx.push_notification(text, 1200);
                    }
                }
                EngineEvent::HardDropped {
                    kind,
                    facing,
                    from,
                    to,
                    ..
                } => {
                    // hard-drop trail. (§13.1.2) cells from `from`..`to`
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
                EngineEvent::TSpin { .. } => { /* notification comes from Scored action */ }
                EngineEvent::LevelUp { level } => {
                    self.fx.level_up_ms = 800;
                    self.fx.push_notification(format!("LEVEL UP {level}"), 1000);
                }
                EngineEvent::GameOver { reason } => {
                    self.fx.game_over_text = Some(format!("{reason:?}"));
                    if self.game.score() > self.high_score {
                        self.high_score = self.game.score();
                    }
                }
                _ => {}
            }
        }
    }

    fn snapshot(&self) -> RenderSnapshot<'_> {
        let e = &self.game.engine;
        let variant_end = match self.game.ended {
            Some(VariantEndReason::MarathonComplete) => Some("LEVEL 15 COMPLETED".to_string()),
            Some(VariantEndReason::SprintComplete) => Some("SPRINT COMPLETE".to_string()),
            Some(VariantEndReason::UltraTimeUp) => Some("TIME UP".to_string()),
            None => None,
        };
        RenderSnapshot {
            matrix: &e.matrix,
            active_cells: e.active_cells(),
            active_kind: e.active.map(|p| p.kind),
            ghost_cells: e.ghost_cells(),
            hold: e.hold,
            hold_used: e.hold_used,
            next: e.next_queue(6),
            score: e.score(),
            high_score: self.high_score.max(e.score()),
            level: e.level,
            lines: e.lines_total,
            goal: e.goal,
            time_ms: self.game.elapsed_ms,
            phase: phase_str(e.phase).to_string(),
            variant_name: self.variant_name.clone(),
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
            variant_end,
            monochrome: self.monochrome,
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
                Event::Window { win_event, .. } => {
                    if matches!(
                        win_event,
                        sdl2::event::WindowEvent::FocusLost | sdl2::event::WindowEvent::Minimized
                    ) {
                        app.paused = true;
                        app.das.clear();
                        app.game.apply(GameCommand::SoftDrop(false));
                    }
                }
                _ => {}
            }
        }

        let now = timer.ticks();
        let mut frame_ms = now.wrapping_sub(app.last_ticks);
        app.last_ticks = now;
        frame_ms = frame_ms.min(MAX_FRAME_MS);
        if app.paused {
            app.accumulator_ms = 0;
        } else {
            app.accumulator_ms += frame_ms;
        }

        while app.accumulator_ms >= TICK_MS {
            for cmd in app.das.tick(TICK_MS) {
                app.dispatch(cmd);
            }
            if !app.paused {
                app.game.tick(TICK_MS);
            }
            let evs = app.game.engine.take_events();
            app.process_events(evs);
            app.fx.tick(TICK_MS);
            app.accumulator_ms -= TICK_MS;
        }

        let snap = app.snapshot();
        render::draw(&mut canvas, &snap, &app.fx);
    }

    Ok(())
}

pub const MAX_FRAME_MS: u32 = 250;

// keep TetrisEngine referenced
#[allow(dead_code)]
fn _assert_engine(_e: &TetrisEngine) {}
