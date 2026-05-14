pub mod gaudry_schost_sim;
pub mod pollard_four_kangaroo;
pub mod pollard_kangaroo;
pub mod pollard_three_kangaroo;

use rand::{Rng, RngExt};

use crate::dlog::stats::{RunningStats, Summary};
use crate::group::KangarooGroup;
use crate::group::helpers::generator_scalar_mul_i64;

pub struct Solution {
    pub dlog: i64,
    pub iters: u64,
}

pub trait DiscreteLogSolver {
    fn solve<G: KangarooGroup>(
        &self,
        low_bound_inclusive: i64,
        high_bound_exclusive: i64,
        element: G,
        rng: &mut impl Rng,
    ) -> Solution;
}

pub fn run_solver_stats<G, S>(
    solver: &mut S,
    range_bits: u32,
    iters: u64,
    rng: &mut impl Rng,
) -> Summary
where
    G: KangarooGroup,
    S: DiscreteLogSolver,
{
    assert!(range_bits < i64::BITS - 1);

    let mut stats = RunningStats::new();
    let low = 0;
    let high = 1i64 << range_bits;

    for _ in 0..iters {
        let dlog = rng.random_range(low..high);
        let elem = generator_scalar_mul_i64::<G>(dlog);
        let result = solver.solve(low, high, elem, rng);
        if result.dlog != dlog {
            println!("result: {}, expected: {}", result.dlog, dlog);
        }
        assert_eq!(result.dlog, dlog);
        let k = (result.iters as f64) / ((high - low) as f64).sqrt();
        stats.update(k);
    }

    stats.summary()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dlog::solver::pollard_four_kangaroo::PollardFourKangarooSolver;
    use crate::dlog::solver::pollard_kangaroo::PollardKangarooSolver;
    use crate::dlog::solver::pollard_three_kangaroo::PollardThreeKangarooSolver;
    use crate::group::ectoy::ECToyGroup;
    use crate::group::toy::ToyGroup;
    use rand::SeedableRng;
    use rand::rngs::{SysRng, Xoshiro256PlusPlus};
    use std::error::Error;

    const RANGE_BITS: u32 = 48;
    const ITERS: u64 = 128;
    const BUCKET_BITS: u32 = 8;
    const DP_BITS: u32 = 4;

    #[test]
    fn run_solver_stats_runs_pollard_kangaroo() -> Result<(), Box<dyn Error>> {
        let mut solver = PollardKangarooSolver::new(BUCKET_BITS, DP_BITS);
        run_solver_stats_test::<ToyGroup>("pollard kangaroo", &mut solver)
    }

    #[test]
    fn run_solver_stats_runs_pollard_three_kangaroo() -> Result<(), Box<dyn Error>> {
        let mut solver = PollardThreeKangarooSolver::new(BUCKET_BITS, DP_BITS);
        run_solver_stats_test::<ToyGroup>("pollard three kangaroo", &mut solver)
    }

    #[test]
    fn run_solver_stats_runs_pollard_four_kangaroo() -> Result<(), Box<dyn Error>> {
        let mut solver = PollardFourKangarooSolver::new(BUCKET_BITS, DP_BITS);
        run_solver_stats_test::<ToyGroup>("pollard four kangaroo", &mut solver)
    }

    #[test]
    fn run_solver_stats_runs_pollard_kangaroo_ectoy() -> Result<(), Box<dyn Error>> {
        let mut solver = PollardKangarooSolver::new(BUCKET_BITS, DP_BITS);
        run_solver_stats_test::<ECToyGroup>("pollard kangaroo ectoy", &mut solver)
    }

    #[test]
    fn run_solver_stats_runs_pollard_three_kangaroo_ectoy() -> Result<(), Box<dyn Error>> {
        let mut solver = PollardThreeKangarooSolver::new(BUCKET_BITS, DP_BITS);
        run_solver_stats_test::<ECToyGroup>("pollard three kangaroo ectoy", &mut solver)
    }

    #[test]
    fn run_solver_stats_runs_pollard_four_kangaroo_ectoy() -> Result<(), Box<dyn Error>> {
        let mut solver = PollardFourKangarooSolver::new(BUCKET_BITS, DP_BITS);
        run_solver_stats_test::<ECToyGroup>("pollard four kangaroo ectoy", &mut solver)
    }

    fn run_solver_stats_test<G: KangarooGroup>(
        name: &str,
        solver: &mut impl DiscreteLogSolver,
    ) -> Result<(), Box<dyn Error>> {
        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let summary = run_solver_stats::<G, _>(solver, RANGE_BITS, ITERS, &mut rng);
        println!(
            "{name} run_solver_stats summary: n={}, mean={:.2}, std_dev={:.2}, median={:.2}",
            summary.n, summary.mean, summary.std_dev, summary.median,
        );

        assert_eq!(summary.n, ITERS);
        assert!(summary.mean.is_finite());
        assert!(summary.mean > 0.0);
        assert!(summary.std_dev.is_finite());
        assert!(summary.median.is_finite());
        Ok(())
    }
}

// pub mod pollard_kangaroo;
// pub mod pollard_three_kangaroo;
// pub mod pollard_four_kangaroo;
// pub mod gaudry_schost_sim;
//
// use std::hash::Hash;
// use std::time::{Duration, Instant};
// use rand::{Rng, RngExt};
// use crate::dlog::stats::{RunningStats, Summary};
// use crate::group::{group_scalar_mul, CyclicGroup, KangarooGroup};
// use crate::group::toy::ToyGroup;
//
// pub struct Solution {
//     pub dlog: i64,
//     pub iters: u64
// }
//
// pub trait DiscreteLogSolver {
//     fn solve<G: KangarooGroup + Hash>(&self, low_bound_inclusive: i64, high_bound_exclusive: i64, element: G, rng: &mut impl Rng) -> Solution;
// }
//
// pub fn run_solver_stats<S: DiscreteLogSolver>(solver: &mut S, range_bits: u32, iters: u64, rng: &mut impl Rng) -> Summary {
//     let mut stats = RunningStats::new();
//     let low = 0;
//     let high = 1 << range_bits;
//
//     let mut last = Instant::now();
//     for i in 0..iters {
//         let dlog = rng.random_range(low..high);
//         let elem = group_scalar_mul(ToyGroup::generator(), dlog);
//         let result = solver.solve(low, high, elem, rng);
//         if result.dlog != dlog {
//             println!("result: {}, expected: {}", result.dlog, dlog);
//         }
//         assert_eq!(result.dlog, dlog);
//         let k = (result.iters as f64) / ((high - low) as f64).sqrt();
//         stats.update(k);
//     }
//
//     stats.summary()
// }
