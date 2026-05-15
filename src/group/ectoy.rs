use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use ff::helpers::sqrt_ratio_generic;
use ff::{Field, FieldBits, PrimeField, PrimeFieldBits};
use group::cofactor::{CofactorCurve, CofactorCurveAffine, CofactorGroup};
use group::prime::{PrimeCurve, PrimeCurveAffine, PrimeGroup};
use group::{Curve, Group, GroupEncoding};
use rand::TryRng;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use crate::group::KangarooGroup;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
#[repr(transparent)]
pub struct ECToyField(u64);

impl ECToyField {
    pub const ZERO: Self = ECToyField(0);
    pub const ONE: Self = ECToyField(1);

    pub const MODULUS: u64 = (1 << 63) - Self::C;
    const C: u64 = 4321;

    #[inline]
    pub fn from_u64_reduce(v: u64) -> Self {
        let mut x = v;
        if x >= Self::MODULUS {
            x -= Self::MODULUS;
        }
        if x >= Self::MODULUS {
            x -= Self::MODULUS;
        }
        ECToyField(x)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: Self) -> Self {
        let v = self.0 + rhs.0;
        if v >= Self::MODULUS {
            ECToyField(v - Self::MODULUS)
        } else {
            ECToyField(v)
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            ECToyField(self.0 - rhs.0)
        } else {
            ECToyField(self.0 + Self::MODULUS - rhs.0)
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
        ECToyField(v as u64)
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

impl Field for ECToyField {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;

    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Self, R::Error> {
        loop {
            let v = rng.try_next_u64()? & ((1 << 63) - 1);
            if v < ECToyField::MODULUS {
                return Ok(ECToyField(v));
            }
        }
    }

    fn square(&self) -> Self {
        ECToyField::mul(*self, *self)
    }

    fn double(&self) -> Self {
        ECToyField::add(*self, *self)
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
        let sqrt = self.pow_u64((ECToyField::MODULUS + 1) / 4);
        CtOption::new(sqrt, sqrt.square().ct_eq(self))
    }
}

impl PrimeField for ECToyField {
    type Repr = [u8; 8];

    const MODULUS: &'static str = "9223372036854771487";
    const NUM_BITS: u32 = 63;
    const CAPACITY: u32 = 62;
    const TWO_INV: Self = ECToyField(4611686018427385744);
    const MULTIPLICATIVE_GENERATOR: Self = ECToyField(3);
    const S: u32 = 1;
    const ROOT_OF_UNITY: Self = ECToyField(ECToyField::MODULUS - 1);
    const ROOT_OF_UNITY_INV: Self = ECToyField(ECToyField::MODULUS - 1);
    const DELTA: Self = ECToyField(9);

    fn from_u128(v: u128) -> Self {
        assert!(v < ECToyField::MODULUS as u128);
        ECToyField(v as u64)
    }

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        let v = u64::from_le_bytes(repr);
        CtOption::new(ECToyField(v), Choice::from((v < ECToyField::MODULUS) as u8))
    }

    fn to_repr(&self) -> Self::Repr {
        self.0.to_le_bytes()
    }

    fn is_odd(&self) -> Choice {
        Choice::from((self.0 & 1) as u8)
    }
}

impl PrimeFieldBits for ECToyField {
    type ReprBits = [u64; 1];

    fn to_le_bits(&self) -> FieldBits<Self::ReprBits> {
        [self.0].into()
    }

    fn char_le_bits() -> FieldBits<Self::ReprBits> {
        [ECToyField::MODULUS].into()
    }
}

impl ConditionallySelectable for ECToyField {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        ECToyField(u64::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for ECToyField {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl From<u64> for ECToyField {
    fn from(value: u64) -> Self {
        assert!(value < ECToyField::MODULUS);
        ECToyField(value)
    }
}

impl Add for ECToyField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ECToyField::add(self, rhs)
    }
}

impl Add<&ECToyField> for ECToyField {
    type Output = ECToyField;

    fn add(self, rhs: &ECToyField) -> Self::Output {
        ECToyField::add(self, *rhs)
    }
}

impl AddAssign for ECToyField {
    fn add_assign(&mut self, rhs: Self) {
        *self = ECToyField::add(*self, rhs);
    }
}

impl AddAssign<&ECToyField> for ECToyField {
    fn add_assign(&mut self, rhs: &ECToyField) {
        *self = ECToyField::add(*self, *rhs);
    }
}

impl Sub for ECToyField {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ECToyField::sub(self, rhs)
    }
}

impl Sub<&ECToyField> for ECToyField {
    type Output = ECToyField;

    fn sub(self, rhs: &ECToyField) -> Self::Output {
        ECToyField::sub(self, *rhs)
    }
}

impl SubAssign for ECToyField {
    fn sub_assign(&mut self, rhs: Self) {
        *self = ECToyField::sub(*self, rhs);
    }
}

impl SubAssign<&ECToyField> for ECToyField {
    fn sub_assign(&mut self, rhs: &ECToyField) {
        *self = ECToyField::sub(*self, *rhs);
    }
}

impl Mul for ECToyField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        ECToyField::mul(self, rhs)
    }
}

