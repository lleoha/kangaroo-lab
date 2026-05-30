use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::Rng;
use std::collections::HashMap;

#[derive(Default)]
pub struct NegMap;

impl NegMap {
    fn solve_zero_based<G: KangarooGroup>(&self, element: G, n: i64) -> Solution {
        let m = (n as f64).sqrt().round() as i64;
        let mut group_ops = 0;
        let mut babies = HashMap::new();

        for i in 0..=((m + 1) / 2) {
            let mut baby_distance = i;
            let mut baby = generator_scalar_mul_i64::<G>(baby_distance);
            if !baby.is_negation_map_representative() {
                (baby_distance, baby) = (-baby_distance, -baby);
            }
            babies.insert(baby, baby_distance);
            group_ops += 1;
        }

        let giant_step = -generator_scalar_mul_i64::<G>(m);
        let mut giant_distance = 0;
        let mut giant = element;

        loop {
            let mut g = giant;
            let mut giant_sign = 1;
            if !g.is_negation_map_representative() {
                giant_sign = -1;
                g = -g;
            }

            if let Some(&baby_distance) = babies.get(&g) {
                let discrete_log = giant_distance + giant_sign * baby_distance;
                return Solution::new(discrete_log, group_ops);
            }

            giant_distance += m;
            giant += giant_step;
            group_ops += 1;
        }
    }
}

impl DiscreteLogSolver for NegMap {
    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, _: &mut impl Rng) -> Solution {
        let n = h - l;
        let shift = l;
        let element = element - generator_scalar_mul_i64::<G>(shift);

        let solution = self.solve_zero_based(element, n);
        Solution::new(solution.discrete_log() + shift, solution.group_ops())
    }
}
