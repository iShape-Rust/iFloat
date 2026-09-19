use crate::float::compatible::FloatPointCompatible;
use crate::float::number::FloatNumber;
use crate::float::point::FloatPoint;
use crate::float::rect::{FloatRect, FloatRectError};
use crate::int::number::int::IntNumber;
use crate::int::number::wide_int::WideIntNumber;
use crate::int::point::IntPoint;
use crate::int::rect::IntRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPointAdapterScaleError {
    /// Input bounds violate the floating-point coordinate contract.
    InvalidRect(FloatRectError),
    /// Requested scale is larger than the safe adapter scale for the input bounds.
    ScaleTooLarge,
    /// Scale is too small to have a finite reciprocal in the scalar type.
    ScaleTooSmall,
    /// Requested scale is zero or negative.
    ScaleNonPositive,
    /// Requested scale is NaN or infinite.
    ScaleNotFinite,
}

impl From<FloatRectError> for FloatPointAdapterScaleError {
    fn from(error: FloatRectError) -> Self {
        Self::InvalidRect(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPointAdapterRangeError {
    /// Point is outside the source rectangle or its enclosing integer grid.
    PointOutOfRange,
}

#[derive(Clone)]
/// Maps a bounded floating-point coordinate space onto an integer grid.
///
/// Input coordinates and rectangle bounds must be finite, with absolute values
/// at most `2^60` for `f32` or `2^500` for `f64`. Bounds must satisfy `min <= max`
/// on each axis. See the [`crate::float`] coordinate-range contract.
/// Constructors validate these bounds, including rectangles assembled through
/// public fields. Fallible constructors return [`FloatPointAdapterScaleError::InvalidRect`];
/// infallible constructors panic. Coordinates are never clamped.
/// The coordinate limits do not apply to the stored scales.
///
/// The adapter controls converted coordinate magnitude, but it cannot prove
/// that every expression in a downstream integer algorithm is safe. For a
/// conservative general-purpose budget covering point differences, dot and
/// cross products, and squared lengths, use the conservative constructors such
/// as [`Self::new_conservative`] or [`Self::with_iter_conservative`]. They use
/// [`Self::CONSERVATIVE_COORDINATE_BITS`], reserving an extra bit for coordinate
/// rounding. Larger explicit budgets require a stronger downstream range analysis.
pub struct FloatPointAdapter<P: FloatPointCompatible, I: IntNumber> {
    dir_scale: P::Scalar,
    inv_scale: P::Scalar,
    offset: P,
    rect: FloatRect<P::Scalar>,
    int_rect: IntRect<I>,
}

impl<P: FloatPointCompatible, I: IntNumber> FloatPointAdapter<P, I> {
    const SCALE_SAFETY_BITS: i32 = 3;

    /// Conservative coordinate-bit budget for point differences and their products.
    ///
    /// Converted coordinates have magnitude at most `2^(I::BITS - 3)`, including
    /// both endpoints. This reserves one extra bit for rounding inside the strict
    /// arithmetic range `(-2^(I::BITS - 2), 2^(I::BITS - 2))` of [`IntPoint`].
    /// For `i16`, `i32`, and `i64`, the budgets are 13, 29, and 61 bits.
    /// This does not guarantee that arbitrary downstream arithmetic fits.
    pub const CONSERVATIVE_COORDINATE_BITS: u32 = I::BITS - 3;

    /// Creates an automatically scaled adapter with the conservative coordinate budget.
    /// See [`Self::CONSERVATIVE_COORDINATE_BITS`] for the range and rounding margin.
    ///
    /// # Panics
    /// Panics for invalid bounds, as in [`Self::with_coordinate_bits`].
    #[inline]
    pub fn new_conservative(rect: FloatRect<P::Scalar>) -> Self {
        Self::with_coordinate_bits(rect, Self::CONSERVATIVE_COORDINATE_BITS)
    }

    /// Creates an automatically scaled adapter from points with the conservative budget.
    /// Empty input uses a zero rectangle and scale one.
    ///
    /// # Panics
    /// Panics for invalid coordinates, as in [`Self::with_iter_and_coordinate_bits`].
    #[inline]
    pub fn with_iter_conservative<'a, Q>(iter: Q) -> Self
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        Self::with_iter_and_coordinate_bits(iter, Self::CONSERVATIVE_COORDINATE_BITS)
    }

    /// Validates bounds and an explicit scale against the conservative coordinate budget.
    /// Preserves the requested scale, including for single-point bounds. Returns the
    /// same errors as [`Self::try_with_scale_and_coordinate_bits`].
    #[inline]
    pub fn try_with_scale_conservative(
        rect: FloatRect<P::Scalar>,
        scale: P::Scalar,
    ) -> Result<Self, FloatPointAdapterScaleError> {
        Self::try_with_scale_and_coordinate_bits(rect, scale, Self::CONSERVATIVE_COORDINATE_BITS)
    }

    /// Creates an adapter from points with an explicit scale and the conservative budget.
    /// Empty input uses a zero rectangle. Preserves the requested scale, including for
    /// empty or single-point input. Returns the same errors as
    /// [`Self::try_with_iter_and_scale_and_coordinate_bits`].
    #[inline]
    pub fn try_with_iter_and_scale_conservative<'a, Q>(
        iter: Q,
        scale: P::Scalar,
    ) -> Result<Self, FloatPointAdapterScaleError>
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        Self::try_with_iter_and_scale_and_coordinate_bits(iter, scale, Self::CONSERVATIVE_COORDINATE_BITS)
    }

