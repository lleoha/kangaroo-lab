use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kangaroo_lab::dlp::DiscreteLogSolver;
use kangaroo_lab::dlp::stats::{SampleCollector, Statistics};
use kangaroo_lab::group::{KangarooGroup, generator_scalar_mul_i64};
use rand::rngs::{SysRng, Xoshiro256PlusPlus};
use rand::{Rng, RngExt, SeedableRng};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

type CollectFn = Box<dyn Fn(u32, usize, Arc<RwLock<SampleCollector>>) + Send + Sync>;

pub(super) struct Job {
    name: &'static str,
    collect: CollectFn,
    progress: ProgressBar,
    stats: Arc<RwLock<SampleCollector>>,
}

impl Job {
    pub fn new<G: KangarooGroup>(
        name: &'static str,
        solver: impl DiscreteLogSolver + Send + Sync + 'static,
        samples: usize,
        multi_progress: &MultiProgress,
        progress_style: &ProgressStyle,
    ) -> Self {
        Self::new_collector(
            name,
            Box::new(move |bits, iters, stats| {
                collect_statistics::<G>(&solver, bits, iters, stats);
            }),
            samples,
            multi_progress,
            progress_style,
        )
    }

    fn new_collector(
        name: &'static str,
        collect: CollectFn,
        samples: usize,
        multi_progress: &MultiProgress,
        progress_style: &ProgressStyle,
    ) -> Self {
        let progress = multi_progress.add(ProgressBar::new(samples as u64));
        progress.set_prefix(name);
        progress.set_style(progress_style.clone());

        Self {
            name,
            collect,
            progress,
            stats: Arc::new(RwLock::new(SampleCollector::default())),
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn n(&self) -> u64 {
        self.stats.read().expect("sample collector is poisoned").n()
    }

    pub fn statistics(&self) -> Statistics {
        self.stats
            .read()
            .expect("sample collector is poisoned")
            .statistics()
    }

    fn run(&self, bits: u32, iters: usize) {
        (self.collect)(bits, iters, Arc::clone(&self.stats));
    }

    fn update_progress(&self) {
        self.progress.set_position(self.n());
    }

    fn finish_progress(&self) {
        self.update_progress();
        self.progress.finish_with_message("done");
    }
}

pub(super) fn run_jobs(jobs: &[Job], range_bits: u32, samples: usize) {
    thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|job| scope.spawn(move || job.run(range_bits, samples)))
            .collect();

        update_progress(jobs);

        while handles.iter().any(|handle| !handle.is_finished()) {
            thread::sleep(Duration::from_secs(1));
            update_progress(jobs);
        }

        for handle in handles {
            handle
                .join()
                .expect("statistics collection thread panicked");
        }
    });

    for job in jobs {
        job.finish_progress();
    }
}

fn update_progress(jobs: &[Job]) {
    for job in jobs {
        job.update_progress();
    }
}

fn collect_samples<G: KangarooGroup>(
    solver: &impl DiscreteLogSolver,
    bits: u32,
    iters: usize,
    stats: Arc<RwLock<SampleCollector>>,
    rng: &mut impl Rng,
) {
    let n = 1_i64 << bits;
    for _ in 0..iters {
        let x = rng.random_range(0..n);
        let element = generator_scalar_mul_i64::<G>(x);
        let solution = solver.solve(element, 0, n, rng);
        assert_eq!(solution.discrete_log(), x);

        let k = solution.group_ops() as f64 / (n as f64).sqrt();
        stats.write().expect("sample collector is poisoned").push(k);
    }
}

fn collect_statistics<G: KangarooGroup>(
    solver: &impl DiscreteLogSolver,
    bits: u32,
    iters: usize,
    stats: Arc<RwLock<SampleCollector>>,
) {
    let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng).expect("system RNG is available");
    collect_samples::<G>(solver, bits, iters, stats, &mut rng);
}
