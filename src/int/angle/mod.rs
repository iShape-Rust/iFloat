//! Integer binary angles and reusable rotation matrices, without allocation.
//! Angle and rotation operations use integer arithmetic by default.
//! [`Angle::from_radians`], [`Angle::sin_cos_with_float`] and [`Angle::atan2_with_float`]
//! use floating-point arithmetic; [`Angle::between_with_float`] combines exact integer
//! products with floating-point angle evaluation.
// The private implementation file mirrors the public type name.
#[allow(clippy::module_inception)]
mod angle;
mod cordic;
mod rotation;

pub use angle::{Angle, AngleDelta};
pub use rotation::Rotation;

#[cfg(test)]
mod tests;
