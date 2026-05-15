use num_integer::gcd;
use rand::{Rng, RngExt};

pub fn generate_uniform_jump_table_distances(
    mean: f64,
    bucket_bits: u32,
    rng: &mut impl Rng,
) -> Vec<i64> {
    let table_size = 1 << bucket_bits;
    let low = 1;
    let high = (2. * mean).ceil() as i64;
    let expected_sum = (mean * table_size as f64).round() as i64;

    loop {
        let mut distances = Vec::with_capacity(table_size);
        let mut running_sum = 0;
        while distances.len() < table_size - 1 {
            let distance = rng.random_range(low..high);
            running_sum += distance;
            distances.push(distance);
        }

        let last_sample = expected_sum - running_sum;
        if last_sample < 1 {
            continue;
        }

        distances.push(last_sample);
        let g = distances.iter().copied().reduce(gcd).unwrap_or(0);
        if g != 1 {
            continue;
        }

        return distances;
    }
}

// use num_integer::gcd;
// use rand::{Rng, RngExt};
//
// pub fn generate_uniform_jump_table_distances(mean: f64, bucket_bits: u32, rng: &mut impl Rng) -> Vec<i64> {
//     let table_size = 1 << bucket_bits;
//     let l = 1;
//     let h = (2. * mean).ceil() as i64;
//     let expected_sum = (mean * table_size as f64).round() as i64;
//
//     loop {
//         let mut distances = Vec::with_capacity(table_size);
//         let mut running_sum = 0;
//         while distances.len() < (table_size - 1) {
//             let distance = rng.random_range(l..h);
//             running_sum += distance;
//             distances.push(distance);
//         }
//         let last_sample = expected_sum - running_sum;
//         if last_sample < 1 {
//             continue;
//         }
//         distances.push(last_sample);
//         let g = distances.iter().copied().reduce(gcd).unwrap_or(0);
//         if g != 1 {
//             continue;
//         }
//
//         return distances;
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use rand::rngs::Xoshiro256PlusPlus;
//     use rand::SeedableRng;
//     use statrs::assert_almost_eq;
//     use crate::dlp::stats::RunningStats;
//     use super::*;
//
//     #[test]
//     fn test_jump_distances() {
//         const MEAN: f64 = 4096.;
//         const BUCKET_BITS: u32 = 8;
//
//         let mut rng = Xoshiro256PlusPlus::seed_from_u64(0xfacefeed12345678);
//         let jump_distances = generate_uniform_jump_table_distances(MEAN, BUCKET_BITS, &mut rng);
//         let mut stats = RunningStats::new();
//         for sample in jump_distances {
//             stats.update(sample as f64);
//         }
//         let summary = stats.summary();
//
//         assert_eq!(summary.n, 1 << BUCKET_BITS);
//         assert_almost_eq!(summary.mean, MEAN, 0.5);
//     }
// }
