use clap::Parser;
use discrete_log_research::dlp::DiscreteLogSolver;
use discrete_log_research::dlp::method::baby_step_giant_step::{
    BabyStepGiantStepBasic, BabyStepGiantStepInterleaved, BabyStepGiantStepNegMap,
};
use discrete_log_research::dlp::stats::{SampleCollector, Statistics};
use discrete_log_research::group::toy::ToyGroup;
use discrete_log_research::group::{KangarooGroup, generator_scalar_mul_i64};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::rngs::{SysRng, Xoshiro256PlusPlus};
use rand::{Rng, RngExt, SeedableRng};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

const DEFAULT_RANGE_BITS: u32 = 32;
const DEFAULT_SAMPLES: usize = 16 * 1024;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = DEFAULT_RANGE_BITS)]
    range_bits: u32,

    #[arg(long, default_value_t = DEFAULT_SAMPLES)]
    samples: usize,
}

type CollectFn = fn(u32, usize, Arc<RwLock<SampleCollector>>);

struct Job {
    name: &'static str,
    collect: CollectFn,
    progress: ProgressBar,
    stats: Arc<RwLock<SampleCollector>>,
}

impl Job {
    fn new(
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

    fn n(&self) -> u64 {
        self.stats.read().expect("sample collector is poisoned").n()
    }

    fn statistics(&self) -> Statistics {
        self.stats
            .read()
            .expect("sample collector is poisoned")
            .statistics()
    }

    fn update_progress(&self) {
        self.progress.set_position(self.n());
    }

    fn finish_progress(&self) {
        self.update_progress();
        self.progress.finish_with_message("done");
    }
}

fn collect_statistics<G: KangarooGroup>(
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

fn collect_solver_statistics<S, G>(bits: u32, iters: usize, stats: Arc<RwLock<SampleCollector>>)
where
    S: Default + DiscreteLogSolver,
    G: KangarooGroup,
{
    let solver = S::default();
    let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng).expect("system RNG is available");

    collect_statistics::<G>(&solver, bits, iters, stats, &mut rng);
}

fn main() {
    let args = Args::parse();
    assert!(args.range_bits < i64::BITS);
    assert!(args.samples > 0);
    let range_bits = args.range_bits;
    let samples = args.samples;

    let multi_progress = MultiProgress::new();
    let progress_style = ProgressStyle::with_template(
        "{prefix:<24} [{bar:40.cyan/blue}] {pos:>5}/{len:5} {percent:>6.2}% {msg}",
    )
    .expect("progress template is valid")
    .progress_chars("##-");

    let jobs = vec![
        // Job::new(
        //     "pollard kangaroo",
        //     collect_solver_statistics::<PollardKangarooBasic, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "pollard three kangaroo",
        //     collect_solver_statistics::<PollardKangarooThree, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "pollard four kangaroo",
        //     collect_solver_statistics::<PollardKangarooFour, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "gaudry-schost basic",
        //     collect_solver_statistics::<GaudrySchostBasicSim, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "gaudry-schost three-set",
        //     collect_solver_statistics::<GaudrySchostThreeSet, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "gaudry-schost four-set",
        //     collect_solver_statistics::<GaudrySchostFourSet, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "gaudry-schost neg-map",
        //     collect_solver_statistics::<GaudrySchostNegMap, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "gaudry-schost sota v2",
        //     collect_solver_statistics::<GaudrySchostSotaV2, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new(
        //     "gaudry-schost sota v2+",
        //     collect_solver_statistics::<GaudrySchostSotaV2Plus, ToyGroup>,
        //     &multi_progress,
        //     &progress_style,
        // ),
        Job::new(
            "bsgs basic",
            collect_solver_statistics::<BabyStepGiantStepBasic, ToyGroup>,
            samples,
            &multi_progress,
            &progress_style,
        ),
        Job::new(
            "bsgs neg map",
            collect_solver_statistics::<BabyStepGiantStepNegMap, ToyGroup>,
            samples,
            &multi_progress,
            &progress_style,
        ),
        Job::new(
            "bsgs interleaved",
            collect_solver_statistics::<BabyStepGiantStepInterleaved, ToyGroup>,
            samples,
            &multi_progress,
            &progress_style,
        ),
    ];

    thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|job| {
                let collect = job.collect;
                let stats = Arc::clone(&job.stats);
                scope.spawn(move || collect(range_bits, samples, stats))
            })
            .collect();

        update_progress(&jobs);

        while handles.iter().any(|handle| !handle.is_finished()) {
            thread::sleep(Duration::from_secs(1));
            update_progress(&jobs);
        }

        for handle in handles {
            handle
                .join()
                .expect("statistics collection thread panicked");
        }
    });

    for job in &jobs {
        job.finish_progress();
    }

    println!("computing final statistics...");

    for job in &jobs {
        print_statistics(job.name, &job.statistics());
    }
}

fn update_progress(jobs: &[Job]) {
    for job in jobs {
        job.update_progress();
    }
}

fn print_statistics(name: &str, statistics: &Statistics) {
    println!(
        "{name}: n = {}, mean = {:.3}, std dev = {:.3}, 95% mean CI = [{:.3}, {:.3}], median = {:.3}",
        statistics.n,
        statistics.mean,
        statistics.std_dev,
        statistics.mean_ci_95.start,
        statistics.mean_ci_95.end,
        statistics.median
    );
}
