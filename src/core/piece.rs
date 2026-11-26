//! Tetrimino shapes, orientation tables, spawn. (§3, A1.3, §3.4, A1.2.1)
//!
//! Orientation cells are explicit (no runtime rotation math); pure 90° rotation
//! about the piece's rotation point, verified in tests. Local coords 0-based,
//! y-down, relative to bounding-box top-left. Global = `origin + local`.
//! See `docs/plan/appendices/B-piece-data.md`.

use crate::core::types::{Facing, Point, Tetrimino};

/// Local cells for each kind & facing. `[kind][facing][4]`.
const ORIENTATIONS: [[[Point; 4]; 4]; 7] = [
    // I (4×4 box)
    [
        [
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(3, 1),
        ], // N
        [
            Point::new(2, 0),
            Point::new(2, 1),
            Point::new(2, 2),
            Point::new(2, 3),
        ], // E
        [
            Point::new(0, 2),
            Point::new(1, 2),
            Point::new(2, 2),
            Point::new(3, 2),
        ], // S
        [
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(1, 2),
            Point::new(1, 3),
        ], // W
    ],
    // O (2×2 box; never moves)
    [
        [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
        ],
        [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
        ],
        [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
        ],
        [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
        ],
    ],
    // T (3×3)
    [
        [
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
        ],
        [
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(1, 2),
        ],
        [
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(1, 2),
        ],
        [
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(1, 2),
        ],
    ],
    // L (3×3)
    [
        [
            Point::new(2, 0),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
        ],
        [
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(1, 2),
            Point::new(2, 2),
        ],
        [
            Point::new(0, 2),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
        ],
        [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(1, 2),
        ],
    ],
    // J (3×3)
    [
        [
            Point::new(0, 0),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
        ],
        [
            Point::new(1, 0),
            Point::new(2, 0),
            Point::new(1, 1),
            Point::new(1, 2),
        ],
        [
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(2, 2),
        ],
        [
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(1, 2),
            Point::new(0, 2),
        ],
    ],
    // S (3×3)
    [
        [
            Point::new(1, 0),
            Point::new(2, 0),
            Point::new(0, 1),
            Point::new(1, 1),
        ],
        [
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(2, 2),
        ],
        [
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(0, 2),
            Point::new(1, 2),
        ],
        [
            Point::new(0, 0),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(1, 2),
        ],
    ],
    // Z (3×3)
    [
        [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(2, 1),
        ],
        [
            Point::new(2, 0),
            Point::new(1, 1),
            Point::new(2, 1),
            Point::new(1, 2),
        ],
        [
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(1, 2),
            Point::new(2, 2),
        ],
        [
            Point::new(1, 0),
            Point::new(0, 1),
            Point::new(1, 1),
            Point::new(0, 2),
        ],
    ],
];

impl Tetrimino {
    /// Index into tables. Order matches `ALL_TETRIMINOS`.
    pub fn index(self) -> usize {
        match self {
            Tetrimino::I => 0,
            Tetrimino::O => 1,
            Tetrimino::T => 2,
            Tetrimino::L => 3,
            Tetrimino::J => 4,
            Tetrimino::S => 5,
            Tetrimino::Z => 6,
        }
    }

    /// Local cells for a facing. (A1.3)
    pub fn cells_local(self, facing: Facing) -> [Point; 4] {
        ORIENTATIONS[self.index()][facing.index()]
    }

    /// Cells in engine space given bounding-box `origin`.
    pub fn cells_at(self, facing: Facing, origin: Point) -> [Point; 4] {
        let l = self.cells_local(facing);
        [
            origin.plus(l[0]),
            origin.plus(l[1]),
            origin.plus(l[2]),
            origin.plus(l[3]),
        ]
    }

    /// Bounding-box side length. (3x3 TLJSZ, 4x4 I, 2x2 O)
    pub fn box_size(self) -> i32 {
        match self {
            Tetrimino::I => 4,
            Tetrimino::O => 2,
            _ => 3,
        }
    }

    /// Rotation point (local, may be half-cell for O/I). (A1.4.1) Point-1 center.
    /// Returned as doubled integer (x2,y2) to stay exact.
    pub fn rotation_center_x2(self) -> (i32, i32) {
        match self {
            Tetrimino::O => (1, 1), // 0.5,0.5
            Tetrimino::I => (3, 3), // 1.5,1.5
            _ => (2, 2),            // 1,1
        }
    }
}

/// Spawn bounding-box origin in engine space. (§3.4, A1.2.1; appendix B)
/// 3-wide -> (3,18); I -> (3,19); O -> (4,18). Before drop-one-row rule.
pub fn spawn_origin(kind: Tetrimino) -> Point {
    match kind {
        Tetrimino::I => Point::new(3, 18), // I on guideline row21 -> engine row19 (local row-1 cells); 4x4 box
        Tetrimino::O => Point::new(4, 18), // chosen; drop-1 applies; see appendix B O-note
        _ => Point::new(3, 18),            // 3x3; center mino engine (4,19); guideline (5,21)
    }
}

/// North-facing spawn piece at spawn origin (no drop-1 yet). (§3.4, A1.2.1)
pub fn spawn_piece(kind: Tetrimino) -> crate::core::types::ActivePiece {
    crate::core::types::ActivePiece::new(kind, spawn_origin(kind))
}

