//! SDL2 application shell. (flows/03 fixed timestep; flows/04 input; flows/13 pause)
//! Phase 2: window + canvas + event pump; Marathon; pause/restart/quit; minimal render.

use crate::core::constants::*;
use crate::core::engine::TetrisEngine;
use crate::core::game::{Game, Variant};
use crate::core::types::{GameCommand, LockRuleset};
use crate::input::{AppKey, DasArr, Direction, Key, KeyAction, KeyMap};
use crate::render::{self, RenderSnapshot};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// Window pixels.
const WIN_W: u32 = (render::MATRIX_PX_W + 24 + 200 + render::MARGIN * 3) as u32;
const WIN_H: u32 = (render::MATRIX_PX_H + render::MARGIN * 2) as u32;
const SEED: u64 = 0x7E77_21F5; // fixed seed Phase 2; random/seeded later.

/// Map SDL keycode → our abstract `Key`. (§4.1; remapping later)
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
}

impl App {
    fn new() -> Self {
        Self {
            game: Game::new(SEED, Variant::Marathon, LEVEL_MIN, LockRuleset::Extended),
            keymap: KeyMap::default(),
            das: DasArr::new(),
            paused: false,
            running: true,
            accumulator_ms: 0,
            last_ticks: 0,
        }
    }

    fn restart(&mut self) {
        self.game = Game::new(SEED, Variant::Marathon, LEVEL_MIN, LockRuleset::Extended);
        self.das.clear();
        self.paused = false;
    }

    fn dispatch(&mut self, cmd: GameCommand) {
        if self.paused {
            return; // ignore game input while paused. (flows/13)
        }
        self.game.apply(cmd);
    }

    fn on_key(&mut self, key: Key, pressed: bool) {
        match self.keymap.classify(key, pressed) {
            KeyAction::Command(cmd) => self.dispatch(cmd),
            KeyAction::SoftDrop(held) => {
                if !self.paused {
                    self.game.apply(GameCommand::SoftDrop(held));
                }
            }
            KeyAction::MoveEdge(dir, p) => {
                let d = match dir {
                    Direction::Left => crate::input::Direction::Left,
                    Direction::Right => crate::input::Direction::Right,
                };
                if let Some(cmd) = self.das.on_edge(d, p)
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

    fn snapshot(&self) -> RenderSnapshot<'_> {
        let e = &self.game.engine;
        RenderSnapshot {
            matrix: &e.matrix,
            active_cells: e.active_cells(),
            active_kind: e.active.map(|p| p.kind),
            ghost_cells: e.ghost_cells(),
            hold: e.hold,
            next: e.next_queue(6),
            score: e.score(),
            level: e.level,
            lines: e.lines_total,
            goal: e.goal,
            paused: self.paused,
            game_over: e.is_game_over(),
        }
    }
}

/// Run the SDL2 app. Returns on quit.
pub fn run() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("tehtrys", WIN_W, WIN_H)
        .position_centered()
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
        // 1. INPUT
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => app.running = false,
                Event::KeyDown {
                    keycode: Some(k),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        // ignore SDL auto-repeat; we run our own DAS. (§5.2)
                        if let Some(key) = sdl_to_key(k) {
                            app.on_key(key, true);
                        }
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
                        // auto-pause + clear held keys. (flows/04)
                        app.paused = true;
                        app.das.clear();
                        app.game.apply(GameCommand::SoftDrop(false));
                    }
                }
                _ => {}
            }
        }

        // 2. TIME
        let now = timer.ticks();
        let mut frame_ms = now.wrapping_sub(app.last_ticks);
        app.last_ticks = now;
        frame_ms = frame_ms.min(MAX_FRAME_MS); // clamp "spiral of death"
        if !app.paused {
            app.accumulator_ms += frame_ms;
        } else {
            app.accumulator_ms = 0;
        }

        // 3. FIXED-STEP UPDATE
        while app.accumulator_ms >= TICK_MS {
            // DAS/ARR emissions
            let emits = app.das.tick(TICK_MS);
            for cmd in emits {
                app.dispatch(cmd);
            }
            if !app.paused {
                app.game.tick(TICK_MS);
            }
            app.accumulator_ms -= TICK_MS;
        }

        // 4. RENDER
        let snap = app.snapshot();
        render::draw(&mut canvas, &snap);
    }

    Ok(())
}

/// fixed timestep. (flows/03)
pub const _FIXED_TICK_MS: u32 = TICK_MS;
/// clamp frame time. (flows/03)
pub const MAX_FRAME_MS: u32 = 250;

// re-export for clarity
#[allow(dead_code)]
fn _assert_engine(_e: &TetrisEngine) {}
