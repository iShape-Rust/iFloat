use crate::f64_point::F64Point;
use crate::f64_rect::F64Rect;
use crate::point::IntPoint;

pub struct PointAdapter {
    pub dir_scale: f64,
    pub inv_scale: f64,
    pub offset: F64Point,
}

impl PointAdapter {
    pub fn new(rect: F64Rect) -> Self {
        let a = rect.width / 2.0;
        let b = rect.height / 2.0;

        let ox = rect.x + a;
        let oy = rect.y + b;

        let offset = F64Point { x: ox, y: oy };

        let max = a.max(b);
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

    pub fn convert_to_float(&self, point: &IntPoint) -> F64Point {
        let x = point.x as f64 * self.inv_scale + self.offset.x;
        let y = point.y as f64 * self.inv_scale + self.offset.y;
        F64Point { x, y }
    }

    pub fn convert_to_int(&self, point: &F64Point) -> IntPoint {
        let x = ((point.x - self.offset.x) * self.dir_scale) as i32;
        let y = ((point.y - self.offset.y) * self.dir_scale) as i32;
        IntPoint { x, y }
    }
}