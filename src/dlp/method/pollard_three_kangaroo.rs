use std::collections::HashMap;

use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::KangarooGroup;
use crate::group::generator_scalar_mul_i64;
use rand::{Rng, RngExt};

#[derive(Default)]
pub struct PollardThreeKangarooSolver;

impl PollardThreeKangarooSolver {
    fn collision_tw1(tame_distance: i64, wild1_distance: i64) -> i64 {
        tame_distance - wild1_distance
    }

    fn collision_tw2(tame_distance: i64, wild2_distance: i64) -> i64 {
        wild2_distance - tame_distance
    }

    fn collision_w1w2(wild1_distance: i64, wild2_distance: i64) -> i64 {
        (wild2_distance - wild1_distance) / 2
    }
}

impl DiscreteLogSolver for PollardThreeKangarooSolver {
    fn solve_symmetric<G: KangarooGroup>(
        &self,
        element: G,
        n: i64,
        rng: &mut impl Rng,
    ) -> Solution {
        let m = 0.375 * (n as f64).sqrt();
        let m2 = (2. * m).round() as i64;

        let mut group_ops = 0;
        let mut tame_distance = 3 * n / 10;
        let mut tame = generator_scalar_mul_i64::<G>(tame_distance);
        let mut wild1 = element;
        let mut wild1_distance = 0;
        let mut wild2 = -element;
        let mut wild2_distance = 0;
        let mut tames = HashMap::new();
        let mut wilds1 = HashMap::new();
        let mut wilds2 = HashMap::new();

        loop {
            let tame_jump_distance = rng.random_range(0..m2);
            let tame_jump = generator_scalar_mul_i64::<G>(tame_jump_distance);
            tame_distance += tame_jump_distance;
            tame += tame_jump;
            group_ops += 1;
            if let Some(&wild1_distance) = wilds1.get(&tame) {
                let discrete_log = Self::collision_tw1(tame_distance, wild1_distance);
                return Solution::new(discrete_log, group_ops);
            }
            if let Some(&wild2_distance) = wilds2.get(&tame) {
                let discrete_log = Self::collision_tw2(tame_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }
            tames.insert(tame, tame_distance);

            let wild1_jump_distance = rng.random_range(0..m2);
            let wild1_jump = generator_scalar_mul_i64::<G>(wild1_jump_distance);
            wild1_distance += wild1_jump_distance;
            wild1 += wild1_jump;
            group_ops += 1;
            if let Some(&tame_distance) = tames.get(&wild1) {
                let discrete_log = Self::collision_tw1(tame_distance, wild1_distance);
                return Solution::new(discrete_log, group_ops);
            }
            if let Some(&wild2_distance) = wilds2.get(&wild1) {
                let discrete_log = Self::collision_w1w2(wild1_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }
            wilds1.insert(wild1, wild1_distance);

            let wild2_jump_distance = rng.random_range(0..m2);
            let wild2_jump = generator_scalar_mul_i64::<G>(wild2_jump_distance);
            wild2_distance += wild2_jump_distance;
            wild2 += wild2_jump;
            group_ops += 1;
            if let Some(&tame_distance) = tames.get(&wild2) {
                let discrete_log = Self::collision_tw2(tame_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }
            if let Some(&wild1_distance) = wilds1.get(&wild2) {
                let discrete_log = Self::collision_w1w2(wild1_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }
            wilds2.insert(wild2, wild2_distance);
        }
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
        let low = rng.random_range(-(1 << 48)..(1 << 48));
        let high = low + (1 << N_BITS);
        let x = rng.random_range(low..high);
        let element = generator_scalar_mul_i64::<ToyGroup>(x);
        let solver = PollardThreeKangarooSolver;
        let result = solver.solve(element, low, high, &mut rng);
        assert_eq!(result.discrete_log(), x);

        let n = (1u64 << N_BITS) as f64;
        let k = result.group_ops() as f64 / n.sqrt();
        println!("k = {:.02}", k);

        Ok(())
    }
}
