use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::Rng;

pub mod jump;
pub mod method;
pub mod stats;

pub struct Solution {
    discrete_log: i64,
    group_ops: u64,
}

impl Solution {
    pub fn new(discrete_log: i64, group_ops: u64) -> Self {
        Self {
            discrete_log,
            group_ops,
        }
    }

    pub fn discrete_log(&self) -> i64 {
        self.discrete_log
    }

    pub fn group_ops(&self) -> u64 {
        self.group_ops
    }
}

pub trait DiscreteLogSolver {
    fn solve_symmetric<G: KangarooGroup>(&self, element: G, n: i64, rng: &mut impl Rng)
    -> Solution;

    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, rng: &mut impl Rng) -> Solution {
        assert!(l < h);

        let n = h - l;
        let mid = (l + h) / 2;
        let element = element - generator_scalar_mul_i64::<G>(mid);
        let symmetric_solution = self.solve_symmetric(element, n, rng);
        let discrete_log = symmetric_solution.discrete_log() + mid;
        let group_ops = symmetric_solution.group_ops();
        Solution::new(discrete_log, group_ops)
    }
}
