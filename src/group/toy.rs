use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::group::KangarooGroup;
use ff::helpers::sqrt_ratio_generic;
use ff::{Field, FieldBits, PrimeField, PrimeFieldBits};
use group::cofactor::CofactorGroup;
use group::prime::PrimeGroup;
use group::{Group, GroupEncoding};
use rand::TryRng;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
#[repr(transparent)]
pub struct ToyScalar(u64);

impl ToyScalar {
    pub const ZERO: ToyScalar = ToyScalar(0);
    pub const ONE: ToyScalar = ToyScalar(1);

    pub const MODULUS: u64 = (1 << 63) - Self::C;
    const C: u64 = 25;

    #[inline]
    pub fn from_u64_reduce(v: u64) -> Self {
        let mut x = v;
        if x >= Self::MODULUS {
            x -= Self::MODULUS;
        }
        if x >= Self::MODULUS {
            x -= Self::MODULUS;
        }
        ToyScalar(x)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: Self) -> Self {
        let v = self.0 + rhs.0;
        if v >= Self::MODULUS {
            ToyScalar(v - Self::MODULUS)
        } else {
            ToyScalar(v)
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            ToyScalar(self.0 - rhs.0)
        } else {
            ToyScalar(self.0 + Self::MODULUS - rhs.0)
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: Self) -> Self {
        let mut v = self.0 as u128 * rhs.0 as u128;
        v = (v >> 63) * (Self::C as u128) + (v & ((1 << 63) - 1));
        v = (v >> 63) * (Self::C as u128) + (v & ((1 << 63) - 1));
        if v >= Self::MODULUS as u128 {
            v -= Self::MODULUS as u128;
        }
        ToyScalar(v as u64)
    }

    pub fn pow_u64(self, rhs: u64) -> Self {
        let mut result = Self::ONE;
        let mut base = self;
        let mut exp = rhs;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul(base);
            }
            base = base.mul(base);
            exp >>= 1;
        }

        result
    }

    pub fn pow_i64(self, rhs: i64) -> Self {
        if rhs < 0 {
            self.inv().pow_u64(rhs.unsigned_abs())
        } else {
            self.pow_u64(rhs.unsigned_abs())
        }
    }

    pub fn inv(self) -> Self {
        assert_ne!(self, Self::ZERO);
        self.pow_u64(Self::MODULUS - 2)
    }
}

impl Field for ToyScalar {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;

    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Self, R::Error> {
        loop {
            let v = rng.try_next_u64()? & ((1 << 63) - 1);
            if v < Self::MODULUS {
                return Ok(ToyScalar(v));
            }
        }
    }

    fn square(&self) -> Self {
        self.mul(*self)
    }

    fn double(&self) -> Self {
        self.add(*self)
    }

    fn invert(&self) -> CtOption<Self> {
        let is_nonzero = !self.is_zero();
        let inverse = if bool::from(is_nonzero) {
            self.inv()
        } else {
            Self::ZERO
        };
        CtOption::new(inverse, is_nonzero)
    }

    fn sqrt_ratio(num: &Self, div: &Self) -> (Choice, Self) {
        sqrt_ratio_generic(num, div)
    }

    fn sqrt(&self) -> CtOption<Self> {
        let sqrt = self.pow_u64((Self::MODULUS + 1) / 4);
        CtOption::new(sqrt, sqrt.square().ct_eq(self))
    }
}

impl PrimeField for ToyScalar {
    type Repr = [u8; 8];

    fn from_u128(v: u128) -> Self {
        assert!(v < Self::MODULUS as u128);
        ToyScalar(v as u64)
    }
    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        let v = u64::from_le_bytes(repr);
        CtOption::new(ToyScalar(v), Choice::from((v < ToyScalar::MODULUS) as u8))
    }
    fn to_repr(&self) -> Self::Repr {
        self.0.to_le_bytes()
    }
    fn is_odd(&self) -> Choice {
        Choice::from((self.0 & 1) as u8)
    }
    const MODULUS: &'static str = "9223372036854775783";
    const NUM_BITS: u32 = 63;
    const CAPACITY: u32 = 62;
    const TWO_INV: Self = ToyScalar(4611686018427387892);
    const MULTIPLICATIVE_GENERATOR: Self = ToyScalar(3);

    const S: u32 = 1;

    const ROOT_OF_UNITY: Self = ToyScalar(ToyScalar::MODULUS - 1);

    const ROOT_OF_UNITY_INV: Self = ToyScalar(ToyScalar::MODULUS - 1);

    const DELTA: Self = ToyScalar(9);
}

