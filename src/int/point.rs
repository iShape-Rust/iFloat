use crate::int::number::int::IntNumber;
use crate::int::number::wide_int::WideIntNumber;
use crate::int::vector::IntVector;
use core::cmp::Ordering;
use core::{fmt, ops};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// A two-dimensional integer point.
///
/// # Arithmetic range
///
/// Point differences use [`IntVector`] and therefore widen each coordinate to
/// [`IntNumber::Wide`]. Products of those components are still evaluated in
/// the same wide type. A conservative common precondition for `dot_product`,
/// `cross_product`, `sqr_length`, `sqr_distance`, and operations on vectors
/// obtained by subtracting points is:
///
/// ```text
/// -2^(T::BITS - 2) < coordinate < 2^(T::BITS - 2)
/// ```
///
/// Use [`Self::is_in_safe_range`] to check this precondition.
///
/// This guarantees enough headroom for a point difference and for the sum or
/// difference of two products. Arithmetic is unchecked beyond Rust's normal
/// debug overflow checks. Callers may use a wider range only when they prove
/// that the intermediate values used by their particular algorithm still fit.
pub struct IntPoint<T: IntNumber = i32> {
    pub x: T,
    pub y: T,
}

impl<T: IntNumber> IntPoint<T> {
    pub const ZERO: Self = Self {
        x: T::ZERO,
        y: T::ZERO,
    };
    pub const EMPTY: Self = Self { x: T::MAX, y: T::MAX };

    #[inline(always)]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Returns whether both coordinates are in the conservative arithmetic range.
    ///
    /// Each coordinate must be strictly between `-2^(T::BITS - 2)` and
    /// `2^(T::BITS - 2)`. See the type's arithmetic range documentation.
    #[inline(always)]
    pub fn is_in_safe_range(&self) -> bool {
        let limit = T::ONE << (T::BITS - 2);
        let min = -limit;
        self.x > min && self.x < limit && self.y > min && self.y < limit
    }

    #[inline(always)]
    pub fn cross_product(self, v: Self) -> T::Wide {
        let a = self.x.to_wide() * v.y.to_wide();
        let b = self.y.to_wide() * v.x.to_wide();

        a - b
    }

    #[inline(always)]
    pub fn dot_product(self, v: Self) -> T::Wide {
        let xx = self.x.to_wide() * v.x.to_wide();
        let yy = self.y.to_wide() * v.y.to_wide();
        xx + yy
    }

    #[inline(always)]
    pub fn sqr_length(self) -> T::WideUInt {
        let x = self.x.to_wide();
        let y = self.y.to_wide();
        let xx = x * x;
        let yy = y * y;
        xx.to_uint() + yy.to_uint()
    }

    #[inline(always)]
    pub fn sqr_distance(self, other: Self) -> T::WideUInt {
        (self - other).sqr_length()
    }
}

impl<T: IntNumber> fmt::Display for IntPoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl<T: IntNumber> From<[T; 2]> for IntPoint<T> {
    #[inline(always)]
    fn from(value: [T; 2]) -> Self {
        IntPoint::new(value[0], value[1])
    }
}

impl<T: IntNumber> From<(T, T)> for IntPoint<T> {
    #[inline(always)]
    fn from(value: (T, T)) -> Self {
        IntPoint::new(value.0, value.1)
    }
}

impl<T: IntNumber> PartialOrd for IntPoint<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: IntNumber> Ord for IntPoint<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let x = self.x == other.x;
        if x && self.y == other.y {
            Ordering::Equal
        } else if self.x < other.x || x && self.y < other.y {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl<T: IntNumber> ops::Add for IntPoint<T> {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        IntPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: IntNumber> ops::Sub for IntPoint<T> {
    type Output = IntVector<T>;

    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        IntVector {
            x: self.x.to_wide() - other.x.to_wide(),
            y: self.y.to_wide() - other.y.to_wide(),
        }
    }
}

impl<T: IntNumber> From<IntVector<T>> for IntPoint<T> {
    #[inline(always)]
    fn from(value: IntVector<T>) -> Self {
        Self {
            x: T::from_wide(value.x),
            y: T::from_wide(value.y),
        }
    }
}

#[macro_export]
macro_rules! int_pnt {
    ($x:expr, $y:expr) => {
        IntPoint::new($x, $y)
    };
}

#[cfg(test)]
mod tests {
    use crate::int::number::int::IntNumber;
    use crate::int::point::IntPoint;

    fn assert_safe_range<T: IntNumber>(limit: T) {
        assert!(IntPoint::<T>::ZERO.is_in_safe_range());
        assert!(!IntPoint::<T>::EMPTY.is_in_safe_range());

        for x in [-limit + T::ONE, T::ZERO, limit - T::ONE] {
            for y in [-limit + T::ONE, T::ZERO, limit - T::ONE] {
                assert!(IntPoint::new(x, y).is_in_safe_range());
            }
        }

        for value in [T::MIN, -limit - T::ONE, -limit, limit, limit + T::ONE, T::MAX] {
            assert!(!IntPoint::new(value, T::ZERO).is_in_safe_range());
            assert!(!IntPoint::new(T::ZERO, value).is_in_safe_range());
        }
    }

    #[test]
    fn test_safe_range_i16() {
        assert_safe_range::<i16>(16_384);
    }

    #[test]
    fn test_safe_range_i32() {
        assert_safe_range::<i32>(1_073_741_824);
    }

    #[test]
    fn test_safe_range_i64() {
        assert_safe_range::<i64>(4_611_686_018_427_387_904);
    }

    #[test]
    fn test_0() {
        let p: IntPoint = (1, 2).into();
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
    }

    #[test]
    fn test_1() {
        let p: IntPoint = [1, 2].into();
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
    }
    #[test]
    fn test_2() {
        assert_eq!(int_pnt![0, 0], int_pnt![0, 0]);
        assert!(int_pnt![0, 0] < int_pnt![0, 4]);
        assert!(int_pnt![1, 0] > int_pnt![0, 4]);
        assert!(int_pnt![0, 4] > int_pnt![0, 0]);
        assert!(int_pnt![0, 4] < int_pnt![1, 0]);
    }

    #[test]
    fn test_generic_i64() {
        let a: IntPoint<i64> = (1, 2).into();
        let b: IntPoint<i64> = [1, 3].into();

        assert_eq!(alloc::format!("{}", a), "[1, 2]");
        assert!(a < b);
    }

    #[test]
    fn test_sub_returns_wide_vector() {
        let a = IntPoint::new(i32::MIN, i32::MIN);
        let b = IntPoint::new(i32::MAX, i32::MAX);
        let v = a - b;

        assert_eq!(v.x, i32::MIN as i64 - i32::MAX as i64);
        assert_eq!(v.y, i32::MIN as i64 - i32::MAX as i64);
    }
}
