use crate::int::number::int::IntNumber;
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
    pub fn sqr_length(self) -> T::Wide {
        let x = self.x.to_wide();
        let y = self.y.to_wide();
        x * x + y * y
    }

    #[inline(always)]
    pub fn sqr_distance(self, other: Self) -> T::Wide {
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
    use crate::int::point::IntPoint;

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