    /// Creates an adapter and automatically selects a power-of-two scale while
    /// reserving internal coordinate safety bits.
    ///
    /// Algorithms with a precise bit budget should prefer
    /// [`Self::with_coordinate_bits`].
    /// The scale is capped at the largest finite power of two in the scalar
    /// type, which may reduce grid precision for very small bounds.
    /// Within the supported coordinate range, automatic scales and their
    /// reciprocals are finite.
    ///
    /// # Panics
    /// Panics if rectangle bounds are non-finite, reversed, or exceed the
    /// supported coordinate limits (`2^60` for `f32`, `2^500` for `f64`).
    #[inline]
    pub fn new(rect: FloatRect<P::Scalar>) -> Self {
        rect.validate().expect("Invalid adapter bounds");
        let (offset, radius) = Self::center_and_radius(&rect);
        let (dir_scale, inv_scale) = Self::automatic_scales(radius, None);
        Self::from_transform(rect, offset, dir_scale, inv_scale)
    }

    /// Creates an adapter whose converted coordinates have magnitude at most
    /// `2^coordinate_bits` for every point inside `rect`.
    ///
    /// The selected scale is a power of two. `coordinate_bits` may not exceed
    /// the highest non-sign bit index in `I`. This bounds converted coordinate
    /// magnitude but does not independently guarantee that every downstream
    /// product fits. `I::BITS - 3` is a conservative budget for the general
    /// point and vector operations provided by this crate.
    /// The scale is capped at the largest finite power of two in the scalar type.
    ///
    /// # Panics
    /// Panics for invalid rectangle bounds or if `coordinate_bits > I::BITS - 2`.
    #[inline]
    pub fn with_coordinate_bits(rect: FloatRect<P::Scalar>, coordinate_bits: u32) -> Self {
        rect.validate().expect("Invalid adapter bounds");
        assert!(coordinate_bits <= I::BITS - 2);

        let (offset, radius) = Self::center_and_radius(&rect);
        let (dir_scale, inv_scale) = Self::automatic_scales(radius, Some(coordinate_bits));
        Self::from_transform(rect, offset, dir_scale, inv_scale)
    }

    /// Creates an adapter with an explicit scale, without checking the coordinate
    /// budget. A rectangle containing a single point uses scale one, as in
    /// [`Self::new`].
    ///
    /// # Panics
    /// Panics for invalid rectangle bounds or if the supplied scale is
    /// non-positive, non-finite, or has a non-finite reciprocal.
    /// Use [`Self::try_with_scale`] to receive an error.
    #[inline]
    pub fn with_scale(rect: FloatRect<P::Scalar>, scale: P::Scalar) -> Self {
        rect.validate().expect("Invalid adapter bounds");
        let inv_scale = Self::inverse_scale(scale).expect("Invalid adapter scale");
        let (offset, radius) = Self::center_and_radius(&rect);
        if radius == P::Scalar::ZERO {
            Self::from_transform(rect, offset, P::Scalar::ONE, P::Scalar::ONE)
        } else {
            Self::from_transform(rect, offset, scale, inv_scale)
        }
    }

    /// Validates rectangle bounds and an explicit scale against the default budget.
    /// Returns [`FloatPointAdapterScaleError::ScaleTooSmall`] if its reciprocal
    /// is non-finite. The requested scale is retained, including for point bounds.
    #[inline]
    pub fn try_with_scale(
        rect: FloatRect<P::Scalar>,
        scale: P::Scalar,
    ) -> Result<Self, FloatPointAdapterScaleError> {
        Self::checked_with_scale(rect, scale, None)
    }

