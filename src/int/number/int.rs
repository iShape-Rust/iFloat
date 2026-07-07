use crate::float::number::FloatNumber;
use crate::int::number::uint::UIntNumber;
use crate::int::number::wide_int::WideIntNumber;
use core::fmt::{Binary, Display};
use core::ops::{Add, AddAssign, Div, Mul, Neg, Shl, Shr, Sub};
use crate::int::number::fixed_scale::FixedScale;

pub trait IntNumber
where
    Self: Copy
        + Mul<Output = Self>
        + Add<Output = Self>
        + Sub<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + AddAssign
        + Shl<u32, Output = Self>
        + Shr<u32, Output = Self>
        + Binary
        + Display
        + Ord
        + Send
        + Sync
        + Default,
{
    type WideUInt: UIntNumber;
    type Wide: WideIntNumber<UInt = Self::WideUInt>;
    const BITS: u32;
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const FOUR: Self;
    const HALF_POWER_OF_TWO: u32;
    const MAX_POWER_OF_TWO: u32;
    const MAX_POSITIVE_POWER_OF_TWO: Self::Wide;
    fn from_wide(value: Self::Wide) -> Self;
    fn from_uint(value: Self::WideUInt) -> Self;
    fn from_u32(value: u32) -> Self;
    fn from_usize(value: usize) -> Self;
    fn from_float<F: FloatNumber>(value: F) -> Self;
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn ilog2(self) -> u32;
    #[inline(always)]
    fn scaled_isqrt(self) -> Self::Wide {
        self.to_scaled_wide().isqrt() << Self::HALF_POWER_OF_TWO
    }
    fn to_usize(self) -> usize;
    fn to_f32(self) -> f32;
    fn to_f64(self) -> f64;
    fn to_wide(self) -> Self::Wide;
    fn to_uint(self) -> Self::WideUInt;
    #[inline(always)]
    fn to_scaled_wide(self) -> Self::Wide {
        self.to_wide() << FixedScale::<Self>::SHIFT
    }
}

impl IntNumber for i16 {
    type WideUInt = u32;
    type Wide = i32;
    const BITS: u32 = i16::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;
    const HALF_POWER_OF_TWO: u32 = (Self::BITS - 2) >> 1;
    const MAX_POWER_OF_TWO: u32 = Self::HALF_POWER_OF_TWO << 1;
    const MAX_POSITIVE_POWER_OF_TWO: Self::Wide = 1 << Self::MAX_POWER_OF_TWO;
    #[inline(always)]
    fn from_wide(value: Self::Wide) -> Self {
        value as Self
    }
    #[inline(always)]
    fn from_uint(value: Self::WideUInt) -> Self {
        value as Self
    }
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
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
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self {
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
    fn ilog2(self) -> u32 {
        self.ilog2()
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
    #[inline(always)]
    fn to_wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn to_uint(self) -> Self::WideUInt {
        self as Self::WideUInt
    }
}

impl IntNumber for i32 {
    type WideUInt = u64;
    type Wide = i64;
    const BITS: u32 = i32::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;
    const HALF_POWER_OF_TWO: u32 = (Self::BITS - 2) >> 1;
    const MAX_POWER_OF_TWO: u32 = Self::HALF_POWER_OF_TWO << 1;
    const MAX_POSITIVE_POWER_OF_TWO: Self::Wide = 1 << Self::MAX_POWER_OF_TWO;
    #[inline(always)]
    fn from_wide(value: Self::Wide) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_uint(value: Self::WideUInt) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
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
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self {
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
    fn ilog2(self) -> u32 {
        self.ilog2()
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

    #[inline(always)]
    fn to_wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn to_uint(self) -> Self::WideUInt {
        self as Self::WideUInt
    }
}

impl IntNumber for i64 {
    type WideUInt = u128;
    type Wide = i128;
    const BITS: u32 = i64::BITS;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const TWO: Self = 2;
    const FOUR: Self = 4;
    const HALF_POWER_OF_TWO: u32 = (Self::BITS - 2) >> 1;
    const MAX_POWER_OF_TWO: u32 = Self::HALF_POWER_OF_TWO << 1;
    const MAX_POSITIVE_POWER_OF_TWO: Self::Wide = 1 << Self::MAX_POWER_OF_TWO;
    #[inline(always)]
    fn from_wide(value: Self::Wide) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_uint(value: Self::WideUInt) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
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
    fn from_rounded_float<F: FloatNumber>(value: F) -> Self {
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
    fn ilog2(self) -> u32 {
        self.ilog2()
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
    #[inline(always)]
    fn to_wide(self) -> Self::Wide {
        self as Self::Wide
    }

    #[inline(always)]
    fn to_uint(self) -> Self::WideUInt {
        self as Self::WideUInt
    }
}

#[cfg(test)]
mod tests {
    use super::IntNumber;
    use crate::int::number::wide_int::WideIntNumber;

    fn assert_scaled_isqrt<T>(values: &[T])
    where
        T: IntNumber,
    {
        let step = T::Wide::ONE << T::HALF_POWER_OF_TWO;
        let mask = step - T::Wide::ONE;

        for &value in values {
            let root = value.scaled_isqrt();
            let q = root >> T::HALF_POWER_OF_TWO;
            let scaled_value = value.to_scaled_wide();

            assert!(root & mask == T::Wide::ZERO);
            assert!(q * q <= scaled_value);
            assert!((q + T::Wide::ONE) * (q + T::Wide::ONE) > scaled_value);
            assert!(root == q << T::HALF_POWER_OF_TWO);
        }
    }

    #[test]
    fn scaled_isqrt_matches_fixed_point_scale_for_i16() {
        assert_scaled_isqrt::<i16>(&[0, 1, 2, 3, 4, 5, 10, 255, i16::MAX]);

        let scale = <i16 as IntNumber>::MAX_POSITIVE_POWER_OF_TWO;
        assert_eq!(0i16.scaled_isqrt(), 0);
        assert_eq!(1i16.scaled_isqrt(), scale);
        assert_eq!(4i16.scaled_isqrt(), 2 * scale);
        assert_eq!(9i16.scaled_isqrt(), 3 * scale);
    }

    #[test]
    fn scaled_isqrt_matches_fixed_point_scale_for_i32() {
        assert_scaled_isqrt::<i32>(&[0, 1, 2, 3, 4, 5, 10, 65_535, i32::MAX]);

        let scale = <i32 as IntNumber>::MAX_POSITIVE_POWER_OF_TWO;
        assert_eq!(0i32.scaled_isqrt(), 0);
        assert_eq!(1i32.scaled_isqrt(), scale);
        assert_eq!(4i32.scaled_isqrt(), 2 * scale);
        assert_eq!(9i32.scaled_isqrt(), 3 * scale);
    }

    #[test]
    fn scaled_isqrt_matches_fixed_point_scale_for_i64() {
        assert_scaled_isqrt::<i64>(&[0, 1, 2, 3, 4, 5, 10, 4_294_967_295, i64::MAX]);

        let scale = <i64 as IntNumber>::MAX_POSITIVE_POWER_OF_TWO;
        assert_eq!(0i64.scaled_isqrt(), 0);
        assert_eq!(1i64.scaled_isqrt(), scale);
        assert_eq!(4i64.scaled_isqrt(), 2 * scale);
        assert_eq!(9i64.scaled_isqrt(), 3 * scale);
    }

    #[test]
    #[should_panic]
    fn scaled_isqrt_panics_for_negative_values() {
        (-1i32).scaled_isqrt();
    }
}
