use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::Rng;
use std::collections::HashMap;

#[derive(Default)]
pub struct Interleaved;

impl Interleaved {
    fn solve_zero_based<G: KangarooGroup>(&self, element: G, n: i64) -> Solution {
        let m = ((n * 2) as f64).sqrt().round() as i64;
        let mut group_ops = 0;

        let mut baby_distance = 0;
        let mut baby = G::identity();
        let mut babies = HashMap::new();
        let baby_step_distance = 1;
        let baby_step = G::generator();

        let mut giant_distance = 0;
        let mut giant = element;
        let mut giants = HashMap::new();
        let giant_step_distance = m;
        let giant_step = generator_scalar_mul_i64::<G>(m);

        loop {
            baby_distance += baby_step_distance;
            baby += baby_step;
            group_ops += 1;
            let (d, b) = if baby.is_negation_map_representative() {
                (baby_distance, baby)
            } else {
                (-baby_distance, -baby)
            };
            if let Some(&giant_distance) = giants.get(&b) {
                let giant_distance: i64 = giant_distance;
                let discrete_log = giant_distance.abs() + giant_distance.signum() * d;
                return Solution::new(discrete_log, group_ops);
            }
            babies.insert(b, d);

            giant_distance += giant_step_distance;
            giant -= giant_step;
            group_ops += 1;
            let (d, g) = if giant.is_negation_map_representative() {
                (giant_distance, giant)
            } else {
                (-giant_distance, -giant)
            };
            if let Some(&baby_distance) = babies.get(&g) {
                let discrete_log = d.abs() + d.signum() * baby_distance;
                return Solution::new(discrete_log, group_ops);
            }
            giants.insert(g, d);
        }
    }
}

impl DiscreteLogSolver for Interleaved {
    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, _: &mut impl Rng) -> Solution {
        let n = h - l;
        let shift = l;
        let element = element - generator_scalar_mul_i64::<G>(shift);

        let solution = self.solve_zero_based(element, n);
        Solution::new(solution.discrete_log() + shift, solution.group_ops())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::toy::ToyGroup;
    use rand::rng;

    #[test]
    fn solves_small_intervals_exhaustively() {
        let solver = Interleaved;
        let mut rng = rng();

        for low in -8..8 {
            for high in low + 1..low + 40 {
                for x in low..high {
                    let element = generator_scalar_mul_i64::<ToyGroup>(x);
                    let result = solver.solve(element, low, high, &mut rng);
                    assert_eq!(result.discrete_log(), x, "interval {low}..{high}");
                }
            }
        }
    }
}
