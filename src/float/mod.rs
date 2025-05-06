#[cfg(any(feature = "float_pt", feature = "core"))]
pub mod number;
#[cfg(any(feature = "float_pt", feature = "core"))]
pub mod compatible;

pub mod point;
pub mod rect;
pub mod vector;