impl Mul<&ECToyField> for ECToyField {
    type Output = ECToyField;

    fn mul(self, rhs: &ECToyField) -> Self::Output {
        ECToyField::mul(self, *rhs)
    }
}

impl MulAssign for ECToyField {
    fn mul_assign(&mut self, rhs: Self) {
        *self = ECToyField::mul(*self, rhs);
    }
}

impl MulAssign<&ECToyField> for ECToyField {
    fn mul_assign(&mut self, rhs: &ECToyField) {
        *self = ECToyField::mul(*self, *rhs);
    }
}

impl Neg for ECToyField {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.is_zero_vartime() {
            Self::ZERO
        } else {
            ECToyField(ECToyField::MODULUS - self.0)
        }
    }
}

impl Sum for ECToyField {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or(Self::ZERO)
    }
}

impl<'a> Sum<&'a ECToyField> for ECToyField {
    fn sum<I: Iterator<Item = &'a ECToyField>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl Product for ECToyField {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Mul::mul).unwrap_or(Self::ONE)
    }
}

impl<'a> Product<&'a ECToyField> for ECToyField {
    fn product<I: Iterator<Item = &'a ECToyField>>(iter: I) -> Self {
        iter.copied().product()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
#[repr(transparent)]
pub struct ECToyScalar(u64);

impl ECToyScalar {
    pub const ZERO: Self = ECToyScalar(0);
    pub const ONE: Self = ECToyScalar(1);

    pub const MODULUS: u64 = (1 << 63) - Self::C;
    const C: u64 = 0x5634b119;

    #[inline]
    pub fn from_u64_reduce(v: u64) -> Self {
        let mut x = v;
        if x >= Self::MODULUS {
            x -= Self::MODULUS;
        }
        if x >= Self::MODULUS {
            x -= Self::MODULUS;
        }
        ECToyScalar(x)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: Self) -> Self {
        let v = self.0 + rhs.0;
        if v >= Self::MODULUS {
            ECToyScalar(v - Self::MODULUS)
        } else {
            ECToyScalar(v)
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            ECToyScalar(self.0 - rhs.0)
        } else {
            ECToyScalar(self.0 + Self::MODULUS - rhs.0)
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
        ECToyScalar(v as u64)
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

impl Field for ECToyScalar {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;

    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Self, R::Error> {
        loop {
            let v = rng.try_next_u64()? & ((1 << 63) - 1);
            if v < ECToyScalar::MODULUS {
                return Ok(ECToyScalar(v));
            }
        }
    }

    fn square(&self) -> Self {
        ECToyScalar::mul(*self, *self)
    }

    fn double(&self) -> Self {
        ECToyScalar::add(*self, *self)
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
        let sqrt = self.pow_u64((ECToyScalar::MODULUS + 1) / 4);
        CtOption::new(sqrt, sqrt.square().ct_eq(self))
    }
}

impl PrimeField for ECToyScalar {
    type Repr = [u8; 8];

    const MODULUS: &'static str = "9223372035408482023";
    const NUM_BITS: u32 = 63;
    const CAPACITY: u32 = 62;
    const TWO_INV: Self = ECToyScalar(4611686017704241012);
    const MULTIPLICATIVE_GENERATOR: Self = ECToyScalar(5);
    const S: u32 = 1;
    const ROOT_OF_UNITY: Self = ECToyScalar(ECToyScalar::MODULUS - 1);
    const ROOT_OF_UNITY_INV: Self = ECToyScalar(ECToyScalar::MODULUS - 1);
    const DELTA: Self = ECToyScalar(25);

    fn from_u128(v: u128) -> Self {
        assert!(v < ECToyScalar::MODULUS as u128);
        ECToyScalar(v as u64)
    }

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        let v = u64::from_le_bytes(repr);
        CtOption::new(
            ECToyScalar(v),
            Choice::from((v < ECToyScalar::MODULUS) as u8),
        )
    }

    fn to_repr(&self) -> Self::Repr {
        self.0.to_le_bytes()
    }

    fn is_odd(&self) -> Choice {
        Choice::from((self.0 & 1) as u8)
    }
}

impl PrimeFieldBits for ECToyScalar {
    type ReprBits = [u64; 1];

    fn to_le_bits(&self) -> FieldBits<Self::ReprBits> {
        [self.0].into()
    }

    fn char_le_bits() -> FieldBits<Self::ReprBits> {
        [ECToyScalar::MODULUS].into()
    }
}

impl ConditionallySelectable for ECToyScalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        ECToyScalar(u64::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for ECToyScalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl From<u64> for ECToyScalar {
    fn from(value: u64) -> Self {
        assert!(value < ECToyScalar::MODULUS);
        ECToyScalar(value)
    }
}

impl Add for ECToyScalar {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ECToyScalar::add(self, rhs)
    }
}

impl Add<&ECToyScalar> for ECToyScalar {
    type Output = ECToyScalar;

    fn add(self, rhs: &ECToyScalar) -> Self::Output {
        ECToyScalar::add(self, *rhs)
    }
}

impl AddAssign for ECToyScalar {
    fn add_assign(&mut self, rhs: Self) {
        *self = ECToyScalar::add(*self, rhs);
    }
}

impl AddAssign<&ECToyScalar> for ECToyScalar {
    fn add_assign(&mut self, rhs: &ECToyScalar) {
        *self = ECToyScalar::add(*self, *rhs);
    }
}

impl Sub for ECToyScalar {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ECToyScalar::sub(self, rhs)
    }
}

impl Sub<&ECToyScalar> for ECToyScalar {
    type Output = ECToyScalar;

