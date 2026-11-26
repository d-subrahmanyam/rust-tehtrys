//! T-Spin recognition. (§9, §9.1, §9.1.1, §9.1.2, A1.4.1, §11 in converted ref)
//!
//! T only; last successful action must be a **rotation**; **3 of 4** diagonal
//! corners around T's center occupied (walls/floor count); full vs **Mini**;
//! **Rotation Point 5** => full.

use crate::core::matrix::Matrix;
use crate::core::types::{Facing, Point, Tetrimino};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TSpinKind {
    #[default]
    None,
    /// full T-Spin. (§9.1.1)
    Full,
    /// Mini T-Spin. (§9.1.2)
    Mini,
}

/// Corner offsets relative to T center. Engine y-down.
const NW: Point = Point::new(-1, -1);
const NE: Point = Point::new(1, -1);
const SW: Point = Point::new(-1, 1);
const SE: Point = Point::new(1, 1);

/// Corner labels A,B,C,D per facing. (§9.1; converted ref §11.1)
/// A,B = side T points toward ("nub" side); C,D = flat back.
/// Returns (A, B, C, D).
fn corners(facing: Facing) -> (Point, Point, Point, Point) {
    match facing {
        Facing::North => (NW, NE, SW, SE), // A=top-left B=top-right C=bot-left D=bot-right
        Facing::East => (NE, SE, NW, SW),  // A=top-right B=bot-right C=top-left D=bot-left
        Facing::South => (SE, SW, NE, NW), // A=bot-right B=bot-left C=top-right D=top-left
        Facing::West => (SW, NW, SE, NE),  // A=bot-left B=top-left C=bot-right D=top-right
    }
}

/// Center cell of T in engine space (local (1,1)).
fn center(kind: Tetrimino, origin: Point) -> Point {
    debug_assert!(kind == Tetrimino::T);
    origin.offset(1, 1)
}

