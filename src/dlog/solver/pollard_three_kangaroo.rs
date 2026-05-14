use std::collections::HashMap;

use rand::Rng;

use crate::dlog::jump::generate_uniform_jump_table_distances;
use crate::dlog::solver::{DiscreteLogSolver, Solution};
use crate::group::KangarooGroup;
use crate::group::helpers::generator_scalar_mul_i64;

pub struct PollardThreeKangarooSolver {
    phi_bits: u32,
    dp_bits: u32,
}

impl PollardThreeKangarooSolver {
    pub fn new(bucket_bits: u32, dp_bits: u32) -> Self {
        PollardThreeKangarooSolver {
            phi_bits: bucket_bits,
            dp_bits,
        }
    }

    fn collision_tame_wild1(
        &self,
        low: i64,
        wild_offset: i64,
        tame_position: i64,
        wild1_distance: i64,
        iters: u64,
    ) -> Solution {
        Solution {
            dlog: low + wild_offset + tame_position - wild1_distance,
            iters,
        }
    }

    fn collision_tame_wild2(
        &self,
        low: i64,
        wild_offset: i64,
        tame_position: i64,
        wild2_distance: i64,
        iters: u64,
    ) -> Solution {
        Solution {
            dlog: low + wild_offset + wild2_distance - tame_position,
            iters,
        }
    }

    fn collision_wild(
        &self,
        low: i64,
        interval_width: i64,
        wild1_distance: i64,
        wild2_distance: i64,
        iters: u64,
    ) -> Solution {
        Solution {
            dlog: low + (interval_width + wild2_distance - wild1_distance) / 2,
            iters,
        }
    }
}

