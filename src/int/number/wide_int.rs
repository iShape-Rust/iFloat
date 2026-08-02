use crate::float::number::FloatNumber;
use crate::int::number::fixed_scale::FixedScale;
use crate::int::number::int::IntNumber;
use crate::int::number::uint::UIntNumber;
use core::fmt::Display;
use core::ops::{Add, BitAnd, Div, Mul, Neg, Shl, Shr, Sub};

pub trait WideIntNumber:
    Copy
    + Ord
    + Send
    + Sync
    + Display
    + Add<Output = Self>
    + BitAnd<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
{
    type UInt: UIntNumber;
    type Narrow: IntNumber;
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const FOUR: Self;
    fn from_u32(value: u32) -> Self;
    fn from_usize(value: usize) -> Self;
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self;
    fn from_uint(value: Self::UInt) -> Self;
    fn to_uint(self) -> Self::UInt;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn unsigned_abs(self) -> Self::UInt;
    fn signum(self) -> Self;
    fn ilog2(self) -> u32;
    fn isqrt(self) -> Self;
    fn to_usize(self) -> usize;
    fn to_f32(self) -> f32;
    fn to_f64(self) -> f64;

    #[inline(always)]
    fn shr_round(self, shift: u32) -> Self {
        if shift == 0 {
            return self;
        }

        debug_assert!(shift < Self::UInt::BITS);

        let negative = self < Self::ZERO;
        let half = Self::UInt::ONE << (shift - 1);
        let rounded_abs = (self.unsigned_abs() + half) >> shift;

        if negative {
            if rounded_abs == Self::UInt::LAST_BIT {
                Self::MIN
            } else {
                -Self::from_uint(rounded_abs)
            }
        } else {
            Self::from_uint(rounded_abs)
        }
    }

    #[inline(always)]
    fn shr_round_positive(self, shift: u32) -> Self {
        debug_assert!(self >= Self::ZERO);

        if shift == 0 {
            return self;
        }

        debug_assert!(shift < Self::UInt::BITS);

        let half = Self::UInt::ONE << (shift - 1);
        Self::from_uint((self.to_uint() + half) >> shift)
    }

    #[inline(always)]
    fn to_scaled(self) -> Self {
        self << FixedScale::<Self::Narrow>::SHIFT
    }
}

impl WideIntNumber for i32 {
    type UInt = u32;
    type Narrow = i16;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self {
        value.to_round_i32()
    }
    #[inline(always)]
    fn from_uint(value: Self::UInt) -> Self {
        value as Self
    }

    #[inline(always)]
    fn to_uint(self) -> Self::UInt {
        self as Self::UInt
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
    fn unsigned_abs(self) -> Self::UInt {
        self.unsigned_abs()
    }

    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
    #[inline(always)]
    fn ilog2(self) -> u32 {
        self.ilog2()
    }
    #[inline(always)]
    fn isqrt(self) -> Self {
        self.isqrt()
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
    type UInt = u64;
    type Narrow = i32;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self {
        value.to_round_i64()
    }

    #[inline(always)]
    fn from_uint(value: Self::UInt) -> Self {
        value as Self
    }

    #[inline(always)]
    fn to_uint(self) -> Self::UInt {
        self as Self::UInt
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
    fn unsigned_abs(self) -> Self::UInt {
        self.unsigned_abs()
    }
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
    #[inline(always)]
    fn ilog2(self) -> u32 {
        self.ilog2()
    }
    #[inline(always)]
    fn isqrt(self) -> Self {
        self.isqrt()
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
    type UInt = u128;
    type Narrow = i64;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self {
        value.to_round_i128()
    }

    #[inline(always)]
    fn from_uint(value: Self::UInt) -> Self {
        value as Self
    }

    #[inline(always)]
    fn to_uint(self) -> Self::UInt {
        self as Self::UInt
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
    fn unsigned_abs(self) -> Self::UInt {
        self.unsigned_abs()
    }
    #[inline(always)]
    fn signum(self) -> Self {
        self.signum()
    }
    #[inline(always)]
    fn ilog2(self) -> u32 {
        self.ilog2()
    }
    #[inline(always)]
    fn isqrt(self) -> Self {
        self.isqrt()
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

#[cfg(test)]
mod tests {
    use super::WideIntNumber;

    #[test]
    fn shr_round_positive_rounds_to_nearest() {
        assert_eq!(100i64.shr_round_positive(4), 6);
        assert_eq!(104i64.shr_round_positive(4), 7);
        assert_eq!(111i64.shr_round_positive(4), 7);
        assert_eq!(112i64.shr_round_positive(4), 7);
    }

    #[test]
    fn shr_round_matches_half_away_from_zero() {
        assert_eq!(100i64.shr_round(4), 6);
        assert_eq!(104i64.shr_round(4), 7);
        assert_eq!((-100i64).shr_round(4), -6);
        assert_eq!((-104i64).shr_round(4), -7);
    }

    #[test]
    fn shr_round_with_zero_shift_returns_input() {
        assert_eq!(123i64.shr_round(0), 123);
        assert_eq!((-123i64).shr_round(0), -123);
        assert_eq!(123i64.shr_round_positive(0), 123);
    }

    #[test]
    fn shr_round_handles_min_value_abs() {
        assert_eq!(i64::MIN.shr_round(1), -(1i64 << 62));
    }
}
