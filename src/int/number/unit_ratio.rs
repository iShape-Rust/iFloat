use crate::float::number::FloatNumber;
use crate::int::number::fixed_scale::FixedScale;
use crate::int::number::int::IntNumber;
use crate::int::point::IntPoint;
use crate::int::vector::IntVector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnitRatio<I: IntNumber> {
    value: I,
}

impl<I: IntNumber> UnitRatio<I> {
    pub const SCALE: u32 = FixedScale::<I>::SHIFT;
    pub const DENOMINATOR: I::Wide = FixedScale::<I>::DENOMINATOR;

    #[inline(always)]
    pub fn value(&self) -> I::Wide {
        self.value.to_wide()
    }

    #[inline(always)]
    pub fn new(value: I) -> Self {
        debug_assert!(value.to_wide() <= Self::DENOMINATOR);
        debug_assert!(value >= I::ZERO);
        Self { value }
    }

    #[inline(always)]
    pub fn from_float<F: FloatNumber>(value: F) -> Self {
        debug_assert!(value >= F::ZERO);
        debug_assert!(value <= F::ONE);

        let denominator = F::from_wide_int(Self::DENOMINATOR);
        Self::new(I::from_rounded_float(value * denominator))
    }

    #[inline(always)]
    pub fn from_int(numerator: I, denominator: I) -> Self {
        debug_assert!(numerator >= I::ZERO);
        debug_assert!(numerator <= denominator);
        debug_assert!(denominator > I::ZERO);

        let value = FixedScale::<I>::div_to_scaled_round(numerator.to_wide(), denominator.to_wide());
        Self::new(I::from_wide(value))
    }

    #[inline(always)]
    pub fn half() -> Self {
        Self::new(I::from_wide(FixedScale::<I>::half()))
    }

    #[inline(always)]
    pub fn mid(self, other: Self) -> Self {
        let value = (self.value() + other.value()) >> 1;
        Self::new(I::from_wide(value))
    }

    #[inline(always)]
    pub fn scale(self, scalar: I) -> I {
        FixedScale::<I>::to_int_round(self.scale_to_scaled_wide(scalar))
    }

    #[inline(always)]
    pub fn scale_to_scaled_wide(self, scalar: I) -> I::Wide {
        let s = self.value();
        s * scalar.to_wide()
    }

    #[inline(always)]
    pub fn scale_wide(self, scalar: I::Wide) -> I {
        let s = self.value();
        FixedScale::<I>::to_int_round(s * scalar)
    }

    #[inline(always)]
    pub fn scale_point(self, point: IntPoint<I>) -> IntPoint<I> {
        let x = self.scale(point.x);
        let y = self.scale(point.y);
        IntPoint { x, y }
    }

    #[inline(always)]
    pub fn scale_point_to_scaled_vector(self, point: IntPoint<I>) -> IntVector<I> {
        let x = self.scale_to_scaled_wide(point.x);
        let y = self.scale_to_scaled_wide(point.y);
        IntVector { x, y }
    }

    #[inline(always)]
    pub fn scale_vector_to_point(self, vector: IntVector<I>) -> IntPoint<I> {
        let x = self.scale_wide(vector.x);
        let y = self.scale_wide(vector.y);
        IntPoint { x, y }
    }
}

impl<I: IntNumber> IntPoint<I> {
    #[inline(always)]
    pub fn to_wide(self) -> IntVector<I> {
        IntVector {
            x: self.x.to_wide(),
            y: self.y.to_wide(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UnitRatio;

    #[test]
    fn mid_uses_direct_average() {
        let a = UnitRatio::<i32>::new(2);
        let b = UnitRatio::<i32>::new(5);

        assert_eq!(a.mid(b).value, 3);
    }

    #[test]
    fn mid_preserves_upper_boundary() {
        let one_i16 = UnitRatio::<i16>::new(UnitRatio::<i16>::DENOMINATOR as i16);
        let one_i32 = UnitRatio::<i32>::new(UnitRatio::<i32>::DENOMINATOR as i32);
        let one_i64 = UnitRatio::<i64>::new(UnitRatio::<i64>::DENOMINATOR as i64);

        assert_eq!(one_i16.mid(one_i16), one_i16);
        assert_eq!(one_i32.mid(one_i32), one_i32);
        assert_eq!(one_i64.mid(one_i64), one_i64);
    }

    #[test]
    fn from_float_scales_unit_value() {
        let t = UnitRatio::<i32>::from_float(0.5_f32);

        assert_eq!(t.value(), UnitRatio::<i32>::DENOMINATOR >> 1);
    }

    #[test]
    fn from_int_scales_fraction() {
        let t = UnitRatio::<i32>::from_int(1, 3);
        let expected = ((UnitRatio::<i32>::DENOMINATOR + 1) / 3) as i32;

        assert_eq!(t.value, expected);
    }

    #[test]
    fn scale_rounds_half_away_from_zero() {
        let half = (UnitRatio::<i32>::DENOMINATOR >> 1) as i32;
        let t = UnitRatio::<i32>::new(half);

        assert_eq!(t.scale(1), 1);
        assert_eq!(t.scale(-1), -1);
        assert_eq!(t.scale(3), 2);
        assert_eq!(t.scale(-3), -2);
    }

    #[test]
    fn scale_wide_rounds_half_away_from_zero() {
        let half = (UnitRatio::<i32>::DENOMINATOR >> 1) as i32;
        let t = UnitRatio::<i32>::new(half);

        assert_eq!(t.scale_wide(1), 1);
        assert_eq!(t.scale_wide(-1), -1);
    }

    #[test]
    fn scale_to_scaled_wide_keeps_fixed_scale() {
        let t = UnitRatio::<i32>::half();

        assert_eq!(
            t.scale_to_scaled_wide(3),
            3 * (UnitRatio::<i32>::DENOMINATOR >> 1)
        );
    }
}