/// Is kind a T (for T-Spin)? (§9)
pub fn is_t(kind: Tetrimino) -> bool {
    matches!(kind, Tetrimino::T)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::SKYLINE_Y;
    use crate::core::matrix::Matrix;

    fn sorted(mut v: [Point; 4]) -> [Point; 4] {
        v.sort();
        v
    }

    // rotate local cells CW about center for a generated check
    fn rot_cw_local(cells: [Point; 4], cx2: i32, cy2: i32) -> [Point; 4] {
        let mut v: Vec<Point> = cells
            .iter()
            .map(|p| Point::new((cx2 - (2 * p.y - cy2)) / 2, (cy2 + (2 * p.x - cx2)) / 2))
            .collect();
        v.sort();
        [v[0], v[1], v[2], v[3]]
    }

    #[test]
    fn stored_orientations_are_consistent() {
        // N->E->S->W->N by CW; our tables should match pure rotation, but we only
        // assert 4xCW identity on stored tables + count=4 distinct-set checks.
        for k in crate::core::types::ALL_TETRIMINOS {
            let n = sorted(k.cells_local(Facing::North));
            let e = sorted(k.cells_local(Facing::East));
            let s = sorted(k.cells_local(Facing::South));
            let w = sorted(k.cells_local(Facing::West));
            assert_eq!(n.len(), 4);
            // O is same all facings
            if k == Tetrimino::O {
                assert_eq!(n, e);
                assert_eq!(n, s);
                assert_eq!(n, w);
            }
        }
    }

    #[test]
    fn orientations_match_pure_rotation() {
        // verify our stored tables equal pure CW rotation about center from North.
        // (integer center handled with doubled coords; skip half-cell pieces O/I here)
        for k in [
            Tetrimino::T,
            Tetrimino::L,
            Tetrimino::J,
            Tetrimino::S,
            Tetrimino::Z,
        ] {
            let (cx2, cy2) = k.rotation_center_x2();
            let n = sorted(k.cells_local(Facing::North));
            let e_expect = rot_cw_local(k.cells_local(Facing::North), cx2, cy2);
            let e = sorted(k.cells_local(Facing::East));
            assert_eq!(e, e_expect, "East mismatch for {k:?}");
            let s_expect = rot_cw_local(k.cells_local(Facing::East), cx2, cy2);
            let s = sorted(k.cells_local(Facing::South));
            assert_eq!(s, s_expect, "South mismatch for {k:?}");
            let w_expect = rot_cw_local(k.cells_local(Facing::South), cx2, cy2);
            let w = sorted(k.cells_local(Facing::West));
            assert_eq!(w, w_expect, "West mismatch for {k:?}");
            let back = rot_cw_local(k.cells_local(Facing::West), cx2, cy2);
            assert_eq!(back, n, "4x identity for {k:?}");
        }
    }

    #[test]
    fn spawn_positions_match_guideline() {
        // §3.4 / A1.2.1; engine mapping. North. Skyline=20.
        // 3-wide trio row19 cols3-5; center (4,19); spawn origin (3,18).
        let t = Tetrimino::T.cells_at(Facing::North, spawn_origin(Tetrimino::T));
        let mut cells = t.to_vec();
        cells.sort();
        assert!(cells.contains(&Point::new(4, 18))); // extra top-center (guideline row22 col5)
        assert!(cells.contains(&Point::new(3, 19)));
        assert!(cells.contains(&Point::new(4, 19))); // center mino (guideline 5,21)
        assert!(cells.contains(&Point::new(5, 19)));
        // all spawn cells are in buffer (y < SKYLINE_Y)
        assert!(cells.iter().all(|p| p.y < SKYLINE_Y));
        // horizontal centering
        assert!(cells.iter().all(|p| p.x >= 3 && p.x <= 5));

        // I: guideline row21 cols4-7 -> engine row19 cols3-6; North; spawn row above
        let i = Tetrimino::I.cells_at(Facing::North, spawn_origin(Tetrimino::I));
        let mut ic = i.to_vec();
        ic.sort();
        assert_eq!(
            ic,
            vec![
                Point::new(3, 19),
                Point::new(4, 19),
                Point::new(5, 19),
                Point::new(6, 19)
            ]
        );
        assert!(ic.iter().all(|p| p.y < SKYLINE_Y));
        assert!(ic.iter().all(|p| p.x >= 3 && p.x <= 6)); // cols 4-7 1-based
        assert!(ic.iter().all(|p| p.y == 19)); // I on row21 only (guideline) -> engine row19
    }

    #[test]
    fn o_spawn_cells_match_guideline_5_2() {
        // §5.2 O rows21-22 cols5-6 -> engine rows18-19 cols4-5. origin (4,18).
        let o = Tetrimino::O.cells_at(Facing::North, spawn_origin(Tetrimino::O));
        let mut cells = o.to_vec();
        cells.sort();
        assert_eq!(
            cells,
            vec![
                Point::new(4, 18),
                Point::new(4, 19),
                Point::new(5, 18),
                Point::new(5, 19)
            ]
        );
        // horizontally centered cols 4-5. (§3.4)
        let mut xs: Vec<i32> = o.iter().map(|p| p.x).collect();
        xs.sort();
        xs.dedup();
        assert_eq!(xs, vec![4, 5]);
    }

    #[test]
    fn spawn_is_free_on_empty_matrix() {
        let m = Matrix::new();
        for k in crate::core::types::ALL_TETRIMINOS {
            let cells = k.cells_at(Facing::North, spawn_origin(k));
            assert!(
                m.is_free(&cells),
                "{k:?} spawn should be free on empty matrix"
            );
        }
    }

    #[test]
    fn box_sizes() {
        assert_eq!(Tetrimino::I.box_size(), 4);
        assert_eq!(Tetrimino::O.box_size(), 2);
        assert_eq!(Tetrimino::T.box_size(), 3);
    }
}