    fn sub(self, rhs: &ECToyScalar) -> Self::Output {
        ECToyScalar::sub(self, *rhs)
    }
}

impl SubAssign for ECToyScalar {
    fn sub_assign(&mut self, rhs: Self) {
        *self = ECToyScalar::sub(*self, rhs);
    }
}

impl SubAssign<&ECToyScalar> for ECToyScalar {
    fn sub_assign(&mut self, rhs: &ECToyScalar) {
        *self = ECToyScalar::sub(*self, *rhs);
    }
}

impl Mul for ECToyScalar {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        ECToyScalar::mul(self, rhs)
    }
}

impl Mul<&ECToyScalar> for ECToyScalar {
    type Output = ECToyScalar;

    fn mul(self, rhs: &ECToyScalar) -> Self::Output {
        ECToyScalar::mul(self, *rhs)
    }
}

impl MulAssign for ECToyScalar {
    fn mul_assign(&mut self, rhs: Self) {
        *self = ECToyScalar::mul(*self, rhs);
    }
}

impl MulAssign<&ECToyScalar> for ECToyScalar {
    fn mul_assign(&mut self, rhs: &ECToyScalar) {
        *self = ECToyScalar::mul(*self, *rhs);
    }
}

impl Neg for ECToyScalar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.is_zero_vartime() {
            Self::ZERO
        } else {
            ECToyScalar(ECToyScalar::MODULUS - self.0)
        }
    }
}

impl Sum for ECToyScalar {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or(Self::ZERO)
    }
}

impl<'a> Sum<&'a ECToyScalar> for ECToyScalar {
    fn sum<I: Iterator<Item = &'a ECToyScalar>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl Product for ECToyScalar {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Mul::mul).unwrap_or(Self::ONE)
    }
}

impl<'a> Product<&'a ECToyScalar> for ECToyScalar {
    fn product<I: Iterator<Item = &'a ECToyScalar>>(iter: I) -> Self {
        iter.copied().product()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct ECToyGroup {
    x: ECToyField,
    y: ECToyField,
}

impl ECToyGroup {
    pub const IDENTITY: Self = ECToyGroup {
        x: ECToyField::ZERO,
        y: ECToyField::ZERO,
    };
    pub const GENERATOR: Self = ECToyGroup {
        x: ECToyField::ONE,
        y: ECToyField(2),
    };

    const B: ECToyField = ECToyField(3);

    pub fn from_xy(x: ECToyField, y: ECToyField) -> CtOption<Self> {
        let point = ECToyGroup { x, y };
        CtOption::new(point, Choice::from(point.is_valid() as u8))
    }

    fn is_identity(&self) -> bool {
        self.x == ECToyField::ZERO
    }

