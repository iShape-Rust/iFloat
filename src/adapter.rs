use crate::float::Float;
use crate::float_point::FloatPoint;
use crate::float_rect::FloatRect;
use crate::point::IntPoint;

pub struct FloatPointAdapter<T: Float> {
    pub dir_scale: T,
    pub inv_scale: T,
    pub offset: FloatPoint<T>,
}

impl<T: Float> FloatPointAdapter<T> {
    #[inline]
    pub fn new(rect: FloatRect<T>) -> Self {
        let a = rect.width() * Float::from_float(0.5);
        let b = rect.height() * Float::from_float(0.5);

        let ox = rect.min_x + a;
        let oy = rect.min_y + b;

        let offset = FloatPoint { x: ox, y: oy };

        let max = a.max(b);

        // degenerate case
        if max == Float::from_float(0.0) {
            return Self {
                dir_scale: Float::from_float(1.0),
                inv_scale: Float::from_float(1.0),
                offset,
            };
        }

        let log2 = max.log2().to_int();
        let e = 29 - log2;

        let dir_scale = Float::from_float(2f64.powi(e));
        let inv_scale = Float::from_float(2f64.powi(-e));

        Self {
            dir_scale,
            inv_scale,
            offset,
        }
    }

    #[inline]
    pub fn with_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=FloatPoint<T>>,
    {
        let rect = FloatRect::with_iter(iter)
            .unwrap_or(FloatRect::new(Float::zero(), Float::zero(), Float::zero(), Float::zero()));
        Self::new(rect)
    }

    #[inline(always)]
    pub fn convert_to_float(&self, point: &IntPoint) -> FloatPoint<T> {
        let fx: T = Float::from_i32(point.x);
        let fy: T = Float::from_i32(point.y);
        let x = fx * self.inv_scale + self.offset.x;
        let y = fy * self.inv_scale + self.offset.y;
        FloatPoint { x, y }
    }

    #[inline(always)]
    pub fn convert_to_int(&self, point: &FloatPoint<T>) -> IntPoint {
        let x = ((point.x - self.offset.x) * self.dir_scale).to_int();
        let y = ((point.y - self.offset.y) * self.dir_scale).to_int();
        IntPoint { x, y }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::FloatPointAdapter;
    use crate::float::Float;
    use crate::float_point::FloatPoint;
    use crate::float_rect::FloatRect;

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

        let f0 = FloatPoint::new(10.0, 2.0);
        let p0 = adapter.convert_to_int(&f0);
        let f1 = adapter.convert_to_float(&p0);

        assert_eq!((f0.x - f1.x).to_f64().abs() < 0.000_0001, true);
        assert_eq!((f0.y - f1.y).to_f64().abs() < 0.000_0001, true);
    }

    #[test]
    fn test_2() {
        let points = [
            [-2.0, -4.0],
            [-2.0, 3.0],
            [5.0, 3.0],
            [5.0, -4.0],
        ];

        let adapter = FloatPointAdapter::with_iter(
            points.iter().map(|s| FloatPoint::new(s[0], s[1]))
        );

        let f0 = FloatPoint::new(1.0, 2.0);
        let p0 = adapter.convert_to_int(&f0);
        let f1 = adapter.convert_to_float(&p0);

        assert_eq!((f0.x - f1.x).to_f64().abs() < 0.000_0001, true);
        assert_eq!((f0.y - f1.y).to_f64().abs() < 0.000_0001, true);
    }
}