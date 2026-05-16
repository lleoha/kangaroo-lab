use rand::rngs::{SysRng, Xoshiro256PlusPlus};
use rand::{Rng, RngExt, SeedableRng};
use std::ops::Range;
use crate::dlp::DiscreteLogSolver;
use crate::group::{generator_scalar_mul_i64, KangarooGroup};

#[derive(Clone, Debug)]
pub struct Statistics {
    pub n: u64,
    pub mean: f64,
    pub std_dev: f64,
    pub mean_ci_95: Range<f64>,
    pub median: f64,
}

#[derive(Default)]
pub struct SampleCollector {
    samples: Vec<f64>,
}

impl SampleCollector {
    const BOOTSTRAP_RESAMPLES: usize = 64 * 1024;

    pub fn push(&mut self, sample: f64) {
        self.samples.push(sample);
    }

    pub fn n(&self) -> u64 {
        self.samples.len() as u64
    }

    pub fn statistics(&self) -> Statistics {
        assert!(!self.samples.is_empty());
        assert!(self.samples.iter().all(|sample| sample.is_finite()));

        let n = self.samples.len();
        let (mean, std_dev) = Self::mean_and_std_dev(&self.samples);
        let median = Self::median(&self.samples);
        let mean_ci_95 = Self::bootstrap_mean_ci_95(&self.samples);

        Statistics {
            n: n as u64,
            mean,
            std_dev,
            mean_ci_95,
            median,
        }
    }

    fn mean_and_std_dev(samples: &[f64]) -> (f64, f64) {
        let mut mean = 0.0;
        let mut m2 = 0.0;

        for (i, sample) in samples.iter().enumerate() {
            let n = i as f64 + 1.0;
            let delta = sample - mean;
            mean += delta / n;
            m2 += delta * (sample - mean);
        }

        let std_dev = if samples.len() < 2 {
            0.0
        } else {
            (m2.max(0.0) / (samples.len() as f64 - 1.0)).sqrt()
        };

        (mean, std_dev)
    }

    fn median(samples: &[f64]) -> f64 {
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);

        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            Self::midpoint(sorted[mid - 1], sorted[mid])
        } else {
            sorted[mid]
        }
    }

    fn bootstrap_mean_ci_95(samples: &[f64]) -> Range<f64> {
        let mut rng =
            Xoshiro256PlusPlus::try_from_rng(&mut SysRng).expect("system RNG is available");
        let mut means = Vec::with_capacity(Self::BOOTSTRAP_RESAMPLES);

        for _ in 0..Self::BOOTSTRAP_RESAMPLES {
            let mut mean = 0.0;
            for i in 0..samples.len() {
                let sample = samples[rng.random_range(0..samples.len())];
                mean += (sample - mean) / (i as f64 + 1.0);
            }
            means.push(mean);
        }

        means.sort_by(f64::total_cmp);
        Self::percentile(&means, 0.025)..Self::percentile(&means, 0.975)
    }

    fn percentile(sorted_samples: &[f64], p: f64) -> f64 {
        debug_assert!(!sorted_samples.is_empty());
        debug_assert!((0.0..=1.0).contains(&p));

        if sorted_samples.len() == 1 {
            return sorted_samples[0];
        }

        let rank = p * (sorted_samples.len() - 1) as f64;
        let lower = rank.floor() as usize;
        let upper = rank.ceil() as usize;
        if lower == upper {
            sorted_samples[lower]
        } else {
            let weight = rank - lower as f64;
            sorted_samples[lower] * (1.0 - weight) + sorted_samples[upper] * weight
        }
    }

    fn midpoint(a: f64, b: f64) -> f64 {
        if a.is_sign_negative() == b.is_sign_negative() {
            a + (b - a) / 2.0
        } else {
            (a + b) / 2.0
        }
    }
}

pub fn run_collect_stats<G: KangarooGroup>(solver: &impl DiscreteLogSolver, bits: u32, iters: usize, rng: &mut impl Rng) -> Statistics {
    let n = 1 << bits;
    let mut collector = SampleCollector::default();
    for _ in 0..iters {
        let x = rng.random_range(0..n);
        let element = generator_scalar_mul_i64::<G>(x);
        let solution = solver.solve(element, 0, n, rng);
        assert_eq!(solution.discrete_log(), x);
        let k = solution.group_ops() as f64 / (n as f64).sqrt();
        collector.push(k);
    }
    collector.statistics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_basic_statistics() {
        let mut collector = SampleCollector::default();
        for sample in [1.0, 2.0, 3.0, 4.0] {
            collector.push(sample);
        }

        let stats = collector.statistics();

        assert_eq!(stats.n, 4);
        assert_eq!(stats.mean, 2.5);
        assert_eq!(stats.median, 2.5);
        assert!((stats.std_dev - (5.0_f64 / 3.0).sqrt()).abs() < f64::EPSILON);
        assert!(stats.mean_ci_95.start <= stats.mean);
        assert!(stats.mean <= stats.mean_ci_95.end);
    }
}
