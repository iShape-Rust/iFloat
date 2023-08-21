use std::ops;
use std::cmp;

#[derive(Debug, Clone, Copy)]
pub struct FixFloat(i64);

impl FixFloat {
    pub const FRACTION_BITS: i64 = 10;
    pub const SQR_FRACTION_BITS: i64 = 20;
    pub const CUBE_FRACTION_BITS: i64 = 30;
    pub const TETRA_FRACTION_BITS: i64 = 40;
    pub const PENTA_FRACTION_BITS: i64 = 50;
    pub const MAX_BITS: i64 = (64 >> 1) - 1;
    pub const MAX_FIX: i64 = (1 << Self::MAX_BITS) - 1;
    pub const MIN_FIX: i64 = -Self::MAX_FIX;
    pub const ZERO: FixFloat = FixFloat(0);
    pub const UNIT: i64 = 1 << Self::FRACTION_BITS;
    pub const SQR_UNIT: i64 = 1 << Self::SQR_FRACTION_BITS;
    pub const CUBE_UNIT: i64 = 1 << Self::CUBE_FRACTION_BITS;
    pub const HALF: i64 = 1 << (Self::FRACTION_BITS - 1);
    pub const PI: i64 = 3217;

    pub fn value(&self) -> i64 {
        self.0
    }

    pub fn new_number(value: i64) -> Self {
        FixFloat(value << Self::FRACTION_BITS)
    }

    pub fn new_i64(value: i64) -> Self {
        FixFloat(value)
    }

    pub fn div(&self, value: FixFloat) -> FixFloat {
        FixFloat((self.0 << Self::FRACTION_BITS) / value.0)
    }

    pub fn mul(&self, value: FixFloat) -> FixFloat {
        FixFloat((self.0 * value.0) / Self::UNIT)
    }

    pub fn sqr(&self) -> FixFloat {
        FixFloat((self.0 * self.0) >> Self::FRACTION_BITS)
    }

    pub fn sqrt(&self) -> FixFloat {
        FixFloat(fast_square_root(self.0 << Self::FRACTION_BITS))
    }

    pub fn double(&self) -> f64 {
        self.0 as f64 / Self::UNIT as f64
    }

    pub fn float(&self) -> f32 {
        self.0 as f32 / Self::UNIT as f32
    }

    pub fn int(&self) -> FixFloat {
        FixFloat(self.0 >> Self::FRACTION_BITS)
    }

    pub fn clamp(&self, min: FixFloat, max: FixFloat) -> FixFloat {
        FixFloat(std::cmp::min(max.0, std::cmp::max(min.0, self.0)))
    }
    pub fn invert(&self) -> FixFloat {
        FixFloat(Self::SQR_UNIT / self.0)
    }
}

pub fn normalize(value: i64) -> FixFloat {
    FixFloat(value / FixFloat::UNIT)
}
pub fn fast_normalize(value: i64) -> FixFloat {
    FixFloat(value >> FixFloat::FRACTION_BITS)
}
pub fn sqr_normalize(value: i64) -> FixFloat {
    FixFloat(value / FixFloat::SQR_UNIT)
}

pub fn fix(value: f64) -> FixFloat {
    FixFloat((value * FixFloat::UNIT as f64) as i64)
}

pub fn fast_square_root(value: i64) -> i64 {
    let a = (value as f64).sqrt() as i64;
    let b = a + 1;

    if b * b > value { a } else { b }
}

impl ops::Neg for FixFloat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        FixFloat(-self.0)
    }
}

impl ops::Add for FixFloat {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        FixFloat(self.0 + other.0)
    }
}

impl ops::Sub for FixFloat {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        FixFloat(self.0 - other.0)
    }
}

impl ops::Mul for FixFloat {

    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        normalize(self.0 * rhs.0)
    }
}

impl ops::Div for FixFloat {

    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        normalize(self.0 * rhs.0)
    }
}

impl cmp::PartialOrd for FixFloat {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FixFloat {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq for FixFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FixFloat {}

