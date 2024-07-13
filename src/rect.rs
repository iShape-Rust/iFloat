use crate::point::IntPoint;

pub struct IntRect {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl IntRect {
    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.max_x - self.min_x
    }

    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.max_y - self.min_y
    }

    #[inline(always)]
    pub fn new(min_x: i32, max_x: i32, min_y: i32, max_y: i32) -> Self {
        Self { min_x, max_x, min_y, max_y }
    }

    pub fn with_points(points: &[IntPoint]) -> Option<Self> {
        let first_point = if let Some(p) = points.first() {
            p
        } else {
            return None;
        };

        let mut rect = Self {
            min_x: first_point.x,
            max_x: first_point.x,
            min_y: first_point.y,
            max_y: first_point.y
        };

        for p in points.iter() {
            rect.unsafe_add_point(p);
        }

        Some(rect)
    }

    #[inline(always)]
    pub fn with_rects(rect0: &Self, rect1: &Self) -> Self {
        let min_x = rect0.min_x.min(rect1.min_x);
        let max_x = rect0.max_x.max(rect1.max_x);
        let min_y = rect0.min_y.min(rect1.min_y);
        let max_y = rect0.max_y.max(rect1.max_y);

        Self::new(min_x, max_x, min_y, max_y)
    }

    #[inline(always)]
    pub fn with_optional_rects(rect0: Option<Self>, rect1: Option<Self>) -> Option<Self> {
        match (rect0, rect1) {
            (Some(r0), Some(r1)) => Some(Self::with_rects(&r0, &r1)),
            (Some(r0), None) => Some(r0),
            (None, Some(r1)) => Some(r1),
            (None, None) => None,
        }
    }

    #[inline]
    pub fn add_point(&mut self, point: &IntPoint) {
        if self.min_x > point.x {
            self.min_x = point.x
        }

        if self.max_x < point.x {
            self.max_x = point.x
        }

        if self.min_y > point.y {
            self.min_y = point.y
        }

        if self.max_y < point.y {
            self.max_y = point.y
        }
    }

    #[inline]
    pub fn unsafe_add_point(&mut self, point: &IntPoint) {
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

    #[inline(always)]
    pub fn contains(&self, point: IntPoint) -> bool {
        self.min_x <= point.x && point.x <= self.max_x && self.min_y <= point.y && point.y <= self.max_y
    }
}

#[cfg(test)]
mod tests {
    use crate::point::IntPoint;
    use crate::rect::IntRect;

    #[test]
    fn test_0() {
        let rect = if let Some(rect) = IntRect::with_points(
            &vec![
                IntPoint::new(0, 0),
                IntPoint::new(-7, 10),
                IntPoint::new(20, -5),
            ]
        ) {
            rect
        } else {
            return;
        };

        assert_eq!(rect.min_x, -7);
        assert_eq!(rect.max_x, 20);
        assert_eq!(rect.min_y, -5);
        assert_eq!(rect.max_y, 10);
    }

    #[test]
    fn test_1() {
        let rect = IntRect::new(-10, 10, -10, 10);

        assert_eq!(rect.contains(IntPoint::new(-20, -20)), false);
        assert_eq!(rect.contains(IntPoint::new(-20, -10)), false);
        assert_eq!(rect.contains(IntPoint::new(-20, 0)), false);
        assert_eq!(rect.contains(IntPoint::new(-20, 10)), false);
        assert_eq!(rect.contains(IntPoint::new(-20, 20)), false);

        assert_eq!(rect.contains(IntPoint::new(-10, -20)), false);
        assert_eq!(rect.contains(IntPoint::new(-10, -10)), true);
        assert_eq!(rect.contains(IntPoint::new(-10, 0)), true);
        assert_eq!(rect.contains(IntPoint::new(-10, 10)), true);
        assert_eq!(rect.contains(IntPoint::new(-10, 20)), false);

        assert_eq!(rect.contains(IntPoint::new(0, -20)), false);
        assert_eq!(rect.contains(IntPoint::new(0, -10)), true);
        assert_eq!(rect.contains(IntPoint::new(0, 0)), true);
        assert_eq!(rect.contains(IntPoint::new(0, 10)), true);
        assert_eq!(rect.contains(IntPoint::new(0, 20)), false);

        assert_eq!(rect.contains(IntPoint::new(10, -20)), false);
        assert_eq!(rect.contains(IntPoint::new(10, -10)), true);
        assert_eq!(rect.contains(IntPoint::new(10, 0)), true);
        assert_eq!(rect.contains(IntPoint::new(10, 10)), true);
        assert_eq!(rect.contains(IntPoint::new(10, 20)), false);

        assert_eq!(rect.contains(IntPoint::new(20, -20)), false);
        assert_eq!(rect.contains(IntPoint::new(20, -10)), false);
        assert_eq!(rect.contains(IntPoint::new(20, 0)), false);
        assert_eq!(rect.contains(IntPoint::new(20, 10)), false);
        assert_eq!(rect.contains(IntPoint::new(20, 20)), false);
    }
}