/// Detect T-Spin at lock time. (§9.1)
///
/// `matrix` = matrix **before** line clear (after piece committed? caller decides).
/// We check corners against the matrix WITHOUT the T's own cells (T cells aren't
/// diagonal anyway). `origin` = T's bounding-box origin. `facing`.
/// `last_was_rotation`. `last_kick_index` (1..=5; Some(5)=Point5).
pub fn detect(
    kind: Tetrimino,
    facing: Facing,
    origin: Point,
    last_was_rotation: bool,
    last_kick_index: Option<usize>,
    matrix: &Matrix,
) -> TSpinKind {
    if kind != Tetrimino::T {
        return TSpinKind::None;
    }
    if !last_was_rotation {
        return TSpinKind::None; // must rotate into slot. (§9)
    }
    let c = center(kind, origin);
    let (a, b, cc, d) = corners(facing);
    let occ = |p: Point| matrix.occupied(c.x + p.x, c.y + p.y); // OOB=occupied. (§9.1)
    let (oa, ob, oc, od) = (occ(a), occ(b), occ(cc), occ(d));
    let count = [oa, ob, oc, od].iter().filter(|x| **x).count();
    if count < 3 {
        return TSpinKind::None; // need 3-of-4. (§9)
    }
    let point5 = last_kick_index == Some(5);

    // full T-Spin conditions. (§9.1.1)
    let front_pair_plus_back = oa && ob && (oc || od); // A&B + (C|D)
    let all_four = count == 4; // fills T-Slot completely, no holes
    let full = front_pair_plus_back || all_four || point5;
    if full {
        return TSpinKind::Full;
    }
    // mini T-Spin conditions. (§9.1.2) C&D + (A|B); or creates holes (unless point5)
    let back_pair_plus_front = oc && od && (oa || ob);
    if back_pair_plus_front {
        return TSpinKind::Mini;
    }
    // count>=3 not full & not mini-back-pair... treat remaining as mini (holes in T-Slot)
    TSpinKind::Mini
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a T-Spin double-ish slot; verify corner counting & full/mini.
    fn empty() -> Matrix {
        Matrix::new()
    }

    #[test]
    fn not_t_no_spin() {
        let m = empty();
        assert_eq!(
            detect(
                Tetrimino::L,
                Facing::North,
                Point::new(4, 19),
                true,
                Some(1),
                &m
            ),
            TSpinKind::None
        );
    }

    #[test]
    fn must_have_been_rotation() {
        let mut m = empty();
        // create 3 occupied corners around a T center
        let origin = Point::new(4, 19);
        let c = origin.offset(1, 1);
        m.set(c.x - 1, c.y - 1, Some(Tetrimino::O));
        m.set(c.x + 1, c.y - 1, Some(Tetrimino::O));
        m.set(c.x - 1, c.y + 1, Some(Tetrimino::O));
        // last action NOT rotation -> no spin
        assert_eq!(
            detect(Tetrimino::T, Facing::North, origin, false, None, &m),
            TSpinKind::None
        );
        // last action rotation -> spin
        assert!(detect(Tetrimino::T, Facing::North, origin, true, Some(1), &m) != TSpinKind::None);
    }

    #[test]
    fn needs_three_of_four_corners() {
        let mut m = empty();
        let origin = Point::new(4, 19);
        let c = origin.offset(1, 1);
        // only 2 corners
        m.set(c.x - 1, c.y - 1, Some(Tetrimino::O));
        m.set(c.x + 1, c.y - 1, Some(Tetrimino::O));
        assert_eq!(
            detect(Tetrimino::T, Facing::North, origin, true, Some(1), &m),
            TSpinKind::None
        );
        // 3rd corner
        m.set(c.x - 1, c.y + 1, Some(Tetrimino::O));
        assert!(detect(Tetrimino::T, Facing::North, origin, true, Some(1), &m) != TSpinKind::None);
    }

    #[test]
    fn walls_and_floor_count_as_occupied() {
        // walls/floor count as occupied. (§9.1)
        let m = empty();
        assert!(m.occupied(-1, 30)); // left wall
        assert!(m.occupied(0, 1000)); // floor / OOB
        assert!(m.occupied(1000, 0)); // right wall
    }

    #[test]
    fn point5_is_full_not_mini() {
        let mut m = empty();
        let origin = Point::new(4, 19);
        let c = origin.offset(1, 1);
        // back pair + one front (mini pattern) but point5 => full. (§9.1.1/§9.1.2)
        m.set(c.x - 1, c.y + 1, Some(Tetrimino::O)); // C
        m.set(c.x + 1, c.y + 1, Some(Tetrimino::O)); // D
        m.set(c.x - 1, c.y - 1, Some(Tetrimino::O)); // A
        assert_eq!(
            detect(Tetrimino::T, Facing::North, origin, true, Some(2), &m),
            TSpinKind::Mini
        );
        assert_eq!(
            detect(Tetrimino::T, Facing::North, origin, true, Some(5), &m),
            TSpinKind::Full
        );
    }

    #[test]
    fn mini_back_pair() {
        let mut m = empty();
        let origin = Point::new(4, 19);
        let c = origin.offset(1, 1);
        // C&D (back) + one front => mini. North: C=SW D=SE, A=NW
        m.set(c.x - 1, c.y + 1, Some(Tetrimino::O));
        m.set(c.x + 1, c.y + 1, Some(Tetrimino::O));
        m.set(c.x - 1, c.y - 1, Some(Tetrimino::O));
        assert_eq!(
            detect(Tetrimino::T, Facing::North, origin, true, Some(1), &m),
            TSpinKind::Mini
        );
    }

    #[test]
    fn full_front_pair_plus_back() {
        let mut m = empty();
        let origin = Point::new(4, 19);
        let c = origin.offset(1, 1);
        // A&B (front/top) + one back => full. North A=NW B=NE C=SW D=SE
        m.set(c.x - 1, c.y - 1, Some(Tetrimino::O));
        m.set(c.x + 1, c.y - 1, Some(Tetrimino::O));
        m.set(c.x + 1, c.y + 1, Some(Tetrimino::O)); // D
        assert_eq!(
            detect(Tetrimino::T, Facing::North, origin, true, Some(1), &m),
            TSpinKind::Full
        );
    }
}
