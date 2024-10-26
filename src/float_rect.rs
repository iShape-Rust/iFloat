use crate::float::Float;
use crate::float_point::{FloatPoint, FloatPointCompatible};

pub struct FloatRect<T> {
    pub min_x: T,
    pub max_x: T,
    pub min_y: T,
    pub max_y: T,
}

impl<T: Float> FloatRect<T> {
    #[inline(always)]
    pub fn width(&self) -> T {
        self.max_x - self.min_x
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        self.max_y - self.min_y
    }

    #[inline(always)]
    pub fn new(min_x: T, max_x: T, min_y: T, max_y: T) -> Self {
        Self { min_x, max_x, min_y, max_y }
    }

    #[inline]
    pub fn with_points<P>(points: &[P]) -> Option<Self>
    where
        P: FloatPointCompatible<T>,
    {
        Self::with_iter(points.iter().map(|p| p.to_float_point()))
    }

    #[inline(always)]
    pub fn with_rects(rect0: &Self, rect1: &Self) -> Self {
        let min_x = rect0.min_x.min(rect1.min_x);
        let max_x = rect0.max_x.max(rect1.max_x);
        let min_y = rect0.min_y.min(rect1.min_y);
        let max_y = rect0.max_y.max(rect1.max_y);

        Self::new(min_x, max_x, min_y, max_y)
    }

    #[inline(always)]
    pub fn with_optional_rects(rect0: Option<Self>, rect1: Option<Self>) -> Option<Self> {
        match (rect0, rect1) {
            (Some(r0), Some(r1)) => Some(Self::with_rects(&r0, &r1)),
            (Some(r0), None) => Some(r0),
            (None, Some(r1)) => Some(r1),
            (None, None) => None,
        }
    }

    #[inline]
    pub fn with_iter<I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item=FloatPoint<T>>,
    {
        let mut iter = iter.into_iter();
        let first_point = iter.next()?;
        let mut rect = Self {
            min_x: first_point.x,
            max_x: first_point.x,
            min_y: first_point.y,
            max_y: first_point.y,
        };

        for p in iter {
            rect.unsafe_add_point(p);
        }

        Some(rect)
    }

    #[inline]
    pub fn add_point(&mut self, point: FloatPoint<T>) {
        if self.min_x > point.x {
            self.min_x = point.x
        }
        if self.max_x < point.x {
            self.max_x = point.x
        }

        if self.min_y > point.y {
            self.min_y = point.y
        }
        if self.max_y < point.y {
            self.max_y = point.y
        }
    }

    #[inline]
    pub fn unsafe_add_point(&mut self, point: FloatPoint<T>) {
        if self.min_x > point.x {
            self.min_x = point.x
        } else if self.max_x < point.x {
            self.max_x = point.x
        }

        if self.min_y > point.y {
            self.min_y = point.y
        } else if self.max_y < point.y {
            self.max_y = point.y
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::float_point::FloatPoint;
    use crate::float_rect::FloatRect;

    #[test]
    fn test_0() {
        let points = [
            [-2.0, -4.0],
            [-2.0, 3.0],
            [5.0, 3.0],
            [5.0, -4.0],
        ];

        let iter = points.iter().map(|s| FloatPoint::new(s[0], s[1]));

        let rect: FloatRect<f64> = FloatRect::with_iter(iter).unwrap();

        assert_eq!((rect.max_x - 5.0).abs() < 0.000_0001, true);
        assert_eq!((rect.min_x + 2.0).abs() < 0.000_0001, true);
        assert_eq!((rect.max_y - 3.0).abs() < 0.000_0001, true);
        assert_eq!((rect.min_y + 4.0).abs() < 0.000_0001, true);
    }
}