impl DiscreteLogSolver for PollardThreeKangarooSolver {
    fn solve<G: KangarooGroup>(
        &self,
        low_bound_inclusive: i64,
        high_bound_exclusive: i64,
        element: G,
        rng: &mut impl Rng,
    ) -> Solution {
        assert!(high_bound_exclusive > low_bound_inclusive);

        let n = high_bound_exclusive - low_bound_inclusive;
        let tame_start = 3 * n / 10;
        let wild_offset = n / 2;
        let shifted_element = element - generator_scalar_mul_i64::<G>(low_bound_inclusive);
        let m = 0.375 * (n as f64).sqrt();
        let jump_table_distances = generate_uniform_jump_table_distances(m, self.phi_bits, rng);
        let jump_table: Vec<_> = jump_table_distances
            .into_iter()
            .map(|d| (d, generator_scalar_mul_i64::<G>(d)))
            .collect();

        let mut iters = 0u64;
        let mut t = generator_scalar_mul_i64::<G>(tame_start);
        let mut t_distance = 0i64;
        let mut w1 = shifted_element - generator_scalar_mul_i64::<G>(wild_offset);
        let mut w1_distance = 0i64;
        let mut w2 = -w1;
        let mut w2_distance = 0i64;
        let mut tames = HashMap::new();
        let mut wilds1 = HashMap::new();
        let mut wilds2 = HashMap::new();

        loop {
            let tame_bucket = t.bucket(self.phi_bits);
            let (tame_jump_distance, tame_jump) = jump_table[tame_bucket];
            t_distance += tame_jump_distance;
            t += tame_jump;
            iters += 1;
            if t.is_distinguished(self.dp_bits) {
                tames.insert(t, t_distance);
                let tame_position = tame_start + t_distance;
                if let Some(&w1_distance) = wilds1.get(&t) {
                    return self.collision_tame_wild1(
                        low_bound_inclusive,
                        wild_offset,
                        tame_position,
                        w1_distance,
                        iters,
                    );
                }
                if let Some(&w2_distance) = wilds2.get(&t) {
                    return self.collision_tame_wild2(
                        low_bound_inclusive,
                        wild_offset,
                        tame_position,
                        w2_distance,
                        iters,
                    );
                }
            }

            let wild1_bucket = w1.bucket(self.phi_bits);
            let (wild1_jump_distance, wild1_jump) = jump_table[wild1_bucket];
            w1_distance += wild1_jump_distance;
            w1 += wild1_jump;
            iters += 1;
            if w1.is_distinguished(self.dp_bits) {
                wilds1.insert(w1, w1_distance);
                if let Some(&t_distance) = tames.get(&w1) {
                    return self.collision_tame_wild1(
                        low_bound_inclusive,
                        wild_offset,
                        tame_start + t_distance,
                        w1_distance,
                        iters,
                    );
                }
                if let Some(&w2_distance) = wilds2.get(&w1) {
                    return self.collision_wild(
                        low_bound_inclusive,
                        n,
                        w1_distance,
                        w2_distance,
                        iters,
                    );
                }
            }

            let wild2_bucket = w2.bucket(self.phi_bits);
            let (wild2_jump_distance, wild2_jump) = jump_table[wild2_bucket];
            w2_distance += wild2_jump_distance;
            w2 += wild2_jump;
            iters += 1;
            if w2.is_distinguished(self.dp_bits) {
                wilds2.insert(w2, w2_distance);
                if let Some(&t_distance) = tames.get(&w2) {
                    return self.collision_tame_wild2(
                        low_bound_inclusive,
                        wild_offset,
                        tame_start + t_distance,
                        w2_distance,
                        iters,
                    );
                }
                if let Some(&w1_distance) = wilds1.get(&w2) {
                    return self.collision_wild(
                        low_bound_inclusive,
                        n,
                        w1_distance,
                        w2_distance,
                        iters,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::Xoshiro256PlusPlus;
    use rand::{RngExt, SeedableRng};

    use super::*;
    use crate::group::helpers::generator_scalar_mul_i64;
    use crate::group::toy::ToyGroup;

    #[test]
    fn solves_toy_group_interval_2_32() {
        const LOW: i64 = 0;
        const HIGH: i64 = 1i64 << 32;
        const BUCKET_BITS: u32 = 8;
        const DP_BITS: u32 = 4;

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0x7a1e_3eed_cafe_babe);
        let dlog = rng.random_range(LOW..HIGH);
        let element = generator_scalar_mul_i64::<ToyGroup>(dlog);
        let solver = PollardThreeKangarooSolver::new(BUCKET_BITS, DP_BITS);

        let result = solver.solve(LOW, HIGH, element, &mut rng);
        let n = (HIGH - LOW) as f64;
        let k = result.iters as f64 / n.sqrt();
        println!(
            "pollard three kangaroo toy interval [0, 2^32): dlog={}, iters={}, k={:.6}",
            dlog, result.iters, k
        );

        assert_eq!(result.dlog, dlog);
    }
}

// use std::collections::{HashMap, HashSet};
// use std::hash::Hash;
// use rand::Rng;
// use crate::dlog::jump::generate_uniform_jump_table_distances;
// use crate::dlog::solver::{DiscreteLogSolver, Solution};
// use crate::group::{group_scalar_mul, KangarooGroup};
//
// pub struct PollardThreeKangarooSolver {
//     phi_bits: u32,
//     dp_bits: u32,
// }
//
// impl PollardThreeKangarooSolver {
//     pub fn new(bucket_bits: u32, dp_bits: u32) -> Self {
//         PollardThreeKangarooSolver {
//             phi_bits: bucket_bits,
//             dp_bits,
//         }
//     }
//
//     fn collision_tame(&self, tame_distance: i64, wild1_distance: i64, iters: u64) -> Solution {
//         let dlog = tame_distance - wild1_distance;
//         Solution {
//             dlog,
//             iters,
//         }
//     }
//
//     fn collision_wild(&self, wild1_distance: i64, wild2_distance: i64, iters: u64) -> Solution {
//         let dlog = (wild2_distance - wild1_distance) / 2;
//         Solution {
//             dlog,
//             iters,
//         }
//     }
// }
//
// impl DiscreteLogSolver for PollardThreeKangarooSolver {
//     fn solve<G: KangarooGroup + Hash>(&self, low_bound_inclusive: i64, high_bound_exclusive: i64, element: G, rng: &mut impl Rng) -> Solution {
//         assert!(high_bound_exclusive > low_bound_inclusive);
//         let n = high_bound_exclusive - low_bound_inclusive;
//         let n_over_two = n / 2;
//         let m = 0.375 * (n as f64).sqrt();
//         let jump_table_distances = generate_uniform_jump_table_distances(m, self.phi_bits, rng);
//         let jump_table: Vec<_> = jump_table_distances.into_iter().map(|d| (d, group_scalar_mul(G::generator(), d))).collect();
//
//         let mut iters = 0u64;
//         let mut t = group_scalar_mul(G::generator(), 3 * n / 10);
//         let mut t_distance = 0i64;
//         let mut w1 = element - group_scalar_mul(G::generator(), n_over_two);
//         let mut w1_distance = 0i64;
//         let mut w2 = -w1;
//         let mut w2_distance = 0i64;
//         let mut tames = HashMap::new();
//         let mut wilds1 = HashMap::new();
//         let mut wilds2 = HashMap::<G, i64>::new();
//         loop {
//             let tame_bucket = t.bucket(self.phi_bits);
//             let (tame_jump_distance, tame_jump) = jump_table[tame_bucket];
//             t_distance += tame_jump_distance;
//             t += tame_jump;
//             iters += 1;
//             if t.is_distinguished(self.dp_bits) {
//                 tames.insert(t, t_distance);
//                 if let Some(&w1_distance) = wilds1.get(&t) {
//                     return self.collision_tame(n_over_two + 3 * n / 10 + t_distance, w1_distance, iters);
//                 }
//                 if let Some(&w2_distance) = wilds2.get(&t) {
//                     return self.collision_tame(n_over_two - (3 * n / 10 + t_distance), -w2_distance, iters);
//                 }
//             }
//
//             let wild1_bucket = w1.bucket(self.phi_bits);
//             let (wild1_jump_distance, wild1_jump) = jump_table[wild1_bucket];
//             w1_distance += wild1_jump_distance;
//             w1 += wild1_jump;
//             iters += 1;
//             if w1.is_distinguished(self.dp_bits) {
//                 wilds1.insert(w1, w1_distance);
//                 if let Some(&t_distance) = tames.get(&w1) {
//                     return self.collision_tame(n_over_two + 3 * n / 10 + t_distance, w1_distance, iters);
//                 }
//                 if let Some(&w2_distance) = wilds2.get(&w1) {
//                     return self.collision_wild(w1_distance, n + w2_distance, iters);
//                 }
//             }
//
//             let wild2_bucket = w2.bucket(self.phi_bits);
//             let (wild2_jump_distance, wild2_jump) = jump_table[wild2_bucket];
//             w2_distance += wild2_jump_distance;
//             w2 += wild2_jump;
//             iters += 1;
//             if w2.is_distinguished(self.dp_bits) {
//                 wilds2.insert(w2, w2_distance);
//                 if let Some(t_distance) = tames.get(&w2) {
//                     return self.collision_tame(n_over_two - (3 * n / 10 + t_distance), -w2_distance, iters);
//                 }
//                 if let Some(&w1_distance) = wilds1.get(&w2) {
//                     return self.collision_wild(w1_distance, n + w2_distance, iters);
//                 }
//             }
//         }
//     }
// }
