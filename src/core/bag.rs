//! "7-bag" random generation + Next Queue. (§3.3, A1.2.1, §2.4.3, §2.5.1)
//!
//! Seven distinct tetriminos shuffled; deal all 7 before refilling + reshuffling.
//! Every 7 consecutive pieces from a bag boundary contain each exactly once. (§3.3)

use crate::core::rng::Rng;
use crate::core::types::{ALL_TETRIMINOS, Tetrimino};
use std::collections::VecDeque;

/// Default visible next-queue length. (§2.4.3 "ideally six"; range 1–6 §2.5.1)
pub const DEFAULT_NEXT_LEN: usize = 6;
pub const MIN_NEXT_LEN: usize = 1;
pub const MAX_NEXT_LEN: usize = 6;

pub struct Bag<R: Rng> {
    rng: R,
    /// remaining pieces in current bag.
    current: Vec<Tetrimino>,
    /// upcoming pieces (front = next to generate). Pre-filled to `next_len + 1`.
    queue: VecDeque<Tetrimino>,
    /// total pieces generated (stats).
    generated: u64,
}

impl<R: Rng> Bag<R> {
    pub fn new(rng: R, next_len: usize) -> Self {
        let next_len = next_len.clamp(MIN_NEXT_LEN, MAX_NEXT_LEN);
        let mut bag = Self {
            rng,
            current: Vec::new(),
            queue: VecDeque::new(),
            generated: 0,
        };
        bag.refill_bag();
        // pre-fill queue
        while bag.queue.len() < next_len + 1 {
            let k = bag.draw_from_bag();
            bag.queue.push_back(k);
        }
        bag
    }

    fn refill_bag(&mut self) {
        let mut b = ALL_TETRIMINOS.to_vec();
        self.rng.shuffle(&mut b);
        self.current = b;
    }

    fn draw_from_bag(&mut self) -> Tetrimino {
        if self.current.is_empty() {
            self.refill_bag();
        }
        self.current.pop().expect("bag non-empty after refill")
    }

    /// Generate next piece: pop front of queue; push one new at back from bag. (§3.3 step 3-4)
    pub fn draw(&mut self) -> Tetrimino {
        let front = self.queue.pop_front().expect("queue non-empty");
        let new_back = self.draw_from_bag();
        self.queue.push_back(new_back);
        self.generated += 1;
        front
    }

    /// Visible next `n` pieces (front first). (§2.4.3)
    pub fn peek(&self, n: usize) -> Vec<Tetrimino> {
        self.queue.iter().take(n).copied().collect()
    }

    /// Peek at the very next piece without consuming.
    pub fn peek_front(&self) -> Option<Tetrimino> {
        self.queue.front().copied()
    }

    /// Insert a piece at the front (used by Hold when swapping back / re-inserting).
    /// For Hold we don't pull from bag; hold manages its own piece. (§5.6)
    /// But a hold-swap with empty hold draws from `next()`.
    pub fn push_front(&mut self, kind: Tetrimino) {
        self.queue.push_front(kind);
    }

    pub fn generated_count(&self) -> u64 {
        self.generated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rng::SplitMix64;

    fn bag(seed: u64, len: usize) -> Bag<SplitMix64> {
        Bag::new(SplitMix64::new(seed), len)
    }

    #[test]
    fn first_seven_contain_each_exactly_once() {
        // every 7 consecutive from bag boundary contain each piece exactly once. (§3.3)
        for seed in 0..50 {
            let mut b = bag(seed, 6);
            let mut got = Vec::new();
            for _ in 0..7 {
                got.push(b.draw());
            }
            let mut sorted = got.clone();
            sorted.sort();
            assert_eq!(sorted.len(), 7, "seed {seed}: {got:?}");
            let mut want = ALL_TETRIMINOS.to_vec();
            want.sort();
            assert_eq!(sorted, want, "seed {seed}");
        }
    }

    #[test]
    fn bag_refills_and_reshuffles() {
        let mut b = bag(123, 6);
        // pull 14 (two bags); each 7-block contains all 7
        let mut blocks = Vec::new();
        for _ in 0..14 {
            blocks.push(b.draw());
        }
        let first7 = &blocks[0..7];
        let second7 = &blocks[7..14];
        let mut a = first7.to_vec();
        a.sort();
        let mut c = second7.to_vec();
        c.sort();
        assert_eq!(a, c); // both are permutations of all 7
        let mut want = ALL_TETRIMINOS.to_vec();
        want.sort();
        assert_eq!(a, want);
    }

    #[test]
    fn next_queue_peek_feeds_correctly() {
        let mut b = bag(9, 5);
        let before = b.peek(5);
        let front = b.peek_front();
        assert_eq!(front, before.first().copied());
        let popped = b.draw();
        assert_eq!(Some(popped), front);
        // queue shifts forward by one; length preserved
        assert_eq!(b.peek(5).len(), 5);
        assert_eq!(b.peek(5)[0], before[1]);
    }

    #[test]
    fn next_len_clamped() {
        let b1 = bag(1, 0); // len 0 -> clamped to MIN=1
        assert_eq!(b1.peek(1).len(), 1);
        let b2 = bag(1, 99); // len 99 -> clamped to MAX=6
        assert_eq!(b2.peek(6).len(), 6);
        assert!(!b2.peek(6).is_empty());
    }

    #[test]
    fn reproducible_with_seed() {
        let mut a = bag(2024, 6);
        let mut b = bag(2024, 6);
        for _ in 0..50 {
            assert_eq!(a.draw(), b.draw());
        }
    }
}
