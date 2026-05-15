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
