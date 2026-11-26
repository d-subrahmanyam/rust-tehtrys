//! Minimal SDL2 rendering. (§2.4, §13; flows/12) Pure read of a snapshot; no logic.
//! Phase 2: filled cells, active + ghost, border. No assets, no text (font Phase 3).
//! Pause hides matrix. (§13.1.6) Full HUD + notifications in Phase 3.

use crate::core::constants::*;
use crate::core::matrix::Matrix;
use crate::core::types::{Cell, Point, Tetrimino};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub const CELL_PX: i32 = 28;
pub const MARGIN: i32 = 12;
pub const MATRIX_PX_W: i32 = MATRIX_WIDTH * CELL_PX;
pub const MATRIX_PX_H: i32 = MATRIX_HEIGHT_VISIBLE * CELL_PX;

const BG: Color = Color::RGB(16, 16, 20);
const EMPTY: Color = Color::RGB(24, 24, 30);
const GRID: Color = Color::RGB(40, 40, 48);
const BORDER: Color = Color::RGB(90, 90, 110);
const GHOST: Color = Color::RGB(70, 70, 90);

fn cell_color(k: Tetrimino) -> Color {
    let c = k.color();
    Color::RGB(c.r, c.g, c.b)
}

/// Immutable render snapshot. (02-architecture)
pub struct RenderSnapshot<'a> {
    pub matrix: &'a Matrix,
    pub active_cells: Vec<Point>,
    pub active_kind: Option<Tetrimino>,
    pub ghost_cells: Vec<Point>,
    pub hold: Option<Tetrimino>,
    pub next: Vec<Tetrimino>,
    pub score: u64,
    pub level: u32,
    pub lines: u32,
    pub goal: u32,
    pub paused: bool,
    pub game_over: bool,
}

/// Draw a frame.
pub fn draw(canvas: &mut Canvas<Window>, snap: &RenderSnapshot) {
    canvas.set_draw_color(BG);
    canvas.clear();

    if snap.paused {
        // §13.1.6: pause hides Matrix/Next/Hold. (no "PAUSE" text until font Phase 3)
        canvas.present();
        return;
    }

    let ox = MARGIN;
    let oy = MARGIN;

    // matrix border
    canvas.set_draw_color(BORDER);
    let _ = canvas.draw_rect(Rect::new(
        ox - 1,
        oy - 1,
        (MATRIX_PX_W + 2) as u32,
        (MATRIX_PX_H + 2) as u32,
    ));

    // visible matrix cells
    for y in 0..MATRIX_HEIGHT_VISIBLE {
        for x in 0..MATRIX_WIDTH {
            let cell: Cell = snap.matrix.get_opt(x, SKYLINE_Y + y);
            let rect = cell_rect(ox, oy, x, y);
            match cell {
                Some(k) => {
                    canvas.set_draw_color(cell_color(k));
                    let _ = canvas.fill_rect(rect);
                }
                None => {
                    canvas.set_draw_color(EMPTY);
                    let _ = canvas.fill_rect(rect);
                    canvas.set_draw_color(GRID);
                    let _ = canvas.draw_rect(rect);
                }
            }
        }
    }

    // ghost (§2.4.4) only visible part
    canvas.set_draw_color(GHOST);
    for p in &snap.ghost_cells {
        if let Some(rect) = visible_rect(ox, oy, p) {
            let _ = canvas.draw_rect(rect);
        }
    }

    // active piece (§2.4.2) only visible part
    if let Some(kind) = snap.active_kind {
        canvas.set_draw_color(cell_color(kind));
        for p in &snap.active_cells {
            if let Some(rect) = visible_rect(ox, oy, p) {
                let _ = canvas.fill_rect(rect);
            }
        }
    }

    canvas.present();
}

fn cell_rect(ox: i32, oy: i32, x: i32, y: i32) -> Rect {
    Rect::new(
        ox + x * CELL_PX,
        oy + y * CELL_PX,
        (CELL_PX - 1) as u32,
        (CELL_PX - 1) as u32,
    )
}

fn visible_rect(ox: i32, oy: i32, p: &Point) -> Option<Rect> {
    let vy = p.y - SKYLINE_Y;
    if (0..MATRIX_HEIGHT_VISIBLE).contains(&vy) && (0..MATRIX_WIDTH).contains(&p.x) {
        Some(cell_rect(ox, oy, p.x, vy))
    } else {
        None
    }
}
