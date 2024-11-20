use glam::IVec2;

use crate::int::point::IntPoint;

impl From<IntPoint> for glam::IVec2 {
    #[inline(always)]
    fn from(point: IntPoint) -> Self {
        IVec2::new(point.x, point.y)
    }
}

impl From<glam::IVec2> for IntPoint {
    #[inline(always)]
    fn from(point: IVec2) -> Self {
        IntPoint::new(point.x, point.y)
    }
}