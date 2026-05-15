use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::{Rng, RngExt};
use std::collections::HashMap;

#[derive(Default)]
pub struct RetiredCoderSotaV2Ideal;

impl RetiredCoderSotaV2Ideal {
    fn collision_tw(tame_distance: i64, wild_distance: i64) -> i64 {
        tame_distance - wild_distance
    }

    fn collision_w1w2(wild1_distance: i64, wild2_distance: i64) -> i64 {
        (wild2_distance - wild1_distance) / 2
    }
}

impl DiscreteLogSolver for RetiredCoderSotaV2Ideal {
    fn solve_symmetric<G: KangarooGroup>(
        &self,
        element: G,
        n: i64,
        rng: &mut impl Rng,
    ) -> Solution {
        let n_half = n / 2;

        let mut group_ops = 0;
        let mut tames = HashMap::new();
        let mut wilds = HashMap::new();
        loop {
            let mut tame_distance = rng.random_range(-n_half / 256..n_half / 256);
            let mut tame = generator_scalar_mul_i64::<G>(tame_distance);
            if !tame.is_negation_map_representative() {
                (tame_distance, tame) = (-tame_distance, -tame);
            }
            group_ops += 1;
            if let Some(&wild_distance) = wilds.get(&tame) {
                let discrete_log = Self::collision_tw(tame_distance, wild_distance);
                return if generator_scalar_mul_i64::<G>(discrete_log) == element {
                    Solution::new(discrete_log, group_ops)
                } else {
                    Solution::new(-discrete_log, group_ops)
                };
            }
            tames.insert(tame, tame_distance);

            for _ in 0..2 {
                let mut wild_distance = rng.random_range(-n_half..n_half) / 2 * 2;
                let mut wild = element + generator_scalar_mul_i64::<G>(wild_distance);
                if !wild.is_negation_map_representative() {
                    (wild_distance, wild) = (-wild_distance, -wild);
                }
                group_ops += 1;
                if let Some(&tame_distance) = tames.get(&wild) {
                    let discrete_log = Self::collision_tw(tame_distance, wild_distance);
                    return if generator_scalar_mul_i64::<G>(discrete_log) == element {
                        Solution::new(discrete_log, group_ops)
                    } else {
                        Solution::new(-discrete_log, group_ops)
                    };
                }
                if let Some(&wild2_distance) = wilds.get(&wild)
                    && wild_distance != wild2_distance
                    && wild_distance != -wild2_distance
                {
                    let discrete_log = Self::collision_w1w2(wild_distance, wild2_distance);
                    return if generator_scalar_mul_i64::<G>(discrete_log) == element {
                        Solution::new(discrete_log, group_ops)
                    } else {
                        Solution::new(-discrete_log, group_ops)
                    };
                }
                wilds.insert(wild, wild_distance);
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
        let low = rng.random_range(-(1 << 48)..(1 << 48));
        let high = low + (1 << N_BITS);
        let x = rng.random_range(low..high);
        let element = generator_scalar_mul_i64::<ToyGroup>(x);
        let solver = RetiredCoderSotaV2Ideal;
        let result = solver.solve(element, low, high, &mut rng);
        assert_eq!(result.discrete_log(), x);

        let n = (1u64 << N_BITS) as f64;
        let k = (result.group_ops() as f64) / n.sqrt();
        println!("k = {:.02}", k);

        Ok(())
    }
}
