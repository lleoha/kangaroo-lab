// use crate::dlog::solver::gaudry_schost_sim::gs;
// use rand::rngs::Xoshiro256PlusPlus;
// use rand::SeedableRng;
// use std::collections::{HashMap, HashSet, VecDeque};
// use std::env;
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread;
// use std::time::{Duration, Instant};
//
// const GAMMA_SCALE: f64 = 1_000_000.0;
//
// #[derive(Debug, Clone)]
// pub struct GsOptimizerConfig {
//     pub gamma_min: f64,
//     pub gamma_max: f64,
//     pub target_runtime: Duration,
//     pub workers: usize,
//     pub initial_grid_points: usize,
//     pub initial_repeats: usize,
//     pub batch_jobs_per_worker: usize,
//     pub candidates_to_refine: usize,
//     pub min_refinement_step: f64,
//     pub seed: u64,
// }
//
// impl GsOptimizerConfig {
//     pub fn overnight() -> Self {
//         let workers = thread::available_parallelism()
//             .map(usize::from)
//             .unwrap_or(1);
//
//         GsOptimizerConfig {
//             gamma_min: 0.05,
//             gamma_max: 1.0,
//             target_runtime: Duration::from_secs(7 * 60 * 60),
//             workers,
//             initial_grid_points: 41,
//             initial_repeats: 2,
//             batch_jobs_per_worker: 4,
//             candidates_to_refine: 8,
//             min_refinement_step: 0.0005,
//             seed: 0x9e3779b97f4a7c15,
//         }
//     }
//
//     pub fn with_env_overrides(mut self) -> Self {
//         if let Some(hours) = read_env::<f64>("GS_OPT_HOURS") {
//             self.target_runtime = Duration::from_secs_f64(hours * 60.0 * 60.0);
//         }
//         if let Some(workers) = read_env::<usize>("GS_OPT_WORKERS") {
//             self.workers = workers.max(1);
//         }
//         if let Some(seed) = read_env::<u64>("GS_OPT_SEED") {
//             self.seed = seed;
//         }
//         if let Some(grid_points) = read_env::<usize>("GS_OPT_INITIAL_GRID") {
//             self.initial_grid_points = grid_points.max(2);
//         }
//         if let Some(repeats) = read_env::<usize>("GS_OPT_INITIAL_REPEATS") {
//             self.initial_repeats = repeats.max(1);
//         }
//
//         self
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct GsOptimizerResult {
//     pub gamma: f64,
//     pub mean: f64,
//     pub std_dev: f64,
//     pub standard_error: f64,
//     pub samples: u64,
//     pub total_evaluations: u64,
//     pub elapsed: Duration,
// }
//
// #[derive(Debug, Clone)]
// struct CandidateStats {
//     key: i64,
//     gamma: f64,
//     n: u64,
//     mean: f64,
//     m2: f64,
//     best_observed: f64,
// }
//
// impl CandidateStats {
//     fn new(key: i64) -> Self {
//         CandidateStats {
//             key,
//             gamma: key_to_gamma(key),
//             n: 0,
//             mean: 0.0,
//             m2: 0.0,
//             best_observed: f64::INFINITY,
//         }
//     }
//
//     fn update(&mut self, sample: f64) {
//         self.n += 1;
//         let delta = sample - self.mean;
//         self.mean += delta / self.n as f64;
//         let delta2 = sample - self.mean;
//         self.m2 += delta * delta2;
//         self.best_observed = self.best_observed.min(sample);
//     }
//
//     fn std_dev(&self) -> f64 {
//         if self.n < 2 {
//             f64::NAN
//         } else {
//             (self.m2 / (self.n - 1) as f64).sqrt()
//         }
//     }
//
//     fn standard_error(&self) -> f64 {
//         if self.n < 2 {
//             f64::INFINITY
//         } else {
//             self.std_dev() / (self.n as f64).sqrt()
//         }
//     }
//
//     fn selection_score(&self) -> f64 {
//         match self.n {
//             0 => f64::INFINITY,
//             1 => self.mean - 0.25,
//             _ => self.mean - 1.5 * self.standard_error(),
//         }
//     }
// }
//
// #[derive(Debug, Clone, Copy)]
// struct Job {
//     candidate_key: i64,
//     gamma: f64,
//     seed: u64,
// }
//
// #[derive(Debug, Clone, Copy)]
// struct Evaluation {
//     candidate_key: i64,
//     value: f64,
// }
//
// pub fn optimize_gs_gamma(config: GsOptimizerConfig) -> GsOptimizerResult {
//     assert!(config.gamma_min < config.gamma_max);
//     assert!(config.initial_grid_points >= 2);
//     assert!(config.initial_repeats >= 1);
//     assert!(config.workers >= 1);
//
//     let started_at = Instant::now();
//     let mut candidates = HashMap::new();
//     let initial_step =
//         (config.gamma_max - config.gamma_min) / (config.initial_grid_points - 1) as f64;
//     let mut refinement_step = initial_step;
//     let mut next_eval_id = 0u64;
//     let mut total_evaluations = 0u64;
//     let mut round = 0u64;
//
//     for i in 0..config.initial_grid_points {
//         let gamma = config.gamma_min + initial_step * i as f64;
//         ensure_candidate(&mut candidates, gamma, &config);
//     }
//
//     println!(
//         "Starting gs gamma optimizer: range [{:.6}, {:.6}], runtime {:?}, workers {}, initial grid {}, seed {}",
//         config.gamma_min,
//         config.gamma_max,
//         config.target_runtime,
//         config.workers,
//         config.initial_grid_points,
//         config.seed
//     );
//
//     loop {
//         if started_at.elapsed() >= config.target_runtime && total_evaluations > 0 {
//             break;
//         }
//
//         round += 1;
//         if round > 1 {
//             add_refinement_candidates(&mut candidates, &config, refinement_step);
//         }
//
//         let jobs = if round == 1 {
//             initial_jobs(&candidates, &config, &mut next_eval_id)
//         } else {
//             let batch_size = config.workers * config.batch_jobs_per_worker;
//             let keys = select_candidate_keys(&candidates, batch_size);
//             make_jobs(keys, &candidates, &config, &mut next_eval_id)
//         };
//
//         if jobs.is_empty() {
//             break;
//         }
//
//         let results = evaluate_jobs(jobs, config.workers);
//         total_evaluations += results.len() as u64;
//         for result in results {
//             candidates
//                 .get_mut(&result.candidate_key)
//                 .unwrap()
//                 .update(result.value);
//         }
//
//         if round > 1 && round % 8 == 0 {
//             refinement_step = (refinement_step * 0.5).max(config.min_refinement_step);
//         }
//
//         print_progress(
//             &config,
//             &candidates,
//             started_at,
//             round,
//             total_evaluations,
//             refinement_step,
//         );
//     }
//
//     let best = best_by_mean(&candidates).expect("at least one gs evaluation must finish");
//     GsOptimizerResult {
//         gamma: best.gamma,
//         mean: best.mean,
//         std_dev: best.std_dev(),
//         standard_error: best.standard_error(),
//         samples: best.n,
//         total_evaluations,
//         elapsed: started_at.elapsed(),
//     }
// }
//
// fn read_env<T: std::str::FromStr>(name: &str) -> Option<T> {
//     env::var(name).ok().and_then(|value| value.parse().ok())
// }
//
// fn initial_jobs(
//     candidates: &HashMap<i64, CandidateStats>,
//     config: &GsOptimizerConfig,
//     next_eval_id: &mut u64,
// ) -> Vec<Job> {
//     let mut keys: Vec<_> = candidates.keys().copied().collect();
//     keys.sort_unstable();
//
//     let mut jobs = Vec::with_capacity(keys.len() * config.initial_repeats);
//     for _ in 0..config.initial_repeats {
//         for &key in &keys {
//             jobs.push(job_for_key(key, candidates, config, next_eval_id));
//         }
//     }
//     jobs
// }
//
// fn make_jobs(
//     keys: Vec<i64>,
//     candidates: &HashMap<i64, CandidateStats>,
//     config: &GsOptimizerConfig,
//     next_eval_id: &mut u64,
// ) -> Vec<Job> {
//     keys.into_iter()
//         .map(|key| job_for_key(key, candidates, config, next_eval_id))
//         .collect()
// }
//
// fn job_for_key(
//     key: i64,
//     candidates: &HashMap<i64, CandidateStats>,
//     config: &GsOptimizerConfig,
//     next_eval_id: &mut u64,
// ) -> Job {
//     let gamma = candidates.get(&key).unwrap().gamma;
//     let seed = mix64(
//         config.seed
//             ^ next_eval_id.wrapping_mul(0x9e3779b97f4a7c15)
//             ^ (key as u64).wrapping_mul(0xbf58476d1ce4e5b9),
//     );
//     *next_eval_id += 1;
//
//     Job {
//         candidate_key: key,
//         gamma,
//         seed,
//     }
// }
//
// fn evaluate_jobs(jobs: Vec<Job>, workers: usize) -> Vec<Evaluation> {
//     let expected_results = jobs.len();
//     let queue = Arc::new(Mutex::new(VecDeque::from(jobs)));
//     let (tx, rx) = mpsc::channel();
//     let workers = workers.min(expected_results).max(1);
//
//     thread::scope(|scope| {
//         for _ in 0..workers {
//             let queue = Arc::clone(&queue);
//             let tx = tx.clone();
//             scope.spawn(move || loop {
//                 let job = queue.lock().unwrap().pop_front();
//                 let Some(job) = job else {
//                     break;
//                 };
//
//                 let mut rng = Xoshiro256PlusPlus::seed_from_u64(job.seed);
//                 let value = gs(job.gamma, &mut rng);
//                 tx.send(Evaluation {
//                     candidate_key: job.candidate_key,
//                     value,
//                 })
//                 .unwrap();
//             });
//         }
//     });
//
//     let mut results = Vec::with_capacity(expected_results);
//     for _ in 0..expected_results {
//         results.push(rx.recv().unwrap());
//     }
//     results
// }
//
// fn add_refinement_candidates(
//     candidates: &mut HashMap<i64, CandidateStats>,
//     config: &GsOptimizerConfig,
//     step: f64,
// ) {
//     let leaders = top_by_mean(candidates, config.candidates_to_refine);
//     for leader in leaders {
//         for offset in [-step, -0.5 * step, 0.5 * step, step] {
//             ensure_candidate(candidates, leader + offset, config);
//         }
//     }
// }
//
// fn ensure_candidate(
//     candidates: &mut HashMap<i64, CandidateStats>,
//     gamma: f64,
//     config: &GsOptimizerConfig,
// ) -> i64 {
//     let gamma = gamma.clamp(config.gamma_min, config.gamma_max);
//     let key = gamma_to_key(gamma);
//     candidates
//         .entry(key)
//         .or_insert_with(|| CandidateStats::new(key));
//     key
// }
//
// fn select_candidate_keys(candidates: &HashMap<i64, CandidateStats>, count: usize) -> Vec<i64> {
//     let mut selected = Vec::with_capacity(count);
//     let mut seen = HashSet::new();
//
//     append_top_by_mean(candidates, count / 2, &mut selected, &mut seen);
//     append_unevaluated(
//         candidates,
//         (selected.len() + count / 4).min(count),
//         &mut selected,
//         &mut seen,
//     );
//     append_top_by_score(candidates, count, &mut selected, &mut seen);
//
//     selected
// }
//
// fn append_top_by_mean(
//     candidates: &HashMap<i64, CandidateStats>,
//     limit: usize,
//     selected: &mut Vec<i64>,
//     seen: &mut HashSet<i64>,
// ) {
//     let mut ranked: Vec<_> = candidates
//         .values()
//         .filter(|candidate| candidate.n > 0)
//         .collect();
//     ranked.sort_by(|a, b| {
//         a.mean
//             .total_cmp(&b.mean)
//             .then_with(|| b.n.cmp(&a.n))
//             .then_with(|| a.gamma.total_cmp(&b.gamma))
//     });
//     append_ranked(ranked, limit, selected, seen);
// }
//
// fn append_unevaluated(
//     candidates: &HashMap<i64, CandidateStats>,
//     limit: usize,
//     selected: &mut Vec<i64>,
//     seen: &mut HashSet<i64>,
// ) {
//     let mut ranked: Vec<_> = candidates
//         .values()
//         .filter(|candidate| candidate.n == 0)
//         .collect();
//     ranked.sort_by(|a, b| a.gamma.total_cmp(&b.gamma));
//     append_ranked(ranked, limit, selected, seen);
// }
//
// fn append_top_by_score(
//     candidates: &HashMap<i64, CandidateStats>,
//     limit: usize,
//     selected: &mut Vec<i64>,
//     seen: &mut HashSet<i64>,
// ) {
//     let mut ranked: Vec<_> = candidates.values().collect();
//     ranked.sort_by(|a, b| {
//         a.selection_score()
//             .total_cmp(&b.selection_score())
//             .then_with(|| a.gamma.total_cmp(&b.gamma))
//     });
//     append_ranked(ranked, limit, selected, seen);
// }
//
// fn append_ranked(
//     ranked: Vec<&CandidateStats>,
//     limit: usize,
//     selected: &mut Vec<i64>,
//     seen: &mut HashSet<i64>,
// ) {
//     for candidate in ranked {
//         if selected.len() >= limit {
//             break;
//         }
//         if seen.insert(candidate.key) {
//             selected.push(candidate.key);
//         }
//     }
// }
//
// fn top_by_mean(candidates: &HashMap<i64, CandidateStats>, count: usize) -> Vec<f64> {
//     let mut ranked: Vec<_> = candidates
//         .values()
//         .filter(|candidate| candidate.n > 0)
//         .collect();
//     ranked.sort_by(|a, b| {
//         a.mean
//             .total_cmp(&b.mean)
//             .then_with(|| b.n.cmp(&a.n))
//             .then_with(|| a.gamma.total_cmp(&b.gamma))
//     });
//     ranked
//         .into_iter()
//         .take(count)
//         .map(|candidate| candidate.gamma)
//         .collect()
// }
//
// fn best_by_mean(candidates: &HashMap<i64, CandidateStats>) -> Option<&CandidateStats> {
//     let min_samples = if candidates.values().any(|candidate| candidate.n >= 2) {
//         2
//     } else {
//         1
//     };
//
//     candidates
//         .values()
//         .filter(|candidate| candidate.n >= min_samples)
//         .min_by(|a, b| {
//             a.mean
//                 .total_cmp(&b.mean)
//                 .then_with(|| b.n.cmp(&a.n))
//                 .then_with(|| a.gamma.total_cmp(&b.gamma))
//         })
// }
//
// fn print_progress(
//     config: &GsOptimizerConfig,
//     candidates: &HashMap<i64, CandidateStats>,
//     started_at: Instant,
//     round: u64,
//     total_evaluations: u64,
//     refinement_step: f64,
// ) {
//     let elapsed = started_at.elapsed();
//     let remaining = config.target_runtime.saturating_sub(elapsed);
//     let evals_per_hour = if elapsed.as_secs_f64() > 0.0 {
//         total_evaluations as f64 * 3600.0 / elapsed.as_secs_f64()
//     } else {
//         0.0
//     };
//
//     let best = best_by_mean(candidates).unwrap();
//     println!(
//         "[gs-opt] round {round} elapsed {:?} remaining {:?} evals {} ({:.1}/hour) candidates {} step {:.6}",
//         elapsed,
//         remaining,
//         total_evaluations,
//         evals_per_hour,
//         candidates.len(),
//         refinement_step
//     );
//     println!(
//         "[gs-opt] best gamma {:.6}: mean {:.6}, stderr {:.6}, stddev {:.6}, n {}, best-observed {:.6}",
//         best.gamma,
//         best.mean,
//         best.standard_error(),
//         best.std_dev(),
//         best.n,
//         best.best_observed
//     );
// }
//
// fn gamma_to_key(gamma: f64) -> i64 {
//     (gamma * GAMMA_SCALE).round() as i64
// }
//
// fn key_to_gamma(key: i64) -> f64 {
//     key as f64 / GAMMA_SCALE
// }
//
// fn mix64(mut x: u64) -> u64 {
//     x ^= x >> 30;
//     x = x.wrapping_mul(0xbf58476d1ce4e5b9);
//     x ^= x >> 27;
//     x = x.wrapping_mul(0x94d049bb133111eb);
//     x ^ (x >> 31)
// }
