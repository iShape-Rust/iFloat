use crate::f64_point::F64Point;
use crate::f64_rect::F64Rect;
use crate::int::point::IntPoint;

pub struct F64PointAdapter {
    pub dir_scale: f64,
    pub inv_scale: f64,
    pub offset: F64Point,
}

impl F64PointAdapter {
    #[inline]
    pub fn new(rect: F64Rect) -> Self {
        let a = rect.width() * 0.5;
        let b = rect.height() * 0.5;

        let ox = rect.min_x + a;
        let oy = rect.min_y + b;

        let offset = F64Point { x: ox, y: oy };

        let max = a.max(b);

        // degenerate case
        if max == 0.0 {
            return F64PointAdapter {
                dir_scale: 1.0,
                inv_scale: 1.0,
                offset,
            };
        }

        let log2 = max.log2() as i32;
        let e = 29 - log2;

        let dir_scale = 2f64.powi(e);
        let inv_scale = 2f64.powi(-e);

        F64PointAdapter {
            dir_scale,
            inv_scale,
            offset,
        }
    }

    #[inline(always)]
    pub fn convert_to_float(&self, point: &IntPoint) -> F64Point {
        let x = point.x as f64 * self.inv_scale + self.offset.x;
        let y = point.y as f64 * self.inv_scale + self.offset.y;
        F64Point { x, y }
    }

    #[inline(always)]
    pub fn convert_to_int(&self, point: &F64Point) -> IntPoint {
        let x = ((point.x - self.offset.x) * self.dir_scale) as i32;
        let y = ((point.y - self.offset.y) * self.dir_scale) as i32;
        IntPoint { x, y }
    }
}

#[cfg(test)]
mod tests {
    use crate::f64_adapter::F64PointAdapter;
    use crate::f64_point::F64Point;
    use crate::f64_rect::F64Rect;

    #[test]
    fn test_0() {
        let rect = F64Rect {
            min_x: 1.0,
            max_x: 1.0,
            min_y: -2.0,
            max_y: -2.0,
        };

        let adapter = F64PointAdapter::new(rect);

        assert_eq!(adapter.dir_scale, 1.0);
        assert_eq!(adapter.inv_scale, 1.0);
    }

    #[test]
    fn test_1() {
        let rect = F64Rect {
            min_x: 0.0,
            max_x: 10.0,
            min_y: 0.0,
            max_y: 100.0,
        };

        let adapter = F64PointAdapter::new(rect);

        let f0 = F64Point::new(10.0, 2.0);
        let p0 = adapter.convert_to_int(&f0);
        let f1 = adapter.convert_to_float(&p0);

        assert_eq!((f0.x - f1.x).abs() < 0.000_0001, true);
        assert_eq!((f0.y - f1.y).abs() < 0.000_0001, true);
    }
}