impl PrimeFieldBits for ToyScalar {
    type ReprBits = [u64; 1];

    fn to_le_bits(&self) -> FieldBits<Self::ReprBits> {
        [self.0].into()
    }

    fn char_le_bits() -> FieldBits<Self::ReprBits> {
        [ToyScalar::MODULUS].into()
    }
}

impl ConditionallySelectable for ToyScalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        ToyScalar(u64::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for ToyScalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl From<u64> for ToyScalar {
    fn from(value: u64) -> Self {
        assert!(value < Self::MODULUS);
        ToyScalar(value)
    }
}

impl Add for ToyScalar {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ToyScalar::add(self, rhs)
    }
}

impl Add<&ToyScalar> for ToyScalar {
    type Output = ToyScalar;

    fn add(self, rhs: &ToyScalar) -> Self::Output {
        ToyScalar::add(self, *rhs)
    }
}

impl AddAssign for ToyScalar {
    fn add_assign(&mut self, rhs: Self) {
        *self = ToyScalar::add(*self, rhs);
    }
}

impl AddAssign<&ToyScalar> for ToyScalar {
    fn add_assign(&mut self, rhs: &ToyScalar) {
        *self = ToyScalar::add(*self, *rhs);
    }
}

impl Sub for ToyScalar {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ToyScalar::sub(self, rhs)
    }
}

impl Sub<&ToyScalar> for ToyScalar {
    type Output = ToyScalar;

    fn sub(self, rhs: &ToyScalar) -> Self::Output {
        ToyScalar::sub(self, *rhs)
    }
}

impl SubAssign for ToyScalar {
    fn sub_assign(&mut self, rhs: Self) {
        *self = ToyScalar::sub(*self, rhs);
    }
}

impl SubAssign<&ToyScalar> for ToyScalar {
    fn sub_assign(&mut self, rhs: &ToyScalar) {
        *self = ToyScalar::sub(*self, *rhs);
    }
}

impl Mul for ToyScalar {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        ToyScalar::mul(self, rhs)
    }
}

impl Mul<&ToyScalar> for ToyScalar {
    type Output = ToyScalar;

    fn mul(self, rhs: &ToyScalar) -> Self::Output {
        ToyScalar::mul(self, *rhs)
    }
}

impl MulAssign for ToyScalar {
    fn mul_assign(&mut self, rhs: Self) {
        *self = ToyScalar::mul(*self, rhs);
    }
}

impl MulAssign<&ToyScalar> for ToyScalar {
    fn mul_assign(&mut self, rhs: &ToyScalar) {
        *self = ToyScalar::mul(*self, *rhs);
    }
}

impl Neg for ToyScalar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.is_zero_vartime() {
            Self::ZERO
        } else {
            ToyScalar(ToyScalar::MODULUS - self.0)
        }
    }
}

impl Sum for ToyScalar {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or(Self::ZERO)
    }
}

impl<'a> Sum<&'a ToyScalar> for ToyScalar {
    fn sum<I: Iterator<Item = &'a ToyScalar>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl Product for ToyScalar {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Mul::mul).unwrap_or(Self::ONE)
    }
}

impl<'a> Product<&'a ToyScalar> for ToyScalar {
    fn product<I: Iterator<Item = &'a ToyScalar>>(iter: I) -> Self {
        iter.copied().product()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
#[repr(transparent)]
pub struct ToyGroup(ToyScalar);

impl ToyGroup {
    pub const ZERO: Self = ToyGroup(ToyScalar::ZERO);
    pub const GENERATOR: Self = ToyGroup(ToyScalar::ONE);

