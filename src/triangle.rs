use crate::int::number::int::IntNumber;
use crate::int::number::wide_int::WideIntNumber;
use crate::int::point::IntPoint;
use core::cmp::Ordering;

/// Integer triangle predicates.
///
/// These predicates compute cross products in [`IntNumber::Wide`] and assume
/// that the result fits. The conservative point-coordinate range documented on
/// [`IntPoint`] is sufficient for these operations. Inputs outside that range
/// require algorithm-specific overflow analysis by the caller.
pub struct Triangle;

impl Triangle {
    #[inline(always)]
    pub fn area_two<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> T::Wide {
        (p1 - p0).cross_product(p2 - p0)
    }

    #[inline(always)]
    pub fn is_clockwise<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> bool {
        Self::area_two(p0, p1, p2) < T::Wide::ZERO
    }

    #[inline(always)]
    pub fn is_cw_or_line<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> bool {
        Self::area_two(p0, p1, p2) <= T::Wide::ZERO
    }

    #[inline(always)]
    pub fn is_not_line<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> bool {
        Self::area_two(p0, p1, p2) != T::Wide::ZERO
    }

    #[inline(always)]
    pub fn is_line<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> bool {
        Self::area_two(p0, p1, p2) == T::Wide::ZERO
    }

    #[inline(always)]
    pub fn clock_direction<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> T::Wide {
        Self::area_two(p0, p2, p1).signum()
    }

    /// Tests whether the point is inside the triangle, including its border.
    ///
    /// # Preconditions
    /// The triangle must be non-degenerate: `area_two(p0, p1, p2) != 0`.
    /// Collinear or coincident vertices are unsupported and may produce
    /// incorrect containment results. Use [`Self::is_not_line`] to check first.
    #[inline]
    pub fn is_contain<T: IntNumber>(
        p: IntPoint<T>,
        p0: IntPoint<T>,
        p1: IntPoint<T>,
        p2: IntPoint<T>,
    ) -> bool {
        let q0 = (p - p1).cross_product(p0 - p1);
        let q1 = (p - p2).cross_product(p1 - p2);
        let q2 = (p - p0).cross_product(p2 - p0);

        let has_neg = q0 < T::Wide::ZERO || q1 < T::Wide::ZERO || q2 < T::Wide::ZERO;
        let has_pos = q0 > T::Wide::ZERO || q1 > T::Wide::ZERO || q2 > T::Wide::ZERO;

        !(has_neg && has_pos)
    }

    /// Tests whether the point is outside the triangle's strict interior,
    /// including points on its border.
    ///
    /// # Preconditions
    /// The triangle must be non-degenerate. Collinear or coincident vertices
    /// are unsupported; see [`Self::is_contain`] for the containment contract.
    #[inline]
    pub fn is_not_contain<T: IntNumber>(
        p: IntPoint<T>,
        p0: IntPoint<T>,
        p1: IntPoint<T>,
        p2: IntPoint<T>,
    ) -> bool {
        let q0 = (p - p1).cross_product(p0 - p1);
        let q1 = (p - p2).cross_product(p1 - p2);
        let q2 = (p - p0).cross_product(p2 - p0);

        let has_neg = q0 <= T::Wide::ZERO || q1 <= T::Wide::ZERO || q2 <= T::Wide::ZERO;
        let has_pos = q0 >= T::Wide::ZERO || q1 >= T::Wide::ZERO || q2 >= T::Wide::ZERO;

        has_neg && has_pos
    }
    /// Tests whether the point is strictly inside the triangle, excluding its border.
    ///
    /// # Preconditions
    /// The triangle must be non-degenerate. Collinear or coincident vertices
    /// are unsupported; see [`Self::is_contain`] for the containment contract.
    #[inline]
    pub fn is_contain_exclude_borders<T: IntNumber>(
        p: IntPoint<T>,
        p0: IntPoint<T>,
        p1: IntPoint<T>,
        p2: IntPoint<T>,
    ) -> bool {
        let q0 = (p - p1).cross_product(p0 - p1);
        let q1 = (p - p2).cross_product(p1 - p2);
        let q2 = (p - p0).cross_product(p2 - p0);

        let has_neg = q0 < T::Wide::ZERO || q1 < T::Wide::ZERO || q2 < T::Wide::ZERO;
        let has_pos = q0 > T::Wide::ZERO || q1 > T::Wide::ZERO || q2 > T::Wide::ZERO;

        !(has_neg && has_pos) && q0 != T::Wide::ZERO && q1 != T::Wide::ZERO && q2 != T::Wide::ZERO
    }

    #[inline(always)]
    pub fn clock_order<T: IntNumber>(p0: IntPoint<T>, p1: IntPoint<T>, p2: IntPoint<T>) -> Ordering {
        Self::area_two(p0, p1, p2).cmp(&T::Wide::ZERO)
    }
}
