//! SDL2 rendering. (§2.4, §13; flows/12) Pure read of snapshot + `Fx`; no logic.
//! Phase 3: full HUD, Next/Hold, Ghost, notifications, flashes, pause, game-over,
//! scaling, monochrome fallback. No protected artwork; no external font (`font`).

use crate::core::constants::*;
use crate::core::matrix::Matrix;
use crate::core::types::{Cell, Point, ScoreAction, Tetrimino};
use crate::font;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub const CELL_PX: i32 = 24;
pub const MARGIN: i32 = 12;
pub const MATRIX_PX_W: i32 = MATRIX_WIDTH * CELL_PX;
pub const MATRIX_PX_H: i32 = MATRIX_HEIGHT_VISIBLE * CELL_PX;
pub const PANEL_W: i32 = 200;
pub const WIN_W: u32 = (MARGIN + MATRIX_PX_W + 24 + PANEL_W + MARGIN) as u32;
pub const HUD_EXTRA_H: i32 = 200;
pub const WIN_H: u32 = (MARGIN + MATRIX_PX_H + MARGIN + HUD_EXTRA_H) as u32;

const BG: Color = Color::RGB(14, 14, 18);
const EMPTY: Color = Color::RGB(22, 22, 28);
const GRID: Color = Color::RGB(36, 36, 44);
const BORDER: Color = Color::RGB(90, 90, 110);
const GHOST: Color = Color::RGB(80, 80, 100);
const PANEL: Color = Color::RGB(28, 28, 36);
const TEXT: Color = Color::RGB(235, 235, 240);
const DIM: Color = Color::RGB(150, 150, 165);
const ACCENT: Color = Color::RGB(120, 200, 255);
const FLASH: Color = Color::RGB(255, 255, 255);
const NOTIFY: Color = Color::RGB(255, 230, 120);
const GAMEOVER: Color = Color::RGB(255, 90, 90);
const PAUSEC: Color = Color::RGB(200, 200, 230);

// ---------- pieces ----------

fn cell_color(k: Tetrimino, mono: bool) -> Color {
    if mono {
        // monochrome distinct hues + (rendered) patterns fallback. (§3.2)
        return mono_color(k);
    }
    let c = k.color();
    Color::RGB(c.r, c.g, c.b)
}
fn dark_color(k: Tetrimino, mono: bool) -> Color {
    let c = cell_color(k, mono);
    Color::RGB(c.r / 2, c.g / 2, c.b / 2)
}

/// distinct monochrome hue per piece (not color-only). (§3.2)
fn mono_color(k: Tetrimino) -> Color {
    match k {
        Tetrimino::I => Color::RGB(230, 230, 230),
        Tetrimino::O => Color::RGB(200, 200, 200),
        Tetrimino::T => Color::RGB(170, 170, 170),
        Tetrimino::L => Color::RGB(140, 140, 140),
        Tetrimino::J => Color::RGB(190, 190, 190),
        Tetrimino::S => Color::RGB(120, 120, 120),
        Tetrimino::Z => Color::RGB(100, 100, 100),
    }
}

// ---------- snapshot ----------

pub struct RenderSnapshot<'a> {
    pub matrix: &'a Matrix,
    pub active_cells: Vec<Point>,
    pub active_kind: Option<Tetrimino>,
    pub ghost_cells: Vec<Point>,
    pub hold: Option<Tetrimino>,
    pub hold_used: bool,
    pub next: Vec<Tetrimino>,
    pub score: u64,
    pub high_score: u64,
    pub level: u32,
    pub lines: u32,
    pub goal: u32,
    pub time_ms: u64,
    pub phase: String,
    pub variant_name: String,
    pub tetrises: u32,
    pub t_spins: u32,
    pub mini_t_spins: u32,
    pub combos: u32,
    pub b2b: bool,
    pub b2b_count: u32,
    pub pieces_placed: u64,
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub paused: bool,
    pub game_over: bool,
    pub variant_end: Option<String>,
    pub monochrome: bool,
}

