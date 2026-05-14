use std::hash::Hash;

use group::Group;

pub mod ectoy;
pub mod helpers;
pub mod toy;

pub trait KangarooGroup: Group + Hash {
    fn is_distinguished(&self, bits: u32) -> bool;
    fn is_negation_map_representative(&self) -> bool;
    fn bucket(&self, bits: u32) -> usize;
    fn footprint(&self) -> u64;
}
