use crate::int::number::int::IntNumber;
use crate::int::point::IntPoint;
use crate::int::vector::IntVector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnitRatio<I: IntNumber> {
    value: I,
}

impl<I: IntNumber> UnitRatio<I> {
    pub const SCALE: u32 = I::MAX_POWER_OF_TWO;
    pub const DENOMINATOR: I::Wide = I::MAX_POSITIVE_POWER_OF_TWO;

    #[inline(always)]
    pub fn new(value: I) -> Self {
        debug_assert!(value.to_wide() <= Self::DENOMINATOR);
        debug_assert!(value >= I::ZERO);
        Self { value }
    }

    #[inline(always)]
    pub fn mid(self, other: Self) -> Self {
        let value = (self.value + other.value) >> 1;
        Self { value }
    }
    #[inline(always)]
    pub fn normalize(value: I::Wide) -> I {
        // hope the compiler will optimize it
        I::from_wide(value / Self::DENOMINATOR)
    }

    #[inline(always)]
    pub fn scale_raw(self, scalar: I) -> I::Wide {
        let s = self.value.to_wide();
        s * scalar.to_wide()
    }

    #[inline(always)]
    pub fn scale_point_raw(self, point: IntPoint<I>) -> IntVector<I> {
        IntVector {
            x: self.scale_raw(point.x),
            y: self.scale_raw(point.y),
        }
    }

    #[inline(always)]
    pub fn scale_vector_raw(self, vector: IntVector<I>) -> IntVector<I> {
        let s = self.value.to_wide();
        let x = vector.x * s;
        let y = vector.y * s;
        IntVector { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::UnitRatio;
    use crate::int::point::IntPoint;
    use crate::int::vector::IntVector;

    #[test]
    fn mid_uses_direct_average() {
        let a = UnitRatio::<i32>::new(2);
        let b = UnitRatio::<i32>::new(5);

        assert_eq!(a.mid(b).value, 3);
    }

    #[test]
    fn scale_raw_keeps_denominator() {
        let half = UnitRatio::<i32>::new((UnitRatio::<i32>::DENOMINATOR / 2) as i32);

        assert_eq!(half.scale_raw(10), UnitRatio::<i32>::DENOMINATOR * 5);
    }

    #[test]
    fn scale_point_raw_keeps_denominator() {
        let half = UnitRatio::<i32>::new((UnitRatio::<i32>::DENOMINATOR / 2) as i32);
        let point = IntPoint::new(2, 3);
        let vector = half.scale_point_raw(point);

        assert_eq!(vector.x, UnitRatio::<i32>::DENOMINATOR);
        assert_eq!(
            vector.y,
            UnitRatio::<i32>::DENOMINATOR + UnitRatio::<i32>::DENOMINATOR / 2
        );
    }

    #[test]
    fn scale_vector_raw_keeps_denominator() {
        let half = UnitRatio::<i32>::new((UnitRatio::<i32>::DENOMINATOR / 2) as i32);
        let vector = IntVector::<i32>::new(2, 3);
        let scaled = half.scale_vector_raw(vector);

        assert_eq!(scaled.x, UnitRatio::<i32>::DENOMINATOR);
        assert_eq!(
            scaled.y,
            UnitRatio::<i32>::DENOMINATOR + UnitRatio::<i32>::DENOMINATOR / 2
        );
    }
}
