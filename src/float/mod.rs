//! Floating-point geometry with a conservative coordinate range.
//!
//! Supported input coordinates and rectangle bounds are finite and satisfy:
//!
//! | Scalar | Maximum absolute coordinate |
//! | --- | --- |
//! | `f32` | `2^60` (approximately `1.15e18`) |
//! | `f64` | `2^500` (approximately `3.27e150`) |
//!
//! These inclusive bounds leave room for point differences, their dot and cross
//! products, squared lengths, and midpoints without floating-point overflow.
//! They do not guarantee exact arithmetic: rounding, cancellation, and underflow
//! still follow the scalar's floating-point behavior. Arbitrary scaling and
//! repeated operations require their own range analysis.
//!
//! Normalization additionally requires a positive, finite, normal squared
//! length (at least `f32::MIN_POSITIVE` or `f64::MIN_POSITIVE`). A nonzero vector
//! alone is insufficient: squaring tiny components can underflow.
//!
//! [`rect::FloatRect`] constructors and checked mutators validate coordinates
//! and bound ordering in every build, returning [`rect::FloatRectError`] for
//! invalid input. Public field writes and [`rect::FloatRect::unsafe_add_point`]
//! remain the caller's responsibility; the latter checks only in debug builds.
//!
//! [`point::FloatPoint::new`] and [`point::FloatPoint::from_point`] assert input
//! coordinate limits in debug builds. Both normalization implementations also
//! assert the squared-length precondition in debug builds. Arithmetic results
//! are not checked against the input limits, since point differences can be
//! larger. Coordinates are never clamped; other preconditions remain the
//! caller's responsibility. Behavior outside this contract is not guaranteed.
//! The coordinate limits do not restrict scalar constants, adapter scales, or
//! every use of [`number::FloatNumber`].

pub mod compatible;
pub mod number;

pub mod point;
pub mod rect;
pub mod vector;
