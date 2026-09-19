use crate::float::compatible::FloatPointCompatible;
use crate::float::number::FloatNumber;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatRectError {
    /// A coordinate is non-finite or exceeds the supported absolute limit.
    CoordinatesOutOfRange,
    /// A minimum bound is greater than its maximum.
    InvalidBounds,
}

impl fmt::Display for FloatRectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CoordinatesOutOfRange => {
                f.write_str("A coordinate is non-finite or exceeds the supported absolute limit")
            }
            Self::InvalidBounds => f.write_str("A minimum bound is greater than its maximum"),
        }
    }
}

impl core::error::Error for FloatRectError {}

/// Floating-point bounds under the [`crate::float`] coordinate-range contract.
/// Bounds must be finite, ordered (`min <= max`), and have absolute values at
/// most `2^60` for `f32` or `2^500` for `f64`. Constructors and checked mutators
/// validate them in every build. Public field writes and [`Self::unsafe_add_point`]
/// remain the caller's responsibility.
#[derive(Debug, Copy, Clone)]
pub struct FloatRect<T: FloatNumber> {
    pub min_x: T,
    pub max_x: T,
    pub min_y: T,
    pub max_y: T,
}

impl<T: FloatNumber> FloatRect<T> {
    /// Whether the bounds are ordered, finite, and within the inclusive
    /// [`FloatNumber::MAX_COORDINATE`] limits on both axes.
    #[inline(always)]
    pub fn is_in_safe_range(&self) -> bool {
        self.validate().is_ok()
    }

    /// Validates public bounds, including rectangles assembled through fields.
    #[inline]
    pub fn validate(&self) -> Result<(), FloatRectError> {
        if !(self.min_x.is_in_safe_range()
            && self.max_x.is_in_safe_range()
            && self.min_y.is_in_safe_range()
            && self.max_y.is_in_safe_range())
        {
            return Err(FloatRectError::CoordinatesOutOfRange);
        }
        if self.min_x > self.max_x || self.min_y > self.max_y {
            return Err(FloatRectError::InvalidBounds);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn width(&self) -> T {
        self.max_x - self.min_x
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        self.max_y - self.min_y
    }

    /// Creates bounds, rejecting out-of-range coordinates and reversed axes.
    #[inline(always)]
    pub fn new(min_x: T, max_x: T, min_y: T, max_y: T) -> Result<Self, FloatRectError> {
        let rect = Self {
            min_x,
            max_x,
            min_y,
            max_y,
        };
        rect.validate()?;
        Ok(rect)
    }

    #[inline(always)]
    pub fn zero() -> Self {
        Self {
            min_x: T::ZERO,
            max_x: T::ZERO,
            min_y: T::ZERO,
            max_y: T::ZERO,
        }
    }

    #[inline]
    pub fn with_point<P: FloatPointCompatible<Scalar = T>>(point: P) -> Result<Self, FloatRectError> {
        Self::new(point.x(), point.x(), point.y(), point.y())
    }

    /// Returns `Ok(None)` for empty input, or an error for any invalid point.
    #[inline]
    pub fn with_points<P>(points: &[P]) -> Result<Option<Self>, FloatRectError>
    where
        P: FloatPointCompatible<Scalar = T>,
    {
        Self::with_iter(points.iter())
    }

    /// Returns `Ok(None)` for empty input, or an error for any invalid point.
    #[inline]
    pub fn with_iter<'a, I, P>(iter: I) -> Result<Option<Self>, FloatRectError>
    where
        I: Iterator<Item = &'a P>,
        P: FloatPointCompatible<Scalar = T> + 'a,
        T: FloatNumber,
    {
        let mut iter = iter;
        let Some(first_point) = iter.next() else {
            return Ok(None);
        };
        let mut rect = Self::with_point(*first_point)?;

        for p in iter {
            if !p.is_in_safe_range() {
                return Err(FloatRectError::CoordinatesOutOfRange);
            }
            rect.unsafe_add_point(p);
        }

        Ok(Some(rect))
    }

    #[inline]
    pub fn with_rects(rect_0: Self, rect_1: Self) -> Result<Self, FloatRectError> {
        rect_0.validate()?;
        rect_1.validate()?;
        let min_x = rect_0.min_x.min(rect_1.min_x);
        let max_x = rect_0.max_x.max(rect_1.max_x);
        let min_y = rect_0.min_y.min(rect_1.min_y);
        let max_y = rect_0.max_y.max(rect_1.max_y);
        FloatRect::new(min_x, max_x, min_y, max_y)
    }

    #[inline]
    pub fn with_optional_rects(
        rect_0: Option<Self>,
        rect_1: Option<Self>,
    ) -> Result<Option<Self>, FloatRectError> {
        match (rect_0, rect_1) {
            (Some(r0), Some(r1)) => Self::with_rects(r0, r1).map(Some),
            (Some(rect), None) | (None, Some(rect)) => {
                rect.validate()?;
                Ok(Some(rect))
            }
            (None, None) => Ok(None),
        }
    }

    /// Expands the bounds to include a valid point. On error, leaves `self` unchanged.
    #[inline]
    pub fn add_point<P: FloatPointCompatible<Scalar = T>>(
        &mut self,
        point: &P,
    ) -> Result<(), FloatRectError> {
        self.validate()?;
        if !point.is_in_safe_range() {
            return Err(FloatRectError::CoordinatesOutOfRange);
        }
        self.unsafe_add_point(point);
        Ok(())
    }

    /// Expands each side by `offset`; negative values shrink the bounds.
    /// Returns an error without changing `self` if the resulting bounds are invalid.
    #[inline]
    pub fn add_offset(&mut self, offset: T) -> Result<(), FloatRectError> {
        self.validate()?;
        let rect = Self::new(
            self.min_x - offset,
            self.max_x + offset,
            self.min_y - offset,
            self.max_y + offset,
        )?;
        *self = rect;
        Ok(())
    }

    /// Expands valid bounds with a point known to satisfy the coordinate limits.
    /// Only debug builds check these preconditions; use [`Self::add_point`] for
    /// validation in every build.
    #[inline]
    pub fn unsafe_add_point<P: FloatPointCompatible<Scalar = T>>(&mut self, point: &P) {
        debug_assert!(self.is_in_safe_range(), "FloatRect bounds out of range");
        debug_assert!(point.is_in_safe_range(), "FloatPoint coordinates out of range");
        if self.min_x > point.x() {
            self.min_x = point.x()
        } else if self.max_x < point.x() {
            self.max_x = point.x()
        }

        if self.min_y > point.y() {
            self.min_y = point.y()
        } else if self.max_y < point.y() {
            self.max_y = point.y()
        }
    }

    #[inline]
    pub fn optional_add_point<P: FloatPointCompatible<Scalar = T>>(
        rect: &mut Option<Self>,
        point: &P,
    ) -> Result<(), FloatRectError> {
        match rect {
            Some(rect) => rect.add_point(point),
            None => {
                *rect = Some(FloatRect::with_point(*point)?);
                Ok(())
            }
        }
    }

    #[inline(always)]
    pub fn contains<P: FloatPointCompatible<Scalar = T>>(&self, point: &P) -> bool {
        self.min_x <= point.x()
            && point.x() <= self.max_x
            && self.min_y <= point.y()
            && point.y() <= self.max_y
    }

    #[inline(always)]
    pub fn contains_with_radius<P: FloatPointCompatible<Scalar = T>>(&self, point: &P, radius: T) -> bool {
        let min_x = self.min_x - radius;
        let max_x = self.max_x + radius;
        let min_y = self.min_y - radius;
        let max_y = self.max_y + radius;
        min_x <= point.x() && point.x() <= max_x && min_y <= point.y() && point.y() <= max_y
    }

    #[inline]
    pub fn is_intersect_with_padding(&self, other: &Self, padding: T) -> bool {
        let x = self.min_x < padding + other.max_x && padding + self.max_x > other.min_x;
        let y = self.min_y < padding + other.max_y && padding + self.max_y > other.min_y;
        x && y
    }
}

impl<T: FloatNumber> fmt::Display for FloatRect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {})-({}, {})",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::float::rect::FloatRect;