    /// Creates an adapter with an explicit scale, rejecting scales that can
    /// produce coordinates with magnitude greater than `2^coordinate_bits`.
    /// The requested scale must also have a finite reciprocal. Rectangle bounds
    /// are validated against the floating-point coordinate contract.
    #[inline]
    pub fn try_with_scale_and_coordinate_bits(
        rect: FloatRect<P::Scalar>,
        scale: P::Scalar,
        coordinate_bits: u32,
    ) -> Result<Self, FloatPointAdapterScaleError> {
        assert!(coordinate_bits <= I::BITS - 2);
        Self::checked_with_scale(rect, scale, Some(coordinate_bits))
    }

    #[inline]
    pub fn with_radius_and_scale(radius: P::Scalar, scale: P::Scalar) -> FloatPointAdapter<P, I> {
        let rect = FloatRect::new(-radius, radius, -radius, radius).unwrap();
        FloatPointAdapter::with_scale(rect, scale)
    }

    /// Creates an adapter from input points.
    /// An empty iterator uses a zero rectangle and scale one.
    ///
    /// # Panics
    /// Panics if any coordinate is non-finite or exceeds the supported absolute
    /// limit (`2^60` for `f32`, `2^500` for `f64`).
    #[inline]
    pub fn with_iter<'a, Q>(iter: Q) -> Self
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        Self::new(
            FloatRect::with_iter(iter)
                .expect("Invalid adapter bounds")
                .unwrap_or(FloatRect::zero()),
        )
    }

    /// Creates an adapter from input points with an explicit coordinate-bit budget.
    /// See [`Self::with_coordinate_bits`] for the scale and range guarantees.
    /// An empty iterator uses a zero rectangle and scale one.
    ///
    /// # Panics
    /// Panics if any coordinate violates the floating-point coordinate contract
    /// or if `coordinate_bits > I::BITS - 2`.
    #[inline]
    pub fn with_iter_and_coordinate_bits<'a, Q>(iter: Q, coordinate_bits: u32) -> Self
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        let rect = FloatRect::with_iter(iter)
            .expect("Invalid adapter bounds")
            .unwrap_or(FloatRect::zero());
        Self::with_coordinate_bits(rect, coordinate_bits)
    }

    /// Creates an adapter from input points with a checked explicit scale and
    /// coordinate-bit budget. See [`Self::try_with_scale_and_coordinate_bits`]
    /// for scale validation. Invalid input coordinates return
    /// [`FloatPointAdapterScaleError::InvalidRect`].
    /// An empty iterator uses a zero rectangle. The requested scale is retained,
    /// including for empty input or bounds containing a single point.
    ///
    /// # Panics
    /// Panics if `coordinate_bits > I::BITS - 2`.
    #[inline]
    pub fn try_with_iter_and_scale_and_coordinate_bits<'a, Q>(
        iter: Q,
        scale: P::Scalar,
        coordinate_bits: u32,
    ) -> Result<Self, FloatPointAdapterScaleError>
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        let rect = FloatRect::with_iter(iter)?.unwrap_or(FloatRect::zero());
        Self::try_with_scale_and_coordinate_bits(rect, scale, coordinate_bits)
    }

    #[inline]
    pub fn with_iter_and_scale_checked<'a, Q>(
        iter: Q,
        scale: P::Scalar,
    ) -> Result<Self, FloatPointAdapterScaleError>
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        Self::try_with_scale(FloatRect::with_iter(iter)?.unwrap_or(FloatRect::zero()), scale)
    }

    #[inline]
    fn center_and_radius(rect: &FloatRect<P::Scalar>) -> (P, P::Scalar) {
        let x = (rect.min_x + rect.max_x) * P::Scalar::HALF;
        let y = (rect.min_y + rect.max_y) * P::Scalar::HALF;
        // The rounded center can lie on a bound for adjacent floats. Measure
        // from it rather than halving the span, which can underestimate radius
        // or underflow to zero for a one-subnormal span.
        let radius = (x - rect.min_x)
            .max(rect.max_x - x)
            .max(y - rect.min_y)
            .max(rect.max_y - y);
        (P::from_xy(x, y), radius)
    }

    #[inline]
    fn scale_exponent(radius: P::Scalar, coordinate_bits: Option<u32>) -> i32 {
        let log2 = radius.log2().to_i32();
        if let Some(bits) = coordinate_bits {
            let power = P::Scalar::from_float(libm::exp2(log2 as f64));
            let ceil_log2 = if power < radius { log2 + 1 } else { log2 };
            bits as i32 - ceil_log2
        } else {
            I::BITS as i32 - Self::SCALE_SAFETY_BITS - log2
        }
    }

    #[inline]
    fn automatic_scales(radius: P::Scalar, coordinate_bits: Option<u32>) -> (P::Scalar, P::Scalar) {
        if radius == P::Scalar::ZERO {
            return (P::Scalar::ONE, P::Scalar::ONE);
        }

        // Validated coordinate bounds keep the reciprocal finite, even with
        // a zero-bit budget. Only tiny bounds need the upper scale cap.
        let max_exponent = P::Scalar::MAX_EXP - 1;
        let exponent = Self::scale_exponent(radius, coordinate_bits).min(max_exponent);
        let scale = P::Scalar::from_float(libm::exp2(exponent as f64));
        (scale, P::Scalar::ONE / scale)
    }

    #[inline]
    fn checked_with_scale(
        rect: FloatRect<P::Scalar>,
        scale: P::Scalar,
        coordinate_bits: Option<u32>,
    ) -> Result<Self, FloatPointAdapterScaleError> {
        rect.validate()?;
        let inv_scale = Self::inverse_scale(scale)?;
        let (offset, radius) = Self::center_and_radius(&rect);
        if radius != P::Scalar::ZERO {
            let exponent = Self::scale_exponent(radius, coordinate_bits);
            // This comparison limit may exceed the scalar range. Unlike the
            // stored automatic scale, it must not cap a valid explicit scale.
            let limit = P::Scalar::from_float(libm::exp2(exponent as f64));
            if limit < scale {
                return Err(FloatPointAdapterScaleError::ScaleTooLarge);
            }
        }
        Ok(Self::from_transform(rect, offset, scale, inv_scale))
    }

    #[inline]
    fn from_transform(
        rect: FloatRect<P::Scalar>,
        offset: P,
        dir_scale: P::Scalar,
        inv_scale: P::Scalar,
    ) -> Self {
        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
            int_rect: Self::grid_rect(&rect, &offset, dir_scale),
        }
    }

    #[inline]
    fn grid_rect(rect: &FloatRect<P::Scalar>, offset: &P, scale: P::Scalar) -> IntRect<I> {
        // Enclose the source bounds in the existing grid without changing its
        // scale or origin. Use the same scalar arithmetic as float_to_int.
        let min_x = ((rect.min_x - offset.x()) * scale).to_f64();
        let max_x = ((rect.max_x - offset.x()) * scale).to_f64();
        let min_y = ((rect.min_y - offset.y()) * scale).to_f64();
        let max_y = ((rect.max_y - offset.y()) * scale).to_f64();
        IntRect::new(
            I::from_float(libm::floor(min_x)),
            I::from_float(libm::ceil(max_x)),
            I::from_float(libm::floor(min_y)),
            I::from_float(libm::ceil(max_y)),
        )
    }

    #[inline]
    fn inverse_scale(scale: P::Scalar) -> Result<P::Scalar, FloatPointAdapterScaleError> {
        if !scale.is_finite() {
            return Err(FloatPointAdapterScaleError::ScaleNotFinite);
        }
        if scale <= P::Scalar::ZERO {
            return Err(FloatPointAdapterScaleError::ScaleNonPositive);
        }
        let inverse = P::Scalar::ONE / scale;
        if !inverse.is_finite() {
            return Err(FloatPointAdapterScaleError::ScaleTooSmall);
        }
        Ok(inverse)
    }

    #[inline(always)]
    pub fn dir_scale(&self) -> P::Scalar {
        self.dir_scale
    }

    #[inline(always)]
    pub fn inv_scale(&self) -> P::Scalar {
        self.inv_scale
    }

    #[inline(always)]
    pub fn offset(&self) -> P {
        self.offset
    }

    /// Returns the original floating-point bounds used to validate input points.
    /// Snapped points may lie outside these bounds, on the enclosing integer grid.
    #[inline(always)]
    pub fn rect(&self) -> &FloatRect<P::Scalar> {
        &self.rect
    }

    /// Converts a point from the enclosing integer grid back to floating point.
    /// Debug builds assert that the point is inside that grid's bounds.
    #[inline(always)]
    pub fn int_to_float(&self, point: &IntPoint<I>) -> P {
        debug_assert!(
            self.int_rect.contains(*point),
            "Integer point [{}, {}] is outside the adapter grid",
            point.x,
            point.y
        );
        let fx: P::Scalar = FloatNumber::from_int(point.x);
        let fy: P::Scalar = FloatNumber::from_int(point.y);
        let x = fx * self.inv_scale + self.offset.x();
        let y = fy * self.inv_scale + self.offset.y();
        P::from_xy(x, y)
    }

    /// Validates the integer point against the enclosing grid before conversion.
    /// The result may lie outside [`Self::rect`].
    #[inline(always)]
    pub fn try_int_to_float(&self, point: &IntPoint<I>) -> Result<P, FloatPointAdapterRangeError> {
        if !self.int_rect.contains(*point) {
            return Err(FloatPointAdapterRangeError::PointOutOfRange);
        }

        Ok(self.int_to_float(point))
    }

    #[inline(always)]
    pub fn float_to_int(&self, point: &P) -> IntPoint<I> {
        if cfg!(debug_assertions) {
            let radius = self.rect.height().max(self.rect.width()) * P::Scalar::from_float(0.01);
            if !self.rect.contains_with_radius(point, radius) {
                panic!(
                    "You are trying to convert a point[{}, {}] which is out of rect: {}",
                    point.x(),
                    point.y(),
                    self.rect
                );
            }
        }

        let sx = (point.x() - self.offset.x()) * self.dir_scale;
        let sy = (point.y() - self.offset.y()) * self.dir_scale;

        let x = I::from_rounded_float(sx);
        let y = I::from_rounded_float(sy);
        IntPoint { x, y }
    }

    #[inline(always)]
    pub fn try_float_to_int(&self, point: &P) -> Result<IntPoint<I>, FloatPointAdapterRangeError> {
        if !self.rect.contains(point) {
            return Err(FloatPointAdapterRangeError::PointOutOfRange);
        }

        let sx = (point.x() - self.offset.x()) * self.dir_scale;
        let sy = (point.y() - self.offset.y()) * self.dir_scale;

        let x = I::from_rounded_float(sx);
        let y = I::from_rounded_float(sy);
        Ok(IntPoint { x, y })
    }

    /// Rounds to the nearest grid point, which may lie outside [`Self::rect`].
    #[inline(always)]
    pub fn snap_to_grid(&self, point: &P) -> P {
        self.int_to_float(&self.float_to_int(point))
    }

    /// Checks the input against [`Self::rect`] and rounds it to the nearest grid
    /// point. Rounding may move the result outside the original float bounds.
    #[inline(always)]
    pub fn try_snap_to_grid(&self, point: &P) -> Result<P, FloatPointAdapterRangeError> {
        self.try_float_to_int(point)
            .and_then(|point| self.try_int_to_float(&point))
    }

    #[inline(always)]
    pub fn round_sqr_len_to_int(&self, value: P::Scalar) -> I::Wide {
        let scale = self.dir_scale;
        // Multiply the area first: scale * scale can overflow even when
        // the scaled area fits, especially with f32 and the i64 engine.
        I::Wide::from_rounded_float((value * scale) * scale)
    }

    #[inline(always)]
    pub fn round_len_to_int(&self, value: P::Scalar) -> I {
        I::from_rounded_float(self.dir_scale * value)
    }

    #[inline(always)]
    pub fn len_to_float(&self, value: I) -> P::Scalar {
        let f: P::Scalar = FloatNumber::from_int(value);
        f * self.inv_scale
    }

    #[inline(always)]
    pub fn to_float_point_adapter(&self) -> FloatPointAdapter<FloatPoint<P::Scalar>, I> {
        FloatPointAdapter {
            dir_scale: self.dir_scale,
            inv_scale: self.inv_scale,
            offset: FloatPoint::from_point(self.offset),
            rect: self.rect,
            int_rect: self.int_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::{FloatPointAdapter, FloatPointAdapterRangeError, FloatPointAdapterScaleError};
    use crate::float::compatible::FloatPointCompatible;
    use crate::float::number::FloatNumber;
    use crate::float::point::FloatPoint;
    use crate::float::rect::FloatRect;
    use crate::int::number::int::IntNumber;
    use crate::int::point::IntPoint;

    #[test]
    fn round_sqr_len_to_int_avoids_intermediate_overflow() {
        let points = [[0.0_f32, 0.0], [0.01, 0.01]];
        let adapter = FloatPointAdapter::<[f32; 2], i64>::with_iter(points.iter());
        let scale = adapter.dir_scale();
        assert!(scale.is_finite());
        assert!((scale * scale).is_infinite());

        let value = 1e-6_f32;
        let wide_scale = f64::from(scale);
        let expected = (wide_scale * f64::from(value) * wide_scale) as i128;
        assert!(expected > 0 && expected < i128::MAX);
        assert_eq!(adapter.round_sqr_len_to_int(value), expected);
        assert_eq!(adapter.round_sqr_len_to_int(0.0), 0);

        // Round the final area, rather than a length that is then squared.
        for (area, expected) in [(1.5_f64, 2_i128), (2.5, 3)] {
            let value = (area / wide_scale / wide_scale) as f32;
            assert_eq!(adapter.round_sqr_len_to_int(value), expected);
        }
    }

    #[test]
    fn test_0() {
        let rect = FloatRect {
            min_x: 1.0,
            max_x: 1.0,
            min_y: -2.0,
            max_y: -2.0,
        };

        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::new(rect);

        assert_eq!(adapter.dir_scale(), 1.0);
        assert_eq!(adapter.inv_scale(), 1.0);
    }

    #[test]
    fn test_1() {
        let rect = FloatRect {
            min_x: 0.0,
            max_x: 10.0,
            min_y: 0.0,
            max_y: 100.0,
        };

        let adapter = FloatPointAdapter::<[f64; 2], i32>::new(rect);

        let f0 = [10.0, 2.0];
        let p0 = adapter.float_to_int(&f0);
        let f1: [f64; 2] = adapter.int_to_float(&p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_2() {
        let points = [[-2.0, -4.0], [-2.0, 3.0], [5.0, 3.0], [5.0, -4.0]];

        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());

        let f0 = [1.0, 2.0];
        let p0 = adapter.float_to_int(&f0);
        let f1: [f64; 2] = adapter.int_to_float(&p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_i64_scale() {
        let rect = FloatRect {
            min_x: -1000.0,
            max_x: 1000.0,
            min_y: -1000.0,
            max_y: 1000.0,
        };

        let adapter = FloatPointAdapter::<[f64; 2], i64>::new(rect);
        let p = adapter.float_to_int(&[1.25, -2.5]);
        let f: [f64; 2] = adapter.int_to_float(&p);

        assert_eq!(p.x.abs() > i32::MAX as i64, true);
        assert_eq!((f.x() - 1.25).abs() < 0.000_0001, true);
        assert_eq!((f.y() + 2.5).abs() < 0.000_0001, true);
    }

    #[test]
    fn coordinate_bits_bound_integer_magnitude() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_coordinate_bits(
            FloatRect::new(-3.0, 3.0, -0.25, 0.25).unwrap(),
            10,
        );

        assert_eq!(adapter.dir_scale(), 256.0);
        assert_eq!(adapter.float_to_int(&[-3.0, -0.25]), IntPoint::new(-768, -64));
        assert_eq!(adapter.float_to_int(&[3.0, 0.25]), IntPoint::new(768, 64));
    }

    #[test]
    fn coordinate_bits_include_exact_power_of_two_boundary() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_coordinate_bits(
            FloatRect::new(-4.0, 4.0, -0.25, 0.25).unwrap(),
            10,
        );

        assert_eq!(adapter.dir_scale(), 256.0);
        assert_eq!(adapter.float_to_int(&[-4.0, 0.0]), IntPoint::new(-1024, 0));
        assert_eq!(adapter.float_to_int(&[4.0, 0.0]), IntPoint::new(1024, 0));
    }

    #[test]
    fn test_round_point_round_length() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            10.0,
        );

        assert_eq!(adapter.float_to_int(&[0.16, -0.16]), IntPoint::new(2, -2));
        assert_eq!(adapter.round_len_to_int(0.16), 2);
    }

    #[test]
    fn test_snap_to_grid() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            10.0,
        );

        assert_eq!(adapter.snap_to_grid(&[0.16, -0.16]), [0.2, -0.2]);
        assert_eq!(adapter.snap_to_grid(&[0.14, -0.14]), [0.1, -0.1]);
    }

    #[test]
    fn test_try_snap_to_grid_range() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            10.0,
        );

        assert_eq!(adapter.try_snap_to_grid(&[1.0, -1.0]).unwrap(), [1.0, -1.0]);
        assert_eq!(
            adapter.try_snap_to_grid(&[1.01, 0.0]).err().unwrap(),
            FloatPointAdapterRangeError::PointOutOfRange
        );
    }

    #[test]
    fn test_try_snap_to_grid_accepts_rounded_point_outside_float_rect() {
        let rect = FloatRect::new(-0.6, 0.6, -0.6, 0.6).unwrap();
        let adapter = FloatPointAdapter::<[f64; 2], i32>::try_with_scale(rect, 1.0).unwrap();

        assert_eq!(adapter.try_snap_to_grid(&[0.4, -0.4]), Ok([0.0, 0.0]));

        for (point, snapped) in [
            ([0.6, 0.0], [1.0, 0.0]),
            ([-0.6, 0.0], [-1.0, 0.0]),
            ([0.0, 0.6], [0.0, 1.0]),
            ([0.0, -0.6], [0.0, -1.0]),
        ] {
            assert!(rect.contains(&point));
            assert!(!adapter.rect().contains(&snapped));
            assert_eq!(adapter.try_snap_to_grid(&point), Ok(snapped));
            assert_eq!(adapter.snap_to_grid(&point), snapped);
        }
        // Expanding the grid must not expand the accepted float inputs.
        assert_eq!(
            adapter.try_snap_to_grid(&[0.61, 0.0]),
            Err(FloatPointAdapterRangeError::PointOutOfRange)
        );
        assert_eq!(
            adapter.try_int_to_float(&IntPoint::new(2, 0)),
            Err(FloatPointAdapterRangeError::PointOutOfRange)
        );
    }

    fn assert_grid_bounds<F: FloatNumber, I: IntNumber>() {
        let f = F::from_float::<f64>;
        for rect in [
            FloatRect::new(f(9.375), f(10.625), f(-2.625), f(-1.375)).unwrap(),
            FloatRect::new(f(10.0), f(10.0), f(-2.625), f(-1.375)).unwrap(),
            FloatRect::new(f(9.375), f(10.625), f(-2.0), f(-2.0)).unwrap(),
            FloatRect::new(f(10.0), f(10.0), f(-2.0), f(-2.0)).unwrap(),
        ] {
            for adapter in [
                FloatPointAdapter::<[F; 2], I>::new(rect),
                FloatPointAdapter::with_coordinate_bits(rect, 2),
                FloatPointAdapter::with_scale(rect, F::ONE),
                FloatPointAdapter::try_with_scale(rect, F::ONE).unwrap(),
                FloatPointAdapter::try_with_scale_and_coordinate_bits(rect, F::ONE, 2).unwrap(),
            ] {
                let copied = adapter.clone().to_float_point_adapter();
                for x in [rect.min_x, rect.max_x] {
                    for y in [rect.min_y, rect.max_y] {
                        let integer = adapter.try_float_to_int(&[x, y]).unwrap();
                        let restored = adapter.try_int_to_float(&integer).unwrap();
                        let converted = copied.try_int_to_float(&integer).unwrap();
                        assert!(restored == adapter.int_to_float(&integer));
                        assert!(restored == adapter.try_snap_to_grid(&[x, y]).unwrap());
                        assert!(restored == [converted.x, converted.y]);
                    }
                }
                // Reject the next grid cell on every side, including after a
                // checked constructor changes its automatically chosen scale.
                let min = adapter.try_float_to_int(&[rect.min_x, rect.min_y]).unwrap();
                let max = adapter.try_float_to_int(&[rect.max_x, rect.max_y]).unwrap();
                for point in [
                    IntPoint::new(min.x - I::ONE, I::ZERO),
                    IntPoint::new(max.x + I::ONE, I::ZERO),
                    IntPoint::new(I::ZERO, min.y - I::ONE),
                    IntPoint::new(I::ZERO, max.y + I::ONE),
                ] {
                    assert!(adapter.try_int_to_float(&point).is_err());
                    assert!(copied.try_int_to_float(&point).is_err());
                }
            }
        }
    }

    #[test]
    fn grid_bounds_cover_all_coordinate_types_and_constructors() {
        assert_grid_bounds::<f32, i16>();
        assert_grid_bounds::<f32, i32>();
        assert_grid_bounds::<f32, i64>();
        assert_grid_bounds::<f64, i16>();
        assert_grid_bounds::<f64, i32>();
        assert_grid_bounds::<f64, i64>();
    }

    #[test]
    fn integer_range_check_preserves_precision_beyond_f64() {
        let limit = 1_i64 << 60;
        let adapter = FloatPointAdapter::<[f64; 2], i64>::try_with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            limit as f64,
        )
        .unwrap();
        assert_eq!(limit as f64, (limit + 1) as f64);
        for sign in [-1, 1] {
            assert!(adapter.try_int_to_float(&IntPoint::new(sign * limit, 0)).is_ok());
            assert_eq!(
                adapter.try_int_to_float(&IntPoint::new(sign * (limit + 1), 0)),
                Err(FloatPointAdapterRangeError::PointOutOfRange)
            );
        }
    }

    #[test]
    fn grid_bounds_enclose_even_inward_rounded_float_bounds() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::try_with_scale(
            FloatRect::new(-0.25, 0.25, -0.25, 0.25).unwrap(),
            1.0,
        )
        .unwrap();
        assert_eq!(adapter.try_snap_to_grid(&[0.25, -0.25]), Ok([0.0, 0.0]));
        assert_eq!(adapter.try_int_to_float(&IntPoint::new(-1, 1)), Ok([-1.0, 1.0]));
        assert_eq!(adapter.try_int_to_float(&IntPoint::new(1, -1)), Ok([1.0, -1.0]));
        assert_eq!(
            adapter.try_int_to_float(&IntPoint::new(2, 0)),
            Err(FloatPointAdapterRangeError::PointOutOfRange)
        );
    }

    #[test]
    fn test_try_with_scale() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::try_with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            10.0,
        )
        .unwrap();

        assert_eq!(adapter.dir_scale(), 10.0);
        assert_eq!(adapter.inv_scale(), 0.1);
        assert_eq!(adapter.float_to_int(&[0.16, -0.16]), IntPoint::new(2, -2));
    }

    #[test]
    fn test_try_with_scale_and_coordinate_bits() {
        let rect = FloatRect::new(-4.0, 4.0, -1.0, 1.0).unwrap();
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale_and_coordinate_bits(rect.clone(), 128.0, 10)
                .unwrap();

        assert_eq!(adapter.float_to_int(&[4.0, 1.0]), IntPoint::new(512, 128));
        assert_eq!(
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale_and_coordinate_bits(rect, 512.0, 10)
                .err()
                .unwrap(),
            FloatPointAdapterScaleError::ScaleTooLarge
        );
    }

    #[test]
    fn test_try_with_scale_errors() {
        let rect = FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap();

        assert_eq!(
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale(rect.clone(), 0.0)
                .err()
                .unwrap(),
            FloatPointAdapterScaleError::ScaleNonPositive
        );
        assert_eq!(
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale(rect.clone(), f64::INFINITY)
                .err()
                .unwrap(),
            FloatPointAdapterScaleError::ScaleNotFinite
        );
        assert_eq!(
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale(rect, i32::MAX as f64)
                .err()
                .unwrap(),
            FloatPointAdapterScaleError::ScaleTooLarge
        );
    }

    #[test]
    fn test_with_iter_and_scale_checked() {
        let points = [[-2.0, -4.0], [-2.0, 3.0], [5.0, 3.0], [5.0, -4.0]];

        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_iter_and_scale_checked(points.iter(), 100.0).unwrap();

        assert_eq!(adapter.dir_scale(), 100.0);
        assert_eq!(adapter.float_to_int(&[1.25, 2.25]), IntPoint::new(-25, 275));
    }

    #[test]
    fn test_degenerate_try_with_scale_keeps_requested_scale() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::try_with_scale(
            FloatRect::new(1.0, 1.0, -2.0, -2.0).unwrap(),
            1000.0,
        )
        .unwrap();

        assert_eq!(adapter.dir_scale(), 1000.0);
        assert_eq!(adapter.float_to_int(&[1.0, -2.0]), IntPoint::ZERO);
    }

    #[test]
    fn test_try_float_to_int_range() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            10.0,
        );

        assert_eq!(
            adapter.try_float_to_int(&[1.0, -1.0]).unwrap(),
            IntPoint::new(10, -10)
        );
        assert_eq!(
            adapter.try_float_to_int(&[1.01, 0.0]).err().unwrap(),
            FloatPointAdapterRangeError::PointOutOfRange
        );
    }

    #[test]
    fn test_try_float_to_int_not_finite() {
        fn check<F: FloatNumber>(nan: F, infinity: F) {
            let limit = F::MAX_COORDINATE;
            let rect = FloatRect::new(-limit, limit, -limit, limit).unwrap();
            let adapter = FloatPointAdapter::<[F; 2], i32>::with_coordinate_bits(rect, 0);
            for value in [nan, infinity, -infinity] {
                for point in [[value, F::ZERO], [F::ZERO, value]] {
                    assert_eq!(
                        adapter.try_float_to_int(&point),
                        Err(FloatPointAdapterRangeError::PointOutOfRange),
                    );
                }
            }
        }
        check(f32::NAN, f32::INFINITY);
        check(f64::NAN, f64::INFINITY);
    }

    #[test]
    fn test_try_int_to_float_range() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_scale(
            FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(),
            10.0,
        );

        assert_eq!(
            adapter.try_int_to_float(&IntPoint::new(10, -10)).unwrap(),
            [1.0, -1.0]
        );
        assert_eq!(
            adapter.try_int_to_float(&IntPoint::new(11, 0)).err().unwrap(),
            FloatPointAdapterRangeError::PointOutOfRange
        );
    }
}