    fn is_valid(&self) -> bool {
        if self.is_identity() {
            return self.y == ECToyField::ZERO;
        }

        self.y.square() == self.x.square() * self.x + Self::B
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: Self) -> Self {
        if self.is_identity() {
            return rhs;
        }
        if rhs.is_identity() {
            return self;
        }
        if self.x == rhs.x {
            if self.y == rhs.y {
                return self.double();
            }
            return Self::IDENTITY;
        }

        let lambda = (rhs.y - self.y) * (rhs.x - self.x).inv();
        let x = lambda.square() - self.x - rhs.x;
        let y = lambda * (self.x - x) - self.y;
        ECToyGroup { x, y }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Self {
        ECToyGroup {
            x: self.x,
            y: -self.y,
        }
    }

    pub fn double(&self) -> Self {
        if self.is_identity() || self.y == ECToyField::ZERO {
            return Self::IDENTITY;
        }

        let lambda = ECToyField(3) * self.x.square() * self.y.double().inv();
        let x = lambda.square() - self.x.double();
        let y = lambda * (self.x - x) - self.y;
        ECToyGroup { x, y }
    }

    pub fn scalar_mul_u64(&self, mut scalar: u64) -> Self {
        let mut result = Self::IDENTITY;
        let mut base = *self;

        while scalar > 0 {
            if scalar & 1 == 1 {
                result += base;
            }
            base = base.double();
            scalar >>= 1;
        }

        result
    }

    pub fn scalar_mul_i64(&self, scalar: i64) -> Self {
        let s = scalar.unsigned_abs();
        if scalar < 0 {
            self.scalar_mul_u64(s).neg()
        } else {
            self.scalar_mul_u64(s)
        }
    }
}

impl Add for ECToyGroup {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ECToyGroup::add(self, rhs)
    }
}

impl Add<&ECToyGroup> for ECToyGroup {
    type Output = ECToyGroup;

    fn add(self, rhs: &ECToyGroup) -> Self::Output {
        ECToyGroup::add(self, *rhs)
    }
}

impl AddAssign for ECToyGroup {
    fn add_assign(&mut self, rhs: Self) {
        *self = ECToyGroup::add(*self, rhs);
    }
}

impl AddAssign<&ECToyGroup> for ECToyGroup {
    fn add_assign(&mut self, rhs: &ECToyGroup) {
        *self = ECToyGroup::add(*self, *rhs);
    }
}

impl Sub for ECToyGroup {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ECToyGroup::sub(self, rhs)
    }
}

impl Sub<&ECToyGroup> for ECToyGroup {
    type Output = ECToyGroup;

    fn sub(self, rhs: &ECToyGroup) -> Self::Output {
        ECToyGroup::sub(self, *rhs)
    }
}

impl SubAssign for ECToyGroup {
    fn sub_assign(&mut self, rhs: Self) {
        *self = ECToyGroup::sub(*self, rhs);
    }
}

impl SubAssign<&ECToyGroup> for ECToyGroup {
    fn sub_assign(&mut self, rhs: &ECToyGroup) {
        *self = ECToyGroup::sub(*self, *rhs);
    }
}

impl Neg for ECToyGroup {
    type Output = Self;

    fn neg(self) -> Self::Output {
        ECToyGroup::neg(self)
    }
}

impl Mul<ECToyScalar> for ECToyGroup {
    type Output = Self;

    fn mul(self, rhs: ECToyScalar) -> Self::Output {
        self.scalar_mul_u64(rhs.0)
    }
}

impl Mul<&ECToyScalar> for ECToyGroup {
    type Output = Self;

    fn mul(self, rhs: &ECToyScalar) -> Self::Output {
        self * *rhs
    }
}

impl MulAssign<ECToyScalar> for ECToyGroup {
    fn mul_assign(&mut self, rhs: ECToyScalar) {
        *self = *self * rhs;
    }
}

impl MulAssign<&ECToyScalar> for ECToyGroup {
    fn mul_assign(&mut self, rhs: &ECToyScalar) {
        *self = *self * rhs;
    }
}

impl Sum for ECToyGroup {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or(Self::IDENTITY)
    }
}

impl<'a> Sum<&'a ECToyGroup> for ECToyGroup {
    fn sum<I: Iterator<Item = &'a ECToyGroup>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl Group for ECToyGroup {
    type Scalar = ECToyScalar;

    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Self, R::Error> {
        loop {
            let scalar = ECToyScalar::try_from_rng(rng)?;
            if !bool::from(scalar.is_zero()) {
                return Ok(Self::mul_by_generator(&scalar));
            }
        }
    }

    fn identity() -> Self {
        Self::IDENTITY
    }

    fn generator() -> Self {
        Self::GENERATOR
    }

    fn is_identity(&self) -> Choice {
        self.x.is_zero()
    }

    fn double(&self) -> Self {
        ECToyGroup::double(self)
    }

    fn mul_by_generator(scalar: &Self::Scalar) -> Self {
        ECToyGroup::scalar_mul_u64(&ECToyGroup::GENERATOR, scalar.0)
    }
}