    #[test]
    fn test_0() {
        let points = [[-2.0, -4.0], [-2.0, 3.0], [5.0, 3.0], [5.0, -4.0]];

        let rect: FloatRect<f64> = FloatRect::with_iter(points.iter()).unwrap().unwrap();

        assert_eq!((rect.max_x - 5.0).abs() < 0.000_0001, true);
        assert_eq!((rect.min_x + 2.0).abs() < 0.000_0001, true);
        assert_eq!((rect.max_y - 3.0).abs() < 0.000_0001, true);
        assert_eq!((rect.min_y + 4.0).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_1() {
        let r0 = Some(FloatRect::new(-2.0, 2.0, -2.0, 2.0).unwrap());
        let r1 = Some(FloatRect::new(-4.0, 4.0, -4.0, 4.0).unwrap());
        let rr = FloatRect::with_optional_rects(r0, r1).unwrap().unwrap();

        assert_eq!(-4.0, rr.min_x);
        assert_eq!(-4.0, rr.min_y);
        assert_eq!(4.0, rr.max_x);
        assert_eq!(4.0, rr.max_y);
        assert!(FloatRect::with_optional_rects(r0, None).unwrap().is_some());
        assert!(FloatRect::with_optional_rects(None, r1).unwrap().is_some());
        assert!(
            FloatRect::<f32>::with_optional_rects(None, None)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn is_intersect_with_padding() {
        let rect = FloatRect::new(0.0, 10.0, 0.0, 10.0).unwrap();

        assert!(rect.is_intersect_with_padding(&FloatRect::new(11.0, 12.0, 0.0, 10.0).unwrap(), 2.0));
        assert!(rect.is_intersect_with_padding(&FloatRect::new(-3.0, -1.0, 0.0, 10.0).unwrap(), 2.0));
        assert!(rect.is_intersect_with_padding(&FloatRect::new(9.0, 11.0, 9.0, 11.0).unwrap(), 0.0));
        assert!(!rect.is_intersect_with_padding(&FloatRect::new(10.0, 12.0, 0.0, 10.0).unwrap(), 0.0));
        assert!(rect.is_intersect_with_padding(&FloatRect::new(10.5, 12.0, 0.0, 10.0).unwrap(), 1.0));
        assert!(!rect.is_intersect_with_padding(&FloatRect::new(11.0, 12.0, 0.0, 10.0).unwrap(), 1.0));
        assert!(!rect.is_intersect_with_padding(&FloatRect::new(12.0, 13.0, 0.0, 10.0).unwrap(), 1.0));
        assert!(!rect.is_intersect_with_padding(&FloatRect::new(9.0, 11.0, 11.0, 12.0).unwrap(), 1.0));
    }
}
