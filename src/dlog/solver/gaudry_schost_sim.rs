use std::collections::{HashMap, HashSet};

use rand::{Rng, RngExt};
use crate::group::helpers::generator_scalar_mul_i64;
use crate::group::KangarooGroup;

pub const DEFAULT_GS_COUNT: u64 = 4096;
pub const DEFAULT_GS_RANGE_BITS: u32 = 50;

#[derive(Debug, Clone, Copy)]
pub struct GaudrySchostSimulator {
    gamma: f64,
}

impl GaudrySchostSimulator {
    pub fn new(gamma: f64) -> Self {
        assert!(gamma.is_finite() && gamma > 0.0);
        GaudrySchostSimulator { gamma }
    }

    pub fn simulate<G: KangarooGroup>(&self, range_bits: u32, rng: &mut impl Rng) -> u64 {
        assert!(range_bits > 1 && range_bits < i64::BITS - 1);

        let n = 1i64 << range_bits;
        let half_n = n / 2;
        let dlog = rng.random_range(-half_n..half_n);
        let h = mul_by_generator::<G>(dlog);

        let wild_bound = half_n;
        let tame_bound = ((half_n as f64) * self.gamma).ceil() as i64;
        assert!(tame_bound > 0);

        let mut tames = HashMap::new();
        let mut wilds = HashMap::new();
        let mut iters = 0u64;

        loop {
            let t_distance = rng.random_range(-tame_bound..tame_bound);
            let t = generator_scalar_mul_i64::<G>(t_distance);
            iters += 1;
            let (t, t_distance) = if t.is_negation_map_representative() {
                (t, t_distance)
            } else {
                (-t, -t_distance)
            };
            if let Some(w_distance) = wilds.get(&t) {
                println!("{}: {} {}", dlog, t_distance - w_distance, w_distance - t_distance);
                return iters;
            }
            tames.insert(t, t_distance);

            let w_distance = rng.random_range(-wild_bound..wild_bound) / 2 * 2;
            let w = h + mul_by_generator::<G>(w_distance);
            iters += 1;
            let (w, w_distance) = if w.is_negation_map_representative() {
                (w, w_distance)
            } else {
                (-w, -w_distance)
            };
            if let Some(t_distance) = tames.get(&w) {
                println!("{}: {} {}", dlog, t_distance - w_distance, w_distance - t_distance);
                return iters;
            }
            if let Some(&w2_distance) = wilds.get(&w) {
                if w_distance != w2_distance {
                    println!("{}: {} {} {} {}", dlog,
                             (w_distance - w2_distance) / 2,
                             (w2_distance - w_distance) / 2,
                             (w_distance + w2_distance) / 2,
                             (-w_distance - w2_distance) / 2
                    );
                    return iters;
                }
            }
            wilds.insert(w, w_distance);

            let w_distance = rng.random_range(-wild_bound..wild_bound) / 2 * 2;
            let w = h + mul_by_generator::<G>(w_distance);
            iters += 1;
            let (w, w_distance) = if w.is_negation_map_representative() {
                (w, w_distance)
            } else {
                (-w, -w_distance)
            };
            if let Some(t_distance) = tames.get(&w) {
                println!("{}: {} {}", dlog, t_distance - w_distance, w_distance - t_distance);
                return iters;
            }
            if let Some(&w2_distance) = wilds.get(&w) {
                if w_distance != w2_distance {
                    println!("{}: {} {} {} {}", dlog,
                             (w_distance - w2_distance) / 2,
                             (w2_distance - w_distance) / 2,
                             (w_distance + w2_distance) / 2,
                             (-w_distance - w2_distance) / 2
                    );
                    return iters;
                }
            }
            wilds.insert(w, w_distance);
        }
    }
}

pub fn gs<G: KangarooGroup>(gamma: f64, rng: &mut impl Rng) -> f64 {
    gs_with_params::<G>(gamma, DEFAULT_GS_RANGE_BITS, DEFAULT_GS_COUNT, rng)
}

pub fn gs_with_params<G: KangarooGroup>(
    gamma: f64,
    range_bits: u32,
    count: u64,
    rng: &mut impl Rng,
) -> f64 {
    assert!(count > 0);

    let sim = GaudrySchostSimulator::new(gamma);
    let n = (1u64 << range_bits) as f64;
    let mut k_sum = 0.0;

    for _ in 0..count {
        let k = sim.simulate::<G>(range_bits, rng);
        k_sum += (k as f64) / n.sqrt();
    }

    k_sum / (count as f64)
}

