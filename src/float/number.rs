use core::fmt::Display;
use core::ops::{Add, Div, Mul, Neg, Sub};

pub trait FloatNumber
where
    Self: Copy + Mul<Output=Self> + Add<Output=Self> + Sub<Output=Self> + Div<Output=Self> + Neg<Output=Self> + Display + PartialOrd,
{
    const MAX: Self;
    const MIN: Self;
    fn abs(self) -> Self;
    fn sqrt(self) -> Self;
    fn from_usize(value: usize) -> Self;
    fn from_i32(value: i32) -> Self;
    fn from_i64(value: i64) -> Self;
    fn from_float(value: f64) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn log2(self) -> Self;
    fn to_i32(self) -> i32;
    fn to_usize(self) -> usize;
    fn to_f64(self) -> f64;
    fn bit_width() -> u8;
}

impl FloatNumber for f32 {
    const MAX: Self = f32::MAX;
    const MIN: Self = f32::MIN;

    #[inline(always)]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline(always)]
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as f32
    }

    #[inline(always)]
    fn from_i32(value: i32) -> Self {
        value as f32
    }

    #[inline(always)]
    fn from_i64(value: i64) -> Self {
        value as f32
    }

    #[inline(always)]
    fn from_float(value: f64) -> Self {
        value as f32
    }

    #[inline(always)]
    fn max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    #[inline(always)]
    fn log2(self) -> Self {
        libm::log2f(self)
    }

    #[inline(always)]
    fn to_i32(self) -> i32 {
        (self + 0.5_f32.copysign(self)) as i32
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        (self + 0.5) as usize
    }

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn bit_width() -> u8 {
        32
    }
}

impl FloatNumber for f64 {
    const MAX: Self = f64::MAX;
    const MIN: Self = f64::MIN;
    
    #[inline(always)]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline(always)]
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as f64
    }

    #[inline(always)]
    fn from_i32(value: i32) -> Self {
        value as f64
    }

    #[inline(always)]
    fn from_i64(value: i64) -> Self {
        value as f64
    }

    #[inline(always)]
    fn from_float(value: f64) -> Self {
        value
    }

    #[inline(always)]
    fn max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    #[inline(always)]
    fn log2(self) -> Self {
        libm::log2(self)
    }

    #[inline(always)]
    fn to_i32(self) -> i32 {
        (self + 0.5_f64.copysign(self)) as i32
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        (self + 0.5) as usize
    }

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self
    }

    #[inline(always)]
    fn bit_width() -> u8 {
        64
    }
}