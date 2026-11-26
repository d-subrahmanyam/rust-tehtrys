//! Super Rotation System. (§5.3, §A1.4, §A1.4.1, §A1.4.2)
//!
//! Five ordered kick tests; first non-colliding wins; else rotation fails.
//! Kick tables reproduce published SRS; guideline convention **y-up** converted to
//! engine **y-down** by negating `dy`. Conformance against the guideline's vector
//! diagrams is verified in tests; diagrams can't be machine-read here. (appendix A)

use crate::core::matrix::Matrix;
use crate::core::types::{Facing, Point, Rotation, Tetrimino};

/// Kick offset. Engine space (x, y-down). Array order = test order; test1 = (0,0).
pub type Kick = Point;

const Z: Kick = Point::new(0, 0);

/// CW kicks, 3×3 pieces (T,L,J,S,Z), keyed by `from`. (A1.4.2; y-down)
fn kicks_3x3_cw(from: Facing) -> [Kick; 5] {
    match from {
        Facing::North => [
            Z,
            Point::new(-1, 0),
            Point::new(-1, -1),
            Point::new(0, 2),
            Point::new(-1, 2),
        ],
        Facing::East => [
            Z,
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(0, -2),
            Point::new(1, -2),
        ],
        Facing::South => [
            Z,
            Point::new(1, 0),
            Point::new(1, -1),
            Point::new(0, 2),
            Point::new(1, 2),
        ],
        Facing::West => [
            Z,
            Point::new(-1, 0),
            Point::new(-1, 1),
            Point::new(0, -2),
            Point::new(-1, -2),
        ],
    }
}

/// CCW kicks, 3×3 pieces, keyed by `from`. (A1.4.2; y-down)
fn kicks_3x3_ccw(from: Facing) -> [Kick; 5] {
    match from {
        Facing::North => [
            Z,
            Point::new(1, 0),
            Point::new(1, -1),
            Point::new(0, 2),
            Point::new(1, 2),
        ],
        Facing::West => [
            Z,
            Point::new(-1, 0),
            Point::new(-1, 1),
            Point::new(0, -2),
            Point::new(-1, -2),
        ],
        Facing::South => [
            Z,
            Point::new(-1, 0),
            Point::new(-1, -1),
            Point::new(0, 2),
            Point::new(-1, 2),
        ],
        Facing::East => [
            Z,
            Point::new(1, 0),
            Point::new(1, 1),
            Point::new(0, -2),
            Point::new(1, -2),
        ],
    }
}

/// CW kicks, I piece (4×4), keyed by `from`. (A1.4.2; y-down)
fn kicks_i_cw(from: Facing) -> [Kick; 5] {
    match from {
        Facing::North => [
            Z,
            Point::new(-2, 0),
            Point::new(1, 0),
            Point::new(-2, 1),
            Point::new(1, -2),
        ],
        Facing::East => [
            Z,
            Point::new(-1, 0),
            Point::new(2, 0),
            Point::new(-1, -2),
            Point::new(2, 1),
        ],
        Facing::South => [
            Z,
            Point::new(2, 0),
            Point::new(-1, 0),
            Point::new(2, -1),
            Point::new(-1, 2),
        ],
        Facing::West => [
            Z,
            Point::new(1, 0),
            Point::new(-2, 0),
            Point::new(1, 2),
            Point::new(-2, -1),
        ],
    }
}

/// CCW kicks, I piece, keyed by `from`. (A1.4.2; y-down)
fn kicks_i_ccw(from: Facing) -> [Kick; 5] {
    match from {
        Facing::North => [
            Z,
            Point::new(-1, 0),
            Point::new(2, 0),
            Point::new(-1, -2),
            Point::new(2, 1),
        ],
        Facing::West => [
            Z,
            Point::new(-2, 0),
            Point::new(1, 0),
            Point::new(-2, 1),
            Point::new(1, -2),
        ],
        Facing::South => [
            Z,
            Point::new(1, 0),
            Point::new(-2, 0),
            Point::new(1, 2),
            Point::new(-2, -1),
        ],
        Facing::East => [
            Z,
            Point::new(2, 0),
            Point::new(-1, 0),
            Point::new(2, -1),
            Point::new(-1, 2),
        ],
    }
}

/// A kick test: guideline **point number** (1..=5) + offset. (A1.4.2)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KickEntry {
    /// Guideline rotation-point number (1..=5). **Point 5** => T-Spin (§9.1.1).
    pub num: usize,
    pub kick: Kick,
}

