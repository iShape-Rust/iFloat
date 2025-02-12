use crate::float::compatible::FloatPointCompatible;
use crate::float::rect::FloatRect;
use crate::float::number::FloatNumber;
use crate::int::point::IntPoint;

#[derive(Clone)]
pub struct FloatPointAdapter<P: FloatPointCompatible<T>, T: FloatNumber> {
    pub dir_scale: T,
    pub inv_scale: T,
    pub offset: P,
    pub rect: FloatRect<T>
}

impl<P: FloatPointCompatible<T>, T: FloatNumber> FloatPointAdapter<P, T> {
    #[inline]
    pub fn new(rect: FloatRect<T>) -> Self {
        let a = rect.width() * FloatNumber::from_float(0.5);
        let b = rect.height() * FloatNumber::from_float(0.5);

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);

        let max = a.max(b);

        // degenerate case
        if max == FloatNumber::from_float(0.0) {
            return Self {
                dir_scale: FloatNumber::from_float(1.0),
                inv_scale: FloatNumber::from_float(1.0),
                offset,
                rect,
            };
        }

        let log2 = max.log2().to_i32();
        let e = 29 - log2;

        let dir_scale = FloatNumber::from_float(2f64.powi(e));
        let inv_scale = FloatNumber::from_float(2f64.powi(-e));

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
        }
    }

    #[inline]
    pub fn with_scale(rect: FloatRect<T>, scale: f64) -> Self {
        let a = rect.width() * FloatNumber::from_float(0.5);
        let b = rect.height() * FloatNumber::from_float(0.5);

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);

        let max = a.max(b);

        // degenerate case
        if max == FloatNumber::from_float(0.0) {
            return Self {
                dir_scale: FloatNumber::from_float(1.0),
                inv_scale: FloatNumber::from_float(1.0),
                offset,
                rect,
            };
        }

        let dir_scale = FloatNumber::from_float(scale);
        let inv_scale = FloatNumber::from_float(1.0 / scale);

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
        }
    }

    #[inline]
    pub fn with_iter<'a, I>(iter: I) -> Self
    where
        I: Iterator<Item=&'a P>,
        T: FloatNumber, P: 'a
    {
        Self::new(FloatRect::with_iter(iter).unwrap_or(FloatRect::zero()))
    }

    #[inline(always)]
    pub fn int_to_float(&self, point: &IntPoint) -> P {
        let fx: T = FloatNumber::from_i32(point.x);
        let fy: T = FloatNumber::from_i32(point.y);
        let x = fx * self.inv_scale + self.offset.x();
        let y = fy * self.inv_scale + self.offset.y();
        let float = P::from_xy(x, y);

        if cfg!(debug_assertions) {
            let radius = self.rect.height().max(self.rect.width()) * T::from_float(0.01);
            if !self.rect.contains_with_radius(&float, radius) {
                panic!(
                    "You are trying to convert a point[{}, {}] which is out of rect: {}",
                    x, y, self.rect
                );
            }
        }

        float
    }

    #[inline(always)]
    pub fn float_to_int(&self, point: &P) -> IntPoint {
        if cfg!(debug_assertions) {
            let radius = self.rect.height().max(self.rect.width()) * T::from_float(0.01);
            if !self.rect.contains_with_radius(point, radius) {
                panic!(
                    "You are trying to convert a point[{}, {}] which is out of rect: {}",
                    point.x(), point.y(), self.rect
                );
            }
        }
        let x = ((point.x() - self.offset.x()) * self.dir_scale).to_i32();
        let y = ((point.y() - self.offset.y()) * self.dir_scale).to_i32();
        IntPoint { x, y }
    }

    #[inline(always)]
    pub fn sqr_float_to_int(&self, value: T) -> usize {
        let scale = self.dir_scale;
        let sqr_scale = scale * scale;
        (sqr_scale * value).to_f64() as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::FloatPointAdapter;
    use crate::float::compatible::FloatPointCompatible;
    use crate::float::point::FloatPoint;
    use crate::float::rect::FloatRect;

    #[test]
    fn test_0() {
        let rect = FloatRect {
            min_x: 1.0,
            max_x: 1.0,
            min_y: -2.0,
            max_y: -2.0,
        };

        let adapter = FloatPointAdapter::<FloatPoint<f64>, f64>::new(rect);

        assert_eq!(adapter.dir_scale, 1.0);
        assert_eq!(adapter.inv_scale, 1.0);
    }

    #[test]
    fn test_1() {
        let rect = FloatRect {
            min_x: 0.0,
            max_x: 10.0,
            min_y: 0.0,
            max_y: 100.0,
        };

        let adapter = FloatPointAdapter::new(rect);

        let f0 = [10.0, 2.0];
        let p0 = adapter.float_to_int(&f0);
        let f1: [f64; 2] = adapter.int_to_float(&p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_2() {
        let points = [
            [-2.0, -4.0],
            [-2.0, 3.0],
            [5.0, 3.0],
            [5.0, -4.0],
        ];

        let adapter = FloatPointAdapter::with_iter(points.iter());

        let f0 = [1.0, 2.0];
        let p0 = adapter.float_to_int(&f0);
        let f1: [f64; 2] = adapter.int_to_float(&p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }
}