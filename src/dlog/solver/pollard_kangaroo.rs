use std::collections::HashMap;

use rand::Rng;

use crate::dlog::jump::generate_uniform_jump_table_distances;
use crate::dlog::solver::{DiscreteLogSolver, Solution};
use crate::group::KangarooGroup;
use crate::group::helpers::generator_scalar_mul_i64;

pub struct PollardKangarooSolver {
    phi_bits: u32,
    dp_bits: u32,
}

impl PollardKangarooSolver {
    pub fn new(bucket_bits: u32, dp_bits: u32) -> Self {
        PollardKangarooSolver {
            phi_bits: bucket_bits,
            dp_bits,
        }
    }

    fn collision(&self, tame_distance: i64, wild_distance: i64, iters: u64) -> Solution {
        Solution {
            dlog: tame_distance - wild_distance,
            iters,
        }
    }
}

impl DiscreteLogSolver for PollardKangarooSolver {
    fn solve<G: KangarooGroup>(
        &self,
        low_bound_inclusive: i64,
        high_bound_exclusive: i64,
        element: G,
        rng: &mut impl Rng,
    ) -> Solution {
        assert!(high_bound_exclusive > low_bound_inclusive);

        let n = high_bound_exclusive - low_bound_inclusive;
        let tame_start = low_bound_inclusive + (n / 2);
        let m = 0.5 * (n as f64).sqrt();
        let jump_table_distances = generate_uniform_jump_table_distances(m, self.phi_bits, rng);
        let jump_table: Vec<_> = jump_table_distances
            .into_iter()
            .map(|d| (d, generator_scalar_mul_i64::<G>(d)))
            .collect();

        let mut iters = 0u64;
        let mut t = generator_scalar_mul_i64::<G>(tame_start);
        let mut t_distance = 0i64;
        let mut w = element;
        let mut w_distance = 0i64;
        let mut tames = HashMap::new();
        let mut wilds = HashMap::new();

        loop {
            let tame_bucket = t.bucket(self.phi_bits);
            let (tame_jump_distance, tame_jump) = jump_table[tame_bucket];
            t_distance += tame_jump_distance;
            t += tame_jump;
            iters += 1;
            if t.is_distinguished(self.dp_bits) {
                tames.insert(t, t_distance);
                if let Some(&wild_distance) = wilds.get(&t) {
                    return self.collision(tame_start + t_distance, wild_distance, iters);
                }
            }

            let wild_bucket = w.bucket(self.phi_bits);
            let (wild_jump_distance, wild_jump) = jump_table[wild_bucket];
            w_distance += wild_jump_distance;
            w += wild_jump;
            iters += 1;
            if w.is_distinguished(self.dp_bits) {
                wilds.insert(w, w_distance);
                if let Some(&tame_distance) = tames.get(&w) {
                    return self.collision(tame_start + tame_distance, w_distance, iters);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::{SysRng, Xoshiro256PlusPlus};
    use rand::{RngExt, SeedableRng};
    use std::error::Error;

    use super::*;
    use crate::group::ectoy::ECToyGroup;
    use crate::group::helpers::generator_scalar_mul_i64;
    use crate::group::toy::ToyGroup;

    #[test]
    fn solves_toy_group_interval_2_32() -> Result<(), Box<dyn Error>> {
        const LOW: i64 = 0;
        const HIGH: i64 = 1i64 << 48;
        const BUCKET_BITS: u32 = 8;
        const DP_BITS: u32 = 4;

        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let dlog = rng.random_range(LOW..HIGH);
        let element = generator_scalar_mul_i64::<ToyGroup>(dlog);
        let solver = PollardKangarooSolver::new(BUCKET_BITS, DP_BITS);

        let result = solver.solve(LOW, HIGH, element, &mut rng);
        let n = (HIGH - LOW) as f64;
        let k = result.iters as f64 / n.sqrt();
        println!(
            "pollard kangaroo toy interval [0, 2^32): dlog={}, iters={}, k={:.6}",
            dlog, result.iters, k
        );

        assert_eq!(result.dlog, dlog);
        Ok(())
    }

    #[test]
    fn solves_ectoy_group_small_interval() -> Result<(), Box<dyn Error>> {
        const LOW: i64 = 0;
        const HIGH: i64 = 1i64 << 32;
        const BUCKET_BITS: u32 = 8;
        const DP_BITS: u32 = 4;

        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let dlog = rng.random_range(LOW..HIGH);
        let element = generator_scalar_mul_i64::<ECToyGroup>(dlog);
        let solver = PollardKangarooSolver::new(BUCKET_BITS, DP_BITS);

        let result = solver.solve(LOW, HIGH, element, &mut rng);
        let n = (HIGH - LOW) as f64;
        let k = result.iters as f64 / n.sqrt();
        println!(
            "pollard kangaroo ectoy interval [0, 2^16): dlog={}, iters={}, k={:.2}",
            dlog, result.iters, k
        );

        assert_eq!(result.dlog, dlog);
        Ok(())
    }
}

// use std::collections::{HashMap, HashSet};
// use std::hash::Hash;
// use rand::Rng;
// use crate::dlog::jump::generate_uniform_jump_table_distances;
// use crate::dlog::solver::{DiscreteLogSolver, Solution};
// use crate::group::{group_scalar_mul, KangarooGroup};
//
// pub struct PollardKangarooSolver {
//     phi_bits: u32,
//     dp_bits: u32,
// }
//
// impl PollardKangarooSolver {
//     pub fn new(bucket_bits: u32, dp_bits: u32) -> Self {
//         PollardKangarooSolver {
//             phi_bits: bucket_bits,
//             dp_bits,
//         }
//     }
//
//     fn collision(&self, tame_distance: i64, wild_distance: i64, iters: u64) -> Solution {
//         let dlog = tame_distance - wild_distance;
//         Solution {
//             dlog,
//             iters,
//         }
//     }
// }
//
// impl DiscreteLogSolver for PollardKangarooSolver {
//     fn solve<G: KangarooGroup + Hash>(&self, low_bound_inclusive: i64, high_bound_exclusive: i64, element: G, rng: &mut impl Rng) -> Solution {
//         assert!(high_bound_exclusive > low_bound_inclusive);
//         let n = high_bound_exclusive - low_bound_inclusive;
//         let n_over_two = n / 2;
//         let m = 0.5 * (n as f64).sqrt();
//         let jump_table_distances = generate_uniform_jump_table_distances(m, self.phi_bits, rng);
//         let jump_table: Vec<_> = jump_table_distances.into_iter().map(|d| (d, group_scalar_mul(G::generator(), d))).collect();
//
//         let mut iters = 0u64;
//         let mut t = group_scalar_mul(G::generator(), n_over_two);
//         let mut t_distance = 0i64;
//         let mut w = element;
//         let mut w_distance = 0i64;
//         let mut tames = HashMap::new();
//         let mut wilds = HashMap::new();
//         loop {
//             let tame_bucket = t.bucket(self.phi_bits);
//             let (tame_jump_distance, tame_jump) = jump_table[tame_bucket];
//             t_distance += tame_jump_distance;
//             t += tame_jump;
//             iters += 1;
//             if t.is_distinguished(self.dp_bits) {
//                 tames.insert(t, t_distance);
//                 if let Some(&wild_distance) = wilds.get(&t) {
//                     return self.collision(n_over_two + t_distance, wild_distance, iters);
//                 }
//             }
//
//             let wild_bucket = w.bucket(self.phi_bits);
//             let (wild_jump_distance, wild_jump) = jump_table[wild_bucket];
//             w_distance += wild_jump_distance;
//             w += wild_jump;
//             iters += 1;
//             if w.is_distinguished(self.dp_bits) {
//                 wilds.insert(w, w_distance);
//                 if let Some(tame_distance) = tames.get(&w) {
//                     return self.collision(n_over_two + tame_distance, w_distance, iters);
//                 }
//             }
//         }
//     }
// }