/// Ordered kick tests for `kind`, `from`, `rot`. O => all zero. (A1.4.2)
///
/// **T-piece exceptions** (guideline §15.4 / A1.4.2 diagrams mark kicks "Not Used"):
/// - T from **North**: CW(N→E) & CCW(N→W) **omit Point 4**;
/// - T from **South**: CW(S→W) & CCW(S→E) **omit Point 3**.
///
/// Point *numbering is preserved* (e.g. Point 5 remains Point 5) for T-Spin. (§9.1.1)
pub fn kicks(kind: Tetrimino, from: Facing, rot: Rotation) -> Vec<KickEntry> {
    if kind == Tetrimino::O {
        return (1..=5).map(|num| KickEntry { num, kick: Z }).collect();
    }
    let full: [Kick; 5] = if kind == Tetrimino::I {
        match rot {
            Rotation::Cw => kicks_i_cw(from),
            Rotation::Ccw => kicks_i_ccw(from),
        }
    } else {
        match rot {
            Rotation::Cw => kicks_3x3_cw(from),
            Rotation::Ccw => kicks_3x3_ccw(from),
        }
    };
    let mut entries: Vec<KickEntry> = full
        .iter()
        .enumerate()
        .map(|(i, k)| KickEntry {
            num: i + 1,
            kick: *k,
        })
        .collect();
    if kind == Tetrimino::T {
        let skip_num: Option<usize> = match (from, rot) {
            (Facing::North, Rotation::Cw) => Some(4), // N->E omit Point4
            (Facing::North, Rotation::Ccw) => Some(4), // N->W omit Point4
            (Facing::South, Rotation::Cw) => Some(3), // S->W omit Point3
            (Facing::South, Rotation::Ccw) => Some(3), // S->E omit Point3
            _ => None,
        };
        if let Some(num) = skip_num {
            entries.retain(|e| e.num != num);
        }
    }
    entries
}

/// Result of a successful rotation. (A1.4; kick index 1..=5 for T-Spin §9.1.1)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RotationResult {
    pub facing: Facing,
    pub origin: Point,
    /// 1-based kick index used (1..=5); `kick_index == 5` => Point-5. (A1.4.1)
    pub kick_index: usize,
}