impl Curve for ECToyGroup {
    type AffineRepr = Self;

    fn to_affine(&self) -> Self::AffineRepr {
        *self
    }
}

impl GroupEncoding for ECToyGroup {
    type Repr = [u8; 16];

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        let mut x_bytes = [0u8; 8];
        let mut y_bytes = [0u8; 8];
        x_bytes.copy_from_slice(&bytes[..8]);
        y_bytes.copy_from_slice(&bytes[8..]);

        let x = ECToyField::from_repr(x_bytes);
        let y = ECToyField::from_repr(y_bytes);
        let point = ECToyGroup {
            x: x.unwrap_or(ECToyField::ZERO),
            y: y.unwrap_or(ECToyField::ZERO),
        };
        let is_valid = bool::from(x.is_some()) && bool::from(y.is_some()) && point.is_valid();

        CtOption::new(point, Choice::from(is_valid as u8))
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        Self::from_bytes(bytes)
    }

    fn to_bytes(&self) -> Self::Repr {
        let mut repr = [0u8; 16];
        repr[..8].copy_from_slice(&self.x.to_repr());
        repr[8..].copy_from_slice(&self.y.to_repr());
        repr
    }
}

impl PrimeGroup for ECToyGroup {}

impl CofactorGroup for ECToyGroup {
    type Subgroup = Self;

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

impl PrimeCurve for ECToyGroup {
    type Affine = Self;
}

impl CofactorCurve for ECToyGroup {
    type Affine = Self;
}

impl PrimeCurveAffine for ECToyGroup {
    type Scalar = ECToyScalar;
    type Curve = Self;

    fn identity() -> Self {
        Group::identity()
    }

    fn generator() -> Self {
        Group::generator()
    }

    fn is_identity(&self) -> Choice {
        Group::is_identity(self)
    }

    fn to_curve(&self) -> Self::Curve {
        *self
    }
}

impl CofactorCurveAffine for ECToyGroup {
    type Scalar = ECToyScalar;
    type Curve = Self;

    fn identity() -> Self {
        Group::identity()
    }

    fn generator() -> Self {
        Group::generator()
    }

    fn is_identity(&self) -> Choice {
        Group::is_identity(self)
    }

    fn to_curve(&self) -> Self::Curve {
        *self
    }
}

impl KangarooGroup for ECToyGroup {
    fn is_distinguished(&self, bits: u32) -> bool {
        assert!(0 < bits && bits < 64);
        (self.x.0 & ((1 << bits) - 1)) == 0
    }

    fn is_negation_map_representative(&self) -> bool {
        self.y.is_even().into()
    }

    fn bucket(&self, bits: u32) -> usize {
        assert!(0 < bits && bits < 32);
        ((self.x.0 ^ (self.x.0 >> 32)).swap_bytes() >> (64 - bits)) as usize
    }

    fn footprint(&self) -> u64 {
        self.x.0
    }
}

#[cfg(test)]
mod tests {
    use group::GroupEncoding;

    use super::*;

    #[test]
    fn curve_parameters_match_claimed_order() {
        assert_eq!(ECToyField::MODULUS, (1u64 << 63) - 4321);
        assert_eq!(ECToyScalar::MODULUS, (1u64 << 63) - 0x5634b119);
        assert!(!bool::from(ECToyField(3).sqrt().is_some()));
        assert!(ECToyGroup::GENERATOR.is_valid());
        assert!(bool::from(Group::is_identity(
            &ECToyGroup::GENERATOR.scalar_mul_u64(ECToyScalar::MODULUS)
        )));
        assert!(!bool::from(Group::is_identity(
            &ECToyGroup::GENERATOR.scalar_mul_u64(ECToyScalar::MODULUS - 1)
        )));
    }

    #[test]
    fn group_law_smoke_checks() {
        let g = ECToyGroup::GENERATOR;
        let two_g = g.double();
        let three_g = two_g + g;

        assert_eq!(g + ECToyGroup::IDENTITY, g);
        assert_eq!(g + (-g), ECToyGroup::IDENTITY);
        assert_eq!(g + g, two_g);
        assert_eq!(g.scalar_mul_u64(3), three_g);
        assert_eq!(three_g - two_g, g);
        assert!(three_g.is_valid());
    }

    #[test]
    fn encoding_roundtrip() {
        let point = ECToyGroup::GENERATOR.scalar_mul_u64(1234567);
        let repr = point.to_bytes();
        let decoded = ECToyGroup::from_bytes(&repr).unwrap();

        assert_eq!(decoded, point);
        assert!(bool::from(ECToyGroup::from_bytes(&[0u8; 16]).is_some()));
    }
}
