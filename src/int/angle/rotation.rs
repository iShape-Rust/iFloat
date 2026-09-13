use super::{Angle, cordic};
use crate::int::number::{fixed_scale::FixedScale, int::IntNumber, wide_int::WideIntNumber};
use crate::int::unit_vector::UnitIntVector;

/// A reusable integer rotation matrix with coefficients stored in `I`.
/// Coefficients share the vector scale: Q14 for i16, Q30 for i32, Q62 for i64.
///
/// Construct once per arc step, then call [`Self::apply`] for each intermediate
/// direction. Coefficients have norm at most one; component truncation toward
/// zero also cannot increase length. Cardinal rotations are exact.
///
/// Each non-cardinal application can lose up to `sqrt(2)` storage units due
/// to truncation, in addition to coefficient error. Thus repeated rotations
/// accumulate error; in particular, `i16` storage is unsuitable for long fine
/// arcs. `i64` coefficients are lifted from Q30 into Q62 without gaining precision.
///
/// For [`Self::new`], coefficient error is at most `2^-28 + sqrt(2)*2^-q + 4e-13` in Euclidean
/// norm, relative to the requested angle, with q = min(I::BITS - 2, 30). Application adds at most
/// `sqrt(2)/DENOMINATOR` in component norm. For input length at least 0.99,
/// `i32`/`i64` angular error is less than 5 binary angle units per application;
/// subdivision must also account for step division and accumulated endpoint
/// error (see the example on [`Angle`]). These bounds concern directions,
/// before a consumer rounds scaled coordinates to the integer grid.
#[derive(Debug, Clone, Copy)]
pub struct Rotation<I: IntNumber> {
    sin: I,
    cos: I,
    angle: Angle,
}

impl<I: IntNumber> Rotation<I> {
    /// Builds a counterclockwise matrix. Negate the binary angle with
    /// `bits.wrapping_neg()` for clockwise rotation.
    pub fn new(angle: Angle) -> Self {
        Self::with_iterations(angle, cordic::ITERATIONS)
    }

    /// Builds a matrix with an angular error budget of `|angle| / 2^precision`.
    /// `precision` is an exponent: 3 allows 1/8 (12.5%), 4 allows 1/16 (6.25%).
    /// The magnitude is that of the shortest signed angle in [-half turn, half
    /// turn], so clockwise steps use the same precision as counterclockwise.
    /// The coefficient format is determined by `I`; this parameter changes only
    /// the number of CORDIC iterations, accounting for coefficient quantization.
    ///
    /// All exponents are valid. Larger exponents request more accuracy, up to
    /// the precision of [`Self::new`]. When the requested budget is below that
    /// floor, the constructor uses the full iteration count. A conservative error
    /// bound in binary units is `max(floor(|angle| / 2^precision), Self::MAX_ERROR)`;
    /// exponents >= 32 select full precision. Zero and cardinal angles are exact.
    ///
    /// This bounds the matrix angle, not accumulated errors from repeated
    /// application or integer coordinate rounding. Use [`Self::angle`] to
    /// calculate the number of repeated rotations and the final remainder;
    /// an approximate matrix may rotate farther than the requested angle.
    ///
    /// ```
    /// use i_float::int::angle::{Angle, Rotation};
    /// let requested = Angle::from_bits(((1u64 << 32) / 18) as u32); // ~20 degrees
    /// let rotation = Rotation::<i32>::with_precision(requested, 4); // angle / 16
    /// let achieved = rotation.angle();
    /// assert!(achieved.bits() > 0);
    /// let clockwise = Rotation::<i32>::with_precision(
    ///     Angle::from_bits(requested.bits().wrapping_neg()), 4);
    /// assert!(clockwise.angle().bits() > 1 << 31);
    /// ```
    #[inline]
    pub fn with_precision(angle: Angle, precision: u32) -> Self {
        Self::with_iterations(
            angle,
            cordic::rotation_iterations(angle.bits(), precision, Self::ANGLE_MAX_ERROR),
        )
    }

    /// Maximum difference between [`Self::angle`] and the stored matrix's actual
    /// angle, in 32-bit binary angle units, for either constructor.
    /// Cardinal rotations have zero error. Component rounding in [`Self::apply`]
    /// adds its own error, dependent on coordinate type and input length.
    /// The bound is 131072 units for i16 (about 0.011 degrees), and 2 for i32/i64.
    pub const ANGLE_MAX_ERROR: u32 = 2 << cordic::COEFFICIENT_BITS.saturating_sub(FixedScale::<I>::SHIFT);

    /// Conservative matrix-angle error of [`Self::new`], in binary angle units.
    /// Also the accuracy floor for [`Self::with_precision`]. The extra three
    /// units cover the full-iteration CORDIC residual, below 2^-28 radians.
    pub const MAX_ERROR: u32 = Self::ANGLE_MAX_ERROR + 3;

    /// Returns the achieved rotation angle, estimated from the CORDIC residual.
    /// This can differ from the requested angle, particularly with
    /// [`Self::with_precision`]. No additional atan2/CORDIC is performed here.
    /// The matrix angle is within [`Self::ANGLE_MAX_ERROR`] binary units of it.
    /// Tiny nonzero requests may report zero; check before dividing by it.
    #[inline]
    pub const fn angle(self) -> Angle {
        self.angle
    }

    #[inline]
    fn with_iterations(angle: Angle, iterations: usize) -> Self {
        let q = FixedScale::<I>::SHIFT.min(cordic::COEFFICIENT_BITS);
        let (sin, cos, bits) = cordic::rotation(angle.bits(), iterations, q);
        let coefficient = |value: i32| {
            let magnitude = I::Wide::from_u32(value.unsigned_abs()) << (FixedScale::<I>::SHIFT - q);
            I::from_wide(if value < 0 { -magnitude } else { magnitude })
        };
        Self {
            sin: coefficient(sin),
            cos: coefficient(cos),
            angle: Angle::from_bits(bits),
        }
    }

    /// Applies the matrix without CORDIC, square roots, or normalization.
    /// Preserves the length-at-most-one invariant of `UnitIntVector`.
    #[inline]
    pub fn apply(self, vector: UnitIntVector<I>) -> UnitIntVector<I> {
        let (sin, cos) = (self.sin.to_wide(), self.cos.to_wide());
        let (x, y) = (vector.x().to_wide(), vector.y().to_wide());
        let scale = FixedScale::<I>::DENOMINATOR;
        // Each product is at most scale^2; even their sum fits I::Wide.
        UnitIntVector::from_components(
            I::from_wide((cos * x - sin * y) / scale),
            I::from_wide((sin * x + cos * y) / scale),
        )
    }
}
