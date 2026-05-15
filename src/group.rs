use std::hash::Hash;

use group::Group;

pub mod ectoy;
pub mod toy;

pub trait KangarooGroup: Group + Hash {
    fn is_distinguished(&self, bits: u32) -> bool;
    fn is_negation_map_representative(&self) -> bool;
    fn bucket(&self, bits: u32) -> usize;
    fn footprint(&self) -> u64;
}

pub fn scalar_mul_i64<G: Group>(g: G, scalar: i64) -> G {
    let s = G::Scalar::from(scalar.unsigned_abs());
    if scalar < 0 { -g * s } else { g * s }
}

pub fn generator_scalar_mul_i64<G: Group>(scalar: i64) -> G {
    let s = G::Scalar::from(scalar.unsigned_abs());
    if scalar < 0 {
        -G::mul_by_generator(&s)
    } else {
        G::mul_by_generator(&s)
    }
}
