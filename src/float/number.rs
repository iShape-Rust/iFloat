use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub trait FloatNumber
where
    Self: Copy + Mul<Output=Self> + Add<Output=Self> + Sub<Output=Self> + Div<Output=Self> + Neg<Output=Self> + Display + PartialOrd,
{
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
    #[inline(always)]
    fn sqrt(self) -> Self {
        self.sqrt()
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
        self.log2()
    }

    #[inline(always)]
    fn to_i32(self) -> i32 {
        self.round() as i32
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self.round() as usize
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
    #[inline(always)]
    fn sqrt(self) -> Self {
        self.sqrt()
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
        self.log2()
    }

    #[inline(always)]
    fn to_i32(self) -> i32 {
        self.round() as i32
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self.round() as usize
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