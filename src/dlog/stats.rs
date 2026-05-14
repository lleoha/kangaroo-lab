use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Sample(f64);

impl PartialEq for Sample {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Sample {}

impl PartialOrd for Sample {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sample {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

pub struct Summary {
    pub n: u64,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
}

pub struct RunningStats {
    n: u64,
    mean: f64,
    m2: f64,
    lower: BinaryHeap<Sample>,
    upper: BinaryHeap<Reverse<Sample>>,
}

impl RunningStats {
    pub fn new() -> Self {
        RunningStats {
            n: 0,
            mean: 0.0,
            m2: 0.0,
            lower: BinaryHeap::new(),
            upper: BinaryHeap::new(),
        }
    }

    pub fn update(&mut self, sample: f64) {
        self.update_mean_std_dev(sample);
        self.update_median(sample);
    }

    pub fn summary(&self) -> Summary {
        let n = self.n;
        let mean = self.mean;
        let std_dev = (self.m2 / ((self.n - 1) as f64)).sqrt();
        let median = if self.lower.len() > self.upper.len() {
            self.lower.peek().unwrap().0
        } else {
            let a = self.lower.peek().unwrap().0;
            let Reverse(sample) = self.upper.peek().unwrap();
            let b = sample.0;
            a + (b - a) / 2.0
        };

        Summary {
            n,
            mean,
            std_dev,
            median,
        }
    }

    fn update_mean_std_dev(&mut self, sample: f64) {
        self.n += 1;
        let n = self.n as f64;
        let delta = sample - self.mean;
        self.mean += delta / n;
        let delta2 = sample - self.mean;
        self.m2 += delta * delta2;
    }

    fn update_median(&mut self, sample: f64) {
        let sample = Sample(sample);

        match self.lower.peek() {
            None => {
                self.lower.push(sample);
            }
            Some(&lower_max) if sample <= lower_max => {
                self.lower.push(sample);
            }
            Some(_) => {
                self.upper.push(Reverse(sample));
            }
        }

        self.rebalance_heaps();
    }

    fn rebalance_heaps(&mut self) {
        if self.lower.len() > self.upper.len() + 1 {
            let sample = self.lower.pop().unwrap();
            self.upper.push(Reverse(sample));
        } else if self.upper.len() > self.lower.len() {
            let Reverse(sample) = self.upper.pop().unwrap();
            self.lower.push(sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::Xoshiro256PlusPlus;
    use rand::{RngExt, SeedableRng};
    use rand_distr::Normal;
    use std::error::Error;

    const SAMPLES: usize = 1024 * 1024;

    #[test]
    fn running_stats_matches_reference_statistics() -> Result<(), Box<dyn Error>> {
        const MEAN: f64 = 2.0;
        const STD_DEV: f64 = 5.0;

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0xdeadbeefcafebabe);
        let mut stats = RunningStats::new();
        let mut samples = Vec::with_capacity(SAMPLES);
        let normal_distribution = Normal::new(MEAN, STD_DEV)?;

        for _ in 0..SAMPLES {
            let sample = rng.sample(normal_distribution);
            stats.update(sample);
            samples.push(sample);
        }

        let summary = stats.summary();
        let reference = reference_summary(&mut samples);
        assert_close(reference.mean, MEAN, 0.01);
        assert_close(reference.std_dev, STD_DEV, 0.01);

        assert_eq!(summary.n, SAMPLES as u64);
        assert_close(summary.mean, reference.mean, 1e-9);
        assert_close(summary.std_dev, reference.std_dev, 1e-9);
        assert_close(summary.median, reference.median, 1e-9);

        Ok(())
    }

    fn reference_summary(samples: &mut [f64]) -> Summary {
        let n = samples.len() as u64;
        let mean = samples.iter().sum::<f64>() / n as f64;
        let std_dev = (samples
            .iter()
            .map(|sample| {
                let deviation = sample - mean;
                deviation * deviation
            })
            .sum::<f64>()
            / (n - 1) as f64)
            .sqrt();

        samples.sort_by(|a, b| a.total_cmp(b));
        let median = if samples.len() % 2 == 0 {
            let upper = samples.len() / 2;
            let a = samples[upper - 1];
            let b = samples[upper];
            a + (b - a) / 2.0
        } else {
            samples[samples.len() / 2]
        };

        Summary {
            n,
            mean,
            std_dev,
            median,
        }
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "actual {actual}, expected {expected}, tolerance {tolerance}"
        );
    }
}
