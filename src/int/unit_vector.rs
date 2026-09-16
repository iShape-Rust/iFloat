use crate::float::number::FloatNumber;
use crate::int::number::fixed_scale::FixedScale;
use crate::int::number::int::IntNumber;
use crate::int::number::uint::UIntNumber;
use crate::int::number::wide_int::WideIntNumber;
use crate::int::vector::IntVector;
use core::ops::Mul;

/// An approximate unit direction stored as fixed-scale integer components.
///
/// Obtain one with [`IntVector::fast_normalize`] or [`Self::normalize_with_float`].
/// The represented components are
/// `x() / DENOMINATOR` and `y() / DENOMINATOR`, each in `[-1, 1]`. The length is
/// approximately one and never exceeds it.
///
/// Integer normalization via `fast_normalize` favors speed over precision:
/// it keeps about 6, 14, or 30 bits
/// of direction precision for `i16`, `i32`, or `i64`, respectively.
/// The storage scale does not imply that all stored bits are accurate.
/// [`Rotation::apply`](crate::int::angle::Rotation::apply) preserves the upper
/// length bound, but repeated rotations accumulate contraction and angular
/// error; the normalization precision above is not a bound on that accumulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitIntVector<T: IntNumber = i32> {
    x: T,
    y: T,
}

impl<T: IntNumber> UnitIntVector<T> {
    // Only for arithmetic that proves the resulting norm is at most one.
    #[inline(always)]
    pub(crate) fn from_components(x: T, y: T) -> Self {
        Self { x, y }
    }
    /// Converts floating-point components to the fixed-scale representation.
    ///
    /// Components are truncated toward zero. This does not normalize the input:
    /// callers supplying a direction should normalize it first. Returns `None`
    /// for non-finite components, components outside [-1, 1], a quantized zero
    /// vector, or an integer representation whose squared length exceeds one.
    /// Floating-point normalization/rotation may overshoot one slightly; callers
    /// must account for that error before conversion. The final length check is
    /// exact integer arithmetic, including for i64.
    #[inline]
    pub fn try_from_float<F: FloatNumber>(x: F, y: F) -> Option<Self> {
        let (x, y) = (x.to_f64(), y.to_f64());
        if x.abs() > 1.0 || y.abs() > 1.0 {
            return None;
        }
        let scale = Self::DENOMINATOR.to_f64();
        let x = T::from_rounded_float(libm::trunc(x * scale));
        let y = T::from_rounded_float(libm::trunc(y * scale));
        let (wx, wy) = (x.to_wide(), y.to_wide());
        let squared = wx * wx + wy * wy;
        if squared == T::Wide::ZERO || squared > Self::DENOMINATOR * Self::DENOMINATOR {
            return None;
        }
        Some(Self { x, y })
    }

    /// Normalizes an integer vector using f64 arithmetic, returning `None` only
    /// for the zero vector. Supports the full built-in wide component range,
    /// including i128::MIN; no integer products of the input are formed.
    ///
    /// Axis directions are exact. Other directions are contracted slightly
    /// before conversion so their fixed-scale length never exceeds one.
    /// Precision is limited by f64 arithmetic and the output component scale.
    /// When normalizing a point difference, subtract the integer points first
    /// to retain small differences between large coordinates.
    #[inline]
    pub fn normalize_with_float(vector: IntVector<T>) -> Option<Self> {
        if vector.x == T::Wide::ZERO && vector.y == T::Wide::ZERO {
            return None;
        }
        let x = vector.x.to_f64();
        let y = vector.y.to_f64();
        if x == 0.0 {
            return Self::try_from_float(0.0, y.signum());
        }
        if y == 0.0 {
            return Self::try_from_float(x.signum(), 0.0);
        }
        // Squaring and adding even two i128 components fits in f64.
        let length = FloatNumber::sqrt(x * x + y * y);
        let inward = 1.0 - 4.0 * f64::EPSILON;
        Self::try_from_float(x / length * inward, y / length * inward)
    }

    /// The stored integer value representing one: 2^14, 2^30, or 2^62.
    pub const DENOMINATOR: T::Wide = FixedScale::<T>::DENOMINATOR;

    /// Returns the stored, scaled x component.
    #[inline(always)]
    pub fn x(self) -> T {
        self.x
    }

    /// Returns the stored, scaled y component.
    #[inline(always)]
    pub fn y(self) -> T {
        self.y
    }

    /// Scales this direction into an integer vector, rounding to the nearest
    /// integer with midpoint values away from zero. Every value of `T` is valid.
    /// Direction approximation error grows with the scalar's magnitude.
    #[inline(always)]
    pub fn scale(self, scalar: T) -> IntVector<T> {
        let scalar = scalar.to_wide();
        IntVector {
            x: (self.x.to_wide() * scalar).shr_round(FixedScale::<T>::SHIFT),
            y: (self.y.to_wide() * scalar).shr_round(FixedScale::<T>::SHIFT),
        }
    }

