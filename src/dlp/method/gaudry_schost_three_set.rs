use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::{Rng, RngExt};
use std::collections::HashMap;

pub struct GaudrySchostImprovedThreeSetIdeal;

impl Default for GaudrySchostImprovedThreeSetIdeal {
    fn default() -> Self {
        Self::new()
    }
}

impl GaudrySchostImprovedThreeSetIdeal {
    const GAMMA: f64 = 0.588;
    const ALPHA: f64 = Self::GAMMA / 2.0;

    pub fn new() -> Self {
        GaudrySchostImprovedThreeSetIdeal
    }

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

impl DiscreteLogSolver for GaudrySchostImprovedThreeSetIdeal {
    fn solve_symmetric<G: KangarooGroup>(
        &self,
        element: G,
        n: i64,
        rng: &mut impl Rng,
    ) -> Solution {
        let n_half = n / 2;
        let tame_low = ((n_half as f64) * Self::ALPHA).floor() as i64;
        let tame_high = ((n_half as f64) * (Self::ALPHA + Self::GAMMA)).ceil() as i64;
        let wild_low = (-(n_half as f64) * (Self::GAMMA / 2.)).floor() as i64;
        let wild_high = ((n_half as f64) * (Self::GAMMA / 2.)).ceil() as i64;

        let mut group_ops = 0;
        let mut tames = HashMap::new();
        let mut wilds1 = HashMap::new();
        let mut wilds2 = HashMap::new();
        loop {
            let tame_distance = rng.random_range(tame_low..tame_high);
            let tame = generator_scalar_mul_i64::<G>(tame_distance);
            tames.insert(tame, tame_distance);
            group_ops += 1;
            if let Some(&wild1_distance) = wilds1.get(&tame) {
                let discrete_log = Self::collision_tw1(tame_distance, wild1_distance);
                return Solution::new(discrete_log, group_ops);
            }
            if let Some(&wild2_distance) = wilds2.get(&tame) {
                let discrete_log = Self::collision_tw2(tame_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }

            let wild1_distance = rng.random_range(wild_low..wild_high);
            let wild1 = element + generator_scalar_mul_i64::<G>(wild1_distance);
            wilds1.insert(wild1, wild1_distance);
            group_ops += 1;
            if let Some(&tame_distance) = tames.get(&wild1) {
                let discrete_log = Self::collision_tw1(tame_distance, wild1_distance);
                return Solution::new(discrete_log, group_ops);
            }
            if let Some(&wild2_distance) = wilds2.get(&wild1) {
                let discrete_log = Self::collision_w1w2(wild1_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }

            let wild2_distance = rng.random_range(wild_low..wild_high);
            let wild2 = -element + generator_scalar_mul_i64::<G>(wild2_distance);
            wilds2.insert(wild2, wild2_distance);
            group_ops += 1;
            if let Some(&tame_distance) = tames.get(&wild2) {
                let discrete_log = Self::collision_tw2(tame_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }
            if let Some(&wild1_distance) = wilds1.get(&wild2) {
                let discrete_log = Self::collision_w1w2(wild1_distance, wild2_distance);
                return Solution::new(discrete_log, group_ops);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::toy::ToyGroup;
    use rand::SeedableRng;
    use rand::rngs::{SysRng, Xoshiro256PlusPlus};
    use std::error::Error;

    #[test]
    fn solves_toy_group_interval_32_bits() -> Result<(), Box<dyn Error>> {
        const N_BITS: u32 = 32;

        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;

        for _ in 1..128 {
            let low = rng.random_range(-(1 << 48)..(1 << 48));
            let high = low + (1 << N_BITS);
            let x = rng.random_range(low..high);
            let element = generator_scalar_mul_i64::<ToyGroup>(x);
            let solver = GaudrySchostImprovedThreeSetIdeal::new();
            let result = solver.solve(element, low, high, &mut rng);
            assert_eq!(result.discrete_log(), x);

            let n = (1u64 << N_BITS) as f64;
            let k = (result.group_ops() as f64) / n.sqrt();
            println!("k = {:.02}", k);
        }
        Ok(())
    }
}