    const DISTINGUISHED_DST: u64 = u64::from_le_bytes(*b"DSTNGSHD");
    const NEGATION_MAP_DST: u64 = u64::from_le_bytes(*b"NEGATMAP");
    const BUCKET_DST: u64 = u64::from_le_bytes(*b"THEBUCKT");
    const FOOTPRINT_DST: u64 = u64::from_le_bytes(*b"FUTPRINT");

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: Self) -> Self {
        ToyGroup(self.0 + rhs.0)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: Self) -> Self {
        ToyGroup(self.0 - rhs.0)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Self {
        ToyGroup(-self.0)
    }

    #[inline]
    pub fn scalar_mul_u64(self, s: u64) -> Self {
        ToyGroup(self.0.mul(ToyScalar::from_u64_reduce(s)))
    }

    #[inline]
    pub fn scalar_mul_i64(self, s: i64) -> Self {
        let s_abs = s.unsigned_abs();
        if s < 0 {
            self.neg().scalar_mul_u64(s_abs)
        } else {
            self.scalar_mul_u64(s_abs)
        }
    }

    #[inline]
    pub fn generator_scalar_mul_u64(s: u64) -> Self {
        // generator is 1 so it's just that
        ToyGroup(ToyScalar::from_u64_reduce(s))
    }

    #[inline]
    pub fn generator_scalar_mul_i64(s: i64) -> Self {
        let s_abs = s.unsigned_abs();
        if s < 0 {
            Self::generator_scalar_mul_u64(s_abs).neg()
        } else {
            Self::generator_scalar_mul_u64(s_abs)
        }
    }

    pub fn digest(&self, dst: u64) -> u64 {
        let mut x = (self.0).0 ^ dst;
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58476d1ce4e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d049bb133111eb);
        x ^= x >> 31;
        x
    }
}

impl Add for ToyGroup {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ToyGroup::add(self, rhs)
    }
}

impl Add<&ToyGroup> for ToyGroup {
    type Output = ToyGroup;

    fn add(self, rhs: &ToyGroup) -> Self::Output {
        ToyGroup::add(self, *rhs)
    }
}

impl AddAssign for ToyGroup {
    fn add_assign(&mut self, rhs: Self) {
        *self = ToyGroup::add(*self, rhs);
    }
}

impl AddAssign<&ToyGroup> for ToyGroup {
    fn add_assign(&mut self, rhs: &ToyGroup) {
        *self = ToyGroup::add(*self, *rhs);
    }
}

impl Sub for ToyGroup {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ToyGroup::sub(self, rhs)
    }
}

impl Sub<&ToyGroup> for ToyGroup {
    type Output = ToyGroup;

    fn sub(self, rhs: &ToyGroup) -> Self::Output {
        ToyGroup::sub(self, *rhs)
    }
}

impl SubAssign for ToyGroup {
    fn sub_assign(&mut self, rhs: Self) {
        *self = ToyGroup::sub(*self, rhs);
    }
}

impl SubAssign<&ToyGroup> for ToyGroup {
    fn sub_assign(&mut self, rhs: &ToyGroup) {
        *self = ToyGroup::sub(*self, *rhs);
    }
}

impl Neg for ToyGroup {
    type Output = Self;

    fn neg(self) -> Self::Output {
        ToyGroup::neg(self)
    }
}

impl Mul<ToyScalar> for ToyGroup {
    type Output = Self;

    fn mul(self, rhs: ToyScalar) -> Self::Output {
        ToyGroup(self.0 * rhs)
    }
}

impl Mul<&ToyScalar> for ToyGroup {
    type Output = Self;

    fn mul(self, rhs: &ToyScalar) -> Self::Output {
        self * *rhs
    }
}

impl MulAssign<ToyScalar> for ToyGroup {
    fn mul_assign(&mut self, rhs: ToyScalar) {
        *self = *self * rhs;
    }
}

impl MulAssign<&ToyScalar> for ToyGroup {
    fn mul_assign(&mut self, rhs: &ToyScalar) {
        *self = *self * rhs;
    }
}

impl Sum for ToyGroup {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or(Self::ZERO)
    }
}

impl<'a> Sum<&'a ToyGroup> for ToyGroup {
    fn sum<I: Iterator<Item = &'a ToyGroup>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl Group for ToyGroup {
    type Scalar = ToyScalar;

    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Self, R::Error> {
        loop {
            let scalar = ToyScalar::try_from_rng(rng)?;
            if !bool::from(scalar.is_zero()) {
                return Ok(ToyGroup(scalar));
            }
        }
    }

    fn identity() -> Self {
        Self::ZERO
    }

    fn generator() -> Self {
        Self::GENERATOR
    }

    fn is_identity(&self) -> Choice {
        self.0.is_zero()
    }

    fn double(&self) -> Self {
        ToyGroup(self.0.double())
    }

    fn mul_by_generator(scalar: &Self::Scalar) -> Self {
        ToyGroup(*scalar)
    }
}

impl GroupEncoding for ToyGroup {
    type Repr = [u8; 8];

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        ToyScalar::from_repr(*bytes).map(ToyGroup)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        Self::from_bytes(bytes)
    }

    fn to_bytes(&self) -> Self::Repr {
        self.0.to_repr()
    }
}

impl PrimeGroup for ToyGroup {}

