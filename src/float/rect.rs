use std::fmt;
use crate::float::compatible::FloatPointCompatible;
use crate::float::number::FloatNumber;

#[derive(Clone)]
pub struct FloatRect<T: FloatNumber> {
    pub min_x: T,
    pub max_x: T,
    pub min_y: T,
    pub max_y: T,
}

impl<T: FloatNumber> FloatRect<T> {
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

    #[inline(always)]
    pub fn zero() -> Self {
        let zero = FloatNumber::from_float(0.0);
        Self { min_x: zero, max_x: zero, min_y: zero, max_y: zero }
    }

    #[inline]
    pub fn with_point<P: FloatPointCompatible<T>>(point: P) -> Self {
        Self {
            min_x: point.x(),
            max_x: point.x(),
            min_y: point.y(),
            max_y: point.y(),
        }
    }

    #[inline]
    pub fn with_points<P>(points: &[P]) -> Option<Self>
    where
        P: FloatPointCompatible<T>,
    {
        Self::with_iter(points.iter())
    }

    #[inline]
    pub fn with_iter<'a, I, P>(iter: I) -> Option<Self>
    where
        I: Iterator<Item = &'a P>,
        P: FloatPointCompatible<T> + 'a,
        T: FloatNumber,
    {
        let mut iter = iter;
        let first_point = iter.next()?;
        let mut rect = Self {
            min_x: first_point.x(),
            max_x: first_point.x(),
            min_y: first_point.y(),
            max_y: first_point.y(),
        };

        for p in iter {
            rect.unsafe_add_point(p);
        }

        Some(rect)
    }

    #[inline]
    pub fn add_point<P: FloatPointCompatible<T>>(&mut self, point: &P) {
        if self.min_x > point.x() {
            self.min_x = point.x()
        }
        if self.max_x < point.x() {
            self.max_x = point.x()
        }

        if self.min_y > point.y() {
            self.min_y = point.y()
        }
        if self.max_y < point.y() {
            self.max_y = point.y()
        }
    }

    #[inline]
    pub fn unsafe_add_point<P: FloatPointCompatible<T>>(&mut self, point: &P) {
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

    #[inline(always)]
    pub fn contains<P: FloatPointCompatible<T>>(&self, point: &P) -> bool {
        self.min_x <= point.x() && point.x() <= self.max_x && self.min_y <= point.y() && point.y() <= self.max_y
    }

    #[inline(always)]
    pub fn contains_with_radius<P: FloatPointCompatible<T>>(&self, point: &P, radius: T) -> bool {
        let min_x = self.min_x - radius;
        let max_x = self.max_x + radius;
        let min_y = self.min_y - radius;
        let max_y = self.max_y + radius;
        min_x <= point.x() && point.x() <= max_x && min_y <= point.y() && point.y() <= max_y
    }
}

impl<T: FloatNumber> fmt::Display for FloatRect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})-({}, {})", self.min_x, self.min_y, self.max_x, self.max_y)
    }
}

#[cfg(test)]
mod tests {
    use crate::float::rect::FloatRect;

    #[test]
    fn test_0() {
        let points = [
            [-2.0, -4.0],
            [-2.0, 3.0],
            [5.0, 3.0],
            [5.0, -4.0],
        ];

        let rect: FloatRect<f64> = FloatRect::with_iter(points.iter()).unwrap();

        assert_eq!((rect.max_x - 5.0).abs() < 0.000_0001, true);
        assert_eq!((rect.min_x + 2.0).abs() < 0.000_0001, true);
        assert_eq!((rect.max_y - 3.0).abs() < 0.000_0001, true);
        assert_eq!((rect.min_y + 4.0).abs() < 0.000_0001, true);
    }
}