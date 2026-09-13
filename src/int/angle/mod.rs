//! Integer binary angles and reusable rotation matrices, without allocation.
//! Angle and rotation operations use integer arithmetic; converting radians
//! with [`Angle::from_radians`] uses floating-point arithmetic.
// The private implementation file mirrors the public type name.
#[allow(clippy::module_inception)]
mod angle;
mod cordic;
mod rotation;

pub use angle::Angle;
pub use rotation::Rotation;

#[cfg(test)]
mod tests;
