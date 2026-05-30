// use crate::dlp::{DiscreteLogSolver, Solution};
// use crate::group::{KangarooGroup, generator_scalar_mul_i64};
// use rand::{Rng, RngExt};
// use std::collections::HashMap;
// use std::ops::Range;
// use std::slice::range;
// use crate::dlp::jump::generate_uniform_jump_table_distances;
// use crate::dlp::method::gaudry_schost::GaudrySchostParams;
//
// pub struct SotaV2 {
//     dp_bits: u32,
//     buckets_bits: u32,
//     jumps_mean_coefficient: f64,
// }
//
// struct Walker<G: KangarooGroup> {
//     distance: i64,
//     item: G,
//     anchor: G,
//     bounds: Range<i64>,
//     dp_bits: u32,
//     buckets_bits: u32,
//     jump_table: Vec<(i64, G)>,
//     cycle_escape: (i64, G),
//     group_ops: u64,
//     iters: u64,
// }
//
// impl<G: KangarooGroup> Walker<G> {
//     pub fn new(anchor: G, bounds: Range<i64>, dp_bits: u32, buckets_bits: u32, jump_table: Vec<(i64, G)>, cycle_escape: (i64, G), rng: &mut impl Rng) -> Self {
//         debug_assert_eq!(1 << buckets_bits, jump_table.len());
//
//         let (distance, item) = Self::init(anchor, bounds.clone(), rng);
//         Walker {
//             anchor,
//             distance,
//             item,
//             bounds,
//             dp_bits,
//             buckets_bits,
//             jump_table,
//             cycle_escape,
//             group_ops: 0,
//             iters: 0,
//         }
//     }
//
//     pub fn walk(&mut self, rng: &mut impl Rng) -> Option<(i64, G)> {
//         let bucket = self.item.bucket(self.buckets_bits);
//         let offset = self.jump_table[bucket];
//         self.walk_with(offset, rng)
//     }
//
//     pub fn walk_cycle_escape(&mut self, cycle_len: usize, rng: &mut impl Rng) -> Option<(i64, G)>{
//         let reference = self.item;
//         let mut escape_footprint = self.item.footprint();
//         let mut escape_distance = self.distance;
//         let mut escape_item = self.item;
//
//         for _ in 0..cycle_len {
//             if let Some(distinguished) = self.walk(rng) {
//                 return Some(distinguished)
//             }
//
//             let item_footprint = self.item.footprint();
//             if item_footprint < escape_footprint {
//                 escape_footprint = item_footprint;
//                 escape_distance = self.distance;
//                 escape_item = self.item;
//             }
//         }
//
//         if self.item == reference {
//             let escape_offset = self.cycle_escape;
//             let distinguished = self.walk_from_with((escape_distance, escape_item), escape_offset, rng);
//             self.iters = 0;
//             if distinguished.is_some() {
//                 return Some(distinguished.unwrap());
//             }
//         }
//
//         None
//     }
//
//     fn walk_with(&mut self, offset: (i64, G), rng: &mut impl Rng) -> Option<(i64, G)> {
//         let from = (self.distance, self.item);
//         self.walk_from_with(from, offset, rng)
//     }
//
//     fn walk_from_with(&mut self, from: (i64, G), offset: (i64, G), rng: &mut impl Rng) -> Option<(i64, G)> {
//         let (from_distance, from) = from;
//         let (offset_distance, offset) = offset;
//
//         self.distance = from_distance + offset_distance;
//         self.item = from + offset;
//         self.group_ops += 1;
//         self.iters += 1;
//         if !self.item.is_negation_map_representative() {
//             self.distance = -self.distance;
//             self.item = -self.item;
//         }
//
//         if self.item.is_distinguished(self.dp_bits) {
//             let distinguished_distance = self.distance;
//             let distinguished_item = self.item;
//             self.reinit(rng);
//             Some((distinguished_distance, distinguished_item))
//         } else {
//             None
//         }
//     }
//
//
//     fn reinit(&mut self, rng: &mut impl Rng) {
//         (self.distance, self.item) = Self::init(self.anchor, self.bounds.clone(), rng);
//         self.iters = 0;
//     }
//
//     fn init(anchor: G, range: Range<i64>, rng: &mut impl Rng) -> (i64, G) {
//         let mut distance = rng.random_range(range);
//         let mut item = anchor + generator_scalar_mul_i64::<G>(distance);
//         if !item.is_negation_map_representative() {
//             distance = -distance;
//             item = -item;
//         }
//         (distance, item)
//     }
// }
//
// impl SotaV2 {
//     pub fn new(dp_bits: u32, buckets_bits: u32, jumps_mean_coefficient: f64) -> Self {
//         SotaV2 {
//             dp_bits,
//             buckets_bits,
//             jumps_mean_coefficient,
//         }
//     }
//
//     fn collision_tw(tame_distance: i64, wild_distance: i64) -> i64 {
//         tame_distance - wild_distance
//     }
//
//     fn collision_w1w2(wild1_distance: i64, wild2_distance: i64) -> i64 {
//         (wild2_distance - wild1_distance) / 2
//     }
// }
//
// impl DiscreteLogSolver for SotaV2 {
//     fn solve_symmetric<G: KangarooGroup>(
//         &self,
//         element: G,
//         n: i64,
//         rng: &mut impl Rng,
//     ) -> Solution {
//         let n_half = n / 2;
//
//         let params = GaudrySchostParams::new(n, self.jumps_mean_coefficient, self.buckets_bits, self.dp_bits);
//         let jumps_distances = generate_uniform_jump_table_distances(params.absolute_jumps_mean(), params.buckets_bits(), rng);
//         let jump_table = jumps_distances.into_iter()
//             .map(|d| (d, generator_scalar_mul_i64::<G>(d)))
//             .collect::<Vec<_>>();
//
//         let m = params.absolute_jumps_mean().round() as i64;
//         let escape_distance = rng.random_range((3 * m)..(5 * m));
//         let escape = (escape_distance, generator_scalar_mul_i64::<G>(escape_distance));
//
//         let mut tame_iters = 0u64;
//         let mut tame_cycle_iters = 0u64;
//         let mut tame = Walker::new(G::identity(), params.tame_bounds(), params.dp_bits(), params.buckets_bits(), jump_table.clone(), escape, rng);
//
//         let mut wild1_iters = 0u64;
//         let mut wild2_cycle_iters = 0u64;
//         let mut wild2 = Walker::new(G::identity(), params.tame_bounds(), params.dp_bits(), params.buckets_bits(), jump_table.clone(), escape, rng);
//
//         // let mut group_ops = 0u64;
//         // let mut tame_iters = 0u64;
//         // let mut wild1_iters = 0u64;
//         // let mut wild2_iters = 0u64;
//         //
//         // let mut tames = HashMap::new();
//         // let mut wilds = HashMap::new();
//         //
//         // let mut tame_distance = rng.random_range(params.tame_bounds());
//         // let mut tame = generator_scalar_mul_i64::<G>(tame_distance);
//         // if !tame.is_negation_map_representative() {
//         //     tame_distance = -tame_distance;
//         //     tame = -tame;
//         // }
//         // let mut wild1_distance = rng.random_range(params.wild_bounds());
//         // let mut wild1 = generator_scalar_mul_i64::<G>(wild1_distance);
//         // if !wild1.is_negation_map_representative() {
//         //     wild1_distance = -wild1_distance;
//         //     wild1 = -wild1;
//         // }
//         // let mut wild2_distance = rng.random_range(params.wild_bounds());
//         // let mut wild2 = generator_scalar_mul_i64::<G>(wild2_distance);
//         // if !wild2.is_negation_map_representative() {
//         //     wild2_distance = -wild2_distance;
//         //     wild2 = -wild2;
//         // }
//
//         loop {
//             {
//                 let tame_bucket = tame.bucket(params.buckets_bits());
//                 let (tame_jump_offset, tame_jump) = jump_table[tame_bucket];
//                 tame_distance += tame_jump_offset;
//                 tame += tame_jump;
//                 group_ops += 1;
//                 if !tame.is_negation_map_representative() {
//                     tame_distance = -tame_distance;
//                     tame = -tame;
//                 }
//                 if tame.is_distinguished(params.dp_bits()) {
//                     tames.insert(tame, tame_distance);
//                     if let Some(wild_distance) = wilds.get(&tame) {
//                         panic!("*")
//                     }
//                 }
//                 tame_iters += 1;
//
//             }
//         }
//
//         // let params = GaudrySchostParams::new()
//         // let mut group_ops = 0;
//         // let mut tames = HashMap::new();
//         // let mut wilds = HashMap::new();
//         // loop {
//         //         let mut tame_distance = rng.random_range(0..n / 128);
//         //         let mut tame = generator_scalar_mul_i64::<G>(tame_distance);
//         //         if !tame.is_negation_map_representative() {
//         //             (tame_distance, tame) = (-tame_distance, -tame);
//         //         }
//         //         group_ops += 1;
//         //         if let Some(&wild_distance) = wilds.get(&tame) {
//         //             let discrete_log = Self::collision_tw(tame_distance, wild_distance);
//         //             return if generator_scalar_mul_i64::<G>(discrete_log) == element {
//         //                 Solution::new(discrete_log, group_ops)
//         //             } else {
//         //                 Solution::new(-discrete_log, group_ops)
//         //             };
//         //         }
//         //         tames.insert(tame, tame_distance);
//         //
//         //     for _ in 0..2 {
//         //         let mut wild_distance = rng.random_range(-n/2..n/2) / 2 * 2;
//         //         let mut wild = element + generator_scalar_mul_i64::<G>(wild_distance);
//         //         if !wild.is_negation_map_representative() {
//         //             (wild_distance, wild) = (-wild_distance, -wild);
//         //         }
//         //         group_ops += 1;
//         //         if let Some(&tame_distance) = tames.get(&wild) {
//         //             let discrete_log = Self::collision_tw(tame_distance, wild_distance);
//         //             return if generator_scalar_mul_i64::<G>(discrete_log) == element {
//         //                 Solution::new(discrete_log, group_ops)
//         //             } else {
//         //                 Solution::new(-discrete_log, group_ops)
//         //             };
//         //         }
//         //         if let Some(&wild2_distance) = wilds.get(&wild)
//         //             && wild_distance != wild2_distance
//         //             && wild_distance != -wild2_distance
//         //         {
//         //             let discrete_log = Self::collision_w1w2(wild_distance, wild2_distance);
//         //             return if generator_scalar_mul_i64::<G>(discrete_log) == element {
//         //                 Solution::new(discrete_log, group_ops)
//         //             } else {
//         //                 Solution::new(-discrete_log, group_ops)
//         //             };
//         //         }
//         //         wilds.insert(wild, wild_distance);
//         //     }
//         // }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::group::toy::ToyGroup;
//     use rand::SeedableRng;
//     use rand::rngs::{SysRng, Xoshiro256PlusPlus};
//     use std::error::Error;
//
//     #[test]
//     fn solves_toy_group_interval_32_bits() -> Result<(), Box<dyn Error>> {
//         const N_BITS: u32 = 32;
//
//         let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
//         let low = rng.random_range(-(1 << 48)..(1 << 48));
//         let high = low + (1 << N_BITS);
//         let x = rng.random_range(low..high);
//         let element = generator_scalar_mul_i64::<ToyGroup>(x);
//         let solver = SotaV2;
//         let result = solver.solve(element, low, high, &mut rng);
//         assert_eq!(result.discrete_log(), x);
//
//         let n = (1u64 << N_BITS) as f64;
//         let k = (result.group_ops() as f64) / n.sqrt();
//         println!("k = {:.02}", k);
//
//         Ok(())
//     }
// }
