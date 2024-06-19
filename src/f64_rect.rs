use crate::f64_point::F64Point;

pub struct F64Rect {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl F64Rect {

    #[inline(always)]
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    #[inline(always)]
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    #[inline(always)]
    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        Self { min_x, max_x, min_y, max_y }
    }

    #[inline]
    pub fn with_points(points: &[F64Point]) -> Self {
        if points.is_empty() {
            return Self { min_x: -f64::MAX, max_x: -f64::MAX, min_y: -f64::MAX, max_y: -f64::MAX };
        }

        let mut rect = Self { min_x: f64::MAX, max_x: -f64::MAX, min_y: f64::MAX, max_y: -f64::MAX };

        for p in points.iter() {
            rect.add_point(p);
        }

        rect
    }

    #[inline(always)]
    pub fn with_rects(rect0: &Self, rect1: &Self) -> Self {
        let min_x = rect0.min_x.min(rect1.min_x);
        let max_x = rect0.max_x.max(rect1.max_x);
        let min_y = rect0.min_y.min(rect1.min_y);
        let max_y = rect0.max_y.max(rect1.max_y);

        Self::new(min_x, max_x, min_y, max_y)
    }

    #[inline]
    pub fn add_point(&mut self, point: &F64Point) {
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