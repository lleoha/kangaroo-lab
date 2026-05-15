// use crate::dlp::gs_optimizer::{optimize_gs_gamma, GsOptimizerConfig};

use std::io::{Write, stdout};
use std::thread::sleep;
use std::time::Duration;

pub mod dlp;
pub mod group;

// const FOOTPRINT_DST: u64 = u64::from_le_bytes(*b"FOTPRINT");
// const DISTINGUISHED_DST: u64 = u64::from_le_bytes(*b"DSTNGSHD");
// const BUCKET_DST: u64 = u64::from_le_bytes(*b"THEBUCKT");
// const NEGATION_MAP_DST: u64 = u64::from_le_bytes(*b"NEGATMAP");

fn main() {
    for i in 1..100 {
        print!("\r{:3}", i);
        sleep(Duration::from_secs(1));
        stdout().flush().unwrap();
    }
    // let config = GsOptimizerConfig::overnight().with_env_overrides();
    // let result = optimize_gs_gamma(config);
    // println!(
    //     "Best gamma {:.6}: mean {:.6}, stderr {:.6}, stddev {:.6}, samples {}, total evaluations {}, elapsed {:?}",
    //     result.gamma,
    //     result.mean,
    //     result.standard_error,
    //     result.std_dev,
    //     result.samples,
    //     result.total_evaluations,
    //     result.elapsed
    // );
}
