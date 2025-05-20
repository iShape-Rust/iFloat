use core::cmp::Ordering;
use crate::fix_float::FixFloat;
use crate::fix_vec::FixVec;
use crate::int::point::IntPoint;

pub struct Triangle;

impl Triangle {
    #[inline(always)]
    pub fn area_two(p0: FixVec, p1: FixVec, p2: FixVec) -> i64 {
        (p1 - p0).cross_product(p1 - p2)
    }

    #[inline(always)]
    pub fn area(p0: FixVec, p1: FixVec, p2: FixVec) -> i64 {
        Self::area_two(p0, p1, p2) / 2
    }

    #[inline(always)]
    pub fn fix_area(p0: FixVec, p1: FixVec, p2: FixVec) -> FixFloat {
        (p1 - p0).fix_cross_product(p1 - p2) / 2
    }

    #[inline(always)]
    pub fn is_clockwise(p0: FixVec, p1: FixVec, p2: FixVec) -> bool {
        Self::area_two(p0, p1, p2) > 0
    }

    #[inline(always)]
    pub fn is_cw_or_line(p0: FixVec, p1: FixVec, p2: FixVec) -> bool {
        Self::area_two(p0, p1, p2) >= 0
    }

    #[inline(always)]
    pub fn is_not_line(p0: FixVec, p1: FixVec, p2: FixVec) -> bool {
        Self::area_two(p0, p1, p2) != 0
    }

    #[inline(always)]
    pub fn is_line(p0: FixVec, p1: FixVec, p2: FixVec) -> bool {
        Self::area_two(p0, p1, p2) == 0
    }

    #[inline(always)]
    pub fn clock_direction(p0: FixVec, p1: FixVec, p2: FixVec) -> i64 {
        Self::area_two(p0, p1, p2).signum()
    }

    #[inline]
    pub fn is_contain(p: FixVec, p0: FixVec, p1: FixVec, p2: FixVec) -> bool {
        let q0 = (p - p1).cross_product(p0 - p1);
        let q1 = (p - p2).cross_product(p1 - p2);
        let q2 = (p - p0).cross_product(p2 - p0);

        let has_neg = q0 < 0 || q1 < 0 || q2 < 0;
        let has_pos = q0 > 0 || q1 > 0 || q2 > 0;

        !(has_neg && has_pos)
    }

    #[inline]
    pub fn is_not_contain(p: FixVec, p0: FixVec, p1: FixVec, p2: FixVec) -> bool {
        let q0 = (p - p1).cross_product(p0 - p1);
        let q1 = (p - p2).cross_product(p1 - p2);
        let q2 = (p - p0).cross_product(p2 - p0);

        let has_neg = q0 <= 0 || q1 <= 0 || q2 <= 0;
        let has_pos = q0 >= 0 || q1 >= 0 || q2 >= 0;

        has_neg && has_pos
    }

    #[inline(always)]
    pub fn area_two_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> i64 {
        let x0 = p1.x as i64 - p0.x as i64;
        let y0 = p1.y as i64 - p0.y as i64;

        let x1 = p1.x as i64 - p2.x as i64;
        let y1 = p1.y as i64 - p2.y as i64;

        x0 * y1 - x1 * y0
    }

    #[inline(always)]
    pub fn is_clockwise_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        Self::area_two_point(p0, p1, p2) > 0
    }

    #[inline(always)]
    pub fn is_cw_or_line_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        Self::area_two_point(p0, p1, p2) >= 0
    }

    #[inline(always)]
    pub fn is_line_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        Self::area_two_point(p0, p1, p2) == 0
    }

    #[inline(always)]
    pub fn is_not_line_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        Self::area_two_point(p0, p1, p2) != 0
    }

    #[inline(always)]
    pub fn clock_direction_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> i64 {
        Self::area_two_point(p0, p1, p2).signum()
    }

    #[inline]
    pub fn is_contain_point(p: IntPoint, p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        let f = FixVec::new_point(p);
        let f0 = FixVec::new_point(p0);
        let f1 = FixVec::new_point(p1);
        let f2 = FixVec::new_point(p2);

        let q0 = (f - f1).cross_product(f0 - f1);
        let q1 = (f - f2).cross_product(f1 - f2);
        let q2 = (f - f0).cross_product(f2 - f0);

        let has_neg = q0 < 0 || q1 < 0 || q2 < 0;
        let has_pos = q0 > 0 || q1 > 0 || q2 > 0;

        !(has_neg && has_pos)
    }

    #[inline]
    pub fn is_contain_point_exclude_borders(p: IntPoint, p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        let f = FixVec::new_point(p);
        let f0 = FixVec::new_point(p0);
        let f1 = FixVec::new_point(p1);
        let f2 = FixVec::new_point(p2);

        let q0 = (f - f1).cross_product(f0 - f1);
        let q1 = (f - f2).cross_product(f1 - f2);
        let q2 = (f - f0).cross_product(f2 - f0);

        let has_neg = q0 < 0 || q1 < 0 || q2 < 0;
        let has_pos = q0 > 0 || q1 > 0 || q2 > 0;

        !(has_neg && has_pos) && q0 != 0 && q1 != 0 && q2 != 0
    }

    #[inline]
    pub fn is_not_contain_point(p: IntPoint, p0: IntPoint, p1: IntPoint, p2: IntPoint) -> bool {
        let f = FixVec::new_point(p);
        let f0 = FixVec::new_point(p0);
        let f1 = FixVec::new_point(p1);
        let f2 = FixVec::new_point(p2);

        let q0 = (f - f1).cross_product(f0 - f1);
        let q1 = (f - f2).cross_product(f1 - f2);
        let q2 = (f - f0).cross_product(f2 - f0);

        let has_neg = q0 <= 0 || q1 <= 0 || q2 <= 0;
        let has_pos = q0 >= 0 || q1 >= 0 || q2 >= 0;

        has_neg && has_pos
    }

    #[inline(always)]
    pub fn clock_order_point(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> Ordering {
        0.cmp(&Self::area_two_point(p0, p1, p2))
    }
}