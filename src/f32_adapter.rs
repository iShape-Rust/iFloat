use crate::f32_point::F32Point;
use crate::f32_rect::F32Rect;
use crate::int::point::IntPoint;

pub struct F32PointAdapter {
    pub dir_scale: f32,
    pub inv_scale: f32,
    pub offset: F32Point,
}

impl F32PointAdapter {
    #[inline]
    pub fn new(rect: F32Rect) -> Self {
        let a = rect.width() * 0.5;
        let b = rect.height() * 0.5;

        let ox = rect.min_x + a;
        let oy = rect.min_y + b;

        let offset = F32Point { x: ox, y: oy };

        let max = a.max(b);

        // degenerate case
        if max == 0.0 {
            return F32PointAdapter {
                dir_scale: 1.0,
                inv_scale: 1.0,
                offset,
            };
        }

        let log2 = max.log2() as i32;
        let e = 29 - log2;

        let dir_scale = 2f32.powi(e);
        let inv_scale = 2f32.powi(-e);

        F32PointAdapter {
            dir_scale,
            inv_scale,
            offset,
        }
    }

    #[inline(always)]
    pub fn convert_to_float(&self, point: &IntPoint) -> F32Point {
        let x = point.x as f32 * self.inv_scale + self.offset.x;
        let y = point.y as f32 * self.inv_scale + self.offset.y;
        F32Point { x, y }
    }

    #[inline(always)]
    pub fn convert_to_int(&self, point: &F32Point) -> IntPoint {
        let x = ((point.x - self.offset.x) * self.dir_scale) as i32;
        let y = ((point.y - self.offset.y) * self.dir_scale) as i32;
        IntPoint { x, y }
    }
}

#[cfg(test)]
mod tests {
    use crate::f32_adapter::F32PointAdapter;
    use crate::f32_point::F32Point;
    use crate::f32_rect::F32Rect;

    #[test]
    fn test_0() {
        let rect = F32Rect {
            min_x: 1.0,
            max_x: 1.0,
            min_y: -2.0,
            max_y: -2.0,
        };

        let adapter = F32PointAdapter::new(rect);

        assert_eq!(adapter.dir_scale, 1.0);
        assert_eq!(adapter.inv_scale, 1.0);
    }

    #[test]
    fn test_1() {
        let rect = F32Rect {
            min_x: 0.0,
            max_x: 10.0,
            min_y: 0.0,
            max_y: 100.0,
        };

        let adapter = F32PointAdapter::new(rect);

        let f0 = F32Point::new(10.0, 2.0);
        let p0 = adapter.convert_to_int(&f0);
        let f1 = adapter.convert_to_float(&p0);

        assert_eq!((f0.x - f1.x).abs() < 0.000_0001, true);
        assert_eq!((f0.y - f1.y).abs() < 0.000_0001, true);
    }
}