fn mul_by_generator<G: KangarooGroup>(scalar: i64) -> G {
    let scalar_abs = G::Scalar::from(scalar.unsigned_abs());
    if scalar < 0 {
        -G::mul_by_generator(&scalar_abs)
    } else {
        G::mul_by_generator(&scalar_abs)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use rand::SeedableRng;
    use rand::rngs::{SysRng, Xoshiro256PlusPlus};

    use super::*;
    use crate::group::ectoy::ECToyGroup;
    use crate::group::toy::ToyGroup;

    #[test]
    fn simulate_gaudry_schost_toy_smoke() -> Result<(), Box<dyn Error>> {
        for _ in 1..256 {
            simulate_gaudry_schost_smoke::<ToyGroup>()?;
        }
        Ok(())
    }

    fn simulate_gaudry_schost_smoke<G: KangarooGroup>() -> Result<(), Box<dyn Error>> {
        let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
        let sim = GaudrySchostSimulator::new(1.0 / 32.0);

        let iters = sim.simulate::<G>(32, &mut rng);
        // println!("iters: {:.2}", ((iters as f64) / ((1u64 << 32) as f64).sqrt()));
        // assert!(iters > 0);

        // let k = gs_with_params::<G>(1.0 / 32.0, 8, 4, &mut rng);
        // assert!(k.is_finite());
        // assert!(k > 0.0);
        Ok(())
    }
}

// use crate::group::toy::ToyGroup;
// use crate::group::{CyclicGroup, KangarooGroup};
// use rand::{Rng, RngExt};
// use std::collections::{HashMap, HashSet};
//
// pub struct GaudrySchostSimulator {
//     gamma: f64,
// }
//
// impl GaudrySchostSimulator {
//     pub fn new(gamma: f64) -> Self {
//         GaudrySchostSimulator { gamma }
//     }
//
//     pub fn simulate(&self, range_bits: u32, rng: &mut impl Rng) -> u64 {
//         let n = 1i64 << range_bits;
//         let half_n = n / 2;
//         let dlog = rng.random_range(-half_n..half_n);
//         let h = ToyGroup::mul_by_generator(dlog);
//
//         let wild_bound = half_n;
//         let tame_bound = ((half_n as f64) * self.gamma).ceil() as i64;
//         let mut tames = HashSet::new();
//         let mut wilds = HashMap::new();
//         let mut i = 0u64;
//
//         loop {
//             let t_distance = rng.random_range(-tame_bound..tame_bound);
//             let t = ToyGroup::mul_by_generator(t_distance);
//             i += 1;
//             let t = if t.is_negation_map_representative() {
//                 t
//             } else {
//                 -t
//             };
//             if wilds.contains_key(&t) {
//                 return i;
//             }
//             tames.insert(t);
//
//             let w_distance = rng.random_range(-wild_bound..wild_bound) / 2 * 2;
//             let w = h + ToyGroup::mul_by_generator(w_distance);
//             i += 1;
//             let w = if w.is_negation_map_representative() {
//                 w
//             } else {
//                 -w
//             };
//             if tames.contains(&w) {
//                 return i;
//             }
//             if let Some(&distance) = wilds.get(&w) && distance != w_distance {
//                 return i;
//             }
//             wilds.insert(w, w_distance);
//
//             let w_distance = rng.random_range(-wild_bound..wild_bound) / 2 * 2;
//             let w = h + ToyGroup::mul_by_generator(w_distance);
//             i += 1;
//             let (w, w_distance) = if w.is_negation_map_representative() {
//                 (w, w_distance)
//             } else {
//                 (-w, -w_distance)
//             };
//             if tames.contains(&w) {
//                 return i;
//             }
//             if let Some(&distance) = wilds.get(&w) && distance != w_distance && distance != -w_distance {
//                 return i;
//             }
//             wilds.insert(w, w_distance);
//         }
//     }
// }
//
// pub fn gs(gamma: f64, rng: &mut impl Rng) -> f64 {
//     let sim = GaudrySchostSimulator::new(gamma);
//
//     const COUNT: u64 = 4096;
//     const RANGE: u32 = 50;
//
//     let mut k_sum = 0.;
//     for _ in 0..COUNT {
//         let k = sim.simulate(RANGE, rng);
//         let k = (k as f64) / ((1u64 << RANGE) as f64).sqrt();
//         k_sum += k;
//     }
//     let k = k_sum / (COUNT as f64);
//     k
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rand::rngs::{SysRng, Xoshiro256PlusPlus};
//     use rand::SeedableRng;
//     use std::error::Error;
//
//     #[test]
//     fn simulate_gaudry_schost() -> Result<(), Box<dyn Error>> {
//         let mut rng = Xoshiro256PlusPlus::try_from_rng(&mut SysRng)?;
//         let k = gs(1.0 / 32.0, &mut rng);
//         println!("{}", k);
//         Ok(())
//     }
// }