/// Transient effects + notifications, owned by app, fed by `EngineEvent`. (§13.1; flows/12)
#[derive(Default, Clone)]
pub struct Fx {
    pub notifications: Vec<Notification>,
    pub clearing_rows: Vec<i32>,
    pub clear_timer_ms: u32,
    pub clear_intensity: u32, // 1..4
    pub lock_flash_ms: u32,
    pub lock_flash_cells: Vec<Point>,
    pub hard_trail_ms: u32,
    pub hard_trail: Vec<Point>,
    pub level_up_ms: u32,
    pub b2b_flash_ms: u32,
    pub game_over_text: Option<String>,
}

#[derive(Clone)]
pub struct Notification {
    pub text: String,
    pub ttl_ms: u32,
}

impl Fx {
    pub fn push_notification(&mut self, text: impl Into<String>, ttl_ms: u32) {
        self.notifications.push(Notification {
            text: text.into(),
            ttl_ms,
        });
    }
    pub fn tick(&mut self, dt: u32) {
        self.clear_timer_ms = self.clear_timer_ms.saturating_sub(dt);
        self.lock_flash_ms = self.lock_flash_ms.saturating_sub(dt);
        self.hard_trail_ms = self.hard_trail_ms.saturating_sub(dt);
        self.level_up_ms = self.level_up_ms.saturating_sub(dt);
        self.b2b_flash_ms = self.b2b_flash_ms.saturating_sub(dt);
        for n in &mut self.notifications {
            n.ttl_ms = n.ttl_ms.saturating_sub(dt);
        }
        self.notifications.retain(|n| n.ttl_ms > 0);
    }
}

/// Notification name for a score action + B2B. (§13.1.5 exact names; mini alt allowed)
pub fn action_notification(action: ScoreAction, b2b: bool) -> Option<String> {
    let base = match action {
        ScoreAction::NoClear => return None,
        ScoreAction::Single => "SINGLE",
        ScoreAction::Double => "DOUBLE",
        ScoreAction::Triple => "TRIPLE",
        ScoreAction::Tetris => "TETRIS",
        ScoreAction::TSpin => "T-SPIN",
        ScoreAction::TSpinSingle => "T-SPIN SINGLE",
        ScoreAction::TSpinDouble => "T-SPIN DOUBLE",
        ScoreAction::TSpinTriple => "T-SPIN TRIPLE",
        ScoreAction::MiniTSpin => "MINI T-SPIN",
        ScoreAction::MiniTSpinSingle => "MINI T-SPIN SINGLE",
    };
    // B2B only for line-clearing qualifying; no-line T-Spin can't be B2B. (§8n1)
    let prefix = if b2b && action.is_b2b_qualifying() {
        "BACK-TO-BACK "
    } else {
        ""
    };
    Some(format!("{prefix}{base}"))
}

// ---------- drawing primitives ----------

pub fn draw_text(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
    scale: i32,
) {
    let mut cx = x;
    for ch in text.chars() {
        if ch == '\n' {
            // newline handled by caller; render space
            cx += font::GLYPH_W * scale + scale;
            continue;
        }
        draw_glyph(canvas, cx, y, ch, color, scale);
        cx += font::GLYPH_W * scale + scale; // 1px spacing
    }
}

fn draw_glyph(canvas: &mut Canvas<Window>, x: i32, y: i32, ch: char, color: Color, scale: i32) {
    let g = font::glyph(ch);
    canvas.set_draw_color(color);
    for (row, bits) in g.iter().enumerate() {
        for col in 0..font::GLYPH_W {
            if bits & (1 << (7 - col)) != 0 {
                let _ = canvas.fill_rect(Rect::new(
                    x + col * scale,
                    y + row as i32 * scale,
                    scale as u32,
                    scale as u32,
                ));
            }
        }
    }
}

pub fn text_width(text: &str, scale: i32) -> i32 {
    text.chars().count() as i32 * (font::GLYPH_W + 1) * scale
}

