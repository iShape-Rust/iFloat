use crate::float::number::FloatNumber;
use core::fmt::{Binary, Display};
use core::ops::{Add, BitAnd, Div, Mul, Neg, Sub};

pub trait WideIntNumber:
    Copy
    + Ord
    + Display
    + Add<Output = Self>
    + BitAnd<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const FOUR: Self;
    fn from_usize(value: usize) -> Self;
    fn from_float_round<F: FloatNumber>(value: F) -> Self;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn signum(self) -> Self;
    fn to_usize(self) -> usize;
    fn to_f32(self) -> f32;
    fn to_f64(self) -> f64;
}

impl WideIntNumber for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_float_round<F: FloatNumber>(value: F) -> Self {
        value.to_round_i32()
    }

    #[inline(always)]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }

    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}
impl WideIntNumber for i64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_float_round<F: FloatNumber>(value: F) -> Self {
        value.to_round_i64()
    }

    #[inline(always)]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }

    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}
impl WideIntNumber for i128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_float_round<F: FloatNumber>(value: F) -> Self {
        value.to_round_i128()
    }

    #[inline(always)]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }

    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}
pub trait IntNumber
where
    Self: Copy
        + Mul<Output = Self>
        + Add<Output = Self>
        + Sub<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + Binary
        + Display
        + Ord,
{
    type Wide: WideIntNumber;
    const BITS: u32;
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const ONE: Self;
    const WIDE_ZERO: Self::Wide;
    const WIDE_ONE: Self::Wide;
    const WIDE_TWO: Self::Wide;

    fn wide(self) -> Self::Wide;
    fn from_usize(value: usize) -> Self;
    fn from_float<F: FloatNumber>(value: F) -> Self;
    fn from_float_round<F: FloatNumber>(value: F) -> Self;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn to_usize(self) -> usize;
    fn to_f32(self) -> f32;
    fn to_f64(self) -> f64;
}

impl IntNumber for i16 {
    type Wide = i32;
    const BITS: u32 = i16::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const WIDE_ZERO: Self::Wide = 0;
    const WIDE_ONE: Self::Wide = 1;
    const WIDE_TWO: Self::Wide = 2;

    #[inline(always)]
    fn wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }
    #[inline(always)]
    fn from_float<F: FloatNumber>(value: F) -> Self {
        value.to_i16()
    }

    #[inline(always)]
    fn from_float_round<F: FloatNumber>(value: F) -> Self {
        value.to_round_i16()
    }

    #[inline(always)]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl IntNumber for i32 {
    type Wide = i64;
    const BITS: u32 = i32::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const WIDE_ZERO: Self::Wide = 0;
    const WIDE_ONE: Self::Wide = 1;
    const WIDE_TWO: Self::Wide = 2;
    #[inline(always)]
    fn wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_float<F: FloatNumber>(value: F) -> Self {
        value.to_i32()
    }

    #[inline(always)]
    fn from_float_round<F: FloatNumber>(value: F) -> Self {
        value.to_round_i32()
    }

    #[inline(always)]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl IntNumber for i64 {
    type Wide = i128;
    const BITS: u32 = i64::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const WIDE_ZERO: Self::Wide = 0;
    const WIDE_ONE: Self::Wide = 1;
    const WIDE_TWO: Self::Wide = 2;
    #[inline(always)]
    fn wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_float<F: FloatNumber>(value: F) -> Self {
        value.to_i64()
    }

    #[inline(always)]
    fn from_float_round<F: FloatNumber>(value: F) -> Self {
        value.to_round_i64()
    }

    #[inline(always)]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}
