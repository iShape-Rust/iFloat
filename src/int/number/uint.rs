use crate::int::number::ext_uint::{CompositeExtUIntNumber, ExtUIntNumber, ExtUIntNumber64};
use core::fmt::Display;
use core::ops::{Add, AddAssign, BitAnd, BitOr, Div, Mul, Shl, Shr, Sub};

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
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + AddAssign
{
    type ExtUInt: ExtUIntNumber<Self>;
    const BITS: u32;
    const HALF_BITS: u32;
    const HALF_MASK: Self;
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;

    fn overflowing_add(self, rhs: Self) -> (Self, bool);
    fn leading_zeros(self) -> u32;
    fn to_u64(self) -> u64;
}

impl UIntNumber for u32 {
    type ExtUInt = ExtUIntNumber64;
    const BITS: u32 = 32;
    const HALF_BITS: u32 = 16;
    const HALF_MASK: Self = 0xFFFF;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
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
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
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
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
}