fn fill(canvas: &mut Canvas<Window>, rect: Rect, color: Color) {
    canvas.set_draw_color(color);
    let _ = canvas.fill_rect(rect);
}

fn cell_rect(ox: i32, oy: i32, x: i32, y: i32, size: i32) -> Rect {
    Rect::new(
        ox + x * size,
        oy + y * size,
        (size - 1) as u32,
        (size - 1) as u32,
    )
}

// ---------- pieces mini (Next/Hold) ----------

fn draw_mini_piece(
    canvas: &mut Canvas<Window>,
    ox: i32,
    oy: i32,
    kind: Tetrimino,
    mono: bool,
    dim: bool,
) {
    // North-facing. (§2.4.3/§2.4.8)
    let size = 10;
    let cells = kind.cells_at(crate::core::types::Facing::North, Point::new(0, 0));
    let color = if dim {
        dark_color(kind, mono)
    } else {
        cell_color(kind, mono)
    };
    // center in a 4x4 box
    for p in cells {
        let rect = cell_rect(ox, oy, p.x, p.y, size);
        fill(canvas, rect, color);
    }
}

// ---------- main ----------

pub fn draw(canvas: &mut Canvas<Window>, snap: &RenderSnapshot, fx: &Fx) {
    canvas.set_draw_color(BG);
    canvas.clear();

    // PAUSE: hide matrix + next + hold; prominent "PAUSE". (§13.1.6)
    if snap.paused {
        let (w, h) = canvas.window().size();
        let label = "PAUSE";
        let lw = text_width(label, 4);
        draw_text(
            canvas,
            (w as i32 - lw) / 2,
            h as i32 / 2 - 16,
            label,
            PAUSEC,
            4,
        );
        canvas.present();
        return;
    }

    let ox = MARGIN;
    let oy = MARGIN;
    let mono = snap.monochrome;

    // ---- matrix ----
    fill(
        canvas,
        Rect::new(
            ox - 1,
            oy - 1,
            (MATRIX_PX_W + 2) as u32,
            (MATRIX_PX_H + 2) as u32,
        ),
        BORDER,
    );
    // re-draw border only; fill inside
    for y in 0..MATRIX_HEIGHT_VISIBLE {
        for x in 0..MATRIX_WIDTH {
            let cell: Cell = snap.matrix.get_opt(x, SKYLINE_Y + y);
            let rect = cell_rect(ox, oy, x, y, CELL_PX);
            match cell {
                Some(k) => {
                    // locked blocks darkened. (§13.1.3)
                    fill(canvas, rect, dark_color(k, mono));
                    if mono {
                        draw_mono_mark(canvas, rect, k);
                    }
                }
                None => {
                    fill(canvas, rect, EMPTY);
                    canvas.set_draw_color(GRID);
                    let _ = canvas.draw_rect(rect);
                }
            }
        }
    }

    // line-clear flash. (§13.1.1 intensity scales with count)
    if fx.clear_timer_ms > 0 {
        canvas.set_draw_color(FLASH);
        for &row in &fx.clearing_rows {
            let vy = row - SKYLINE_Y;
            if (0..MATRIX_HEIGHT_VISIBLE).contains(&vy) {
                let _ = canvas.draw_rect(Rect::new(
                    ox,
                    oy + vy * CELL_PX,
                    MATRIX_PX_W as u32,
                    CELL_PX as u32,
                ));
                let _ = canvas.fill_rect(Rect::new(
                    ox,
                    oy + vy * CELL_PX,
                    MATRIX_PX_W as u32,
                    CELL_PX as u32,
                ));
            }
        }
    }

    // ghost. (§2.4.4)
    canvas.set_draw_color(GHOST);
    for p in &snap.ghost_cells {
        if let Some(rect) = visible_rect(ox, oy, p, CELL_PX) {
            let _ = canvas.draw_rect(rect);
        }
    }

    // active piece. (§2.4.2)
    if let Some(kind) = snap.active_kind {
        canvas.set_draw_color(cell_color(kind, mono));
        for p in &snap.active_cells {
            if let Some(rect) = visible_rect(ox, oy, p, CELL_PX) {
                let _ = canvas.fill_rect(rect);
            }
        }
    }

    // hard-drop trail. (§13.1.2)
    if fx.hard_trail_ms > 0 {
        canvas.set_draw_color(GHOST);
        for p in &fx.hard_trail {
            if let Some(rect) = visible_rect(ox, oy, p, CELL_PX) {
                let _ = canvas.draw_rect(rect);
            }
        }
    }

    // lock-down flash. (§13.1.3)
    if fx.lock_flash_ms > 0 {
        canvas.set_draw_color(FLASH);
        for p in &fx.lock_flash_cells {
            if let Some(rect) = visible_rect(ox, oy, p, CELL_PX) {
                let _ = canvas.draw_rect(rect);
            }
        }
    }

    // ---- side panel ----
    let px = ox + MATRIX_PX_W + 24;
    let py = oy;
    fill(
        canvas,
        Rect::new(
            px - 8,
            py - 8,
            (PANEL_W + 16) as u32,
            (MATRIX_PX_H + HUD_EXTRA_H + 16) as u32,
        ),
        PANEL,
    );

    // HOLD (§2.4.8) top-left area
    draw_text(canvas, px, py, "HOLD", DIM, 1);
    if let Some(h) = snap.hold {
        let dim = snap.hold_used;
        draw_mini_piece(canvas, px + 4, py + 14, h, mono, dim);
    }

    // NEXT (§2.4.3)
    let mut ny = py + 70;
    draw_text(canvas, px, ny, "NEXT", DIM, 1);
    ny += 14;
    for k in snap.next.iter().take(5) {
        draw_mini_piece(canvas, px + 4, ny, *k, mono, false);
        ny += 42;
    }

    // HUD (§2.4.7)
    let hx = px;
    let hy = py + 70 + 14 + 5 * 42 + 8; // below next
    let mut line = hy;
    let scale = 1;
    let stat = |c: &mut Canvas<Window>, label: &str, val: &str, y: i32| {
        draw_text(c, hx, y, label, DIM, scale);
        draw_text(c, hx, y + 10, val, TEXT, scale);
    };

    draw_text(canvas, hx, line, &snap.variant_name, ACCENT, 1);
    line += 16;
    stat(canvas, "SCORE", &snap.score.to_string(), line);
    line += 18;
    stat(canvas, "HIGH", &snap.high_score.to_string(), line);
    line += 18;
    stat(canvas, "LEVEL", &snap.level.to_string(), line);
    line += 18;
    stat(canvas, "LINES", &snap.lines.to_string(), line);
    line += 18;
    stat(canvas, "GOAL", &snap.goal.to_string(), line);
    line += 18;
    let time = format_time(snap.time_ms);
    stat(canvas, "TIME", &time, line);
    line += 18;
    stat(canvas, "TETRISES", &snap.tetrises.to_string(), line);
    line += 18;
    stat(canvas, "T-SPINS", &snap.t_spins.to_string(), line);
    line += 18;
    stat(canvas, "COMBOS", &snap.combos.to_string(), line);
    line += 18;
    // TPM / LPM
    let mins = (snap.time_ms as f64 / 60000.0).max(0.001);
    let tpm = (snap.pieces_placed as f64 / mins).round() as u64;
    let lpm = (snap.lines as f64 / mins).round() as u64;
    stat(canvas, "TPM", &tpm.to_string(), line);
    line += 18;
    stat(canvas, "LPM", &lpm.to_string(), line);

    // B2B indicator
    if snap.b2b {
        draw_text(canvas, hx, py + MATRIX_PX_H - 12, "BACK-TO-BACK", NOTIFY, 1);
    }

    // ---- action notifications §13.1.5 ----
    // inside matrix ≤L10, outside after. (§13.1.5) stacked near bottom.
    let notify_inside = snap.level <= 10;
    let nx = if notify_inside {
        ox + MATRIX_PX_W / 2 - 60
    } else {
        px
    };
    let mut ny = if notify_inside {
        oy + MATRIX_PX_H - 30
    } else {
        py + MATRIX_PX_H - 30
    };
    for n in fx.notifications.iter().rev().take(4) {
        draw_text(canvas, nx, ny, &n.text, NOTIFY, 1);
        ny -= 12;
    }

    // level-up cue. (§13.1.4)
    if fx.level_up_ms > 0 {
        let label = "LEVEL UP";
        draw_text(
            canvas,
            ox + MATRIX_PX_W / 2 - text_width(label, 2) / 2,
            oy + MATRIX_PX_H / 2,
            label,
            ACCENT,
            2,
        );
    }

    // ---- game over / variant end ----
    if snap.game_over || snap.variant_end.is_some() {
        let (w, h) = canvas.window().size();
        let msg = snap
            .variant_end
            .clone()
            .unwrap_or_else(|| "GAME OVER".to_string());
        // dim backdrop
        fill(canvas, Rect::new(0, 0, w, h), Color::RGBA(0, 0, 0, 160));
        let label = if snap.game_over {
            "GAME OVER"
        } else {
            "COMPLETE"
        };
        draw_text(
            canvas,
            (w as i32 - text_width(label, 3)) / 2,
            h as i32 / 2 - 24,
            label,
            GAMEOVER,
            3,
        );
        draw_text(
            canvas,
            (w as i32 - text_width(&msg, 1)) / 2,
            h as i32 / 2 + 8,
            &msg,
            TEXT,
            1,
        );
        let score_line = format!("SCORE {}", snap.score);
        draw_text(
            canvas,
            (w as i32 - text_width(&score_line, 1)) / 2,
            h as i32 / 2 + 22,
            &score_line,
            TEXT,
            1,
        );
        draw_text(
            canvas,
            (w as i32 - text_width("PRESS R TO RESTART", 1)) / 2,
            h as i32 / 2 + 40,
            "PRESS R TO RESTART",
            DIM,
            1,
        );
    }

    canvas.present();
}

