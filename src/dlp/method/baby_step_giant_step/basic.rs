use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::Rng;
use std::collections::HashMap;

#[derive(Default)]
pub struct Basic;

impl Basic {
    fn solve_zero_based<G: KangarooGroup>(&self, element: G, n: i64) -> Solution {
        let m = (n as f64).sqrt().round() as i64;
        let mut group_ops = 0;
        let mut babies = HashMap::new();
        for i in 0..m {
            babies.insert(generator_scalar_mul_i64::<G>(i), i);
            group_ops += 1;
        }

        let giant_step = -generator_scalar_mul_i64::<G>(m);
        let mut giant_distance = 0;
        let mut giant = element;
        loop {
            if let Some(baby_distance) = babies.get(&giant) {
                let discrete_log = giant_distance + baby_distance;
                return Solution::new(discrete_log, group_ops);
            }

            giant += giant_step;
            giant_distance += m;
            group_ops += 1;
        }
    }
}

impl DiscreteLogSolver for Basic {
    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, _: &mut impl Rng) -> Solution {
        let n = h - l;
        let shift = l;
        let element = element - generator_scalar_mul_i64::<G>(shift);

        let solution = self.solve_zero_based(element, n);
        Solution::new(solution.discrete_log() + shift, solution.group_ops())
    }
}
