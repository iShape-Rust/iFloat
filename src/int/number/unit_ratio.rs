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
    pub fn mid(self, other: Self) -> Self {
        let value = (self.value + other.value) >> 1;
        Self { value }
    }

    #[inline(always)]
    pub fn scale(self, scalar: I) -> I {
        let s = self.value();
        I::from_scaled_wide(s * scalar.to_wide())
    }

    #[inline(always)]
    pub fn scale_raw(self, scalar: I) -> I::Wide {
        let s = self.value();
        s * scalar.to_wide()
    }

    #[inline(always)]
    pub fn scale_wide_to_int(self, scalar: I::Wide) -> I {
        let s = self.value();
        I::from_scaled_wide(s * scalar)
    }

    #[inline(always)]
    pub fn scale_point(self, point: IntPoint<I>) -> IntPoint<I> {
        let x = self.scale(point.x);
        let y = self.scale(point.y);
        IntPoint {
            x,
            y,
        }
    }

    #[inline(always)]
    pub fn scale_point_raw(self, point: IntPoint<I>) -> IntVector<I> {
        let x = self.scale_raw(point.x);
        let y = self.scale_raw(point.y);
        IntVector {
            x,
            y,
        }
    }

    #[inline(always)]
    pub fn scale_vector_to_point(self, vector: IntVector<I>) -> IntPoint<I> {
        let x = self.scale_wide_to_int(vector.x);
        let y = self.scale_wide_to_int(vector.y);
        IntPoint {
            x,
            y,
        }
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
}
