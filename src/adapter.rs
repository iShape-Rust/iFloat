use crate::float::compatible::FloatPointCompatible;
use crate::float::rect::FloatRect;
use crate::float::number::FloatNumber;
use crate::int::point::IntPoint;

#[derive(Clone)]
pub struct FloatPointAdapter<T: FloatNumber> {
    pub dir_scale: T,
    pub inv_scale: T,
    pub x_offset: T,
    pub y_offset: T,
    pub rect: FloatRect<T>
}

impl<T: FloatNumber> FloatPointAdapter<T> {
    #[inline]
    pub fn new(rect: FloatRect<T>) -> Self {
        let a = rect.width() * FloatNumber::from_float(0.5);
        let b = rect.height() * FloatNumber::from_float(0.5);

        let x_offset = rect.min_x + a;
        let y_offset = rect.min_y + b;

        let max = a.max(b);

        // degenerate case
        if max == FloatNumber::from_float(0.0) {
            return Self {
                dir_scale: FloatNumber::from_float(1.0),
                inv_scale: FloatNumber::from_float(1.0),
                x_offset,
                y_offset,
                rect,
            };
        }

        let log2 = max.log2().to_int();
        let e = 29 - log2;

        let dir_scale = FloatNumber::from_float(2f64.powi(e));
        let inv_scale = FloatNumber::from_float(2f64.powi(-e));

        Self {
            dir_scale,
            inv_scale,
            x_offset,
            y_offset,
            rect,
        }
    }

    #[inline]
    pub fn with_iter<'a, I, P>(iter: I) -> Self
    where
        I: IntoIterator<Item=&'a P>,
        P: FloatPointCompatible<T> + 'a,
        T: FloatNumber,
    {
        Self::new(FloatRect::with_iter(iter).unwrap_or(FloatRect::zero()))
    }

    #[inline(always)]
    pub fn int_to_float<P>(&self, point: IntPoint) -> P
    where
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let fx: T = FloatNumber::from_i32(point.x);
        let fy: T = FloatNumber::from_i32(point.y);
        let x = fx * self.inv_scale + self.x_offset;
        let y = fy * self.inv_scale + self.y_offset;
        let float = P::from_xy(x, y);

        debug_assert!(
            self.rect.contains(float),
            "You are trying to convert a point which is out of rect: {}",
            self.rect
        );

        float
    }

    #[inline(always)]
    pub fn float_to_int<P>(&self, point: P) -> IntPoint
    where
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        debug_assert!(
            self.rect.contains(point),
            "You are trying to convert a point which is out of rect: {}",
            self.rect
        );
        let x = ((point.x() - self.x_offset) * self.dir_scale).to_int();
        let y = ((point.y() - self.y_offset) * self.dir_scale).to_int();
        IntPoint { x, y }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::FloatPointAdapter;
    use crate::float::compatible::FloatPointCompatible;
    use crate::float::rect::FloatRect;

    #[test]
    fn test_0() {
        let rect = FloatRect {
            min_x: 1.0,
            max_x: 1.0,
            min_y: -2.0,
            max_y: -2.0,
        };

        let adapter = FloatPointAdapter::new(rect);

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
        let p0 = adapter.float_to_int(f0);
        let f1: [f64; 2] = adapter.int_to_float(p0);

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
        let p0 = adapter.float_to_int(f0);
        let f1: [f64; 2] = adapter.int_to_float(p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }
}