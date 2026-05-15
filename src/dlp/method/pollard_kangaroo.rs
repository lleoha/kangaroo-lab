use std::collections::HashMap;

use crate::dlp::jump::generate_uniform_jump_table_distances;
use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::KangarooGroup;
use crate::group::generator_scalar_mul_i64;
use rand::Rng;

pub struct PollardKangarooSolver {
    bucket_bits: u32,
    dp_bits: u32,
}

impl PollardKangarooSolver {
    pub fn new(bucket_bits: u32, dp_bits: u32) -> Self {
        PollardKangarooSolver {
            bucket_bits,
            dp_bits,
        }
    }

    fn collision(tame_distance: i64, wild_distance: i64) -> i64 {
        tame_distance - wild_distance
    }
}

impl DiscreteLogSolver for PollardKangarooSolver {
    fn solve_symmetric<G: KangarooGroup>(
        &self,
        element: G,
        n: i64,
        rng: &mut impl Rng,
    ) -> Solution {
        let m = 0.5 * (n as f64).sqrt();
        let jump_table_distances = generate_uniform_jump_table_distances(m, self.bucket_bits, rng);
        let jump_table: Vec<_> = jump_table_distances
            .into_iter()
            .map(|d| (d, generator_scalar_mul_i64::<G>(d)))
            .collect();

        let mut group_ops = 0;
        let mut tame = G::generator();
        let mut tame_distance = 1;
        let mut wild = element;
        let mut wild_distance = 0;
        let mut tames = HashMap::new();
        let mut wilds = HashMap::new();

        loop {
            let tame_bucket = tame.bucket(self.bucket_bits);
            let (tame_jump_distance, tame_jump) = jump_table[tame_bucket];
            tame_distance += tame_jump_distance;
            tame += tame_jump;
            group_ops += 1;
            // if tame.is_distinguished(self.dp_bits) {
            tames.insert(tame, tame_distance);
            if let Some(&w_distance) = wilds.get(&tame) {
                let discrete_log = Self::collision(tame_distance, w_distance);
                return Solution::new(discrete_log, group_ops);
            }
            // }

            let wild_bucket = wild.bucket(self.bucket_bits);
            let (wild_jump_distance, wild_jump) = jump_table[wild_bucket];
            wild_distance += wild_jump_distance;
            wild += wild_jump;
            group_ops += 1;
            // if wild.is_distinguished(self.dp_bits) {
            wilds.insert(wild, wild_distance);
            if let Some(&tame_distance) = tames.get(&wild) {
                let discrete_log = Self::collision(tame_distance, wild_distance);
                return Solution::new(discrete_log, group_ops);
            }
            // }
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
        const N_BITS: u32 = 48;
        const BUCKET_BITS: u32 = 8;
        const DP_BITS: u32 = 4;

        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let low = rng.random_range(-(1 << 32)..(1 << 32));
        let high = low + (1 << N_BITS);
        let x = rng.random_range(low..high);
        let element = generator_scalar_mul_i64::<ToyGroup>(x);
        let solver = PollardKangarooSolver::new(BUCKET_BITS, DP_BITS);

        let result = solver.solve(element, low, high, &mut rng);
        assert_eq!(result.discrete_log(), x);
        let n = (1u64 << N_BITS) as f64;
        let k = result.group_ops() as f64 / n.sqrt();
        println!("k = {:.02}", k);

        Ok(())
    }
}
