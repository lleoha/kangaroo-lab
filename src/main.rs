pub mod cli;

use crate::cli::{Job, run_jobs};
use clap::Parser;
use indicatif::{MultiProgress, ProgressStyle};
use kangaroo_lab::dlp::method::baby_step_giant_step::{
    BabyStepGiantStepBasic, BabyStepGiantStepInterleaved, BabyStepGiantStepNegMap,
};
use kangaroo_lab::dlp::method::gaudry_schost::{GaudrySchostBasicSim, GaudrySchostFourSet, GaudrySchostImprovedNegMap, GaudrySchostNegMap, GaudrySchostSixSet, GaudrySchostSotaV2, GaudrySchostSotaV2Plus, GaudrySchostThreeSet};
use kangaroo_lab::dlp::method::pollard_kangaroo::{
    PollardKangarooBasic, PollardKangarooFour, PollardKangarooThree,
};
use kangaroo_lab::group::toy::ToyGroup;

const DEFAULT_RANGE_BITS: u32 = 48;
const DEFAULT_SAMPLES: usize = 32 * 1024;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = DEFAULT_RANGE_BITS)]
    range_bits: u32,

    #[arg(long, default_value_t = DEFAULT_SAMPLES)]
    samples: usize,
}

fn main() {
    let args = Args::parse();
    let range_bits = args.range_bits;
    let samples = args.samples;

    let multi_progress = MultiProgress::new();
    let progress_style = ProgressStyle::with_template(
        "{prefix:<36} [{bar:60}] {pos:>5}/{len:5} {percent:>6.2}% {msg}",
    )
    .expect("progress template is valid")
    .progress_chars("=> ");

    let jobs = vec![
        // Job::new::<ToyGroup>(
        //     "pollard kangaroo",
        //     PollardKangarooBasic,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "pollard three kangaroo",
        //     PollardKangarooThree,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "pollard four kangaroo",
        //     PollardKangarooFour,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost",
        //     GaudrySchostBasicSim,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost three-set",
        //     GaudrySchostThreeSet,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost four-set",
        //     GaudrySchostFourSet,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        Job::new::<ToyGroup>(
            "gaudry-schost six-set (α = 1/8)",
            GaudrySchostSixSet::new(1.0 / 64.0),
            samples,
            &multi_progress,
            &progress_style,
        ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost neg-map",
        //     GaudrySchostNegMap,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost improved (α = 0.1)",
        //     GaudrySchostImprovedNegMap::new_unchecked(0.1),
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost improved (α = 0.05)",
        //     GaudrySchostImprovedNegMap::new_unchecked(0.05),
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost sota-v2 (α = 1/8)",
        //     GaudrySchostSotaV2::new_unchecked(1.0 / 8.0),
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost sota-v2 (α = 1/64)",
        //     GaudrySchostSotaV2::new_unchecked(1.0 / 64.0),
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "gaudry-schost sota-v2+ (α = 1/64)",
        //     GaudrySchostSotaV2Plus::new_unchecked(1.0 / 64.0),
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "baby step giant step",
        //     BabyStepGiantStepBasic,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "baby step giant step neg-map",
        //     BabyStepGiantStepNegMap,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
        // Job::new::<ToyGroup>(
        //     "baby step giant step interleaved",
        //     BabyStepGiantStepInterleaved,
        //     samples,
        //     &multi_progress,
        //     &progress_style,
        // ),
    ];

    run_jobs(&jobs, range_bits, samples);

    println!("\ncomputing final statistics...");
    print_statistics(&jobs);
}

fn print_statistics(jobs: &[Job]) {
    println!(
        "{:<36} {:>5} {:>7} {:>14} {:^6}",
        "solver", "mean", "std dev", "95% mean CI", "median"
    );

    for job in jobs {
        let statistics = job.statistics();
        println!(
            "{:<36} {:>5.3} {:>7.3} [{:>5.3}, {:>5.3}] {:>6.3}",
            job.name(),
            statistics.mean,
            statistics.std_dev,
            statistics.mean_ci_95.start,
            statistics.mean_ci_95.end,
            statistics.median
        );
    }
}
