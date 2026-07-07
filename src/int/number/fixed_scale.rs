use crate::int::number::int::IntNumber;
use crate::int::number::product_uint::UIntProduct;
use crate::int::number::uint::UIntNumber;
use crate::int::number::wide_int::WideIntNumber;
use core::marker::PhantomData;

pub struct FixedScale<I: IntNumber>(PhantomData<I>);

impl<I: IntNumber> FixedScale<I> {
    pub const SHIFT: u32 = I::MAX_POWER_OF_TWO;
    pub const DENOMINATOR: I::Wide = I::MAX_POSITIVE_POWER_OF_TWO;

    #[inline(always)]
    pub fn half() -> I::Wide {
        Self::DENOMINATOR >> 1
    }

    #[inline(always)]
    pub fn to_int_round(scaled: I::Wide) -> I {
        I::from_wide(Self::div_round(scaled, Self::DENOMINATOR))
    }

    #[inline(always)]
    pub fn div_to_scaled_round(numerator: I::Wide, denominator: I::Wide) -> I::Wide {
        debug_assert!(denominator != I::Wide::ZERO);

        let negative = (numerator < I::Wide::ZERO) != (denominator < I::Wide::ZERO);
        let product = <I::WideUInt as UIntNumber>::Product::multiply(
            numerator.unsigned_abs(),
            Self::DENOMINATOR.to_uint(),
        );
        let quotient = product.divide_with_rounding(denominator.unsigned_abs());

        Self::from_unsigned_abs(quotient, negative)
    }

    #[inline(always)]
    pub fn div_round(numerator: I::Wide, denominator: I::Wide) -> I::Wide {
        debug_assert!(denominator != I::Wide::ZERO);

        let negative = (numerator < I::Wide::ZERO) != (denominator < I::Wide::ZERO);
        let product = <I::WideUInt as UIntNumber>::Product::from_uint(numerator.unsigned_abs());
        let quotient = product.divide_with_rounding(denominator.unsigned_abs());

        Self::from_unsigned_abs(quotient, negative)
    }

    #[inline(always)]
    fn from_unsigned_abs(value: I::WideUInt, negative: bool) -> I::Wide {
        debug_assert!(value <= I::WideUInt::LAST_BIT);

        if negative {
            if value == I::WideUInt::LAST_BIT {
                I::Wide::MIN
            } else {
                -I::Wide::from_uint(value)
            }
        } else {
            debug_assert!(value < I::WideUInt::LAST_BIT);
            I::Wide::from_uint(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FixedScale;

    #[test]
    fn to_int_round_rounds_scaled_half_away_from_zero() {
        assert_eq!(FixedScale::<i32>::to_int_round(FixedScale::<i32>::half()), 1);
        assert_eq!(FixedScale::<i32>::to_int_round(-FixedScale::<i32>::half()), -1);
        assert_eq!(
            FixedScale::<i32>::to_int_round(FixedScale::<i32>::DENOMINATOR + FixedScale::<i32>::half()),
            2
        );
    }

    #[test]
    fn div_round_rounds_half_away_from_zero() {
        assert_eq!(FixedScale::<i32>::div_round(3, 2), 2);
        assert_eq!(FixedScale::<i32>::div_round(-3, 2), -2);
        assert_eq!(FixedScale::<i32>::div_round(3, -2), -2);
        assert_eq!(FixedScale::<i32>::div_round(-3, -2), 2);
    }

    #[test]
    fn div_round_preserves_min_negative_magnitude() {
        assert_eq!(FixedScale::<i16>::div_round(i32::MIN, 1), i32::MIN);
    }

    #[test]
    fn div_to_scaled_round_returns_scaled_ratio() {
        assert_eq!(
            FixedScale::<i32>::div_to_scaled_round(1, 2),
            FixedScale::<i32>::half()
        );
        assert_eq!(
            FixedScale::<i32>::div_to_scaled_round(-1, 2),
            -FixedScale::<i32>::half()
        );
    }
}
