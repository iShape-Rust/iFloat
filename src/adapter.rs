use crate::f64_point::F64Point;
use crate::f64_rect::F64Rect;
use crate::point::IntPoint;

pub struct PointAdapter {
    pub dir_scale: f64,
    pub inv_scale: f64,
    pub offset: F64Point,
}

impl PointAdapter {

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
            return PointAdapter {
                dir_scale: 1.0,
                inv_scale: 1.0,
                offset,
            };
        }

        let log2 = max.log2() as i32;
        let e = 29 - log2;

        let dir_scale = 2f64.powi(e);
        let inv_scale = 2f64.powi(-e);

        PointAdapter {
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