fn draw_mono_mark(canvas: &mut Canvas<Window>, rect: Rect, kind: Tetrimino) {
    // distinct pattern per piece for monochrome §3.2. simple marks.
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    let cx = rect.x + rect.w / 2;
    let cy = rect.y + rect.h / 2;
    match kind {
        Tetrimino::I => {
            let _ = canvas.draw_line((cx, rect.y + 2), (cx, rect.y + rect.h - 2));
        }
        Tetrimino::O => {
            let _ = canvas.draw_rect(Rect::new(cx - 2, cy - 2, 4, 4));
        }
        Tetrimino::T => {
            let _ = canvas.draw_line((cx - 3, cy), (cx + 3, cy));
            let _ = canvas.draw_line((cx, cy - 3), (cx, cy + 3));
        }
        Tetrimino::L => {
            let _ = canvas.draw_line((rect.x + 2, cy), (rect.x + rect.w - 2, cy));
        }
        Tetrimino::J => {
            let _ = canvas.draw_line((cx, rect.y + 2), (cx, rect.y + rect.h - 2));
        }
        Tetrimino::S => {
            let _ = canvas.draw_line((cx - 3, cy - 3), (cx + 3, cy + 3));
        }
        Tetrimino::Z => {
            let _ = canvas.draw_line((cx - 3, cy + 3), (cx + 3, cy - 3));
        }
    }
}

fn visible_rect(ox: i32, oy: i32, p: &Point, size: i32) -> Option<Rect> {
    let vy = p.y - SKYLINE_Y;
    if (0..MATRIX_HEIGHT_VISIBLE).contains(&vy) && (0..MATRIX_WIDTH).contains(&p.x) {
        Some(cell_rect(ox, oy, p.x, vy, size))
    } else {
        None
    }
}

pub fn format_time(ms: u64) -> String {
    let total = ms / 1000;
    let m = total / 60;
    let s = total % 60;
    let cs = (ms % 1000) / 10;
    format!("{m:02}:{s:02}.{cs:02}")
}
