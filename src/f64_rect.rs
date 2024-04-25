use crate::f64_point::F64Point;

pub struct F64Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl F64Rect {
    pub fn max_x(&self) -> f64 {
        self.x + self.width
    }

    pub fn max_y(&self) -> f64 {
        self.x + self.width
    }

    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        let width = max_x - min_x;
        let height = max_y - min_y;

        Self {
            x: min_x,
            y: min_y,
            width,
            height,
        }
    }

    pub fn with_points(points: &[F64Point]) -> Self {
        if points.is_empty() {
            return Self {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }

        let mut min_x = f64::MAX;
        let mut max_x = -f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_y = -f64::MAX;

        for p in points.iter() {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }

        Self::new(min_x, max_x, min_y, max_y)
    }

    pub fn with_rects(rect0: &Self, rect1: &Self) -> Self {
        let min_x = rect0.x.min(rect1.x);
        let max_x = rect0.max_x().max(rect1.max_x());
        let min_y = rect0.y.min(rect1.y);
        let max_y = rect0.max_y().max(rect1.max_y());

        Self::new(min_x, max_x, min_y, max_y)
    }
}