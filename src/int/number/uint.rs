use crate::int::number::ext_uint::{CompositeExtUIntNumber, ExtUIntNumber, ExtUIntNumber64};
use core::fmt::Display;
use core::ops::{Add, AddAssign, BitAnd, BitOr, BitOrAssign, Div, Mul, Shl, ShlAssign, Shr, Sub};

pub trait UIntNumber:
    Copy
    + Clone
    + Ord
    + Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitOrAssign
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + ShlAssign
    + AddAssign
{
    type ExtUInt: ExtUIntNumber<Self>;
    const BITS: u32;
    const HALF_BITS: u32;
    const HALF_MASK: Self;
    const LAST_BIT_INDEX: u32;
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    const LAST_BIT: Self;

    fn overflowing_add(self, rhs: Self) -> (Self, bool);
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn leading_zeros(self) -> u32;
    fn from_u64(value: u64) -> Self;
    fn to_u64(self) -> u64;
}

impl UIntNumber for u32 {
    type ExtUInt = ExtUIntNumber64;
    const BITS: u32 = 32;
    const HALF_BITS: u32 = 16;
    const HALF_MASK: Self = 0xFFFF;
    const LAST_BIT_INDEX: u32 = 31;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    const LAST_BIT: Self = Self::ONE << Self::LAST_BIT_INDEX;

    #[inline(always)]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Self {
        value as u32
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
}

impl UIntNumber for u64 {
    type ExtUInt = CompositeExtUIntNumber<Self>;
    const BITS: u32 = 64;
    const HALF_BITS: u32 = 32;
    const HALF_MASK: Self = 0xFFFF_FFFF;
    const LAST_BIT_INDEX: u32 = 63;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    const LAST_BIT: Self = Self::ONE << Self::LAST_BIT_INDEX;

    #[inline(always)]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Self {
        value
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self
    }
}

impl UIntNumber for u128 {
    type ExtUInt = CompositeExtUIntNumber<Self>;
    const BITS: u32 = 128;
    const HALF_BITS: u32 = 64;
    const HALF_MASK: Self = 0xFFFF_FFFF_FFFF_FFFF;
    const LAST_BIT_INDEX: u32 = 127;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    const LAST_BIT: Self = Self::ONE << Self::LAST_BIT_INDEX;
    #[inline(always)]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    #[inline(always)]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Self {
        value as u128
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
}
