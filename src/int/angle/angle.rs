use super::cordic;
use crate::float::number::FloatNumber;
use crate::int::number::{int::IntNumber, uint::UIntNumber, wide_int::WideIntNumber};
use crate::int::unit_vector::UnitIntVector;
use core::f64::consts::TAU;

const ANGLE_SCALE: f64 = (1_u64 << 32) as f64;

/// Counterclockwise binary angle: one turn is `2^32` units.
///
/// This resolution is independent of an arc's minimum subdivision step.
/// Radians can be converted with [`Self::from_radians`]. Arc traversal policies
/// live outside this type.
///
/// Example of the future consumer's loop (the reusable buffer and subdivision
/// policy live outside this library):
/// ```
/// use i_float::int::{angle::{Angle, Rotation}, vector::IntVector};
/// let from = IntVector::<i32>::new(1, 0).fast_normalize().unwrap();
/// let to = IntVector::<i32>::new(0, -1).fast_normalize().unwrap();
/// let clockwise = false; // a 270-degree counterclockwise arc
/// let max_step = (1u32 << 27).clamp(1 << 22, 1 << 29);
/// let sweep = if clockwise { Angle::between(to, from) }
///             else { Angle::between(from, to) };
/// let mut directions = Vec::new(); // retain this buffer between builds
/// directions.clear();
/// if sweep.bits() != 0 {
///     // Include vectoring uncertainty before rounding the count upward.
///     let upper = sweep.bits() as u64 + Angle::MAX_ERROR as u64;
///     let mut segments = upper.div_ceil(max_step as u64);
///     // Budget the final gap too: see Rotation's error contract. For fresh
///     // i32/i64 normalized inputs, 8 binary units per step cover matrix
///     // error, component truncation, and the integer step division below.
///     while upper + 8 * segments * segments > segments * max_step as u64 {
///         segments += 1;
///     }
///     if segments > 1 {
///         let step = (sweep.bits() as u64 / segments) as u32;
///         let rotation = Rotation::new(Angle::from_bits(
///             if clockwise { step.wrapping_neg() } else { step }));
///         let mut v = from;
///         for _ in 1..segments {
///             v = rotation.apply(v);
///             directions.push(v);
///         }
///     }
/// }
/// // Endpoints are excluded; the consumer uses its original exact contacts.
/// assert!(!directions.is_empty());
/// ```
/// This example's error budget applies to fresh `i32`/`i64` normalized inputs,
/// and at most 1030 steps. It does not apply to `i16` or already heavily
/// contracted directions. Subdivision and its policy remain consumer code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Angle(u32);

impl Angle {
    /// Conservative absolute error of [`Self::between`], in binary angle units
    /// (about 4.7e-8 radians), relative to the *stored* input directions. Also applies to [`Self::atan2`].
    /// Input normalization error is additional. Exact axes have zero error.
    pub const MAX_ERROR: u32 = 32;

    #[inline]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Converts radians modulo one turn to binary angle units.
    /// Rounds to the nearest integer, with ties away from zero.
    /// Returns `None` for NaN or infinity.
    #[inline]
    pub fn from_radians<T: FloatNumber>(radians: T) -> Option<Self> {
        let radians = radians.to_f64();
        if !radians.is_finite() {
            return None;
        }

        let scaled = (radians % TAU) * (ANGLE_SCALE / TAU);
        let truncated = scaled as i64;
        let fraction = scaled - truncated as f64;
        let rounded = if fraction >= 0.5 {
            truncated + 1
        } else if fraction <= -0.5 {
            truncated - 1
        } else {
            truncated
        };

        Some(Self(rounded as u32))
    }

    #[inline]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Counterclockwise sweep from `from` to `to`, in `[0, one turn)`.
    /// Swap the arguments to obtain the clockwise sweep magnitude.
    ///
    /// Equal rays return zero regardless of their approximate lengths;
    /// opposite rays return exactly `2^31`. A nonzero cross product is never
    /// rounded to an empty arc, even arbitrarily close to a full turn.
    /// Products fit in the associated wide type, including `i128` for `i64`:
    /// components are approximately bounded by S = 2^(I::BITS-2), so products
    /// and sums/differences fit with room for float rounding near unit length.
    /// No products of cross/dot are formed.
    pub fn between<I: IntNumber>(from: UnitIntVector<I>, to: UnitIntVector<I>) -> Self {
        let (ax, ay) = (from.x().to_wide(), from.y().to_wide());
        let (bx, by) = (to.x().to_wide(), to.y().to_wide());
        let cross = ax * by - ay * bx;
        let dot = ax * bx + ay * by;
        Self::atan2(cross, dot).unwrap_or(Self::from_bits(0))
    }

    /// Floating-point alternative to [`Self::between`], using [`Self::atan2_with_float`].
    /// Returns the counterclockwise sweep in `[0, one turn)`; swap the arguments
    /// for the clockwise sweep. Equal rays return zero and opposite rays return
    /// exactly half a turn. Cross/dot products are computed in wide integer
    /// arithmetic before conversion, preserving the side of near-parallel turns.
    #[inline]
    pub fn between_with_float<I: IntNumber>(from: UnitIntVector<I>, to: UnitIntVector<I>) -> Self {
        let (ax, ay) = (from.x().to_wide(), from.y().to_wide());
        let (bx, by) = (to.x().to_wide(), to.y().to_wide());
        let cross = ax * by - ay * bx;
        let dot = ax * bx + ay * by;
        Self::atan2_with_float(cross, dot).unwrap_or(Self::from_bits(0))
    }

