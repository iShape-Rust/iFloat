use crate::float::point::FloatPoint;

use super::FloatPointCompatible;


impl FloatPointCompatible<f32> for glam::Vec2 {
    #[inline(always)]
    fn from_xy(x: f32, y: f32) -> Self {
        glam::Vec2::new(x, y)
    }

    #[inline(always)]
    fn x(&self) -> f32 {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> f32 {
        self.y
    }
}

impl From<FloatPoint<f32>> for glam::Vec2 {
    #[inline(always)]
    fn from(point: FloatPoint<f32>) -> Self {
        glam::Vec2::new(point.x, point.y)
    }
}

impl From<glam::Vec2> for FloatPoint<f32> {
    #[inline(always)]
    fn from(point: glam::Vec2) -> Self {
        FloatPoint::new(point.x, point.y)
    }
}

impl FloatPointCompatible<f64> for glam::DVec2 {
    #[inline(always)]
    fn from_xy(x: f64, y: f64) -> Self {
        glam::DVec2::new(x, y)
    }

    #[inline(always)]
    fn x(&self) -> f64 {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> f64 {
        self.y
    }
}

impl From<FloatPoint<f64>> for glam::DVec2 {
    #[inline(always)]
    fn from(point: FloatPoint<f64>) -> Self {
        glam::DVec2::new(point.x, point.y)
    }
}

impl From<glam::DVec2> for FloatPoint<f64> {
    #[inline(always)]
    fn from(point: glam::DVec2) -> Self {
        FloatPoint::new(point.x, point.y)
    }
}