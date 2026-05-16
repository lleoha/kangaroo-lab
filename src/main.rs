use crate::dlp::DiscreteLogSolver;
use crate::dlp::method::gaudry_schost::GaudrySchostBasicIdeal;
use crate::dlp::method::gaudry_schost_four_set::GaudrySchostImprovedFourSetIdeal;
use crate::dlp::method::gaudry_schost_neg_map::GaudrySchostNegationMapIdeal;
use crate::dlp::method::gaudry_schost_three_set::GaudrySchostImprovedThreeSetIdeal;
use crate::dlp::method::pollard_four_kangaroo::PollardFourKangarooSolver;
use crate::dlp::method::pollard_kangaroo::PollardKangarooSolver;
use crate::dlp::method::pollard_three_kangaroo::PollardThreeKangarooSolver;
use crate::dlp::method::retired_coder_sota_v2::RetiredCoderSotaV2Ideal;
use crate::dlp::stats::{SampleCollector, Statistics};
use crate::group::toy::ToyGroup;
use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType};
use rand::rngs::{SysRng, Xoshiro256PlusPlus};
use rand::{Rng, RngExt, SeedableRng};
use std::io::{Write, stdout};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub mod dlp;
pub mod group;

const RANGE_BITS: u32 = 48;
const SAMPLES: usize = 16 * 1024;

type CollectFn = fn(u32, usize, Arc<RwLock<SampleCollector>>);

struct Job {
    name: &'static str,
    collect: CollectFn,
    stats: Arc<RwLock<SampleCollector>>,
}

impl Job {
    fn new(name: &'static str, collect: CollectFn) -> Self {
        Self {
            name,
            collect,
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
}

fn collect_statistics<G: KangarooGroup>(
    solver: &impl DiscreteLogSolver,
    bits: u32,
    iters: usize,
    stats: Arc<RwLock<SampleCollector>>,
    rng: &mut impl Rng,
) {
    let n = 1 << bits;
    for _ in 0..iters {
        let x = rng.random_range(0..n);
        let element = generator_scalar_mul_i64::<G>(x);
        let solution = solver.solve(element, 0, n, rng);
        assert_eq!(solution.discrete_log(), x);

        let k = (solution.group_ops() as f64) / (n as f64).sqrt();
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
    let jobs = vec![
        Job::new(
            "pollard kangaroo",
            collect_solver_statistics::<PollardKangarooSolver, ToyGroup>,
        ),
        Job::new(
            "pollard three kangaroo",
            collect_solver_statistics::<PollardThreeKangarooSolver, ToyGroup>,
        ),
        Job::new(
            "pollard four kangaroo",
            collect_solver_statistics::<PollardFourKangarooSolver, ToyGroup>,
        ),
        Job::new(
            "gaudry-schost basic",
            collect_solver_statistics::<GaudrySchostBasicIdeal, ToyGroup>,
        ),
        Job::new(
            "gaudry-schost three-set",
            collect_solver_statistics::<GaudrySchostImprovedThreeSetIdeal, ToyGroup>,
        ),
        Job::new(
            "gaudry-schost four-set",
            collect_solver_statistics::<GaudrySchostImprovedFourSetIdeal, ToyGroup>,
        ),
        Job::new(
            "gaudry-schost neg-map",
            collect_solver_statistics::<GaudrySchostNegationMapIdeal, ToyGroup>,
        ),
        Job::new(
            "retired coder sota v2",
            collect_solver_statistics::<RetiredCoderSotaV2Ideal, ToyGroup>,
        ),
    ];

    thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|job| {
                let collect = job.collect;
                let stats = Arc::clone(&job.stats);
                scope.spawn(move || collect(RANGE_BITS, SAMPLES, stats))
            })
            .collect();

        let mut progress = ProgressRenderer::new(jobs.len());
        progress.render(&jobs, SAMPLES);

        while handles.iter().any(|handle| !handle.is_finished()) {
            thread::sleep(Duration::from_secs(1));
            progress.render(&jobs, SAMPLES);
        }

        for handle in handles {
            handle
                .join()
                .expect("statistics collection thread panicked");
        }
    });

    println!("computing final statistics...");

    for job in &jobs {
        print_statistics(job.name, &job.statistics());
    }
}

struct ProgressRenderer {
    rendered: bool,
    lines: usize,
    _cursor_guard: CursorGuard,
}

impl ProgressRenderer {
    fn new(lines: usize) -> Self {
        Self {
            rendered: false,
            lines,
            _cursor_guard: CursorGuard::new(),
        }
    }

    fn render(&mut self, jobs: &[Job], samples: usize) {
        let mut stdout = stdout();
        if self.rendered {
            execute!(stdout, cursor::MoveUp(self.lines as u16))
                .expect("progress cursor can be moved");
        }

        let terminal_width = terminal::size()
            .map(|(width, _)| width as usize)
            .unwrap_or(100);
        let name_width = jobs.iter().map(|job| job.name.len()).max().unwrap_or(0);
        let bar_width = bar_width(terminal_width, name_width);
        let count_width = samples.to_string().len();

        for job in jobs {
            execute!(stdout, Clear(ClearType::CurrentLine)).expect("progress line can be cleared");
            writeln!(
                stdout,x
                "{}",
                progress_line(
                    job.name,
                    job.n(),
                    samples,
                    name_width,
                    bar_width,
                    count_width,
                )
            )
            .expect("progress line can be written");
        }

        stdout.flush().expect("stdout can be flushed");
        self.rendered = true;
    }
}

struct CursorGuard;

impl CursorGuard {
    fn new() -> Self {
        let mut stdout = stdout();
        execute!(stdout, cursor::Hide).expect("cursor can be hidden");
        Self
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = execute!(stdout, cursor::Show);
    }
}

fn bar_width(terminal_width: usize, name_width: usize) -> usize {
    terminal_width.saturating_sub(name_width + 30).clamp(10, 40)
}

fn progress_line(
    name: &str,
    n: u64,
    samples: usize,
    name_width: usize,
    bar_width: usize,
    count_width: usize,
) -> String {
    let percent = if samples == 0 {
        100.0
    } else {
        (n as f64 * 100.0) / samples as f64
    };
    let filled = if samples == 0 {
        bar_width
    } else {
        (n as usize)
            .saturating_mul(bar_width)
            .checked_div(samples)
            .unwrap_or(bar_width)
            .min(bar_width)
    };
    let empty = bar_width - filled;

    format!(
        "{name:<name_width$} [{}{}] {n:>count_width$}/{samples:<count_width$} {percent:>6.2}%",
        "#".repeat(filled),
        "-".repeat(empty),
    )
}

fn print_statistics(name: &str, statistics: &Statistics) {
    println!(
        "{name}: n = {}, mean = {:.6}, std dev = {:.6}, 95% mean CI = [{:.6}, {:.6}], median = {:.6}",
        statistics.n,
        statistics.mean,
        statistics.std_dev,
        statistics.mean_ci_95.start,
        statistics.mean_ci_95.end,
        statistics.median
    );
}
