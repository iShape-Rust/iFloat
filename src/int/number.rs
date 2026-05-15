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
}

impl WideIntNumber for i32 {
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
}
impl WideIntNumber for i64 {
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
}
impl WideIntNumber for i128 {
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
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
        + PartialOrd,
{
    type Wide: WideIntNumber;
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const WIDE_ZERO: Self::Wide;
    fn wide(self) -> Self::Wide;
    fn from_usize(value: usize) -> Self;
}

impl IntNumber for i16 {
    type Wide = i32;
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
}

impl IntNumber for i32 {
    type Wide = i64;
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
}

impl IntNumber for i64 {
    type Wide = i128;
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
}
