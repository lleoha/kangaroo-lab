use crate::dlp::method::gaudry_schost::params::{Params, Parity, Type, Walk};
use crate::dlp::{DiscreteLogSolver, Solution};
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::Rng;
use std::collections::HashMap;
use std::ops::ControlFlow;

pub struct SotaV2Overhead {
    alpha: f64,
    c: f64,
    buckets_bits: u32,
    dp_bits: u32,
}

impl SotaV2Overhead {
    pub fn new_unchecked(alpha: f64, c: f64, buckets_bits: u32, dp_bits: u32) -> Self {
        debug_assert!(0.0 < alpha && alpha <= 1.0);
        debug_assert!(0.0 < c);
        debug_assert!(0 < buckets_bits && buckets_bits < 32);
        debug_assert!(0 < dp_bits && dp_bits < 32);
        SotaV2Overhead {
            alpha,
            c,
            buckets_bits,
            dp_bits,
        }
    }

    fn solve_symmetric<G: KangarooGroup>(
        &self,
        element: G,
        n: i64,
        rng: &mut impl Rng,
    ) -> ControlFlow<(i64, u64)> {
        let params = Params::new(n, self.c, self.buckets_bits, self.dp_bits, rng);

        let mut group_ops = 0;
        let mut dp_map = HashMap::new();

        let mut t1 = Walk::new(params.clone(), Type::TAME, Parity::ODD, G::identity(), self.alpha, rng);
        let mut t2 = Walk::new(params.clone(), Type::TAME, Parity::EVEN, G::identity(), self.alpha, rng);
        let mut w1 = Walk::new(params.clone(), Type::WILD, Parity::EVEN, element, 1.0, rng);
        let mut w2 = Walk::new(params.clone(), Type::WILD, Parity::EVEN, element, 1.0, rng);
        let mut w3 = Walk::new(params.clone(), Type::WILD, Parity::EVEN, element, 1.0, rng);
        let mut w4 = Walk::new(params.clone(), Type::WILD, Parity::EVEN, element, 1.0, rng);

        loop {
            group_ops += t1
                .walk(&mut dp_map, rng)
                .map_break(|(dlog, ops)| (dlog, group_ops + ops))?;
            group_ops += t2
                .walk(&mut dp_map, rng)
                .map_break(|(dlog, ops)| (dlog, group_ops + ops))?;
            group_ops += w1
                .walk(&mut dp_map, rng)
                .map_break(|(dlog, ops)| (dlog, group_ops + ops))?;
            group_ops += w2
                .walk(&mut dp_map, rng)
                .map_break(|(dlog, ops)| (dlog, group_ops + ops))?;
            group_ops += w3
                .walk(&mut dp_map, rng)
                .map_break(|(dlog, ops)| (dlog, group_ops + ops))?;
            group_ops += w4
                .walk(&mut dp_map, rng)
                .map_break(|(dlog, ops)| (dlog, group_ops + ops))?;
        }
    }
}

impl DiscreteLogSolver for SotaV2Overhead {
    fn solve<G: KangarooGroup>(&self, element: G, l: i64, h: i64, rng: &mut impl Rng) -> Solution {
        let n = h - l;
        let n_half = n / 2;
        let shift = l + n_half;
        let element = element - generator_scalar_mul_i64::<G>(shift);

        let (dlog, ops) = self.solve_symmetric(element, n, rng).break_value().unwrap();
        let dlog = if element == generator_scalar_mul_i64::<G>(dlog) {
            dlog
        } else {
            -dlog
        };
        Solution::new(dlog + shift, ops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::toy::ToyGroup;
    use rand::rngs::{SysRng, Xoshiro256PlusPlus};
    use rand::{RngExt, SeedableRng};
    use std::error::Error;

    #[test]
    fn solves_toy_group_interval_32_bits() -> Result<(), Box<dyn Error>> {
        const N_BITS: u32 = 32;

        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let low = rng.random_range(-(1 << 48)..(1 << 48));
        let high = low + (1 << N_BITS);
        let x = rng.random_range(low..high);
        let element = generator_scalar_mul_i64::<ToyGroup>(x);
        let solver = SotaV2Overhead::new_unchecked(1. / 64., 1. / 4., 9, 4);
        let result = solver.solve(element, low, high, &mut rng);
        assert_eq!(result.discrete_log(), x);

        let n = (1u64 << N_BITS) as f64;
        let k = (result.group_ops() as f64) / n.sqrt();
        println!("k = {:.02}", k);

        Ok(())
    }
}