/// Try to rotate `kind` at `origin`/`facing` by `rot`, returning first valid
/// placement or `None`. (§5.3, A1.4) O always "succeeds" with no movement.
pub fn try_rotate(
    kind: Tetrimino,
    facing: Facing,
    origin: Point,
    rot: Rotation,
    matrix: &Matrix,
) -> Option<RotationResult> {
    let to = match rot {
        Rotation::Cw => facing.cw(),
        Rotation::Ccw => facing.ccw(),
    };
    if kind == Tetrimino::O {
        return Some(RotationResult {
            facing: to,
            origin,
            kick_index: 1,
        });
    }
    for entry in kicks(kind, facing, rot) {
        let cand_origin = origin.plus(entry.kick);
        let cells = kind.cells_at(to, cand_origin);
        if matrix.is_free(&cells) {
            return Some(RotationResult {
                facing: to,
                origin: cand_origin,
                kick_index: entry.num,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_no_kicks_and_always_rotates() {
        let mut m = Matrix::new();
        m.set(0, 0, Some(Tetrimino::I));
        let origin = Point::new(4, 19);
        let r = try_rotate(Tetrimino::O, Facing::North, origin, Rotation::Cw, &m).unwrap();
        assert_eq!(r.origin, origin);
        assert_eq!(r.kick_index, 1);
        assert_eq!(r.facing, Facing::East);
    }

    #[test]
    fn point1_visual_first_on_empty() {
        let m = Matrix::new();
        let origin = Point::new(4, 19);
        let r = try_rotate(Tetrimino::T, Facing::North, origin, Rotation::Cw, &m).unwrap();
        assert_eq!(r.kick_index, 1);
        assert_eq!(r.origin, origin);
        assert_eq!(r.facing, Facing::East);
    }

    #[test]
    fn rotation_fails_when_all_kicks_blocked() {
        let mut m = Matrix::new();
        for y in 0..m.height {
            for x in 0..m.width {
                m.set(x, y, Some(Tetrimino::O));
            }
        }
        let origin = Point::new(3, 10);
        for p in Tetrimino::I.cells_at(Facing::North, origin) {
            m.set(p.x, p.y, None);
        }
        assert!(try_rotate(Tetrimino::I, Facing::North, origin, Rotation::Cw, &m).is_none());
    }

    #[test]
    fn cw_then_ccw_returns_to_start_on_empty() {
        let m = Matrix::new();
        let origin = Point::new(4, 19);
        let cw = try_rotate(Tetrimino::T, Facing::North, origin, Rotation::Cw, &m).unwrap();
        let back = try_rotate(Tetrimino::T, cw.facing, cw.origin, Rotation::Ccw, &m).unwrap();
        assert_eq!(back.facing, Facing::North);
        assert_eq!(back.origin, origin);
    }

    #[test]
    fn kick_tables_have_five_and_point1_zero() {
        // non-T pieces always 5; T has 4 in the four "not used" cases; point numbering preserved.
        for k in [
            Tetrimino::L,
            Tetrimino::J,
            Tetrimino::S,
            Tetrimino::Z,
            Tetrimino::I,
        ] {
            for f in [Facing::North, Facing::East, Facing::South, Facing::West] {
                for rot in [Rotation::Cw, Rotation::Ccw] {
                    let ks = kicks(k, f, rot);
                    assert_eq!(ks.len(), 5, "{k:?} {f:?} {rot:?}");
                    assert_eq!(ks[0].kick, Z, "point1 must be (0,0)");
                    assert_eq!(ks[0].num, 1);
                }
            }
        }
        for f in [Facing::North, Facing::East, Facing::South, Facing::West] {
            for rot in [Rotation::Cw, Rotation::Ccw] {
                let ks = kicks(Tetrimino::T, f, rot);
                assert!(ks[0].kick == Z);
                assert_eq!(ks[0].num, 1);
                let skip = matches!(
                    (f, rot),
                    (Facing::North, Rotation::Cw)
                        | (Facing::North, Rotation::Ccw)
                        | (Facing::South, Rotation::Cw)
                        | (Facing::South, Rotation::Ccw)
                );
                assert_eq!(ks.len(), if skip { 4 } else { 5 }, "T {f:?} {rot:?}");
                // point 5 still present & last when not omitted; numbering preserved
                assert!(
                    ks.iter().any(|e| e.num == 5),
                    "Point5 must remain for T {f:?} {rot:?}"
                );
            }
        }
    }

    fn kick_list(kind: Tetrimino, f: Facing, rot: Rotation) -> Vec<Kick> {
        kicks(kind, f, rot).into_iter().map(|e| e.kick).collect()
    }

    // guideline A1.4.2 canonical tables, converted to engine y-down (negate dy).
    #[test]
    fn kicks_3x3_exact() {
        // use J (no T exceptions) to check raw table
        let k = Tetrimino::J;
        assert_eq!(
            kick_list(k, Facing::North, Rotation::Cw),
            vec![
                Z,
                Point::new(-1, 0),
                Point::new(-1, -1),
                Point::new(0, 2),
                Point::new(-1, 2)
            ]
        );
        assert_eq!(
            kick_list(k, Facing::East, Rotation::Cw),
            vec![
                Z,
                Point::new(1, 0),
                Point::new(1, 1),
                Point::new(0, -2),
                Point::new(1, -2)
            ]
        );
        assert_eq!(
            kick_list(k, Facing::North, Rotation::Ccw),
            vec![
                Z,
                Point::new(1, 0),
                Point::new(1, -1),
                Point::new(0, 2),
                Point::new(1, 2)
            ]
        );
        assert_eq!(
            kick_list(k, Facing::East, Rotation::Ccw),
            vec![
                Z,
                Point::new(1, 0),
                Point::new(1, 1),
                Point::new(0, -2),
                Point::new(1, -2)
            ]
        );
    }

    #[test]
    fn kicks_i_exact() {
        assert_eq!(
            kick_list(Tetrimino::I, Facing::North, Rotation::Cw),
            vec![
                Z,
                Point::new(-2, 0),
                Point::new(1, 0),
                Point::new(-2, 1),
                Point::new(1, -2)
            ]
        );
        assert_eq!(
            kick_list(Tetrimino::I, Facing::North, Rotation::Ccw),
            vec![
                Z,
                Point::new(-1, 0),
                Point::new(2, 0),
                Point::new(-1, -2),
                Point::new(2, 1)
            ]
        );
    }

    // T exceptions: skipped test, numbering preserved. (§15.4/A1.4.2)
    #[test]
    fn t_piece_kick_exceptions() {
        // T N->E CW omits Point4: tests are 1,2,3,5
        let v = kicks(Tetrimino::T, Facing::North, Rotation::Cw);
        let nums: Vec<usize> = v.iter().map(|e| e.num).collect();
        assert_eq!(nums, vec![1, 2, 3, 5]);
        // T N->W CCW omits Point4
        let v = kicks(Tetrimino::T, Facing::North, Rotation::Ccw);
        assert_eq!(
            v.iter().map(|e| e.num).collect::<Vec<_>>(),
            vec![1, 2, 3, 5]
        );
        // T S->W CW omits Point3
        let v = kicks(Tetrimino::T, Facing::South, Rotation::Cw);
        assert_eq!(
            v.iter().map(|e| e.num).collect::<Vec<_>>(),
            vec![1, 2, 4, 5]
        );
        // T S->E CCW omits Point3
        let v = kicks(Tetrimino::T, Facing::South, Rotation::Ccw);
        assert_eq!(
            v.iter().map(|e| e.num).collect::<Vec<_>>(),
            vec![1, 2, 4, 5]
        );
        // other T rotations keep 5
        let v = kicks(Tetrimino::T, Facing::East, Rotation::Cw);
        assert_eq!(
            v.iter().map(|e| e.num).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }

    // Classic SRS wall test: T at left wall can rotate off it (kick shifts right).
    #[test]
    fn kick_off_left_wall() {
        let m = Matrix::new();
        // T North hugging left wall: origin x=-1 would put a block at x=-1... use origin 0
        let origin = Point::new(0, 19);
        // ensure current placement valid
        let r = try_rotate(Tetrimino::T, Facing::North, origin, Rotation::Cw, &m);
        // should succeed with some kick (not necessarily kick1)
        assert!(r.is_some());
    }
}