    #[inline(always)]
    pub(crate) fn with_vector(vector: IntVector<T>) -> Option<Self> {
        let sqr_length = vector.sqr_length();
        if sqr_length == T::WideUInt::ZERO {
            return None;
        }

        let precision = T::HALF_POWER_OF_TWO - 1;
        let shift = (sqr_length.ilog2().saturating_sub(2 * precision) + 1) >> 1;
        // Round the reduced squared length upward so the reciprocal never
        // makes the represented direction longer than one. This also works
        // with shift == 0 and avoids an overflowing addition before shifting.
        let one = T::WideUInt::ONE;
        let reduced_length = ((sqr_length - one) >> (2 * shift)) + one;
        let sqr_unit = one << (2 * FixedScale::<T>::SHIFT);
        let scale = (sqr_unit / reduced_length).isqrt();

        // scale represents S * 2^shift / length. Apply it to the original
        // components, then discard the extra fractional bits toward zero.
        let x = T::from_uint((scale * vector.x.unsigned_abs()) >> shift);
        let y = T::from_uint((scale * vector.y.unsigned_abs()) >> shift);
        Some(Self {
            x: if vector.x < T::Wide::ZERO { -x } else { x },
            y: if vector.y < T::Wide::ZERO { -y } else { y },
        })
    }
}

impl<T: IntNumber> Mul<T> for UnitIntVector<T> {
    type Output = IntVector<T>;

    #[inline(always)]
    fn mul(self, scalar: T) -> Self::Output {
        self.scale(scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::UnitIntVector;
    use crate::int::point::IntPoint;
    use crate::int::vector::IntVector;

    #[test]
    fn scales_direction_into_integer_vector() {
        let direction = IntVector::<i32>::new(3, 4).fast_normalize().unwrap();
        assert_eq!(direction * 10, IntVector::new(6, 8));
        assert_eq!(direction * -10, IntVector::new(-6, -8));
        assert_eq!(direction * 0, IntVector::new(0, 0));
    }

    macro_rules! check_type {
        ($name:ident, $t:ty, $wide:ty, $tolerance:expr) => {
            #[test]
            fn $name() {
                assert!(IntVector::<$t>::new(0, 0).fast_normalize().is_none());
                let denominator = UnitIntVector::<$t>::DENOMINATOR;
                let axis = IntVector::<$t>::new(-1, 0).fast_normalize().unwrap();
                // Negating the minimum scalar must produce a positive wide result.
                assert_eq!((axis * <$t>::MIN).x, -(<$t>::MIN as $wide));
                assert_eq!((axis * <$t>::MIN).y, 0);

                let check = |x: $wide, y: $wide| {
                    let vector = IntVector::<$t>::new(x, y);
                    let Some(unit) = vector.fast_normalize() else {
                        assert_eq!((x, y), (0, 0));
                        return;
                    };
                    let (a, b) = (
                        unit.x() as f64 / denominator as f64,
                        unit.y() as f64 / denominator as f64,
                    );
                    assert!(a.abs() <= 1.0 && b.abs() <= 1.0);
                    let (raw_x, raw_y) = (unit.x() as $wide, unit.y() as $wide);
                    assert!(raw_x * raw_x + raw_y * raw_y <= denominator * denominator);
                    let length = libm::hypot(x as f64, y as f64);
                    assert!(
                        (a - x as f64 / length).abs() < $tolerance,
                        "x: ({x}, {y}) -> ({a}, {b})"
                    );
                    assert!(
                        (b - y as f64 / length).abs() < $tolerance,
                        "y: ({x}, {y}) -> ({a}, {b})"
                    );
                    if x != <$wide>::MIN && y != <$wide>::MIN {
                        let opposite = IntVector::<$t>::new(-x, -y).fast_normalize().unwrap();
                        assert_eq!((opposite.x(), opposite.y()), (-unit.x(), -unit.y()));
                    }
                    for scalar in [<$t>::MIN, <$t>::MAX, -3, -1, 0, 1, 3] {
                        let actual = unit * scalar;
                        for (component, result) in [(unit.x(), actual.x), (unit.y(), actual.y)] {
                            let product = component as i128 * scalar as i128;
                            let divisor = denominator as i128;
                            let quotient = product / divisor;
                            let remainder = product % divisor;
                            let rounded = quotient
                                + if remainder.abs() * 2 >= divisor {
                                    product.signum()
                                } else {
                                    0
                                };
                            assert_eq!(result as i128, rounded);
                        }
                    }
                };

                let limit = 2 * denominator - 2;
                for x in [-limit, -limit + 1, -3, -1, 0, 1, 3, limit] {
                    for y in [-limit, -4, -1, 0, 1, 4, limit] {
                        check(x, y);
                    }
                }
                // A valid point difference longer than S must not collapse to zero.
                let coordinate = (3 * (denominator / 4)) as $t;
                let long = IntPoint::new(coordinate, 0) - IntPoint::new(-coordinate, 0);
                check(long.x, long.y);
                let direction = long.fast_normalize().unwrap();
                assert!(direction.x() as f64 / denominator as f64 > 1.0 - $tolerance);
                assert_eq!(direction.y(), 0);
                // Exercise zero and nonzero reciprocal shifts near bit boundaries.
                for power in 0..(<$t>::BITS - 1) {
                    let magnitude = (1 as $wide) << power;
                    for x in [magnitude - 1, magnitude, magnitude.saturating_add(1)] {
                        for y in [1, x / 3, x / 2, x] {
                            check(x, y);
                            check(-x, y);
                        }
                    }
                }
            }
        };
    }

    check_type!(normalize_and_scale_i16, i16, i32, 0.04);
    check_type!(normalize_and_scale_i32, i32, i64, 0.00015);
    check_type!(normalize_and_scale_i64, i64, i128, 1e-9);
}