impl CofactorGroup for ToyGroup {
    type Subgroup = ToyGroup;

    fn clear_cofactor(&self) -> Self::Subgroup {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self::Subgroup> {
        CtOption::new(self, Choice::from(1))
    }

    fn is_torsion_free(&self) -> Choice {
        Choice::from(1)
    }
}

impl KangarooGroup for ToyGroup {
    fn is_distinguished(&self, bits: u32) -> bool {
        assert!(0 < bits && bits < 64);
        (self.digest(Self::DISTINGUISHED_DST) >> (64 - bits)) == 0
    }

    fn is_negation_map_representative(&self) -> bool {
        self.digest(Self::NEGATION_MAP_DST) < self.neg().digest(Self::NEGATION_MAP_DST)
    }

    fn bucket(&self, bits: u32) -> usize {
        assert!(0 < bits && bits < 64);
        (self.digest(Self::BUCKET_DST) >> (64 - bits)) as usize
    }

    fn footprint(&self) -> u64 {
        self.digest(Self::FOOTPRINT_DST)
    }
}

// use crate::group::{CyclicGroup, KangarooGroup};
// use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
//
// #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
// pub struct ToyGroup(u64);
//
// impl ToyGroup {
//     pub const ORDER: u64 = (1u64 << 63) - 25;
//
//     pub fn mul_by_generator(scalar: i64) -> Self {
//         let abs = scalar.unsigned_abs();
//         assert!(abs < Self::ORDER);
//         if scalar < 0 {
//             -ToyGroup(abs)
//         } else {
//             ToyGroup(abs)
//         }
//     }
//
//     pub fn digest(&self, domain_sep: u64) -> u64 {
//         let mut x = self.0 ^ domain_sep;
//         x ^= x >> 30;
//         x = x.wrapping_mul(0xbf58476d1ce4e5b9);
//         x ^= x >> 27;
//         x = x.wrapping_mul(0x94d049bb133111eb);
//         x ^= x >> 31;
//         x
//     }
//
//     const DISTINGUISHED_SEP: u64 = 0xab9b3596f5cbddaf;
//     const NEGATION_MAP_SEP: u64 = 0xda38a1025b06995e;
//     const BUCKET_SEP: u64 = 0x3c7c1a5ae7a7556e;
// }
//
// impl CyclicGroup for ToyGroup {
//     fn zero() -> Self {
//         ToyGroup(0)
//     }
//
//     fn generator() -> Self {
//         ToyGroup(1)
//     }
// }
//
// impl Add for ToyGroup {
//     type Output = Self;
//
//     fn add(self, rhs: Self) -> Self::Output {
//         let v = self.0 + rhs.0;
//         if v >= Self::ORDER {
//             ToyGroup(v - Self::ORDER)
//         } else {
//             ToyGroup(v)
//         }
//     }
// }
//
// impl AddAssign for ToyGroup {
//     fn add_assign(&mut self, rhs: Self) {
//         *self = *self + rhs;
//     }
// }
//
// impl Sub for ToyGroup {
//     type Output = Self;
//     fn sub(self, rhs: Self) -> Self::Output {
//         if rhs.0 > self.0 {
//             ToyGroup(self.0 + Self::ORDER - rhs.0)
//         } else {
//             ToyGroup(self.0 - rhs.0)
//         }
//     }
// }
//
// impl SubAssign for ToyGroup {
//     fn sub_assign(&mut self, rhs: Self) {
//         *self = *self - rhs;
//     }
// }
//
// impl Neg for ToyGroup {
//     type Output = Self;
//     fn neg(self) -> Self::Output {
//         if self.0 == 0 {
//             ToyGroup::zero()
//         } else {
//             ToyGroup(Self::ORDER - self.0)
//         }
//     }
// }
//
// impl KangarooGroup for ToyGroup {
//     fn is_distinguished(&self, bits: u32) -> bool {
//         debug_assert!(0 < bits && bits < 64);
//         let h = self.digest(Self::DISTINGUISHED_SEP);
//         (h >> (64 - bits)) == 0
//     }
//
//     fn is_negation_map_representative(&self) -> bool {
//         self.digest(Self::NEGATION_MAP_SEP) < (-*self).digest(Self::NEGATION_MAP_SEP)
//     }
//
//     fn bucket(&self, bits: u32) -> usize {
//         debug_assert!(0 < bits && bits < 64);
//         let h = self.digest(Self::BUCKET_SEP);
//         (h >> (64 - bits)) as usize
//     }
// }
