use crate::float::compatible::FloatPointCompatible;
use crate::float::number::FloatNumber;
use crate::float::point::FloatPoint;
use crate::float::rect::FloatRect;
use crate::int::number::int::IntNumber;
use crate::int::number::wide_int::WideIntNumber;
use crate::int::point::IntPoint;
use crate::int::rect::IntRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPointAdapterScaleError {
    /// Requested scale is larger than the safe adapter scale for the input bounds.
    ScaleTooLarge,
    /// Requested scale is zero or negative.
    ScaleNonPositive,
    /// Requested scale is NaN or infinite.
    ScaleNotFinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPointAdapterRangeError {
    /// Point is outside the source rectangle or its enclosing integer grid.
    PointOutOfRange,
}

#[derive(Clone)]
/// Maps a bounded floating-point coordinate space onto an integer grid.
///
/// The adapter controls converted coordinate magnitude, but it cannot prove
/// that every expression in a downstream integer algorithm is safe. For a
/// conservative general-purpose budget covering point differences, dot and
/// cross products, and squared lengths, use [`Self::with_coordinate_bits`]
/// with `coordinate_bits <= I::BITS - 3`. A larger value is appropriate only
/// when the downstream algorithm has a stronger range analysis.
pub struct FloatPointAdapter<P: FloatPointCompatible, I: IntNumber> {
    dir_scale: P::Scalar,
    inv_scale: P::Scalar,
    offset: P,
    rect: FloatRect<P::Scalar>,
    int_rect: IntRect<I>,
}

impl<P: FloatPointCompatible, I: IntNumber> FloatPointAdapter<P, I> {
    const SCALE_SAFETY_BITS: i32 = 3;

    /// Creates an adapter and automatically selects a power-of-two scale while
    /// reserving internal coordinate safety bits.
    ///
    /// Algorithms with a precise bit budget should prefer
    /// [`Self::with_coordinate_bits`].
    #[inline]
    pub fn new(rect: FloatRect<P::Scalar>) -> Self {
        let a = rect.width() * P::Scalar::HALF;
        let b = rect.height() * P::Scalar::HALF;

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);

        let max = a.max(b);

        // degenerate case
        if max == P::Scalar::ZERO {
            return Self {
                dir_scale: P::Scalar::ONE,
                inv_scale: P::Scalar::ONE,
                offset,
                rect,
                int_rect: IntRect::with_point(IntPoint::ZERO),
            };
        }

        let log2 = max.log2().to_i32();
        let safe_bits = I::BITS as i32 - Self::SCALE_SAFETY_BITS;
        let ie = safe_bits - log2;
        let e = ie as f64;

        let dir_scale = FloatNumber::from_float(libm::exp2(e));
        let inv_scale = FloatNumber::from_float(libm::exp2(-e));

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
            int_rect: Self::grid_rect(&rect, &offset, dir_scale),
        }
    }

    /// Creates an adapter whose converted coordinates have magnitude at most
    /// `2^coordinate_bits` for every point inside `rect`.
    ///
    /// The selected scale is a power of two. `coordinate_bits` may not exceed
    /// the highest non-sign bit index in `I`. This bounds converted coordinate
    /// magnitude but does not independently guarantee that every downstream
    /// product fits. `I::BITS - 3` is a conservative budget for the general
    /// point and vector operations provided by this crate.
    #[inline]
    pub fn with_coordinate_bits(rect: FloatRect<P::Scalar>, coordinate_bits: u32) -> Self {
        assert!(coordinate_bits <= I::BITS - 2);

        let a = rect.width() * P::Scalar::HALF;
        let b = rect.height() * P::Scalar::HALF;

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);
        let max = a.max(b);

        // degenerate case
        if max == P::Scalar::ZERO {
            return Self {
                dir_scale: P::Scalar::ONE,
                inv_scale: P::Scalar::ONE,
                offset,
                rect,
                int_rect: IntRect::with_point(IntPoint::ZERO),
            };
        }

        let log2 = max.log2().to_i32();
        let log2_scale = P::Scalar::from_float(libm::exp2(log2 as f64));
        let ceil_log2 = if log2_scale < max { log2 + 1 } else { log2 };
        let exponent = coordinate_bits as i32 - ceil_log2;
        let e = exponent as f64;

        let dir_scale = P::Scalar::from_float(libm::exp2(e));
        let inv_scale = P::Scalar::from_float(libm::exp2(-e));

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
            int_rect: Self::grid_rect(&rect, &offset, dir_scale),
        }
    }

    #[inline]
    pub fn with_scale(rect: FloatRect<P::Scalar>, scale: P::Scalar) -> Self {
        let a = rect.width() * P::Scalar::HALF;
        let b = rect.height() * P::Scalar::HALF;

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);

        let max = a.max(b);

        // degenerate case
        if max == P::Scalar::ZERO {
            return Self {
                dir_scale: P::Scalar::ONE,
                inv_scale: P::Scalar::ONE,
                offset,
                rect,
                int_rect: IntRect::with_point(IntPoint::ZERO),
            };
        }

        let dir_scale = scale;
        let inv_scale = P::Scalar::ONE / scale;

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
            int_rect: Self::grid_rect(&rect, &offset, dir_scale),
        }
    }

    #[inline]
    pub fn try_with_scale(
        rect: FloatRect<P::Scalar>,
        scale: P::Scalar,
    ) -> Result<Self, FloatPointAdapterScaleError> {
        let scale = Self::validate_scale(scale)?;
        let mut adapter = Self::new(rect);

        let is_degenerate =
            adapter.rect.width() == P::Scalar::ZERO && adapter.rect.height() == P::Scalar::ZERO;
        if !is_degenerate && adapter.dir_scale < scale {
            return Err(FloatPointAdapterScaleError::ScaleTooLarge);
        }

        adapter.dir_scale = scale;
        adapter.inv_scale = P::Scalar::ONE / scale;
        adapter.int_rect = Self::grid_rect(&adapter.rect, &adapter.offset, scale);
        Ok(adapter)
    }

    /// Creates an adapter with an explicit scale, rejecting scales that can
    /// produce coordinates with magnitude greater than `2^coordinate_bits`.
    #[inline]
    pub fn try_with_scale_and_coordinate_bits(
        rect: FloatRect<P::Scalar>,
        scale: P::Scalar,
        coordinate_bits: u32,
    ) -> Result<Self, FloatPointAdapterScaleError> {
        let scale = Self::validate_scale(scale)?;
        let mut adapter = Self::with_coordinate_bits(rect, coordinate_bits);

        let zero = P::Scalar::ZERO;
        let is_degenerate = adapter.rect.width() == zero && adapter.rect.height() == zero;
        if !is_degenerate && adapter.dir_scale < scale {
            return Err(FloatPointAdapterScaleError::ScaleTooLarge);
        }

        adapter.dir_scale = scale;
        adapter.inv_scale = P::Scalar::ONE / scale;
        adapter.int_rect = Self::grid_rect(&adapter.rect, &adapter.offset, scale);
        Ok(adapter)
    }

    #[inline]
    pub fn with_radius_and_scale(radius: P::Scalar, scale: P::Scalar) -> FloatPointAdapter<P, I> {
        let rect = FloatRect::new(-radius, radius, -radius, radius);
        FloatPointAdapter::with_scale(rect, scale)
    }

    #[inline]
    pub fn with_iter<'a, Q>(iter: Q) -> Self
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        Self::new(FloatRect::with_iter(iter).unwrap_or(FloatRect::zero()))
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
        Self::try_with_scale(FloatRect::with_iter(iter).unwrap_or(FloatRect::zero()), scale)
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
    fn validate_scale(scale: P::Scalar) -> Result<P::Scalar, FloatPointAdapterScaleError> {
        if !scale.is_finite() {
            return Err(FloatPointAdapterScaleError::ScaleNotFinite);
        }
        if scale <= P::Scalar::ZERO {
            return Err(FloatPointAdapterScaleError::ScaleNonPositive);
        }
        Ok(scale)
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
        if !point.is_finite() || !self.rect.contains(point) {
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
            FloatRect::new(-3.0, 3.0, -0.25, 0.25),
            10,
        );

        assert_eq!(adapter.dir_scale(), 256.0);
        assert_eq!(adapter.float_to_int(&[-3.0, -0.25]), IntPoint::new(-768, -64));
        assert_eq!(adapter.float_to_int(&[3.0, 0.25]), IntPoint::new(768, 64));
    }

    #[test]
    fn coordinate_bits_include_exact_power_of_two_boundary() {
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_coordinate_bits(
            FloatRect::new(-4.0, 4.0, -0.25, 0.25),
            10,
        );

        assert_eq!(adapter.dir_scale(), 256.0);
        assert_eq!(adapter.float_to_int(&[-4.0, 0.0]), IntPoint::new(-1024, 0));
        assert_eq!(adapter.float_to_int(&[4.0, 0.0]), IntPoint::new(1024, 0));
    }

    #[test]
    fn test_round_point_round_length() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

        assert_eq!(adapter.float_to_int(&[0.16, -0.16]), IntPoint::new(2, -2));
        assert_eq!(adapter.round_len_to_int(0.16), 2);
    }

    #[test]
    fn test_snap_to_grid() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

        assert_eq!(adapter.snap_to_grid(&[0.16, -0.16]), [0.2, -0.2]);
        assert_eq!(adapter.snap_to_grid(&[0.14, -0.14]), [0.1, -0.1]);
    }

    #[test]
    fn test_try_snap_to_grid_range() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

        assert_eq!(adapter.try_snap_to_grid(&[1.0, -1.0]).unwrap(), [1.0, -1.0]);
        assert_eq!(
            adapter.try_snap_to_grid(&[1.01, 0.0]).err().unwrap(),
            FloatPointAdapterRangeError::PointOutOfRange
        );
    }

    #[test]
    fn test_try_snap_to_grid_accepts_rounded_point_outside_float_rect() {
        let rect = FloatRect::new(-0.6, 0.6, -0.6, 0.6);
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
            FloatRect::new(f(9.375), f(10.625), f(-2.625), f(-1.375)),
            FloatRect::new(f(10.0), f(10.0), f(-2.625), f(-1.375)),
            FloatRect::new(f(9.375), f(10.625), f(-2.0), f(-2.0)),
            FloatRect::new(f(10.0), f(10.0), f(-2.0), f(-2.0)),
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
            FloatRect::new(-1.0, 1.0, -1.0, 1.0),
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
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale(FloatRect::new(-0.25, 0.25, -0.25, 0.25), 1.0)
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
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0)
                .unwrap();

        assert_eq!(adapter.dir_scale(), 10.0);
        assert_eq!(adapter.inv_scale(), 0.1);
        assert_eq!(adapter.float_to_int(&[0.16, -0.16]), IntPoint::new(2, -2));
    }

    #[test]
    fn test_try_with_scale_and_coordinate_bits() {
        let rect = FloatRect::new(-4.0, 4.0, -1.0, 1.0);
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
        let rect = FloatRect::new(-1.0, 1.0, -1.0, 1.0);

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
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::try_with_scale(FloatRect::new(1.0, 1.0, -2.0, -2.0), 1000.0)
                .unwrap();

        assert_eq!(adapter.dir_scale(), 1000.0);
        assert_eq!(adapter.float_to_int(&[1.0, -2.0]), IntPoint::ZERO);
    }

    #[test]
    fn test_try_float_to_int_range() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

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
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

        assert_eq!(
            adapter.try_float_to_int(&[f64::NAN, 0.0]).err().unwrap(),
            FloatPointAdapterRangeError::PointOutOfRange
        );
        assert_eq!(
            adapter.try_float_to_int(&[0.0, f64::INFINITY]).err().unwrap(),
            FloatPointAdapterRangeError::PointOutOfRange
        );
    }

    #[test]
    fn test_try_int_to_float_range() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

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
