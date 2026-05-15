use core::fmt::{Binary, Display};
use core::ops::{Add, Div, Mul, Neg, Sub};

pub trait WideIntNumber:
    Copy
    + Ord
    + Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn signum(self) -> Self;
    fn from_f64(value: f64) -> Self;
}

impl WideIntNumber for i32 {
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        value as Self
    }
}
impl WideIntNumber for i64 {
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        value as Self
    }
}
impl WideIntNumber for i128 {
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        value as Self
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
    const WIDE_ZERO: Self::Wide;
    fn wide(self) -> Self::Wide;
    fn from_usize(value: usize) -> Self;
    fn from_f64(value: f64) -> Self;
    fn to_f64(self) -> f64;
}

impl IntNumber for i16 {
    type Wide = i32;
    const BITS: u32 = i16::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const WIDE_ZERO: Self::Wide = 0;

    #[inline(always)]
    fn wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        (value + 0.5_f64.copysign(value)) as Self
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
    const WIDE_ZERO: Self::Wide = 0;
    #[inline(always)]
    fn wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        (value + 0.5_f64.copysign(value)) as Self
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
    const WIDE_ZERO: Self::Wide = 0;
    #[inline(always)]
    fn wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Self {
        (value + 0.5_f64.copysign(value)) as Self
    }

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
}
