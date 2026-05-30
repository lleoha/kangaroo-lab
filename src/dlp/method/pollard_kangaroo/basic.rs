use std::collections::HashMap;

use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::KangarooGroup;
use crate::group::generator_scalar_mul_i64;
use rand::{Rng, RngExt};

#[derive(Default)]
pub struct Basic;

impl Basic {
    fn collision(tame_distance: i64, wild_distance: i64) -> i64 {
        tame_distance - wild_distance
    }

    fn solve_symmetric<G: KangarooGroup>(
        &self,
        element: G,
        n: i64,
        rng: &mut impl Rng,
    ) -> Solution {
        let m = 0.5 * (n as f64).sqrt();
        let m2 = (2. * m).round() as i64;

        let mut group_ops = 0;
        let mut tame = G::generator();
        let mut tame_distance = 1;
        let mut wild = element;
        let mut wild_distance = 0;
        let mut tames = HashMap::new();
        let mut wilds = HashMap::new();

        loop {
            let tame_jump_distance = rng.random_range(0..m2);
            let tame_jump = generator_scalar_mul_i64::<G>(tame_jump_distance);
            tame_distance += tame_jump_distance;
            tame += tame_jump;
            group_ops += 1;
            if let Some(&w_distance) = wilds.get(&tame) {
                let discrete_log = Self::collision(tame_distance, w_distance);
                return Solution::new(discrete_log, group_ops);
            }
            tames.insert(tame, tame_distance);

            let wild_jump_distance = rng.random_range(0..m2);
            let wild_jump = generator_scalar_mul_i64::<G>(wild_jump_distance);
            wild_distance += wild_jump_distance;
            wild += wild_jump;
            group_ops += 1;
            if let Some(&tame_distance) = tames.get(&wild) {
                let discrete_log = Self::collision(tame_distance, wild_distance);
                return Solution::new(discrete_log, group_ops);
            }
            wilds.insert(wild, wild_distance);
        }
    }
}

impl DiscreteLogSolver for Basic {
    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, rng: &mut impl Rng) -> Solution {
        let n = h - l;
        let n_half = n / 2;
        let shift = l + n_half;
        let element = element - generator_scalar_mul_i64::<G>(shift);

        let solution = self.solve_symmetric(element, n, rng);
        Solution::new(solution.discrete_log + shift, solution.group_ops)
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::{SysRng, Xoshiro256PlusPlus};
    use rand::{RngExt, SeedableRng};
    use std::error::Error;

    use super::*;
    use crate::group::generator_scalar_mul_i64;
    use crate::group::toy::ToyGroup;

    #[test]
    fn solves_toy_group_interval_32_bits() -> Result<(), Box<dyn Error>> {
        const N_BITS: u32 = 32;

        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let low = rng.random_range(-(1 << 32)..(1 << 32));
        let high = low + (1 << N_BITS);
        let x = rng.random_range(low..high);
        let element = generator_scalar_mul_i64::<ToyGroup>(x);
        let solver = Basic;
        let result = solver.solve(element, low, high, &mut rng);
        assert_eq!(result.discrete_log(), x);

        let n = (1u64 << N_BITS) as f64;
        let k = result.group_ops() as f64 / n.sqrt();
        println!("k = {:.02}", k);

        Ok(())
    }
}