    /// Returns atan2(y, x) as a counterclockwise angle in [0, one turn).
    /// Every value of the built-in wide integer types is valid, including MIN.
    /// Returns None only for (0, 0); no normalization is required.
    pub fn atan2<W: WideIntNumber>(y: W, x: W) -> Option<Self> {
        if y == W::ZERO {
            return if x == W::ZERO {
                None
            } else {
                Some(Self(if x < W::ZERO { 1 << 31 } else { 0 }))
            };
        }
        if x == W::ZERO {
            return Some(Self(if y > W::ZERO { 1 << 30 } else { 3 << 30 }));
        }
        let (negative_x, negative_y) = (x < W::ZERO, y < W::ZERO);
        let (x, y) = (x.unsigned_abs(), y.unsigned_abs());
        let top = x.max(y).ilog2();
        // Preserve signs before reducing magnitudes; this conversion fits on
        // 32-bit hosts too. Unsigned abs also handles i128::MIN.
        let input_top = cordic::VECTOR_INPUT_BITS - 1;
        let reduce = |v: W::UInt| -> i64 {
            let v = if top > input_top {
                v >> (top - input_top)
            } else {
                v << (input_top - top)
            };
            (v.to_usize() as i64) << cordic::VECTOR_GUARD_BITS
        };
        let mut z = cordic::vectoring(reduce(x), reduce(y), cordic::ITERATIONS);
        if negative_x {
            z = (1 << 31) - z;
        }
        let magnitude = z.clamp(1, (1 << 31) - 1) as u32;
        Some(Self(if negative_y {
            magnitude.wrapping_neg()
        } else {
            magnitude
        }))
    }

    /// Floating-point alternative to [`Self::atan2`], with the same integer inputs
    /// and binary-angle output. Uses f64 internally, including for i128 inputs.
    /// Returns None only for (0, 0); axes are exact. For nonzero y, rounding
    /// preserves the open upper/lower half-turn, including near zero and pi.
    #[inline]
    pub fn atan2_with_float<W: WideIntNumber>(y: W, x: W) -> Option<Self> {
        if y == W::ZERO {
            return if x == W::ZERO {
                None
            } else {
                Some(Self(if x < W::ZERO { 1 << 31 } else { 0 }))
            };
        }
        if x == W::ZERO {
            return Some(Self(if y > W::ZERO { 1 << 30 } else { 3 << 30 }));
        }
        let radians = FloatNumber::atan2(y.to_f64().abs(), x.to_f64());
        let magnitude = Self::from_radians(radians)?.bits().clamp(1, (1 << 31) - 1);
        Some(Self(if y < W::ZERO {
            magnitude.wrapping_neg()
        } else {
            magnitude
        }))
    }

    /// Integer value representing one in sin/cos results (Q30).
    pub const SIN_COS_SCALE: i32 = 1 << 30;

    /// Returns (sine, cosine) in Q30, using a single CORDIC rotation.
    /// Axes are exact; the coefficient pair has length at most [`Self::SIN_COS_SCALE`].
    /// See [`super::Rotation`] for error bounds. The result uses the same scale for
    /// every coordinate type.
    #[inline]
    pub fn sin_cos(self) -> (i32, i32) {
        cordic::sin_cos(self.0, cordic::ITERATIONS, cordic::COEFFICIENT_BITS)
    }

    /// Returns (sine, cosine) in Q30, using f64 trigonometry internally.
    /// Like [`Self::sin_cos`], axes are exact and the coefficient pair has length
    /// at most [`Self::SIN_COS_SCALE`]. Components are rounded to the nearest
    /// integer, then contracted toward zero if needed to preserve that bound.
    #[inline]
    pub fn sin_cos_with_float(self) -> (i32, i32) {
        let scale = Self::SIN_COS_SCALE;
        match self.0 {
            0 => return (0, scale),
            0x4000_0000 => return (scale, 0),
            0x8000_0000 => return (0, -scale),
            0xc000_0000 => return (-scale, 0),
            _ => {}
        }
        let radians = self.0 as f64 * (core::f64::consts::TAU / 4294967296.0);
        let (sin, cos) = FloatNumber::sin_cos(radians);
        let (mut sin, mut cos) = (
            (sin * scale as f64).to_round_i32(),
            (cos * scale as f64).to_round_i32(),
        );
        // Check the rounded norm exactly. Reducing the larger component removes
        // the most excess squared length per integer unit of adjustment.
        while sin as i64 * sin as i64 + cos as i64 * cos as i64 > scale as i64 * scale as i64 {
            if sin.abs() >= cos.abs() {
                sin -= sin.signum();
            } else {
                cos -= cos.signum();
            }
        }
        (sin, cos)
    }

    /// Returns the Q30 sine. Use sin_cos when both components are needed.
    #[inline]
    pub fn sin(self) -> i32 {
        self.sin_cos().0
    }

    /// Returns the Q30 cosine. Use sin_cos when both components are needed.
    #[inline]
    pub fn cos(self) -> i32 {
        self.sin_cos().1
    }
}
