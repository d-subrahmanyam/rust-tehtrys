//! The Matrix: 10 wide × 40 high (20 visible + 20 buffer). y-down. (§1.4, §10)
//!
//! Cells `None` empty, `Some(kind)` locked Block. Collision (§5.1): cannot move
//! into occupied cells, walls, floor, or above the top; out-of-bounds = blocked.

use crate::core::constants::*;
use crate::core::types::{Cell, Point, Tetrimino};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    pub width: i32,
    pub height: i32,
    cells: Vec<Cell>,
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Matrix {
    pub fn new() -> Self {
        Self {
            width: MATRIX_WIDTH,
            height: MATRIX_HEIGHT_TOTAL,
            cells: vec![None; (MATRIX_WIDTH * MATRIX_HEIGHT_TOTAL) as usize],
        }
    }

    #[inline]
    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// Cell occupied (locked Block)? Out-of-bounds counts as occupied for collision.
    #[inline]
    pub fn occupied(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return true; // walls / floor / ceiling / outside
        }
        self.cells[self.idx(x, y)].is_some()
    }

    /// Is `cell` free (in-bounds & empty)?
    #[inline]
    pub fn is_free_cell(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.cells[self.idx(x, y)].is_none()
    }

    pub fn get(&self, x: i32, y: i32) -> Cell {
        if !self.in_bounds(x, y) {
            return Some(Tetrimino::O); // treat outside as solid for callers that don't check bounds
        }
        self.cells[self.idx(x, y)]
    }

    /// Raw cell ignoring bounds (returns None if OOB). Prefer `get`/`occupied`.
    pub fn get_opt(&self, x: i32, y: i32) -> Cell {
        if !self.in_bounds(x, y) {
            return None;
        }
        self.cells[self.idx(x, y)]
    }

    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.cells[i] = cell;
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(None);
    }

    /// Are all given cells free (non-overlapping, in-bounds)? (§5.1)
    pub fn is_free(&self, cells: &[Point]) -> bool {
        cells.iter().all(|p| self.is_free_cell(p.x, p.y))
    }

    /// Would a set of cells be fully within the horizontal walls? (collision §5.1)
    pub fn no_overlap(&self, cells: &[Point]) -> bool {
        cells.iter().all(|p| self.is_free_cell(p.x, p.y))
    }

    /// Place locked cells. Caller ensures `is_free`. (A1.2.3 commit)
    pub fn lock_cells(&mut self, cells: &[Point], kind: Tetrimino) {
        for p in cells {
            self.set(p.x, p.y, Some(kind));
        }
    }

    /// Full rows (all 10 occupied). Scans **visible** region. (A1.2.4)
    /// (Guideline line clears occur in the matrix; buffer rows aren't cleared.)
    pub fn find_full_rows_visible(&self) -> Vec<i32> {
        (SKYLINE_Y..self.height)
            .filter(|&y| (0..self.width).all(|x| self.cells[self.idx(x, y)].is_some()))
            .collect()
    }

    /// Full rows across the entire grid. (A1.2.4 general) — usually visible-only used.
    pub fn find_full_rows_all(&self) -> Vec<i32> {
        (0..self.height)
            .filter(|&y| (0..self.width).all(|x| self.cells[self.idx(x, y)].is_some()))
            .collect()
    }

    /// Remove given rows; collapse everything above down. (A1.2.7 Eliminate)
    /// Returns number of rows removed. Rows are processed correctly regardless of order.
    pub fn clear_rows(&mut self, rows: &[i32]) -> u32 {
        if rows.is_empty() {
            return 0;
        }
        let mut sorted = rows.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        // Rebuild column-wise: remove rows, shift down, fill empty at top. Simple & robust.
        // Process from bottom; standard approach: filter out rows.
        let removed = sorted.len() as u32;
        // Rebuild bottom-up: copy non-removed rows down, fill top with empty. (A1.2.7)
        let mut new_cells = vec![None; self.cells.len()];
        let mut dest_y = self.height - 1;
        let mut src_y = self.height - 1;
        while src_y >= 0 {
            if !sorted.contains(&src_y) {
                for x in 0..self.width {
                    new_cells[(dest_y * self.width + x) as usize] = self.cells[self.idx(x, src_y)];
                }
                dest_y -= 1;
            }
            src_y -= 1;
        }
        self.cells = new_cells;
        removed
    }

    /// Lock-out predicate for a just-committed piece: all cells above skyline. (§10.6)
    pub fn all_cells_above_skyline(cells: &[Point]) -> bool {
        cells.iter().all(|p| p.y < SKYLINE_Y)
    }

    /// Render snapshot of a row. (visible rows 0..total incl buffer)
    pub fn row(&self, y: i32) -> Vec<Cell> {
        (0..self.width).map(|x| self.get_opt(x, y)).collect()
    }

    /// Iterate visible cells y-down. (rendering/helpers)
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill_row(m: &mut Matrix, y: i32) {
        for x in 0..m.width {
            m.set(x, y, Some(Tetrimino::O));
        }
    }

    #[test]
    fn empty_matrix_nothing_occupied() {
        let m = Matrix::new();
        for y in 0..m.height {
            for x in 0..m.width {
                assert!(!m.occupied(x, y));
            }
        }
        // out of bounds = blocked
        assert!(m.occupied(-1, 0));
        assert!(m.occupied(0, -1));
        assert!(m.occupied(m.width, 0));
        assert!(m.occupied(0, m.height));
    }

    #[test]
    fn lock_and_detect_full_row_visible() {
        let mut m = Matrix::new();
        let y = SKYLINE_Y + 5;
        for x in 0..m.width - 1 {
            m.set(x, y, Some(Tetrimino::I));
        }
        assert!(m.find_full_rows_visible().is_empty());
        m.set(m.width - 1, y, Some(Tetrimino::I));
        assert_eq!(m.find_full_rows_visible(), vec![y]);
    }

    #[test]
    fn full_row_in_buffer_not_counted_visible() {
        let mut m = Matrix::new();
        let y = 2; // buffer row
        fill_row(&mut m, y);
        assert!(m.find_full_rows_visible().is_empty());
        assert!(m.find_full_rows_all().contains(&y));
    }

    #[test]
    fn clear_single_row_collapses_above() {
        let mut m = Matrix::new();
        let clear_y = SKYLINE_Y + 10;
        // block above the cleared row
        m.set(3, clear_y - 1, Some(Tetrimino::T));
        m.set(4, clear_y - 3, Some(Tetrimino::L));
        fill_row(&mut m, clear_y);
        // block below cleared row stays put
        m.set(7, clear_y + 1, Some(Tetrimino::S));

        let removed = m.clear_rows(&[clear_y]);
        assert_eq!(removed, 1);
        // cleared row is re-filled from above (blocks shift down); no FULL row remains
        assert!(m.find_full_rows_visible().is_empty());
        // blocks above shifted down by 1
        assert_eq!(m.get_opt(3, clear_y), Some(Tetrimino::T));
        assert_eq!(m.get_opt(4, clear_y - 2), Some(Tetrimino::L));
        // block below unaffected
        assert_eq!(m.get_opt(7, clear_y + 1), Some(Tetrimino::S));
    }

    #[test]
    fn clear_multiple_rows_collapse() {
        let mut m = Matrix::new();
        let base = SKYLINE_Y + 10;
        fill_row(&mut m, base);
        fill_row(&mut m, base + 1);
        // block two rows above
        m.set(1, base - 2, Some(Tetrimino::Z));
        let removed = m.clear_rows(&[base, base + 1]);
        assert_eq!(removed, 2);
        // block shifted down by 2
        assert_eq!(m.get_opt(1, base), Some(Tetrimino::Z));
        assert!(m.find_full_rows_visible().is_empty());
    }

    #[test]
    fn clear_rows_unsorted_and_dup() {
        let mut m = Matrix::new();
        let base = SKYLINE_Y + 5;
        fill_row(&mut m, base);
        fill_row(&mut m, base + 1);
        let removed = m.clear_rows(&[base + 1, base, base]);
        assert_eq!(removed, 2);
        assert!(m.find_full_rows_visible().is_empty());
    }

    #[test]
    fn lock_out_predicate() {
        // whole piece above skyline
        let above = vec![
            Point::new(4, 1),
            Point::new(5, 1),
            Point::new(4, 2),
            Point::new(5, 2),
        ];
        assert!(Matrix::all_cells_above_skyline(&above));
        // peeking straddling skyline -> not lock out
        let peek = vec![Point::new(4, SKYLINE_Y - 1), Point::new(4, SKYLINE_Y)];
        assert!(!Matrix::all_cells_above_skyline(&peek));
    }
}
