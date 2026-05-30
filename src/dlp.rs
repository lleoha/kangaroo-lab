use crate::group::KangarooGroup;
use rand::Rng;

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
    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, rng: &mut impl Rng) -> Solution;
}
