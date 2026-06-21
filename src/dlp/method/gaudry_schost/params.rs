use crate::group::{KangarooGroup, generator_scalar_mul_i64};
use num_integer::gcd;
use rand::{Rng, RngExt};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::ops::ControlFlow;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Type {
    TAME,
    WILD,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Parity {
    ODD,
    EVEN,
}

#[derive(Clone)]
pub struct Params<G: KangarooGroup> {
    n: i64,
    buckets_bits: u32,
    dp_bits: u32,
    jump_table_distances: Vec<i64>,
    jump_table_elements: Vec<G>,
    escape_distance: i64,
    escape_element: G,
    max_steps: u64,
    guard_interval: i64,
}

impl<G: KangarooGroup> Params<G> {
    pub fn new(n: i64, c: f64, buckets_bits: u32, dp_bits: u32, rng: &mut impl Rng) -> Self {
        debug_assert!(0 < n);
        debug_assert!(0 < buckets_bits && buckets_bits < 32);
        debug_assert!(0 < dp_bits && dp_bits < 64);
        debug_assert!(0. < c);

        let absolute_jump_mean_f64 = (n as f64).sqrt() * c;
        let max_absolute_jump = (absolute_jump_mean_f64 * 2.).ceil() as i64;
        let jump_table_size = 1_usize << buckets_bits;
        let mut jump_table_distances = Vec::with_capacity(jump_table_size);
        let mut jump_table_elements = Vec::with_capacity(jump_table_size);
        let available_jump_distances = max_absolute_jump.saturating_sub(1) as usize;
        if available_jump_distances >= jump_table_size {
            let mut used_jump_distances = HashSet::with_capacity(jump_table_size);
            while jump_table_distances.len() < jump_table_size {
                let d = rng.random_range(2..max_absolute_jump) / 2 * 2;
                if !used_jump_distances.insert(d) {
                    continue;
                }
                let e = generator_scalar_mul_i64::<G>(d);
                jump_table_distances.push(d);
                jump_table_elements.push(e);
            }
        } else {
            for _ in 0..jump_table_size {
                let d = rng.random_range(2..max_absolute_jump) / 2 * 2;
                let e = generator_scalar_mul_i64::<G>(d);
                jump_table_distances.push(d);
                jump_table_elements.push(e);
            }
        }
        debug_assert!(jump_table_distances.iter().copied().reduce(gcd).unwrap() == 2);
        debug_assert!(
            available_jump_distances < jump_table_size
                || jump_table_distances
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>()
                    .len()
                    == jump_table_distances.len()
        );

        let mut escape_distance = rng.random_range(2..max_absolute_jump) / 2 * 2;
        if available_jump_distances > jump_table_size {
            while jump_table_distances.iter().any(|&d| d == escape_distance) {
                escape_distance = rng.random_range(2..max_absolute_jump) / 2 * 2;
            }
        }
        let escape_element = generator_scalar_mul_i64::<G>(escape_distance);
        debug_assert!(
            available_jump_distances <= jump_table_size
                || jump_table_distances.iter().all(|&d| d != escape_distance)
        );

        let dp_probability = 2.0_f64.powi(-(dp_bits as i32));
        let max_steps = (1_u64 << dp_bits) * 20;
        let guard_interval =
            (absolute_jump_mean_f64 * (2.0 / (3.0 * PI * dp_probability)).sqrt()).ceil() as i64;

        Params {
            n,
            buckets_bits,
            dp_bits,
            jump_table_distances,
            jump_table_elements,
            escape_distance,
            escape_element,
            max_steps,
            guard_interval,
        }
    }
}

pub struct Walk<G: KangarooGroup> {
    inner: InnerWalk<G>,
    steps_c2: u64,
    max_c2: u64,
    steps: u64,
}

impl<G: KangarooGroup> Walk<G> {
    pub fn new(params: Params<G>, typ: Type, parity: Parity, anchor: G, alpha: f64, rng: &mut impl Rng) -> Self {
        let inner = InnerWalk::new(params, typ, anchor, parity, alpha, rng);
        Walk {
            inner,
            steps_c2: 0,
            max_c2: 24,
            steps: 0,
        }
    }

    pub fn walk(
        &mut self,
        dp_map: &mut HashMap<G, (Type, i64)>,
        rng: &mut impl Rng,
    ) -> ControlFlow<(i64, u64), u64> {
        let mut ops = 1;
        if self.inner.step(dp_map, rng).map_break(|s| (s, ops))? {
            self.steps_c2 = 0;
            self.steps = 0;
        } else {
            self.steps_c2 += 1;
            self.steps += 1;
        }

        if self.steps >= self.inner.params.max_steps {
            ops += 1;
            self.inner.reset(rng);
            self.steps_c2 = 0;
            self.steps = 0;
            return ControlFlow::Continue(ops);
        }

        if self.steps_c2 >= self.max_c2 {
            self.steps_c2 = 0;
            let mut footprint = self.inner.p.footprint();
            for _ in 1..=4 {
                ops += 1;
                self.steps += 1;
                if self.inner.step(dp_map, rng).map_break(|s| (s, ops))? {
                    self.steps = 0;
                    return ControlFlow::Continue(ops);
                }
                if self.inner.p.footprint() == footprint {
                    // cycle
                    ops += 1;
                    self.steps += 1;
                    if self.inner.escape(dp_map, rng).map_break(|s| (s, ops))? {
                        self.steps = 0;
                    }
                    return ControlFlow::Continue(ops);
                }
                if self.inner.p.footprint() < footprint {
                    footprint = self.inner.p.footprint();
                }
            }
        }

        ControlFlow::Continue(ops)
    }
}

struct InnerWalk<G: KangarooGroup> {
    params: Params<G>,
    typ: Type,
    parity: Parity,
    anchor: G,
    bound: i64,
    d: i64,
    p: G,
}

impl<G: KangarooGroup> InnerWalk<G> {
    pub fn new(params: Params<G>, typ: Type, anchor: G, parity: Parity, alpha: f64, rng: &mut impl Rng) -> Self {
        debug_assert!(0.0 < alpha);

        let bound = ((((params.n / 2) as f64) * alpha).round() as i64) - params.guard_interval;
        debug_assert!(0 < bound);

        let mut walk = InnerWalk {
            params,
            typ,
            parity,
            anchor,
            bound,
            d: 0,
            p: G::identity(),
        };
        walk.reset(rng);
        walk
    }

    pub fn reset(&mut self, rng: &mut impl Rng) {
        self.d = rng.random_range(-self.bound..self.bound);
        if self.parity == Parity::EVEN {
            self.d = self.d / 2 * 2;
        } else {
            self.d = self.d / 2 * 2 + 1;
        }

        self.p = self.anchor + generator_scalar_mul_i64::<G>(self.d);
        if !self.p.is_negation_map_representative() {
            self.d = -self.d;
            self.p = -self.p;
        }
    }

    pub fn step(
        &mut self,
        dp_map: &mut HashMap<G, (Type, i64)>,
        rng: &mut impl Rng,
    ) -> ControlFlow<i64, bool> {
        let b = self.p.bucket(self.params.buckets_bits);
        let jump_d = self.params.jump_table_distances[b];
        let jump_p = self.params.jump_table_elements[b];
        self.d += jump_d;
        self.p += jump_p;
        if !self.p.is_negation_map_representative() {
            self.d = -self.d;
            self.p = -self.p;
        }
        if self.p.is_distinguished(self.params.dp_bits) {
            if let Some(&(typ2, d2)) = dp_map.get(&self.p) {
                if self.typ == Type::TAME && typ2 == Type::WILD {
                    return ControlFlow::Break(self.d - d2);
                }
                if self.typ == Type::WILD && typ2 == Type::TAME {
                    return ControlFlow::Break(d2 - self.d);
                }
                if self.typ == Type::WILD && typ2 == Type::WILD && self.d != d2 && self.d != -d2 {
                    return ControlFlow::Break((self.d - d2) / 2);
                }
            }
            dp_map.insert(self.p, (self.typ, self.d));
            self.reset(rng);
            return ControlFlow::Continue(true);
        }

        ControlFlow::Continue(false)
    }

    pub fn escape(
        &mut self,
        dp_map: &mut HashMap<G, (Type, i64)>,
        rng: &mut impl Rng,
    ) -> ControlFlow<i64, bool> {
        self.d += self.params.escape_distance;
        self.p += self.params.escape_element;
        if !self.p.is_negation_map_representative() {
            self.d = -self.d;
            self.p = -self.p;
        }
        if self.p.is_distinguished(self.params.dp_bits) {
            if let Some(&(typ2, d2)) = dp_map.get(&self.p) {
                if self.typ == Type::TAME && typ2 == Type::WILD {
                    return ControlFlow::Break(self.d - d2);
                }
                if self.typ == Type::WILD && typ2 == Type::TAME {
                    return ControlFlow::Break(d2 - self.d);
                }
                if self.typ == Type::WILD && typ2 == Type::WILD && self.d != d2 && self.d != -d2 {
                    return ControlFlow::Break((self.d - d2) / 2);
                }
            }
            dp_map.insert(self.p, (self.typ, self.d));
            self.reset(rng);
            return ControlFlow::Continue(true);
        }

        ControlFlow::Continue(false)
    }
}
